use std::env;
use std::ops::Deref;
use reqwest::Client;
use serde_json::json;
use tokio::sync::watch::{Receiver, Ref};
use crate::libs::thread_pipelines::AsyncThreadPipeline;

use crate::libs::logger::{LOGGER, LogLevel};
pub struct GeminiClient<'a, T> where T: Clone {
    net_client: Client,
    pipeline_message_from_discord: &'a AsyncThreadPipeline<T>,
    query_function: fn(T) -> Vec<String>,
}

pub trait GeminiClientTrait<'a, T> where T: Clone {
    fn new(pipe: &'a AsyncThreadPipeline<T>, query_fn: fn(T) -> Vec<String>) -> Self;
    async fn send_query_to_gemini(&mut self, query: Vec<String>) -> Result<String, String>;
    async fn await_for_msg(&mut self);
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

    async fn send_query_to_gemini(&mut self, query: Vec<String>) -> Result<String, String> {
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

        let response_str = response.text().await.map_err(|e| e.to_string())?;

        Ok(response_str)
    }

    async fn await_for_msg(&mut self) {
        let mut pipeline_receiver = self.pipeline_message_from_discord.receiver.clone();
        loop {
            let msg = pipeline_receiver.changed().await.unwrap();
            let msg = pipeline_receiver.borrow_and_update().clone();

            LOGGER.log(LogLevel::Debug, "Received message from Discord pipeline.");
            let querys = (self.query_function)(msg);
            match self.send_query_to_gemini(querys).await {
                Ok(response) => {
                    LOGGER.log(LogLevel::Debug, &format!("Response: {:?}",response));
                }
                Err(e) => {
                    LOGGER.log(LogLevel::Error, &format!("Error: {:?}", e));
                }
            }

        }
    }
}
