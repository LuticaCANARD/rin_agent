use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager.alter_table(
            Table::alter()
                .table(TbDiscordAiContext::Table)
                .add_column(
                    ColumnDef::new(TbDiscordAiContext::ShowThought)
                        .boolean()
                        .not_null()
                        .default(true) // Default to true if no value is provided
                        .to_owned(),
                )
            .to_owned(),
        ).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(TbDiscordAiContext::Table)
                    .drop_column(TbDiscordAiContext::ShowThought)
                    .to_owned(),
            )
            .await
            .map_err(|e| e.into())
            .and_then(|_| Ok(()))

    }
}

#[derive(DeriveIden)]
enum TbDiscordAiContext {
    Table,
    Id,
    ShowThought
}