use axum::{
    Json,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug)]
pub enum AuthError {
    Unauthorized,
    Forbidden,
    InvalidLogin,
    Provider,
    Unavailable,
    Internal,
}

#[derive(Serialize, ToSchema)]
pub struct AuthErrorResponse {
    pub error: AuthErrorDetail,
}

#[derive(Serialize, ToSchema)]
pub struct AuthErrorDetail {
    pub code: &'static str,
    pub message: &'static str,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Sesión inexistente, expirada o revocada",
            ),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden", "Acceso denegado"),
            Self::InvalidLogin => (
                StatusCode::BAD_REQUEST,
                "invalid_login",
                "Login inválido o expirado; inicia sesión de nuevo",
            ),
            Self::Provider => (
                StatusCode::BAD_GATEWAY,
                "identity_provider_error",
                "No se pudo validar el login con el proveedor",
            ),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "auth_unavailable",
                "Autenticación temporalmente no disponible",
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Error interno de autenticación",
            ),
        };
        (
            status,
            [
                (header::CACHE_CONTROL, "no-store"),
                (header::REFERRER_POLICY, "no-referrer"),
            ],
            Json(AuthErrorResponse {
                error: AuthErrorDetail { code, message },
            }),
        )
            .into_response()
    }
}

impl From<sea_orm::DbErr> for AuthError {
    fn from(_: sea_orm::DbErr) -> Self {
        // Driver messages can contain SQL, personal data or credentials.
        tracing::error!("authentication database operation failed");
        Self::Unavailable
    }
}
