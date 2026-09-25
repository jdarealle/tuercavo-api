use chrono::{DateTime, FixedOffset};
use common::patch::Patch;
use serde::{Deserialize, Serialize};
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
    pub department_public_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AssignRole {
    pub role: String,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AssignDepartment {
    #[serde(default)]
    #[schema(value_type = Option<Uuid>, required = true)]
    pub department_public_id: Patch<Uuid>,
}
