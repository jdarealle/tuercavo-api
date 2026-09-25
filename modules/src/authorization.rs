use common::error::AppError;
use sea_orm::{DatabaseConnection, DatabaseTransaction, IsolationLevel, TransactionTrait};
use uuid::Uuid;

pub(crate) async fn begin(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    permission: &str,
) -> Result<(DatabaseTransaction, i16), AppError> {
    begin_with_role(db, actor_id, tenant, permission, false).await
}

pub(crate) async fn begin_admin(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    permission: &str,
) -> Result<(DatabaseTransaction, i16), AppError> {
    begin_with_role(db, actor_id, tenant, permission, true).await
}

pub(crate) fn require_admin(actor: &auth::Principal) -> Result<(), AppError> {
    if actor.role == auth::authorization::ADMIN_ROLE {
        Ok(())
    } else {
        Err(auth::AuthError::Forbidden.into())
    }
}

async fn begin_with_role(
    db: &DatabaseConnection,
    actor_id: i64,
    tenant: Uuid,
    permission: &str,
    admin_only: bool,
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
    if admin_only {
        require_admin(&actor)?;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn principal(role: &str) -> auth::Principal {
        auth::Principal {
            user_id: 1,
            public_id: Uuid::from_u128(1),
            email: None,
            full_name: None,
            department_public_id: None,
            tenant_id: Uuid::from_u128(2),
            object_id: Uuid::from_u128(3),
            role: role.into(),
            permissions: vec![
                "departments.read".into(),
                "departments.create".into(),
                "users.assign_department".into(),
            ],
        }
    }

    #[test]
    fn department_management_requires_admin_role_even_with_department_permissions() {
        assert!(matches!(
            require_admin(&principal("gerente")),
            Err(AppError::Auth(auth::AuthError::Forbidden))
        ));
        assert!(require_admin(&principal(auth::authorization::ADMIN_ROLE)).is_ok());
    }
}
