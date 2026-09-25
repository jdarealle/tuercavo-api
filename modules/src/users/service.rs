use super::dto::*;
use common::{
    error::AppError,
    pagination::{Page, Pagination},
};
use entity::{departments, roles, users};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use uuid::Uuid;

fn response(user: users::Model, role: String, department_public_id: Option<Uuid>) -> UserResponse {
    UserResponse {
        public_id: user.public_id,
        entra_tenant_id: user.entra_tenant_id,
        entra_object_id: user.entra_object_id,
        email: user.email,
        full_name: user.full_name,
        role,
        department_public_id,
        is_active: user.is_active,
        created_at: user.created_at,
        updated_at: user.updated_at,
    }
}

async fn department_public_id<C: ConnectionTrait>(
    db: &C,
    department_id: Option<i32>,
) -> Result<Option<Uuid>, AppError> {
    match department_id {
        Some(id) => Ok(Some(
            departments::Entity::find_by_id(id)
                .one(db)
                .await?
                .ok_or(AppError::Internal)?
                .public_id,
        )),
        None => Ok(None),
    }
}

async fn describe<C: ConnectionTrait>(
    db: &C,
    user: users::Model,
) -> Result<UserResponse, AppError> {
    let role = roles::Entity::find_by_id(user.role_id)
        .one(db)
        .await?
        .ok_or(AppError::Internal)?;
    let department_public_id = department_public_id(db, user.department_id).await?;
    Ok(response(user, role.code, department_public_id))
}

pub async fn list(
    db: &DatabaseConnection,
    tenant: Uuid,
    p: Pagination,
) -> Result<Page<UserResponse>, AppError> {
    p.validate()?;
    let query = users::Entity::find()
        .filter(users::Column::EntraTenantId.eq(tenant))
        .order_by_asc(users::Column::Id)
        .paginate(db, p.per_page);
    let total = query.num_items().await?;
    let roles: std::collections::HashMap<_, _> = roles::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|role| (role.id, role.code))
        .collect();
    let departments: std::collections::HashMap<_, _> = departments::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|department| (department.id, department.public_id))
        .collect();
    let rows = query
        .fetch_page(p.page - 1)
        .await?
        .into_iter()
        .map(|user| {
            let role = roles.get(&user.role_id).ok_or(AppError::Internal)?.clone();
            let department_public_id = match user.department_id {
                Some(id) => Some(*departments.get(&id).ok_or(AppError::Internal)?),
                None => None,
            };
            Ok(response(user, role, department_public_id))
        })
        .collect::<Result<Vec<_>, AppError>>()?;
    Ok(Page::new(rows, total, &p))
}

pub async fn get(
    db: &DatabaseConnection,
    tenant: Uuid,
    public_id: Uuid,
) -> Result<UserResponse, AppError> {
    let user = users::Entity::find()
        .filter(users::Column::PublicId.eq(public_id))
        .filter(users::Column::EntraTenantId.eq(tenant))
        .one(db)
        .await?
        .ok_or(AppError::NotFound)?;
    describe(db, user).await
}

async fn target(
    tx: &DatabaseTransaction,
    tenant: Uuid,
    public_id: Uuid,
) -> Result<users::Model, AppError> {
    users::Entity::find()
        .filter(users::Column::PublicId.eq(public_id))
        .filter(users::Column::EntraTenantId.eq(tenant))
        .lock_exclusive()
        .one(tx)
        .await?
        .ok_or(AppError::NotFound)
}

async fn set_active(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    public_id: Uuid,
    active: bool,
) -> Result<UserResponse, AppError> {
    let (tx, admin_id) = crate::authorization::begin(db, actor_id, tenant, "users.update").await?;
    let user = target(&tx, tenant, public_id).await?;
    if !active {
        // Revoking on repeated calls also covers any session created by a
        // concurrent login before this transaction acquired the user lock.
        auth::session::revoke_user(&tx, user.id).await?;
    }
    let user = if user.is_active == active {
        user
    } else {
        let mut model = user.into_active_model();
        model.is_active = Set(active);
        model.updated_at = Set(chrono::Utc::now().fixed_offset());
        model.update(&tx).await?
    };
    crate::authorization::preserve_admin(&tx, tenant, admin_id).await?;
    let result = describe(&tx, user).await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn deactivate(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    public_id: Uuid,
) -> Result<UserResponse, AppError> {
    set_active(db, actor_id, tenant, public_id, false).await
}

pub async fn reactivate(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    public_id: Uuid,
) -> Result<UserResponse, AppError> {
    set_active(db, actor_id, tenant, public_id, true).await
}

pub async fn assign_role(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    public_id: Uuid,
    code: &str,
) -> Result<UserResponse, AppError> {
    let (tx, admin_id) =
        crate::authorization::begin(db, actor_id, tenant, "users.assign_role").await?;
    let user = target(&tx, tenant, public_id).await?;
    let role = roles::Entity::find()
        .filter(roles::Column::Code.eq(code))
        .filter(roles::Column::IsActive.eq(true))
        .one(&tx)
        .await?
        .ok_or_else(|| AppError::bad("El rol no existe o está retirado"))?;
    let user = if user.role_id == role.id {
        user
    } else {
        let mut update = user.into_active_model();
        update.role_id = Set(role.id);
        update.updated_at = Set(chrono::Utc::now().fixed_offset());
        let user = update.update(&tx).await?;
        crate::authorization::preserve_admin(&tx, tenant, admin_id).await?;
        auth::session::revoke_user(&tx, user.id).await?;
        user
    };
    let result = describe(&tx, user).await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn assign_department(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    public_id: Uuid,
    department_public_id: Option<Uuid>,
) -> Result<UserResponse, AppError> {
    let (tx, _) =
        crate::authorization::begin_admin(db, actor_id, tenant, "users.assign_department").await?;
    let user = target(&tx, tenant, public_id).await?;
    let department_id = match department_public_id {
        Some(public_id) => Some(
            departments::Entity::find()
                .filter(departments::Column::PublicId.eq(public_id))
                .one(&tx)
                .await?
                .ok_or_else(|| AppError::bad("El departamento no existe"))?
                .id,
        ),
        None => None,
    };
    let user = if user.department_id == department_id {
        user
    } else {
        let mut update = user.into_active_model();
        update.department_id = Set(department_id);
        update.updated_at = Set(chrono::Utc::now().fixed_offset());
        update.update(&tx).await?
    };
    let result = describe(&tx, user).await?;
    tx.commit().await?;
    Ok(result)
}
