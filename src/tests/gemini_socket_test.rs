#[cfg(test)]
use crate::libs::logger::{LOGGER, LogLevel};
use crate::gemini::service::socket_client::GeminiSocketClient;
use crate::gemini::service::socket_manager;
use dotenv::dotenv;
use tokio_tungstenite::tungstenite::Message;
use std::env;


#[tokio::test]
async fn make_client() {
    dotenv().ok();
    let gemini_token = env::var("GEMINI_API_KEY").unwrap();

    LOGGER.log(LogLevel::Debug, format!("GEMINI_API_KEY: {}", gemini_token).as_str());
    let mut client = GeminiSocketClient::new(1, "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent".to_string());
    let connection_result = client.connect().await;
    if let Err(e) = connection_result {
        LOGGER.log(LogLevel::Error, format!("Failed to connect: {}", e).as_str());
    }
    
    let connected = client.start_managing_connection(message_handler_tx).await;
    if let Err(e) = connected {
        LOGGER.log(LogLevel::Error, format!("Failed to start managing connection: {}", e).as_str());
    }

    


}

#[test]
fn test_socket_manager() {}
