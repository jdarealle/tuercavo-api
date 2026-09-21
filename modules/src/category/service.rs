use super::dto::*;
use crate::catalog::{Filters, Status};
use ::entity::categories::{self, Column, Entity};
use common::{error::AppError, pagination::Page, validation as v};
use sea_orm::sea_query::Expr;
use sea_orm::{sea_query::extension::postgres::PgExpr, *};
use uuid::Uuid;

pub async fn list(db: &DatabaseConnection, f: Filters) -> Result<Page<CategoryResponse>, AppError> {
    let p = f.pagination()?;
    let mut q = Entity::find();
    if let Some(s) = f.status {
        q = q.filter(Column::Status.eq(::entity::sea_orm_active_enums::CatalogStatus::from(s)));
    }
    if let Some(s) = f.search {
        q = q.filter(Expr::col(Column::Name).ilike(v::search(&s)?));
    }
    let pages = q.order_by_asc(Column::Id).paginate(db, p.per_page);
    let total = pages.num_items().await?;
    Ok(Page::new(
        pages
            .fetch_page(p.page - 1)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
        total,
        &p,
    ))
}
pub async fn get<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<categories::Model, AppError> {
    Entity::find()
        .filter(Column::PublicId.eq(id))
        .one(db)
        .await?
        .ok_or(AppError::NotFound)
}
pub async fn create(
    db: &DatabaseConnection,
    input: CreateCategory,
) -> Result<CategoryResponse, AppError> {
    let model = categories::ActiveModel {
        name: Set(v::text(input.name, "name", 150)?),
        description: Set(v::optional(input.description, "description", 4000)?),
        status: Set(input.status.unwrap_or(Status::Active).into()),
        ..Default::default()
    }
    .insert(db)
    .await?;
    Ok(model.into())
}
pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    input: UpdateCategory,
) -> Result<CategoryResponse, AppError> {
    let mut model = get(db, id).await?.into_active_model();
    if let Some(v) = input.name.required("name")? {
        model.name = Set(v::text(v, "name", 150)?);
    }
    if let Some(v) = input.description.optional() {
        model.description = Set(v::optional(v, "description", 4000)?);
    }
    if let Some(v) = input.status.required("status")? {
        model.status = Set(v.into());
    }
    model.updated_at = Set(chrono::Utc::now().fixed_offset());
    Ok(model.update(db).await?.into())
}
pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
    let result = Entity::delete_many()
        .filter(Column::PublicId.eq(id))
        .exec(db)
        .await?;
    if result.rows_affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}
