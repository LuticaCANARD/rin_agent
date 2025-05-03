use sea_orm::ColIdx;
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::gemini::gemini_client::GeminiClientTrait;
use crate::libs::logger::{LOGGER, LogLevel};
use crate::gemini::GEMINI_CLIENT;


pub async fn run(_ctx: &Context, _options: &CommandInteraction) -> Result<String, serenity::Error> {
    let options = _options.data.options();
    let query = options.iter().find(|o| o.name == "query");
    if query.is_none() {
        _options.create_response(_ctx,CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("질문을 입력하세요"))).await?;
        return Ok("질문을 입력하세요".to_string());
    }
    let query = query.unwrap().value.clone();
    match query {
        ResolvedValue::String(ref s) => {
            // Do something with the string value
            LOGGER.log(LogLevel::Debug, &format!("질문: {}", s));
            let querys: Vec<String> = vec![
                s.to_string()
            ];
            let response = GEMINI_CLIENT.lock().await.send_query_to_gemini(querys).await.unwrap();

            LOGGER.log(LogLevel::Debug, &format!("Gemini 응답: {}", response));

            let message = CreateInteractionResponseMessage::new().content(&response);

            // Send a response to the interaction
            _options.create_response(_ctx,CreateInteractionResponse::Message(message)).await?;
            return Ok(response);

        },
        _ => {
            // Handle other types if necessary
            LOGGER.log(LogLevel::Error, "질문이 잘못되었습니다.");
            _options.create_response(_ctx,CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("질문이 잘못되었습니다."))).await?;
            return Ok("질문이 잘못되었습니다.".to_string());
        }
        
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("gemini_query")
        .description("Gemini에게 질문하기")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "query", "질문할 내용을 입력하세요")
                .required(true)
            )
}