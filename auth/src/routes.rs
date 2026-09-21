use axum::{
    Json,
    extract::{FromRef, Query, State, rejection::QueryRejection},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use openidconnect::{CsrfToken, PkceCodeChallenge};
use serde::Deserialize;
use utoipa::IntoParams;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{
    AuthError, AuthErrorResponse, AuthState, AuthUser, Principal, SameOrigin, SessionToken,
    state::LoginFlow,
};

pub fn router<S>() -> OpenApiRouter<S>
where
    S: Clone + Send + Sync + 'static,
    AuthState: FromRef<S>,
{
    OpenApiRouter::new()
        .routes(routes!(login))
        .routes(routes!(callback))
        .routes(routes!(me))
        .routes(routes!(logout))
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

#[utoipa::path(get, path = "/auth/login", tag = "auth", operation_id = "login", responses(
    (status = 303, description = "Redirección a Microsoft Entra ID"),
    (status = 503, body = AuthErrorResponse)))]
async fn login(State(state): State<AuthState>, jar: CookieJar) -> Result<Response, AuthError> {
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let (url, csrf, nonce) = state.0.provider.authorize(challenge);
    let token = SessionToken::generate().map_err(|_| AuthError::Internal)?;
    let previous = jar
        .get(state.0.cookies.flow_name())
        .and_then(|cookie| SessionToken::hash_cookie(cookie.value()));
    state.insert_flow(
        *token.hash(),
        previous,
        LoginFlow {
            state: csrf,
            nonce,
            verifier,
            started: std::time::Instant::now(),
        },
    )?;
    Ok((
        [
            (header::CACHE_CONTROL, "no-store"),
            (header::REFERRER_POLICY, "no-referrer"),
        ],
        jar.add(state.0.cookies.login_flow(token.value().to_owned())),
        Redirect::to(url.as_str()),
    )
        .into_response())
}

#[utoipa::path(get, path = "/auth/callback", tag = "auth", operation_id = "oidc_callback", params(Callback), responses(
    (status = 303, description = "Sesión local creada; redirige a la ruta local configurada"),
    (status = 400, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse),
    (status = 502, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)))]
async fn callback(
    State(state): State<AuthState>,
    jar: CookieJar,
    params: Result<Query<Callback>, QueryRejection>,
) -> Response {
    let result = complete_login(&state, &jar, params).await;
    let jar = jar.add(state.0.cookies.clear_login_flow());
    match result {
        Ok(token) => (
            [
                (header::CACHE_CONTROL, "no-store"),
                (header::REFERRER_POLICY, "no-referrer"),
            ],
            jar.add(state.0.cookies.session(token.value().to_owned())),
            Redirect::to(&state.0.post_login_redirect_path),
        )
            .into_response(),
        Err(error) => (jar, error).into_response(),
    }
}

async fn complete_login(
    state: &AuthState,
    jar: &CookieJar,
    params: Result<Query<Callback>, QueryRejection>,
) -> Result<SessionToken, AuthError> {
    let hash = jar
        .get(state.0.cookies.flow_name())
        .and_then(|cookie| SessionToken::hash_cookie(cookie.value()))
        .ok_or(AuthError::InvalidLogin)?;
    let flow = state.take_flow(hash)?;
    let Query(params) = params.map_err(|_| AuthError::InvalidLogin)?;
    let returned_state = params.state.ok_or(AuthError::InvalidLogin)?;
    if returned_state.len() > 1024
        || flow.state != CsrfToken::new(returned_state)
        || params.error.is_some()
    {
        return Err(AuthError::InvalidLogin);
    }
    let code = params
        .code
        .filter(|code| !code.is_empty() && code.len() <= 16_384)
        .ok_or(AuthError::InvalidLogin)?;
    let identity = state
        .0
        .provider
        .exchange(code, flow.verifier, flow.nonce)
        .await?;
    let token = SessionToken::generate().map_err(|_| AuthError::Internal)?;
    let previous = jar
        .get(state.0.cookies.session_name())
        .and_then(|cookie| SessionToken::hash_cookie(cookie.value()));
    state
        .0
        .sessions
        .replace(
            previous.as_ref().map(|hash| hash.as_slice()),
            token.hash(),
            &identity,
        )
        .await?;
    Ok(token)
}

#[utoipa::path(get, path = "/auth/me", tag = "auth", operation_id = "me", responses(
    (status = 200, body = Principal), (status = 401, body = AuthErrorResponse),
    (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
async fn me(AuthUser(principal): AuthUser) -> Json<Principal> {
    Json(principal)
}

#[utoipa::path(post, path = "/auth/logout", tag = "auth", operation_id = "logout", responses(
    (status = 204, description = "Revoca la sesión local y elimina la cookie; no cierra la sesión de Microsoft"),
    (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
async fn logout(
    _: SameOrigin,
    State(state): State<AuthState>,
    jar: CookieJar,
) -> Result<Response, AuthError> {
    if let Some(hash) = jar
        .get(state.0.cookies.session_name())
        .and_then(|cookie| SessionToken::hash_cookie(cookie.value()))
    {
        state.0.sessions.revoke(&hash).await?;
    }
    // Also cancel a pending login in this browser so it cannot recreate the session.
    if let Some(hash) = jar
        .get(state.0.cookies.flow_name())
        .and_then(|cookie| SessionToken::hash_cookie(cookie.value()))
    {
        let _ = state.take_flow(hash);
    }
    Ok((
        [(header::CACHE_CONTROL, "no-store")],
        jar.add(state.0.cookies.clear_session())
            .add(state.0.cookies.clear_login_flow()),
        StatusCode::NO_CONTENT,
    )
        .into_response())
}
