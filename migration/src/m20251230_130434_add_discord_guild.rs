use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TbDiscordGuilds::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TbDiscordGuilds::Id).big_integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(TbDiscordGuilds::GuildId).big_integer().not_null().unique_key())
                    .col(ColumnDef::new(TbDiscordGuilds::JoinedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(TbDiscordGuilds::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TbDiscordGuilds::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TbDiscordGuilds {
    Table,
    Id,
    GuildId,
    JoinedAt,
    UpdatedAt,
}
