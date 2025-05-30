use entity::tb_alarm_model;
use rocket::time::{Date, OffsetDateTime, UtcOffset};
use rs_ervice::RSContextService;
use sea_orm::{prelude::{DateTime, TimeDateTime}, EntityTrait, TransactionTrait};
use sqlx::types::{chrono::Local, time};
use std::sync::{Arc};
use tokio::sync::{watch::Sender, Mutex};

use crate::{libs::{thread_message::DiscordToGeminiMessage, thread_pipelines::{AsyncThreadPipeline, GEMINI_FUNCTION_EXECUTION_ALARM, SCHEDULE_TO_DISCORD_PIPELINE}}, model::db::driver::DB_CONNECTION_POOL, service::discord_error_msg::{send_additional_log, send_debug_error_log}};


pub struct ScheduleRequest {
    pub start: DateTime,
    pub end: DateTime,
    pub timezone: String,
    pub name: String,
    pub description: Option<String>,
    pub repeat: Option<ScheduleRepeatRequest>,
}
pub struct ScheduleRepeatRequest {
    pub repeat_type: String,
    pub repeat_interval: i32,
    pub repeat_end: Option<DateTime>,
}

pub struct ScheduleResponse {
    pub id: i32,
    pub start: DateTime,
    pub end: DateTime,
    pub timezone: String,
    pub name: String,
    pub description: Option<String>,
    pub repeat: Option<ScheduleRepeatRequest>,
}

pub fn make_alarm_schedule(
    schedule: &ScheduleRequest,
) -> Result<String, String> {



    Ok("".to_string())
}

pub struct ScheduleService {
    alarm_target_model: Option<tb_alarm_model::Model>,
    alarm_thread: Option<tokio::task::JoinHandle<()>>,
    alarm_pipe_sender: Sender<DiscordToGeminiMessage<Option<tb_alarm_model::Model>>>,
}

impl RSContextService for ScheduleService {
    async fn on_register_crate_instance() -> Self where Self: Sized {
        Self::new()
    }

    async fn on_service_created(&mut self, builder: &rs_ervice::RSContextBuilder) -> Result<(), rs_ervice::RsServiceError> {

        Ok(())
    }

    async fn on_all_services_built(&self, context: &rs_ervice::RSContext) -> Result<(), rs_ervice::RsServiceError> {
        let service = Arc::new(Mutex::new(ScheduleService::new()));
        ScheduleService::start_alarm_thread(Arc::clone(&service));
        Ok(())
    }
}

impl ScheduleService {
    pub fn new() -> Self {
        Self {
            alarm_target_model:None,
            alarm_thread: None,
            alarm_pipe_sender: SCHEDULE_TO_DISCORD_PIPELINE.sender.clone(),
        }
    }

    pub fn start_alarm_thread(this: Arc<Mutex<ScheduleService>>) {
        let this_clone = Arc::clone(&this);
        let handle = tokio::spawn(async move {
            loop {
                {
                    let service = this_clone.lock().await;
                    service.simulate_schedule();
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
        // alarm_thread 핸들 저장
        tokio::spawn(async move {
            let mut guard = this.lock().await;
            guard.alarm_thread = Some(handle);
        });
    }


    pub async fn add_schedule(&mut self, schedule: ScheduleRequest) {
        let db = DB_CONNECTION_POOL.get();
        let conn = db.unwrap();

        let insert_result = conn.transaction(|ts| {
            Box::pin(async move {
                let new_alarm = tb_alarm_model::ActiveModel {
                    time: sea_orm::Set(schedule.start),
                    message: sea_orm::Set(schedule.description.unwrap_or_default()),
                    repeat_circle: sea_orm::Set(schedule.repeat.as_ref()
                    .map(|r| r.repeat_type.clone())),
                    repeat_end_at:sea_orm::Set(schedule.repeat.as_ref()
                    .and_then(|r| r.repeat_end)),
                    created_at: sea_orm::Set(DateTime::from(Local::now().naive_local())),
                    updated_at: sea_orm::Set(DateTime::from(Local::now().naive_local())),
                    ..Default::default()
                };
                let ret = tb_alarm_model::Entity::insert(new_alarm)
                    .exec_with_returning(ts)
                    .await?;
                Ok::<tb_alarm_model::Model, sea_orm::DbErr>(ret)
            })
        }).await;

        if let Err(e) = insert_result {
            send_debug_error_log(format!("Failed to insert schedule: {}", e)).await;
            return;
        }

        
        let inserted_schedule = insert_result.unwrap();
        // 이하는 캐싱 교체 !
        if let Some(remain) = &self.alarm_target_model {
            if schedule.start < remain.time {
                self.alarm_target_model = Some(inserted_schedule.clone());
            }
        } else {
            self.alarm_target_model = Some(inserted_schedule.clone());
        }
    }

    fn simulate_schedule(&self){
        if self.alarm_target_model.is_none() {
            return; // 아직 알람이 설정되지 않았다.
        }
        // 1초에 1회 실행된다. 
        let now = Local::now().naive_local(); // 올바른 변환
        let last_schedule = self.alarm_target_model.as_ref().unwrap().time;

        let need_alarm = now > last_schedule;
        if need_alarm == false {
            return; // 아직 알람을 울릴 시간이 아니다.
        }


    }
    
}