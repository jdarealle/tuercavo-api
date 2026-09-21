use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{DbErr, RuntimeErr, SqlErr};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug)]
pub enum AppError {
    Auth(auth::AuthError),
    BadRequest(String),
    NotFound,
    Conflict(String),
    Internal,
    Unavailable,
    Timeout,
    MethodNotAllowed,
    PayloadTooLarge,
}
#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}
#[derive(Serialize, ToSchema)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub request_id: String,
}
impl AppError {
    pub fn bad(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::Auth(error) => return error.into_response(),
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, "invalid_request", m),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "Recurso no encontrado".into(),
            ),
            Self::Conflict(m) => (StatusCode::CONFLICT, "conflict", m),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "unavailable",
                "Servicio no disponible".into(),
            ),
            Self::Timeout => (
                StatusCode::REQUEST_TIMEOUT,
                "timeout",
                "Tiempo de espera agotado".into(),
            ),
            Self::MethodNotAllowed => (
                StatusCode::METHOD_NOT_ALLOWED,
                "method_not_allowed",
                "Método no permitido".into(),
            ),
            Self::PayloadTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                "payload_too_large",
                "Cuerpo demasiado grande".into(),
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Error interno".into(),
            ),
        };
        (
            status,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: code.into(),
                    message,
                    request_id: crate::telemetry::request_id(),
                },
            }),
        )
            .into_response()
    }
}
impl From<auth::AuthError> for AppError {
    fn from(error: auth::AuthError) -> Self {
        Self::Auth(error)
    }
}
impl From<DbErr> for AppError {
    fn from(error: DbErr) -> Self {
        match error.sql_err() {
            Some(SqlErr::UniqueConstraintViolation(_)) => {
                return Self::conflict("Ya existe un registro con ese identificador o nombre");
            }
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => {
                return Self::conflict(
                    "El registro tiene referencias dependientes o la referencia cambió",
                );
            }
            _ => {}
        }
        if let DbErr::Query(RuntimeErr::SqlxError(e)) | DbErr::Exec(RuntimeErr::SqlxError(e)) =
            &error
            && let sea_orm::sqlx::Error::Database(db) = e.as_ref()
            && let Some("23514" | "23502" | "22001" | "22003") = db.code().as_deref()
        {
            return Self::bad("Los datos incumplen una restricción");
        }
        // Do not log driver messages: they may contain SQL, credentials or user data.
        tracing::error!("database operation failed");
        Self::Internal
    }
}
