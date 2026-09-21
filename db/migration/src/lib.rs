use sea_orm_migration::prelude::*;

mod m20260917_000001_types;
mod m20260917_000002_rbac;
mod m20260917_000003_users;
mod m20260917_000004_categories;
mod m20260917_000005_suppliers;
mod m20260917_000006_products;
mod m20260917_000007_indexes;
mod m20260917_000008_reference_data;
mod m20260921_000001_sessions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260917_000001_types::Migration),
            Box::new(m20260917_000002_rbac::Migration),
            Box::new(m20260917_000003_users::Migration),
            Box::new(m20260917_000004_categories::Migration),
            Box::new(m20260917_000005_suppliers::Migration),
            Box::new(m20260917_000006_products::Migration),
            Box::new(m20260917_000007_indexes::Migration),
            Box::new(m20260917_000008_reference_data::Migration),
            Box::new(m20260921_000001_sessions::Migration),
        ]
    }
}
