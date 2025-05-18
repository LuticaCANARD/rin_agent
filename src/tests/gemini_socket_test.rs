#[cfg(test)]
use crate::libs::logger::{LOGGER, LogLevel};
use crate::gemini::service::socket_client::GeminiSocketClient;
use crate::gemini::service::socket_manager;
use dotenv::dotenv;
use std::env;


#[tokio::test]
async fn make_client() {
    dotenv().ok();
    let gemini_token = env::var("GEMINI_API_KEY").unwrap();

    LOGGER.log(LogLevel::Debug, format!("GEMINI_API_KEY: {}", gemini_token).as_str());
    let mut client = GeminiSocketClient::new(1, "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent".to_string());
    let result = client.connect().await;
    match result {
        Ok(_) => {
            LOGGER.log(LogLevel::Debug, "Connected successfully");
        }
        Err(e) => {
            LOGGER.log(LogLevel::Error, format!("Connection failed: {}", e).as_str());
        }
    }
}