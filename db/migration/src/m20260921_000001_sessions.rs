use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // PostgreSQL executes this migration atomically through SeaORM's migrator.
        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .col(ColumnDef::new(Sessions::SessionIdHash).binary().not_null())
                    .col(ColumnDef::new(Sessions::UserId).big_integer().not_null())
                    .col(ColumnDef::new(Sessions::Issuer).string_len(512).not_null())
                    .col(ColumnDef::new(Sessions::Subject).string_len(255).not_null())
                    .col(ColumnDef::new(Sessions::TenantId).uuid().not_null())
                    .col(
                        ColumnDef::new(Sessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Sessions::LastSeenAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Sessions::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Sessions::RevokedAt).timestamp_with_time_zone())
                    .primary_key(
                        Index::create()
                            .name("pk_sessions")
                            .col(Sessions::SessionIdHash),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_sessions_user")
                            .from(Sessions::Table, Sessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .check((
                        "ck_sessions_hash_length",
                        Expr::expr(
                            Func::cust(Alias::new("octet_length"))
                                .arg(Expr::col(Sessions::SessionIdHash)),
                        )
                        .eq(32),
                    ))
                    .check(("ck_sessions_issuer", Expr::col(Sessions::Issuer).ne("")))
                    .check(("ck_sessions_subject", Expr::col(Sessions::Subject).ne("")))
                    .check((
                        "ck_sessions_expiration",
                        Expr::col(Sessions::ExpiresAt).gt(Expr::col(Sessions::CreatedAt)),
                    ))
                    .check((
                        "ck_sessions_activity",
                        Expr::col(Sessions::LastSeenAt).gte(Expr::col(Sessions::CreatedAt)),
                    ))
                    .check((
                        "ck_sessions_revocation",
                        Expr::col(Sessions::RevokedAt)
                            .is_null()
                            .or(Expr::col(Sessions::RevokedAt).gte(Expr::col(Sessions::CreatedAt))),
                    ))
                    .to_owned(),
            )
            .await?;

        for (name, column) in [
            ("idx_sessions_user_id", Sessions::UserId),
            ("idx_sessions_expires_at", Sessions::ExpiresAt),
            ("idx_sessions_last_seen_at", Sessions::LastSeenAt),
        ] {
            manager
                .create_index(
                    Index::create()
                        .name(name)
                        .table(Sessions::Table)
                        .col(column)
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Dropping this table also drops its indexes and constraints, not users.
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Sessions {
    Table,
    SessionIdHash,
    UserId,
    Issuer,
    Subject,
    TenantId,
    CreatedAt,
    LastSeenAt,
    ExpiresAt,
    RevokedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
