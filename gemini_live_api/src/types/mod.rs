pub mod enums;
pub mod live_api_types;





use std::{collections::BTreeMap, default};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use enums::{DynamicRetrievalConfigMode, GeminiCodeExecutionResultOutcome, GeminiContentRole, GeminiSchemaFormat, GeminiSchemaType};
#[derive(Debug, Clone, PartialEq, Eq,Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThinkingConfig {
    pub include_thoughts: bool,
    pub thinking_budget: i32,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmBlockThreshold{
    HarmBlockThresholdUnspecified,
    BlockLowAndAbove,
    BlockMediumAndAbove,
    BlockOnlyHigh,
    BlockNone,
    Off
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmCategory{
    HarmCategoryUnspecified,//카테고리가 지정되지 않았습니다.
    HarmCategoryDerogatory,//PaLM - ID 또는 보호 속성을 대상으로 하는 부정적이거나 유해한 댓글
    HarmCategoryToxicity,//PaLM - 무례하거나 모욕적이거나 욕설이 있는 콘텐츠
    HarmCategoryViolence,//PaLM - 개인 또는 그룹에 대한 폭력을 묘사하는 시나리오 또는 유혈 콘텐츠에 대한 일반적인 설명을 묘사
    HarmCategorySexual,//PaLM - 성적 행위 또는 기타 외설적인 콘텐츠에 대한 언급을 포함합니다.
    HarmCategoryMedical,//PaLM - 검증되지 않은 의학적 조언을 홍보합니다.
    HarmCategoryDangerous,//PaLM: 유해한 행위를 조장, 촉진 또는 장려하는 위험한 콘텐츠입니다.
    HarmCategoryHarassment,//Gemini - 괴롭힘 콘텐츠
    HarmCategoryHateSpeech,//Gemini - 증오심 표현 및 콘텐츠
    HarmCategorySexuallyExplicit,//Gemini - 성적으로 노골적인 콘텐츠
    HarmCategoryDangerousContent,//Gemini - 위험한 콘텐츠
    HarmCategoryCivicIntegrity,//Gemini - 시민의 품위를 해치는 데 사용될 수 있는 콘텐츠
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetySetting{
    pub category: HarmCategory,
    pub threshold: HarmBlockThreshold
}

#[derive(Debug, Clone,Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGenerationConfig {
//  https://ai.google.dev/api/generate-content?hl=ko#generationconfig
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<GeminiSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_modalities:Option<Vec<GeminiResponseModalities>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub candidate_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_enhanced_civic_answers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech_config: Option<GeminiSpeechConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_config: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_resolution: Option<GeminiMediaResolution>,
}

impl Default for GeminiGenerationConfig {
    fn default ()-> Self {
        GeminiGenerationConfig {
            candidate_count: Some(1),
            max_output_tokens: Some(100),
            temperature: Some(0.5),
            top_p: Some(0.9),
            top_k: Some(40),
            presence_penalty: Some(0.0),
            frequency_penalty: Some(0.0),
            response_modalities: None,
            stop_sequences: None,
            response_mime_type: None,
            response_schema: None,
            response_logprobs: None,
            logprobs: None,
            enable_enhanced_civic_answers: None,
            speech_config: None,
            thinking_config: None,
            media_resolution: None,
            seed: None,
        }
    }
}
#[derive(Debug, Clone,Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiSpeechConfig{
    pub voice_config: Option<GeminiVoiceConfig>,
    pub language_code:Option<String>,
}

#[derive(Debug, Clone,Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiVoiceConfig{
    pub prebuilt_voice_config: Option<GeminiPrebuiltVoiceConfig>,
}

#[derive(Debug, Clone,Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiPrebuiltVoiceConfig{
    pub voice_name:String
}

#[derive(Debug, Clone,Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeminiMediaResolution{
    MediaResolutionUnspecified,
    MediaResolutionLow,
    MediaResolutionMedium,
    MediaResolutionHigh,
}
#[derive(Debug, Clone,Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeminiResponseModalities {
    #[serde(rename = "RESPONSE_MODALITY_UNSPECIFIED")]
    ResponseModalityUnspecified,
    #[serde(rename = "TEXT")]
    Text,
    #[serde(rename = "IMAGE")]
    Image,
    #[serde(rename = "AUDIO")]
    Audio,
    #[serde(rename = "VIDEO")]
    Video,
}



#[derive(Debug, Clone,Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiSchema {
    #[serde(rename="type")]
    pub schema_type: GeminiSchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<GeminiSchemaFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
    #[serde(rename="enum",skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<BTreeMap<String, GeminiSchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<GeminiSchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_ordering: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<GeminiSchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f32>,
}
impl Default for GeminiSchema {
    fn default() -> Self {
        GeminiSchema {
            schema_type: GeminiSchemaType::String,
            format: None,
            title: None,
            description: None,
            nullable: None,
            enum_values: None,
            max_items: None,
            min_items: None,
            properties: None,
            required: None,
            min_properties: None,
            max_properties: None,
            pattern: None,
            example: None,
            any_of: None,
            property_ordering: None,
            default: None,
            items: None,
            minimum: None,
            maximum: None
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiContents{
    pub parts:Vec<GeminiParts>,
    pub role: GeminiContentRole
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiParts{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<bool>,
    //--------------------------
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inline_data: Option<GeminiInlineBlob>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<GeminiFunctionCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_response: Option<GeminiFunctionResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data:Option<GeminiFileData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executable_code:Option<GeminiExecutableCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_execution_result:Option<GeminiExecutableCodeResult>,

}
impl GeminiParts {
    pub fn set_text(&mut self, text: String) {
        self.text = Some(text);
        self.inline_data = None;
        self.function_call = None;
        self.function_response = None;
        self.file_data = None;
        self.executable_code = None;
        self.code_execution_result = None;
    }
    pub fn set_inline_data(&mut self, item:GeminiInlineBlob) {
        self.text = None;
        self.inline_data = Some(item);
        self.function_call = None;
        self.function_response = None;
        self.file_data = None;
        self.executable_code = None;
        self.code_execution_result = None;
    }
    pub fn set_function_call(&mut self, item:GeminiFunctionCall) {
        self.text = None;
        self.inline_data = None;
        self.function_call = Some(item);
        self.function_response = None;
        self.file_data = None;
        self.executable_code = None;
        self.code_execution_result = None;
    }
    pub fn set_function_response(&mut self, item:GeminiFunctionResponse) {
        self.text = None;
        self.inline_data = None;
        self.function_call = None;
        self.function_response = Some(item);
        self.file_data = None;
        self.executable_code = None;
        self.code_execution_result = None;
    }
    pub fn set_file_data(&mut self, item:GeminiFileData) {
        self.text = None;
        self.inline_data = None;
        self.function_call = None;
        self.function_response = None;
        self.file_data = Some(item);
        self.executable_code = None;
        self.code_execution_result = None;
    }
    pub fn set_executable_code(&mut self, item:GeminiExecutableCode) {
        self.text = None;
        self.inline_data = None;
        self.function_call = None;
        self.function_response = None;
        self.file_data = None;
        self.executable_code = Some(item);
        self.code_execution_result = None;
    }
    pub fn set_code_execution_result(&mut self, item:GeminiExecutableCodeResult) {
        self.text = None;
        self.inline_data = None;
        self.function_call = None;
        self.function_response = None;
        self.file_data = None;
        self.executable_code = None;
        self.code_execution_result = Some(item);
    }
    
}
impl Default for GeminiParts {
    fn default() -> Self {
        GeminiParts {
            thought: None,
            text: None,
            inline_data: None,
            function_call: None,
            function_response: None,
            file_data: None,
            executable_code: None,
            code_execution_result: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiInlineBlob{
    pub mime_type: String,
    pub data: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionCall{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<BTreeMap<String,Value>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionResponse{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub args: BTreeMap<String,Value>
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFileData{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub file_uri: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiExecutableCode{
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<BTreeMap<String,Value>>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiExecutableCodeResult{
    pub outcome: GeminiCodeExecutionResultOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGenerationConfigTool{
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_declarations: Option<Vec<GeminiFunctionDeclaration>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search_retrieval: Option<GeminiGoogleSearchRetrieval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_execution: Option<GeminiCodeExecutionTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub google_search: Option<GeminiGoogleSearchTool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url_context: Option<UrlContext>, // 구글 검색 도구
} 
impl Default for GeminiGenerationConfigTool {
    fn default() -> Self {
        GeminiGenerationConfigTool {
            function_declarations: None,
            google_search_retrieval: None,
            code_execution: None,
            google_search: None,
            url_context: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UrlContext{}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGoogleSearchTool{}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiCodeExecutionTool{}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGoogleSearchRetrieval{
    pub dynamic_retrieval_config:GeminiGoogleSearchRetrievalOption
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGoogleSearchRetrievalOption{
    pub mode:DynamicRetrievalConfigMode,
    pub dynamic_threshold: f32,
}

fn get_object_type() -> GeminiSchemaType {
    GeminiSchemaType::Object
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiSchemaObject{
    #[serde(rename="type",default="get_object_type")]
    pub schema_type: GeminiSchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable: Option<bool>,
    pub properties: BTreeMap<String, GeminiSchema>,
    pub required: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<GeminiSchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_ordering: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Box<GeminiSchemaObject>>,
}
impl Default for GeminiSchemaObject {
    fn default() -> Self {
        GeminiSchemaObject {
            schema_type: GeminiSchemaType::Object,
            title: None,
            description: None,
            nullable: None,
            properties: BTreeMap::new(),
            required: Vec::new(),
            min_properties: None,
            max_properties: None,
            example: None,
            any_of: None,
            property_ordering: None,
            default: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionDeclaration{
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<GeminiSchemaObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<GeminiSchema>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingMetadata{
    pub grounding_chunks: Vec<GroundingChunk>,
    pub grounding_supports: Vec<GroundingSupport>,
    pub web_search_queries: String,
    pub search_entry_point: Option<SearchEntryPoint>,
    pub retrieval_metadata: Option<RetrievalMetadata>,

}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetrievalMetadata{
    pub google_search_dynamic_retrieval_score: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchEntryPoint{
    pub rendered_content:Option<String>,
    pub sdk_blob:Option<String>,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingSupport{
    pub grounding_chunk_indices: Vec<i32>,
    pub confidence_scores: Vec<f32>,
    pub segments: Vec<GroundingSegment>,

}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingSegment{
    pub part_index: i32,
    pub start_index: i32,
    pub end_index: i32,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundingChunk{
    //chunk_type Union type...
    pub web:Option<GeminiWebResult>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiWebResult{
    pub uri:String,
    pub title:String,
}

#[derive(Debug, Clone, Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiCachedContent{
    pub contents: Vec<GeminiContents>,
    pub tools: Vec<GeminiGenerationConfigTool>,
    pub ttl: String, // Time to live in seconds
    pub model: String, // Model name, e.g., "gemini-1.5-flash"
    pub display_name: Option<String>, // Optional display name for the cached content
    pub system_instruction: Option<GeminiContents>, // System instructions for the model
    pub tool_config: Option<GeminiToolConfig>, // Tool configuration for the model
}
///["Debug"] Gemini API > Resp: {
//   "name": "cachedContents/bzjcqjn5zf9b8ev2a2ig76ewucihbabkqawxbigw",
//   "model": "models/gemini-2.5-flash-preview-04-17",
//   "createTime": "2025-05-29T11:33:29.292302Z",
//   "updateTime": "2025-05-29T11:33:29.292302Z",
//   "expireTime": "2025-05-29T11:33:40.809421037Z",
//   "displayName": "",
//   "usageMetadata": {
//     "totalTokenCount": 1170
//   }
// }
#[derive(Debug, Clone, Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiCachedContentResponse {
    pub name :String,
    pub model: String, // Model name, e.g., "gemini-1.5-flash"
    pub create_time: String, // Creation time in ISO 8601 format
    pub update_time: String, // Last update time in ISO 8601 format
    pub expire_time: String, // Expiration time in ISO 8601 format
    pub display_name: Option<String>, // Optional display name for the cached content
    pub usage_metadata: Option<GeminiUsageMetadata>, // Usage metadata for the cached content
}

#[derive(Debug, Clone, Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiUsageMetadata {
    pub total_token_count: i32, // Total token count used in the cached content
}


#[derive(Debug, Clone, Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiToolConfig {
    pub function_calling_config: Option<GeminiFunctionCallingConfig>,
}
#[derive(Debug, Clone, Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiFunctionCallingConfig{
    pub mode: Option<GeminiToolConfigMode>,
    pub allowed_function_names: Option<Vec<String>>,
}
#[derive(Debug, Clone, Serialize,Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeminiToolConfigMode {
    #[serde(rename = "AUTO")]
    Auto,
    #[serde(rename = "ANY")]
    Any,
    #[serde(rename = "VALIDATED")]
    Validated,
    #[serde(rename = "NONE")]
    None,
}