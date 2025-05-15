use std::collections::HashMap;

use serde_json::json;

use crate::gemini::types::{GeminiBotTools, GeminiBotToolInputType, GeminiBotToolInput,GeminiBotToolInputValue};

async fn set_alarm(params : HashMap<String,GeminiBotToolInputValue>) -> Result<serde_json::Value, String> {

    let msg = params.get("msg");
    if msg.is_none() {
        return Err("Missing 'msg' parameter".to_string());
    }
    let msg = msg.unwrap().value.to_string();
    // Here you would implement the logic to set the alarm
    Ok(json!({
        "res": format!("주인님께 답합니다 : {}", msg),
    }))
    
}

pub fn get_command() -> GeminiBotTools {
    GeminiBotTools {
        name: "response_msg".to_string(),
        description: "주인님에게 답합니다.".to_string(),
        parameters: vec![
            GeminiBotToolInput {
                name: "msg".to_string(),
                description: "주인님께 답할 메시지".to_string(),
                input_type: GeminiBotToolInputType::STRING("주인님께 답할 메시지".to_string()),
                required: true,
            },
        ],
        action: |params| Box::pin(async move { set_alarm(params).await }),
    }
}
