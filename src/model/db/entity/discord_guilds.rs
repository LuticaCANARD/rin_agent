use sea_orm::entity::prelude::*;
use sea_orm::prelude::DateTime;

// https://www.sea-ql.org/SeaORM/docs/generate-entity/entity-structure/
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tb_discord_guilds")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub guild_id: i64,
    pub joined_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {

}

impl ActiveModelBehavior for ActiveModel {

}

