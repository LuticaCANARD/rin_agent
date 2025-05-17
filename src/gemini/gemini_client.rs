use std::collections::hash_map;
use std::hash::Hasher;
use std::{env, hash};
use std::hash::Hash;

use reqwest::Client;
use serde_json::{json, Value};
use serenity::json;

use crate::gemini::utils::generate_gemini_user_chunk;
use crate::libs::logger::{LOGGER, LogLevel};
use crate::setting::gemini_setting::{get_begin_query, get_gemini_bot_tools, get_gemini_generate_config, GEMINI_BOT_TOOLS, GEMINI_MODEL_FLASH, GEMINI_MODEL_PRO};
use crate::gemini::types::{GeminiChatChunk, GeminiResponse};

use super::types::{GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputType, GeminiBotToolInputValue};
use super::utils::translate_to_gemini_param;

pub struct GeminiClient {
    net_client: Client
}

fn generate_gemini_error_message(error: &str) -> String {
    format!("Gemini API > Error: {}", error)
}

pub trait GeminiClientTrait {
    fn new() -> Self;
    async fn send_query_to_gemini(&mut self, query: Vec<GeminiChatChunk>,begin_query:&GeminiChatChunk,use_pro:bool) -> Result<GeminiResponse, String>;
    fn generate_to_gemini_query(&self, query: Vec<GeminiChatChunk>,begin_query:&GeminiChatChunk) -> serde_json::Value {
        json!({
            "contents": 
                query.iter().map(generate_gemini_user_chunk).collect::<Vec<_>>()
            ,
            "systemInstruction": generate_gemini_user_chunk(begin_query),
            "generationConfig": get_gemini_generate_config(),
            "tools": get_gemini_bot_tools(),
            "toolConfig":{
                "functionCallingConfig": {"mode": "ANY"},
            }
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

    async fn send_query_to_gemini(&mut self, query: Vec<GeminiChatChunk>,begin_query:&GeminiChatChunk,use_pro:bool) -> Result<GeminiResponse, String> {
        let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            if use_pro{ GEMINI_MODEL_PRO }else{ GEMINI_MODEL_FLASH },
            api_key
        );

        let objected_query = self.generate_to_gemini_query(query,begin_query);
        LOGGER.log(LogLevel::Debug, &format!("Gemini API > Req: {}", objected_query));

        let response = self.net_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(objected_query.to_string())
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let response_result = response.text().await.unwrap();
        LOGGER.log(LogLevel::Debug, &format!("Gemini API > Response: {}", response_result));
        let response_str = response_result;
        let response_json: serde_json::Value = serde_json::from_str(&response_str).map_err(|e| e.to_string())?;
        if response_json["error"].is_object() {
            let error_message = response_json["error"]["message"].as_str().unwrap_or("Unknown error");
            LOGGER.log(LogLevel::Error, &format!("Gemini API > Error: {}", error_message));
            return Err(format!("Gemini API > Error: {}", error_message));
        }
        let candidates = response_json["candidates"].as_array();
        if candidates.is_none() {
            return Err("Invalid response format".to_string());
        }
        let candidates = candidates.unwrap();
        if candidates.is_empty() {
            return Err("No candidates found in response".to_string());
        }
        let first_candidate = &candidates[0];
        let content = first_candidate["content"].as_object();
        if content.is_none() {
            return Err(generate_gemini_error_message(
                "There is no content in response"
                )
            )
        }
        let content = content.unwrap();
        if content.contains_key("parts") == false {
            return Err(generate_gemini_error_message(
                "No parts found in response"
                )
            );
        }
        let gemini_response_parts = content["parts"].as_array();
        if gemini_response_parts.is_none() {
            return Err(generate_gemini_error_message(
                "No content [parts] found in response"
                )
            );
        }
        let gemini_response_parts = gemini_response_parts.unwrap().clone();

        let mut parts: Vec<Value> = gemini_response_parts.clone();
        let mut command_result: Vec<Result<GeminiActionResult,String>> = vec![];
        let mut command_try_count = 0;
        let mut hash_command_params = hash::DefaultHasher::new();
        let mut recent_hash= 0;
        let gemini_response_part = gemini_response_parts.last().cloned();
        if gemini_response_part.is_none() {
            return Err("No content found in response".to_string());
        }
        let mut latest_response = gemini_response_part.unwrap();
        while latest_response["functionCall"]["name"].as_str() != Some("response_msg") {
            let mut res_objected_query = objected_query.clone();
            let fn_obj = &latest_response;
            let argus = fn_obj["functionCall"]["args"].as_object();
            if argus.is_none() {
                return Err("Invalid function call format".to_string());
            }
            let argus = argus.unwrap();
            let origin_argu = argus.clone();
            hash_command_params.write(fn_obj.to_string().as_bytes());
            let hash_target = hash_command_params.finish();
            if hash_target != recent_hash {
                hash_command_params = hash::DefaultHasher::new();
                recent_hash = hash_target;
                let argus = argus.iter()
                .map(|(k, v)| {
                    let arg = translate_to_gemini_param(v);
                    let arg = GeminiBotToolInputValue {
                        name: k.clone(),
                        value: arg
                    };
                    (k.clone(), arg)
                })
                .collect::<hash_map::HashMap<String, GeminiBotToolInputValue>>();
                let fn_name = fn_obj["functionCall"]["name"].as_str();
                if fn_name.is_none() {
                    return Err("Invalid function name".to_string());
                }
                let fn_name = fn_name.unwrap();
                let fn_action = GEMINI_BOT_TOOLS.get(fn_name);
                if fn_action.is_none() {
                    return Err(format!("Function {} not found", fn_name));
                }
                let fn_action = fn_action.unwrap().action;

                let fn_res =  (fn_action)(argus).await;
                command_result.push(fn_res.clone());
                parts.push(
                    json!({"role": "model",
                        "parts": [{
                            "functionCall":{
                                "name": fn_name,
                                "args": origin_argu
                            }
                        }]
                    })
                );
                match &fn_res {
                    Err(e) => {
                        LOGGER.log(LogLevel::Error, &format!("Gemini API - FN > Error: {}", e));
                        parts.push(
                            json!({"role": "user",
                                "parts": [{
                                    "functionResponse":{
                                        "name": fn_name,
                                        "response": {
                                            "error": {"message": e}
                                        }
                                    }
                                }]
                            })
                        );
                    },
                    Ok(ok_res) => {
                        let fn_res_val = &ok_res.result;
                        let fn_res_json = serde_json::to_value(fn_res_val).map_err(|e| e.to_string())?;
                        parts.push(
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
                        );
                    }
                }
                let obj_part = json!(parts);
                if obj_part.is_null() {
                    return Err("Invalid response format".to_string());
                }
                let arrayed = res_objected_query["contents"].as_array();
                if arrayed.is_none() {
                    return Err("Invalid response format".to_string());
                }
                let arrayed = arrayed.unwrap();
                let mut a = arrayed.clone();
                a.push(obj_part);
                res_objected_query["contents"] = json!(a);
            }
            if command_try_count > 10 || hash_target == recent_hash {
                LOGGER.log(LogLevel::Error, &format!("Gemini API - FN > Error: {}", "Infinite loop detected"));
                res_objected_query["toolConfig"]["functionCallingConfig"]["allowedFunctionNames"] = json!(["response_msg"]);
            } else {
                command_try_count += 1;
            }
            let response = self.net_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(res_objected_query.to_string())
            .send()
            .await
            .map_err(|e| e.to_string())?;

            if response.status() != 200 {
                LOGGER.log(LogLevel::Error, &format!("Gemini API > Error: {}", response.status()));
                return Err(format!("Gemini API > Error: {}", response.status()));
            }

            let response_result = response.text().await;
            if response_result.is_err() {
                return Err(generate_gemini_error_message(
                    "There is no response from Gemini API"
                    )
                )
            }
            let response_result = response_result.unwrap();

            LOGGER.log(LogLevel::Debug, &format!("Gemini API > cmd Response: {}", response_result));
            let response_result: Result<Value, String> = serde_json::from_str(&response_result)
            .map_err(|e| e.to_string());

            if response_result.is_err() {
                LOGGER.log(LogLevel::Error, &format!("Gemini API > Error: {}", response_result.as_ref().err().unwrap()));
                return Err(format!("Gemini API > Error: {}", response_result.as_ref().err().unwrap()));
            }

            let response_result = response_result.unwrap();
            if response_result["error"].is_object() {
                let error_message = response_result["error"]["message"].as_str().unwrap_or("Unknown error");
                LOGGER.log(LogLevel::Error, &format!("Gemini API > Error: {}", error_message));
                return Err(format!("Gemini API > Error: {}", error_message));
            }

            let candidates = response_result["candidates"].as_array();
            if candidates.is_none() {
                return Err("Invalid response format".to_string());
            }
            let candidates = candidates.unwrap();
            if candidates.is_empty() {
                return Err("No candidates found in response".to_string());
            }
            
            let first_candidate = &candidates[0];

            let content = first_candidate["content"].as_object();
            if content.is_none() {
                return Err("No content found in response".to_string());
            }
            let content = content.unwrap();
            let gemini_response_part = content["parts"].as_array();
            if gemini_response_part.is_none() {
                return Err("No content found in response".to_string());
            }
            let gemini_response_part = gemini_response_part.unwrap();
            if gemini_response_part.is_empty() {
                return Err("No content found in response".to_string());
            } else if gemini_response_part.len() == 0 {
                return Err("No content found in response in array!".to_string());
            }

            let content = gemini_response_part.last();
            if content.is_none() {
                return Err("No content found in response".to_string());
            }
            let content = content.unwrap();
            latest_response = content.clone();
            
        }
        
        let last_part = &latest_response["functionCall"];
        if last_part.is_null() {
            return Err("No content found in response".to_string());
        }
        LOGGER.log(LogLevel::Debug, &format!("Gemini API > cmd Response - last msg: {}", last_part.to_string()));
        let last_argus = last_part["args"].as_object();
        if last_argus.is_none() {
            return Err("Invalid function call format".to_string());
        }
        let last_argus = last_argus.unwrap();

        let sub_items = if last_argus.get_key_value("subItems") != None {
            Some(last_argus["subItems"]
            .as_array()
            .expect("there is no subitem")
            .iter()
            .filter_map(|item| item.as_str())
            .map(|s| s.to_string())
            .collect::<Vec<String>>())
        } else {
            None
        };
        
        
        let discord_msg = if last_argus.get_key_value("msg") != None {
            Ok(last_argus["msg"]
            .to_string())
        }  else {
            Err("No content found in response".to_string())
        };
        if let Err(e) = discord_msg { return Err(e); }

        let discord_msg = discord_msg.unwrap();

        let finish_reason = first_candidate["finishReason"].as_str().unwrap_or("").to_string();
        
        let avg_logprobs = first_candidate["avgLogprobs"].as_f64().unwrap_or(0.0);

        let gemini_response = GeminiResponse {
            discord_msg,
            sub_items,
            finish_reason,
            avg_logprobs,
            command_result,
        };

        Ok(gemini_response)
    }

}

// pub async fn send_query_to_cached_gemini() -> Result<GeminiResponse, String> {

// } 