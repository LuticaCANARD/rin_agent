use std::collections::{BTreeMap, HashMap};

use gemini_live_api::types::enums::GeminiSchemaType;
use serde_json::json;

use crate::{gemini::{types::{DiscordUserInfo, GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue, GeminiBotTools, GeminiImageInputType, GenerationModality, UnifiedGenerationConfig}, unified_generation::unified_generate}, setting::gemini_setting::GEMINI_NANO_BANANA};


pub async fn generate_image(params : HashMap<String,GeminiBotToolInputValue>,info:Option<DiscordUserInfo>) -> Result<GeminiActionResult, String> {
    let prompt = params.get("prompt");
    if prompt.is_none() {
      return Err("Missing 'prompt' parameter".to_string());
    }
    let prompt = prompt.unwrap().value.to_string();

    let image_url = params.get("image_url");
    let image_input = if let Some(image_url_param) = image_url {
      let image_url = image_url_param.value.to_string();
      let split_url = image_url.split('?').collect::<Vec<&str>>();
      let image_url_origin = split_url[0]; // Remove query parameters for mime type detection

      let extension = std::path::Path::new(image_url_origin)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();

      let mime = match extension.as_str() {
          "png" => "image/png",
          "jpg" | "jpeg" => "image/jpeg",
          "webp" => "image/webp",
          "heic" => "image/heic",
          "heif" => "image/heif",
          _ => { "application/octet-stream" }
      };
      if mime == "application/octet-stream" {
        return Ok(
          GeminiActionResult{
              result_message: format!("error!! : Unsupported image format"),
              result: json!({
                  "res": format!("error!! : Unsupported image format"),
              }),
              error: Some("Unsupported image format".to_string()),
              show_user: Some("지원하지 않는 이미지 형식입니다.".to_string()),
              image: None,
              audio: None,
          }
        )
      }

      Some(GeminiImageInputType {
        file_url: Some(image_url.clone()),
        mime_type: mime.to_string(),
        base64_image: None,
      })
    } else {
      None
    };

    // Use unified generation
    let config = UnifiedGenerationConfig {
        modalities: vec![GenerationModality::Text, GenerationModality::Image],
        model: GEMINI_NANO_BANANA.to_string(),
        max_output_tokens: Some(2048),
    };

    let result = unified_generate(prompt, config, image_input).await?;
    
    // Customize result for image generation
    let customized_result = GeminiActionResult {
        result_message: if result.image.is_some() {
            result.result_message
        } else {
            "이미지 데이터를 찾지 못했습니다.".to_string()
        },
        result: if result.image.is_some() {
            result.result
        } else {
            json!({ "error": "no image in response" })
        },
        error: if result.image.is_none() {
            Some("no image in response".to_string())
        } else {
            result.error
        },
        show_user: if result.image.is_some() {
            Some("이미지를 생성했어요.".to_string())
        } else {
            Some("이미지 생성 결과에서 이미지 데이터를 찾지 못했습니다.".to_string())
        },
        image: result.image,
        audio: result.audio,
    };

    Ok(customized_result)
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