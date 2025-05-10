use gemini_client::GeminiClientTrait;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
pub mod utils;
pub mod types;

pub mod gemini_client;


lazy_static!{
    pub static ref GEMINI_CLIENT: Mutex<gemini_client::GeminiClient> = Mutex::new(
        gemini_client::GeminiClient::new()
    ) ; 
}