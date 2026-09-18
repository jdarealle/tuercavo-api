use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(Suppliers::Table)
            .col(ColumnDef::new(Suppliers::Id).integer().extra("GENERATED ALWAYS AS IDENTITY"))
            .col(ColumnDef::new(Suppliers::PublicId).uuid().not_null().default(Expr::cust("gen_random_uuid()")))
            .col(ColumnDef::new(Suppliers::Code).custom("citext").not_null())
            .col(ColumnDef::new(Suppliers::Name).string_len(150).not_null())
            .col(ColumnDef::new(Suppliers::ContactName).string_len(150))
            .col(ColumnDef::new(Suppliers::Email).custom("citext"))
            .col(ColumnDef::new(Suppliers::Phone).string_len(32))
            .col(ColumnDef::new(Suppliers::Status).custom("catalog_status").not_null().default("active"))
            .col(ColumnDef::new(Suppliers::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .col(ColumnDef::new(Suppliers::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .primary_key(Index::create().name("pk_suppliers").col(Suppliers::Id))
            .index(Index::create().name("uq_suppliers_public_id").unique().col(Suppliers::PublicId))
            .index(Index::create().name("uq_suppliers_code").unique().col(Suppliers::Code))
            .check(("ck_suppliers_code", Expr::cust(r#"char_length(code) BETWEEN 1 AND 64 AND code::text ~ '^[A-Za-z0-9][A-Za-z0-9._-]*$'"#)))
            .check(("ck_suppliers_name", Expr::cust(r#"char_length(name) BETWEEN 1 AND 150 AND name = btrim(name) AND name !~ '[[:cntrl:]]'"#)))
            .check(("ck_suppliers_contact_name", Expr::cust(r#"char_length(contact_name) BETWEEN 1 AND 150 AND contact_name = btrim(contact_name) AND contact_name !~ '[[:cntrl:]]'"#)))
            .check(("ck_suppliers_email", Expr::cust(r#"char_length(email) BETWEEN 1 AND 254 AND email::text = btrim(email::text) AND email::text !~ '[[:space:][:cntrl:]]'"#)))
            .check(("ck_suppliers_phone", Expr::cust(r#"char_length(phone) BETWEEN 1 AND 32 AND phone = btrim(phone) AND phone !~ '[[:cntrl:]]'"#)))
            .to_owned()).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Suppliers::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Suppliers {
    Table,
    Id,
    PublicId,
    Code,
    Name,
    ContactName,
    Email,
    Phone,
    Status,
    CreatedAt,
    UpdatedAt,
}
