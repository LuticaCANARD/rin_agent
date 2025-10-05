use std::{collections::{BTreeMap, HashMap}, env};

use base64::{prelude::BASE64_STANDARD, Engine};
use gemini_live_api::{libs::logger::{LogLevel, LOGGER}, types::{enums::{GeminiContentRole, GeminiSchemaType}, GeminiContents, GeminiParts}};

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
      "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
      GEMINI_NANO_BANANA
    );
    let image_url = params.get("image_url");
    let image_part = if image_url.is_some() {
      let image_url = image_url.unwrap().value.to_string();
      Some(
        GeminiParts{
            image: Some(image_url.clone()),
            ..Default::default()
        }
      )
    } else {
      None
    };

    let mut contents = vec![
      GeminiContents{
        role: GeminiContentRole::User,
        parts: vec![
          GeminiParts{
            text: Some(prompt.clone()),
            ..Default::default()
          }
        ],
      }
    ];

    if let Some(image_part) = image_part {
      contents[0].parts.push(image_part);
    }

    let response = reqwest::Client::new()
        .post(&url)
        .body(json!({
            "contents": contents
        }).to_string())
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
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
            image: None,
        }
      )
    }
    if !response.as_ref().unwrap().status().is_success() {
        let text = response.unwrap().text().await.unwrap_or_default();
        return Ok(GeminiActionResult{
            result_message: format!("Gemini error: {}", text),
            result: json!({ "error": text }),
            error: Some(text),
            show_user: Some("이미지 생성 중 오류가 발생했습니다.".to_string()),
            image: None,
        });
    }
    let response = response.unwrap();
    // Extract first inlineData image from candidates
    let mut found_b64: Option<(String, String)> = None; // (mime, data_b64)
    let value = response.text().await.map_err(|e| e.to_string())?;

    let value: serde_json::Value = serde_json::from_str(&value).map_err(|e| e.to_string())?;
    if let Some(cands) = value.get("candidates").and_then(|c| c.as_array()) {
        for c in cands {
            if let Some(parts) = c.get("content").and_then(|ct| ct.get("parts")).and_then(|p| p.as_array()) {
                for p in parts {
                    // Support both inlineData and inline_data keys
                    if let Some(inline) = p.get("inlineData").or_else(|| p.get("inline_data")) {
                        if let (Some(mime), Some(data)) = (inline.get("mimeType").and_then(|m| m.as_str()), inline.get("data").and_then(|d| d.as_str())) {
                            found_b64 = Some((mime.to_string(), data.to_string()));
                            break;
                        }
                    }
                }
            }
            if found_b64.is_some() { break; }
        }
    }

    match found_b64 {
        Some((mime, b64)) => {
            let bytes = BASE64_STANDARD.decode(b64.as_bytes()).map_err(|e| format!("base64 decode error: {e}"))?;
            Ok(GeminiActionResult{
                result_message: format!("이미지 생성 완료 ({}, {} bytes)", mime, bytes.len()),
                result: json!({
                    "model": "gemini-2.5-flash-image-preview",
                    "mime": mime,
                    "size": bytes.len()
                }),
                error: None,
                show_user: Some("이미지를 생성했어요.".to_string()),
                image: Some(bytes),
            })
        }
        None => {
            // Fallback: return raw JSON for debugging
            Ok(GeminiActionResult{
                result_message: "이미지 데이터를 찾지 못했습니다.".to_string(),
                result: value,
                error: Some("no inlineData image in response".to_string()),
                show_user: Some("이미지 생성 결과에서 이미지 데이터를 찾지 못했습니다.".to_string()),
                image: None,
            })
        }
    }
    
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
        ("image_url".to_string(),GeminiBotToolInput{
          name: "image_url".to_string(),
          input_type: GeminiSchemaType::String,
          description: "Optional URL of an image to base the generation on.".to_string(),
          required: false,
          example: Some(serde_json::json!("https://example.com/image.png")),
          pattern: None,
          enum_values: None,
          format: None,
          default: None,
        })
      ]),
      action: |params, info| {
        Box::pin(async move { generate_image(params,info).await })
      },
      response: None,
    }
}