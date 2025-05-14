use sqlx::types::time;

use crate::model::db::driver::DB_CONNECTION_POOL;


pub struct ScheduleRequest {
    pub start: time::PrimitiveDateTime,
    pub end: time::PrimitiveDateTime,
    pub timezone: String,
    pub name: String,
    pub description: Option<String>,
    pub repeat: Option<ScheduleRepeatRequest>,
}
pub struct ScheduleRepeatRequest {
    pub repeat_type: String,
    pub repeat_interval: i32,
    pub repeat_end: Option<time::PrimitiveDateTime>,
}

pub struct ScheduleResponse {
    pub id: i32,
    pub start: time::PrimitiveDateTime,
    pub end: time::PrimitiveDateTime,
    pub timezone: String,
    pub name: String,
    pub description: Option<String>,
    pub repeat: Option<ScheduleRepeatRequest>,
}

pub fn make_alarm_schedule(
    schedule: &ScheduleRequest,
) -> Result<String, String> {

    let db = DB_CONNECTION_POOL.get();
    let conn = db.unwrap();
    
    
    Ok("".to_string())
}