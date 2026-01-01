use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DebtReceipt::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DebtReceipt::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DebtReceipt::DebtorId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DebtReceipt::Amount)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DebtReceipt::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(DebtReceipt::IsCleared)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(DebtReceipt::ClearedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-debt_receipt-debtor_id")
                            .from(DebtReceipt::Table, DebtReceipt::DebtorId)
                            .to(Debtor::Table, Debtor::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-debt_receipt-debtor_id")
                    .table(DebtReceipt::Table)
                    .col(DebtReceipt::DebtorId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-debt_receipt-debtor_id")
                    .table(DebtReceipt::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(DebtReceipt::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DebtReceipt {
    #[sea_orm(iden = "tb_debt_receipt")]
    Table,
    Id,
    DebtorId,
    Amount,
    CreatedAt,
    IsCleared,
    ClearedAt,
}

#[derive(DeriveIden)]
enum Debtor {
    #[sea_orm(iden = "tb_debtor")]
    Table,
    Id,
}
