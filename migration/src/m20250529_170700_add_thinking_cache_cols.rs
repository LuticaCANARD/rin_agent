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
                    .add_column_if_not_exists(
                        ColumnDef::new(TbDiscordAiContext::CacheKey)
                            .text()
                            .unique_key()
                        
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(TbDiscordAiContext::CacheCreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .add_column_if_not_exists(
                        ColumnDef::new(TbDiscordAiContext::CacheExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp())
                    )
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(TbDiscordAiContext::Table)
                .drop_column(TbDiscordAiContext::CacheKey)
                .drop_column(TbDiscordAiContext::CacheCreatedAt)
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
}
