#[derive(Default,Clone)]
pub struct DiscordToGeminiMessage<T> where T: Default + Clone + 'static {
    pub message: T,
    pub sender: String,
    pub channel_id: String,
    pub message_id: String,
    pub guild_id: String,

}

#[derive(Default,Clone)]
pub struct GeminiFunctionAlarm<T> where T: Default + Clone + 'static {
    pub message: T,
    pub sender: String,
    pub channel_id: String,
    pub message_id: String,
    pub guild_id: String,
}

#[derive(Default,Clone)]
pub struct GeminiThreadResponse<T>{
    pub message: T,
}

