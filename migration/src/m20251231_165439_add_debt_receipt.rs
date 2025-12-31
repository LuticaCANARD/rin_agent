use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;
struct NowCall;

impl Iden for NowCall {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(s, "NOW").unwrap();
    }
}
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
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(DebtReceipt::UserId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DebtReceipt::Amount)
                            .decimal()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(DebtReceipt::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(SimpleExpr::FunctionCall(
                                Func::cust(NowCall)
                            )),
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
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DebtReceipt::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum DebtReceipt {
    Table,
    Id,
    UserId,
    Amount,
    CreatedAt,
    IsCleared,
    ClearedAt,
}
