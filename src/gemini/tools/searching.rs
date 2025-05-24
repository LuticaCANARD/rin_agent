use gemini_live_api::types::enums::GeminiSchemaType;
use serde_json::{json, Value};

use crate::api::google_searching::{google_searching, GoogleSearchItem};
use crate::gemini::types::{generate_input_to_dict, GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue, GeminiBotTools};

use std::collections::HashMap;
use std::sync::LazyLock;

fn example_result() -> Option<Value> {
    Some(
        json!(
            vec![
                GoogleSearchItem {
                    title: Some("Apple".to_string()),
                    link: Some("https://www.apple.com".to_string()),
                    snippet: Some("Apple Inc. is an American multinational technology company headquartered in Cupertino, California.".to_string()),
                    kind: None,
                    html_title: None,
                }
            ]
        )
    )
}
static EXAMPLE_RESULT: LazyLock<Option<Value>> = LazyLock::new(|| example_result());
async fn searching(params: HashMap<String, GeminiBotToolInputValue>) -> Result<GeminiActionResult, String> {
    let query = params.get("query");
    if query.is_none() {
        return Err("Missing 'query' parameter".to_string());
    }
    let query = query.unwrap().value.to_string();
    let google_searching_result = google_searching(query.clone()).await;
    if google_searching_result.is_err() {
        return Ok(
            GeminiActionResult{
                result_message: "Error occurred while searching".to_string(),
                result: json!({}),
                error: Some(google_searching_result.err().unwrap()),
								show_user: Some(true),
            }
        )
    }
    let google_searching_result = google_searching_result.unwrap();
    let result_message = format!("Search results for '{}':", query);
    Ok(
        GeminiActionResult{
            result_message,
            result: json!(google_searching_result),
            error: None,
						show_user: Some(false),
        }
    )
}

pub fn get_command() -> GeminiBotTools {
    GeminiBotTools {
        name: "searching".to_string(),
        description: "query에 대해서 구글 검색을 합니다. 검색한 결과는 link들의 집합으로 나옵니다. 따라서, 답해줄 정보가 부족할 결루 이 이후에 링크를 web_connect에 넣어 호출하는걸 추천합니다. ".to_string(),
        parameters: vec![
            GeminiBotToolInput {
                name:"query".to_string(),
                description:"searching".to_string(),
                input_type:GeminiSchemaType::String,
                required:true,
                format:None,
                default:None,
                enum_values:None,
                example:Some(json!("Apple".to_string())), 
                pattern:None
            }
        ]
        .into_iter()
        .map(generate_input_to_dict)
        .collect(),
        action: |params| Box::pin(async move { searching(params).await }),
        result_example: EXAMPLE_RESULT.clone()
    }
}
