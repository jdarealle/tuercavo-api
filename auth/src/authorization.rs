//! Shared transaction protocol for local authorization changes and first login.
use entity::{roles, users};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseTransaction, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QuerySelect,
};
use uuid::Uuid;

pub const ADMIN_ROLE: &str = "admin";
pub const DEFAULT_ROLE: &str = "consultor";
pub const ADMIN_REQUIRED: &[&str] = &[
    "users.read",
    "users.update",
    "users.assign_role",
    "departments.read",
    "departments.create",
    "users.assign_department",
    "roles.read",
    "roles.create",
    "roles.update",
    "roles.assign_permissions",
    "permissions.read",
];
pub const DEFAULT_ALLOWED: &[&str] = &["products.read", "categories.read", "suppliers.read"];
pub const ADMIN_ONLY: &[&str] = &[
    "departments.read",
    "departments.create",
    "users.assign_department",
];

pub fn is_system_role(code: &str) -> bool {
    matches!(code, ADMIN_ROLE | DEFAULT_ROLE)
}

pub fn validate_grants(code: &str, grants: &[String]) -> Result<(), &'static str> {
    if code == ADMIN_ROLE
        && ADMIN_REQUIRED
            .iter()
            .any(|required| !grants.iter().any(|g| g == required))
    {
        return Err("admin debe conservar los permisos de administración");
    }
    if code == DEFAULT_ROLE
        && grants
            .iter()
            .any(|g| !DEFAULT_ALLOWED.contains(&g.as_str()))
    {
        return Err("consultor solo admite permisos de lectura del catálogo");
    }
    if code != ADMIN_ROLE && grants.iter().any(|g| ADMIN_ONLY.contains(&g.as_str())) {
        return Err("los permisos de departamentos son exclusivos de admin");
    }
    Ok(())
}

// Always acquire this lock BEFORE user/session locks and before checking actor
// permissions. Serializes RBAC writers across API replicas and bootstrap jobs.
pub async fn lock(tx: &DatabaseTransaction) -> Result<roles::Model, DbErr> {
    roles::Entity::find()
        .filter(roles::Column::Code.eq(ADMIN_ROLE))
        .lock_exclusive()
        .one(tx)
        .await?
        .ok_or_else(|| DbErr::Custom("Missing system role".into()))
}

// Concurrent logins may proceed together; none can create a session while an
// authorization change is revoking sessions or retiring a role.
pub(crate) async fn login_lock(tx: &DatabaseTransaction) -> Result<(), DbErr> {
    roles::Entity::find()
        .filter(roles::Column::Code.eq(ADMIN_ROLE))
        .lock_shared()
        .one(tx)
        .await?
        .ok_or_else(|| DbErr::Custom("Missing system role".into()))?;
    Ok(())
}

pub async fn active_admins<C: ConnectionTrait>(
    db: &C,
    tenant: Uuid,
    admin_role_id: i16,
) -> Result<u64, DbErr> {
    users::Entity::find()
        .filter(users::Column::EntraTenantId.eq(tenant))
        .filter(users::Column::RoleId.eq(admin_role_id))
        .filter(users::Column::IsActive.eq(true))
        .count(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_role_cannot_gain_write_or_administration_permissions() {
        for permission in [
            "products.create",
            "users.read",
            "departments.read",
            "users.assign_department",
            "roles.assign_permissions",
        ] {
            assert!(validate_grants(DEFAULT_ROLE, &[permission.into()]).is_err());
        }
        assert!(validate_grants(DEFAULT_ROLE, &["products.read".into()]).is_ok());
        assert!(validate_grants(DEFAULT_ROLE, &[]).is_ok());
    }

    #[test]
    fn admin_must_keep_control_permissions_but_custom_roles_are_dynamic() {
        let all: Vec<String> = ADMIN_REQUIRED.iter().map(|p| (*p).into()).collect();
        assert!(validate_grants(ADMIN_ROLE, &all).is_ok());
        for missing in 0..all.len() {
            let mut grants = all.clone();
            grants.remove(missing);
            assert!(validate_grants(ADMIN_ROLE, &grants).is_err());
        }
        assert!(validate_grants("inventario", &["products.update".into()]).is_ok());
        for permission in ADMIN_ONLY {
            assert!(validate_grants("inventario", &[(*permission).into()]).is_err());
        }
    }
}
