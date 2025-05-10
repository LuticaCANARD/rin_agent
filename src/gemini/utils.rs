use std::env;

use crate::{gemini::types::GeminiImageInputType, libs::logger::LOGGER};
use base64::Engine;
use serde_json::json;
use reqwest::header::{HeaderName, HeaderValue};

use super::types::GeminiChatChunk;



fn generate_gemini_string_from_chunk(chunk: &GeminiChatChunk) -> String {
    format!("
    time : {} 
    sender : {}
    message : {}
    ",&chunk.timestamp,if !chunk.is_bot {chunk.user_id.clone().unwrap()} else {String::from("0")}, chunk.query
    ).to_string()
}
fn generate_gemini_image_chunk(chunk:Option<GeminiImageInputType>) -> Option<serde_json::Value> {
    if chunk.is_none() {
        return None;
    }
    let chunk = chunk.unwrap();
    if let Some(base64_image) = chunk.base64_image {
        Some(json!({
            "inline_data": {
                "mime_type": chunk.mime_type,
                "data": base64_image
            }
        }))
    } else if let Some(file_url) = chunk.file_url {
        Some(json!({
            "file_data": {
                "mime_type": chunk.mime_type,
                "file_uri": file_url
            }
        }))
    } else {
        None
    }
}

pub fn generate_gemini_user_chunk(chunk: &GeminiChatChunk)->serde_json::Value {
    // 이미지가 있는 경우에만 진입
    if let Some(image) = generate_gemini_image_chunk(chunk.image.clone()) {
        json!({
            "role" : if chunk.is_bot {"model"} else {"user"},
            "parts": 
                [
                    { "text": generate_gemini_string_from_chunk(chunk)},
                    image
                ]
        })
    } else {
        json!({
            "role" : if chunk.is_bot {"model"} else {"user"},
            "parts": [{ "text": generate_gemini_string_from_chunk(chunk) }]
        })
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