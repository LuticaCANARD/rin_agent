pub use sea_orm_migration::prelude::*;

mod m20250516_133959_add_alarm_schema;
mod m20250526_152732_add_thinking_conf_cols;
mod m20250529_170700_add_thinking_cache_cols;
mod m20250530_213902_add_alarm_user;
mod m20250531_073854_modify_alarm;
mod m20250718_074038_add_ai_though_show;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250516_133959_add_alarm_schema::Migration),
            Box::new(m20250526_152732_add_thinking_conf_cols::Migration),
            Box::new(m20250529_170700_add_thinking_cache_cols::Migration),
            Box::new(m20250530_213902_add_alarm_user::Migration),
            Box::new(m20250531_073854_modify_alarm::Migration),
            Box::new(m20250718_074038_add_ai_though_show::Migration),
        ]
    }
}
