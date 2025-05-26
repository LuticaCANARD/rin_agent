use gemini_live_api::types::enums::GeminiSchemaType;
use sea_orm::sea_query::time_format;
use serde_json::{json, Value};

use crate::api::instances::RIN_SERVICES;
use crate::api::schedule::{ScheduleRepeatRequest, ScheduleRequest, ScheduleService};
use crate::gemini::types::{generate_input_to_dict, GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue, GeminiBotToolInputValueType, GeminiBotTools};

use std::collections::{BTreeMap, HashMap};
use std::pin::Pin;
use std::future::Future;
use sqlx::types::time;
use gemini_live_api::types::enums::GeminiSchemaFormat;
use time_macros::format_description; 

async fn set_alarm(params: HashMap<String, GeminiBotToolInputValue>) -> Result<GeminiActionResult, String> {
    let time = params.get("time");
    if time.is_none() {
        return Err("Missing 'time' parameter".to_string());
    }
    let time = time.unwrap().value.to_string();
    // Here you would implement the logic to set the alarm

    let message = params.get("message");
    let set_message = if message.is_none() {
        format!("Alarm set for {} with no message", time)
    } else {
        format!("Alarm set for {} with message: {}", time, message.unwrap().value.to_string())
    };

    let repeat = params.get("repeat");
    let repeat_set = if repeat.is_none() {
        "No repeat".to_string()
    } else {
        format!("Repeat: {}", repeat.unwrap().value.to_string())
    };
    let set_message = format!("{} - {}", set_message, repeat_set);

    let date_format = time_format::FORMAT_DATETIME_TZ;
    
    let start = time::PrimitiveDateTime::parse(
        &time, 
        date_format)
        .map_err(|_| "Invalid time format".to_string())?;
    let end_str = params.get("end_date")
        .map_or_else(|| "".to_string(), |v| v.value.to_string());
    let end = time::PrimitiveDateTime::parse(
        &end_str,
        date_format
    );
    let timezone = params.get("timezone")
        .map_or("UTC".to_string(), |v| v.value.to_string());

    let description = if let Some(desc) = message {
        Some(desc.value.to_string())
    } else {
        None
    };

    let name = params.get("name")
        .map_or("Alarm".to_string(), |v| v.value.to_string());

    let repeat = repeat.map(|repeat_type_val| {
        let repeat_interval = params.get("repeatcount")
            .and_then(|v| match &v.value {
                GeminiBotToolInputValueType::Integer(i) => Some(*i as i32),
                GeminiBotToolInputValueType::String(s) => s.parse::<i32>().ok(),
                _ => None,
            })
            .unwrap_or(0);
        let repeat_type = repeat_type_val.value.to_string();
        let repeat_end = end.ok();
        ScheduleRepeatRequest {
            repeat_type,
            repeat_interval,
            repeat_end,
        }
    });

    let end = match end {
        Ok(e) => e,
        Err(_) => start, // Default to max if no end date is provided
    };
    let alarm_item = ScheduleRequest{start, end, timezone, name, description, repeat};
    RIN_SERVICES.call::<ScheduleService>()
        .unwrap()
        .lock()
        .await
        .add_schedule(alarm_item)
        .await;

    Ok(
        GeminiActionResult{
            result_message: set_message.clone(),
            result: json!({
                "res": set_message,
            }),
            error: None,
            show_user: Some(set_message),
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
                name: "name".to_string(),
                input_type: GeminiSchemaType::String,
                description: "알람의 이름.".to_string(),
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
                name: "repeatcount".to_string(),
                description: "반복 횟수(0은 end_date까지 무제한)".to_string(),
                input_type: GeminiSchemaType::Integer,
                required: false,
                format: Some(GeminiSchemaFormat::Int32),
                default: None,
                enum_values: None,
                example: Some(json!("0".to_string())),
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
                example: Some(json!("2024-08-21 12:00:00".to_string())),
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
