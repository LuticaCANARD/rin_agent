use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;


pub async fn run(_ctx: &Context, _options: &CommandInteraction) -> Result<String, serenity::Error> {
    Ok("Hey, I'm alive!".to_string())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("A ping command")
}