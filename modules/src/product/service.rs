use super::dto::*;
use crate::catalog::{Status, Unit};
use ::entity::{
    categories,
    products::{self, Column, Entity},
    sea_orm_active_enums::CatalogStatus,
    suppliers,
};
use common::{error::AppError, pagination::Page, validation as v};
use sea_orm::sea_query::Expr;
use sea_orm::{
    sea_query::{LockType, extension::postgres::PgExpr},
    *,
};
use std::collections::HashMap;
use uuid::Uuid;

async fn responses<C: ConnectionTrait>(
    db: &C,
    models: Vec<products::Model>,
) -> Result<Vec<ProductResponse>, AppError> {
    if models.is_empty() {
        return Ok(Vec::new());
    }
    let categories: HashMap<_, _> = categories::Entity::find()
        .filter(categories::Column::Id.is_in(models.iter().map(|m| m.category_id)))
        .all(db)
        .await?
        .into_iter()
        .map(|m| (m.id, m.public_id))
        .collect();
    let suppliers: HashMap<_, _> = suppliers::Entity::find()
        .filter(suppliers::Column::Id.is_in(models.iter().filter_map(|m| m.supplier_id)))
        .all(db)
        .await?
        .into_iter()
        .map(|m| (m.id, m.public_id))
        .collect();
    models
        .into_iter()
        .map(|m| {
            Ok(ProductResponse {
                public_id: m.public_id,
                sku: m.sku,
                name: m.name,
                description: m.description,
                brand: m.brand,
                category: *categories.get(&m.category_id).ok_or(AppError::Internal)?,
                supplier: m
                    .supplier_id
                    .map(|id| suppliers.get(&id).copied().ok_or(AppError::Internal))
                    .transpose()?,
                unit: Unit::parse(&m.unit)?,
                price: format!("{:.2}", m.price),
                status: m.status.into(),
                created_at: m.created_at,
                updated_at: m.updated_at,
            })
        })
        .collect()
}
async fn response<C: ConnectionTrait>(
    db: &C,
    m: products::Model,
) -> Result<ProductResponse, AppError> {
    responses(db, vec![m])
        .await?
        .pop()
        .ok_or(AppError::Internal)
}
pub async fn list(
    db: &DatabaseConnection,
    f: ProductFilters,
) -> Result<Page<ProductResponse>, AppError> {
    let p = f.pagination()?;
    let mut q = Entity::find();
    if let Some(s) = f.status {
        q = q.filter(Column::Status.eq(CatalogStatus::from(s)));
    }
    if let Some(s) = f.search {
        let s = v::search(&s)?;
        q = q.filter(
            Condition::any()
                .add(Expr::col(Column::Name).ilike(s.clone()))
                .add(Expr::col(Column::Sku).ilike(s)),
        );
    }
    if let Some(id) = f.category {
        q = q.filter(
            Column::CategoryId.in_subquery(
                categories::Entity::find()
                    .select_only()
                    .column(categories::Column::Id)
                    .filter(categories::Column::PublicId.eq(id))
                    .into_query(),
            ),
        );
    }
    if let Some(id) = f.supplier {
        q = q.filter(
            Column::SupplierId.in_subquery(
                suppliers::Entity::find()
                    .select_only()
                    .column(suppliers::Column::Id)
                    .filter(suppliers::Column::PublicId.eq(id))
                    .into_query(),
            ),
        );
    }
    let pages = q.order_by_asc(Column::Id).paginate(db, p.per_page);
    let total = pages.num_items().await?;
    Ok(Page::new(
        responses(db, pages.fetch_page(p.page - 1).await?).await?,
        total,
        &p,
    ))
}
pub async fn get(db: &DatabaseConnection, id: Uuid) -> Result<ProductResponse, AppError> {
    response(
        db,
        Entity::find()
            .filter(Column::PublicId.eq(id))
            .one(db)
            .await?
            .ok_or(AppError::NotFound)?,
    )
    .await
}
async fn category(
    db: &DatabaseTransaction,
    id: Uuid,
    current: Option<i32>,
) -> Result<i32, AppError> {
    let m = categories::Entity::find()
        .filter(categories::Column::PublicId.eq(id))
        .lock(LockType::Share)
        .one(db)
        .await?
        .ok_or_else(|| AppError::bad("category no existe"))?;
    if Some(m.id) != current && m.status != CatalogStatus::Active {
        return Err(AppError::conflict("category está inactiva"));
    }
    Ok(m.id)
}
async fn supplier(
    db: &DatabaseTransaction,
    id: Uuid,
    current: Option<i32>,
) -> Result<i32, AppError> {
    let m = suppliers::Entity::find()
        .filter(suppliers::Column::PublicId.eq(id))
        .lock(LockType::Share)
        .one(db)
        .await?
        .ok_or_else(|| AppError::bad("supplier no existe"))?;
    if Some(m.id) != current && m.status != CatalogStatus::Active {
        return Err(AppError::conflict("supplier está inactivo"));
    }
    Ok(m.id)
}
pub async fn create(
    db: &DatabaseConnection,
    input: CreateProduct,
) -> Result<ProductResponse, AppError> {
    let tx = db.begin().await?;
    let category_id = category(&tx, input.category, None).await?;
    let supplier_id = match input.supplier {
        Some(id) => Some(supplier(&tx, id, None).await?),
        None => None,
    };
    let m = products::ActiveModel {
        sku: Set(v::code(input.sku, "sku")?),
        name: Set(v::text(input.name, "name", 150)?),
        description: Set(v::optional(input.description, "description", 4000)?),
        brand: Set(v::optional(input.brand, "brand", 100)?),
        category_id: Set(category_id),
        supplier_id: Set(supplier_id),
        unit: Set(input.unit.as_str().into()),
        price: Set(v::price(&input.price)?),
        status: Set(input.status.unwrap_or(Status::Active).into()),
        ..Default::default()
    }
    .insert(&tx)
    .await?;
    let result = response(&tx, m).await?;
    tx.commit().await?;
    Ok(result)
}
pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    input: UpdateProduct,
) -> Result<ProductResponse, AppError> {
    let tx = db.begin().await?;
    let old = Entity::find()
        .filter(Column::PublicId.eq(id))
        .lock_exclusive()
        .one(&tx)
        .await?
        .ok_or(AppError::NotFound)?;
    let old_category = old.category_id;
    let old_supplier = old.supplier_id;
    let mut m = old.into_active_model();
    if let Some(x) = input.sku.required("sku")? {
        m.sku = Set(v::code(x, "sku")?);
    }
    if let Some(x) = input.name.required("name")? {
        m.name = Set(v::text(x, "name", 150)?);
    }
    if let Some(x) = input.description.optional() {
        m.description = Set(v::optional(x, "description", 4000)?);
    }
    if let Some(x) = input.brand.optional() {
        m.brand = Set(v::optional(x, "brand", 100)?);
    }
    if let Some(x) = input.category.required("category")? {
        m.category_id = Set(category(&tx, x, Some(old_category)).await?);
    }
    if let Some(x) = input.supplier.optional() {
        m.supplier_id = Set(match x {
            Some(x) => Some(supplier(&tx, x, old_supplier).await?),
            None => None,
        });
    }
    if let Some(x) = input.unit.required("unit")? {
        m.unit = Set(x.as_str().into());
    }
    if let Some(x) = input.price.required("price")? {
        m.price = Set(v::price(&x)?);
    }
    if let Some(x) = input.status.required("status")? {
        m.status = Set(x.into());
    }
    m.updated_at = Set(chrono::Utc::now().fixed_offset());
    let result = response(&tx, m.update(&tx).await?).await?;
    tx.commit().await?;
    Ok(result)
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
