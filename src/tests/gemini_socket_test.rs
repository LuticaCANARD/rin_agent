use crate::gemini::schema::live_api_types::BidiGenerateContentSetup;
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
    let mut client = GeminiSocketClient::<i64>::new(
        1, 
        "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent".to_string(),
        BidiGenerateContentSetup{
            model:"models/chat-bison-001".to_string(),
            generation_config: todo!(), 
            system_instruction: todo!(), 
            tools: todo!(), 
            realtime_input_config: todo!(), 
            session_resumption: todo!(), 
            context_window_compression: todo!(), 
            input_audio_transcription: todo!(), 
            output_audio_generation: todo!() 
        }
    );
    let connection_result = client.connect().await;
    if let Err(e) = connection_result {
        LOGGER.log(LogLevel::Error, format!("Failed to connect: {}", e).as_str());
    }
    
    


}

#[test]
fn test_socket_manager() {}
