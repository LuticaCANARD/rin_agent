use chrono::{DateTime, FixedOffset};
use entity::tb_alarm_model;
use gemini_live_api::libs::logger::LOGGER;
use rocket::time::{Date, OffsetDateTime, UtcOffset};
use rs_ervice::RSContextService;
use sea_orm::{prelude::Expr, Condition, EntityTrait, QueryFilter, QueryOrder, TransactionTrait};
use serenity::all::{ChannelId, UserId};
use sqlx::types::{chrono::Local, time};
use std::sync::{Arc};
use tokio::sync::{watch::Sender, Mutex};
use crate::{libs::{thread_message::GeminiFunctionAlarm, thread_pipelines::SCHEDULE_TO_DISCORD_PIPELINE}, model::db::driver::DB_CONNECTION_POOL, service::discord_error_msg::{send_additional_log, send_debug_error_log}};


pub struct ScheduleRequest {
    pub start: DateTime<FixedOffset>,
    pub end: DateTime<FixedOffset>,
    pub name: String,
    pub sender: UserId,
    pub guild_id: Option<u64>,
    pub channel_id: ChannelId,
    pub description: Option<String>,
    pub repeat: Option<ScheduleRepeatRequest>,
    pub context_id: Option<i64>,
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
    alarm_target_model: Option<Vec<tb_alarm_model::Model>>,
    alarm_thread: Option<tokio::task::JoinHandle<()>>,
    alarm_pipe_sender: Sender<GeminiFunctionAlarm<Option<tb_alarm_model::Model>>>,
}

impl RSContextService for ScheduleService {
    async fn on_register_crate_instance() -> Self where Self: Sized {
        Self::new()
    }

    async fn on_service_created(&mut self, builder: &rs_ervice::RSContextBuilder) -> Result<(), rs_ervice::RsServiceError> {
        // Register the service with the context
        Ok(())
    }

    async fn on_all_services_built(&self, context: &rs_ervice::RSContext) -> Result<(), rs_ervice::RsServiceError> {
        ScheduleService::start_alarm_thread(
            context.call::<ScheduleService>().unwrap()
        );
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

    pub fn start_alarm_thread(this: Arc<Mutex<Self>>) {
        let cloned_this = this.clone();
        let handle = tokio::spawn(async move {

            let db = DB_CONNECTION_POOL.get();
            if db.is_none() {
                send_debug_error_log("DB connection pool is not initialized".to_string()).await;
                return;
            }
            let db = db.unwrap();
            let last_cache = tb_alarm_model::Entity::find()
                .order_by_asc(tb_alarm_model::Column::Time)
                .filter(Condition::all()
                    .add(Expr::col(tb_alarm_model::Column::Time)
                        .gt(Local::now().to_utc())
                    )
                ) // 0은 모든 유저를 의미한다.
                .one(db)
                .await;            
            LOGGER.log(
                gemini_live_api::libs::logger::LogLevel::Debug,
                &format!("Last alarm cache fetched: {:?}", last_cache),
            );

            if last_cache.is_ok() && last_cache.as_ref().unwrap().is_some() {
                let last_cache = last_cache.unwrap().unwrap();
                LOGGER.log(
                    gemini_live_api::libs::logger::LogLevel::Debug,
                    &format!("Last alarm cache: {:?}", last_cache),
                );
                this.lock().await.alarm_target_model = Some(vec![last_cache]);
            } else {
                LOGGER.log(
                    gemini_live_api::libs::logger::LogLevel::Debug,
                    "No alarms found in the database.",
                );
            }

            loop {
                let mut guard = this.lock().await;
                guard.simulate_schedule().await;
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        tokio::spawn(async move {
            let mut this = cloned_this.lock().await;
            this.alarm_thread = Some(handle);
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
                    user_id: sea_orm::Set(schedule.sender.get() as i64),
                    channel_id: sea_orm::Set(schedule.channel_id.get() as i64),
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
            if schedule.start < remain.get(0).unwrap().time {
                LOGGER.log(
                    gemini_live_api::libs::logger::LogLevel::Debug,
                    &format!("기존 알람보다 더 빠른 시간에 알람이 설정되었습니다: {:?}", inserted_schedule),
                );


                self.alarm_target_model = Some(vec![inserted_schedule.clone()]);
            } else if schedule.start == remain.get(0).unwrap().time {
                LOGGER.log(
                    gemini_live_api::libs::logger::LogLevel::Debug,
                    &format!("기존 알람과 동일한 시간에 알람이 설정되었습니다: {:?}", inserted_schedule),
                );
                let mut origin = self.alarm_target_model.as_ref().unwrap().clone();
                origin.push(inserted_schedule.clone());
                self.alarm_target_model = Some(origin);
            }
        } else {
            LOGGER.log(
                gemini_live_api::libs::logger::LogLevel::Debug,
                &format!("새로운 알람이 설정되었습니다: {:?}", inserted_schedule),
            );
            self.alarm_target_model = Some(vec![inserted_schedule.clone()]);
        }
    }

    async fn simulate_schedule(&mut self){
        if self.alarm_target_model.is_none() {
            return; // 아직 알람이 설정되지 않았다.

        }
        // 1초에 1회 실행된다. 
        let now = Local::now().to_utc(); // 올바른 변환
        let last_schedule = self.alarm_target_model.as_ref().unwrap().last().unwrap().time;
        LOGGER.log(
            gemini_live_api::libs::logger::LogLevel::Debug,
            &format!("현재 시간: {}, 마지막 알람 시간: {}", now, last_schedule),
        );

        // Convert last_schedule to NaiveDateTime for comparison
        let last_schedule_naive = last_schedule.to_utc();

        let need_alarm = now > last_schedule_naive;
        if need_alarm == false {
            return; // 아직 알람을 울릴 시간이 아니다.
        }

        // 알람을 울린다.

        for alarm_model in self.alarm_target_model.as_ref().unwrap().iter() {
            // 시간은 모두 동일하다...
            let alarm_model = tb_alarm_model::Model {
                id: alarm_model.id,
                time: alarm_model.time,
                message: alarm_model.message.clone(),
                repeat_circle: alarm_model.repeat_circle.clone(),
                repeat_end_at: alarm_model.repeat_end_at,
                created_at: alarm_model.created_at,
                updated_at: Local::now().into(),
                user_id: alarm_model.user_id,
                user_name: alarm_model.user_name.clone(),
                channel_id: alarm_model.channel_id,
            };

            let alarm_item = GeminiFunctionAlarm {
                message: Some(alarm_model.clone()),
                sender: alarm_model.clone().user_id.to_string(),
                channel_id: alarm_model.clone().channel_id.to_string(),
                message_id: "".to_string(),
                guild_id: "0".to_string(),
                need_send: false,
                context_id: 0 // 나중에는 context_id를 넣어주자.
            };
            self.alarm_pipe_sender.send(alarm_item)
            .unwrap_or_else(|e| {
                // Fire and forget the async error log
                tokio::spawn(send_debug_error_log(format!("Failed to send alarm message: {}", e)));
            });
        } 
        LOGGER.log(
            gemini_live_api::libs::logger::LogLevel::Debug,
            &format!("알람을 울렸습니다: {:?}", self.alarm_target_model),
        );
        self.alarm_target_model = None; // 알람을 울린 후에는 알람을 초기화한다.

        let db = DB_CONNECTION_POOL.get();
        if db.is_none() {
            send_debug_error_log("DB connection pool is not initialized".to_string()).await;
            return;
        }
        let db = db.unwrap();
        let next_schedule = tb_alarm_model::Entity::find()
            .order_by_asc(tb_alarm_model::Column::Time)
            .filter(Condition::all()
                .add(Expr::col(tb_alarm_model::Column::Time)
                    .gt(Local::now().to_utc())
                )
            ) // 0은 모든 유저를 의미한다.
            .one(db)
            .await;
        if let Ok(Some(next)) = next_schedule {
            LOGGER.log(
                gemini_live_api::libs::logger::LogLevel::Debug,
                &format!("다음 알람이 설정되었습니다: {:?}", next),
            );
            self.alarm_target_model = Some(vec![next]);
        } else {
            LOGGER.log(
                gemini_live_api::libs::logger::LogLevel::Debug,
                "다음 알람이 없습니다.",
            );
            self.alarm_target_model = None; // 다음 알람이 없다면 초기화한다.
        }

    }
    
}