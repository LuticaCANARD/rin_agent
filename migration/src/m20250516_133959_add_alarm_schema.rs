use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
        .create_table(
            Table::create()
                .table(TbAlarmModel::Table)
                .if_not_exists()
                .col(pk_auto(TbAlarmModel::Id))
                .col(date_time(TbAlarmModel::Time))
                .col(ColumnDef::new(TbAlarmModel::Message).text().not_null())
                .col(string_null(TbAlarmModel::RepeatCircle).string_len(100)) 
                .col(date_time_null(TbAlarmModel::RepeatEndAt))
                .col(date_time(TbAlarmModel::CreatedAt))
                .col(date_time(TbAlarmModel::UpdatedAt))
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        
        manager
            .drop_table(Table::drop().table(TbAlarmModel::Table).to_owned())
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
}
