use crate::catalog::Status;
use chrono::{DateTime, FixedOffset};
use common::patch::Patch;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateCategory {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<Status>,
}
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateCategory {
    #[serde(default)]
    #[schema(value_type = String, required = false)]
    pub name: Patch<String>,
    #[serde(default)]
    #[schema(value_type = Option<String>, required = false)]
    pub description: Patch<String>,
    #[serde(default)]
    #[schema(value_type = Status, required = false)]
    pub status: Patch<Status>,
}
#[derive(Serialize, ToSchema)]
pub struct CategoryResponse {
    pub public_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: Status,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}
impl From<entity::categories::Model> for CategoryResponse {
    fn from(m: entity::categories::Model) -> Self {
        Self {
            public_id: m.public_id,
            name: m.name,
            description: m.description,
            status: m.status.into(),
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}
