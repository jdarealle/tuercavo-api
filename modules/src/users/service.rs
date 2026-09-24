use super::dto::*;
use common::{
    error::AppError,
    pagination::{Page, Pagination},
};
use entity::{permissions, roles, users};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};
use uuid::Uuid;

fn response(user: users::Model, role: String) -> UserResponse {
    UserResponse {
        public_id: user.public_id,
        entra_tenant_id: user.entra_tenant_id,
        entra_object_id: user.entra_object_id,
        email: user.email,
        full_name: user.full_name,
        role,
        is_active: user.is_active,
        created_at: user.created_at,
        updated_at: user.updated_at,
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
    Ok(response(user, role.code))
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
    let rows = query
        .fetch_page(p.page - 1)
        .await?
        .into_iter()
        .map(|user| {
            let role = roles.get(&user.role_id).ok_or(AppError::Internal)?.clone();
            Ok(response(user, role))
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

// Serialize changes to the active admin set before rechecking the actor's permissions.
async fn lock_admin(tx: &DatabaseTransaction) -> Result<i16, AppError> {
    Ok(roles::Entity::find()
        .filter(roles::Column::Code.eq("admin"))
        .lock_exclusive()
        .one(tx)
        .await?
        .ok_or(AppError::Internal)?
        .id)
}

async fn actor<C: ConnectionTrait>(
    db: &C,
    actor_id: i64,
    tenant: Uuid,
    permission: &str,
) -> Result<auth::Principal, AppError> {
    let actor = auth::identity::load_user(db, actor_id).await?;
    if actor.tenant_id != tenant {
        return Err(auth::AuthError::Forbidden.into());
    }
    actor.require(permission)?;
    Ok(actor)
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

async fn protect_last_admin(
    tx: &DatabaseTransaction,
    user: &users::Model,
    admin_id: i16,
    new_role: i16,
    active: bool,
) -> Result<(), AppError> {
    if user.is_active && user.role_id == admin_id && (!active || new_role != admin_id) {
        let others = users::Entity::find()
            .filter(users::Column::RoleId.eq(admin_id))
            .filter(users::Column::EntraTenantId.eq(user.entra_tenant_id))
            .filter(users::Column::IsActive.eq(true))
            .filter(users::Column::Id.ne(user.id))
            .count(tx)
            .await?;
        if others == 0 {
            return Err(AppError::conflict(
                "No se puede desactivar ni degradar al último administrador activo",
            ));
        }
    }
    Ok(())
}

pub async fn update(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    public_id: Uuid,
    input: UpdateUser,
) -> Result<UserResponse, AppError> {
    let active = input.is_active.required("is_active")?;
    let tx = db.begin().await?;
    let admin_id = lock_admin(&tx).await?;
    actor(&tx, actor_id, tenant, "users.update").await?;
    let user = target(&tx, tenant, public_id).await?;
    protect_last_admin(
        &tx,
        &user,
        admin_id,
        user.role_id,
        active.unwrap_or(user.is_active),
    )
    .await?;
    if active == Some(false) {
        auth::session::revoke_user(&tx, user.id).await?;
    }
    let mut user = user.into_active_model();
    if let Some(active) = active {
        user.is_active = Set(active);
    }
    user.updated_at = Set(chrono::Utc::now().fixed_offset());
    let result = describe(&tx, user.update(&tx).await?).await?;
    tx.commit().await?;
    Ok(result)
}

pub async fn roles(db: &DatabaseConnection) -> Result<Vec<RoleResponse>, AppError> {
    Ok(roles::Entity::find()
        .order_by_asc(roles::Column::Code)
        .all(db)
        .await?
        .into_iter()
        .map(|role| RoleResponse {
            code: role.code,
            name: role.name,
        })
        .collect())
}

pub async fn permissions(db: &DatabaseConnection) -> Result<Vec<PermissionResponse>, AppError> {
    Ok(permissions::Entity::find()
        .order_by_asc(permissions::Column::Code)
        .all(db)
        .await?
        .into_iter()
        .map(|permission| PermissionResponse {
            code: permission.code,
            description: permission.description,
        })
        .collect())
}
