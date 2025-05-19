use std::{collections::BTreeMap, fmt::Debug};

use crate::{gemini::schema::live_api_types::BidiGenerateContentSetup, libs::logger::LOGGER};

use super::socket_client::GeminiSocketClient;

pub struct GeminiSocketManager<TKey: Ord+Debug+Clone> {
    socket_map: BTreeMap<TKey, GeminiSocketClient<TKey>>,
}

impl<TKey: Ord+Debug+Clone> GeminiSocketManager<TKey> {
    pub fn new() -> Self {
        GeminiSocketManager {
            socket_map: BTreeMap::new(),
        }
    }

    pub async fn get_or_create_socket_client(
        &mut self,
        id: TKey,
        url: String,
        connect_init: BidiGenerateContentSetup,
    ) -> &mut GeminiSocketClient<TKey> {
        let id_clone = id.clone();
        if !self.socket_map.contains_key(&id) {
            let url_clone = url.clone();
            let mut client = GeminiSocketClient::new(
                id, 
                url, 
                connect_init
            );
            match client.connect().await {
                Ok(_) => {
                    LOGGER.log(crate::libs::logger::LogLevel::Debug, format!("Connected to WebSocket: {}", url_clone).as_str());
                }
                Err(e) => {
                    LOGGER.log(crate::libs::logger::LogLevel::Error, format!("Failed to connect to WebSocket {}: {}", url_clone, e).as_str());
                }
            }
            self.socket_map.insert(id_clone.clone(), client);
        }
        self.socket_map.get_mut(&id_clone).unwrap()
    }

}