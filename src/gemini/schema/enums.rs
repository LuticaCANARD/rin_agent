use serde::{Deserialize, Serialize};

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