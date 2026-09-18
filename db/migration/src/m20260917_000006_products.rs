use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(Products::Table)
            .col(ColumnDef::new(Products::Id).big_integer().extra("GENERATED ALWAYS AS IDENTITY"))
            .col(ColumnDef::new(Products::PublicId).uuid().not_null().default(Expr::cust("gen_random_uuid()")))
            .col(ColumnDef::new(Products::Sku).string_len(64).not_null())
            .col(ColumnDef::new(Products::Name).string_len(150).not_null())
            .col(ColumnDef::new(Products::Description).text())
            .col(ColumnDef::new(Products::Brand).string_len(100))
            .col(ColumnDef::new(Products::CategoryId).integer().not_null())
            .col(ColumnDef::new(Products::SupplierId).integer())
            .col(ColumnDef::new(Products::Unit).string_len(16).not_null())
            .col(ColumnDef::new(Products::Price).decimal_len(12, 2).not_null())
            .col(ColumnDef::new(Products::Status).custom("catalog_status").not_null().default("active"))
            .col(ColumnDef::new(Products::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .col(ColumnDef::new(Products::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .primary_key(Index::create().name("pk_products").col(Products::Id))
            .index(Index::create().name("uq_products_public_id").unique().col(Products::PublicId))
            .foreign_key(ForeignKey::create().name("fk_products_category").from(Products::Table, Products::CategoryId).to(Categories::Table, Categories::Id).on_delete(ForeignKeyAction::Restrict))
            .foreign_key(ForeignKey::create().name("fk_products_supplier").from(Products::Table, Products::SupplierId).to(Suppliers::Table, Suppliers::Id).on_delete(ForeignKeyAction::Restrict))
            .check(("ck_products_sku", Expr::cust(r#"char_length(sku) BETWEEN 1 AND 64 AND sku ~ '^[A-Za-z0-9][A-Za-z0-9._-]*$'"#)))
            .check(("ck_products_name", Expr::cust(r#"char_length(name) BETWEEN 1 AND 150 AND name = btrim(name) AND name !~ '[[:cntrl:]]'"#)))
            .check(("ck_products_description", Expr::cust(r#"char_length(description) BETWEEN 1 AND 4000 AND description = btrim(description) AND description !~ '[[:cntrl:]]'"#)))
            .check(("ck_products_brand", Expr::cust(r#"char_length(brand) BETWEEN 1 AND 100 AND brand = btrim(brand) AND brand !~ '[[:cntrl:]]'"#)))
            .check(("ck_products_unit", Expr::cust(r#"unit IN ('piece', 'box', 'pack', 'meter', 'liter', 'kg')"#)))
            .check(("ck_products_price", Expr::cust(r#"price >= 0 AND price <> 'NaN'::numeric"#)))
            .to_owned()).await?;
        manager
            .create_index(
                Index::create()
                    .name("uq_products_sku")
                    .table(Products::Table)
                    .col(Func::lower(Expr::col(Products::Sku)))
                    .unique()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Products::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Products {
    Table,
    Id,
    PublicId,
    Sku,
    Name,
    Description,
    Brand,
    CategoryId,
    SupplierId,
    Unit,
    Price,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Suppliers {
    Table,
    Id,
}
