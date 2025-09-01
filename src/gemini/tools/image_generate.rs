use std::{collections::{BTreeMap, HashMap}, env};

use gemini_live_api::types::{enums::{GeminiContentRole, GeminiSchemaType}, GeminiContents, GeminiParts};

use serde_json::json;

use crate::{gemini::types::{DiscordUserInfo, GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue, GeminiBotTools}, setting::gemini_setting::{GEMINI_NANO_BANANA}};


pub async fn generate_image(params : HashMap<String,GeminiBotToolInputValue>,info:Option<DiscordUserInfo>) -> Result<GeminiActionResult, String> {
    let prompt = params.get("prompt");
    if prompt.is_none() {
      return Err("Missing 'prompt' parameter".to_string());
    }
    let prompt = prompt.unwrap().value.to_string();
    // Here you would implement the logic to generate the image based on the prompt

    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let url = format!(
      "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
      GEMINI_NANO_BANANA,
      api_key
    );

    let contents = vec![
      GeminiContents{
        role: GeminiContentRole::User,
        parts: vec![
          GeminiParts{
            text: Some(prompt.clone()),
            ..Default::default()
          }
        ]
      }
    ];

    let response = reqwest::Client::new()
        .post(&url)
        .body(json!({
            "contents": contents
        }).to_string())
        .send()
        .await;
    if response.is_err() {
      let err_msg = response.err().unwrap().to_string();
      return Ok(
        GeminiActionResult{
            result_message: format!("error!! : {}", err_msg),
            result: json!({
                "res": format!("error!! : {}", err_msg),
            }),
            error: Some(err_msg),
            show_user: Some("이미지 생성 중 오류가 발생했습니다.".to_string()),
        }
      )
    }
    Ok(
      GeminiActionResult{
          result_message: format!("답했습니다! : {}", prompt),
          result: json!({
              "res": format!("답했습니다! : {}", prompt),
          }),
          error: None,
          show_user: None,
      }
    )
    
}

pub fn get_command() -> GeminiBotTools {
    GeminiBotTools {
      name: "generate_image".to_string(),
      description: "Generates an image based on the provided prompt.".to_string(),
      parameters: BTreeMap::from([
        ("prompt".to_string(), GeminiBotToolInput{
          name: "prompt".to_string(),
          input_type: GeminiSchemaType::String,
          description: "The prompt to generate the image.".to_string(),
          required: true,
          example: Some(serde_json::json!("A futuristic cityscape at sunset.")),
          pattern: None,
          enum_values: None,
          format: None,
          default: None,
        }),
      ]),
      action: |params, info| {
        Box::pin(async move { generate_image(params,info).await })
      },
      response: None,
    }
}