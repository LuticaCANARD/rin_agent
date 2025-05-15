use std::collections::HashMap;

use serde_json::json;

use crate::gemini::types::{GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputType, GeminiBotToolInputValue, GeminiBotTools};

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
        }
    )
    
}

pub fn get_command() -> GeminiBotTools {
    GeminiBotTools {
        name: "response_msg".to_string(),
        description: "주인님에게 답합니다.".to_string(),
        parameters: vec![
            GeminiBotToolInput {
                name: "msg".to_string(),
                description: "주인님께 답할 메시지".to_string(),
                input_type: GeminiBotToolInputType::STRING,
                required: true,
            },
        ],
        action: |params| Box::pin(async move { set_alarm(params).await }),
    }
}
