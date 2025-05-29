#[cfg(test)]
#[tokio::test]
async fn test_gemini_cache() {
    use gemini_live_api::libs::logger::LOGGER;
    use serde_json::to_string_pretty;
    use sqlx::types::chrono;
    dotenv::dotenv().ok();
    use crate::{gemini::gemini_client::{generate_gemini_cache_setting, GeminiClient, GeminiClientTrait}, setting::{self, gemini_setting::get_begin_query}};

    let mut gemini_client = GeminiClient::new();
    let test_query = "What is the capital of France?";
    let chunk_for_query = crate::gemini::types::GeminiChatChunk {
        image: None,
        is_bot: false,
        user_id: Some("test_user".to_string()),
        guild_id: Some(121212121212),
        channel_id: Some(1234567890), // Example channel ID
        timestamp: chrono::Utc::now().to_string(),
        query: test_query.to_string(),
    };
    let begin_query = get_begin_query("ko".to_string(), "test_user".to_string());
    let use_pro = false; // Set to true if you want to use the pro version
    let ttl:f32 = 12.0; // Time to live in seconds\
    let v_q = vec![chunk_for_query];
    let setting = generate_gemini_cache_setting(v_q.clone(), &begin_query, use_pro, ttl);
    let response = gemini_client.start_gemini_cache(
      v_q, &begin_query, use_pro, ttl)
      .await;
    LOGGER.log(gemini_live_api::libs::logger::LogLevel::Debug, &format!("Starting Gemini cache with query: {}", to_string_pretty(&setting).unwrap()));
    LOGGER.log(gemini_live_api::libs::logger::LogLevel::Debug, &format!("Response: {:?}", response));
    assert!(response.is_ok(), "Failed to start Gemini cache: {:?}", response.err());

    let response = response.unwrap();
  LOGGER.log(gemini_live_api::libs::logger::LogLevel::Debug, &format!("Test response: {:?}", response));
}