use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTime;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tb_discord_guilds")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub guild_id: i64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {

}

impl ActiveModelBehavior for ActiveModel {

}

