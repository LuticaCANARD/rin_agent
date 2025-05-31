// 이 파일에는 thread간 통신을 위한 파이프라인이 포함되어야 함.

use tokio::sync::watch;
use entity::tb_alarm_model;
use crate::{gemini::types::GeminiActionResult, libs::thread_message::{
    DiscordToGeminiMessage,GeminiFunctionAlarm
}};
use lazy_static::lazy_static;

pub struct AsyncThreadPipeline<T> {
    pub sender: watch::Sender<T>,
    pub receiver: watch::Receiver<T>,
}

impl<T> AsyncThreadPipeline<T> where
    T: Default + 'static,{
    pub fn new() -> Self {
        let (sender, receiver) = watch::channel(T::default());
        AsyncThreadPipeline { sender, receiver }
    }
}

pub type GeminiChannelResult = GeminiFunctionAlarm<GeminiActionResult>;

lazy_static! {
    pub static ref GEMINI_FUNCTION_EXECUTION_ALARM: AsyncThreadPipeline<GeminiChannelResult> =
        AsyncThreadPipeline::new();

    pub static ref SCHEDULE_TO_DISCORD_PIPELINE: AsyncThreadPipeline<GeminiFunctionAlarm<Option<tb_alarm_model::Model>>> =
        AsyncThreadPipeline::new();
}