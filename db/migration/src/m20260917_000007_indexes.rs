use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_index(
                Index::create()
                    .name("idx_role_permissions_permission_id")
                    .table(RolePermissions::Table)
                    .col(RolePermissions::PermissionId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_users_role_id")
                    .table(Users::Table)
                    .col(Users::RoleId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_products_category_id")
                    .table(Products::Table)
                    .col(Products::CategoryId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_products_supplier_id")
                    .table(Products::Table)
                    .col(Products::SupplierId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for name in [
            "idx_products_supplier_id",
            "idx_products_category_id",
            "idx_users_role_id",
            "idx_role_permissions_permission_id",
        ] {
            manager
                .drop_index(Index::drop().name(name).to_owned())
                .await?;
        }
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Products {
    Table,
    CategoryId,
    SupplierId,
}

#[derive(DeriveIden)]
enum RolePermissions {
    Table,
    PermissionId,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    RoleId,
}
