use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::discord::utils::GuildCommandResponse;


pub async fn run(_ctx: &Context, _options: &CommandInteraction) -> Result<GuildCommandResponse, serenity::Error> {
    Ok(
        GuildCommandResponse {
            content: CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                .content("Lutica Repository:
                https://github.com/LuticaCANARD/rin_agent")),
            do_not_send: false,
        }

    )
}

pub fn register() -> CreateCommand {
    CreateCommand::new("lutica_repo").description("Show repo information")
}