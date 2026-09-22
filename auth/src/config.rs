use std::{env, net::IpAddr};
use url::{Host, Url};
use uuid::Uuid;

pub const CALLBACK_PATH: &str = "/api/auth/callback";
const DEFAULT_POST_LOGIN_REDIRECT_PATH: &str = "/api/auth/me";
pub const LOGIN_FLOW_TTL_SECS: i64 = 300;
const MAX_SESSION_TTL_SECS: i64 = 604_800;

// Deliberately omit Debug: configuration contains the client secret.
pub struct AuthConfig {
    pub tenant_id: Uuid,
    pub client_id: Uuid,
    pub client_secret: String,
    pub redirect_uri: Url,
    pub post_login_redirect_path: String,
    pub post_logout_redirect_uri: Option<Url>,
    pub session_ttl_secs: i64,
    pub session_idle_ttl_secs: i64,
}

impl AuthConfig {
    pub fn from_env() -> Result<Self, String> {
        let config = Self {
            tenant_id: required("ENTRA_TENANT_ID")?
                .parse()
                .map_err(|_| "ENTRA_TENANT_ID debe ser un UUID")?,
            client_id: required("ENTRA_CLIENT_ID")?
                .parse()
                .map_err(|_| "ENTRA_CLIENT_ID debe ser un UUID")?,
            client_secret: required("ENTRA_CLIENT_SECRET")?,
            redirect_uri: Url::parse(&required("OIDC_REDIRECT_URI")?)
                .map_err(|_| "OIDC_REDIRECT_URI debe ser una URL absoluta válida")?,
            post_login_redirect_path: match env::var("POST_LOGIN_REDIRECT_PATH") {
                Ok(path) => path,
                Err(env::VarError::NotPresent) => DEFAULT_POST_LOGIN_REDIRECT_PATH.into(),
                Err(_) => return Err("POST_LOGIN_REDIRECT_PATH inválida".into()),
            },
            post_logout_redirect_uri: match env::var("POST_LOGOUT_REDIRECT_URI") {
                Ok(uri) => Some(
                    Url::parse(&uri)
                        .map_err(|_| "POST_LOGOUT_REDIRECT_URI debe ser una URL absoluta válida")?,
                ),
                Err(env::VarError::NotPresent) => None,
                Err(_) => return Err("POST_LOGOUT_REDIRECT_URI inválida".into()),
            },
            session_ttl_secs: ttl("SESSION_TTL_SECS", 28_800)?,
            session_idle_ttl_secs: ttl("SESSION_IDLE_TTL_SECS", 1_800)?,
        };
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.tenant_id.is_nil() || self.client_id.is_nil() {
            return Err("Los UUID del tenant y de la aplicación no pueden ser nil".into());
        }
        if self.client_secret.trim().is_empty() {
            return Err("ENTRA_CLIENT_SECRET no puede estar vacío".into());
        }
        for (name, seconds) in [
            ("SESSION_TTL_SECS", self.session_ttl_secs),
            ("SESSION_IDLE_TTL_SECS", self.session_idle_ttl_secs),
        ] {
            if !(1..=MAX_SESSION_TTL_SECS).contains(&seconds) {
                return Err(format!("{name} debe estar entre 1 y 604800 segundos"));
            }
        }
        let redirect = &self.redirect_uri;
        if redirect.host().is_none()
            || !redirect.username().is_empty()
            || redirect.password().is_some()
            || redirect.path() != CALLBACK_PATH
            || redirect.query().is_some()
            || redirect.fragment().is_some()
        {
            return Err(format!(
                "OIDC_REDIRECT_URI debe tener host y ruta {CALLBACK_PATH}, sin credenciales, query ni fragmento"
            ));
        }
        let loopback = match redirect.host() {
            Some(Host::Domain("localhost")) => true,
            Some(Host::Ipv4(ip)) => IpAddr::V4(ip).is_loopback(),
            Some(Host::Ipv6(ip)) => IpAddr::V6(ip).is_loopback(),
            _ => false,
        };
        if redirect.scheme() != "https" && !(redirect.scheme() == "http" && loopback) {
            return Err("El callback requiere HTTPS, salvo HTTP local para desarrollo".into());
        }
        let path = self.post_login_redirect_path.as_bytes();
        if !path.starts_with(b"/")
            || path.starts_with(b"//")
            || !path
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || b"/-._~".contains(byte))
        {
            return Err("POST_LOGIN_REDIRECT_PATH debe ser una ruta local absoluta sin query ni fragmento, por ejemplo /app".into());
        }
        if let Some(uri) = &self.post_logout_redirect_uri
            && (uri.origin() != redirect.origin()
                || !uri.username().is_empty()
                || uri.password().is_some()
                || uri.query().is_some()
                || uri.fragment().is_some())
        {
            return Err("POST_LOGOUT_REDIRECT_URI debe usar el mismo origen que OIDC_REDIRECT_URI, sin credenciales, query ni fragmento".into());
        }
        Ok(())
    }

    pub fn issuer(&self) -> Url {
        Url::parse(&format!(
            "https://login.microsoftonline.com/{}/v2.0",
            self.tenant_id
        ))
        .expect("UUID interpolated into a fixed HTTPS issuer URL")
    }

    pub fn public_origin(&self) -> String {
        self.redirect_uri.origin().ascii_serialization()
    }
}

fn required(name: &str) -> Result<String, String> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty() && !value.starts_with("REEMPLAZAR_"))
        .ok_or_else(|| format!("Configura {name}"))
}

fn ttl(name: &str, default: i64) -> Result<i64, String> {
    match env::var(name) {
        Ok(value) => value
            .parse()
            .map_err(|_| format!("{name} debe ser un entero")),
        Err(env::VarError::NotPresent) => Ok(default),
        Err(_) => Err(format!("{name} inválido")),
    }
}

#[cfg(test)]
mod tests {
    use super::AuthConfig;
    use url::Url;
    use uuid::Uuid;

    #[test]
    fn post_logout_redirect_must_use_the_public_origin() {
        let mut config = AuthConfig {
            tenant_id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            client_secret: "test-secret".into(),
            redirect_uri: Url::parse("http://localhost:3000/api/auth/callback").unwrap(),
            post_login_redirect_path: "/app".into(),
            post_logout_redirect_uri: Some(Url::parse("https://other.example/signed-out").unwrap()),
            session_ttl_secs: 28_800,
            session_idle_ttl_secs: 1_800,
        };
        assert!(config.validate().is_err());

        config.post_logout_redirect_uri =
            Some(Url::parse("http://localhost:3000/signed-out").unwrap());
        assert!(config.validate().is_ok());
    }
}
