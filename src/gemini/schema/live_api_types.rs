use serde::{Deserialize, Serialize};
use super::types::{GeminiContents, GeminiFunctionCall, GeminiGenerationConfig};

//https://ai.google.dev/api/live?hl=ko#receive-messages
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiLiveApiSession {
    pub model: String,
    pub generation_config: GeminiLiveApiGenerationConfig,
    pub system_instruction: String,
    pub tools: Vec<GeminiLiveApiTool>
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiLiveApiTool {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RealtimeInputConfig {
    pub automatic_activity_detection:Option<AutomaticActivityDetection>,
    pub activity_handling:Option<ActivityHandling>,
    pub turn_coverage:Option<TurnCoverage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityHandling {
    ActivityHandlingUnspecified,
    StartOfActivityInterrupts,
    NoInterruption
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TurnCoverage {
    TurnCoverageUnspecified,
    TurnIncludesOnlyActivity,
    TurnIncludesAllInput
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutomaticActivityDetection{
    pub disable: Option<bool>,
    pub start_of_speech_sensitivity: Option<StartSensitivity>,
    pub prefix_padding_ms: Option<i32>,
    pub end_of_speech_sensitivity:Option<EndSensitivity>,
    pub silence_duration_ms: Option<i32>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EndSensitivity{
    EndSensitivityUnspecified,
    EndSensitivityHigh,
    EndSensitivityLow
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StartSensitivity {
    StartSensitivityUnspecified,
    StartSensitivityHigh,
    StartSensitivityLow
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeminiResponseModalities {
    Text,
    Image,
    Audio,
    ModalityUnspecified
}

pub enum GeminiLiveApiWebSocketMessage {
    Setup(BidiGenerateContentSetup),
    ClientContent,
    RealtimeInput,
    ToolResponse
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiGenerateContentSetup {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config:Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction:Option<GeminiContents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<GeminiLiveApiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realtime_input_config: Option<RealtimeInputConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_resumption: Option<SessionResumptionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window_compression: Option<ContextWindowCompression>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: Option<AudioTranscriptionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_audio_generation: Option<AudioTranscriptionConfig>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioTranscriptionConfig;
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResumptionConfig{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextWindowCompression{
    pub sliding_window: SlidingWindow,
    pub trigger_tokens: i32,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlidingWindow{
    pub target_tokens: i32
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiGenerateContentSetupComplete;
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiGenerateContentToolCall{
    pub function_calls: Vec<GeminiFunctionCall>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiGenerateContentToolCallCancellation{
    pub ids: Vec<String>
}
