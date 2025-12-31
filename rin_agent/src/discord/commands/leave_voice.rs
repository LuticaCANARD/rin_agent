use gemini_live_api::libs::logger::{LogLevel, LOGGER};
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::discord::utils::GuildCommandResponse;


pub async fn run(_ctx: &Context, _options: &CommandInteraction) -> Result<GuildCommandResponse, serenity::Error> {
    let guild_id = match _options.guild_id {
        Some(id) => id,
        None => return Err(
            serenity::Error::Other("Guild ID was not provided."),
        ),
    };
    let (channel_id_to_join, user_name) = {
        let guild = _ctx.cache.guild(guild_id);
        if guild.is_none() {
            LOGGER.log(LogLevel::Error, "길드를 캐시에서 찾을 수 없습니다.");
            return Err(serenity::Error::Other("길드를 캐시에서 찾을 수 없습니다."));
        }
        let guild = guild.unwrap();
        // 길드 캐시의 voice_states에서 사용자 ID로 직접 음성 채널을 찾습니다.
        // 이 방법이 모든 채널과 멤버를 순회하는 것보다 훨씬 효율적입니다.
        let channel_id = guild
            .voice_states
            .get(&_options.user.id)
            .and_then(|voice_state| voice_state.channel_id);
        if channel_id.is_none() {
            LOGGER.log(LogLevel::Error, 
                "사용자가 음성 채널에 연결되어 있지 않습니다."
            );
            drop(guild);
            let res = GuildCommandResponse{
                content: CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                    .content(
                        "사용자가 음성 채널에 연결되어 있지 않습니다."
                    )),
                do_not_send: false,

            };
            return Ok(res);
        }
        let connect_to = channel_id.unwrap();
        // user_name도 미리 복사해 둡니다.
        (connect_to, _options.user.name.clone())
    };
    let manager = songbird::get(_ctx).await;
    let manager = match manager {
        Some(m) => m,
        None => return Ok(GuildCommandResponse{
            content: CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                .content(
                    "음성 채널 관리자에 접근할 수 없습니다."
                )),
            do_not_send: false,
        }),
    };

    manager.leave(guild_id).await.map_err(|e| {
        LOGGER.log(LogLevel::Error, 
            &format!("음성 채널에서 나가는 데 실패했습니다: {}", e)
        );
        serenity::Error::Other("음성 채널에서 나가는 데 실패했습니다.")
    })?;

    Ok(
        GuildCommandResponse {
            content: CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                .content("음성 채널에서 성공적으로 나갔습니다.")),
            do_not_send: false,
        }
    )
}

pub fn register() -> CreateCommand {
    CreateCommand::new("leave_voice").description("A leave voice command")
}