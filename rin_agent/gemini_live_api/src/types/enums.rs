use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GeminiSchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
    Null
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GeminiContentRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "model")]
    Model
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GeminiCodeExecutionResultOutcome {
    OutcomeUnspecified,
    OutcomeOk,
    OutcomeError,
    OutcomeDeadlineExceeded,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DynamicRetrievalConfigMode{
    ModeUnspecified,
    ModeDynamic
}

//
// #[derive(Debug, Clone)]
// pub enum GeminiBotToolInputType {
//     STRING,
//     NUMBER,
//     INTEGER,
//     BOOLEAN,
//     ARRAY(Vec<GeminiBotToolInputType>),
//     OBJECT(Value), // JSON 객체
//     NULL,
// }
// impl GeminiBotToolInputType {
//     pub fn to_string(&self) -> String {
//         match self {
//             GeminiBotToolInputType::STRING=> "STRING".to_string(),
//             GeminiBotToolInputType::NUMBER=> "number".to_string(),
//             GeminiBotToolInputType::INTEGER => "integer".to_string(),
//             GeminiBotToolInputType::BOOLEAN => "BOOLEAN".to_string(),
//             GeminiBotToolInputType::ARRAY(arr) => format!("{:?}", arr),
//             GeminiBotToolInputType::OBJECT(obj) => obj.to_string(),
//             GeminiBotToolInputType::NULL => "null".to_string(),
//         }
//     }
//     pub fn to_schema(&self) -> serde_json::Value {
//         match self {
//             GeminiBotToolInputType::STRING => json!({"type": "STRING"}),
//             GeminiBotToolInputType::NUMBER => json!({"type": "NUMBER"}),
//             GeminiBotToolInputType::INTEGER => json!({"type": "INTEGER"}),
//             GeminiBotToolInputType::BOOLEAN => json!({"type": "BOOLEAN"}),
//             GeminiBotToolInputType::ARRAY(arr) => json!({"type": "ARRAY", "items": arr.to_vec().into_iter().map(|x| x.to_schema()).collect::<Vec<_>>() }),
//             GeminiBotToolInputType::OBJECT(obj) => json!({"type": "OBJECT", "properties": obj}),
//             GeminiBotToolInputType::NULL => json!({"type": "NULL"}),
//         }
//     }
// }