use std::collections::BTreeMap;

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
            let client = GeminiSocketClient::new(id, url).await;
            self.socket_map.insert(id, client);
        }
        self.socket_map.get_mut(&id).unwrap()
    }

}