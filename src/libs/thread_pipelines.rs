// 이 파일에는 thread간 통신을 위한 파이프라인이 포함되어야 함.

use tokio::sync::mpsc;
use crate::libs::thread_message::DiscordToGeminiMessage;
use lazy_static::lazy_static;

pub struct AsyncThreadPipeline<T> {
    pub sender: mpsc::Sender<T>,
    pub receiver: mpsc::Receiver<T>,
}

impl<T> AsyncThreadPipeline<T> {
    pub fn new(buffer: usize) -> Self {
        let (sender, receiver) = mpsc::channel(buffer);
        AsyncThreadPipeline { sender, receiver }
    }
}

lazy_static! {
    pub static ref DISCORD_TO_GEMINI_PIPELINE: AsyncThreadPipeline<DiscordToGeminiMessage<String>> =
        AsyncThreadPipeline::new(100); // 버퍼 크기 설정
}