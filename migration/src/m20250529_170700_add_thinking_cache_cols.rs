use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(TbDiscordAiContext::Table)
                    .add_column(
                        ColumnDef::new(TbDiscordAiContext::CacheKey)
                            .string()
                            .not_null()
                            .unique_key()
                    )
                    .add_column(
                        ColumnDef::new(TbDiscordAiContext::CacheCreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .add_column(
                        ColumnDef::new(TbDiscordAiContext::CacheTTL)
                            .integer()
                            .not_null()
                            .default(3600)
                            .comment("seconds")
                    )
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(TbDiscordAiContext::Table)
                .drop_column(TbDiscordAiContext::ThinkingBought)
                .to_owned(),
        ).await
    }
}

#[derive(DeriveIden)]
enum TbDiscordAiContext {
    Table,
    Id,
    GuildId,
    RootMsg,
    UsingProModel,
    ParentContext,
    ThinkingBought,
    CacheKey,
    CacheExpiresAt,
    CacheCreatedAt,
    CacheTTL
}
