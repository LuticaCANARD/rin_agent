use std::collections::{hash_map, HashMap};
use std::hash::Hasher;
use std::{env, hash};
use std::hash::Hash;

use gemini_live_api::types::ThinkingConfig;
use reqwest::Client;
use serde_json::{json, Map, Value};
use serenity::json;

use crate::gemini::utils::generate_gemini_user_chunk;
use crate::libs::logger::{LOGGER, LogLevel};
use crate::service::discord_error_msg::send_debug_error_log;
use crate::setting::gemini_setting::{GEMINI_BOT_TOOLS, GEMINI_BOT_TOOLS_JSON, GEMINI_MODEL_FLASH, GEMINI_MODEL_PRO, GENERATE_CONF, SAFETY_SETTINGS};
use crate::gemini::types::{GeminiChatChunk, GeminiResponse};

use super::types::{GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue, GeminiBotToolInputValueType};
use super::utils::translate_to_gemini_param;

pub struct GeminiClient {
    net_client: Client
}

fn generate_gemini_error_message(error: &str) -> String {
    format!("Gemini API > Error: {}", error)
}

fn make_fncall_result(fn_name:String, origin_argu:Map<String, Value>) -> Value {
    json!({
        "role": "model",
        "parts": [{
            "functionCall":{
                "name": fn_name,
                "args": origin_argu
            }
        }]
    })
}
fn make_fncall_error(fn_name:String, error: String) -> Value {
    json!({
        "role": "user",
        "parts": [{
            "functionResponse":{
                "name": fn_name,
                "response": {
                    "error": {"message": error}
                }
            }
        }]
    })
}
fn make_fncall_result_with_value(fn_name:String, fn_res_json:Value) -> Value {
    json!({ 
            "role": "user", 
            "parts": [{
                "functionResponse":{
                    "name": fn_name,
                    "response": {
                        "result": fn_res_json
                    }
            }
        }]
    })
}

pub trait GeminiClientTrait {
    fn new() -> Self;
    async fn send_query_to_gemini(&mut self, query: Vec<GeminiChatChunk>,begin_query:&GeminiChatChunk,use_pro:bool,thinking_bought:Option<i32>) -> Result<GeminiResponse, String>;
    fn generate_to_gemini_query(&self, query: Vec<GeminiChatChunk>,begin_query:&GeminiChatChunk,thinking_bought:Option<i32>) -> serde_json::Value {
        let generation_conf = if thinking_bought.is_some() {
            let mut origin = GENERATE_CONF.clone();
            origin.thinking_config = Some(
                ThinkingConfig {
                    include_thoughts: true,
                    thinking_budget: thinking_bought.unwrap(),
                }
            );
            origin
        } else {
            GENERATE_CONF.clone()
        };


        json!({
            "contents": 
                query.iter().map(generate_gemini_user_chunk).collect::<Vec<_>>()
            ,
            "systemInstruction": generate_gemini_user_chunk(begin_query),
            "generationConfig": generation_conf,
            "tools": GEMINI_BOT_TOOLS_JSON.clone(),
            "toolConfig":{
                "functionCallingConfig": {"mode": "ANY"},
            },
            "safetySettings": SAFETY_SETTINGS.clone(),
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

    async fn send_query_to_gemini(&mut self, query: Vec<GeminiChatChunk>,begin_query:&GeminiChatChunk,use_pro:bool,thinking_bought:Option<i32>) -> Result<GeminiResponse, String> {
        let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            if use_pro{ GEMINI_MODEL_PRO }else{ GEMINI_MODEL_FLASH },
            api_key
        );

        let objected_query = self.generate_to_gemini_query(query,begin_query,thinking_bought);
        LOGGER.log(LogLevel::Debug, &format!("Gemini API > Req: {}", objected_query));
        let mut integral_content_part = vec![objected_query.get("contents").unwrap().as_array().unwrap().last().unwrap().clone()];
        let response = self.net_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(objected_query.to_string())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let response_result = response.text().await.unwrap();
        LOGGER.log(LogLevel::Debug, &format!("Gemini API > Resp: {}", response_result));
        let mut response_found = false;
        let mut gemini_sending_query = objected_query.clone();

        let mut last_contents = response_result.clone();
        let mut discord_msg = String::new();
        let mut command_result = Vec::new();
        let mut finish_reason = String::new();
        let mut sub_items: Option<Vec<String>> = None;
        let mut avg_logprobs = 0.0;
        let mut thoughts: Option<String> = None;
        let mut hasher = hash::DefaultHasher::new();
        let mut last_hash: u64 = 0;
        last_hash = {
            response_result.hash(&mut hasher);
            hasher.finish()
        };
        



        while response_found == false {
            let now_contents = serde_json::from_str::<Value>(&last_contents)
                .map_err(|e| generate_gemini_error_message(&format!("Failed to parse response: {}", e)))?;

            if now_contents.get("error").is_some() {
                let error_message = now_contents.get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                send_debug_error_log(
                    format!("Gemini API > Error: {}", error_message)
                ).await;
                return Err(generate_gemini_error_message(&error_message));
            }
            let now_parts = now_contents.get("candidates")
                .and_then(|candidates| candidates.as_array())
                .and_then(|candidates| candidates.last())
                .and_then(|candidates| candidates.as_object())
                .and_then(|candidates| candidates.get("content"))
                .and_then(|contents| contents.as_object())
                .and_then(|candidates| candidates.get("parts"))
                .and_then(|parts| parts.as_array());

            if let Some(parts) = now_parts {
                for part in parts {
                    if let Some(fn_call) = part.get("functionCall") {
                        if let Some(fn_name) = fn_call.get("name").and_then(|n| n.as_str()) {
                            if fn_name == "response_msg" {
                                LOGGER.log(LogLevel::Debug, "Gemini API > Function call: response_msg");
                                response_found = true;
                                discord_msg = fn_call.get("args")
                                    .and_then(|args| args.as_object())
                                    .and_then(|args_obj| args_obj.get("msg"))
                                    .and_then(|text| text.as_str())
                                    .map_or_else(|| "".to_string(), |s| s.to_string());
                            } else if fn_name == "sub_items" {
                                LOGGER.log(LogLevel::Debug, "Gemini API > Function call: sub_items");
                                response_found = true;
                                sub_items = fn_call.get("args")
                                    .and_then(|args| args.as_object())
                                    .and_then(|args_obj| args_obj.get("items"))
                                    .and_then(|items| items.as_array())
                                    .map(|items| items.iter()
                                        .filter_map(|item| item.as_str().map(|s| s.to_string()))
                                        .collect());
                            } else {
                                let args = fn_call.get("args")
                                    .and_then(|args| args.as_object())
                                    .cloned()
                                    .unwrap_or_default();
                                
                                if let Some(fn_result) = GEMINI_BOT_TOOLS.get(fn_name) {
                                    let fn_args: HashMap<String, GeminiBotToolInputValue> = args.clone().into_iter()
                                        .map(|(k, v)| {
                                            let value = GeminiBotToolInputValue{
                                                name: k.clone(),
                                                value: match v {
                                                    Value::String(s) => GeminiBotToolInputValueType::String(s),
                                                    Value::Number(n) if n.is_f64() => GeminiBotToolInputValueType::Number(n.as_f64().unwrap()),
                                                    Value::Number(n) if n.is_i64() => GeminiBotToolInputValueType::Integer(n.as_i64().unwrap()),
                                                    Value::Bool(b) => GeminiBotToolInputValueType::Boolean(b),
                                                    Value::Array(arr) => GeminiBotToolInputValueType::Array(arr.into_iter()
                                                        .map(|v| match v {
                                                            Value::String(s) => GeminiBotToolInputValueType::String(s),
                                                            Value::Number(n) if n.is_f64() => GeminiBotToolInputValueType::Number(n.as_f64().unwrap()),
                                                            Value::Number(n) if n.is_i64() => GeminiBotToolInputValueType::Integer(n.as_i64().unwrap()),
                                                            Value::Bool(b) => GeminiBotToolInputValueType::Boolean(b),
                                                            _ => GeminiBotToolInputValueType::Null,
                                                        }).collect()),
                                                    _ => GeminiBotToolInputValueType::Null,
                                                },
                                            };
                                            (k, value)
                                        })
                                        .collect();
                                    integral_content_part.push(make_fncall_result(fn_name.to_string(), args));
                                    let res = (fn_result.action)(fn_args).await;
                                    match res {
                                        Ok(result) => {
                                            command_result.push(Ok(result.clone()));
                                            integral_content_part.push(make_fncall_result_with_value(fn_name.to_string(), json!(result.result)));
                                        },
                                        Err(e) => {
                                            let error_message = e.to_string();
                                            send_debug_error_log(
                                                format!("Gemini API > Function call error: {}", error_message)
                                            ).await;
                                            command_result.push(Err(error_message.clone()));
                                            integral_content_part.push(make_fncall_error(fn_name.to_string(), error_message));
                                        }
                                    }

                                } else {
                                    command_result.push(Err("Gemini API > Function call not found".to_string()));
                                }
                            }
                        } else {
                            send_debug_error_log(
                                "Gemini API > Function call without name".to_string()
                            ).await;
                        }
                    } else if let Some(though) = part.get("thought") {
                        LOGGER.log(LogLevel::Debug, "Gemini API > Thought received");
                        thoughts = part.get("text").and_then(|t| t.as_str()).map(|s| s.to_string());
                        LOGGER.log(LogLevel::Debug, &format!("Gemini API > Thought: {}", thoughts.as_deref().unwrap_or("No thought")));
                    } else {
                        send_debug_error_log(
                            format!("Gemini API > Unexpected response part: {:?}", part)
                        ).await;
                    }
                }



                gemini_sending_query["contents"] = json!(integral_content_part);
                if response_found == false {
                    let response = self.net_client
                        .post(&url)
                        .header("Content-Type", "application/json")
                        .body(gemini_sending_query.to_string())
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;
                    let response_result = response.text().await.unwrap();
                    hasher.write(response_result.as_bytes());
                    let hash_value = hasher.finish();
                    if hash_value == last_hash {
                        send_debug_error_log(
                            format!("Gemini API > No new response received, hash value unchanged: {}", hash_value)
                        ).await;
                        gemini_sending_query["toolConfig"]["functionCallingConfig"]["allowedFunctionNames"] = json!(["response_msg"]);
                    }
                    last_hash = hash_value;
                    last_contents = response_result.clone();
                }

                avg_logprobs = now_contents.get("averageLogprobs")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                finish_reason = now_contents.get("finishReason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
            }
        }

        let thoughts = thoughts;
        let gemini_response = GeminiResponse {
            discord_msg,
            sub_items,
            finish_reason,
            avg_logprobs,
            command_result,
            thoughts,
        };

        Ok(gemini_response)
    }

}

// pub async fn send_query_to_cached_gemini() -> Result<GeminiResponse, String> {

// } 