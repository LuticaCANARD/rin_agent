#[derive(Debug, Clone)]
pub struct GeminiResponse {
    pub discord_msg: String,
    pub sub_items: Vec<String>,
    pub finish_reason: String,
    pub avg_logprobs: f64,
    pub commands : Option<Vec<String>>,
}
#[derive(Debug, Clone)]
pub struct GeminiImageInputType {
    pub base64_image: Option<String>,
    pub file_url: Option<String>,
    // e.g. "image/png", "image/jpeg"
    pub mime_type: String,
}
#[derive(Debug, Clone)]
pub struct GeminiChatChunk {
    pub query: String,
    pub image: Option<GeminiImageInputType>,
    pub is_bot: bool,
    pub timestamp: String,
    pub user_id: Option<String>, 
}

