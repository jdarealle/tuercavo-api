use common::patch::Patch;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateRole {
    pub code: String,
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateRole {
    #[serde(default)]
    #[schema(value_type = String, required = false)]
    pub name: Patch<String>,
    #[serde(default)]
    #[schema(value_type = bool, required = false)]
    pub is_active: Patch<bool>,
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SetPermissions {
    pub permissions: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct RoleResponse {
    pub code: String,
    pub name: String,
    pub is_active: bool,
    pub is_system: bool,
    pub permissions: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct PermissionResponse {
    pub code: String,
    pub description: String,
}
