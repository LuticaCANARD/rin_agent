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
            .table(TbAlarmModel::Table)
            .modify_column(
                ColumnDef::new(TbAlarmModel::Time)
                    .date_time()
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Expr::current_timestamp())
            )
            .modify_column(
                ColumnDef::new(TbAlarmModel::CreatedAt)
                    .date_time()
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Expr::current_timestamp())
            )
            .modify_column(
                ColumnDef::new(TbAlarmModel::UpdatedAt)
                    .date_time()
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Expr::current_timestamp())
            )
            .modify_column(
                ColumnDef::new(TbAlarmModel::UserId)
                    .big_unsigned()
                    .not_null()
                    .default(Expr::value(0))
            )
            .add_column_if_not_exists(
                ColumnDef::new(TbAlarmModel::ChannelId)
                    .big_unsigned()
                    .not_null()
                    .default(Expr::value(0))
            )
            .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(Table::alter().table(TbAlarmModel::Table)
                .modify_column(
                    ColumnDef::new(TbAlarmModel::Time)
                        .date_time()
                        .not_null()
                        .default(Expr::current_timestamp())
                )
                .modify_column(
                    ColumnDef::new(TbAlarmModel::CreatedAt)
                        .date_time()
                        .not_null()
                        .default(Expr::current_timestamp())
                )
                .modify_column(
                    ColumnDef::new(TbAlarmModel::UpdatedAt)
                        .date_time()
                        .not_null()
                        .default(Expr::current_timestamp())
                )
                .modify_column(
                    ColumnDef::new(TbAlarmModel::UserId)
                        .big_unsigned()
                        .not_null()
                        .default(Expr::value(0))
                )
                .drop_column(TbAlarmModel::ChannelId)
                .to_owned(),
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
    UserId,
    ChannelId
}

