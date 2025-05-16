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
                .table(Alarm::Table)
                .if_not_exists()
                .col(pk_auto(Alarm::Id))
                .col(date_time(Alarm::Time))
                .col(string(Alarm::Message))
                .col(string_null(Alarm::RepeatCircle))
                .col(date_time_null(Alarm::RepeatEndAt))
                .col(date_time(Alarm::CreatedAt))
                .col(date_time(Alarm::UpdatedAt))
                .to_owned(),
        ).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        
        manager
            .drop_table(Table::drop().table(Alarm::Table).to_owned())
            .await
    }
}
#[derive(DeriveIden)]
enum Alarm {
    Table,
    Id,
    Time,
    Message,
    RepeatCircle,
    RepeatEndAt,
    CreatedAt,
    UpdatedAt,
}
