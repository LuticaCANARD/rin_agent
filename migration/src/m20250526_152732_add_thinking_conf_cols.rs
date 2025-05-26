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
                        ColumnDef::new(TbDiscordAiContext::ThinkingBought)
                            .integer()
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
    ThinkingBought
}
