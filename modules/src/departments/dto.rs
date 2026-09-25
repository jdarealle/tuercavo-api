use entity::departments;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct DepartmentResponse {
    pub public_id: Uuid,
    pub name: String,
}

impl From<departments::Model> for DepartmentResponse {
    fn from(department: departments::Model) -> Self {
        Self {
            public_id: department.public_id,
            name: department.name,
        }
    }
}

#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateDepartment {
    pub name: String,
}
