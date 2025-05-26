pub use sea_orm_migration::prelude::*;

mod m20250516_133959_add_alarm_schema;
mod m20250526_152732_add_thinking_conf_cols;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250516_133959_add_alarm_schema::Migration),
            Box::new(m20250526_152732_add_thinking_conf_cols::Migration),
        ]
    }
}
