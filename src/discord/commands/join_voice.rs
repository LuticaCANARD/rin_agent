use serde_json::json;
use serenity::builder::*;
use serenity::model::{prelude::*, voice};
use serenity::prelude::*;

pub async fn run(_ctx: &Context, _options: &CommandInteraction) -> Result<String, serenity::Error> {
    let guild_id = match _options.guild_id {
        Some(id) => id,
        None => return Err(serenity::Error::Other("Command must be used in a guild")),
    };

    // .await 전에 필요한 정보를 모두 추출하기 위한 블록
    let channel_id_to_join = {
        let guild = match _ctx.cache.guild(guild_id) {
            Some(g) => g,
            None => return Err(serenity::Error::Other("Guild not found in cache")),
        };

        let channel_id = guild
            .voice_states
            .get(&_options.user.id)
            .and_then(|vs| vs.channel_id);

        if let Some(id) = channel_id {
            // 채널이 음성 채널인지 확인
            if let Some(channel) = guild.channels.get(&id) {
                if channel.kind == ChannelType::Voice {
                    Some(id) // 음성 채널 ID 반환
                } else {
                    return Err(serenity::Error::Other("You are not in a voice channel."));
                }
            } else {
                None // 채널 정보를 찾을 수 없음
            }
        } else {
            return Err(serenity::Error::Other("You need to be in a voice channel to use this command."));
        }
    };

    if let Some(channel_id) = channel_id_to_join {
      let map = json!({
        "channel_id": channel_id.to_string(),
      });
      _ctx
        .http
        .edit_voice_state_me(guild_id, &map)
        .await?;

        // 음성 채널에 성공적으로 참여했음을 알리는 메시지 반환
        return Ok(format!("Joined voice channel!"))
    } else {
        // 이 경우는 이미 위에서 처리되었지만, 만약을 위해 남겨둡니다.
        Err(serenity::Error::Other("Could not find a voice channel to join."))
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("join_voice")
      .description("Join a voice channel to User")
}