use std::{collections::hash_map, future::Future, pin::Pin};

use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub struct GeminiResponse {
    pub discord_msg: String,
    pub sub_items: Vec<String>,
    pub finish_reason: String,
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
    STRING(String),
    NUMBER(f32),
    INTEGER(i32),
    BOOLEAN(bool),
    ARRAY(Vec<GeminiBotToolInputType>),
    OBJECT(serde_json::Value), // JSON 객체
    NULL,
}
impl GeminiBotToolInputType {
    pub fn to_string(&self) -> String {
        match self {
            GeminiBotToolInputType::STRING(s) => s.clone(),
            GeminiBotToolInputType::NUMBER(n) => n.to_string(),
            GeminiBotToolInputType::INTEGER(n) => n.to_string(),
            GeminiBotToolInputType::BOOLEAN(b) => b.to_string(),
            GeminiBotToolInputType::ARRAY(arr) => format!("{:?}", arr),
            GeminiBotToolInputType::OBJECT(obj) => obj.to_string(),
            GeminiBotToolInputType::NULL => "null".to_string(),
        }
    }
    pub fn to_schema(&self) -> serde_json::Value {
        match self {
            GeminiBotToolInputType::STRING(_) => json!({"type": "STRING"}),
            GeminiBotToolInputType::NUMBER(_) => json!({"type": "NUMBER"}),
            GeminiBotToolInputType::INTEGER(_) => json!({"type": "INTEGER"}),
            GeminiBotToolInputType::BOOLEAN(_) => json!({"type": "BOOLEAN"}),
            GeminiBotToolInputType::ARRAY(arr) => json!({"type": "ARRAY", "items": arr.to_vec().into_iter().map(|x| x.to_schema()).collect::<Vec<_>>() }),
            GeminiBotToolInputType::OBJECT(obj) => json!({"type": "OBJECT", "properties": obj}),
            GeminiBotToolInputType::NULL => json!({"type": "NULL"}),
        }
    }
}
pub struct GeminiBotToolInput {
    pub name: String,
    pub description: String,
    pub input_type: GeminiBotToolInputType,
    pub required: bool,
}
pub struct GeminiBotToolInputValue {
    pub name: String,
    pub value: GeminiBotToolInputType
}

pub struct GeminiBotTools {
    pub name: String,
    pub description: String,
    pub parameters: Vec<GeminiBotToolInput>,    
    pub action: fn(hash_map::HashMap<String, GeminiBotToolInputValue>) 
        -> Pin<Box<dyn Future<Output = Result<Value,String>> + Send>>
    
}