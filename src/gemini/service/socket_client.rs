use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use futures_util::{stream::{SplitSink, SplitStream}, SinkExt, StreamExt};
use std::time::Duration;

use crate::libs::logger::{LogLevel, LOGGER}; // 재연결 시 딜레이 등에 사용

// 클라이언트 상태를 나타내는 Enum (선택 사항)
#[derive(Debug, Clone, PartialEq)]
pub enum ClientState {
    Initial,
    Connecting,
    Connected,
    Disconnected,
    Reconnecting,
}

pub struct GeminiSocketClient {
    pub id: i64,
    tx: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>, // Option으로 변경하여 재연결 시 관리 용이
    url: String, // 재연결 시 사용
    state: ClientState,
}

impl GeminiSocketClient {
    pub async fn new(
        id: i64,
        url: String,
    ) -> Self {
        // 초기에는 tx가 None일 수 있으므로, 실제 연결은 별도 메서드 (connect)에서 처리
        GeminiSocketClient {
            id,
            tx: None,
            url,
            state: ClientState::Initial, // 초기 상태
        }
    }

    // 실제 연결 시도
    pub async fn connect(&mut self) -> Result<
    SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    String> {
        LOGGER.log(LogLevel::Info,
            format!("Attempting to connect to WebSocket: {}", self.url).as_str()
        );
        match connect_async(&self.url).await {
            Ok((socket_stream, response)) => {
                LOGGER.log(LogLevel::Info,
                    format!("Connected to WebSocket: {}. Response: {:?}", self.url, response).as_str()
                );
                let (tx, rx) = socket_stream.split();
                self.tx = Some(tx);
                Ok(rx)
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
    pub fn start_managing_connection(
        mut self, // 소유권을 가져옴
        message_handler_tx: mpsc::Sender<Result<Message, tokio_tungstenite::tungstenite::Error>>
    ) {
        tokio::spawn(async move {
            loop {
                match self.connect().await {
                    Ok(mut rx) => {
                        self.state = ClientState::Connected; // 상태 업데이트
                        println!("WebSocket listener started for id: {}", self.id);
                        let handler_tx = message_handler_tx.clone();

                        // 메시지 수신 루프
                        while let Some(message_result) = rx.next().await {
                            let is_close_msg = matches!(message_result, Ok(Message::Close(_)));
                            if handler_tx.send(message_result).await.is_err() {
                                eprintln!("[{}] Handler channel closed. Stopping listener.", self.id);
                                // tx를 통해 close 프레임 전송 시도
                                if let Some(tx) = self.tx.as_mut() {
                                    if let Err(e) = tx.send(Message::Close(None)).await {
                                        eprintln!("[{}] Error sending close frame: {}", self.id, e);
                                    }
                                }
                                self.tx = None; // tx 사용 불가 표시
                                self.state = ClientState::Disconnected;
                                return; // 전체 태스크 종료 (재연결 로직으로 가지 않음)
                            }
                            if is_close_msg {
                                println!("[{}] Received close frame. Listener will stop.", self.id);
                                self.tx = None; // tx 사용 불가 표시
                                // self.state = ClientState::Disconnected;
                                break; // 내부 루프 종료 -> 재연결 시도
                            }
                        }
                        // self.state = ClientState::Disconnected; // 연결 종료 상태
                        println!("[{}] Listener stopped. Attempting to clean up sink.", self.id);
                        if let Some(mut tx) = self.tx.take() { // tx 소유권 가져와서 close
                            if let Err(e) = tx.close().await {
                                eprintln!("[{}] Error closing WebSocket sink: {}", self.id, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[{}] Connection failed: {}. Will retry after a delay.", self.id, e);
                        // self.state = ClientState::Reconnecting; // 재연결 중 상태
                    }
                }
                // self.state = ClientState::Reconnecting;
                println!("[{}] Attempting to reconnect after 5 seconds...", self.id);
                tokio::time::sleep(Duration::from_secs(5)).await; // 재연결 딜레이
            }
        });
    }


    async fn send_message(&mut self, message: String) -> Result<(), String> {
        if let Some(tx) = self.tx.as_mut() {
            tx.send(Message::Text(message.into()))
                .await
                .map_err(|e| format!("Failed to send text message: {}", e))
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
        println!("[{}] Shutdown requested.", self.id);
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
    


}