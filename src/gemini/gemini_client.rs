use std::env;
use reqwest::Client;
use serde_json::json;

pub struct GeminiClient {
    net_client: Client,
}

trait GeminiClientTrait {
    fn new() -> Self;
    async fn send_query_to_gemini(&mut self, query: Vec<&str>) -> Result<String, String>;
    
}
impl GeminiClientTrait for GeminiClient {
    fn new() -> Self {
        GeminiClient {
            net_client: Client::new()
        }
    }
/**
 * curl "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=${GEMINI_API_KEY}" \
-H 'Content-Type: application/json' \
-X POST \
-d '{
    "contents": [
    {
        "parts": [
        {
            "text": "Write a story about a magic backpack."
        }
        ]
    }
    ]
}'
* 
*/
    async fn send_query_to_gemini(&mut self, query: Vec<&str>) -> Result<String, String> {
        let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}", api_key);
        let objected_query = json!({
            "contents": [
                {
                    "parts": query.iter().map(|&q| {
                        json!({ "text": q })
                    }).collect::<Vec<_>>()
                }
            ]
        });
        
        let response = self.net_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(objected_query.to_string())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        
        let response_str = response.text().await.map_err(|e| e.to_string())?;

        Ok(response_str)
    }
} 