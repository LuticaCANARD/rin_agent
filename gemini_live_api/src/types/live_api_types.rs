use std::time::Duration;

use serde::{Deserialize, Serialize};
use super::{GeminiContents, GeminiFunctionCall, GeminiFunctionResponse, GeminiGenerationConfig, GeminiGenerationConfigTool, GeminiInlineBlob, GroundingMetadata};

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
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiLiveApiWebSocketMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup: Option<BidiGenerateContentSetup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_content: Option<BidiGenerateContentClientContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub real_time_input: Option<BidiGenerateContentRealTimeInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_response: Option<BidiGenerateContentToolResponse>,
}

impl GeminiLiveApiWebSocketMessage {
    // 첫 번째 메시지로 전송할 세션 구성
    pub fn set_setup(&mut self, setup: BidiGenerateContentSetup) {
        self.setup = Some(setup);
        self.client_content = None;
        self.real_time_input = None;
        self.tool_response = None;
    }
    // 클라이언트에서 전송된 현재 대화의 증분 콘텐츠 업데이트
    pub fn set_client_content(&mut self, client_content: BidiGenerateContentClientContent) {
        self.client_content = Some(client_content);
        self.setup = None;
        self.real_time_input = None;
        self.tool_response = None;
    }
    // 실시간 오디오, 동영상 또는 텍스트 입력
    pub fn set_real_time_input(&mut self, real_time_input: BidiGenerateContentRealTimeInput) {
        self.real_time_input = Some(real_time_input);
        self.setup = None;
        self.client_content = None;
        self.tool_response = None;
    }
    // 서버에서 수신한 ToolCallMessage에 대한 응답
    pub fn set_tool_response(&mut self, tool_response: BidiGenerateContentToolResponse) {
        self.tool_response = Some(tool_response);
        self.setup = None;
        self.client_content = None;
        self.real_time_input = None;
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiGenerateContentRealTimeInput{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_chunks: Option<Vec<GeminiInlineBlob>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio : Option<GeminiInlineBlob>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<GeminiInlineBlob>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_start: Option<ActivityStart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_end: Option<ActivityEnd>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_stream_end: Option<bool>,
    pub text:String,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivityStart;
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivityEnd;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiGenerateContentSetup {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config:Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_instruction:Option<GeminiContents>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<GeminiGenerationConfigTool>>,
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
impl Default for BidiGenerateContentSetup {
    fn default() -> Self {
        BidiGenerateContentSetup {
            model: "gemini-2.0-flash-live-001".to_string(),
            generation_config: None,
            system_instruction: None,
            tools: None,
            realtime_input_config: None,
            session_resumption: None,
            context_window_compression: None,
            input_audio_transcription: None,
            output_audio_generation: None
        }
    }
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiGenerateContentToolResponse{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_responses:Option<Vec<GeminiFunctionResponse>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiGenerateContentTranscription{
    pub text:String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiGenerateContentClientContent{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turns: Option<Vec<GeminiContents>>,
    pub turn_complete: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiGenerateContentServerContent{
    pub generation_complete: bool,
    pub turn_complete: bool,
    pub interrupted: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounding_metadata: Option<GroundingMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_transcription: Option<BidiGenerateContentTranscription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_transcription: Option<BidiGenerateContentTranscription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_turn: Option<GeminiContents>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoAway{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_left:Option<Duration>,
}



#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResumptionUpdate{
    pub new_handle: Option<String>,
    pub resumable: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiGenerateContentServerMessage {
    pub setup_complete: Option<BidiGenerateContentSetupComplete>,
    pub server_content: Option<BidiGenerateContentServerContent>,
    pub go_away: Option<GoAway>,
    pub tool_call: Option<BidiGenerateContentToolCall>,
    pub tool_call_cancellation: Option<BidiGenerateContentToolCallCancellation>,
    pub session_resumption_update: Option<SessionResumptionUpdate>,
}