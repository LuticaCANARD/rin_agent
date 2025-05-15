use std::{collections::hash_map, future::Future, pin::Pin};

use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct GeminiActionResult {
    pub result_message: String,
    pub result: serde_json::Value,
    pub error: Option<String>,
}
#[derive(Debug, Clone)]
pub struct GeminiResponse {
    pub discord_msg: String,
    pub sub_items: Vec<String>,
    pub finish_reason: String,
    pub command_result: Vec<Result<GeminiActionResult,String>>,
    pub avg_logprobs: f64,
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
}

#[derive(Debug, Clone)]
pub enum GeminiBotToolInputType {
    STRING,
    NUMBER,
    INTEGER,
    BOOLEAN,
    ARRAY(Vec<GeminiBotToolInputType>),
    OBJECT(Value), // JSON 객체
    NULL,
}
impl GeminiBotToolInputType {
    pub fn to_string(&self) -> String {
        match self {
            GeminiBotToolInputType::STRING=> "STRING".to_string(),
            GeminiBotToolInputType::NUMBER=> "number".to_string(),
            GeminiBotToolInputType::INTEGER => "integer".to_string(),
            GeminiBotToolInputType::BOOLEAN => "BOOLEAN".to_string(),
            GeminiBotToolInputType::ARRAY(arr) => format!("{:?}", arr),
            GeminiBotToolInputType::OBJECT(obj) => obj.to_string(),
            GeminiBotToolInputType::NULL => "null".to_string(),
        }
    }
    pub fn to_schema(&self) -> serde_json::Value {
        match self {
            GeminiBotToolInputType::STRING => json!({"type": "STRING"}),
            GeminiBotToolInputType::NUMBER => json!({"type": "NUMBER"}),
            GeminiBotToolInputType::INTEGER => json!({"type": "INTEGER"}),
            GeminiBotToolInputType::BOOLEAN => json!({"type": "BOOLEAN"}),
            GeminiBotToolInputType::ARRAY(arr) => json!({"type": "ARRAY", "items": arr.to_vec().into_iter().map(|x| x.to_schema()).collect::<Vec<_>>() }),
            GeminiBotToolInputType::OBJECT(obj) => json!({"type": "OBJECT", "properties": obj}),
            GeminiBotToolInputType::NULL => json!({"type": "NULL"}),
        }
    }
}
#[derive(Debug, Clone)]
pub enum GeminiBotToolInputValueType {
    STRING(String),
    NUMBER(f64),
    INTEGER(i64),
    BOOLEAN(bool),
    ARRAY(Vec<GeminiBotToolInputValueType>),
    OBJECT(hash_map::HashMap<String, GeminiBotToolInputValueType>), // JSON 객체
    NULL,
}

impl GeminiBotToolInputValueType {
    pub fn to_string(&self) -> String {
        match self {
            GeminiBotToolInputValueType::STRING(s) => s.clone(),
            GeminiBotToolInputValueType::NUMBER(n) => n.to_string(),
            GeminiBotToolInputValueType::INTEGER(n) => n.to_string(),
            GeminiBotToolInputValueType::BOOLEAN(b) => b.to_string(),
            GeminiBotToolInputValueType::ARRAY(arr) => format!("{:?}", arr),
            GeminiBotToolInputValueType::OBJECT(obj) => obj.iter()
                .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                .collect::<Vec<_>>()
                .join(", ").to_string(),
            GeminiBotToolInputValueType::NULL => "null".to_string(),
        }
    }
}

pub struct GeminiBotToolInput {
    pub name: String,
    pub description: String,
    pub input_type: GeminiBotToolInputType,
    pub required: bool,
}
#[derive(Debug, Clone)]
pub struct GeminiBotToolInputValue {
    pub name: String,
    pub value: GeminiBotToolInputValueType
}

pub struct GeminiBotTools {
    pub name: String,
    pub description: String,
    pub parameters: Vec<GeminiBotToolInput>,    
    pub action: fn(hash_map::HashMap<String, GeminiBotToolInputValue>) 
        -> Pin<Box<dyn Future<Output = Result<GeminiActionResult,String>> + Send>>
    
}
