use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.alter_table(
            Table::alter()
                .table(TbAlarmModel::Table)
                .add_column_if_not_exists(
                    ColumnDef::new(TbAlarmModel::UserId)
                        .integer()
                        .not_null()
                        .default(0) // Default to 0 if no user ID is provided
                        .to_owned(),
                )
                .add_column_if_not_exists(
                    ColumnDef::new(TbAlarmModel::UserName)
                        .string_len(100)
                        .not_null()
                        .default("") // Default to empty string if no user name is provided
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
                .table(TbAlarmModel::Table)
                    .drop_column(TbAlarmModel::UserId)
                    .drop_column(TbAlarmModel::UserName)
                    .to_owned()
            )
            .await
    }
}



#[derive(DeriveIden)]
enum TbAlarmModel {
    Table,
    Id,
    Time,
    Message,
    RepeatCircle,
    RepeatEndAt,
    CreatedAt,
    UpdatedAt,
    UserId, // New column for user ID
    UserName, // New column for user name
}

