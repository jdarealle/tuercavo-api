use super::dto::{CreateDepartment, DepartmentResponse};
use common::{error::AppError, validation};
use entity::departments;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

pub async fn list(db: &DatabaseConnection) -> Result<Vec<DepartmentResponse>, AppError> {
    Ok(departments::Entity::find()
        .order_by_asc(departments::Column::Name)
        .all(db)
        .await?
        .into_iter()
        .map(Into::into)
        .collect())
}

pub async fn get(db: &DatabaseConnection, public_id: Uuid) -> Result<DepartmentResponse, AppError> {
    Ok(departments::Entity::find()
        .filter(departments::Column::PublicId.eq(public_id))
        .one(db)
        .await?
        .ok_or(AppError::NotFound)?
        .into())
}

pub async fn create(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    body: CreateDepartment,
) -> Result<DepartmentResponse, AppError> {
    let name = validation::text(body.name, "name", 150)?;
    let (tx, _) =
        crate::authorization::begin_admin(db, actor_id, tenant, "departments.create").await?;
    let department = departments::ActiveModel {
        name: Set(name),
        ..Default::default()
    }
    .insert(&tx)
    .await?;
    tx.commit().await?;
    Ok(department.into())
}
