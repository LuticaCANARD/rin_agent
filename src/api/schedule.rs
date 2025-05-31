use chrono::{DateTime, FixedOffset};
use entity::tb_alarm_model;
use gemini_live_api::libs::logger::LOGGER;
use rocket::time::{Date, OffsetDateTime, UtcOffset};
use rs_ervice::RSContextService;
use sea_orm::{EntityTrait, TransactionTrait};
use sqlx::types::{chrono::Local, time};
use std::sync::{Arc};
use tokio::sync::{watch::Sender, Mutex};

use crate::{libs::{thread_message::{DiscordToGeminiMessage, GeminiFunctionAlarm}, thread_pipelines::{AsyncThreadPipeline, GEMINI_FUNCTION_EXECUTION_ALARM, SCHEDULE_TO_DISCORD_PIPELINE}}, model::db::driver::DB_CONNECTION_POOL, service::discord_error_msg::{send_additional_log, send_debug_error_log}};


pub struct ScheduleRequest {
    pub start: DateTime<FixedOffset>,
    pub end: DateTime<FixedOffset>,
    pub timezone: String,
    pub name: String,
    pub description: Option<String>,
    pub repeat: Option<ScheduleRepeatRequest>,
}
pub struct ScheduleRepeatRequest {
    pub repeat_type: String,
    pub repeat_interval: i32,
    pub repeat_end: Option<DateTime<FixedOffset>>,
}

pub struct ScheduleResponse {
    pub id: i32,
    pub start: DateTime<FixedOffset>,
    pub end: DateTime<FixedOffset>,
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
    alarm_pipe_sender: Sender<GeminiFunctionAlarm<Option<tb_alarm_model::Model>>>,
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
                    repeat_end_at: sea_orm::Set(
                        schedule.repeat.as_ref()
                            .and_then(|r| r.repeat_end)
                            .map(|dt| dt.naive_local())
                    ),
                    created_at: sea_orm::Set(Local::now().with_timezone(&Local::now().offset())),
                    updated_at: sea_orm::Set(Local::now().with_timezone(&Local::now().offset())),
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
        LOGGER.log(
            gemini_live_api::libs::logger::LogLevel::Debug,
            &format!("알람이 설정되었습니다: {:?}", inserted_schedule),
        );

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

        // Convert last_schedule to NaiveDateTime for comparison
        let last_schedule_naive = last_schedule.naive_local();

        let need_alarm = now > last_schedule_naive;
        if need_alarm == false {
            return; // 아직 알람을 울릴 시간이 아니다.
        }

        // 알람을 울린다.
        let alarm_model = self.alarm_target_model.as_ref().unwrap().clone();
        let alarm_model = tb_alarm_model::Model {
            id: alarm_model.id,
            time: alarm_model.time,
            message: alarm_model.message,
            repeat_circle: alarm_model.repeat_circle,
            repeat_end_at: alarm_model.repeat_end_at,
            created_at: alarm_model.created_at,
            updated_at: Local::now().into(),
            user_id: alarm_model.user_id,
            user_name: alarm_model.user_name,
        };
        let alarm_item = GeminiFunctionAlarm {
            message: Some(alarm_model.clone()),
            sender:alarm_model.clone().user_id.to_string(),
            channel_id: alarm_model.clone().id.to_string(),
            message_id: "".to_string(),
            guild_id: "0".to_string(),
        };
        self.alarm_pipe_sender.send(alarm_item)
        .unwrap_or_else(|e| {
            // Fire and forget the async error log
            tokio::spawn(send_debug_error_log(format!("Failed to send alarm message: {}", e)));
        });
    }
    
}