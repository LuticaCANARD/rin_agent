use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::discord::utils::GuildCommandResponse;


pub async fn run(_ctx: &Context, _options: &CommandInteraction) -> Result<GuildCommandResponse, serenity::Error> {

    let response = "Pong!";
    let message = CreateInteractionResponseMessage::new().content(response);

    // Send a response to the interaction
    _options.create_response(_ctx,CreateInteractionResponse::Message(message)).await?;
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