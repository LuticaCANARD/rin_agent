use rs_ervice::RSContextService;
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

pub struct ScheduleService {
    alarm_schedules: Vec<ScheduleRequest>,
}

impl RSContextService for ScheduleService {
    fn on_register_crate_instance() -> Self where Self: Sized {
        Self::new()
    }

    fn on_service_created(&mut self, builder: &rs_ervice::RSContextBuilder) -> Result<(), rs_ervice::RsServiceError> {
        Ok(())
    }

    fn on_all_services_built(&self, context: &rs_ervice::RSContext) -> Result<(), rs_ervice::RsServiceError> {
        Ok(())
    }
}

impl ScheduleService {
    pub fn new() -> Self {
        Self {
            alarm_schedules: Vec::new(),
        }
    }

    pub async fn add_schedule(&mut self, schedule: ScheduleRequest) {
        self.alarm_schedules.push(schedule);
    }
    
}