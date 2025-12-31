use serenity::all::{ChannelId, CreateMessage, Message};
use tokio::sync::{mpsc, oneshot};
use std::sync::OnceLock;

/// Discord 메시지 전송 요청을 나타내는 구조체
#[derive(Debug)]
pub struct MessageSendRequest {
    pub channel_id: ChannelId,
    pub content: CreateMessage,
    pub response_sender: oneshot::Sender<Result<Message, serenity::Error>>,
}
/// Discord 메시지 전송 요청을 나타내는 구조체
#[derive(Debug)]
pub struct EditMessageRequest {
    pub channel_id: ChannelId,
    pub message_id: u64,
    pub content: CreateMessage,
    pub response_sender: oneshot::Sender<Result<Message, serenity::Error>>,
}


/// 메시지 전송 요청을 받는 채널의 수신부
pub type MessageSendReceiver = mpsc::UnboundedReceiver<MessageSendRequest>;

/// 메시지 전송 요청을 보내는 채널의 송신부
pub type MessageSendSender = mpsc::UnboundedSender<MessageSendRequest>;


pub enum DiscordMessageEnum {
    MessageSendSender(MessageSendSender),
    EditMessageSender(mpsc::UnboundedSender<EditMessageRequest>),
}

/// 전역 메시지 전송자 - 한 번 초기화되면 전체 애플리케이션에서 사용
static MESSAGE_SENDER: OnceLock<DiscordMessageEnum> = OnceLock::new();

/// 메시지 전송자를 초기화합니다.
/// 이 함수는 Discord 봇이 시작될 때 한 번만 호출되어야 합니다.
pub fn init_message_sender(sender: MessageSendSender) {
    if MESSAGE_SENDER.set(DiscordMessageEnum::MessageSendSender(sender)).is_err() {
        eprintln!("Warning: Message sender already initialized");
    }
}

/// Discord 채널에 메시지를 전송합니다.
/// 이 함수는 채널 시스템을 통해 비동기적으로 메시지를 전송하며,
/// Discord 봇의 메인 스레드와 독립적으로 동작합니다.
pub async fn send_discord_message(
    channel_id: ChannelId, 
    content: CreateMessage
) -> Result<Message, String> {
    let sender_enum = MESSAGE_SENDER.get()
        .ok_or("Message sender not initialized. Make sure Discord bot is running.")?;
    
    let sender = match sender_enum {
        DiscordMessageEnum::MessageSendSender(s) => s,
        _ => return Err("Wrong message sender type".to_string()),
    };
    
    let (response_sender, response_receiver) = oneshot::channel();
    
    let request = MessageSendRequest {
        channel_id,
        content,
        response_sender,
    };
    
    // 요청을 채널로 전송
    sender.send(request)
        .map_err(|_| "Failed to send message request to Discord bot")?;
    
    // 응답 대기
    response_receiver.await
        .map_err(|_| "Failed to receive response from Discord bot")?
        .map_err(|e| format!("Discord API error: {:?}", e))
}

pub async fn edit_discord_message(
    channel_id: ChannelId, 
    message_id: u64,
    content: CreateMessage
) -> Result<Message, String> {
    let sender_enum = MESSAGE_SENDER.get()
        .ok_or("Message sender not initialized. Make sure Discord bot is running.")?;
    
    let sender = match sender_enum {
        DiscordMessageEnum::EditMessageSender(s) => s,
        _ => return Err("Wrong message sender type for edit".to_string()),
    };
    
    let (response_sender, response_receiver) = oneshot::channel();

    let request = EditMessageRequest {
        channel_id,
        message_id,
        content,
        response_sender,
    };
    
    // 요청을 채널로 전송
    sender.send(request)
        .map_err(|_| "Failed to send edit message request to Discord bot")?;
    
    // 응답 대기
    response_receiver.await
        .map_err(|_| "Failed to receive response from Discord bot")?
        .map_err(|e| format!("Discord API error: {:?}", e))
}

/// 메시지 전송 채널을 생성합니다.
/// 반환값: (송신부, 수신부)
pub fn create_message_channel() -> (MessageSendSender, MessageSendReceiver) {
    mpsc::unbounded_channel()
}