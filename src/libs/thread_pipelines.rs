// 이 파일에는 thread간 통신을 위한 파이프라인이 포함되어야 함.

use tokio::sync::watch;
use crate::libs::thread_message::DiscordToGeminiMessage;
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

lazy_static! {
    pub static ref DISCORD_TO_GEMINI_PIPELINE: AsyncThreadPipeline<DiscordToGeminiMessage<String>> =
        AsyncThreadPipeline::new(); // 버퍼 크기 설정
}