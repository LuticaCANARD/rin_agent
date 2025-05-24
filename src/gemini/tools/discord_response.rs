use std::collections::{BTreeMap, HashMap};

use gemini_live_api::types::enums::GeminiSchemaType;
use serde_json::json;

use crate::gemini::types::{GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue, GeminiBotTools};

async fn set_alarm(params : HashMap<String,GeminiBotToolInputValue>) -> Result<GeminiActionResult, String> {

    let msg = params.get("msg");
    if msg.is_none() {
        return Err("Missing 'msg' parameter".to_string());
    }
    let msg = msg.unwrap().value.to_string();
    // Here you would implement the logic to set the alarm
    Ok(
        GeminiActionResult{
            result_message: format!("답했습니다! : {}", msg),
            result: json!({
                "res": format!("답했습니다! : {}", msg),
            }),
            error: None,
            show_user: None,
        }
    )
    
}

pub fn get_command() -> GeminiBotTools {
    GeminiBotTools {
        name: "response_msg".to_string(),
        description: "주인님에게 답합니다.".to_string(),
        parameters: BTreeMap::from([
            ("msg".to_string(), GeminiBotToolInput {
                name: "msg".to_string(),
                description: "주인님께 답할 메시지".to_string(),
                input_type: GeminiSchemaType::String,
                required: true,
                format: None,
                default: None,
                enum_values: None,
                example: Some(
                    json!("주인님, 안녕하세요!".to_string())
                ),
                pattern: None,
            })
        ]),
        action: |params| Box::pin(async move { set_alarm(params).await }),
        result_example: Some(serde_json::json!({
            "result_message": "답했습니다! : 주인님, 안녕하세요!",
            "result": { "res": "답했습니다! : 주인님, 안녕하세요!" },
            "error": null
        })),
    }
}
