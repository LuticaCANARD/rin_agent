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
        let candidates = response_json["candidates"].as_array().ok_or("DInvalid response format")?;
        if candidates.is_empty() {
            return Err("No candidates found in response".to_string());
        }
        let first_candidate = &candidates[0];
        let content = first_candidate["content"].as_object().ok_or("CInvalid response format")?;
        let mut gemini_responese_part = content["parts"].as_array().ok_or("BInvalid response format")?.clone();
        let mut parts: Vec<Value> = vec![];
        let mut command_result: Vec<Result<GeminiActionResult,String>> = vec![];
        let mut command_try_count = 0;
        let mut hash_command_params = hash::DefaultHasher::new();
        let mut recent_hash= 0;
        
        while gemini_responese_part.last().unwrap()["functionCall"]["name"].as_str() != Some("response_msg") {
            let mut res_objected_query = objected_query.clone();
            let fn_obj = gemini_responese_part.last();
            let argus = gemini_responese_part.last().unwrap()["functionCall"]["args"].as_object().ok_or("Invalid function call format")?;
            let origin_argu = argus.clone();
            hash_command_params.write(fn_obj.unwrap().to_string().as_bytes());
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
                let fn_name = gemini_responese_part.last().unwrap()["functionCall"]["name"].as_str().ok_or("Invalid function name")?;
                let fn_name = fn_name.to_string();
                let fn_res =  (GEMINI_BOT_TOOLS.get(fn_obj.unwrap()["functionCall"]["name"].as_str().unwrap()).ok_or("Invalid function name")?.action)(argus).await;
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
                let mut a = res_objected_query["contents"].as_array().unwrap().clone();
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
            let response_result = response.text().await.unwrap();
            LOGGER.log(LogLevel::Debug, &format!("Gemini API > cmd Response: {}", response_result));
            let response_result: Result<Value, String> = serde_json::from_str(&response_result).map_err(|e| e.to_string());
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
            let candidates = response_result["candidates"].as_array().ok_or("Invalid response format")?;
            if candidates.is_empty() {
                return Err("No candidates found in response".to_string());
            }
            let first_candidate = &candidates[0];
            let content = first_candidate["content"]["parts"].as_array().ok_or("Invalid response format")?.last().unwrap();
            gemini_responese_part.push(content.clone());

        }
        

        let last_part = &gemini_responese_part.last().unwrap()["functionCall"];
        LOGGER.log(LogLevel::Debug, &format!("Gemini API > cmd Response - last msg: {}", last_part.to_string()));
        let last_argus = last_part["args"].as_object().ok_or("Invalid function call format")?;

        let text = last_argus["msg"].as_str().ok_or("AInvalid response format")?;

        let mut sub_items:&Vec<Value> = &vec![];
        if text.len() < 1 {
            return Err("No text found in response".to_string());
        }
        if last_argus.get_key_value("subItems") != None  {
            sub_items = content["subItems"].as_array().ok_or("2Invalid response format")?;
        }
        let sub_items: Vec<String> = sub_items.iter()
            .filter_map(|item| item.as_str())
            .map(|s| s.to_string())
            .collect();
        let discord_msg = text.to_string();

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