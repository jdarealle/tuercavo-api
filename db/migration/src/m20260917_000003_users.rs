use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(Users::Table)
            .col(ColumnDef::new(Users::Id).big_integer().extra("GENERATED ALWAYS AS IDENTITY"))
            .col(ColumnDef::new(Users::PublicId).uuid().not_null().default(Expr::cust("gen_random_uuid()")))
            .col(ColumnDef::new(Users::RoleId).small_integer().not_null())
            .col(ColumnDef::new(Users::EntraTenantId).uuid().not_null())
            .col(ColumnDef::new(Users::EntraObjectId).uuid().not_null())
            .col(ColumnDef::new(Users::Email).custom("citext").not_null())
            .col(ColumnDef::new(Users::FullName).string_len(150).not_null())
            .col(ColumnDef::new(Users::IsActive).boolean().not_null().default(true))
            .col(ColumnDef::new(Users::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .col(ColumnDef::new(Users::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .primary_key(Index::create().name("pk_users").col(Users::Id))
            .index(Index::create().name("uq_users_public_id").unique().col(Users::PublicId))
            .index(Index::create().name("uq_users_entra_identity").unique().col(Users::EntraTenantId).col(Users::EntraObjectId))
            .foreign_key(ForeignKey::create().name("fk_users_role").from(Users::Table, Users::RoleId).to(Roles::Table, Roles::Id).on_delete(ForeignKeyAction::Restrict))
            .check(("ck_users_email", Expr::cust(r#"char_length(email) BETWEEN 1 AND 254 AND email::text = btrim(email::text) AND email::text !~ '[[:space:][:cntrl:]]'"#)))
            .check(("ck_users_full_name", Expr::cust(r#"char_length(full_name) BETWEEN 1 AND 150 AND full_name = btrim(full_name) AND full_name !~ '[[:cntrl:]]'"#)))
            .to_owned()).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Roles {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    PublicId,
    RoleId,
    EntraTenantId,
    EntraObjectId,
    Email,
    FullName,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
