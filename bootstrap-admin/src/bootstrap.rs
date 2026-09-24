//! First administrator promotion, used only by the bootstrap-admin executable.
use entity::users;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, IntoActiveModel,
    IsolationLevel, PaginatorTrait, QueryFilter, QuerySelect, Set, TransactionTrait,
};
use uuid::Uuid;

use auth::authorization;

#[derive(Debug)]
pub enum BootstrapError {
    Refused(&'static str),
    Database,
}

impl std::fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Refused(message) => f.write_str(message),
            Self::Database => f.write_str("No se pudo completar la transacción de bootstrap"),
        }
    }
}
impl std::error::Error for BootstrapError {}
impl From<DbErr> for BootstrapError {
    fn from(_: DbErr) -> Self {
        Self::Database
    }
}

pub async fn bootstrap_admin(
    db: &DatabaseConnection,
    tenant: Uuid,
    object: Uuid,
) -> Result<Uuid, BootstrapError> {
    if tenant.is_nil() || object.is_nil() {
        return Err(BootstrapError::Refused(
            "Tenant ID y Object ID deben ser UUID no nulos",
        ));
    }
    let tx = db
        .begin_with_config(Some(IsolationLevel::ReadCommitted), None)
        .await?;
    let admin = authorization::lock(&tx).await?;
    if users::Entity::find()
        .filter(users::Column::EntraTenantId.eq(tenant))
        .filter(users::Column::RoleId.eq(admin.id))
        .count(&tx)
        .await?
        > 0
    {
        return Err(BootstrapError::Refused(
            "Ya existe un administrador en este tenant",
        ));
    }
    let user = users::Entity::find()
        .filter(users::Column::EntraTenantId.eq(tenant))
        .filter(users::Column::EntraObjectId.eq(object))
        .lock_exclusive()
        .one(&tx)
        .await?
        .ok_or(BootstrapError::Refused(
            "El usuario debe completar primero un login en Tuercavo",
        ))?;
    if !user.is_active {
        return Err(BootstrapError::Refused("El usuario está desactivado"));
    }
    let public_id = user.public_id;
    let user_id = user.id;
    let mut update = user.into_active_model();
    update.role_id = Set(admin.id);
    update.updated_at = Set(chrono::Utc::now().fixed_offset());
    update.update(&tx).await?;
    // Check the seeded administrator grants before making this bootstrap permanent.
    let principal = auth::identity::load_user(&tx, user_id)
        .await
        .map_err(|_| BootstrapError::Database)?;
    authorization::validate_grants(authorization::ADMIN_ROLE, &principal.permissions)
        .map_err(BootstrapError::Refused)?;
    auth::session::revoke_user(&tx, user_id).await?;
    tx.commit().await?;
    Ok(public_id)
}
