use super::{dto::*, service};
use crate::catalog::Filters;
use auth::{AuthErrorResponse, Permission, Require};
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
    const CODE: &'static str = "suppliers.read";
}
pub struct Create;
impl Permission for Create {
    const CODE: &'static str = "suppliers.create";
}
pub struct Update;
impl Permission for Update {
    const CODE: &'static str = "suppliers.update";
}
pub struct Delete;
impl Permission for Delete {
    const CODE: &'static str = "suppliers.delete";
}

#[utoipa::path(get, path = "/suppliers", tag = "suppliers", operation_id = "list_suppliers", params(Filters), responses((status = 200, body = Page<SupplierResponse>), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn list(
    _user: Require<Read>,
    State(state): State<AppState>,
    Query(filters): Query<Filters>,
) -> Result<Json<Page<SupplierResponse>>, AppError> {
    Ok(Json(service::list(&state.db, filters).await?))
}
#[utoipa::path(get, path = "/suppliers/{public_id}", tag = "suppliers", operation_id = "get_supplier", params(("public_id" = Uuid, Path)), responses((status = 200, body = SupplierResponse), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn get(
    _user: Require<Read>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SupplierResponse>, AppError> {
    Ok(Json(service::get(&state.db, id).await?.into()))
}
#[utoipa::path(post, path = "/suppliers", tag = "suppliers", operation_id = "create_supplier", request_body = CreateSupplier, responses((status = 201, body = SupplierResponse, headers(("Location" = String, description = "URL del recurso creado"))), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn create(
    _user: Require<Create>,
    State(state): State<AppState>,
    Input(body): Input<CreateSupplier>,
) -> Result<
    (
        StatusCode,
        [(header::HeaderName, String); 1],
        Json<SupplierResponse>,
    ),
    AppError,
> {
    let result = service::create(&state.db, body).await?;
    Ok((
        StatusCode::CREATED,
        [(
            header::LOCATION,
            format!("/api/suppliers/{}", result.public_id),
        )],
        Json(result),
    ))
}
#[utoipa::path(patch, path = "/suppliers/{public_id}", tag = "suppliers", operation_id = "update_supplier", params(("public_id" = Uuid, Path)), request_body = UpdateSupplier, responses((status = 200, body = SupplierResponse), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn update(
    _user: Require<Update>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Input(body): Input<UpdateSupplier>,
) -> Result<Json<SupplierResponse>, AppError> {
    Ok(Json(service::update(&state.db, id, body).await?))
}
#[utoipa::path(delete, path = "/suppliers/{public_id}", tag = "suppliers", operation_id = "delete_supplier", params(("public_id" = Uuid, Path)), responses((status = 204, description = "Registro eliminado"), (status = 400, body = ErrorResponse), (status = 404, body = ErrorResponse), (status = 409, body = ErrorResponse), (status = 500, body = ErrorResponse), (status = 401, body = AuthErrorResponse), (status = 403, body = AuthErrorResponse), (status = 503, body = AuthErrorResponse)), security(("session" = [])))]
pub async fn delete(
    _user: Require<Delete>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    service::delete(&state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
