use std::collections::{hash_map, BTreeMap, HashMap};
use std::fmt::format;
use std::hash::Hasher;
use std::{env, hash};
use std::hash::Hash;

use gemini_live_api::types::enums::GeminiContentRole;
use gemini_live_api::types::{GeminiCachedContent, GeminiCachedContentResponse, GeminiContents, GeminiExecutableCode, GeminiExecutableCodeResult, GeminiFileData, GeminiFunctionCall, GeminiFunctionCallingConfig, GeminiFunctionResponse, GeminiGenerationConfigTool, GeminiInlineBlob, GeminiParts, GeminiToolConfig, GeminiToolConfigMode, ThinkingConfig};
use reqwest::Client;
use sea_orm::sea_query::IdenList;
use serde_json::{json, Map, Value};
use serenity::all::{ChannelId, CreateMessage};
use crate::discord::discord_bot_manager::{get_discord_service, BotManager};
use crate::gemini::utils::generate_gemini_user_chunk;
use crate::libs::logger::{LOGGER, LogLevel};
use crate::libs::thread_pipelines::{GeminiChannelResult, GEMINI_FUNCTION_EXECUTION_ALARM};
use crate::service::discord_error_msg::send_debug_error_log;
use crate::setting::gemini_setting::{GEMINI_BOT_TOOLS, GEMINI_BOT_TOOLS_JSON, GEMINI_BOT_TOOLS_MODULES, GEMINI_MODEL_FLASH, GEMINI_MODEL_PRO, GENERATE_CONF, SAFETY_SETTINGS};
use crate::gemini::types::{GeminiChatChunk, GeminiResponse};

use super::types::{DiscordUserInfo, GeminiBotToolInputValue, GeminiBotToolInputValueType};

pub struct GeminiCacheInfo {
    pub cached_key : String,
}

pub struct GeminiClient {
    net_client: Client,
}

fn generate_gemini_error_message(error: &str) -> String {
    format!("Gemini API > Error: {}", error)
}

fn make_fncall_result(fn_name:String, origin_argu:Map<String, Value>) -> GeminiContents {
    let origin_argu_btree: BTreeMap<String, Value> = origin_argu.into_iter().collect();
    let function_execution_result = GeminiParts::new().set_function_call(
        GeminiFunctionCall {
            name: fn_name.clone(),
            id: None,
            args: Some(origin_argu_btree),
        }
    );
    GeminiContents{
        role: GeminiContentRole::User,
        parts: vec![
            function_execution_result,
        ]
    }
}
fn make_fncall_error(fn_name:String, error: String) -> GeminiContents {

    let function_execution_result = GeminiParts::new().set_function_response(
        GeminiFunctionResponse {
            name: fn_name.clone(),
            response: Some(
                json!({
                    "error": {
                        "message": error
                    }
                })
            ),
            id: None,
            will_continue:None,
            scheduling: None,
        }
    );
    GeminiContents{
        role: GeminiContentRole::User,
        parts: vec![
            function_execution_result,
        ]
    }
}
fn make_fncall_result_with_value(fn_res:GeminiFunctionResponse) -> GeminiContents {
    let function_execution_result = GeminiParts::new().set_function_response(fn_res);

    GeminiContents{
        role: GeminiContentRole::Model,
        parts: vec![
            function_execution_result,
        ]
    }
}


fn generate_to_value(k:String,v:Value) -> (String, GeminiBotToolInputValue) {
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
} 

pub fn generate_gemini_cache_setting(
    query: Vec<GeminiChatChunk>,
    begin_query: &GeminiChatChunk,
    use_pro: bool,
    ttl: f32
) -> GeminiCachedContent {
    GeminiCachedContent {
        contents: query.iter().map(generate_gemini_user_chunk).collect(),
        system_instruction: Some(generate_gemini_user_chunk(begin_query)),
        tools: GEMINI_BOT_TOOLS_JSON.clone(),
        tool_config: Some(GeminiToolConfig{
            function_calling_config: Some(
                GeminiFunctionCallingConfig {
                    mode:Some(GeminiToolConfigMode::Any),
                    allowed_function_names:None
                }
            ),
        }),
        ttl : format!("{:.7}s", ttl),
        model: format!("models/{}", if use_pro { GEMINI_MODEL_PRO } else { GEMINI_MODEL_FLASH}).to_string(),
        display_name: None,
    }
}

fn generate_value_to_content(value: &Value) -> Result<GeminiParts, String> {
    if let Some(v) = value.get("text") {
        Ok(GeminiParts::new().set_text(v.as_str().unwrap_or("").to_string()))
    } else if let Some(v) = value.get("inlineData") {
        let inline_data = serde_json::from_value::<GeminiInlineBlob>(v.clone()).unwrap();
        Ok(GeminiParts::new().set_inline_data(inline_data))
    }  else if let Some(v) = value.get("functionCall") {
        let function_call = match v.as_object() {
            Some(obj) => obj,
            None => Err(()).expect("Expected functionCall to be an object")
        };
        if function_call.get("name").is_none() {
            return Err("Function call without name".to_string());
        }
        let name = function_call.get("name").and_then(|n| n.as_str())
            .unwrap_or_else(|| {
                "There is no function name in the function call"
            });
        let args = function_call.get("args").cloned().unwrap_or(Value::Null);
        Ok(GeminiParts::new().set_function_call(
            GeminiFunctionCall {
                name: name.to_string(),
                id: None,
                args: Some(args.as_object().unwrap().clone().into_iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .into_iter().collect::<BTreeMap<_, _>>())
                }
            )
        )
    } else if let Some(v) = value.get("functionResponse") {
        let function_res = serde_json::from_value::<GeminiFunctionResponse>(v.clone()).unwrap();
        Ok(GeminiParts::new().set_function_response(
            function_res.clone()
        ))
    } else if let Some(v) = value.get("fileData") {
        let inline_data = serde_json::from_value::<GeminiFileData>(v.clone()).unwrap();
        Ok(GeminiParts::new().set_file_data(inline_data))
    } else if let Some(v) = value.get("executableCode") {
        let code = serde_json::from_value::<GeminiExecutableCode>(v.clone()).unwrap();
        Ok(GeminiParts::new().set_executable_code(
            code
        ))
    } else if let Some(v) =   value.get("executableCodeResult") {
        let code = serde_json::from_value::<GeminiExecutableCodeResult>(v.clone()).unwrap();
        Ok(GeminiParts::new().set_code_execution_result(
            code
        ))
    } else if let Some(v) = value.get("image") {
        Ok(GeminiParts::new().set_image_link(v.as_str().unwrap_or_else(|| {
            "There is no image link in the image data"
        }).to_string()))
    } else {
        Ok(GeminiParts::new().set_text(value.to_string()))
    }
}


pub trait GeminiClientTrait {
    async fn start_gemini_cache(&mut self, query: Vec<GeminiChatChunk>, begin_query: &GeminiChatChunk, use_pro: bool, ttl:f32) -> 
    Result<GeminiCachedContentResponse, String>;
    fn new() -> Self;
    async fn send_query_to_gemini(&mut self, query: Vec<GeminiChatChunk>,begin_query:&GeminiChatChunk,
        use_pro:bool,
        thinking_bought:Option<i32>,
        cached:Option<String>,
        user_info:Option<DiscordUserInfo>
) -> Result<GeminiResponse, String>;
    fn generate_to_gemini_query(&self, query: Vec<GeminiChatChunk>,
        begin_query:&GeminiChatChunk,thinking_bought:Option<i32>,
        cached:Option<String>,is_start:bool) -> Value {
        let generation_conf = if thinking_bought.is_some() {
            let mut origin = GENERATE_CONF.clone();
            origin.thinking_config = Some(
                ThinkingConfig {
                    include_thoughts: true,
                    thinking_budget: thinking_bought.unwrap_or(1000),
                }
            );
            origin
        } else {
            GENERATE_CONF.clone()
        };
        let cached_info = cached;
        let ret = if is_start {
            json!({
            "contents": query.iter().map(generate_gemini_user_chunk).collect::<Vec<_>>(),
            "generationConfig": generation_conf,
            "safetySettings": SAFETY_SETTINGS.clone(),
            "toolConfig": {
                "functionCallingConfig": {
                    "mode": "ANY",
                    "allowedFunctionNames": GEMINI_BOT_TOOLS_MODULES.iter().map(|tool| tool.name.clone()).collect::<Vec<_>>()
                }
            },
            "systemInstruction": generate_gemini_user_chunk(begin_query),
            "tools": GEMINI_BOT_TOOLS_JSON.clone()
        })
        } else {
            json!({
            "contents": query.iter().map(generate_gemini_user_chunk).collect::<Vec<_>>(),
            "generationConfig": generation_conf,
            "safetySettings": SAFETY_SETTINGS.clone()
        })
        };
        if let Some(cached_key) = cached_info {
            let mut ret_map = ret.as_object().unwrap().clone();
            ret_map.insert("cachedContent".to_string(), Value::String(cached_key));
            Value::Object(ret_map)
        } else {
            ret
        }
    }

    async fn drop_cache(&mut self, cache_key: &str) -> Result<(), String>;
}
impl GeminiClientTrait for GeminiClient {
    fn new() -> Self {
        GeminiClient {
            net_client: Client::new(),
        }
    }
    async fn send_query_to_gemini(
        &mut self, 
        query: Vec<GeminiChatChunk>,
        begin_query:&GeminiChatChunk,
        use_pro:bool,
        thinking_bought:Option<i32>,
        cached:Option<String>,
        user_info:Option<DiscordUserInfo>
    ) -> Result<GeminiResponse, String> {
        let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            if use_pro{ GEMINI_MODEL_PRO }else{ GEMINI_MODEL_FLASH },
            api_key
        );
        let objected_query = self.generate_to_gemini_query(query,begin_query,thinking_bought,cached.clone(),cached.is_none());
        
        LOGGER.log(LogLevel::Debug, &format!("Gemini API > Req: {}", objected_query));
        let contents = objected_query.get("contents");
        let mut integral_content_part:Vec<GeminiContents> = if contents.is_some() && contents.unwrap().as_array().is_some() {
            let contents = contents.unwrap();
            contents.as_array().unwrap().into_iter().map(|c| {
                GeminiContents {
                    role : if c.get("role").is_some_and(|r| r.as_str() == Some("user")) {
                        GeminiContentRole::User
                    } else {
                        GeminiContentRole::Model
                    },
                    parts: c.get("parts").and_then(|p| p.as_array())
                        .map_or_else(|| vec![GeminiParts::new().set_text(c.to_string())], |parts| {
                            parts.iter().map(generate_value_to_content).filter_map(
                                |part| part.ok()
                            ).collect()
                        }),
                }
            }).collect::<Vec<GeminiContents>>()
        } else {
            vec![]
        };
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
        let mut trycount = 0;
        let mut thoughts: Option<String> = None;
        let mut hasher = hash::DefaultHasher::new();
        let mut last_hash: u64 = {
            response_result.hash(&mut hasher);
            hasher.finish()
        };
        while response_found == false && trycount < 10 {
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

            LOGGER.log(LogLevel::Debug, &format!("Gemini API > Response: {}", serde_json::to_string_pretty(&now_contents).unwrap_or_default()));
            let now_parts = now_contents.get("candidates")
                .and_then(|candidates| candidates.as_array())
                .and_then(|candidates| candidates.last())
                .and_then(|candidates| candidates.as_object())
                .and_then(|candidates| candidates.get("content"))
                .and_then(|contents| contents.as_object())
                .and_then(|candidates| candidates.get("parts"))
                .and_then(|parts| parts.as_array());
            let mut response_message_id:Option<u64> = None;
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
                                        .map(|(k,v)| generate_to_value(k,v))
                                        .collect();
                                    integral_content_part.push(make_fncall_result(fn_name.to_string(), args));
                                    let res = (fn_result.action)(fn_args,user_info.clone()).await;
                                    match res {
                                        Ok(result) => {
                                            command_result.push(Ok(result.clone()));
                                            integral_content_part.push(make_fncall_result_with_value(
                                                GeminiFunctionResponse {
                                                    name: fn_name.to_string(),
                                                    response: Some(json!(result.clone())),
                                                    id: None,
                                                    will_continue:None,
                                                    scheduling: None,
                                                }
                                            )
                                        );
                                        if(response_message_id.is_none()) {
                                            let msg = result.clone().result_message;
                                            let channel = ChannelId::new(begin_query.channel_id.unwrap_or(0));
                                            let _discord_client = get_discord_service().await;
                                            let locked = _discord_client.call::<BotManager>().unwrap();
                                            let discord_client = locked.try_lock().unwrap();
                                            let sent = discord_client.send_message(channel, CreateMessage::new().content(msg)).await;
                                            if let Ok(sent_msg) = sent {
                                                response_message_id = Some(sent_msg.id.get());
                                            } else {
                                                send_debug_error_log(
                                                    format!("Gemini API > Failed to send Discord message for function {}: {:?}", fn_name, sent.err())
                                                ).await;
                                            }
                                        } else {
                                            let _ = GEMINI_FUNCTION_EXECUTION_ALARM.sender.send(
                                            GeminiChannelResult{
                                                message: result.clone(),
                                                channel_id: begin_query.channel_id.unwrap().to_string(),
                                                sender: begin_query.user_id.clone().unwrap().clone(),
                                                guild_id: begin_query.guild_id.unwrap().to_string(),
                                                message_id: fn_name.to_string(),
                                            });
                                        }
                                        
                                        },
                                        Err(e) => {
                                            let error_message = e.to_string();
                                            send_debug_error_log(
                                                format!("Gemini API > Function call error: {}, at : {}", error_message, fn_name)
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
                    } else if let Some(text) = part.get("text") {
                        LOGGER.log(LogLevel::Debug, "Gemini API > Text received");
                        let text_content = text.as_str().unwrap_or("");
                        if !text_content.is_empty() {
                            integral_content_part.push(
                                GeminiContents {
                                    role: GeminiContentRole::Model,
                                    parts: vec![
                                        GeminiParts::new().set_text(text_content.to_string())
                                    ],
                                }
                            );
                        }
                        response_found = true;
                        discord_msg = text_content.to_string();
                    } else if let Some(image) = part.get("image") {
                        LOGGER.log(LogLevel::Debug, "Gemini API > Image received");
                        integral_content_part.push(
                            GeminiContents {
                                role: GeminiContentRole::Model,
                                parts: vec![
                                    GeminiParts::new().set_image_link(
                                        image.to_string()
                                    )
                                ]
                            }
                        );
                    } else {
                        send_debug_error_log(
                            format!("Gemini API > Unexpected response part: {:?}", part)
                        ).await;
                    }
                }
                trycount += 1;
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
                    let body_jsoned = serde_json::from_str::<Value>(&response_result).unwrap()
                        .get("candidates")
                        .and_then(|candidates| candidates.as_array())
                        .and_then(|candidates| candidates.last())
                        .and_then(|candidates| candidates.as_object())
                        .and_then(|candidates| candidates.get("content"))
                        .and_then(|contents| contents.as_object())
                        .and_then(|candidates| candidates.get("parts"))
                        .and_then(|parts| parts.as_array())
                        .map(|arr| {
                            arr.iter().filter(|p| {
                                p.as_object()
                                    .map(|obj| obj.get("thought").is_none())
                                    .unwrap_or(true)
                            }).collect::<Vec<_>>()
                        })
                        .map(|vec| serde_json::to_string(&vec).unwrap_or_default()) // <-- 변경
                        .unwrap_or_default();

                    hasher.write(body_jsoned.as_bytes());
                    let hash_value = hasher.finish();
                    if hash_value == last_hash && trycount > 10 {
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

    async fn start_gemini_cache(
        &mut self, 
        query: Vec<GeminiChatChunk>,
        begin_query:&GeminiChatChunk,
        use_pro:bool,
        ttl: f32
    ) -> Result<GeminiCachedContentResponse, String> {
        let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/cachedContents?key={}",
            api_key
        );
        let start_cache = generate_gemini_cache_setting(query, begin_query, use_pro, ttl);
        let body = serde_json::to_string(&start_cache).expect("Failed to serialize cache content");
        LOGGER.log(LogLevel::Debug, &format!("Gemini Cache API > Start post Req: {:?}", &body));
        let response = self.net_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await;
        match response {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let resp_status = resp.status();
                    let reason = resp.text().await.unwrap_or_else(|_| "No response text".to_string());
                    let error_message = format!(
                        "Gemini API > Send Post Cache > Error_status: {} / {}", resp_status, 
                        reason
                    );
                    if reason.contains("Cached content is too small") {
                        return Err(format!("Gemini API > Error: {}", reason));
                    } else {
                        send_debug_error_log(error_message.clone()).await;
                        return Err(error_message);
                    }

                }
                let rest_text = resp.text().await.map_err(|e| format!("Failed to read response text: {}", e))?;

                LOGGER.log(LogLevel::Debug, &format!("Gemini API > Resp: {}", rest_text));
                let response_result = serde_json::from_str::<GeminiCachedContentResponse>(
                    rest_text.as_str()
                ).map_err(|e| format!("Failed to parse response: {}", e))?;
                LOGGER.log(LogLevel::Debug, &format!("Gemini API > Cache created: {:?}", response_result));
                Ok(response_result)
            },
            Err(e) => {
                LOGGER.log(LogLevel::Error, &format!("Gemini API > Error: {}", e));
                send_debug_error_log(
                    format!("Gemini API > Error: {}", e)
                ).await;
                Err(format!("Gemini API > Error: {}", e))
            }
        }
    }
    async fn drop_cache(&mut self, cache_key: &str) -> Result<(), String>
    {
        let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/{}?key={}",
            cache_key, api_key
        );
        let response = self.net_client
            .delete(&url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            let error_message = format!(
                "Gemini API > Drop Cache > Error_status: {} / {}", response.status(), 
                response.text().await.unwrap_or_else(|_| "No response text".to_string())
            );
            send_debug_error_log(error_message.clone()).await;
            return Err(error_message);
        }
        LOGGER.log(LogLevel::Debug, &format!("Gemini API > Cache dropped: {}", cache_key));
        Ok(())
    }

}

// pub async fn send_query_to_cached_gemini() -> Result<GeminiResponse, String> {

// } 

