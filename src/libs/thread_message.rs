pub struct DiscordToGeminiMessage<T>{
    pub message: T,
    pub sender: String,
    pub channel_id: String,
    pub message_id: String,
    pub guild_id: String,

}
pub struct GeminiThreadResponse<T>{
    pub message: T,
}

