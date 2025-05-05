use std::env;
use std::ops::Deref;
use reqwest::Client;
use serde_json::json;
use tokio::sync::watch::{Receiver, Ref};
use crate::libs::thread_pipelines::AsyncThreadPipeline;

use crate::libs::logger::{LOGGER, LogLevel};

#[derive(Debug, Clone)]
pub struct GeminiResponse {
    pub content: String,
    pub finish_reason: String,
    pub avg_logprobs: f64,
}

pub struct GeminiClient<'a, T> where T: Clone {
    net_client: Client,
    pipeline_message_from_discord: &'a AsyncThreadPipeline<T>,
    query_function: fn(T) -> Vec<String>,
}

pub trait GeminiClientTrait<'a, T> where T: Clone {
    fn new(pipe: &'a AsyncThreadPipeline<T>, query_fn: fn(T) -> Vec<String>) -> Self;
    async fn send_query_to_gemini(&mut self, query: Vec<String>) -> Result<GeminiResponse, String>;
}
impl<'a,T> GeminiClientTrait<'a,T> for GeminiClient<'a,T> where T: Clone {
    fn new(pipe:&'a AsyncThreadPipeline<T>,query_fn: fn(T) -> Vec<String>) -> Self {
        GeminiClient {
            net_client: Client::new(),
            pipeline_message_from_discord: pipe,
            query_function: query_fn,
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
    ]
}'
* 
*/

    async fn send_query_to_gemini(&mut self, query: Vec<String>) -> Result<GeminiResponse, String> {
        let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
            api_key
        );
        let objected_query = json!({
            "contents": [
                {
                    "parts": query.iter().map(|q| {
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
/*
 {
  "candidates": [
    {
      "content": {
        "parts": [
          {
            "text": "안녕하세요! 무엇을 도와드릴까요? 😊\n"
          }
        ],
        "role": "model"
      },
      "finishReason": "STOP",
      "avgLogprobs": -0.12830255925655365
    }
  ],
  "usageMetadata": {
    "promptTokenCount": 3,
    "candidatesTokenCount": 16,
    "totalTokenCount": 19,
    "promptTokensDetails": [
      {
        "modality": "TEXT",
        "tokenCount": 3
      }
    ],
    "candidatesTokensDetails": [
      {
        "modality": "TEXT",
        "tokenCount": 16
      }
    ]
  },
  "modelVersion": "gemini-2.0-flash"
}
 */
        let response_str = response.text().await.map_err(|e| e.to_string())?;
        let response_json: serde_json::Value = serde_json::from_str(&response_str).map_err(|e| e.to_string())?;
        let candidates = response_json["candidates"].as_array().ok_or("Invalid response format")?;
        if candidates.is_empty() {
            return Err("No candidates found in response".to_string());
        }
        let first_candidate = &candidates[0];
        let content = first_candidate["content"].as_object().ok_or("Invalid response format")?;
        let parts = content["parts"].as_array().ok_or("Invalid response format")?;
        let last_end = parts.len() - 1;
        let last_part = &parts[last_end];
        let text = last_part["text"].as_str().ok_or("Invalid response format")?;
        let response_str = text.to_string(); 
        
        let finish_reason = first_candidate["finishReason"].as_str().unwrap_or("").to_string();
        let avg_logprobs = first_candidate["avgLogprobs"].as_f64().unwrap_or(0.0);
        let gemini_response = GeminiResponse {
            content: response_str.clone(),
            finish_reason,
            avg_logprobs,
        };

        Ok(gemini_response)
    }

}
