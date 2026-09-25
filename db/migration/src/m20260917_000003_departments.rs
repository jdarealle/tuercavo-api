use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(Departments::Table)
            .col(ColumnDef::new(Departments::Id).integer().extra("GENERATED ALWAYS AS IDENTITY"))
            .col(ColumnDef::new(Departments::Name).string_len(150).not_null())
            .primary_key(Index::create().name("pk_departments").col(Departments::Id))
            .check(("ck_departments_name", Expr::cust(r#"char_length(name) BETWEEN 1 AND 150 AND name = btrim(name) AND name !~ '[[:cntrl:]]'"#)))
            .to_owned()).await?;
        manager
            .create_index(
                Index::create()
                    .name("uq_departments_name")
                    .table(Departments::Table)
                    .col(Func::lower(Expr::col(Departments::Name)))
                    .unique()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Departments::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Departments {
    Table,
    Id,
    Name,
}
