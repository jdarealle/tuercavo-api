use common::error::AppError;
use sea_orm::{DatabaseConnection, DatabaseTransaction, IsolationLevel, TransactionTrait};
use uuid::Uuid;

pub(crate) async fn begin(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    permission: &str,
) -> Result<(DatabaseTransaction, i16), AppError> {
    let tx = db
        .begin_with_config(Some(IsolationLevel::ReadCommitted), None)
        .await?;
    let admin = auth::authorization::lock(&tx).await?;
    // The extractor ran before waiting on the lock. Recheck the current grants.
    let actor = auth::identity::load_user(&tx, actor_id).await?;
    if actor.tenant_id != tenant {
        return Err(auth::AuthError::Forbidden.into());
    }
    actor.require(permission)?;
    Ok((tx, admin.id))
}

pub(crate) async fn preserve_admin(
    tx: &DatabaseTransaction,
    tenant: Uuid,
    admin_id: i16,
) -> Result<(), AppError> {
    if auth::authorization::active_admins(tx, tenant, admin_id).await? == 0 {
        return Err(AppError::conflict(
            "Debe permanecer al menos un administrador activo",
        ));
    }
    Ok(())
}
