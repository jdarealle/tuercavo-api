use common::{error::AppError, pagination::Pagination};
use entity::sea_orm_active_enums::CatalogStatus;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Inactive,
    Archived,
}
impl From<Status> for CatalogStatus {
    fn from(s: Status) -> Self {
        match s {
            Status::Active => Self::Active,
            Status::Inactive => Self::Inactive,
            Status::Archived => Self::Archived,
        }
    }
}
impl From<CatalogStatus> for Status {
    fn from(s: CatalogStatus) -> Self {
        match s {
            CatalogStatus::Active => Self::Active,
            CatalogStatus::Inactive => Self::Inactive,
            CatalogStatus::Archived => Self::Archived,
        }
    }
}
#[derive(Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Piece,
    Box,
    Pack,
    Meter,
    Liter,
    Kg,
}
impl Unit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Piece => "piece",
            Self::Box => "box",
            Self::Pack => "pack",
            Self::Meter => "meter",
            Self::Liter => "liter",
            Self::Kg => "kg",
        }
    }
    pub fn parse(s: &str) -> Result<Self, AppError> {
        match s {
            "piece" => Ok(Self::Piece),
            "box" => Ok(Self::Box),
            "pack" => Ok(Self::Pack),
            "meter" => Ok(Self::Meter),
            "liter" => Ok(Self::Liter),
            "kg" => Ok(Self::Kg),
            _ => Err(AppError::Internal),
        }
    }
}
#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct Filters {
    #[serde(default = "common::pagination::first_page")]
    #[param(minimum = 1, default = 1)]
    pub page: u64,
    #[serde(default = "common::pagination::page_size")]
    #[param(minimum = 1, maximum = 100, default = 20)]
    pub per_page: u64,
    pub search: Option<String>,
    pub status: Option<Status>,
}
impl Filters {
    pub fn pagination(&self) -> Result<Pagination, AppError> {
        let p = Pagination {
            page: self.page,
            per_page: self.per_page,
        };
        p.validate()?;
        Ok(p)
    }
}
