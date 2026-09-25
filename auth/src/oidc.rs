use openidconnect::{core::*, *};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::{AuthConfig, AuthError};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EntraClaims {
    tid: Uuid,
    oid: Uuid,
    #[serde(default)]
    nbf: Option<i64>,
}
impl AdditionalClaims for EntraClaims {}

type EntraResponse = StandardTokenResponse<
    IdTokenFields<
        EntraClaims,
        EmptyExtraTokenFields,
        CoreGenderClaim,
        CoreJweContentEncryptionAlgorithm,
        CoreJwsSigningAlgorithm,
    >,
    CoreTokenType,
>;
type EntraClient<A = EndpointNotSet, T = EndpointNotSet, U = EndpointNotSet> = Client<
    EntraClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    EntraResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    A,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    T,
    U,
>;

pub(crate) struct OidcProvider {
    client: EntraClient<EndpointSet, EndpointMaybeSet, EndpointMaybeSet>,
    http: reqwest::Client,
    tenant_id: Uuid,
    end_session_endpoint: EndSessionUrl,
    post_logout_redirect_uri: Option<PostLogoutRedirectUrl>,
}

pub(crate) struct VerifiedIdentity {
    pub issuer: String,
    pub subject: String,
    pub tenant_id: Uuid,
    pub object_id: Uuid,
    pub email: Option<String>,
    pub full_name: Option<String>,
}

fn display_name(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| {
            !value.is_empty()
                && value.chars().count() <= 150
                && !value.chars().any(char::is_control)
        })
        .map(str::to_owned)
}

fn display_email(value: Option<&str>) -> Option<String> {
    value
        .filter(|value| {
            !value.is_empty()
                && value.chars().count() <= 254
                && !value
                    .chars()
                    .any(|character| character.is_whitespace() || character.is_control())
        })
        .map(str::to_owned)
}

impl OidcProvider {
    pub async fn discover(config: &AuthConfig) -> Result<Self, AuthError> {
        let http = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|_| AuthError::Internal)?;
        let metadata = ProviderMetadataWithLogout::discover_async(
            IssuerUrl::new(config.issuer().to_string()).map_err(|_| AuthError::Internal)?,
            &http,
        )
        .await
        .map_err(|_| AuthError::Provider)?;
        let end_session_endpoint = metadata
            .additional_metadata()
            .end_session_endpoint
            .clone()
            .ok_or(AuthError::Provider)?;
        let post_logout_redirect_uri = config
            .post_logout_redirect_uri
            .as_ref()
            .map(|uri| PostLogoutRedirectUrl::new(uri.to_string()))
            .transpose()
            .map_err(|_| AuthError::Internal)?;
        let client = EntraClient::from_provider_metadata(
            metadata,
            ClientId::new(config.client_id.to_string()),
            Some(ClientSecret::new(config.client_secret.clone())),
        )
        .set_redirect_uri(
            RedirectUrl::new(config.redirect_uri.to_string()).map_err(|_| AuthError::Internal)?,
        );
        Ok(Self {
            client,
            http,
            tenant_id: config.tenant_id,
            end_session_endpoint,
            post_logout_redirect_uri,
        })
    }

    pub fn authorize(
        &self,
        challenge: PkceCodeChallenge,
        prompt: Option<CoreAuthPrompt>,
    ) -> (url::Url, CsrfToken, Nonce) {
        let mut request = self
            .client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("profile".into()))
            .add_scope(Scope::new("email".into()))
            .set_pkce_challenge(challenge);
        if let Some(prompt) = prompt {
            request = request.add_prompt(prompt);
        }
        request.url()
    }

    pub fn logout_url(&self) -> url::Url {
        let mut request = LogoutRequest::from(self.end_session_endpoint.clone());
        if let Some(uri) = &self.post_logout_redirect_uri {
            request = request.set_post_logout_redirect_uri(uri.clone());
        }
        request.http_get_url()
    }

    pub async fn exchange(
        &self,
        code: String,
        verifier: PkceCodeVerifier,
        nonce: Nonce,
    ) -> Result<VerifiedIdentity, AuthError> {
        let response = self
            .client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|_| AuthError::Internal)?
            .set_pkce_verifier(verifier)
            .request_async(&self.http)
            .await
            .map_err(|_| AuthError::Provider)?;
        let token = response.id_token().ok_or(AuthError::Provider)?;
        let verifier = self.client.id_token_verifier();
        let claims = token
            .claims(&verifier, &nonce)
            .map_err(|_| AuthError::Provider)?;
        let extra = claims.additional_claims();
        if extra.tid != self.tenant_id
            || extra.oid.is_nil()
            || extra
                .nbf
                .is_some_and(|nbf| nbf > chrono::Utc::now().timestamp())
        {
            return Err(AuthError::Forbidden);
        }
        if let Some(expected) = claims.access_token_hash() {
            let actual = AccessTokenHash::from_token(
                response.access_token(),
                token.signing_alg().map_err(|_| AuthError::Provider)?,
                token
                    .signing_key(&verifier)
                    .map_err(|_| AuthError::Provider)?,
            )
            .map_err(|_| AuthError::Provider)?;
            if actual != *expected {
                return Err(AuthError::Provider);
            }
        }
        let subject = claims.subject().as_str();
        let issuer = claims.issuer().as_str();
        if subject.is_empty() || subject.chars().count() > 255 || issuer.chars().count() > 512 {
            return Err(AuthError::Provider);
        }
        let full_name = display_name(
            claims
                .name()
                .and_then(|name| name.get(None))
                .map(|value| value.as_str()),
        );
        let email = display_email(claims.email().map(|value| value.as_str()));
        // Return only verified identity. ID/access/refresh tokens are dropped here.
        Ok(VerifiedIdentity {
            issuer: issuer.into(),
            subject: subject.into(),
            tenant_id: extra.tid,
            object_id: extra.oid,
            email,
            full_name,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{EntraClaims, display_email};

    #[test]
    fn identity_claims_do_not_require_or_interpret_app_roles() {
        let mut value = serde_json::json!({
            "tid": "00000000-0000-0000-0000-000000000001",
            "oid": "00000000-0000-0000-0000-000000000002"
        });
        assert!(serde_json::from_value::<EntraClaims>(value.clone()).is_ok());
        value["roles"] = serde_json::json!(["admin", "unknown"]);
        assert!(serde_json::from_value::<EntraClaims>(value).is_ok());
    }

    #[test]
    fn email_claim_is_optional_and_must_fit_the_users_column() {
        assert_eq!(display_email(None), None);
        assert_eq!(
            display_email(Some("person@example.com")),
            Some("person@example.com".into())
        );
        assert_eq!(display_email(Some(" person@example.com")), None);
        assert_eq!(display_email(Some("person\n@example.com")), None);
        assert_eq!(display_email(Some(&"x".repeat(255))), None);
    }
}
