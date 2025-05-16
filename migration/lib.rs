pub use sea_orm_migration::*;

mod mYYYYMMDD_HHMMSS_create_users_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(mYYYYMMDD_HHMMSS_create_users_table::Migration),
        ]
    }
}