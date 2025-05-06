macro_rules! create_table_generate {
    ($table_name:ident, $($column_name:ident : $column_type:ty),*) => {
        #[derive(DeriveIden)]
        enum $table_name {
            Table,
            $(
                $column_name,
            )*
        }
        
        impl MigrationTrait for Migration {
            async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager
                    .create_table(
                        Table::create()
                            .table($table_name::Table)
                            .if_not_exists()
                            $(.col($column_type($table_name::$column_name)))*
                            .to_owned(),
                    )
                    .await
            }
        }
    };
}