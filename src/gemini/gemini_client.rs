use std::env;
use std::ops::Deref;
use reqwest::Client;
use serde_json::{json, Value};
use tokio::sync::watch::{Receiver, Ref};
use crate::libs::thread_pipelines::AsyncThreadPipeline;

use crate::libs::logger::{LOGGER, LogLevel};
use crate::setting::gemini_setting::{get_gemini_generate_config, GEMINI_MODEL};

#[derive(Debug, Clone)]
pub struct GeminiResponse {
    pub discord_msg: String,
    pub sub_items: Vec<String>,
    pub finish_reason: String,
    pub avg_logprobs: f64,
}
#[derive(Debug, Clone)]
pub struct GeminiImageInputType {
    pub base64_image: String,
    // e.g. "image/png", "image/jpeg"
    pub mime_type: String,
}
#[derive(Debug, Clone)]
pub struct GeminiChatChunk {
    pub query: String,
    pub image: Option<GeminiImageInputType>,
    pub is_bot: bool,
    pub user_id: Option<String>, 
}


pub struct GeminiClient {
    net_client: Client
}


pub trait GeminiClientTrait {
    fn new() -> Self;
    async fn send_query_to_gemini(&mut self, query: Vec<GeminiChatChunk>) -> Result<GeminiResponse, String>;
    fn generate_to_gemini_query(&self, query: Vec<GeminiChatChunk>) -> serde_json::Value {
        json!({
            "contents": [
                query.iter().map(|chunk| {
                if chunk.image.is_none() {
                    json!({
                        "role" : if chunk.is_bot {"model" } else {"user"},
                        "parts": [{ "text": chunk.query,}]
                    })
                } else {
                    json!({
                        "role" : if chunk.is_bot {"model"} else {"user"},
                        "parts": [{"text": chunk.query},
                            {
                                "inline_data": {
                                    "mime_type": chunk.image.as_ref().map(|img| img.mime_type.clone()).unwrap_or_default(),
                                    "data": chunk.image.as_ref().map(|img| img.base64_image.clone()).unwrap_or_default()
                                }
                            }
                        ]
                    })
                }
            }).collect::<Vec<_>>()
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

    async fn send_query_to_gemini(&mut self, query: Vec<GeminiChatChunk>) -> Result<GeminiResponse, String> {
        let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            GEMINI_MODEL,
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
        let content = text[text.len()-1].as_object().ok_or("1Invalid response format")?;
        if content.get_key_value("subItems") != None  {
          sub_items = content["subItems"].as_array().ok_or("2Invalid response format")?;
        }
        let sub_items: Vec<String> = sub_items.iter()
            .filter_map(|item| item.as_str())
            .map(|s| s.to_string())
            .collect();
        let discord_msg = content["discordMessage"].as_str().ok_or("3Invalid response format")?.to_string();

        let finish_reason = first_candidate["finishReason"].as_str().unwrap_or("").to_string();
        let avg_logprobs = first_candidate["avgLogprobs"].as_f64().unwrap_or(0.0);
        let gemini_response = GeminiResponse {
            discord_msg,
            sub_items,
            finish_reason,
            avg_logprobs,
        };

        Ok(gemini_response)
    }

}
