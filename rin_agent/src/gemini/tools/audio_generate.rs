use std::collections::{BTreeMap, HashMap};

use gemini_live_api::types::enums::GeminiSchemaType;
use serde_json::json;

use crate::gemini::{
    types::{
        DiscordUserInfo, GeminiActionResult, GeminiBotToolInput, GeminiBotToolInputValue,
        GeminiBotTools, GenerationModality, UnifiedGenerationConfig,
    },
    unified_generation::unified_generate,
};
use crate::setting::gemini_setting::GEMINI_NANO_BANANA;

pub async fn generate_audio(
    params: HashMap<String, GeminiBotToolInputValue>,
    info: Option<DiscordUserInfo>,
) -> Result<GeminiActionResult, String> {
    let prompt = params.get("prompt");
    if prompt.is_none() {
        return Err("Missing 'prompt' parameter".to_string());
    }
    let prompt = prompt.unwrap().value.to_string();

    // Use unified generation for audio
    let config = UnifiedGenerationConfig {
        modalities: vec![GenerationModality::Text, GenerationModality::Audio],
        model: GEMINI_NANO_BANANA.to_string(),
        max_output_tokens: Some(2048),
    };

    let result = unified_generate(prompt, config, None).await?;

    // Customize result for audio generation
    let customized_result = GeminiActionResult {
        result_message: if result.audio.is_some() {
            result.result_message
        } else {
            "오디오 데이터를 찾지 못했습니다.".to_string()
        },
        result: if result.audio.is_some() {
            result.result
        } else {
            json!({ "error": "no audio in response" })
        },
        error: if result.audio.is_none() {
            Some("no audio in response".to_string())
        } else {
            result.error
        },
        show_user: if result.audio.is_some() {
            Some("음성을 생성했어요.".to_string())
        } else {
            Some("음성 생성 결과에서 오디오 데이터를 찾지 못했습니다.".to_string())
        },
        image: result.image,
        audio: result.audio,
    };

    Ok(customized_result)
}

pub fn get_command() -> GeminiBotTools {
    GeminiBotTools {
        name: "generate_audio".to_string(),
        description: "Generates audio/voice based on the provided text prompt.".to_string(),
        parameters: BTreeMap::from([(
            "prompt".to_string(),
            GeminiBotToolInput {
                name: "prompt".to_string(),
                input_type: GeminiSchemaType::String,
                description: "The text prompt to generate audio from.".to_string(),
                required: true,
                example: Some(serde_json::json!("안녕하세요, 만나서 반갑습니다.")),
                pattern: None,
                enum_values: None,
                format: None,
                default: None,
            },
        )]),
        action: |params, info| Box::pin(async move { generate_audio(params, info).await }),
        response: None,
    }
}
