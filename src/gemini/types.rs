use std::{collections::{hash_map, BTreeMap}, default, future::Future, pin::Pin};

use gemini_live_api::types::{enums::{GeminiSchemaFormat, GeminiSchemaType}, GeminiSchema, GeminiSchemaObject};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use serenity::all::{ChannelId, UserId};

#[derive(Debug, Clone,Default,Deserialize,Serialize)]
pub struct GeminiActionResult {
    pub result_message: String,
    pub result: serde_json::Value,
    pub error: Option<String>,
    pub show_user: Option<String>
}
#[derive(Debug, Clone)]
pub struct GeminiResponse {
    pub discord_msg: String,
    pub sub_items: Option<Vec<String>>,
    pub finish_reason: String,
    pub command_result: Vec<Result<GeminiActionResult,String>>,
    pub avg_logprobs: f64,
    pub thoughts: Option<String>,
}
#[derive(Debug, Clone)]
pub struct GeminiImageInputType {
    pub base64_image: Option<String>,
    pub file_url: Option<String>,
    // e.g. "image/png", "image/jpeg"
    pub mime_type: String,
}
#[derive(Debug, Clone)]
pub struct GeminiChatChunk {
    pub query: String,
    pub image: Option<GeminiImageInputType>,
    pub is_bot: bool,
    pub timestamp: String,
    pub user_id: Option<String>, 
    pub guild_id: Option<u64>,
    pub channel_id: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum GeminiBotToolInputValueType {
    String(String),
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Array(Vec<GeminiBotToolInputValueType>),
    Object(hash_map::HashMap<String, GeminiBotToolInputValueType>), // JSON 객체
    Null,
}

impl GeminiBotToolInputValueType {
    pub fn to_string(&self) -> String {
        match self {
            GeminiBotToolInputValueType::String(s) => s.clone(),
            GeminiBotToolInputValueType::Number(n) => n.to_string(),
            GeminiBotToolInputValueType::Integer(n) => n.to_string(),
            GeminiBotToolInputValueType::Boolean(b) => b.to_string(),
            GeminiBotToolInputValueType::Array(arr) => format!("{:?}", arr),
            GeminiBotToolInputValueType::Object(obj) => obj.iter()
                .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                .collect::<Vec<_>>()
                .join(", ").to_string(),
            GeminiBotToolInputValueType::Null => "null".to_string(),
        }
    }
}

pub struct GeminiBotToolInput {
    pub name: String,
    pub description: String,
    pub input_type: GeminiSchemaType,
    pub required: bool,
    pub format: Option<GeminiSchemaFormat>,
    pub pattern: Option<String>,
    pub default: Option<Value>,
    pub enum_values: Option<Vec<String>>,
    pub example: Option<Value>,
}

pub fn generate_to_schema(input: &GeminiBotToolInput) -> GeminiSchema {
    GeminiSchema{
        schema_type: input.input_type.clone(),
        title: Some(input.name.clone()),
        description: Some(input.description.clone()),
        format: input.format.clone(),
        pattern: input.pattern.clone(),
        enum_values: input.enum_values.clone(),
        default: input.default.clone(),
        example: input.example.clone(),
        ..Default::default()
    }
}

#[derive(Debug, Clone)]
pub struct GeminiBotToolInputValue {
    pub name: String,
    pub value: GeminiBotToolInputValueType
}
pub type GeminiAPIObjectStruct = BTreeMap<String, GeminiBotToolInput>;

pub struct GeminiBotTools {
    pub name: String,
    pub description: String,
    pub parameters: GeminiAPIObjectStruct,    
    pub action: fn(hash_map::HashMap<String, GeminiBotToolInputValue>, Option<DiscordUserInfo>) 
        -> Pin<Box<dyn Future<Output = Result<GeminiActionResult,String>> + Send>>,
    pub response :  Option<GeminiSchema>
}
impl Default for GeminiBotTools{
    fn default() -> Self {
        GeminiBotTools {
            name: "default".to_string(),
            description: "default".to_string(),
            parameters: GeminiAPIObjectStruct::new(),
            action: |params,info| Box::pin(async move { Ok(GeminiActionResult{
                result_message: "default".to_string(),
                result: json!({}),
                error: None,
                show_user: None
            }) }),
            response: None    
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiscordUserInfo {
    pub user_id: UserId,
    pub username: Option<String>,
    pub channel_id: ChannelId,
    pub context_id: Option<i64>, // AI context ID
}

pub fn generate_input_to_dict(input: GeminiBotToolInput) -> (String, GeminiBotToolInput) {
    (input.name.clone(), input)
}