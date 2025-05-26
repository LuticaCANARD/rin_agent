
use crate::libs::logger::{LogLevel,LOGGER};



pub async fn send_debug_error_log(
    message: String
){
    dotenv::dotenv().ok();
    let is_dev = if cfg!(debug_assertions) {
        "dev"
    } else {
        "prod"
    };
  let token = std::env::var("DISCORD_WEBHOOK_TOKEN").unwrap_or_else(|_| "default_token".to_string());
  reqwest::Client::new()
    .post(format!("https://discord.com/api/webhooks/1375871801409798186/{}", token))
    .header("Content-Type", "application/json")
    .body(
        serde_json::to_string(&serde_json::json!({
            "embeds": [
                {
                    "title": "Rin Agent Debug Error",
                    "description": format!("{}> {}",is_dev,message),
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


pub async fn send_additional_log(
    message: String
){
    dotenv::dotenv().ok();
    let token = std::env::var("DISCORD_WEBHOOK_TOKEN").unwrap_or_else(|_| "default_token".to_string());
    let is_dev = if cfg!(debug_assertions) {
        "dev"
    } else {
        "prod"
    };
    let req = reqwest::Client::new()
        .post(format!("https://discord.com/api/webhooks/1375871801409798186/{}", token))
        .header("Content-Type", "application/json")
        .body(
            serde_json::to_string(&serde_json::json!({
                "embeds": [
                    {
                        "title": "Rin Agent Additional Log",
                        "description": format!("{}> {}",is_dev,message),
                        "color": 16776960, // Yellow color
                        "footer": {
                            "text": "Rin Agent Additional Log"
                        }
                    }
                ],
            }))
            .unwrap()
        )
        .send()
        .await;
    LOGGER.log(
        LogLevel::Debug,
        &format!("Sent additional log: {:?}", req)
    );
}