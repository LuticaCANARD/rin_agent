use sea_orm::ColIdx;
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::gemini::gemini_client::GeminiClientTrait;
use crate::libs::logger::{LOGGER, LogLevel};
use crate::gemini::GEMINI_CLIENT;
use serenity::all::Embed;

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
            let discord_response_message = CreateInteractionResponseMessage::new().content(&format!("질문 : {}", s));

            // Send a response to the interaction
            _options.create_response(_ctx,CreateInteractionResponse::Message(discord_response_message)).await?;

            let response = GEMINI_CLIENT.lock().await.send_query_to_gemini(vec![
                s.to_string()
            ]).await.unwrap();
            let response_avg_logprobs = response.avg_logprobs;
            let response = response.content;

            LOGGER.log(LogLevel::Debug, &format!("Gemini 응답: {}", response));

            let response_msg = CreateMessage::new()
                .content(response)
                .add_embed(
                    CreateEmbed::new()
                        .title("Gemini 응답")
                        .description(response_avg_logprobs.to_string())
                        .color(0x00FF00) // Green color
                        .footer(CreateEmbedFooter::new("Gemini API"))
                );

            _options.channel_id.send_message(_ctx, response_msg).await?;


            // _options.channel_id.say(_ctx, response).await?;

            return Ok("ok".to_string());

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

pub async fn continue_query(_ctx: &Context,_ping:&PingInteraction) {
    
}