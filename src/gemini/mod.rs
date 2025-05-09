use gemini_client::GeminiClientTrait;
use lazy_static::lazy_static;
use crate::libs::thread_pipelines::DISCORD_TO_GEMINI_PIPELINE;
use crate::libs::thread_message::{DiscordToGeminiMessage};
use tokio::sync::Mutex;

pub mod gemini_client;

lazy_static!{
    pub static ref GEMINI_CLIENT: Mutex<gemini_client::GeminiClient> = Mutex::new(
        gemini_client::GeminiClient::new()
    ) ; 
}