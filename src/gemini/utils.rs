use std::{collections::BTreeMap, env};

use crate::{gemini::types::{generate_to_schema, GeminiImageInputType}, libs::logger::LOGGER};
use base64::Engine;
use gemini_live_api::types::{enums::{GeminiContentRole, GeminiSchemaType}, GeminiContents, GeminiFileData, GeminiFunctionDeclaration, GeminiInlineBlob, GeminiParts, GeminiSchemaObject};
use serde_json::json;
use reqwest::header::{HeaderName, HeaderValue};

use super::types::{ GeminiBotToolInputValueType, GeminiBotTools, GeminiChatChunk};



fn generate_gemini_string_from_chunk(chunk: &GeminiChatChunk) -> String {
    format!("
    guild_id : {}
    channel_id : {}
    time : {} 
    sender : {}
    message : {}

    ",
    &chunk.guild_id.unwrap_or(0),
    &chunk.channel_id.unwrap_or(0),
    &chunk.timestamp,if !chunk.is_bot {chunk.user_id.clone().unwrap()} else {String::from("0")}, chunk.query
    ).to_string()
}
fn generate_gemini_image_chunk(chunk:Option<GeminiImageInputType>) -> Option<GeminiParts> {
    if chunk.is_none() {
        return None;
    }
    let chunk = chunk.unwrap();
    if let Some(base64_image) = chunk.base64_image {
        Some(GeminiParts{
            inline_data: Some(GeminiInlineBlob{
                mime_type: chunk.mime_type,
                data: base64_image,
            }),
            ..Default::default()
        })
    } else if let Some(file_url) = chunk.file_url {
        Some(GeminiParts{
            file_data: Some(GeminiFileData{
                mime_type: Some(chunk.mime_type),
                file_uri: file_url,
            }),
            ..Default::default()
        }
        )
    } else {
        None
    }
}

pub fn generate_gemini_user_chunk(chunk: &GeminiChatChunk)->GeminiContents {
    // 이미지가 있는 경우에만 진입
    if let Some(image) = generate_gemini_image_chunk(chunk.image.clone()) {
        GeminiContents{
            role: if chunk.is_bot {GeminiContentRole::Model} else {GeminiContentRole::User},
            parts: vec![
                GeminiParts {
                    text: Some(generate_gemini_string_from_chunk(chunk)),
                    inline_data: None,
                    file_data: None,
                    ..Default::default()
                },
                image
            ]
        }
    } else {
        GeminiContents{
            role: if chunk.is_bot {GeminiContentRole::Model} else {GeminiContentRole::User},
            parts: vec![GeminiParts {
                text: Some(generate_gemini_string_from_chunk(chunk)),
                inline_data: None,
                file_data: None,
                ..Default::default()
            }]
        }
    }
    
}

pub async fn generate_attachment_url_to_gemini(attachment_url: String) -> Result<GeminiImageInputType, String> {
    LOGGER.log(crate::libs::logger::LogLevel::Debug, &format!("Image fetch start: {:?}", attachment_url));

    let response = reqwest::get(attachment_url.clone()).await;
    match response{
        Ok(response) => {
            let headers = response.headers().clone();
            let bytes = response.bytes().await.map_err(|e| e.to_string())?;
            let base64_image = base64::engine::general_purpose::STANDARD.encode(&bytes);
            let mime_type = headers.get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("image/png")
                .to_string();
            LOGGER.log(crate::libs::logger::LogLevel::Debug, &format!("Image fetch success: {:?}", attachment_url));
            Ok(GeminiImageInputType {
                base64_image : Some(base64_image),
                file_url : None,
                mime_type
            })
        },
        Err(e) => {
            LOGGER.log(crate::libs::logger::LogLevel::Error, &format!("Image fetch error: {:?}", e));
            Err(format!("Failed to fetch image: {}", e))
        }
    }
}

pub async fn generate_attachment_url_to_gemini_with_url(attachment_url: String,mime:String) -> Result<GeminiImageInputType, String> {
    Ok(GeminiImageInputType{
        base64_image: None,
        file_url: Some(attachment_url),
        mime_type: mime,
    })
}


pub async fn upload_image_to_gemini(image: GeminiImageInputType,display_name:String) -> Result<GeminiImageInputType, String> {
    
// curl "https://generativelanguage.googleapis.com/upload/v1beta/files?key=${GOOGLE_API_KEY}" \
//   -D upload-header.tmp \
//   -H "X-Goog-Upload-Protocol: resumable" \
//   -H "X-Goog-Upload-Command: start" \
//   -H "X-Goog-Upload-Header-Content-Length: ${NUM_BYTES}" \
//   -H "X-Goog-Upload-Header-Content-Type: ${MIME_TYPE}" \
//   -H "Content-Type: application/json" \
//   -d "{'file': {'display_name': '${DISPLAY_NAME}'}}"

    let api_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY must be set");
    let url = format!("https://generativelanguage.googleapis.com/upload/v1beta/files?key={}", api_key);
    let mut target_image:Vec<u8> = Vec::new();
    if let Some(base64_image) = image.base64_image {
        //let base64_image = base64::engine::general_purpose::STANDARD.decode(image.base64_image.unwrap()).unwrap();
        target_image = base64::engine::general_purpose::STANDARD.decode(base64_image).unwrap();
    } else if let Some(file_url) = image.file_url {
        let response = reqwest::get(file_url.clone()).await.map_err(|e| e.to_string())?;
        if response.status().is_success() {
            target_image = response.bytes().await.map_err(|e| e.to_string())?.to_vec();
        } else {
            return Err(format!("Failed to fetch image from URL: {}", file_url));
        }
    } else {
        return Err("No image data provided".to_string());
    }
    

    let mut request_header = reqwest::header::HeaderMap::new();

    // Example: Add a header with a valid name and value
     request_header.insert(
        HeaderName::from_static("x-goog-upload-header-content-length"),
        reqwest::header::HeaderValue::from(target_image.len() as u64),
        );
    request_header.insert(
        HeaderName::from_static("x-goog-upload-protocol"),
        HeaderValue::from_static("resumable"),
    );
    request_header.insert(
        HeaderName::from_static("x-goog-upload-command"),
        HeaderValue::from_static("start"),
    );
    request_header.insert(
        HeaderName::from_static("x-goog-upload-header-content-type"),
        HeaderValue::from_str(&image.mime_type).unwrap(),
    );
    request_header.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_static("application/json"),
    );
    let body_for_upload = json!({
        "file": {
            "display_name": display_name
        }
    });
    let client = reqwest::Client::new();

    let response = client
        .post(&url)
        .headers(request_header)
        .body(
            body_for_upload
                .to_string()
                .into_bytes()
        ).send()
        .await
        .map_err(|e| e.to_string())?;
    //upload_url=$(grep -i "x-goog-upload-url: " "${tmp_header_file}" | cut -d" " -f2 | tr -d "\r")
    //rm "${tmp_header_file}"
    let upload_url = response.headers().get("x-goog-upload-url").unwrap().to_str().unwrap().to_string();

    //curl "${upload_url1}" \
    //   -H "Content-Length: ${NUM1_BYTES}" \
    //   -H "X-Goog-Upload-Offset: 0" \
    //   -H "X-Goog-Upload-Command: upload, finalize" \
    //   --data-binary "@${IMAGE1_PATH}" 2> /dev/null > file_info1.json

    // file1_uri=$(jq ".file.uri" file_info1.json)
    // echo file1_uri=$file1_uri

    let upload_client = reqwest::Client::new();
    let response = upload_client
        .post(&upload_url)
        .header("Content-Length", target_image.len())
        .header("X-Goog-Upload-Offset", 0)
        .header("X-Goog-Upload-Command", "upload, finalize")
        .body(target_image)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    //file1_uri=$(jq ".file.uri" file_info1.json)
    //echo file1_uri=$file1_uri
    let file_info = response.text().await.map_err(|e| e.to_string())?;
    let file_info: serde_json::Value = serde_json::from_str(&file_info).map_err(|e| e.to_string())?;
    let file_uri = file_info["file"]["uri"].as_str().unwrap_or("").to_string();
    Ok(GeminiImageInputType{
        base64_image: None,
        file_url: Some(file_uri),
        mime_type: image.mime_type,
    })
}

pub fn generate_fns_to_gemini(tool:&GeminiBotTools) -> GeminiFunctionDeclaration {
    let mut properties = BTreeMap::new();
    let mut required_param = Vec::new();

    let keys = tool.parameters.keys();
    for parameter_key in keys {
        let param = tool.parameters.get(parameter_key).unwrap();

        properties.insert(
            parameter_key.to_string(),
            generate_to_schema(&param)
        );
        if param.required {
            required_param.push(param.name.clone());
        }
    }
    let parameters = Some(GeminiSchemaObject {
        title: Some(tool.name.clone()),
        properties,
        required: required_param.clone(),
        description: Some(tool.description.clone()),
        default: None,
        example: None,
        ..Default::default()
    });


    GeminiFunctionDeclaration{
        name: tool.name.clone(),
        description: tool.description.clone(),
        parameters,
        response : tool.response.clone(), 
    }
} 

pub fn translate_to_gemini_param(value: &serde_json::Value) -> GeminiBotToolInputValueType {
    match value {
        serde_json::Value::String(s) => GeminiBotToolInputValueType::String(s.clone()),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                GeminiBotToolInputValueType::Integer(i as i64)
            } else if let Some(f) = n.as_f64() {
                GeminiBotToolInputValueType::Number(f as f64)
            } else {
                GeminiBotToolInputValueType::Null
            }
        }
        serde_json::Value::Bool(b) => GeminiBotToolInputValueType::Boolean(*b),
        serde_json::Value::Array(arr) => GeminiBotToolInputValueType::Array(
            arr.iter()
                .map(|v| translate_to_gemini_param(v))
                .collect(),
        ),
        serde_json::Value::Object(obj) => GeminiBotToolInputValueType::Object(
            obj.iter()
                .map(|(k, v)| (k.clone(), translate_to_gemini_param(v)))
                .collect(),
        ),
        _ => GeminiBotToolInputValueType::Null,
    }
}
