use sea_orm::{ColIdx, EntityTrait};
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::gemini::gemini_client::GeminiClientTrait;
use crate::libs::logger::{LOGGER, LogLevel};
use crate::gemini::GEMINI_CLIENT;
use crate::model::db::driver::{connect_to_db, DB_CONNECTION_POOL};
use crate::utils::split_text::split_text_by_length_and_markdown;

use serenity::all::Embed;
use entity::tb_ai_context::ActiveModel as AiContextModel;
use entity::tb_ai_context::Entity as AiContextEntity;


fn generate_message_block(box_msg: String, title:String , description:String,need_emded:bool ) -> CreateMessage{
    let msg = CreateMessage::new()                
    .content( box_msg);
    if need_emded {
        msg.add_embed(
            CreateEmbed::new()
                .title(title)
                .description(description)
                .color(0x00FF00) // Green color
                .footer(CreateEmbedFooter::new("Gemini API"))
        )
    } else {
        msg
    }
}

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
            let str_query = s.to_string();
            let response = GEMINI_CLIENT.lock().await.send_query_to_gemini(vec![
                str_query.clone()
            ]).await.unwrap();
            let response_avg_logprobs = response.avg_logprobs;
            let response = response.content;

            LOGGER.log(LogLevel::Debug, &format!("Gemini 응답: {}", response));
            let mut send_msgs:Vec<Message> = vec![];
            if response.len() > 1950 {
                let mut called_user = false;
                let chuncks = split_text_by_length_and_markdown(&response, 1950);
                let user_mention = format!("<@{}>\n", _options.user.id).to_string();
                for chunk in 0..chuncks.len() {
                    let mut msg_last = String::new();
                    if called_user == false {
                        let strs = &chuncks.get(chunk).unwrap().clone();

                        msg_last = user_mention.clone() + strs;
                        called_user = true;
                    } else {
                        msg_last = chuncks.get(chunk).unwrap().clone();
                    }

                    let response_msg = generate_message_block(
                        msg_last, 
                        "Gemini 응답".to_string(),
                        response_avg_logprobs.to_string(),
                        chunk == chuncks.len() - 1 // 마지막 chunk에만 embed 추가
                    );
                    send_msgs.push(_options.channel_id.send_message(_ctx, response_msg).await.unwrap());
                }
            } else {
                let user_mention = format!("<@{}>\n", _options.user.id).to_string();
                let response_msg = generate_message_block(
                    user_mention + &response,
                    "Gemini 응답".to_string(),
                    response_avg_logprobs.to_string(),true);
                send_msgs.push(_options.channel_id.send_message(_ctx, response_msg).await.unwrap());
            }
            let inserted = AiContextModel {
                user_id: sea_orm::Set(_options.user.id.get() as i64),
                context: sea_orm::Set(str_query),
                ..Default::default()
            };
            let db = DB_CONNECTION_POOL.get().unwrap().clone();


            let response_record = AiContextModel {
                user_id: sea_orm::Set(_options.user.id.get() as i64),
                context: sea_orm::Set(response.clone()),
                ..Default::default()
            };
            let a = AiContextEntity::insert(inserted)
            .add(response_record)
            .exec(&db)
            .await
            .unwrap();

            

            LOGGER.log(LogLevel::Debug, &format!("DB Inserted: {:?}", a));

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