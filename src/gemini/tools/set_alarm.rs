use chrono::DateTime;
use gemini_live_api::libs::logger::LOGGER;
use gemini_live_api::types::enums::GeminiSchemaType;
use gemini_live_api::types::GeminiSchema;
use sea_orm::sea_query::time_format;
use serde_json::{json, Value};

use crate::api::instances::get_rin_services;
use crate::api::schedule::{ScheduleRepeatRequest, ScheduleRequest, ScheduleService};
use crate::gemini::types::{generate_input_to_dict, GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue, GeminiBotToolInputValueType, GeminiBotTools};

use std::collections::{BTreeMap, HashMap};
use sqlx::types::time;
use gemini_live_api::types::enums::GeminiSchemaFormat;

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
    let time_spl = time.split_at(time.len() - 6);
    let mut time_spl_iter = time_spl.0.split('.');
    let first_part: &str = time_spl_iter.next().unwrap_or("");
    let second_part = time_spl_iter.next().unwrap_or("");
    let mut second_time_iter = second_part.split(' ');
    let second_time_first = second_time_iter.next().unwrap_or("");
    let second_time_pp = format!("{:6}", second_time_first);
    // let time_zone = 

    let time = format!("{}.{} {}", first_part, second_time_pp, time_spl.1);
    let date_format = time_format::FORMAT_DATETIME_TZ;
    
    LOGGER.log(gemini_live_api::libs::logger::LogLevel::Debug, &format!("Setting alarm with params: {:?}", time));
    let start = DateTime::parse_from_str(
        &time, 
        "%Y-%m-%d %H:%M:%S%.f %z")
        .map_err(|_| "Invalid Start datetime format".to_string())?;


    let end_str = params.get("end_date");
    
    let end = if end_str.is_none() {
        Ok(start)
    } else {
        let end_str = end_str.unwrap().value.to_string();
        DateTime::parse_from_str(
            &end_str,
            "%Y-%m-%d %H:%M:%S%.f %z"
        )
    };

    LOGGER.log(gemini_live_api::libs::logger::LogLevel::Debug, &format!("Parsed start time: {:?}", start));

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
    get_rin_services()
        .await
        .call::<ScheduleService>()
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
                description: format!("Set the time for the alarm (Format is {})", "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6] [offset_hour sign:mandatory]:[offset_minute]").to_string(),
                input_type: GeminiSchemaType::String,
                required: true,
                format: Some(GeminiSchemaFormat::DateTime),
                default: None,
                enum_values: None,
                example: Some(sea_orm::JsonValue::String("2024-03-21 12:00:00.000000+09:00".to_string())),
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
            GeminiBotToolInput{
                name: "callguild".to_string(),
                input_type: GeminiSchemaType::Integer,
                description: "알람을 보낼 길드 ID".to_string(),
                required: true,
                format: Some(GeminiSchemaFormat::Int64),
                default: None,
                enum_values: None,
                example: Some(
                    json!("123456789012345678".to_string())
                ),
                pattern: None,
            },
            GeminiBotToolInput{
                name: "callchannel".to_string(),
                input_type: GeminiSchemaType::Integer,
                description: "알람을 보낼 채널 ID".to_string(),
                required: true,
                format: Some(GeminiSchemaFormat::Int64),
                default: None,
                enum_values: None,
                example: Some(
                    json!("123456789012345678".to_string())
                ),
                pattern: None,
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
        response: Some(GeminiSchema{
        schema_type: GeminiSchemaType::Object,
        title: Some("Set Alarm Response Schema".to_string()),
        description: Some("Schema for the set alarm tool response".to_string()),
        properties: Some(
            BTreeMap::from([
                ("result_message".to_string(), GeminiSchema {
                    schema_type: GeminiSchemaType::String,
                    description: Some("The message that was sent in response".to_string()),
                    format: None,
                    default: None,
                    enum_values: None,
                    example: Some(json!("Alarm set for 2024-03-21 12:00:00 with message: Hello!".to_string())),
                    pattern: None,
                    ..Default::default()
                }),
                ("result".to_string(), GeminiSchema {
                    description: Some("The result of the action".to_string()),
                    schema_type: GeminiSchemaType::Object,
                    required: vec!["res".to_string()].into(),
                    default: None,
                    enum_values: None,
                    example: Some(json!({"res": "Alarm set for 2024-03-21 12:00:00 with message: Hello!"})),
                    pattern: None,
                    properties: Some(BTreeMap::from([
                        ("res".to_string(), GeminiSchema {
                            schema_type: GeminiSchemaType::String,
                            description: Some("The response message".to_string()),
                            format: None,
                            default: None,
                            enum_values: None,
                            example: Some(json!("Alarm set for 2024-03-21 12:00:00 with message: Hello!".to_string())),
                            pattern: None,
                            ..Default::default()
                        }),
                    ])),
                    title: Some("result".to_string()),
                    ..Default::default()
                }),
                ("error".to_string(), GeminiSchema {
                    schema_type: GeminiSchemaType::String,
                    description: Some("Error message if any occurred during the action".to_string()),
                    format: None,
                    default: None,
                    enum_values: None,
                    example: Some(json!(null)),
                    pattern: None,
                    title: Some("error".to_string()),
                    ..Default::default()
                }),
            ])
        ),
        required: vec!["result_message".to_string(), "result".to_string()].into(),
        default: None,
        enum_values: None,
        
            ..Default::default()
        }) 
        ,
    }
}
