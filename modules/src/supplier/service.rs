use super::dto::*;
use crate::catalog::{Filters, Status};
use ::entity::suppliers::{self, Column, Entity};
use common::{error::AppError, pagination::Page, validation as v};
use sea_orm::sea_query::Expr;
use sea_orm::{sea_query::extension::postgres::PgExpr, *};
use uuid::Uuid;
pub async fn list(db: &DatabaseConnection, f: Filters) -> Result<Page<SupplierResponse>, AppError> {
    let p = f.pagination()?;
    let mut q = Entity::find();
    if let Some(s) = f.status {
        q = q.filter(Column::Status.eq(::entity::sea_orm_active_enums::CatalogStatus::from(s)));
    }
    if let Some(s) = f.search {
        let s = v::search(&s)?;
        q = q.filter(
            Condition::any()
                .add(Expr::col(Column::Name).ilike(s.clone()))
                .add(Expr::col(Column::Code).ilike(s)),
        );
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
pub async fn get<C: ConnectionTrait>(db: &C, id: Uuid) -> Result<suppliers::Model, AppError> {
    Entity::find()
        .filter(Column::PublicId.eq(id))
        .one(db)
        .await?
        .ok_or(AppError::NotFound)
}
pub async fn create(
    db: &DatabaseConnection,
    input: CreateSupplier,
) -> Result<SupplierResponse, AppError> {
    Ok(suppliers::ActiveModel {
        code: Set(v::code(input.code, "code")?),
        name: Set(v::text(input.name, "name", 150)?),
        contact_name: Set(v::optional(input.contact_name, "contact_name", 150)?),
        email: Set(input.email.map(v::email).transpose()?),
        phone: Set(input.phone.map(v::phone).transpose()?),
        status: Set(input.status.unwrap_or(Status::Active).into()),
        ..Default::default()
    }
    .insert(db)
    .await?
    .into())
}
pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    input: UpdateSupplier,
) -> Result<SupplierResponse, AppError> {
    let mut m = get(db, id).await?.into_active_model();
    if let Some(x) = input.code.required("code")? {
        m.code = Set(v::code(x, "code")?);
    }
    if let Some(x) = input.name.required("name")? {
        m.name = Set(v::text(x, "name", 150)?);
    }
    if let Some(x) = input.contact_name.optional() {
        m.contact_name = Set(v::optional(x, "contact_name", 150)?);
    }
    if let Some(x) = input.email.optional() {
        m.email = Set(x.map(v::email).transpose()?);
    }
    if let Some(x) = input.phone.optional() {
        m.phone = Set(x.map(v::phone).transpose()?);
    }
    if let Some(x) = input.status.required("status")? {
        m.status = Set(x.into());
    }
    m.updated_at = Set(chrono::Utc::now().fixed_offset());
    Ok(m.update(db).await?.into())
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
