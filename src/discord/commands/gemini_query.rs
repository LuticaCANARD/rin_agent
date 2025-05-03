use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::libs::logger::{LOGGER, LogLevel};


pub async fn run(_ctx: &Context, _options: &CommandInteraction) -> Result<String, serenity::Error> {
    // let query = _options.data.get("query").unwrap().as_str().unwrap();
    // let response = format!("Gemini에게 질문하기: {}", query);
    // let message = CreateInteractionResponseMessage::new().content(response);

    // // Send a response to the interaction
    // _options.create_response(_ctx, CreateInteractionResponse::Message(message)).await?;
    Ok("Gemini에게 질문하기".to_string())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("Query").description("Gemini에게 질문하기")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "query", "질문할 내용을 입력하세요")
                .min_length(1)
                .max_length(1000)
                .required(true),
    )
}