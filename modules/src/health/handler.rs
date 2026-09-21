use super::{dto::Health, service};
use axum::{Json, extract::State};
use common::{
    error::{AppError, ErrorResponse},
    state::AppState,
};
#[utoipa::path(get, path = "/health/live", tag = "health", operation_id = "live", responses((status = 200, body = Health)))]
pub async fn live() -> Json<Health> {
    Json(Health { status: "ok" })
}
#[utoipa::path(get, path = "/health/ready", tag = "health", operation_id = "ready", responses((status = 200, body = Health), (status = 503, body = ErrorResponse)))]
pub async fn ready(State(state): State<AppState>) -> Result<Json<Health>, AppError> {
    service::ready(&state.db).await?;
    Ok(Json(Health { status: "ok" }))
}
