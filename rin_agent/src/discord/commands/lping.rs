use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::discord::utils::GuildCommandResponse;


pub async fn run(_ctx: &Context, _options: &CommandInteraction) -> Result<GuildCommandResponse, serenity::Error> {
    Ok(
        GuildCommandResponse {
            content: CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                .content("Pong! - L")),
            do_not_send: false,
        }
    )
}

pub fn register() -> CreateCommand {
    CreateCommand::new("lping").description("A ping command")
}