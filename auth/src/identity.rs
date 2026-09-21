use entity::{permissions, role_permissions, roles, users};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::AuthError;

#[derive(Clone, Serialize, ToSchema)]
pub struct Principal {
    #[serde(skip)]
    #[schema(ignore)]
    pub user_id: i64,
    pub public_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub tenant_id: Uuid,
    pub object_id: Uuid,
    pub role: String,
    pub permissions: Vec<String>,
}

impl Principal {
    pub fn require(&self, permission: &str) -> Result<(), AuthError> {
        if self.permissions.iter().any(|value| value == permission) {
            Ok(())
        } else {
            Err(AuthError::Forbidden)
        }
    }
}

pub async fn load_user<C: ConnectionTrait>(db: &C, id: i64) -> Result<Principal, AuthError> {
    let user = users::Entity::find_by_id(id)
        .filter(users::Column::IsActive.eq(true))
        .one(db)
        .await?
        .ok_or(AuthError::Forbidden)?;
    let role = roles::Entity::find_by_id(user.role_id)
        .one(db)
        .await?
        .ok_or(AuthError::Forbidden)?;
    let grants = role_permissions::Entity::find()
        .filter(role_permissions::Column::RoleId.eq(role.id))
        .all(db)
        .await?;
    let permissions = permissions::Entity::find()
        .filter(permissions::Column::Id.is_in(grants.into_iter().map(|grant| grant.permission_id)))
        .order_by_asc(permissions::Column::Code)
        .all(db)
        .await?
        .into_iter()
        .map(|permission| permission.code)
        .collect();
    Ok(Principal {
        user_id: user.id,
        public_id: user.public_id,
        email: user.email,
        full_name: user.full_name,
        tenant_id: user.entra_tenant_id,
        object_id: user.entra_object_id,
        role: role.code,
        permissions,
    })
}
