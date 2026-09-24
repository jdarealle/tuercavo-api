use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub public_id: Uuid,
    pub entra_tenant_id: Uuid,
    pub entra_object_id: Uuid,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub role: String,
    pub is_active: bool,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Serialize, ToSchema)]
pub struct RoleResponse {
    pub code: String,
    pub name: String,
}

#[derive(Serialize, ToSchema)]
pub struct PermissionResponse {
    pub code: String,
    pub description: String,
}
