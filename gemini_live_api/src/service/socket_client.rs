use tokio::{
    net::TcpStream,
    sync::mpsc
};
use tokio_tungstenite::{
    connect_async, 
    tungstenite::Message, 
    MaybeTlsStream, 
    WebSocketStream
};
use futures_util::{
    stream::{
        self, SplitSink, SplitStream
    }, 
    SinkExt, 
    StreamExt
};
use std::{
    fmt::Debug, 
    time::Duration
};

use crate::{
    libs::logger::{
        LogLevel, 
        LOGGER
    }, types::live_api_types::{
        BidiGenerateContentClientContent, 
        BidiGenerateContentServerMessage, 
        BidiGenerateContentSetup, GeminiLiveApiWebSocketMessage
    }
}; // 재연결 시 딜레이 등에 사용

// 클라이언트 상태를 나타내는 Enum (선택 사항)
#[derive(Debug, Clone, PartialEq)]
pub enum ClientState {
    Initial,
    Connecting,
    Connected,
    Disconnected,
    Reconnecting,
}



pub struct GeminiSocketClient<TKey: Ord + Debug+Clone> {
    pub id: TKey,
    tx: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    rx: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>, // 추가
    url: String, // 재연결 시 사용
    state: ClientState,
    connect_init:BidiGenerateContentSetup
}

impl<TKey: Ord + Debug+Clone> GeminiSocketClient<TKey> {
    pub fn new(
        id: TKey,
        url: String,
        connect_init:BidiGenerateContentSetup
    ) -> Self {
        // 초기에는 tx가 None일 수 있으므로, 실제 연결은 별도 메서드 (connect)에서 처리
        GeminiSocketClient {
            id,
            tx: None, // 송신
            rx: None, // 수신
            url,
            state: ClientState::Initial, // 초기 상태
            connect_init
        }
    }

    // 실제 연결 시도
    pub async fn connect(
        &mut self,
    ) -> Result<(),String> {
        LOGGER.log(LogLevel::Info,
            format!("Attempting to connect to WebSocket: {}", self.url).as_str()
        );
        match connect_async(&self.url).await {
            Ok((socket_stream, response)) => {
                LOGGER.log(LogLevel::Info,
                    format!("Connected to WebSocket: {}. Response: {:?}", self.url, response).as_str()
                );
                let (tx, rx) = socket_stream.split();
                let mut init_msg= GeminiLiveApiWebSocketMessage::default();
                init_msg.set_setup(self.connect_init.clone());
                let init_msg = init_msg;
                self.tx = Some(tx);
                self.send_message(
                    serde_json::to_string(&init_msg)
                        .map_err(|e| format!("Failed to serialize message: {}", e))?
                ).await
                    .map_err(|e| format!("Failed to send init: {}", e))?;
                self.state = ClientState::Connected; // 연결 상태로 변경
                self.rx = Some(rx); 
                Ok(())
            }
            Err(e) => {
                LOGGER.log(LogLevel::Error,
                    format!("Failed to connect to WebSocket {}: {}", self.url, e).as_str()
                );
                Err(format!("Failed to connect: {}", e))
            }
        }
    }


    // 리스닝 태스크와 재연결 로직 관리

    async fn send_message(&mut self, message: String) -> Result<(), String> {
        if let Some(tx) = self.tx.as_mut() {
            let message = Message::Text(message.into());
            LOGGER.log(LogLevel::Debug,
                format!("Sending message: {}", message).as_str()
            );
            tx.send(message)
                .await
                .map_err(|e| format!("Failed to send message: {}", e))
        } else {
            Err("WebSocket not connected or tx is not available.".to_string())
        }
    }

    async fn send_byte(&mut self, message: Vec<u8>) -> Result<(), String> {
        if let Some(tx) = self.tx.as_mut() {

            tx.send(Message::Binary(message.into()))
                .await
                .map_err(|e| format!("Failed to send binary message: {}", e))
        } else {
            Err("WebSocket not connected or tx is not available.".to_string())
        }
    }

    // 클라이언트 종료 (외부에서 호출)
    pub async fn shutdown(&mut self) -> Result<(), String> {
        println!("[{:?}] Shutdown requested.", self.id);
        if let Some(mut tx) = self.tx.take() { // tx의 소유권을 가져와서 close
            self.state = ClientState::Disconnected;
            tx.close()
                .await
                .map_err(|e| format!("Error during WebSocket close: {}", e))
        } else {
            self.state = ClientState::Disconnected;
            Ok(()) // 이미 연결이 없거나 tx가 없음
        }
    }
    
    pub async fn send_new_part(
        &mut self,
        part:BidiGenerateContentClientContent
    ) -> Result<(), String> {
        let str_msg = serde_json::to_string(&part)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;
        self.send_message(str_msg).await
            .map_err(|e| format!("Failed to send message: {}", e))
    }
    pub async fn listen<F>(&mut self, mut handler: F)
    where
        F: FnMut(BidiGenerateContentServerMessage) + Send,
    {
        if let Some(rx) = self.rx.as_mut() {
            while let Some(msg) = rx.next().await {
                LOGGER.log(LogLevel::Debug,
                    format!("Received message: {:?}", msg).as_str()
                );
                if let Ok(msg) = msg {
                    if msg.is_text() {
                        let text = msg.into_text().unwrap();
                        let parsed_msg: BidiGenerateContentServerMessage = serde_json::from_str(&text)
                            .expect("Failed to parse message");
                        handler(parsed_msg);
                    }
                }
            }
        }
    }
}