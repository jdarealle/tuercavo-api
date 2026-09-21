use crate::{AuthError, AuthState, Principal, SessionToken};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{HeaderMap, Method, header, request::Parts},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::CookieJar;
use std::marker::PhantomData;

pub struct AuthUser(pub Principal);
pub struct SameOrigin;
pub trait Permission: Send + Sync {
    const CODE: &'static str;
}
pub struct Require<P: Permission>(pub Principal, PhantomData<P>);

pub(crate) fn check_origin(headers: &HeaderMap, expected: &str) -> Result<(), AuthError> {
    let mut origins = headers.get_all(header::ORIGIN).iter();
    if origins.next().and_then(|value| value.to_str().ok()) == Some(expected)
        && origins.next().is_none()
    {
        Ok(())
    } else {
        Err(AuthError::Forbidden)
    }
}

impl<S> FromRequestParts<S> for SameOrigin
where
    S: Send + Sync,
    AuthState: FromRef<S>,
{
    type Rejection = AuthError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        check_origin(&parts.headers, &AuthState::from_ref(state).0.public_origin)?;
        Ok(Self)
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AuthState: FromRef<S>,
{
    type Rejection = Response;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AuthState::from_ref(state);
        let jar = CookieJar::from_headers(&parts.headers);
        let result = async {
            let hash = jar
                .get(state.0.cookies.session_name())
                .and_then(|cookie| SessionToken::hash_cookie(cookie.value()))
                .ok_or(AuthError::Unauthorized)?;
            if !matches!(parts.method, Method::GET | Method::HEAD | Method::OPTIONS) {
                check_origin(&parts.headers, &state.0.public_origin)?;
            }
            state.0.sessions.authenticate(&hash).await
        }
        .await;
        match result {
            Ok(principal) => Ok(Self(principal)),
            Err(AuthError::Unauthorized) => Err((
                jar.add(state.0.cookies.clear_session()),
                AuthError::Unauthorized,
            )
                .into_response()),
            Err(error) => Err(error.into_response()),
        }
    }
}

impl<S, P> FromRequestParts<S> for Require<P>
where
    S: Send + Sync,
    AuthState: FromRef<S>,
    P: Permission,
{
    type Rejection = Response;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(principal) = AuthUser::from_request_parts(parts, state).await?;
        principal
            .require(P::CODE)
            .map_err(IntoResponse::into_response)?;
        Ok(Self(principal, PhantomData))
    }
}
