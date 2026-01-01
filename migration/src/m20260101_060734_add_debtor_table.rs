use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Debtor::Table)
                    .if_not_exists()
                    .col(pk_auto(Debtor::Id))
                    .col(string(Debtor::Name))
                    .col(date_time(Debtor::CreatedAt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Debtor::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Debtor{
    #[sea_orm(iden = "tb_debtor")]
    Table,
    Id,
    Name,
    CreatedAt,
}