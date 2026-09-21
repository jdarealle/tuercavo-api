use crate::catalog::Status;
use chrono::{DateTime, FixedOffset};
use common::patch::Patch;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateSupplier {
    pub code: String,
    pub name: String,
    pub contact_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<Status>,
}
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateSupplier {
    #[serde(default)]
    #[schema(value_type = String, required = false)]
    pub code: Patch<String>,
    #[serde(default)]
    #[schema(value_type = String, required = false)]
    pub name: Patch<String>,
    #[serde(default)]
    #[schema(value_type = Option<String>, required = false)]
    pub contact_name: Patch<String>,
    #[serde(default)]
    #[schema(value_type = Option<String>, required = false)]
    pub email: Patch<String>,
    #[serde(default)]
    #[schema(value_type = Option<String>, required = false)]
    pub phone: Patch<String>,
    #[serde(default)]
    #[schema(value_type = Status, required = false)]
    pub status: Patch<Status>,
}
#[derive(Serialize, ToSchema)]
pub struct SupplierResponse {
    pub public_id: Uuid,
    pub code: String,
    pub name: String,
    pub contact_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Status,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}
impl From<entity::suppliers::Model> for SupplierResponse {
    fn from(m: entity::suppliers::Model) -> Self {
        Self {
            public_id: m.public_id,
            code: m.code,
            name: m.name,
            contact_name: m.contact_name,
            email: m.email,
            phone: m.phone,
            status: m.status.into(),
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}
