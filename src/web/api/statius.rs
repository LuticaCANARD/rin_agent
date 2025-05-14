use entity::tb_ai_context;
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};

use crate::model::db::driver::DB_CONNECTION_POOL;


#[get("/status")]
pub async fn get_status() -> String {

    let db = DB_CONNECTION_POOL.get();
    let conn = db.unwrap();

    let a = tb_ai_context::Entity::find()
        .order_by_desc(tb_ai_context::Column::Id)
        .limit(10)
        .all(conn)
        .await
        .unwrap();
    let mut result = String::new();
    for i in &a {
        result.push_str(&format!("{}: {}\n", i.id, i.context));
    }
    result.push_str(&format!("{}: {}\n", "count", a.len()));
    result
}