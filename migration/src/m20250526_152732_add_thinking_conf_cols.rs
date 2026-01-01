use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Ensure table exists first
        manager
            .create_table(
                Table::create()
                    .table(TbDiscordAiContext::Table)
                    .if_not_exists()
                    .col(pk_auto(TbDiscordAiContext::Id))
                    .col(ColumnDef::new(TbDiscordAiContext::GuildId).big_integer().not_null())
                    .col(ColumnDef::new(TbDiscordAiContext::RootMsg).big_integer().not_null())
                    .col(
                        ColumnDef::new(TbDiscordAiContext::UsingProModel)
                        .boolean()
                        .null()
                    )
                    .col(ColumnDef::new(TbDiscordAiContext::ParentContext).array(ColumnType::BigInteger).not_null())
                    .to_owned(),
            )
            .await?;

        // Then add the new column
        // We check if the column exists or just try to add it. 
        // Since alter_table doesn't have if_not_exists for columns easily in all backends without checking,
        // and we know we just created the table (or it existed), we can try to add it.
        // However, if the table existed and ALREADY had the column, this might fail.
        // But standard migration flow assumes if this migration runs, it hasn't been applied.
        
        // To be safe against "Table created but column missing" vs "Table created AND column exists" (if we are re-running):
        // We can just run the alter. If it fails because column exists, we might want to ignore.
        // But for now, let's stick to the original logic for the column, just prepended with create_table.
        
        manager
            .alter_table(
                Table::alter()
                    .table(TbDiscordAiContext::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(TbDiscordAiContext::ThinkingBought)
                            .integer()
                            .null()
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
