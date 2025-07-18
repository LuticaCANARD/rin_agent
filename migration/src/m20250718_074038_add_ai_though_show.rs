use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager.alter_table(
            Table::alter()
                .table(TbAiContext::Table)
                .add_column(
                    ColumnDef::new(TbAiContext::ShowThought)
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
                    .table(TbAiContext::Table)
                    .drop_column(TbAiContext::ShowThought)
                    .to_owned(),
            )
            .await
            .map_err(|e| e.into())
            .and_then(|_| Ok(()))

    }
}

#[derive(DeriveIden)]
enum TbAiContext {
    Table,
    Id,
    ShowThought
}