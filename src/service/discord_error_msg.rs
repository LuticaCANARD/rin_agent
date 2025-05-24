

pub async fn send_debug_error_log(
    message: String
){
  let token = std::env::var("DISCORD_WEBHOOK_TOKEN").unwrap_or_else(|_| "default_token".to_string());
  reqwest::Client::new()
    .post(format!("https://discord.com/api/webhooks/1375871801409798186/{}", token))
    .body(
        serde_json::to_string(&serde_json::json!({
            "embeds": [
                {
                    "title": "Rin Agent Debug Error",
                    "description": message,
                    "color": 16711680, // Red color
                    "footer": {
                        "text": "Rin Agent Debug Log"
                    }
                }
            ],
        }))
        .unwrap()
    )
    .send()
    .await
    .map_err(|e| e.to_string())
    .ok();
}