use std::{collections::{BTreeMap, HashMap}, vec};

use gemini_live_api::types::{enums::GeminiSchemaType, GeminiSchema, GeminiSchemaObject};
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
        action: |params,info| Box::pin(async move { set_alarm(params).await }),
        response: Some(GeminiSchema {
            schema_type: GeminiSchemaType::Object,
            title: Some("Response Message Schema".to_string()),
            description: Some("Schema for the response message tool".to_string()),
            properties: Some(BTreeMap::from([
                ("result_message".to_string(), GeminiSchema {
                    schema_type: GeminiSchemaType::String,
                    description: Some("The message that was sent in response".to_string()),
                    format: None,
                    default: None,
                    enum_values: None,
                    example: Some(json!("답했습니다! : 주인님, 안녕하세요!".to_string())),
                    pattern: None,
                    ..Default::default()
                }),
                ("result".to_string(), GeminiSchema {
                    description: Some("The result of the action".to_string()),
                    schema_type: GeminiSchemaType::Object,
                    required: vec!["res".to_string()].into(),
                    default: None,
                    enum_values: None,
                    example: Some(json!({"res": "답했습니다! : 주인님, 안녕하세요!"})),
                    pattern: None,
                    properties: Some(BTreeMap::from([
                        ("res".to_string(), GeminiSchema {
                            schema_type: GeminiSchemaType::String,
                            description: Some("The response message".to_string()),
                            format: None,
                            default: None,
                            enum_values: None,
                            example: Some(json!("답했습니다! : 주인님, 안녕하세요!".to_string())),
                            pattern: None,
                            ..Default::default()
                        }),
                    ])),
                    ..Default::default()
                }),
                ("error".to_string(), GeminiSchema {
                    schema_type: GeminiSchemaType::String,
                    description: Some("The message that was sent in response".to_string()),
                    format: None,
                    default: None,
                    enum_values: None,
                    example: Some(json!(null)),
                    pattern: None,
                    ..Default::default()
                }),
            ])),
            required: Some(vec!["result_message".to_string(), "result".to_string()]),
            ..Default::default()
        })
    }
}
