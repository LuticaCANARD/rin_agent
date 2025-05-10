use base64::Engine;
use sea_orm::prelude::Expr;
use sea_orm::sea_query::Alias;
use sea_orm::{Condition, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect};
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use sqlx::types::chrono;
use crate::discord::constant::DISCORD_DB_ERROR;
use crate::gemini::gemini_client::{GeminiChatChunk, GeminiClientTrait, GeminiImageInputType, GeminiResponse};
use crate::libs::logger::{LOGGER, LogLevel};
use crate::gemini::GEMINI_CLIENT;
use crate::model::db::driver::DB_CONNECTION_POOL;
use crate::utils::split_text::split_text_by_length_and_markdown;
use crate::setting::gemini_setting::{get_begin_query, GEMINI_MODEL_FLASH, GEMINI_MODEL_PRO};

use entity::tb_ai_context::{self, ActiveModel as AiContextModel};
use entity::tb_ai_context::Entity as AiContextEntity;
use entity::tb_discord_ai_context::{self, ActiveModel as AiContextDiscordModel, Entity as AiContextDiscordEntity};
use entity::tb_discord_message_to_at_context::{self, ActiveModel as AiContextDiscordMessageModel, Column as TbDiscordMessageToAtContext, Entity as AiContextDiscordMessageEntity};

fn generate_message_block(box_msg: String, title:String , description:String,footer:String,need_emded:bool) -> CreateMessage{
    let msg = CreateMessage::new()                
    .content( box_msg);
    if need_emded {
        msg.add_embed(
            CreateEmbed::new()
                .title(title)
                .description(description)
                .color(0x00FF00) // Green color
                .footer(CreateEmbedFooter::new(footer))
        )
    } else {
        msg
    }
}

fn user_mention(user: &User) -> String {
    format!("<@{}>\n", user.id.get())
}

fn context_process(origin:&tb_ai_context::Model) -> GeminiChatChunk {
    GeminiChatChunk{
        query: origin.context.clone(),
        is_bot: origin.by_bot,
        timestamp: origin.created_at.to_utc().to_string(),
        image: None,
        user_id: Some(origin.user_id.to_string()),
    }
}

async fn send_split_msg(_ctx: &Context,channel_context:ChannelId,origin_user:User,message_context:GeminiResponse,ref_msg:Option<Message>,need_mention_first:bool,use_pro:bool)->Vec<Message> {
    let origin_msg = message_context.discord_msg.clone();
    let mut send_msgs:Vec<Message> = vec![];
    let chuncks: Vec<String> = split_text_by_length_and_markdown(&origin_msg, 1950);
    for chunk in 0..chuncks.len() {
        let msg_last = if need_mention_first == true && chunk == 0 {
            user_mention(&origin_user) + &chuncks.get(chunk).unwrap().clone()
        } else {
            chuncks.get(chunk).unwrap().clone()
        };
        let mut response_msg: CreateMessage = CreateMessage::new()
        .content(msg_last);
        if chunk == chuncks.len() - 1 {
            let mut sub_items = String::new();
            if message_context.sub_items.len() >0 {
                
                for sub_item in message_context.sub_items.iter() {
                    sub_items.push_str(&format!("{}\n", sub_item));
                }
            }
            let strs = if need_mention_first == true && chunk == 0 {  
                user_mention(&origin_user) + &chuncks.get(chunk).unwrap().clone()
            } else {
                chuncks.get(chunk).unwrap().clone()
            };
            response_msg = generate_message_block(strs,
            "Gemini API".to_string(), sub_items,
            if use_pro {GEMINI_MODEL_PRO.to_string()} else {GEMINI_MODEL_FLASH.to_string()},chunk == chuncks.len() - 1);
        }
        if chunk == 0 {
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
    let use_pro = options.iter().find(|o| o.name == "use_pro");
    let use_pro = if use_pro.is_some() {
        let unwarped = use_pro.unwrap().value.clone();
        match unwarped {
            ResolvedValue::Boolean(s) => s,
            _ => false,
        }
    } else {
        false
    };
    if query.is_none() {
        _options.create_response(_ctx,CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("질문을 입력하세요"))).await?;
        return Ok("질문을 입력하세요".to_string());
    }
    let query = query.unwrap().value.clone();
    match query {
        
        ResolvedValue::String(ref s) => {
            let chatting_channel = _ctx.http.get_channel(_options.channel_id).await.unwrap();
            let typing: serenity::all::Typing = chatting_channel.guild().unwrap().start_typing(&_ctx.http);

            // Do something with the string value
            LOGGER.log(LogLevel::Debug, &format!("질문: {}", s));
            let discord_response_message = CreateInteractionResponseMessage::new().content(&format!("질문 : {}", s));

            // Send a response to the interaction
            _options.create_response(_ctx,CreateInteractionResponse::Message(discord_response_message)).await?;
            let str_query = s.to_string();
            let response = GEMINI_CLIENT.lock().await.send_query_to_gemini(vec![
                get_begin_query(_options.locale.clone(), _options.user.clone()),
                GeminiChatChunk{
                    query: str_query.clone(),
                    is_bot: false,
                    image: None,
                    timestamp: chrono::Utc::now().to_string(),
                    user_id: Some(_options.user.id.get().to_string()),
                }
            ],use_pro).await;
            if response.is_err() {
                LOGGER.log(LogLevel::Error, &format!("Gemini API Error: {:?}", response));
                return Err(SerenityError::Other("Gemini API Error"));
            }
            let response = response.unwrap();
            let send_msgs:Vec<Message> = send_split_msg(_ctx, 
                _options.channel_id, 
                _options.user.clone(),
                response.clone(),
                None,true,
                use_pro
            ).await;
            typing.stop();
            let inserted_user_question = AiContextModel {
                user_id: sea_orm::Set(_options.user.id.get() as i64),
                context: sea_orm::Set(str_query),
                guild_id: sea_orm::Set(_options.guild_id.unwrap().get() as i64),
                channel_id: sea_orm::Set(_options.channel_id.get() as i64),
                by_bot: sea_orm::Set(false),
                ..Default::default()
            };

            let db = DB_CONNECTION_POOL.get();
            if db.is_none() {
                LOGGER.log(LogLevel::Error, "DB Connection Error");
                return Err(serenity::Error::Other(DISCORD_DB_ERROR));
            }
            let db = db.unwrap().clone();
            let response_record = AiContextModel {
                user_id: sea_orm::Set(_options.user.id.get() as i64),
                context: sea_orm::Set(response.discord_msg),
                guild_id: sea_orm::Set(_options.guild_id.unwrap().get() as i64),
                channel_id: sea_orm::Set(_options.channel_id.get() as i64),
                by_bot: sea_orm::Set(true),
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
                using_pro_model: sea_orm::Set(use_pro),
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
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            "use_pro",
            "Gemini Pro 모델을 사용할지 선택하세요",
        )
        .required(false)
        )
}

pub async fn continue_query(_ctx: &Context,calling_msg:&Message,user:&User) {
    let channel_lock = _ctx.http.get_channel(calling_msg.channel_id).await.unwrap();
    let typing = channel_lock.guild().unwrap().start_typing(&_ctx.http);
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
        typing.stop();
        return;
    }
    LOGGER.log(LogLevel::Debug, &format!("AI Context: {:?}", ai_context));
    let ai_contexts = ai_context.iter().map(|x| x.ai_context_id as i64).collect::<Vec<i64>>();
    let before_messages = tb_ai_context::Entity::find()
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
    .order_by(tb_ai_context::Column::Id, sea_orm::Order::Asc)
    .all(&db)
    .await
    .unwrap();

    LOGGER.log(LogLevel::Debug, &format!("before_messages: {:?}", before_messages));

    let context_using_pro = AiContextDiscordEntity::find()
    .filter(
        Condition::all()
            .add(
                Expr::col(tb_discord_ai_context::Column::Id)
                .is_in(ai_contexts.clone())
            )
    ).one(&db)
    .await
    .unwrap();
    let context_using_pro = if context_using_pro.is_some() {
        context_using_pro.unwrap().using_pro_model
    } else {
        false
    };


    let mut before_messages:Vec<GeminiChatChunk> = before_messages.iter().map(context_process).collect();

    let user_locale = user.locale.clone();

    if let Some(user_locale_open) = user_locale {
        let begin_query = get_begin_query(user_locale_open, user.clone());
        before_messages.insert(0, begin_query);
    } else {
        let begin_query = get_begin_query("ko".to_string(), user.clone());
        before_messages.insert(0, begin_query);

    }
    LOGGER.log(LogLevel::Debug, &format!("before_messages: {:?}", before_messages));
    let attachment_user_msg = calling_msg.attachments.clone();
    let mut image = None;
    if attachment_user_msg.len() > 0 {
        let image_origin = attachment_user_msg.get(0).unwrap();
        //fetching image
        LOGGER.log(LogLevel::Debug, &format!("Image fetch start: {:?}", image_origin.url));
        let data = reqwest::get(image_origin.url.clone())
        .await
        .unwrap();

        
        match data.error_for_status(){
            Ok(res) => {
                LOGGER.log(LogLevel::Debug, &format!("Image fetch success: {:?}", image_origin.url));
                let image_data = res.bytes().await.unwrap();
                let engine = base64::engine::general_purpose::STANDARD;
                let image_base64 = engine.encode(image_data);
                let image_mime = image_origin.content_type.clone().unwrap_or("image/png".to_string());
                image = Some(GeminiImageInputType {
                    base64_image: image_base64,
                    mime_type: image_mime
                });
            },
            _ => {
                LOGGER.log(LogLevel::Error, &format!("Image fetch failed: {:?}", image_origin.url));
                image = None;
            }
        }

        
    }
    let user_msg_current = GeminiChatChunk{
        query: calling_msg.content.clone(),
        is_bot: false,
        user_id: Some(calling_msg.author.id.get().to_string()),
        timestamp: calling_msg.timestamp.to_string(),
        image
    };
    let _push_query = before_messages.push(user_msg_current);

    let ai_response = GEMINI_CLIENT.lock().await
    .send_query_to_gemini(before_messages,context_using_pro).await;
    if ai_response.is_err() {
        typing.stop();
        return;
    }
    let ai_response = ai_response.unwrap();

    let send_msgs:Vec<Message> = send_split_msg(_ctx, 
        calling_msg.channel_id, 
        calling_msg.author.clone(),
        ai_response.clone(), 
        Some(calling_msg.clone()),
        false,
        context_using_pro
    ).await;
    typing.stop();
    let inserted = AiContextModel {
        user_id: sea_orm::Set(calling_msg.author.id.get() as i64),
        context: sea_orm::Set(calling_msg.content.clone()),
        guild_id: sea_orm::Set(calling_msg.guild_id.unwrap().get() as i64),
        channel_id: sea_orm::Set(calling_msg.channel_id.get() as i64),
        by_bot: sea_orm::Set(false),
        ..Default::default()
    };
    let response_record = AiContextModel {
        user_id: sea_orm::Set(calling_msg.author.id.get() as i64),
        context: sea_orm::Set(ai_response.discord_msg.clone()),
        guild_id: sea_orm::Set(calling_msg.guild_id.unwrap().get() as i64),
        channel_id: sea_orm::Set(calling_msg.channel_id.get() as i64),
        by_bot: sea_orm::Set(true),
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
        using_pro_model: sea_orm::Set(context_using_pro),
        ..Default::default()
    }).exec_with_returning(&db)
    .await
    .unwrap();
    let continue_context = ai_contexts.last().unwrap().clone();
    let ai_context_discord_messages = send_msgs.iter().map(|msg| {
        AiContextDiscordMessageModel {
            discord_message: sea_orm::Set(msg.id.get() as i64),
            ai_context_id: sea_orm::Set(continue_context),
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
