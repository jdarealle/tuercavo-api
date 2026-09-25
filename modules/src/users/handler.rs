use super::{dto::*, service};
use auth::{AuthErrorResponse, Permission, Require, permission};
use axum::{Json, extract::State};
use common::{
    error::{AppError, ErrorResponse},
    http::{Json as Input, Path, Query},
    pagination::{Page, Pagination},
    state::AppState,
};
use uuid::Uuid;

pub struct Read;
impl Permission for Read {
    const CODE: &'static str = permission::USERS_READ;
}
pub struct Update;
impl Permission for Update {
    const CODE: &'static str = permission::USERS_UPDATE;
}
pub struct Assign;
impl Permission for Assign {
    const CODE: &'static str = permission::USERS_ASSIGN_ROLE;
}
pub struct AssignDepartmentPermission;
impl Permission for AssignDepartmentPermission {
    const CODE: &'static str = permission::USERS_ASSIGN_DEPARTMENT;
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
    (status = 200, body = UserResponse), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse),
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
    (status = 200, body = UserResponse, description = "Usuario desactivado y sesiones revocadas"), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse),
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
    (status = 200, body = UserResponse, description = "Bloqueo local retirado; requiere un nuevo login"), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse),
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

#[utoipa::path(put, path = "/users/{public_id}/role", tag = "users", operation_id = "assign_user_role",
    description = "Asigna un rol local activo y revoca las sesiones si cambia. Protege al último administrador activo.",
    params(("public_id" = Uuid, Path)), request_body = AssignRole,
    responses((status = 200, body = UserResponse), (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse),
        (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)),
    security(("session" = [])))]
pub async fn assign_role(
    actor: Require<Assign>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Input(body): Input<AssignRole>,
) -> Result<Json<UserResponse>, AppError> {
    Ok(Json(
        service::assign_role(
            &state.db,
            actor.0.user_id,
            state.auth.tenant_id(),
            id,
            &body.role,
        )
        .await?,
    ))
}

#[utoipa::path(put, path = "/users/{public_id}/department", tag = "users", operation_id = "assign_user_department",
    description = "Solo el rol local admin puede asignar un departamento por UUID público o quitar la asignación con department_public_id: null. No cambia el rol ni las sesiones.",
    params(("public_id" = Uuid, Path)), request_body = AssignDepartment,
    responses((status = 200, body = UserResponse), (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse),
        (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)),
    security(("session" = [])))]
pub async fn assign_department(
    actor: Require<AssignDepartmentPermission>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Input(body): Input<AssignDepartment>,
) -> Result<Json<UserResponse>, AppError> {
    let department_public_id = body
        .department_public_id
        .optional()
        .ok_or_else(|| AppError::bad("Indica department_public_id o null"))?;
    Ok(Json(
        service::assign_department(
            &state.db,
            actor.0.user_id,
            state.auth.tenant_id(),
            id,
            department_public_id,
        )
        .await?,
    ))
}
