use super::{dto::*, service};
use auth::{AuthErrorResponse, AuthUser, Permission, Require};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
};
use common::{
    error::{AppError, ErrorResponse},
    http::{Json as Input, Path},
    state::AppState,
};
use uuid::Uuid;

pub struct Read;
impl Permission for Read {
    const CODE: &'static str = "departments.read";
}

pub struct Create;
impl Permission for Create {
    const CODE: &'static str = "departments.create";
}

#[utoipa::path(get, path = "/departments", tag = "departments", operation_id = "list_departments",
    description = "Solo el rol local admin puede consultar departamentos.",
    responses((status = 200, body = Vec<DepartmentResponse>),
        (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse),
        (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn list(
    actor: Require<Read>,
    State(state): State<AppState>,
) -> Result<Json<Vec<DepartmentResponse>>, AppError> {
    crate::authorization::require_admin(&actor.0)?;
    Ok(Json(service::list(&state.db).await?))
}

#[utoipa::path(get, path = "/departments/me", tag = "departments", operation_id = "get_my_department",
    description = "Consulta el departamento del usuario autenticado, sin depender de su rol. Devuelve null si no tiene departamento asignado.",
    responses((status = 200, body = Option<DepartmentResponse>),
        (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse),
        (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn mine(
    actor: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Option<DepartmentResponse>>, AppError> {
    let department = match actor.0.department_public_id {
        Some(public_id) => Some(service::get(&state.db, public_id).await?),
        None => None,
    };
    Ok(Json(department))
}

#[utoipa::path(get, path = "/departments/{public_id}", tag = "departments", operation_id = "get_department",
    description = "Solo el rol local admin puede consultar departamentos.",
    params(("public_id" = Uuid, Path)),
    responses((status = 200, body = DepartmentResponse), (status = 400, body = ErrorResponse),
        (status = 404, body = ErrorResponse), (status = 401, body = AuthErrorResponse),
        (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)),
    security(("session" = [])))]
pub async fn get(
    actor: Require<Read>,
    State(state): State<AppState>,
    Path(public_id): Path<Uuid>,
) -> Result<Json<DepartmentResponse>, AppError> {
    crate::authorization::require_admin(&actor.0)?;
    Ok(Json(service::get(&state.db, public_id).await?))
}

#[utoipa::path(post, path = "/departments", tag = "departments", operation_id = "create_department",
    description = "Solo el rol local admin puede crear departamentos.",
    request_body = CreateDepartment,
    responses((status = 201, body = DepartmentResponse, headers(("Location" = String, description = "URL del departamento creado"))),
        (status = 400, body = ErrorResponse), (status = 409, body = ErrorResponse),
        (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse),
        (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn create(
    actor: Require<Create>,
    State(state): State<AppState>,
    Input(body): Input<CreateDepartment>,
) -> Result<
    (
        StatusCode,
        [(header::HeaderName, String); 1],
        Json<DepartmentResponse>,
    ),
    AppError,
> {
    crate::authorization::require_admin(&actor.0)?;
    let department =
        service::create(&state.db, actor.0.user_id, state.auth.tenant_id(), body).await?;
    Ok((
        StatusCode::CREATED,
        [(
            header::LOCATION,
            format!("/api/departments/{}", department.public_id),
        )],
        Json(department),
    ))
}
