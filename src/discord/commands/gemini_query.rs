use core::hash;
use std::collections::{hash_map, hash_set, HashMap};

use base64::Engine;
use entity::{tb_context_to_msg_id, tb_image_attach_file};
use sea_orm::prelude::Expr;
use sea_orm::sea_query::Alias;
use sea_orm::{Condition, ConnectionTrait, EntityTrait, Insert, InsertResult, JoinType, QueryFilter, QueryOrder, QuerySelect, Statement, TransactionError, TransactionTrait, Update};
use serenity::builder::*;
use serenity::model::prelude::*;
use serenity::prelude::*;
use sqlx::types::chrono;
use crate::discord::constant::DISCORD_DB_ERROR;
use crate::gemini::gemini_client::{self, GeminiClientTrait};
use crate::gemini::types::{GeminiChatChunk, GeminiImageInputType, GeminiResponse};
use crate::gemini::utils::upload_image_to_gemini;
use crate::libs::logger::{LOGGER, LogLevel};
use crate::model::db::driver::DB_CONNECTION_POOL;
use crate::utils::split_text::split_text_by_length_and_markdown;
use crate::setting::gemini_setting::{get_begin_query, GEMINI_MODEL_FLASH, GEMINI_MODEL_PRO};

use entity::tb_ai_context::{self, ActiveModel as AiContextModel};
use entity::tb_ai_context::Entity as AiContextEntity;
use entity::tb_discord_ai_context::{self, ActiveModel as AiContextDiscordModel, Entity as AiContextDiscordEntity};
use entity::tb_discord_message_to_at_context::{self, ActiveModel as AiContextDiscordMessageModel, Column as TbDiscordMessageToAtContext, Entity as AiContextDiscordMessageEntity};
type PastQuery = (entity::tb_ai_context::Model, Option<entity::tb_image_attach_file::Model>, Option<entity::tb_context_to_msg_id::Model>);
type QueryDBVector = Vec<PastQuery>;

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

fn context_process(origin:&PastQuery) -> GeminiChatChunk {
    GeminiChatChunk{
        query: origin.0.context.clone(),
        is_bot: origin.0.by_bot,
        timestamp: origin.0.created_at.to_utc().to_string(),
        image: if origin.1.is_some() {
            let image = origin.1.clone().unwrap();
            let image = GeminiImageInputType{
                base64_image: None,
                file_url: Some(image.file_src),
                mime_type: image.mime_type.unwrap_or("image/png".to_string()),
            };
            Some(image)
        } else {
            None
        },
        user_id: Some(origin.0.user_id.to_string()),
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
            let response = 
            gemini_client::GeminiClient::new()
                .send_query_to_gemini(vec![
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
                parent_context: sea_orm::Set([].to_vec()),
                using_pro_model: sea_orm::Set(use_pro),
                ..Default::default()
            })
            .exec_with_returning(&db)
            .await
            .unwrap();
            LOGGER.log(LogLevel::Debug, &format!("DB Inserted with Context: {:?}", make_context));
            
            let inserted_msg_context_user = tb_context_to_msg_id::ActiveModel {
                ai_msg: sea_orm::Set(insert_user_and_bot_talk[0].id as i64),
                ai_context: sea_orm::Set(make_context.id as i64),
                ..Default::default()
            };
            let inserted_msg_context_bot = tb_context_to_msg_id::ActiveModel {
                ai_msg: sea_orm::Set(insert_user_and_bot_talk[1].id as i64),
                ai_context: sea_orm::Set(make_context.id as i64),
                ..Default::default()
            };
            let _insert_context = tb_context_to_msg_id::Entity::insert_many(vec![
                inserted_msg_context_user,
                inserted_msg_context_bot,
            ])
            .exec(&db)
            .await
            .unwrap();
            let ai_context_discord_messages = send_msgs.iter().map(|msg| {
                AiContextDiscordMessageModel {
                    discord_message: sea_orm::Set(msg.id.get() as i64),
                    ai_msg_id: sea_orm::Set(insert_user_and_bot_talk[1].id as i64),
                    ..Default::default()
                }
            }).collect::<Vec<_>>();
            let insert_user_msg_to_context =  AiContextDiscordMessageModel {
                discord_message: sea_orm::Set(_options.id.get() as i64),
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

    let parent_context = AiContextEntity::find()
    .join_as(
        JoinType::LeftJoin,
        Into::<sea_orm::RelationDef>::into(
            AiContextEntity::belongs_to(tb_discord_message_to_at_context::Entity)
                .from(<entity::prelude::TbAiContext as EntityTrait>::Column::Id)
                .to(TbDiscordMessageToAtContext::AiMsgId)
        ),
        Alias::new("rel_discord_ctx"),
    )
    .filter(
        Expr::col((
            Alias::new("rel_discord_ctx"),
            TbDiscordMessageToAtContext::DiscordMessage,
        ))
        .eq(msg_ref_id)
    )
    .order_by(<entity::prelude::TbAiContext as EntityTrait>::Column::Id, sea_orm::Order::Asc)
    .all(&db)
    .await
    .unwrap();

    LOGGER.log(LogLevel::Debug, &format!("parent_context: {:?}", parent_context));

    let ai_context = AiContextDiscordMessageEntity::find()
    .join_as(JoinType::InnerJoin,
        Into::<sea_orm::RelationDef>::into(
            AiContextDiscordMessageEntity::belongs_to(tb_context_to_msg_id::Entity)
                .from(TbDiscordMessageToAtContext::AiMsgId)
                .to(tb_context_to_msg_id::Column::AiMsg)
        ), Alias::new("rel_discord_ctx") )
    .filter(
        Condition::all()
            .add(
                Expr::col(TbDiscordMessageToAtContext::DiscordMessage)
                .eq(msg_ref_id)
            )
    )
    .distinct_on(vec![
        (Alias::new("rel_discord_ctx"), tb_context_to_msg_id::Column::AiContext)
    ])
    .find_also_related(tb_context_to_msg_id::Entity)
    .all(&db)
    .await
    .unwrap();

    let ai_context_parent = AiContextDiscordEntity::find()
    .filter(
        Condition::all()
            .add(
                Expr::col(tb_discord_ai_context::Column::Id).eq(ai_context.clone().last().unwrap().1.clone().unwrap().ai_context as i64)
            )
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
    let ai_contexts = ai_context_parent.iter().map(|x| x.parent_context.clone()).collect::<Vec<Vec<i64>>>();
    let mut ai_contexts = ai_contexts.iter().flat_map(|x| x.iter()).map(|x| *x).collect::<Vec<i64>>();
    ai_contexts.push(ai_context.last().unwrap().1.clone().unwrap().ai_context as i64);
    let ai_contexts = ai_contexts; 
    let before_messages:QueryDBVector = tb_ai_context::Entity::find()
    .join_as(
        JoinType::LeftJoin,
        Into::<sea_orm::RelationDef>::into(
            tb_ai_context::Entity::belongs_to(tb_image_attach_file::Entity)
                .from(tb_ai_context::Column::ImageFileId)
                .to(tb_image_attach_file::Column::ImageId)
        ),
        Alias::new("rel_image"),
    )
    .join_as(
        JoinType::InnerJoin,
        Into::<sea_orm::RelationDef>::into(
            tb_ai_context::Entity::belongs_to(tb_discord_message_to_at_context::Entity)
                .from(tb_ai_context::Column::Id)
                .to(tb_discord_message_to_at_context::Column::AiMsgId)
        ),
        Alias::new("rel_discord_ctx"),
    )
    .join_as(
        JoinType::LeftJoin,
        Into::<sea_orm::RelationDef>::into(
            tb_ai_context::Entity::belongs_to(tb_context_to_msg_id::Entity)
                .from(tb_ai_context::Column::Id)
                .to(tb_context_to_msg_id::Column::AiMsg)
        ),
        Alias::new("rel_context")
    )
    .filter(
        Expr::col((
            Alias::new("rel_context"),
            tb_context_to_msg_id::Column::AiContext,
        ))
        .is_in(ai_contexts.clone())
    )
    .order_by(tb_ai_context::Column::Id, sea_orm::Order::Asc)
    .find_also_related(tb_image_attach_file::Entity)
    .find_also_related(tb_context_to_msg_id::Entity) 
    .all(&db)
    .await
    .unwrap();

    
    LOGGER.log(LogLevel::Debug, &format!("before_messages: {:?}", before_messages));

    let context_using_pro = if before_messages.is_empty() {
        false
    } else {
        ai_context_parent.last().unwrap().using_pro_model
    };
    LOGGER.log(LogLevel::Debug, &format!("context_info: {:?}", before_messages));

    let ai_context_map = ai_context.iter().map(|x| (
        x.1.clone().unwrap().ai_context as i64
    )).collect::<hash_set::HashSet<i64>>();
    let mut last_context_id = ai_context.last().unwrap().1.clone().unwrap().ai_context as i64;
    let mut last_node: i64 = parent_context.last().unwrap().id as i64;
    let mut last_time = parent_context.last().unwrap().created_at.to_utc();
    let mut before_messages:Vec<GeminiChatChunk> = before_messages
        .iter()
        .rev()
        .fold(Vec::new(), |mut acc, curr| {
            let current_context_id = curr.2.clone().unwrap().ai_context as i64;
            LOGGER.log(LogLevel::Debug, &format!("curr: {:?},{},{},{:?}", curr,last_node,ai_context_map.contains(&current_context_id),ai_context_map));
            if curr.0.id <= last_node 
            && ai_context_map.contains(&current_context_id)
            && last_context_id <= current_context_id
            && curr.0.created_at.to_utc() <= last_time {
                LOGGER.log(LogLevel::Debug, &format!("FILTERED! curr: {:?}", curr));
                acc.push(curr);
                last_node = curr.0.id;
                last_time = curr.0.created_at.to_utc();
                last_context_id = current_context_id;
            } 
        acc
        })
        .into_iter()
        .rev()
        .map(context_process)
        .collect();

    let user_locale = user.locale.clone();

    if let Some(user_locale_open) = user_locale {
        let begin_query = get_begin_query(user_locale_open, user.clone());
        before_messages.insert(0, begin_query);
    } else {
        let begin_query = get_begin_query("ko".to_string(), user.clone());
        before_messages.insert(0, begin_query);

    }
    LOGGER.log(LogLevel::Debug, &format!("before_messages_filtered: {:?}", before_messages));
    let attachment_user_msg = calling_msg.attachments.clone();
    let mut image = None;
    if attachment_user_msg.len() > 0 {
        let image_origin = attachment_user_msg.get(0).cloned().unwrap();
        if let Some(mime_type) = image_origin.content_type.clone() {
            let ts = &db.transaction(
                |conn| Box::pin(async move {
                    let image_model = entity::tb_image_attach_file::ActiveModel {
                        file_src: sea_orm::Set(image_origin.url.clone()),
                        ..Default::default()
                    };
                    let image_model = entity::tb_image_attach_file::Entity::insert(image_model)
                        .exec_with_returning(conn)
                        .await
                        .unwrap();
                    let image = GeminiImageInputType{
                        base64_image: None,
                        file_url: Some(image_model.file_src),
                        mime_type: mime_type.clone(),
                    };
                    let image = upload_image_to_gemini(image, image_model.image_id.to_string()).await;
                    if image.is_err() {
                        LOGGER.log(LogLevel::Error, &format!("Image upload failed: {:?}", image));
                        return Err(serenity::Error::Other("Image upload failed"));
                    }
                    Ok(image)
                })
            ).await;
            if ts.is_err() {
                LOGGER.log(LogLevel::Error, &format!("Image fetch failed: {:?}", ts));
                calling_msg.channel_id.say(_ctx, "이미지 전송에 실패했습니다.").await.unwrap();
                image = None;
            } else if let Some(tsimage) = ts.as_ref().ok() {
                image = Some(tsimage.clone());
            } else {
                LOGGER.log(LogLevel::Error, "DB Transaction Error");
                calling_msg.channel_id.say(_ctx, "이미지 전송에 실패했습니다.").await.unwrap();
                image = None;
            }
        }
    }
    let image = if image.is_some() {
        let image = image.unwrap();
        let image_res = image.ok().unwrap();
        let image = GeminiImageInputType{
            base64_image: image_res.base64_image,
            file_url: image_res.file_url,
            mime_type: image_res.mime_type.clone(),
        };
        Some(image)
    } else {
        None
    };
    let image_record = image.clone();

    let user_msg_current = GeminiChatChunk{
        query: calling_msg.content.clone(),
        is_bot: false,
        user_id: Some(calling_msg.author.id.get().to_string()),
        timestamp: calling_msg.timestamp.to_string(),
        image
    };
    let _push_query: () = before_messages.push(user_msg_current);
    LOGGER.log(LogLevel::Debug, &format!("Sending Query: {:?}", before_messages));
    let ai_response = gemini_client::GeminiClient::new()
    .send_query_to_gemini(before_messages,context_using_pro).await;
    if ai_response.is_err() {
        typing.stop();
        LOGGER.log(LogLevel::Error, &format!("Gemini API Error: {:?}", ai_response));
        let error_msg = ai_response.unwrap_err();
        let mut response_msg: CreateMessage = CreateMessage::new()
        .content(error_msg);
        if let Some(ref ref_msg) = calling_msg.referenced_message {
            response_msg = response_msg.reference_message(&**ref_msg);
        }
        calling_msg.channel_id.send_message(_ctx, 
            response_msg
        ).await.unwrap();
        LOGGER.log(LogLevel::Error, "Gemini API Error");
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
    let mut image_id:Option<i64> = None;
    if let Some(image) = image_record {
        // insert image...
        if let Some(file_src) = image.file_url {
            let image_inserted =        entity::tb_image_attach_file::ActiveModel {
                file_src: sea_orm::Set(file_src),
                mime_type: sea_orm::Set(Some(image.mime_type.to_string())),
                ..Default::default()
            };
            let image_inserted = entity::tb_image_attach_file::Entity::insert(image_inserted)
            .exec_with_returning(&db)
            .await
            .unwrap();
            image_id = Some(image_inserted.image_id);
        }
    }
    let inserted = AiContextModel {
        user_id: sea_orm::Set(calling_msg.author.id.get() as i64),
        context: sea_orm::Set(calling_msg.content.clone()),
        guild_id: sea_orm::Set(calling_msg.guild_id.unwrap().get() as i64),
        channel_id: sea_orm::Set(calling_msg.channel_id.get() as i64),
        by_bot: sea_orm::Set(false),
        image_file_id: sea_orm::Set(image_id),
        ..Default::default()
    };
    let response_record = AiContextModel {
        user_id: sea_orm::Set(calling_msg.author.id.get() as i64),
        context: sea_orm::Set(ai_response.discord_msg.clone()),
        guild_id: sea_orm::Set(calling_msg.guild_id.unwrap().get() as i64),
        channel_id: sea_orm::Set(calling_msg.channel_id.get() as i64),
        by_bot: sea_orm::Set(true),
        image_file_id: sea_orm::Set(None),
        ..Default::default()
    };
    let _insert_context_desc = AiContextEntity::insert(inserted)
        .add(response_record)
        .exec_with_returning_many(&db)
        .await
        .unwrap();
    LOGGER.log(LogLevel::Debug, &format!("DB Inserted: {:?}", _insert_context_desc));

    let mut continue_context = ai_contexts.last().unwrap().clone();

    let after_parent = tb_discord_message_to_at_context::Entity::find()
    .filter(
        Condition::all()
            .add(
                Expr::col(tb_discord_message_to_at_context::Column::AiMsgId).eq(continue_context)
            )
            .add(
                Expr::col(tb_discord_message_to_at_context::Column::UpdateAt).gt(parent_context.last().unwrap().created_at.to_utc())
            )
    )
    .all(&db)
    .await
    .unwrap();


    let there_is_next_context = after_parent.len() > 0;
    LOGGER.log(LogLevel::Debug, &format!("there_is_next_context: {:?}", there_is_next_context));
    if there_is_next_context {
        // 컨텍스트 분기를 실행해야 함.
        let parent_context = if ai_contexts.len() > 0 {
            let mut parent_context = ai_contexts.clone();
            parent_context.push(continue_context);
            parent_context
        } else {
            vec![]
        };

        let insert_context = AiContextDiscordEntity::insert(AiContextDiscordModel {
        guild_id: sea_orm::Set(calling_msg.guild_id.unwrap().get() as i64),
        root_msg: sea_orm::Set(send_msgs[0].id.get() as i64),
        parent_context: sea_orm::Set(parent_context),
        using_pro_model: sea_orm::Set(context_using_pro),
        ..Default::default()
        }).exec_with_returning(&db)
        .await
        .unwrap();

        continue_context = insert_context.id as i64;
    }

    let _insert_user_and_bot_talk = tb_context_to_msg_id::Entity::insert_many(
        vec![
            tb_context_to_msg_id::ActiveModel {
                ai_msg: sea_orm::Set(_insert_context_desc[0].id as i64),
                ai_context: sea_orm::Set(continue_context),
                ..Default::default()
            },
            tb_context_to_msg_id::ActiveModel {
                ai_msg: sea_orm::Set(_insert_context_desc[1].id as i64),
                ai_context: sea_orm::Set(continue_context),
                ..Default::default()
            }
        ]).exec(&db)
        .await
        .unwrap();


    let ai_context_discord_messages = send_msgs.iter().map(|msg| {
        AiContextDiscordMessageModel {
            discord_message: sea_orm::Set(msg.id.get() as i64),
            ai_msg_id: sea_orm::Set(_insert_context_desc[1].id),
            ..Default::default()
        }
    }).collect::<Vec<_>>();
    let insert_user_msg_to_context =  AiContextDiscordMessageModel {
        discord_message: sea_orm::Set(calling_msg.id.get() as i64),
        ai_msg_id: sea_orm::Set(_insert_context_desc[0].id),
        ..Default::default()
    };
    let _context_inserted = AiContextDiscordMessageEntity::insert_many(ai_context_discord_messages)
    .add(insert_user_msg_to_context)
        .exec(&db)
        .await
        .unwrap();
    

}
