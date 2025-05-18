use std::collections::BTreeMap;

use crate::libs::logger::LOGGER;

use super::socket_client::GeminiSocketClient;

struct GeminiSocketManager {
    socket_map: BTreeMap<i64, GeminiSocketClient>,
}

impl GeminiSocketManager {
    pub fn new() -> Self {
        GeminiSocketManager {
            socket_map: BTreeMap::new(),
        }
    }

    pub async fn get_or_create_socket_client(
        &mut self,
        id: i64,
        url: String,
    ) -> &mut GeminiSocketClient {
        if !self.socket_map.contains_key(&id) {
            let url_clone = url.clone();
            let mut client = GeminiSocketClient::new(id, url);
            match client.connect().await {
                Ok(_) => {
                    LOGGER.log(crate::libs::logger::LogLevel::Debug, format!("Connected to WebSocket: {}", url_clone).as_str());
                }
                Err(e) => {
                    LOGGER.log(crate::libs::logger::LogLevel::Error, format!("Failed to connect to WebSocket {}: {}", url_clone, e).as_str());
                }
            }
            self.socket_map.insert(id, client);
        }
        self.socket_map.get_mut(&id).unwrap()
    }

}