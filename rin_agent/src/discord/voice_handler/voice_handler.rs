use std::sync::Arc;

use rocket::async_trait;
use serenity::{all::{ShardRunnerMessage, VoiceGatewayManager}, prelude::TypeMapKey};
use tokio::sync::Mutex;
use futures::channel::mpsc::UnboundedSender as Sender;

pub struct VoiceHandler;
impl TypeMapKey for VoiceHandler {
    type Value = Arc<Mutex<Arc<dyn VoiceGatewayManager>>>;
}
#[async_trait]
impl VoiceGatewayManager for VoiceHandler  {
    async fn initialise(&self, _shard_count: u32, _user_id: serenity::all::UserId) {

    }
    async fn register_shard(
        &self,
        _shard_id: u32,
        _sender : Sender<ShardRunnerMessage>
    ) {

    }

    async fn deregister_shard(&self, _shard_id: u32) {

    }

    async fn server_update(
        &self,
        _guild_id: serenity::all::GuildId,
        _endpoint: &Option<String>,
        _token: &str,
    ) {
        
    }

    async fn state_update(&self, _guild_id: serenity::all::GuildId, _voice_state: &serenity::all::VoiceState) {

    }
}
