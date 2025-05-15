use serde_json::{json, Value};

use crate::gemini::types::{GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputType, GeminiBotToolInputValue, GeminiBotTools};

use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;

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


    Ok(
        GeminiActionResult{
            result_message: message.clone(),
            result: json!({
                "res": message,
            }),
            error: None,
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
                input_type: GeminiBotToolInputType::STRING("Set the time for the alarm (Format is YY-MM-DD HH:MM:SS)".to_string()),
                required: true,
            },
            GeminiBotToolInput {
                name: "message".to_string(),
                input_type: GeminiBotToolInputType::STRING("알람과 함께 주인님께 보낼 메시지".to_string()),
                required: false,
            }

        ],
        action: |params| Box::pin(async move { set_alarm(params).await }),
    }
}
