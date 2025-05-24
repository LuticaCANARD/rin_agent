use std::str::FromStr;

use serde::{Deserialize, Serialize};


#[derive(Debug, Clone,Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSearchItem{
    pub kind: Option<String>,
    pub title: Option<String>,
    pub link: Option<String>,
    pub snippet: Option<String>,
    pub html_title: Option<String>,
}

pub async fn google_searching(query: String) -> Result<Vec<GoogleSearchItem>, String> {
    let token = std::env::var("GOOGLE_SEARCH_TOKEN")
        .map_err(|_| "Missing GOOGLE_SEARCH_TOKEN environment variable".to_string())?;
    let cx = std::env::var("GOOGLE_SEARCH_CX")
        .map_err(|_| "Missing GOOGLE_SEARCH_CX environment variable".to_string())?;

    let request_url = format!(
        "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}",
        token, cx, query
    );

    let response = reqwest::Client::new()
        .get(&request_url)
        .send()
        .await
        .map_err(|e| format!("Failed to perform search: {}", e))?;

    let response_txt = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response text: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&response_txt)
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

    let items = json.get("items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "No items found in the response".to_string())?;

    let results: Vec<GoogleSearchItem> = serde_json::from_value(items.clone().into())
        .map_err(|e| format!("Failed to parse items: {}", e))?;

    Ok(results)
}