use sea_orm::prelude::Expr;
use sea_orm::sea_query::{Alias, ExprTrait};
use sea_orm::{ColIdx, Condition, EntityTrait, JoinType, QueryFilter, QuerySelect};
use serenity::all::ActivityData;
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::gemini::gemini_client::GeminiClientTrait;
use crate::libs::logger::{LOGGER, LogLevel};
use crate::gemini::GEMINI_CLIENT;
use crate::model::db::driver::{connect_to_db, DB_CONNECTION_POOL};
use crate::utils::split_text::split_text_by_length_and_markdown;

use entity::tb_ai_context::{self, ActiveModel as AiContextModel};
use entity::tb_ai_context::Entity as AiContextEntity;
use entity::tb_discord_ai_context::{ActiveModel as AiContextDiscordModel, Entity as AiContextDiscordEntity};
use entity::tb_discord_message_to_at_context::{self, ActiveModel as AiContextDiscordMessageModel, Column as TbDiscordMessageToAtContext, Entity as AiContextDiscordMessageEntity};

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

async fn send_split_msg(_ctx: &Context,channel_context:ChannelId,origin_msg:String,ref_msg:Option<Message>)->Vec<Message> {
    let mut send_msgs:Vec<Message> = vec![];
    let mut called_user = false;
    let chuncks = split_text_by_length_and_markdown(&origin_msg, 1950);
    for chunk in 0..chuncks.len() {
        let mut msg_last = String::new();
        if called_user == false {
            let strs = &chuncks.get(chunk).unwrap().clone();

            msg_last = strs.to_string();
            called_user = true;
        } else {
            msg_last = chuncks.get(chunk).unwrap().clone();
        }
        let mut response_msg = CreateMessage::new()
            .content(msg_last);
        
        LOGGER.log(LogLevel::Debug, &format!("chunk: {},leng:{}", chunk, chuncks.len()));
        if chunk == chuncks.len() - 1 {
            let strs = &chuncks.get(chunk).unwrap().clone();
            response_msg = generate_message_block(strs.to_string(),
            "Gemini API".to_string(), "Gemini API".to_string(),
            chunk == chuncks.len() - 1);
        }
        if chunk == 0 {
            LOGGER.log(LogLevel::Debug, &format!("chunk: {},leng:{},there is ref {}", chunk, chuncks.len(),ref_msg.is_some()));
            if let Some(ref ref_msg) = ref_msg {
                response_msg = response_msg.reference_message(ref_msg);
            }
        } 
        send_msgs.push(channel_context.send_message(_ctx,response_msg).await.unwrap());
    }
    
    send_msgs
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
            let response = response.content;

            LOGGER.log(LogLevel::Debug, &format!("Gemini 응답: {}", response));
            let send_msgs:Vec<Message> = send_split_msg(_ctx, _options.channel_id, response.clone(),None).await;
            let inserted_user_question = AiContextModel {
                user_id: sea_orm::Set(_options.user.id.get() as i64),
                context: sea_orm::Set(str_query),
                guild_id: sea_orm::Set(_options.guild_id.unwrap().get() as i64),
                channel_id: sea_orm::Set(_options.channel_id.get() as i64),
                ..Default::default()
            };

            let db = DB_CONNECTION_POOL.get().unwrap().clone();

            let response_record = AiContextModel {
                user_id: sea_orm::Set(_options.user.id.get() as i64),
                context: sea_orm::Set(response.clone()),
                guild_id: sea_orm::Set(_options.guild_id.unwrap().get() as i64),
                channel_id: sea_orm::Set(_options.channel_id.get() as i64),

                ..Default::default()
            };

            
            let insert_user_and_bot_talk: Vec<tb_ai_context::Model> = AiContextEntity::insert_many(
                vec![
                    inserted_user_question,
                    
                ]).add(response_record)
                .exec_with_returning_many(&db)
                .await
                .unwrap();
            LOGGER.log(LogLevel::Debug, &format!("DB Inserted: {:?}", insert_user_and_bot_talk));

            let make_context = AiContextDiscordEntity::insert(AiContextDiscordModel {
                guild_id: sea_orm::Set(_options.guild_id.unwrap().get() as i64),
                root_msg: sea_orm::Set(send_msgs[0].id.get() as i64),
                parent_context: sea_orm::Set(None),
                ..Default::default()
            })
            .exec_with_returning(&db)
            .await
            .unwrap();
            LOGGER.log(LogLevel::Debug, &format!("DB Inserted with Context: {:?}", make_context));
            

            let ai_context_discord_messages = send_msgs.iter().map(|msg| {
                AiContextDiscordMessageModel {
                    discord_message: sea_orm::Set(msg.id.get() as i64),
                    ai_context_id: sea_orm::Set(make_context.id as i64),
                    ai_msg_id: sea_orm::Set(insert_user_and_bot_talk[1].id as i64),
                    ..Default::default()
                }
            }).collect::<Vec<_>>();
            let insert_user_msg_to_context =  AiContextDiscordMessageModel {
                discord_message: sea_orm::Set(_options.id.get() as i64),
                ai_context_id: sea_orm::Set(make_context.id as i64),
                ai_msg_id: sea_orm::Set(insert_user_and_bot_talk[0].id as i64),
                ..Default::default()
            };
            let _context_inserted = AiContextDiscordMessageEntity::insert_many(ai_context_discord_messages)
            .add(insert_user_msg_to_context)
                .exec(&db)
                .await
                .unwrap();

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

pub async fn continue_query(_ctx: &Context,calling_msg:&Message) {
    

    _ctx.set_activity(Some(ActivityData::playing("Gemini Query")));
    
    let db = DB_CONNECTION_POOL.get().unwrap().clone();
    LOGGER.log(LogLevel::Debug, &format!("DB Connection: {:?}", db));
    let msg_ref_id = calling_msg.referenced_message.clone().unwrap().id.get() as i64;
    LOGGER.log(LogLevel::Debug, &format!("msg_ref_id: {:?}", msg_ref_id));
    let ai_context = AiContextDiscordMessageEntity::find().filter(
        Condition::all()
            .add(
                Expr::col(TbDiscordMessageToAtContext::DiscordMessage)
                .eq(msg_ref_id)
            )
    )
    .distinct_on(
        vec![TbDiscordMessageToAtContext::AiContextId]
    )
    .all(&db)
    .await
    .unwrap();

    if ai_context.len() == 0 {
        LOGGER.log(LogLevel::Error, "AI Context가 없습니다.");
        return;
    }
    LOGGER.log(LogLevel::Debug, &format!("AI Context: {:?}", ai_context));
    let ai_contexts = ai_context.iter().map(|x| x.ai_context_id as i64).collect::<Vec<i64>>();
    let mut before_messages = tb_ai_context::Entity::find()
    .join_as(
        JoinType::InnerJoin,
        Into::<sea_orm::RelationDef>::into(tb_discord_message_to_at_context::Entity::belongs_to(tb_ai_context::Entity)
            .from(tb_discord_message_to_at_context::Column::AiMsgId)
            .to(tb_ai_context::Column::Id)) // 혹은 .into() 그대로 사용하셔도 됩니다.
            .rev(), // <--- 이 부분을 추가합니다.
        Alias::new("rel_discord_ctx"), // 여기 alias 이름
    )
    .filter(
        Expr::col((
            Alias::new("rel_discord_ctx"), // 이 별칭은 join_as에서 정의한 별칭을 참조
            tb_discord_message_to_at_context::Column::AiContextId,
        ))
        .is_in(
            ai_context.iter().map(|x| x.ai_context_id as i64).collect::<Vec<i64>>()
        ),
    )
    .all(&db)
    .await
    .unwrap(); // 이 unwrap은 오류 발생 시 패닉을 유발하므로, 실제 코드에서는 .map_err 등으로 오류 처리를 해주시는 것이 좋습니다.
    LOGGER.log(LogLevel::Debug, &format!("before_messages: {:?}", before_messages));
    let mut before_messages:Vec<String> = before_messages.iter().map(|x| x.context.clone()).collect();
    
    LOGGER.log(LogLevel::Debug, &format!("before_messages: {:?}", before_messages));
    let _push_query = before_messages.push(calling_msg.content.clone());

    let ai_response = GEMINI_CLIENT.lock().await
    .send_query_to_gemini(before_messages).await
    .unwrap();
    
    // TODO : 분기처리.

    let send_msgs:Vec<Message> = send_split_msg(_ctx, calling_msg.channel_id, ai_response.content.clone(), Some(calling_msg.clone())).await;
    _ctx.set_activity(None);
    let inserted = AiContextModel {
        user_id: sea_orm::Set(calling_msg.author.id.get() as i64),
        context: sea_orm::Set(calling_msg.content.clone()),
        guild_id: sea_orm::Set(calling_msg.guild_id.unwrap().get() as i64),
        channel_id: sea_orm::Set(calling_msg.channel_id.get() as i64),
        ..Default::default()
    };
    let response_record = AiContextModel {
        user_id: sea_orm::Set(calling_msg.author.id.get() as i64),
        context: sea_orm::Set(ai_response.content.clone()),
        guild_id: sea_orm::Set(calling_msg.guild_id.unwrap().get() as i64),
        channel_id: sea_orm::Set(calling_msg.channel_id.get() as i64),
        ..Default::default()
    };
    let _insert_context_desc = AiContextEntity::insert(inserted)
        .add(response_record)
        .exec_with_returning_many(&db)
        .await
        .unwrap();
    LOGGER.log(LogLevel::Debug, &format!("DB Inserted: {:?}", _insert_context_desc));

    let _insert_context = AiContextDiscordEntity::insert(AiContextDiscordModel {
        guild_id: sea_orm::Set(calling_msg.guild_id.unwrap().get() as i64),
        root_msg: sea_orm::Set(send_msgs[0].id.get() as i64),
        parent_context: sea_orm::Set(Some(ai_context[0].ai_context_id as i64)),
        ..Default::default()
    }).exec_with_returning(&db)
    .await
    .unwrap();
    let continue_context = ai_contexts.last().unwrap().clone();
    let ai_context_discord_messages = send_msgs.iter().map(|msg| {
        AiContextDiscordMessageModel {
            discord_message: sea_orm::Set(msg.id.get() as i64),
            ai_context_id: sea_orm::Set( continue_context ),
            ai_msg_id: sea_orm::Set(_insert_context_desc[1].id),
            ..Default::default()
        }
    }).collect::<Vec<_>>();
    let insert_user_msg_to_context =  AiContextDiscordMessageModel {
        discord_message: sea_orm::Set(calling_msg.id.get() as i64),
        ai_context_id: sea_orm::Set(continue_context),
        ai_msg_id: sea_orm::Set(_insert_context_desc[0].id),
        ..Default::default()
    };
    let _context_inserted = AiContextDiscordMessageEntity::insert_many(ai_context_discord_messages)
    .add(insert_user_msg_to_context)
        .exec(&db)
        .await
        .unwrap();
    

}