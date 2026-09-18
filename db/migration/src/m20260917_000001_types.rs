use sea_orm_migration::{
    prelude::*,
    sea_query::extension::postgres::{Extension, Type},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // SeaQuery ofrece el builder; SchemaManager no tiene create_extension.
        let extension = Extension::create()
            .name("citext")
            .to_string(PostgresQueryBuilder);
        manager
            .get_connection()
            .execute_unprepared(&extension)
            .await?;
        manager
            .create_type(
                Type::create()
                    .as_enum("catalog_status")
                    .values(["active", "inactive", "archived"])
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_type(Type::drop().name("catalog_status").to_owned())
            .await?;
        let extension = Extension::drop()
            .name("citext")
            .to_string(PostgresQueryBuilder);
        manager
            .get_connection()
            .execute_unprepared(&extension)
            .await?;
        Ok(())
    }
}
