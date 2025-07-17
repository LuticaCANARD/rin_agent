use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;


pub async fn run(_ctx: &Context, _options: &CommandInteraction) -> Result<String, serenity::Error> {

    let response = "Pong!";
    let message = CreateInteractionResponseMessage::new().content(response);

    // Send a response to the interaction
    _options.create_response(_ctx,CreateInteractionResponse::Message(message)).await?;
    Ok("Pong!".to_string())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("leave_voice").description("A leave voice command")
}