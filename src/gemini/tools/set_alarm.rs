use gemini_live_api::types::enums::GeminiSchemaType;
use serde_json::{json, Value};

use crate::gemini::types::{generate_input_to_dict, GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue, GeminiBotTools};

use std::collections::{BTreeMap, HashMap};
use std::pin::Pin;
use std::future::Future;
use gemini_live_api::types::enums::GeminiSchemaFormat;

async fn set_alarm(params: HashMap<String, GeminiBotToolInputValue>) -> Result<GeminiActionResult, String> {
    let time = params.get("time");
    if time.is_none() {
        return Err("Missing 'time' parameter".to_string());
    }
    let time = time.unwrap().value.to_string();
    // Here you would implement the logic to set the alarm

    let message = params.get("message");
    let message = if message.is_none() {
        format!("Alarm set for {} with no message", time)
    } else {
        format!("Alarm set for {} with message: {}", time, message.unwrap().value.to_string())
    };

    let repeat = params.get("repeat");
    let repeat = if repeat.is_none() {
        "No repeat".to_string()
    } else {
        format!("Repeat: {}", repeat.unwrap().value.to_string())
    };
    let message = format!("{} - {}", message, repeat);


    Ok(
        GeminiActionResult{
            result_message: message.clone(),
            result: json!({
                "res": message,
            }),
            error: None,
            show_user: Some(true),
        }
    )
}

pub fn get_command() -> GeminiBotTools {
    GeminiBotTools {
        name: "set_alarm".to_string(),
        description: "Set an alarm".to_string(),
        parameters: vec![
            GeminiBotToolInput {
                name: "time".to_string(),
                description: "Set the time for the alarm (Format is YYYY-MM-DD HH:MM:SS)".to_string(),
                input_type: GeminiSchemaType::String,
                required: true,
                format: Some(GeminiSchemaFormat::DateTime),
                //Some("2024-03-21 12:00:00".to_string()),
                default: None,
                enum_values: None,
                example: None,
                pattern: None
                //Some("^[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}$".to_string()),

            },
            GeminiBotToolInput {
                name: "timezone".to_string(),
                description: "Set the timezone for the alarm (UTC+9 => +9, UTC-1 = -1)".to_string(),
                input_type: GeminiSchemaType::Integer,
                required: true,
                format: Some(GeminiSchemaFormat::Int32),
                default: None,
                enum_values: None,
                example: Some(
                    json!("UTC+9 => +9, UTC-1 = -1".to_string())
                ),
                pattern: None,
                //Some("^[+-][0-9]{1,2}$".to_string()),

            },
            GeminiBotToolInput {
                name: "message".to_string(),
                input_type: GeminiSchemaType::String,
                description: "알람과 함꼐 주인님께 보낼 메시지 혹은, 주인님이 알림에 메모한 사항.".to_string(),
                required: false,
                format: None,
                default: None,
                enum_values: None,
                example: None,
                pattern: None,
            },
            GeminiBotToolInput {
                name: "repeat".to_string(),
                description: "반복 주기(cron 표현식)".to_string(),
                input_type: GeminiSchemaType::String,
                required: false,
                format: None,
                default: None,
                enum_values: None,
                example: Some(
                    json!("* * * * * *".to_string())),
                pattern: None,
            },
            GeminiBotToolInput {
                name: "end_date".to_string(),
                description: "종료되는 일자. (Format is YYYY-MM-DD HH:MM:SS)".to_string(),
                input_type: GeminiSchemaType::String,
                required: false,
                format: Some(
                    GeminiSchemaFormat::DateTime
                ),
                default: None,
                enum_values: None,
                example: Some(
                    json!("2024-08-21 12:00:00".to_string())),
                pattern: None
                //Some("^[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}:[0-9]{2}$".to_string()),
            },
        ].into_iter().map(generate_input_to_dict).collect(),
        action: |params| Box::pin(async move { set_alarm(params).await }),
        result_example: Some(serde_json::json!({
            "result_message": "Alarm set for 2024-03-21 12:00:00 with message: Hello!",
            "result": { "res": "Alarm set for 2024-03-21 12:00:00 with message: Hello!" },
            "error": null
        })),
    }
}
