use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(Roles::Table)
            .col(ColumnDef::new(Roles::Id).small_integer().extra("GENERATED ALWAYS AS IDENTITY"))
            .col(ColumnDef::new(Roles::Code).string_len(32).not_null())
            .col(ColumnDef::new(Roles::Name).string_len(80).not_null())
            .primary_key(Index::create().name("pk_roles").col(Roles::Id))
            .index(Index::create().name("uq_roles_code").unique().col(Roles::Code))
            .check(("ck_roles_code", Expr::cust(r#"code IN ('admin', 'capturista', 'consultor')"#)))
            .check(("ck_roles_name", Expr::cust(r#"char_length(name) BETWEEN 1 AND 80 AND name = btrim(name) AND name !~ '[[:cntrl:]]'"#)))
            .to_owned()).await?;
        manager.create_table(Table::create().table(Permissions::Table)
            .col(ColumnDef::new(Permissions::Id).small_integer().extra("GENERATED ALWAYS AS IDENTITY"))
            .col(ColumnDef::new(Permissions::Code).string_len(80).not_null())
            .col(ColumnDef::new(Permissions::Description).string_len(200).not_null())
            .primary_key(Index::create().name("pk_permissions").col(Permissions::Id))
            .index(Index::create().name("uq_permissions_code").unique().col(Permissions::Code))
            .check(("ck_permissions_code", Expr::cust(r#"code ~ '^[a-z]+\.[a-z]+(_[a-z]+)*$'"#)))
            .check(("ck_permissions_description", Expr::cust(r#"char_length(description) BETWEEN 1 AND 200 AND description = btrim(description) AND description !~ '[[:cntrl:]]'"#)))
            .to_owned()).await?;
        manager
            .create_table(
                Table::create()
                    .table(RolePermissions::Table)
                    .col(
                        ColumnDef::new(RolePermissions::RoleId)
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RolePermissions::PermissionId)
                            .small_integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_role_permissions")
                            .col(RolePermissions::RoleId)
                            .col(RolePermissions::PermissionId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_role_permissions_role")
                            .from(RolePermissions::Table, RolePermissions::RoleId)
                            .to(Roles::Table, Roles::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_role_permissions_permission")
                            .from(RolePermissions::Table, RolePermissions::PermissionId)
                            .to(Permissions::Table, Permissions::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RolePermissions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Permissions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Permissions {
    Table,
    Id,
    Code,
    Description,
}

#[derive(DeriveIden)]
enum RolePermissions {
    Table,
    RoleId,
    PermissionId,
}

#[derive(DeriveIden)]
enum Roles {
    Table,
    Id,
    Code,
    Name,
}
