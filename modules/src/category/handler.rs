use super::{dto::*, service};
use crate::catalog::Filters;
use auth::{AuthErrorResponse, Permission, Require, permission};
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
};
use common::{
    error::{AppError, ErrorResponse},
    http::{Json as Input, Path, Query},
    pagination::Page,
    state::AppState,
};
use uuid::Uuid;
pub struct Read;
impl Permission for Read {
    const CODE: &'static str = permission::CATEGORIES_READ;
}
pub struct Create;
impl Permission for Create {
    const CODE: &'static str = permission::CATEGORIES_CREATE;
}
pub struct Update;
impl Permission for Update {
    const CODE: &'static str = permission::CATEGORIES_UPDATE;
}
pub struct Delete;
impl Permission for Delete {
    const CODE: &'static str = permission::CATEGORIES_DELETE;
}

#[utoipa::path(get, path = "/categories", tag = "categories", operation_id = "list_categories", params(Filters), responses((status = 200, body = Page<CategoryResponse>), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn list(
    _user: Require<Read>,
    State(state): State<AppState>,
    Query(filters): Query<Filters>,
) -> Result<Json<Page<CategoryResponse>>, AppError> {
    Ok(Json(service::list(&state.db, filters).await?))
}
#[utoipa::path(get, path = "/categories/{public_id}", tag = "categories", operation_id = "get_category", params(("public_id" = Uuid, Path)), responses((status = 200, body = CategoryResponse), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn get(
    _user: Require<Read>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CategoryResponse>, AppError> {
    Ok(Json(service::get(&state.db, id).await?.into()))
}
#[utoipa::path(post, path = "/categories", tag = "categories", operation_id = "create_category", request_body = CreateCategory, responses((status = 201, body = CategoryResponse, headers(("Location" = String, description = "URL del recurso creado"))), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn create(
    _user: Require<Create>,
    State(state): State<AppState>,
    Input(body): Input<CreateCategory>,
) -> Result<
    (
        StatusCode,
        [(header::HeaderName, String); 1],
        Json<CategoryResponse>,
    ),
    AppError,
> {
    let result = service::create(&state.db, body).await?;
    Ok((
        StatusCode::CREATED,
        [(
            header::LOCATION,
            format!("/api/categories/{}", result.public_id),
        )],
        Json(result),
    ))
}
#[utoipa::path(patch, path = "/categories/{public_id}", tag = "categories", operation_id = "update_category", params(("public_id" = Uuid, Path)), request_body = UpdateCategory, responses((status = 200, body = CategoryResponse), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn update(
    _user: Require<Update>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Input(body): Input<UpdateCategory>,
) -> Result<Json<CategoryResponse>, AppError> {
    Ok(Json(service::update(&state.db, id, body).await?))
}
#[utoipa::path(delete, path = "/categories/{public_id}", tag = "categories", operation_id = "delete_category", params(("public_id" = Uuid, Path)), responses((status = 204, description = "Registro eliminado"), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn delete(
    _user: Require<Delete>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    service::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
