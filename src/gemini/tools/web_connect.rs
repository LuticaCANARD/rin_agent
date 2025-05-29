use std::{collections::{BTreeMap, HashMap}, sync::LazyLock};

use gemini_live_api::types::{enums::{GeminiSchemaFormat, GeminiSchemaType}, GeminiSchema};
use serde_json::{json, Value};

use crate::gemini::types::{generate_input_to_dict, generate_to_schema, GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue, GeminiBotTools};



fn example_result() -> Option<Value> {
    Some(
      json!({
        "html" : "<html><body><h1>Example Web Page</h1><p>This is an example web page content.</p></body></html>",
      })
    )
}

async fn web_connect(params: HashMap<String, GeminiBotToolInputValue>) -> Result<GeminiActionResult, String> {
    let url_value = params.get("url");
    if url_value.is_none() {
        return Err("Missing 'url' parameter".to_string());
    }
    let url = url_value.unwrap().value.to_string();

    // Perform the web request
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await.map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Err(format!("Failed to fetch URL: {}", response.status()));
    }

    let html_content = response.text().await.map_err(|e| e.to_string())?;

    Ok(GeminiActionResult {
        result_message: "Web page content fetched successfully.".to_string(),
        result: json!({ "html": html_content }),
        error: None,
        show_user: Some(format!("웹 페이지 {} 의 HTML 내용을 가져왔습니다.", url)),
    })
}

static EXAMPLE_RESULT: LazyLock<Option<Value>> = LazyLock::new(|| example_result());


pub fn get_command()-> GeminiBotTools {
    GeminiBotTools {
        name: "web_connect".to_string(),
        description: "웹 페이지의 URL을 입력하면 해당 페이지의 HTML 내용을 반환합니다.".to_string(),
        parameters: vec![
            GeminiBotToolInput {
                name: "url".to_string(),
                description: "웹 페이지의 URL".to_string(),
                input_type: GeminiSchemaType::String,
                required: true,
                format: None,
                default: None,
                enum_values: None,
                example: Some(json!("https://example.com".to_string())),
                pattern: None,
            }
        ]
        .into_iter()
        .map(generate_input_to_dict)
        .collect(),
        response: Some(GeminiSchema {
            schema_type: GeminiSchemaType::Object,
            title: Some("Web Connect Schema".to_string()),
            description: Some("Schema for the web connect tool".to_string()),
            properties: Some(BTreeMap::from([
                ("html".to_string(), generate_to_schema(&GeminiBotToolInput {
                    name: "html".to_string(),
                    description: "HTML content of the web page".to_string(),
                    input_type: GeminiSchemaType::String,
                    required: true,
                    format: None,
                    default: None,
                    enum_values: None,
                    example: Some(json!("<html><body><h1>Example</h1></body></html>")),
                    pattern: None,
                })),
            ])),
            ..Default::default()
        }),
        action: |params| Box::pin(async move { web_connect(params).await }),
    }
}