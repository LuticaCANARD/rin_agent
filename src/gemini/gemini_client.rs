use std::env;

use reqwest::Client;
use serde_json::{json, Value};
use serenity::json;

use crate::gemini::utils::generate_gemini_user_chunk;
use crate::libs::logger::{LOGGER, LogLevel};
use crate::setting::gemini_setting::{get_gemini_generate_config, GEMINI_MODEL_FLASH, GEMINI_MODEL_PRO};
use crate::gemini::types::{GeminiChatChunk, GeminiResponse};

pub struct GeminiClient {
    net_client: Client
}

pub trait GeminiClientTrait {
    fn new() -> Self;
    async fn send_query_to_gemini(&mut self, query: Vec<GeminiChatChunk>,use_pro:bool) -> Result<GeminiResponse, String>;
    fn generate_to_gemini_query(&self, query: Vec<GeminiChatChunk>) -> serde_json::Value {
        json!({
            "contents": [
                query.iter().map(generate_gemini_user_chunk).collect::<Vec<_>>()
            ],
            "generationConfig": get_gemini_generate_config()
        })
    }
}
impl GeminiClientTrait for GeminiClient {
    fn new() -> Self {
        GeminiClient {
            net_client: Client::new(),
        }
    }
/* curl "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=${GEMINI_API_KEY}" \
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
    ],
    "generationConfig": {
        "responseMimeType": "application/json",
        "responseSchema": {
          "type": "ARRAY",
          "items": {
            "type": "OBJECT",
            "properties": {
              "recipeName": { "type": "STRING" },
              "ingredients": {
                "type": "ARRAY",
                "items": { "type": "STRING" }
              }
            },
            "propertyOrdering": ["recipeName", "ingredients"]
          }
        }
      }
}'
* 
*/

    async fn send_query_to_gemini(&mut self, query: Vec<GeminiChatChunk>,use_pro:bool) -> Result<GeminiResponse, String> {
        let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            if use_pro{ GEMINI_MODEL_PRO }else{ GEMINI_MODEL_FLASH },
            api_key
        );
        let objected_query = self.generate_to_gemini_query(query);
        let response = self.net_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(objected_query.to_string())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let response_result = response.text().await.unwrap();
        let response_str = response_result;
        let response_json: serde_json::Value = serde_json::from_str(&response_str).map_err(|e| e.to_string())?;
        let candidates = response_json["candidates"].as_array().ok_or("DInvalid response format")?;
        if candidates.is_empty() {
            return Err("No candidates found in response".to_string());
        }
        let first_candidate = &candidates[0];
        let content = first_candidate["content"].as_object().ok_or("CInvalid response format")?;
        let parts = content["parts"].as_array().ok_or("BInvalid response format")?;
        let last_end = parts.len() - 1;
        let last_part = &parts[last_end];
        let text = last_part["text"].as_str().ok_or("AInvalid response format")?;

        let text_objed = serde_json::from_str::<serde_json::Value>(text).map_err(|e| e.to_string())?;
        let text = text_objed.as_array().ok_or("Invalid response format")?;
        if text.is_empty() {
            return Err("No text found in response".to_string());
        }

        let mut sub_items:&Vec<Value> = &vec![];
        if text.len() < 1 {
            return Err("No text found in response".to_string());
        }
        let content = text[text.len()-1].as_object().ok_or("1Invalid response format")?;
        if content.get_key_value("subItems") != None  {
            sub_items = content["subItems"].as_array().ok_or("2Invalid response format")?;
        }
        let sub_items: Vec<String> = sub_items.iter()
            .filter_map(|item| item.as_str())
            .map(|s| s.to_string())
            .collect();
        let discord_msg = content.get_key_value("discordMessage");
        if discord_msg == None {
            LOGGER.log(LogLevel::Error, "Gemini API > No discordMessage found in response");
            return Err("No discordMessage found in response".to_string());
        }
        let discord_msg = discord_msg.unwrap().1.as_str().ok_or("Invalid discordMessage format")?.to_string();

        let finish_reason = first_candidate["finishReason"].as_str().unwrap_or("").to_string();
        let avg_logprobs = first_candidate["avgLogprobs"].as_f64().unwrap_or(0.0);

        let commnds = first_candidate.get("userCommand");
        let commands = match commnds {
            Some(commands) => {
                let commands = commands.as_array().ok_or("Invalid userCommand format")?;
                let commands: Vec<String> = commands.iter()
                    .filter_map(|item| item.as_str())
                    .map(|s| s.to_string())
                    .collect();
                Some(commands)
            }
            None => None,
        };
        let gemini_response = GeminiResponse {
            discord_msg,
            sub_items,
            finish_reason,
            avg_logprobs,
            commands,
        };

        Ok(gemini_response)
    }

}