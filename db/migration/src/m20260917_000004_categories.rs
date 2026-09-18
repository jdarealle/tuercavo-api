use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(Categories::Table)
            .col(ColumnDef::new(Categories::Id).integer().extra("GENERATED ALWAYS AS IDENTITY"))
            .col(ColumnDef::new(Categories::PublicId).uuid().not_null().default(Expr::cust("gen_random_uuid()")))
            .col(ColumnDef::new(Categories::Name).custom("citext").not_null())
            .col(ColumnDef::new(Categories::Description).text())
            .col(ColumnDef::new(Categories::Status).custom("catalog_status").not_null().default("active"))
            .col(ColumnDef::new(Categories::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .col(ColumnDef::new(Categories::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .primary_key(Index::create().name("pk_categories").col(Categories::Id))
            .index(Index::create().name("uq_categories_public_id").unique().col(Categories::PublicId))
            .index(Index::create().name("uq_categories_name").unique().col(Categories::Name))
            .check(("ck_categories_name", Expr::cust(r#"char_length(name) BETWEEN 1 AND 150 AND name::text = btrim(name::text) AND name::text !~ '[[:cntrl:]]'"#)))
            .check(("ck_categories_description", Expr::cust(r#"char_length(description) BETWEEN 1 AND 4000 AND description = btrim(description) AND description !~ '[[:cntrl:]]'"#)))
            .to_owned()).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
    PublicId,
    Name,
    Description,
    Status,
    CreatedAt,
    UpdatedAt,
}
