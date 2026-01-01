use std::collections::HashMap;

enum WebRequestType {
    Get,
    Post,
    Put,
    Delete,
}

pub async fn get_web_result(
    url: String,
    request_type: WebRequestType,
    body: Option<HashMap<String, String>>,
    headers: Option<HashMap<String, String>>,
) -> Result<String, String> {
    let request = reqwest::Client::new()
        .get(format!("https://webdocu.com/api/v1/documents?url={}", url))
        .send()
        .await;
    match request {
        Ok(response) => {
            let response_text = response.text().await;
            match response_text {
                Ok(text) => Ok(text),
                Err(e) => Err(format!("Failed to read response text: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to perform request: {}", e)),
    }
}
