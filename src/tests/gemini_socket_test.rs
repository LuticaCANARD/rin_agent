use crate::gemini::schema::enums::GeminiContentRole;
use crate::gemini::schema::live_api_types::{BidiGenerateContentClientContent, BidiGenerateContentSetup, ContextWindowCompression, GeminiLiveApiTool, SessionResumptionConfig};
use crate::gemini::schema::types::{GeminiContents, GeminiFunctionDeclaration, GeminiGenerationConfig, GeminiGenerationConfigTool, GeminiGoogleSearchTool, GeminiParts};
use crate::gemini::types::GeminiBotTools;
#[cfg(test)]
use crate::libs::logger::{LOGGER, LogLevel};
use crate::gemini::service::socket_client::GeminiSocketClient;
use crate::gemini::service::socket_manager;
use crate::setting::gemini_setting::GEMINI_MODEL_PRO;
use dotenv::dotenv;
use tokio_tungstenite::tungstenite::Message;
use std::env;


#[tokio::test]
async fn make_client() {
    dotenv().ok();
    let gemini_token = env::var("GEMINI_API_KEY").unwrap();

    LOGGER.log(LogLevel::Debug, format!("GEMINI_API_KEY: {}", gemini_token).as_str());
    let generation_config = Some(
        GeminiGenerationConfig {
            candidate_count: Some(1),
            max_output_tokens: Some(100),
            temperature: Some(1.5),
            top_p: Some(0.9),
            top_k: Some(40),
            presence_penalty: Some(0.0),
            frequency_penalty: Some(0.0),
            response_modalities: None,
            ..Default::default()
        }
    );
    let mut inst_part = GeminiParts::default();
    inst_part.set_text("you are a maid for a master".to_string());
    let parts = vec![inst_part];
    let system_instruction = Some(
        GeminiContents {
            parts,
            role: GeminiContentRole::User
        }
    );

    let fun_declare = GeminiGenerationConfigTool{
        function_declarations:Some( vec![]),
        ..Default::default()
    };
    let google_search = GeminiGenerationConfigTool{
        google_search: Some(GeminiGoogleSearchTool),
        ..Default::default()
    };
    let tools = Some(vec![
        fun_declare,
        google_search,
    ]);
    let realtime_input_config = None;
    let session_resumption = None;
    let context_window_compression = None;
    let audio_conf = None;

    let mut client = GeminiSocketClient::<i64>::new(
        1, 
        "wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent".to_string(),
        BidiGenerateContentSetup{
            model:"gemini-2.5-flash-live-001".to_string(),
            generation_config, 
            system_instruction, 
            tools,
            realtime_input_config, 
            session_resumption, 
            context_window_compression, 
            input_audio_transcription: audio_conf.clone(), 
            output_audio_generation: audio_conf.clone()
        }
    );
    let connection_result = client.connect().await;
    if let Err(e) = connection_result {
        LOGGER.log(LogLevel::Error, format!("Failed to connect: {}", e).as_str());
    }
    
    let message = "Hello, Gemini!";
    let mut part_msg = GeminiParts::default();
    part_msg.set_text(message.to_string());
    
    let parts = vec![part_msg];
    let msgcontent = BidiGenerateContentClientContent{
        turns: Some(vec![
            GeminiContents {
                parts,
                role: GeminiContentRole::User
            }
        ]),
        turn_complete: Some(true),
    };
    client.send_new_part(
        msgcontent
    ).await
        .expect("Failed to send message");
    

    


}

#[test]
fn test_socket_manager() {}
