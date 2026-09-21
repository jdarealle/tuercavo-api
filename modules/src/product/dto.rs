use crate::catalog::{Status, Unit};
use chrono::{DateTime, FixedOffset};
use common::{error::AppError, pagination::Pagination, patch::Patch};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateProduct {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub category: Uuid,
    pub supplier: Option<Uuid>,
    pub unit: Unit,
    #[schema(example = "125.50")]
    pub price: String,
    pub status: Option<Status>,
}
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateProduct {
    #[serde(default)]
    #[schema(value_type = String, required = false)]
    pub sku: Patch<String>,
    #[serde(default)]
    #[schema(value_type = String, required = false)]
    pub name: Patch<String>,
    #[serde(default)]
    #[schema(value_type = Option<String>, required = false)]
    pub description: Patch<String>,
    #[serde(default)]
    #[schema(value_type = Option<String>, required = false)]
    pub brand: Patch<String>,
    #[serde(default)]
    #[schema(value_type = Uuid, required = false)]
    pub category: Patch<Uuid>,
    #[serde(default)]
    #[schema(value_type = Option<Uuid>, required = false)]
    pub supplier: Patch<Uuid>,
    #[serde(default)]
    #[schema(value_type = Unit, required = false)]
    pub unit: Patch<Unit>,
    #[serde(default)]
    #[schema(value_type = String, required = false, example = "125.50")]
    pub price: Patch<String>,
    #[serde(default)]
    #[schema(value_type = Status, required = false)]
    pub status: Patch<Status>,
}
#[derive(Serialize, ToSchema)]
pub struct ProductResponse {
    pub public_id: Uuid,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub brand: Option<String>,
    pub category: Uuid,
    pub supplier: Option<Uuid>,
    pub unit: Unit,
    pub price: String,
    pub status: Status,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct ProductFilters {
    #[serde(default = "common::pagination::first_page")]
    #[param(minimum = 1, default = 1)]
    pub page: u64,
    #[serde(default = "common::pagination::page_size")]
    #[param(minimum = 1, maximum = 100, default = 20)]
    pub per_page: u64,
    pub search: Option<String>,
    pub status: Option<Status>,
    pub category: Option<Uuid>,
    pub supplier: Option<Uuid>,
}
impl ProductFilters {
    pub fn pagination(&self) -> Result<Pagination, AppError> {
        let p = Pagination {
            page: self.page,
            per_page: self.per_page,
        };
        p.validate()?;
        Ok(p)
    }
}
