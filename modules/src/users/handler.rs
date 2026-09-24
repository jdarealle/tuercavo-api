use super::{dto::*, service};
use auth::{AuthErrorResponse, Permission, Require};
use axum::{Json, extract::State};
use common::{
    error::{AppError, ErrorResponse},
    http::{Path, Query},
    pagination::{Page, Pagination},
    state::AppState,
};
use uuid::Uuid;

pub struct Read;
impl Permission for Read {
    const CODE: &'static str = "users.read";
}
pub struct Update;
impl Permission for Update {
    const CODE: &'static str = "users.update";
}
pub struct ReadRoles;
impl Permission for ReadRoles {
    const CODE: &'static str = "roles.read";
}
pub struct ReadPermissions;
impl Permission for ReadPermissions {
    const CODE: &'static str = "permissions.read";
}

#[utoipa::path(get, path = "/users", tag = "users", operation_id = "list_users", params(Pagination), responses(
    (status = 200, body = Page<UserResponse>), (status = 400, body = ErrorResponse),
    (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn list(
    _actor: Require<Read>,
    State(state): State<AppState>,
    Query(p): Query<Pagination>,
) -> Result<Json<Page<UserResponse>>, AppError> {
    Ok(Json(
        service::list(&state.db, state.auth.tenant_id(), p).await?,
    ))
}

#[utoipa::path(get, path = "/users/{public_id}", tag = "users", operation_id = "get_user", params(("public_id" = Uuid, Path)), responses(
    (status = 200, body = UserResponse), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse),
    (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn get(
    _actor: Require<Read>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(
        service::get(&state.db, state.auth.tenant_id(), id).await?,
    ))
}

#[utoipa::path(post, path = "/users/{public_id}/deactivate", tag = "users", operation_id = "deactivate_user", description = "Desactiva el acceso local y revoca todas las sesiones del usuario en una transacción. Su asignación en Entra se retira por separado.", params(("public_id" = Uuid, Path)), responses(
    (status = 200, body = UserResponse, description = "Usuario desactivado y sesiones revocadas"), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse),
    (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn deactivate(
    actor: Require<Update>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(
        service::deactivate(&state.db, actor.0.user_id, state.auth.tenant_id(), id).await?,
    ))
}

#[utoipa::path(post, path = "/users/{public_id}/reactivate", tag = "users", operation_id = "reactivate_user", description = "Retira el bloqueo local sin restaurar sesiones. La asignación en Entra debe estar vigente y el usuario debe iniciar sesión de nuevo.", params(("public_id" = Uuid, Path)), responses(
    (status = 200, body = UserResponse, description = "Bloqueo local retirado; requiere un nuevo login"), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse),
    (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn reactivate(
    actor: Require<Update>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(
        service::reactivate(&state.db, actor.0.user_id, state.auth.tenant_id(), id).await?,
    ))
}

#[utoipa::path(get, path = "/roles", tag = "users", operation_id = "list_roles", responses(
    (status = 200, body = Vec<RoleResponse>), (status = 401, body = AuthErrorResponse),
    (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn roles(
    _actor: Require<ReadRoles>,
    State(state): State<AppState>,
) -> Result<Json<Vec<RoleResponse>>, AppError> {
    Ok(Json(service::roles(&state.db).await?))
}

#[utoipa::path(get, path = "/permissions", tag = "users", operation_id = "list_permissions", responses(
    (status = 200, body = Vec<PermissionResponse>), (status = 401, body = AuthErrorResponse),
    (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn permissions(
    _actor: Require<ReadPermissions>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PermissionResponse>>, AppError> {
    Ok(Json(service::permissions(&state.db).await?))
}
