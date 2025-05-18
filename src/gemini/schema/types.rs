use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, PartialEq, Eq,Deserialize, Serialize)]
pub struct ThinkingConfig {
    pub include_thoughts: bool,
    pub thinking_budget: i32,
}
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum HarmBlockThreshold{
    HarmBlockThresholdUnspecified,
    BlockLowAndAbove,
    BlockMediumAndAbove,
    BlockOnlyHigh,
    BlockNone,
    Off
}
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SafetySetting{
    category: HarmCategory,
    threshold: HarmBlockThreshold
}

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone,Deserialize, Serialize)]
pub struct GeminiGenerationConfig {
//  https://ai.google.dev/api/generate-content?hl=ko#generationconfig
    pub stop_sequences: Option<Vec<String>>,
    pub response_mime_type: Option<String>,
    pub response_schema: Option<GeminiSchema>,
    
}
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone,Deserialize, Serialize)]
pub struct GeminiSchema {
    #[serde(rename="type")]
    pub schema_type: GeminiSchemaType,
    pub format: Option<GeminiSchemaFormat>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub nullable: Option<bool>,
    #[serde(rename="enum")]
    pub enum_values: Option<Vec<String>>,
    pub max_items: Option<String>,
    pub min_items: Option<String>,
    pub properties: Option<BTreeMap<String, GeminiSchema>>,
    pub required: Option<Vec<String>>,
    pub min_properties: Option<String>,
    pub max_properties: Option<String>,
    pub pattern: Option<String>,
    pub example: Option<serde_json::Value>,
    pub any_of: Option<Vec<GeminiSchema>>,
    pub property_ordering: Option<Vec<String>>,
    pub default: Option<serde_json::Value>,
    pub items: Option<Box<GeminiSchema>>,
    pub minimum: Option<f32>,
    pub maximum: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeminiSchemaType {
    STRING,
    NUMBER,
    INTEGER,
    BOOLEAN,
    ARRAY,
    OBJECT,
    NULL
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeminiSchemaFormat {
    Float,
    Double,
    Int32,
    Int64,
    #[serde(rename = "enum")]
    EnumString,
    #[serde(rename = "date-time")]
    DateTime,
}