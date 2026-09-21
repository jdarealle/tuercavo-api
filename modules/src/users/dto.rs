use chrono::{DateTime, FixedOffset};
use common::patch::Patch;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoleCode {
    Admin,
    Capturista,
    Consultor,
}

impl RoleCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::Capturista => "capturista",
            Self::Consultor => "consultor",
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateUser {
    pub entra_tenant_id: Uuid,
    pub entra_object_id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role: RoleCode,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateUser {
    #[serde(default)]
    #[schema(value_type = String, required = false)]
    pub email: Patch<String>,
    #[serde(default)]
    #[schema(value_type = String, required = false)]
    pub full_name: Patch<String>,
    #[serde(default)]
    #[schema(value_type = bool, required = false)]
    pub is_active: Patch<bool>,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AssignRole {
    pub role: RoleCode,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub public_id: Uuid,
    pub entra_tenant_id: Uuid,
    pub entra_object_id: Uuid,
    pub email: String,
    pub full_name: String,
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
