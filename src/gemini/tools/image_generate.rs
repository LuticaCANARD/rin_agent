use std::{collections::{BTreeMap, HashMap}, env};

use base64::{prelude::BASE64_STANDARD, Engine};
use gemini_live_api::{libs::logger::{LogLevel, LOGGER}, types::{enums::{GeminiContentRole, GeminiSchemaType}, GeminiContents, GeminiParts}};

use libc::rand;
use serde_json::json;

use crate::{gemini::{types::{DiscordUserInfo, GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue, GeminiBotTools, GeminiImageInputType}, utils::upload_image_to_gemini}, setting::gemini_setting::GEMINI_NANO_BANANA};


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
      let split_url = image_url.split('?').collect::<Vec<&str>>();
      let image_url_origin = split_url[0]; // Remove query parameters for mime type detection
      let mime = if image_url_origin.ends_with(".png") {
        "image/png"
      } else if image_url_origin.ends_with(".jpg") || image_url_origin.ends_with(".jpeg") {
        "image/jpeg"
      } else if image_url_origin.ends_with(".gif") {
        "image/gif"
      } else if image_url_origin.ends_with(".webp") {
        "image/webp"
      } else {
        "application/octet-stream" // Fallback mime type
      };

      let upload = upload_image_to_gemini(
        GeminiImageInputType{
          file_url: Some(image_url.clone()),
          mime_type: mime.to_string(),
          base64_image: None,
        },
        format!("image_{}.{}", image_url_origin.split('/').last().unwrap_or("unknown"), mime.split('/').nth(1).unwrap_or("png"))).await;
      println!("Image URL provided: {}, detected mime type: {}", image_url, mime);
      if let Err(err) = upload {
        return Ok(
          GeminiActionResult{
              result_message: format!("error!! : {}", err.clone()),
              result: json!({
                  "res": format!("error!! : {}", err.clone()),
              }),
              error: Some(err.clone()),
              show_user: Some("이미지 업로드 중 오류가 발생했습니다.".to_string()),
              image: None,
          }
        )
      }
      let upload = upload.unwrap();
      let mime = upload.mime_type;
      let file_url = upload.file_url;
      Some(
        GeminiParts{
            file_data: Some(gemini_live_api::types::GeminiFileData{
                mime_type: Some(mime.to_string()),
                file_uri: file_url.unwrap(),
            }),
            ..Default::default()
        }
      )
    } else {
      None
    };

    
    println!("Generating image with prompt: {}", prompt);
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
            "contents": contents,
            "generation_config" : {
              "response_modalities": ["TEXT", "IMAGE"],
              "maxOutputTokens" : 2048
            },
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
    let mut text_result:Option<String> = None;

    let value: serde_json::Value = serde_json::from_str(&value).map_err(|e| e.to_string())?;
    if let Some(cands) = value.get("candidates").and_then(|c| c.as_array()) {
        for c in cands {
            if let Some(parts) = c.get("content").and_then(|ct| ct.get("parts")).and_then(|p| p.as_array()) {
                for p in parts {
                    // Support both inlineData and inline_data keys
                    if let Some(inline) = p.get("inlineData").or_else(|| p.get("inline_data")) {
                        if let (Some(mime), Some(data)) = (inline.get("mimeType").and_then(|m| m.as_str()), 
                        inline.get("data").and_then(|d| d.as_str())) {
                            found_b64 = Some((mime.to_string(), data.to_string()));
                        }
                        LOGGER.log(LogLevel::Debug, &format!("Found inlineData with mime: {} ", inline.get("mimeType")
                        .and_then(|m| m.as_str()).unwrap_or_default()));
                    }
                    if text_result.is_none() {
                        if let Some(text) = p.get("text").and_then(|t| t.as_str()) {
                            text_result = Some(text.to_string());
                        }
                    }
                }
            }
        }
    }

    match found_b64 {
        Some((mime, b64)) => {
            let bytes = BASE64_STANDARD.decode(b64.as_bytes()).map_err(|e| format!("base64 decode error: {e}"))?;
            Ok(GeminiActionResult{
                result_message: text_result.unwrap_or("이미지 생성 완료".to_string()),
                result: json!({
                    "model": GEMINI_NANO_BANANA,
                    "mime": mime,
                    "size": bytes.len(),
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
        }),
      ]),
      action: |params, info| {
        Box::pin(async move { generate_image(params,info).await })
      },
      response: None,
    }
}