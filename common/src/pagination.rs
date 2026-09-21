use crate::error::AppError;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct Pagination {
    #[serde(default = "first_page")]
    #[param(minimum = 1, default = 1)]
    pub page: u64,
    #[serde(default = "page_size")]
    #[param(minimum = 1, maximum = 100, default = 20)]
    pub per_page: u64,
}
pub fn first_page() -> u64 {
    1
}
pub fn page_size() -> u64 {
    20
}
impl Pagination {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.page == 0
            || !(1..=100).contains(&self.per_page)
            || (self.page - 1).checked_mul(self.per_page).is_none()
        {
            return Err(AppError::bad("page debe ser >= 1 y per_page entre 1 y 100"));
        }
        Ok(())
    }
}
#[derive(Serialize, ToSchema)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}
impl<T> Page<T> {
    pub fn new(data: Vec<T>, total: u64, p: &Pagination) -> Self {
        Self {
            data,
            total,
            page: p.page,
            per_page: p.per_page,
            total_pages: total.div_ceil(p.per_page),
        }
    }
}
