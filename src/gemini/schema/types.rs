use serde::{Deserialize, Serialize};
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, PartialEq, Eq,Deserialize, Serialize)]
pub struct ThinkingConfig {
    includeThoughts: bool,
    thinkingBudget: i32,
}
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmBlockThreshold{
    HARM_BLOCK_THRESHOLD_UNSPECIFIED,
    BLOCK_LOW_AND_ABOVE,
    BLOCK_MEDIUM_AND_ABOVE,
    BLOCK_ONLY_HIGH,
    BLOCK_NONE,
    OFF
}
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarmCategory{
    HARM_CATEGORY_UNSPECIFIED,//카테고리가 지정되지 않았습니다.
    HARM_CATEGORY_DEROGATORY,//PaLM - ID 또는 보호 속성을 대상으로 하는 부정적이거나 유해한 댓글
    HARM_CATEGORY_TOXICITY,//PaLM - 무례하거나 모욕적이거나 욕설이 있는 콘텐츠
    HARM_CATEGORY_VIOLENCE,//PaLM - 개인 또는 그룹에 대한 폭력을 묘사하는 시나리오 또는 유혈 콘텐츠에 대한 일반적인 설명을 묘사
    HARM_CATEGORY_SEXUAL,//PaLM - 성적 행위 또는 기타 외설적인 콘텐츠에 대한 언급을 포함합니다.
    HARM_CATEGORY_MEDICAL,//PaLM - 검증되지 않은 의학적 조언을 홍보합니다.
    HARM_CATEGORY_DANGEROUS,//PaLM: 유해한 행위를 조장, 촉진 또는 장려하는 위험한 콘텐츠입니다.
    HARM_CATEGORY_HARASSMENT,//Gemini - 괴롭힘 콘텐츠
    HARM_CATEGORY_HATE_SPEECH,//Gemini - 증오심 표현 및 콘텐츠
    HARM_CATEGORY_SEXUALLY_EXPLICIT,//Gemini - 성적으로 노골적인 콘텐츠
    HARM_CATEGORY_DANGEROUS_CONTENT,//Gemini - 위험한 콘텐츠
    HARM_CATEGORY_CIVIC_INTEGRITY,//Gemini - 시민의 품위를 해치는 데 사용될 수 있는 콘텐츠
}
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, PartialEq, Eq,Deserialize, Serialize)]
pub struct SafetySetting{
    category: HarmCategory,
    threshold: HarmBlockThreshold
}
