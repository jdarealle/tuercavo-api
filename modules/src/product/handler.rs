use super::{dto::*, service};
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
    const CODE: &'static str = permission::PRODUCTS_READ;
}
pub struct Create;
impl Permission for Create {
    const CODE: &'static str = permission::PRODUCTS_CREATE;
}
pub struct Update;
impl Permission for Update {
    const CODE: &'static str = permission::PRODUCTS_UPDATE;
}
pub struct Delete;
impl Permission for Delete {
    const CODE: &'static str = permission::PRODUCTS_DELETE;
}

#[utoipa::path(get, path = "/products", tag = "products", operation_id = "list_products", params(ProductFilters), responses((status = 200, body = Page<ProductResponse>), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn list(
    _user: Require<Read>,
    State(state): State<AppState>,
    Query(filters): Query<ProductFilters>,
) -> Result<Json<Page<ProductResponse>>, AppError> {
    Ok(Json(service::list(&state.db, filters).await?))
}
#[utoipa::path(get, path = "/products/{public_id}", tag = "products", operation_id = "get_product", params(("public_id" = Uuid, Path)), responses((status = 200, body = ProductResponse), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn get(
    _user: Require<Read>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductResponse>, AppError> {
    Ok(Json(service::get(&state.db, id).await?))
}
#[utoipa::path(post, path = "/products", tag = "products", operation_id = "create_product", request_body = CreateProduct, responses((status = 201, body = ProductResponse, headers(("Location" = String, description = "URL del recurso creado"))), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn create(
    _user: Require<Create>,
    State(state): State<AppState>,
    Input(body): Input<CreateProduct>,
) -> Result<
    (
        StatusCode,
        [(header::HeaderName, String); 1],
        Json<ProductResponse>,
    ),
    AppError,
> {
    let result = service::create(&state.db, body).await?;
    Ok((
        StatusCode::CREATED,
        [(
            header::LOCATION,
            format!("/api/products/{}", result.public_id),
        )],
        Json(result),
    ))
}
#[utoipa::path(patch, path = "/products/{public_id}", tag = "products", operation_id = "update_product", params(("public_id" = Uuid, Path)), request_body = UpdateProduct, responses((status = 200, body = ProductResponse), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn update(
    _user: Require<Update>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Input(body): Input<UpdateProduct>,
) -> Result<Json<ProductResponse>, AppError> {
    Ok(Json(service::update(&state.db, id, body).await?))
}
#[utoipa::path(delete, path = "/products/{public_id}", tag = "products", operation_id = "delete_product", params(("public_id" = Uuid, Path)), responses((status = 204, description = "Registro eliminado"), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn delete(
    _user: Require<Delete>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    service::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
