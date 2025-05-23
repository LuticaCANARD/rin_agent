use gemini_live_api::types::enums::GeminiSchemaType;
use serde_json::{json, Value};

use crate::gemini::types::{generate_input_to_dict, GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue, GeminiBotTools};

use std::collections::HashMap;
use std::pin::Pin;
use std::future::Future;

async fn searching(params: HashMap<String, GeminiBotToolInputValue>) -> Result<GeminiActionResult, String> {
    let query = params.get("query");
    if query.is_none() {
        return Err("Missing 'query' parameter".to_string());
    }
    let query = query.unwrap().value.to_string();
    
    // Here you would implement the logic to perform the search
    let result_message = format!("Searching for: {}", query);
    
    Ok(
        GeminiActionResult{
            result_message: result_message.clone(),
            result: json!({
                "res": result_message,
            }),
            error: None,
        }
    )
}

pub fn get_command() -> GeminiBotTools {
    GeminiBotTools {
        name: "searching".to_string(),
        description: "searching".to_string(),
        parameters: vec![
            GeminiBotToolInput {
                name: "query".to_string(),
                description: "searching".to_string(),
                input_type: GeminiSchemaType::STRING,
                required: true,
                format: None,
                default: None,
                enum_values: None,
                example: Some(
                    json!("Apple".to_string())
                ),
                pattern: None,
            }
        ].into_iter().map(generate_input_to_dict).collect(),
        
        action: |params| Box::pin(async move { searching(params).await }),
        result_example: Some(serde_json::json!({
            "result_message": "Searching for: Apple",
            "result": { "res": "Searching for: Apple" },
            "error": null
        })),
    }
}
