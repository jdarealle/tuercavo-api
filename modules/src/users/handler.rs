use super::{dto::*, service};
use auth::{AuthErrorResponse, Permission, Require};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
};
use common::{
    error::{AppError, ErrorResponse},
    http::{Json as Input, Path, Query},
    pagination::{Page, Pagination},
    state::AppState,
};
use uuid::Uuid;

pub struct Read;
impl Permission for Read {
    const CODE: &'static str = "users.read";
}
pub struct Create;
impl Permission for Create {
    const CODE: &'static str = "users.create";
}
pub struct Update;
impl Permission for Update {
    const CODE: &'static str = "users.update";
}
pub struct Assign;
impl Permission for Assign {
    const CODE: &'static str = "users.assign_role";
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

#[utoipa::path(post, path = "/users", tag = "users", operation_id = "create_user", request_body = CreateUser, responses(
    (status = 201, body = UserResponse, headers(("Location" = String))), (status = 400, body = ErrorResponse), (status = 409, body = ErrorResponse),
    (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn create(
    actor: Require<Create>,
    State(state): State<AppState>,
    Input(body): Input<CreateUser>,
) -> Result<
    (
        StatusCode,
        [(header::HeaderName, String); 1],
        Json<UserResponse>,
    ),
    AppError,
> {
    let result = service::create(&state.db, actor.0.user_id, state.auth.tenant_id(), body).await?;
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("/api/users/{}", result.public_id))],
        Json(result),
    ))
}

#[utoipa::path(patch, path = "/users/{public_id}", tag = "users", operation_id = "update_user", params(("public_id" = Uuid, Path)), request_body = UpdateUser, responses(
    (status = 200, body = UserResponse), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse),
    (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn update(
    actor: Require<Update>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Input(body): Input<UpdateUser>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(
        service::update(&state.db, actor.0.user_id, state.auth.tenant_id(), id, body).await?,
    ))
}

#[utoipa::path(put, path = "/users/{public_id}/role", tag = "users", operation_id = "assign_role", params(("public_id" = Uuid, Path)), request_body = AssignRole, responses(
    (status = 200, body = UserResponse), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse),
    (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn assign_role(
    actor: Require<Assign>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Input(body): Input<AssignRole>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(
        service::assign_role(&state.db, actor.0.user_id, state.auth.tenant_id(), id, body).await?,
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
