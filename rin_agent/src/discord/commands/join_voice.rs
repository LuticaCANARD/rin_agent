use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use songbird::SerenityInit; // songbird 임포트

use crate::{discord::utils::GuildCommandResponse, libs::logger::{LogLevel, LOGGER}};

pub async fn run(_ctx: &Context, _options: &CommandInteraction) -> Result<GuildCommandResponse, serenity::Error> {
    let guild_id = match _options.guild_id {
        Some(id) => id,
        None => return Err(
            serenity::Error::Other(" 길드 ID가 제공되지 않았습니다."),
        ),
    };
    // .await 전에 필요한 모든 정보를 추출합니다.
    // 이 블록 안에서는 .await를 사용하지 않습니다.
    let (channel_id_to_join, user_name) = {
        let guild = _ctx.cache.guild(guild_id).unwrap();
        // 길드 캐시의 voice_states에서 사용자 ID로 직접 음성 채널을 찾습니다.
        // 이 방법이 모든 채널과 멤버를 순회하는 것보다 훨씬 효율적입니다.
        let channel_id = guild
            .voice_states
            .get(&_options.user.id)
            .and_then(|voice_state| voice_state.channel_id);
        if channel_id.is_none() {
            LOGGER.log(LogLevel::Error, "사용자가 음성 채널에 연결되어 있지 않습니다.");
            drop(guild);
            let res = GuildCommandResponse{
                content: CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                    .content("사용자가 음성 채널에 연결되어 있지 않습니다.")),
                do_not_send: false,

            };
            return Ok(res);
        }
        let connect_to = channel_id.unwrap();
        // user_name도 미리 복사해 둡니다.
        (connect_to, _options.user.name.clone())
    };

    LOGGER.log(
        LogLevel::Info,
        &format!("사용자 {}의 음성 채널 {}에 참여 시도 중", user_name, channel_id_to_join),
    );
    // songbird 관리자를 가져옵니다.
    let manager = songbird::get(_ctx).await;
    let manager = match manager {
        Some(m) => m,
        None => return Ok(GuildCommandResponse{
            content: CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                .content("음성 채널 관리자에 접근할 수 없습니다.")),
            do_not_send: false,
        }),
    };


    // `join`을 사용하여 음성 채널에 연결합니다.
    if manager.join(guild_id, channel_id_to_join).await.is_ok() {
        LOGGER.log(LogLevel::Info, &format!("음성 채널 {}에 성공적으로 참여했습니다.", channel_id_to_join));
        Ok(GuildCommandResponse{
            content: CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                .content("음성 채널에 성공적으로 참여했습니다!")),
            do_not_send: false,

        })
    } else {
        LOGGER.log(LogLevel::Error, &format!("음성 채널 {} 참여에 실패했습니다.", channel_id_to_join));
        Ok(GuildCommandResponse{
            content: CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                .content("음성 채널에 참여하는 데 실패했습니다.")),
            do_not_send: false,

        })
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("join_voice")
        .description("사용자가 있는 음성 채널에 참여합니다.")
}