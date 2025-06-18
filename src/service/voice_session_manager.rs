use std::{collections::BTreeMap, sync::{Arc, LazyLock}};
use tokio::sync::{Mutex, MutexGuard};

use crate::libs::voice_session::VoiceSession;

pub static VOICE_SESSION_MANAGER: LazyLock<Arc<Mutex<BTreeMap<i64, VoiceSession>>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(BTreeMap::new()))
});

pub async fn add_session(guild_id: i64, session: VoiceSession) {
    let mut map = VOICE_SESSION_MANAGER.lock().await;
    map.insert(guild_id, session);
    drop(map);
}