use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::types::GeminiGenerationConfig;

//https://ai.google.dev/api/live?hl=ko#receive-messages
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeminiLiveApiSession {
    pub model: String,
    pub generation_config: GeminiLiveApiGenerationConfig,
    pub system_instruction: String,
    pub tools: Vec<GeminiLiveApiTool>
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeminiLiveApiTool {
    pub name: String,
    pub description: String,
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeminiLiveApiGenerationConfig {
    pub candidate_count: Option<i32>,
    pub max_output_tokens: Option<i32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
    pub presence_penalty: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub response_modalities: Option<Vec<GeminiResponseModalities>>,
}


#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum GeminiResponseModalities {
    TEXT,
    IMAGE,
    AUDIO,
    ModalityUnspecified
}

pub enum GeminiLiveApiWebSocketMessage {
    Setup(BidiGenerateContentSetup),
    ClientContent,
    RealtimeInput,
    ToolResponse
}

pub struct BidiGenerateContentSetup {
    pub model: String,
    pub generation_config:GeminiGenerationConfig
}