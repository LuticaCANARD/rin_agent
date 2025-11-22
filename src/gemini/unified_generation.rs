use std::env;
use base64::{prelude::BASE64_STANDARD, Engine};
use gemini_live_api::types::{enums::GeminiContentRole, GeminiContents, GeminiParts};
use serde_json::json;

use crate::gemini::types::{GenerationModality, UnifiedGenerationConfig, GeminiActionResult, GeminiImageInputType};
use crate::gemini::utils::upload_image_to_gemini;
use crate::setting::gemini_setting::GEMINI_NANO_BANANA;

/// Unified generation function that can handle text, image, and audio generation.
/// 
/// This function provides a single interface for generating content across multiple modalities:
/// - Text generation
/// - Image generation
/// - Audio/voice generation
/// 
/// # Arguments
/// 
/// * `prompt` - The text prompt for generation
/// * `config` - Configuration specifying the model and modalities to use
/// * `image_input` - Optional image input for image-to-image or multimodal generation
/// 
/// # Returns
/// 
/// Returns a `GeminiActionResult` containing the generated content. Depending on the modalities
/// specified in the config, the result may contain text, image data, or audio data.
/// 
/// # Example
/// 
/// ```rust,ignore
/// use crate::gemini::types::{GenerationModality, UnifiedGenerationConfig};
/// 
/// // Generate an image
/// let config = UnifiedGenerationConfig {
///     modalities: vec![GenerationModality::Text, GenerationModality::Image],
///     model: "gemini-2.5-flash-image".to_string(),
///     max_output_tokens: Some(2048),
/// };
/// let result = unified_generate("A beautiful sunset".to_string(), config, None).await?;
/// 
/// // Generate audio
/// let config = UnifiedGenerationConfig {
///     modalities: vec![GenerationModality::Text, GenerationModality::Audio],
///     model: "gemini-2.5-flash-image".to_string(),
///     max_output_tokens: Some(2048),
/// };
/// let result = unified_generate("Hello world".to_string(), config, None).await?;
/// ```
pub async fn unified_generate(
    prompt: String,
    config: UnifiedGenerationConfig,
    image_input: Option<GeminiImageInputType>,
) -> Result<GeminiActionResult, String> {
    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        config.model
    );

    // Prepare input parts
    let mut parts = vec![
        GeminiParts {
            text: Some(prompt.clone()),
            ..Default::default()
        }
    ];

    // Add image if provided
    if let Some(image_input_data) = image_input {
        let image_url = image_input_data.file_url.clone();
        let mime = image_input_data.mime_type.clone();
        
        if let Some(url) = image_url {
            let upload = upload_image_to_gemini(
                GeminiImageInputType {
                    file_url: Some(url.clone()),
                    mime_type: mime.clone(),
                    base64_image: None,
                },
                format!("image_{}.{}", url.split('/').last().unwrap_or("unknown"), mime.split('/').nth(1).unwrap_or("png"))
            ).await;

            if let Err(err) = upload {
                return Ok(GeminiActionResult {
                    result_message: format!("error!! : {}", err.clone()),
                    result: json!({
                        "res": format!("error!! : {}", err.clone()),
                    }),
                    error: Some(err.clone()),
                    show_user: Some("이미지 업로드 중 오류가 발생했습니다.".to_string()),
                    image: None,
                    audio: None,
                });
            }

            let upload = upload.unwrap();
            parts.push(GeminiParts {
                file_data: Some(gemini_live_api::types::GeminiFileData {
                    mime_type: Some(upload.mime_type),
                    file_uri: upload.file_url.unwrap(),
                }),
                ..Default::default()
            });
        }
    }

    let contents = vec![GeminiContents {
        role: GeminiContentRole::User,
        parts,
    }];

    // Build modalities array
    let modalities: Vec<String> = config.modalities.iter().map(|m| {
        match m {
            GenerationModality::Text => "TEXT".to_string(),
            GenerationModality::Image => "IMAGE".to_string(),
            GenerationModality::Audio => "AUDIO".to_string(),
        }
    }).collect();

    // Make API request
    let response = reqwest::Client::new()
        .post(&url)
        .body(json!({
            "contents": contents,
            "generation_config": {
                "response_modalities": modalities,
                "maxOutputTokens": config.max_output_tokens.unwrap_or(2048)
            },
        }).to_string())
        .header("x-goog-api-key", api_key)
        .header("Content-Type", "application/json")
        .send()
        .await;

    if response.is_err() {
        let err_msg = response.err().unwrap().to_string();
        return Ok(GeminiActionResult {
            result_message: format!("error!! : {}", err_msg),
            result: json!({
                "res": format!("error!! : {}", err_msg),
            }),
            error: Some(err_msg),
            show_user: Some("생성 중 오류가 발생했습니다.".to_string()),
            image: None,
            audio: None,
        });
    }

    let response = response.unwrap();
    if !response.status().is_success() {
        let text = response.text().await.unwrap_or_default();
        return Ok(GeminiActionResult {
            result_message: format!("Gemini error: {}", text),
            result: json!({ "error": text }),
            error: Some(text.clone()),
            show_user: Some("생성 중 오류가 발생했습니다.".to_string()),
            image: None,
            audio: None,
        });
    }

    // Parse response
    let value = response.text().await.map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_str(&value).map_err(|e| e.to_string())?;

    let mut text_result: Option<String> = None;
    let mut found_image: Option<(String, Vec<u8>)> = None; // (mime, data)
    let mut found_audio: Option<(String, Vec<u8>)> = None; // (mime, data)

    if let Some(cands) = value.get("candidates").and_then(|c| c.as_array()) {
        for c in cands {
            if let Some(parts) = c.get("content").and_then(|ct| ct.get("parts")).and_then(|p| p.as_array()) {
                for p in parts {
                    // Extract text
                    if text_result.is_none() {
                        if let Some(text) = p.get("text").and_then(|t| t.as_str()) {
                            text_result = Some(text.to_string());
                        }
                    }

                    // Extract image
                    if found_image.is_none() {
                        if let Some(inline) = p.get("inlineData").or_else(|| p.get("inline_data")) {
                            if let (Some(mime), Some(data)) = (
                                inline.get("mimeType").and_then(|m| m.as_str()),
                                inline.get("data").and_then(|d| d.as_str())
                            ) {
                                if mime.starts_with("image/") {
                                    let bytes = BASE64_STANDARD.decode(data.as_bytes())
                                        .map_err(|e| format!("base64 decode error: {e}"))?;
                                    found_image = Some((mime.to_string(), bytes));
                                }
                            }
                        }
                    }

                    // Extract audio
                    if found_audio.is_none() {
                        if let Some(inline) = p.get("inlineData").or_else(|| p.get("inline_data")) {
                            if let (Some(mime), Some(data)) = (
                                inline.get("mimeType").and_then(|m| m.as_str()),
                                inline.get("data").and_then(|d| d.as_str())
                            ) {
                                if mime.starts_with("audio/") {
                                    let bytes = BASE64_STANDARD.decode(data.as_bytes())
                                        .map_err(|e| format!("base64 decode error: {e}"))?;
                                    found_audio = Some((mime.to_string(), bytes));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Build result
    let result_msg = text_result.unwrap_or("생성 완료".to_string());
    let mut result_json = json!({
        "model": config.model,
    });

    if let Some((mime, _)) = &found_image {
        result_json["image_mime"] = json!(mime);
    }
    if let Some((mime, _)) = &found_audio {
        result_json["audio_mime"] = json!(mime);
    }

    Ok(GeminiActionResult {
        result_message: result_msg,
        result: result_json,
        error: None,
        show_user: Some("생성이 완료되었습니다.".to_string()),
        image: found_image.map(|(_, data)| data),
        audio: found_audio.map(|(_, data)| data),
    })
}
