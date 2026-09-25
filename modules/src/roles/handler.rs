use super::{dto::*, service};
use auth::{AuthErrorResponse, Permission, Require, permission};
use axum::{Json, extract::State, http::StatusCode};
use common::{
    error::{AppError, ErrorResponse},
    http::{Json as Input, Path},
    state::AppState,
};

pub struct Read;
impl Permission for Read {
    const CODE: &'static str = permission::ROLES_READ;
}
pub struct ReadPermissions;
impl Permission for ReadPermissions {
    const CODE: &'static str = permission::PERMISSIONS_READ;
}
pub struct Create;
impl Permission for Create {
    const CODE: &'static str = permission::ROLES_CREATE;
}
pub struct Update;
impl Permission for Update {
    const CODE: &'static str = permission::ROLES_UPDATE;
}
pub struct AssignPermissions;
impl Permission for AssignPermissions {
    const CODE: &'static str = permission::ROLES_ASSIGN_PERMISSIONS;
}

#[utoipa::path(get, path = "/roles", tag = "roles", operation_id = "list_roles",
    responses((status = 200, body = Vec<RoleResponse>), (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse),
        (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)),
    security(("session" = [])))]
pub async fn list(
    _actor: Require<Read>,
    State(state): State<AppState>,
) -> Result<Json<Vec<RoleResponse>>, AppError> {
    Ok(Json(service::list(&state.db).await?))
}

#[utoipa::path(get, path = "/roles/{code}", tag = "roles", operation_id = "get_role",
    params(("code" = String, Path)),
    responses((status = 200, body = RoleResponse), (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse),
        (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)),
    security(("session" = [])))]
pub async fn get(
    _actor: Require<Read>,
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<RoleResponse>, AppError> {
    Ok(Json(service::get(&state.db, &code).await?))
}

#[utoipa::path(get, path = "/permissions", tag = "roles", operation_id = "list_permissions",
    responses((status = 200, body = Vec<PermissionResponse>), (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse),
        (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)),
    security(("session" = [])))]
pub async fn permissions(
    _actor: Require<ReadPermissions>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PermissionResponse>>, AppError> {
    Ok(Json(service::catalog(&state.db).await?))
}

#[utoipa::path(post, path = "/roles", tag = "roles", operation_id = "create_role",
    request_body = CreateRole,
    responses((status = 201, body = RoleResponse), (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse),
        (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)),
    security(("session" = [])))]
pub async fn create(
    actor: Require<Create>,
    State(state): State<AppState>,
    Input(body): Input<CreateRole>,
) -> Result<(StatusCode, Json<RoleResponse>), AppError> {
    Ok((
        StatusCode::CREATED,
        Json(service::create(&state.db, actor.0.user_id, state.auth.tenant_id(), body).await?),
    ))
}

#[utoipa::path(patch, path = "/roles/{code}", tag = "roles", operation_id = "update_role",
    params(("code" = String, Path)),
    request_body = UpdateRole,
    responses((status = 200, body = RoleResponse), (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse),
        (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)),
    security(("session" = [])))]
pub async fn update(
    actor: Require<Update>,
    State(state): State<AppState>,
    Path(code): Path<String>,
    Input(body): Input<UpdateRole>,
) -> Result<Json<RoleResponse>, AppError> {
    Ok(Json(
        service::update(
            &state.db,
            actor.0.user_id,
            state.auth.tenant_id(),
            &code,
            body,
        )
        .await?,
    ))
}

#[utoipa::path(put, path = "/roles/{code}/permissions", tag = "roles", operation_id = "set_role_permissions",
    params(("code" = String, Path)),
    request_body = SetPermissions,
    responses((status = 200, body = RoleResponse), (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse),
        (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)),
    security(("session" = [])))]
pub async fn set_permissions(
    actor: Require<AssignPermissions>,
    State(state): State<AppState>,
    Path(code): Path<String>,
    Input(body): Input<SetPermissions>,
) -> Result<Json<RoleResponse>, AppError> {
    Ok(Json(
        service::set_permissions(
            &state.db,
            actor.0.user_id,
            state.auth.tenant_id(),
            &code,
            body,
        )
        .await?,
    ))
}
