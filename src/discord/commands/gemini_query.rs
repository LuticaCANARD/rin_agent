use core::hash;
use std::collections::{hash_map, hash_set, HashMap};
use std::time::Duration;
use std::vec;

use base64::Engine;
use entity::{tb_context_to_msg_id, tb_image_attach_file};
use gemini_live_api::types::GeminiCachedContentResponse;
use rocket::time::Date;
use sea_orm::prelude::{DateTime, Expr};
use sea_orm::sea_query::{Alias, ExprTrait};
use sea_orm::{Condition, ConnectionTrait, DbErr, EntityTrait, Insert, InsertResult, JoinType, QueryFilter, QueryOrder, QuerySelect, Statement, TransactionError, TransactionTrait, Update};
use serenity::builder::*;
use serenity::model::{guild, prelude::*};
use serenity::prelude::*;
use chrono::{DateTime as ChronoDateTime, Utc, Duration as ChronoDuration};
use crate::discord::constant::DISCORD_DB_ERROR;
use crate::gemini::gemini_client::{self, GeminiCacheInfo, GeminiClientTrait};
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
const DISCORD_MAX_MSG_LENGTH: usize = 1950;
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
        guild_id: Some(origin.0.guild_id.try_into().unwrap()),
        channel_id: Some(origin.0.channel_id.try_into().unwrap()),
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
    let origin_msg = message_context.discord_msg;
    let mut send_msgs:Vec<Message> = vec![];

    let chuncks: Vec<String> = split_text_by_length_and_markdown(&origin_msg, DISCORD_MAX_MSG_LENGTH);
    for chunk in 0..chuncks.len() {
        let msg_last = if need_mention_first == true && chunk == 0 {
            user_mention(&origin_user) + &chuncks.get(chunk).unwrap()
        } else {
            chuncks.get(chunk).unwrap().clone()
        };
        let mut response_msg: CreateMessage = CreateMessage::new()
        .content(msg_last);
        if chunk == chuncks.len() - 1 {
            let sub_items = message_context.thoughts.clone().unwrap_or("".to_string());
            let strs = if need_mention_first == true && chunk == 0 {  
                user_mention(&origin_user) + &chuncks.get(chunk).unwrap()
            } else {
                chuncks.get(chunk).unwrap().clone()
            };
            let used_model = if use_pro {
                GEMINI_MODEL_PRO.to_string()
            } else {
                GEMINI_MODEL_FLASH.to_string()
            };
            response_msg = generate_message_block(strs,
                "Gemini API".to_string(), sub_items,
                used_model,chunk == chuncks.len() - 1
            );
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
    let thinking_bought = options.iter().find(|o| o.name == "thinking_bought");
    let thinking_bought = if thinking_bought.is_some() {
        let unwarped = thinking_bought.unwrap().value.clone();
        match unwarped {
            ResolvedValue::Integer(i) => Some(i as i32),
            _ => None,
        }
    } else {
        None
    };
    

    let query = query.unwrap().value.clone();
    match query {
        
        ResolvedValue::String(ref s) => {
            let chatting_channel = _ctx.http.get_channel(_options.channel_id).await.unwrap();
            let typing: serenity::all::Typing = chatting_channel.guild().unwrap().start_typing(&_ctx.http);

            // Do something with the string value
            LOGGER.log(LogLevel::Debug, &format!("질문: {}", s));
            let discord_response_message = 
                CreateInteractionResponseMessage::new()
                    .content(&format!("{}> {}",user_mention(&_options.user), s));

            // Send a response to the interaction
            _options.create_response(_ctx,CreateInteractionResponse::Message(discord_response_message)).await?;
            let str_query = s.to_string();
            let locale =_options.user.locale.clone().unwrap_or("ko".to_string());
            let user_id = _options.user.id.get();
            let start_user_msg = GeminiChatChunk{
                        query: str_query.clone(),
                        is_bot: false,
                        image: None,
                        timestamp: chrono::Utc::now().to_string(),
                        user_id: Some(user_id.to_string()),
                        guild_id: Some(_options.guild_id.unwrap().get()),
                        channel_id: Some(_options.channel_id.get()),
                    };
            let start_query = get_begin_query(
                locale.clone(),
                user_id.to_string(),
                Some(_options.guild_id.unwrap().get()),
                Some(_options.channel_id.get())
            );
            let mut gemini_client = gemini_client::GeminiClient::new();
            let response = 
                            gemini_client
                            .send_query_to_gemini(
                                vec![
                                    start_user_msg.clone()
                                ],
                                &start_query,
                                use_pro,
                                thinking_bought,
                                None
                            )
                            .await;
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

            //------- DB Action --------
            let db = DB_CONNECTION_POOL.get();
            if db.is_none() {
                LOGGER.log(LogLevel::Error, "DB Connection Error");
                return Err(serenity::Error::Other(DISCORD_DB_ERROR));
            }
            let guild_id = _options.guild_id.unwrap().get();
            let db = db.unwrap().clone();
            let insert_user_and_bot_talk: Vec<tb_ai_context::Model> = AiContextEntity::insert_many(vec![
                AiContextModel {
                    user_id: sea_orm::Set(user_id as i64),
                    context: sea_orm::Set(str_query),
                    guild_id: sea_orm::Set(_options.guild_id.unwrap().get() as i64),
                    channel_id: sea_orm::Set(_options.channel_id.get() as i64),
                    by_bot: sea_orm::Set(false),
                    ..Default::default()
                },AiContextModel {
                    user_id: sea_orm::Set(user_id as i64),
                    context: sea_orm::Set(response.discord_msg),
                    guild_id: sea_orm::Set(guild_id as i64),
                    channel_id: sea_orm::Set(_options.channel_id.get() as i64),
                    by_bot: sea_orm::Set(true),
                    ..Default::default()
                }])
                .exec_with_returning_many(&db)
                .await
                .unwrap();
            LOGGER.log(LogLevel::Debug, &format!("DB Inserted: {:?}", insert_user_and_bot_talk));

            let user_said = GeminiChatChunk {
                query: insert_user_and_bot_talk[0].context.clone(),
                is_bot: false,
                image: None,
                timestamp: insert_user_and_bot_talk[0].created_at.to_utc().to_string(),
                user_id: Some(insert_user_and_bot_talk[0].user_id.to_string()),
                guild_id: Some(guild_id),
                channel_id: Some(_options.channel_id.get()),
            };
            let bot_said = GeminiChatChunk {
                query: insert_user_and_bot_talk[1].context.clone(),
                is_bot: true,
                image: None,
                timestamp: insert_user_and_bot_talk[1].created_at.to_utc().to_string(),
                user_id: Some(insert_user_and_bot_talk[1].user_id.to_string()),
                guild_id: Some(guild_id),
                channel_id: Some(_options.channel_id.get()),
            };

            let gemini_cached_info = gemini_client.start_gemini_cache(
                vec![user_said, bot_said], &start_query, use_pro, 600.0
            ).await;
            let cache_key = if gemini_cached_info.is_err() {
                sea_orm::Set(None)
            } else {
                sea_orm::Set(Some(gemini_cached_info.clone().unwrap().name.clone()))
            };
            let cache_created_at = if gemini_cached_info.is_err() {
                sea_orm::Set(ChronoDateTime::from(chrono::Utc::now()))
            } else {
                sea_orm::Set(ChronoDateTime::parse_from_rfc3339(&gemini_cached_info.clone().unwrap().create_time)
                    .unwrap_or_else(|_| ChronoDateTime::from(chrono::Utc::now())))
            };
            let cache_expires_at = if gemini_cached_info.is_err() {
                sea_orm::Set(ChronoDateTime::from(chrono::Utc::now()))
            } else {
                sea_orm::Set(
                    ChronoDateTime::parse_from_rfc3339(&gemini_cached_info.unwrap().expire_time)
                        .unwrap_or_else(|_| ChronoDateTime::from(chrono::Utc::now()))
                ) 
            };

            let make_context = AiContextDiscordEntity::insert(AiContextDiscordModel { // Create a new context
                guild_id: sea_orm::Set(guild_id as i64),
                root_msg: sea_orm::Set(insert_user_and_bot_talk.get(0).unwrap().id),
                parent_context: sea_orm::Set([].to_vec()),
                using_pro_model: sea_orm::Set(use_pro),
                thinking_bought: sea_orm::Set(thinking_bought),
                cache_key,
                cache_created_at,
                cache_expires_at,
                ..Default::default()
            })
            .exec_with_returning(&db)
            .await
            .unwrap();
            LOGGER.log(LogLevel::Debug, &format!("DB Inserted with Context: {:?}", make_context));
            let _insert_context = tb_context_to_msg_id::Entity::insert_many(
                vec![
                    tb_context_to_msg_id::ActiveModel {
                    ai_msg: sea_orm::Set(insert_user_and_bot_talk[0].id as i64),
                    ai_context: sea_orm::Set(make_context.id as i64),
                    ..Default::default()
                },
                    tb_context_to_msg_id::ActiveModel {
                    ai_msg: sea_orm::Set(insert_user_and_bot_talk[1].id as i64),
                    ai_context: sea_orm::Set(make_context.id as i64),
                    ..Default::default()
                }]
            )
            .exec(&db)
            .await
            .unwrap();
            // 유저가 디코에 보낸 질문
            let insert_user_msg_to_context = AiContextDiscordMessageModel {
                discord_message: sea_orm::Set(_options.id.get() as i64),
                ai_msg_id: sea_orm::Set(insert_user_and_bot_talk[0].id as i64),
                ..Default::default()
            };
            let ai_context_discord_messages = send_msgs.iter().map(|msg| { // AI가 디코에 보낸 답
                AiContextDiscordMessageModel {
                    discord_message: sea_orm::Set(msg.id.get() as i64),
                    ai_msg_id: sea_orm::Set(insert_user_and_bot_talk[1].id as i64),
                    ..Default::default()
                }
            }).collect::<Vec<_>>();
            let _context_inserted = AiContextDiscordMessageEntity::insert_many(ai_context_discord_messages)
                .add(insert_user_msg_to_context)
                .exec(&db)
                .await
                .unwrap();
            // 끝.
            return Ok("ok".to_string());

        },
        _ => {
            // Handle other types if necessary
            LOGGER.log(LogLevel::Error, "질문이 잘못되었습니다.");
            _options.create_response(_ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                    .content("질문이 잘못되었습니다.")
                )
            ).await?;
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
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "thinking_bought",
                "Thinking Bought",
            )
            .max_int_value(20000)
            .min_int_value(100)
            .required(false)
        )
}

pub async fn continue_query(_ctx: &Context,calling_msg:&Message,user:&User) -> Result<(), String> {
    let channel_lock = _ctx.http.get_channel(calling_msg.channel_id)
    .await
    .unwrap();
    let typing = channel_lock.guild().unwrap().start_typing(&_ctx.http);
    let db = DB_CONNECTION_POOL.get();
    if db.is_none() {
        LOGGER.log(LogLevel::Error, "DB Connection Error");
        typing.stop();
        return Err(DISCORD_DB_ERROR.to_string());
    }
    let db = db.unwrap().clone();
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
    .join_as(
        JoinType::InnerJoin,
        Into::<sea_orm::RelationDef>::into(
            AiContextEntity::belongs_to(tb_context_to_msg_id::Entity)
                .from(<entity::prelude::TbAiContext as EntityTrait>::Column::Id)
                .to(tb_context_to_msg_id::Column::AiMsg)
        ), Alias::new("rel_context"))
    .filter(
        Expr::col((
            Alias::new("rel_discord_ctx"),
            TbDiscordMessageToAtContext::DiscordMessage,
        ))
        .eq(msg_ref_id)
    )
    .order_by(<entity::prelude::TbAiContext as EntityTrait>::Column::Id, sea_orm::Order::Desc)
    .find_also_related(tb_context_to_msg_id::Entity)
    .all(&db)
    .await
    .unwrap();

    LOGGER.log(LogLevel::Debug, &format!("parent_context: {:?}", parent_context));
        // parent_context에서 ai_context_id 추출
    let ai_context_id = parent_context
        .last()
        .and_then(|(_, ctx_to_msg)| ctx_to_msg.as_ref().map(|c| c.ai_context));
    if ai_context_id.is_none() {
        LOGGER.log(LogLevel::Error, "AI Context가 없습니다.");
        typing.stop();
        return Err("AI Context가 없습니다.".to_string());
    }
    let ai_context_id = ai_context_id.unwrap() as i64;

    // 한 번의 쿼리로 현재 및 부모 컨텍스트 모두 조회
    let context_ids = {
        let mut ids = ai_context_id.to_string();
        if let Some(ai_context_info) = tb_discord_ai_context::Entity::find_by_id(ai_context_id)
            .one(&db)
            .await
            .unwrap()
        {
            let mut parent_ids = ai_context_info.parent_context.clone();
            parent_ids.push(ai_context_info.id as i64);
            ids = parent_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(",");
        }
        ids
    };

    let map_context_info_vec = tb_discord_ai_context::Entity::find()
        .filter(Expr::col(tb_discord_ai_context::Column::Id).is_in(context_ids.split(',').map(|s| s.parse::<i64>().unwrap()).collect::<Vec<_>>()))
        .all(&db)
        .await
        .unwrap();

    let map_context_info = map_context_info_vec
        .into_iter()
        .map(|x| (x.id as i64, x))
        .collect::<hash_map::HashMap<i64, tb_discord_ai_context::Model>>();

    let ai_context_info = map_context_info.get(&ai_context_id).cloned();
    
    if ai_context_info.is_none() {
        LOGGER.log(LogLevel::Error, "AI Context가 없습니다.");
        typing.stop();
        return Err("AI Context가 없습니다.".to_string());
    }
    let ai_context_info = ai_context_info.unwrap();
    let mut need_load_context_list = ai_context_info.parent_context.clone();
    need_load_context_list.push(ai_context_info.id as i64);
    let need_load_context_list = need_load_context_list.clone();
    LOGGER.log(LogLevel::Debug, &format!("AI Context: {:?}", ai_context_info));

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
        .is_in(need_load_context_list.clone())
    )
    .order_by(tb_ai_context::Column::Id, sea_orm::Order::Asc)
    .find_also_related(tb_image_attach_file::Entity)
    .find_also_related(tb_context_to_msg_id::Entity) 
    .all(&db)
    .await
    .unwrap();

    LOGGER.log(LogLevel::Debug, &format!("before_messages: {:?}", before_messages));

    let context_using_pro = ai_context_info.using_pro_model;
    LOGGER.log(LogLevel::Debug, &format!("context_info: {:?}", before_messages));

    let ai_context_map = need_load_context_list.iter().map(|x| (
        x.clone() as i64
    )).collect::<hash_set::HashSet<i64>>();
    let mut last_context_id = ai_context_info.id as i64;
    let mut check_node = ai_context_info.root_msg;
    let mut curr_check_context = ai_context_info.clone();

    let mut last_node: i64 = parent_context.last().unwrap().0.id as i64;
    let mut before_messages:Vec<GeminiChatChunk> = before_messages
        .iter()
        .rev()
        .fold(Vec::new(), |mut acc, curr| {
            let current_context_id = curr.2.clone().unwrap().ai_context as i64;
            if curr.0.id <= last_node 
            && ai_context_map.contains(&current_context_id)
            && last_context_id == current_context_id {
                acc.push(curr);
                if check_node == curr.0.id as i64 { 
                    // 현재 노드가 컨텍스트의 마지막 노드인 경우이다.
                    if curr_check_context.parent_context.last().is_some() { 
                        // 부모 컨텍스트가 존재하는 경우이다.
                        // context에 대한 모델이다.
                        let info: Option<&tb_discord_ai_context::Model> = map_context_info.get(&current_context_id);
                        if info.is_some() {
                            let info = info.unwrap();
                            check_node = info.root_msg as i64;
                            last_node = check_node;
                            // 현재 컨텍스트를 마지막 컨텍스트 ID로 설정
                            last_context_id = info.id as i64;
                            curr_check_context = info.clone();
                        }
                    } else if curr_check_context.parent_context.len() == 0 {
                        // 루트 노드인 경우이다. 
                        // 아직은 별도의 행동정의를 하지 않아도 좋다.
                    }
                }
            } 
        acc
        })
        .into_iter()
        .rev()
        .map(context_process)
        .collect();

    let user_locale = user.locale.clone();
    LOGGER.log(LogLevel::Debug, &format!("before_messages_filtered: {:?}", before_messages));
    let attachment_user_msg = calling_msg.attachments.clone();
    let mut image = None;
    if attachment_user_msg.len() > 0 {
        let image_origin = attachment_user_msg.get(0).cloned().unwrap();
        if let Some(mime_type) = image_origin.content_type.clone() {
            let ts = &db.transaction(
                |conn| Box::pin(async move {
                    let image_model = entity::tb_image_attach_file::ActiveModel {
                        file_src: sea_orm::Set(image_origin.url),
                        ..Default::default()
                    };
                    let image_model = entity::tb_image_attach_file::Entity::insert(image_model)
                        .exec_with_returning(conn)
                        .await
                        .unwrap();
                    let image = GeminiImageInputType{
                        base64_image: None,
                        file_url: Some(image_model.file_src),
                        mime_type: mime_type
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
    let image_record_for_tx = image_record.clone();

    let user_msg_current = GeminiChatChunk{
        query: calling_msg.content.clone(),
        is_bot: false,
        user_id: Some(calling_msg.author.id.get().to_string()),
        timestamp: calling_msg.timestamp.to_string(),
        image,
        guild_id: calling_msg.guild_id.map(|g| g.get()),
        channel_id: Some(calling_msg.channel_id.get()),
    };
    let _push_query: () = before_messages.push(user_msg_current.clone());
    LOGGER.log(LogLevel::Debug, &format!("Sending Query: {:?}", before_messages));
    let after_parent = tb_discord_message_to_at_context::Entity::find()
    .join_as(
        JoinType::LeftJoin,
        Into::<sea_orm::RelationDef>::into(
            tb_discord_message_to_at_context::Entity::belongs_to(tb_context_to_msg_id::Entity)
                .from(tb_discord_message_to_at_context::Column::AiMsgId)
                .to(tb_context_to_msg_id::Column::AiMsg)
        ),
        Alias::new("rel_discord_ctx"),
    )
    .filter(
        Condition::all()
            .add(
                // 다음노드가 생성될 필요가 있는지만 묻는 것이므로, in 설정은 기각한다.
                Expr::col(tb_context_to_msg_id::Column::AiContext).eq(ai_context_info.id as i64)
            )
            .add(
                Expr::col(tb_discord_message_to_at_context::Column::AiMsgId).gt(parent_context.last().unwrap().0.id as i64)
            )
    )
    .all(&db)
    .await
    .unwrap();
    let continue_context = ai_context_info.id as i64;
    LOGGER.log(LogLevel::Debug, &format!("after_parent: {:?}", after_parent));
    // 이후의 컨텍스트가 존재하면 true
    let there_is_next_context = after_parent.len() > 0;
    let cache_is_valid =  !there_is_next_context && ai_context_info.cache_expires_at.to_utc() > (chrono::Utc::now() + ChronoDuration::seconds(2));
    let cache_key: Option<String> = if cache_is_valid == true && ai_context_info.cache_key.is_some() {
        ai_context_info.cache_key.clone()
    } else {
        None
    };
    let thinking_bought = if ai_context_info.thinking_bought.is_some() {
        Some(ai_context_info.thinking_bought.unwrap())
    } else {
        None
    };
    let send_vector = if cache_is_valid {
        vec![user_msg_current.clone()]
    } else {
        before_messages.clone()
    };
    let mut gemini_client = gemini_client::GeminiClient::new();
    let begin_query = get_begin_query(user_locale.unwrap_or("ko".to_string()),calling_msg.author.id.get().to_string()
    ,Some(calling_msg.guild_id.unwrap().get()),
    Some(calling_msg.channel_id.get())
    );


    let ai_response = gemini_client
    .send_query_to_gemini(
        send_vector,
        &begin_query,
        context_using_pro,
        thinking_bought,
        cache_key,
    )
    .await;

    
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
        return Err("Gemini API Error".to_string() );
    }
    let ai_response = ai_response.unwrap();
    let guild_id = calling_msg.guild_id.unwrap().get();
    let send_msgs:Vec<Message> = send_split_msg(
        _ctx, 
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
            .await;
            if image_inserted.is_err() {
                LOGGER.log(LogLevel::Error, &format!("Image insert failed: {:?}", image_inserted));
                calling_msg.channel_id.say(_ctx, "이미지 전송에 실패했습니다.").await.unwrap();
                return Err("이미지 전송에 실패했습니다.".to_string());
            }
            image_id = Some(image_inserted.unwrap().image_id);
        }
    }
    let calling_msg = calling_msg.clone();
    let transaction_result: Result<(), TransactionError<sea_orm::DbErr>> = db
    .transaction(
        move |txn| Box::pin(async move {
        let mut _final_image_id = None;
        if let Some(image_data) = image_record_for_tx.clone() { // image_record was cloned and moved
            if let Some(file_src) = image_data.file_url {
                let image_to_insert = entity::tb_image_attach_file::ActiveModel {
                    file_src: sea_orm::Set(file_src),
                    mime_type: sea_orm::Set(Some(image_data.mime_type.to_string())),
                    ..Default::default()
                };
                let inserted_image_record = entity::tb_image_attach_file::Entity::insert(image_to_insert)
                    .exec_with_returning(txn)
                    .await?;
                _final_image_id = Some(inserted_image_record.image_id);
            }
        } // 유저 질의
        let inserted_context_desc = AiContextEntity::insert_many(
            vec![
                AiContextModel {
                    user_id: sea_orm::Set(calling_msg.author.id.get() as i64),
                    context: sea_orm::Set(calling_msg.content.clone()),
                    guild_id: sea_orm::Set(calling_msg.guild_id.unwrap().get() as i64),
                    channel_id: sea_orm::Set(calling_msg.channel_id.get() as i64),
                    by_bot: sea_orm::Set(false),
                    image_file_id: sea_orm::Set(image_id),
                    ..Default::default()
                }, AiContextModel { // AI 응답
                    user_id: sea_orm::Set(calling_msg.author.id.get() as i64),
                    context: sea_orm::Set(ai_response.discord_msg.clone()),
                    guild_id: sea_orm::Set(calling_msg.guild_id.unwrap().get() as i64),
                    channel_id: sea_orm::Set(calling_msg.channel_id.get() as i64),
                    by_bot: sea_orm::Set(true),
                    image_file_id: sea_orm::Set(None),
                    ..Default::default()
                } ]
        )
        .exec_with_returning_many(txn)
        .await?;
        LOGGER.log(LogLevel::Debug, &format!("DB Inserted: {:?}\nthere_is_next_context: {:?}", inserted_context_desc,there_is_next_context));
        let mut current_context_id_for_relation = continue_context; // Original continue_context before potential update


        if there_is_next_context {
            // 컨텍스트 분기를 실행해야 함.
            let mut parent_context_lst = ai_context_info.parent_context.clone();
            parent_context_lst.push(ai_context_info.id as i64);
            let parent_context_lst = parent_context_lst;
            let mut concated_query = before_messages.clone();
            concated_query.push(GeminiChatChunk{
                query: calling_msg.content.clone(),
                is_bot: false,
                user_id: Some(calling_msg.author.id.get().to_string()),
                timestamp: calling_msg.timestamp.to_string(),
                image: image_record_for_tx,
                guild_id: calling_msg.guild_id.map(|g| g.get()),
                channel_id: Some(calling_msg.channel_id.get()),
            });
            concated_query.push(GeminiChatChunk{
                query: ai_response.discord_msg.clone(),
                is_bot: true,
                user_id: Some(calling_msg.author.id.get().to_string()),
                timestamp: chrono::Utc::now().to_string(),
                image: None,
                guild_id: calling_msg.guild_id.map(|g| g.get()),
                channel_id: Some(calling_msg.channel_id.get()),
            });
            let concated_query = concated_query;
            let cache_result = gemini_client.       
                start_gemini_cache(
                    concated_query, 
                    &begin_query, 
                context_using_pro, 
                600.0)
                .await;
            let cache_result = cache_result.unwrap();
            let ct = ChronoDateTime::parse_from_rfc3339(&cache_result.create_time)
                .unwrap_or_else(|_| ChronoDateTime::from(chrono::Utc::now()));
            let cache_created_at = sea_orm::Set(ct);
            let cache_expires_at = sea_orm::Set(
                ChronoDateTime::parse_from_rfc3339(&cache_result.expire_time)
                    .unwrap_or_else(|_| ChronoDateTime::from(chrono::Utc::now()))
            );

            let new_discord_context = AiContextDiscordEntity::insert(
                AiContextDiscordModel {
                guild_id: sea_orm::Set(guild_id as i64),
                root_msg: sea_orm::Set(inserted_context_desc[0].id),
                parent_context: sea_orm::Set(parent_context_lst),
                using_pro_model: sea_orm::Set(context_using_pro),
                thinking_bought: sea_orm::Set(thinking_bought),
                cache_key: sea_orm::Set(Some(cache_result.name.clone())),
                cache_created_at,
                cache_expires_at,
                ..Default::default()})
            .exec_with_returning(txn)
            .await?;
            current_context_id_for_relation = new_discord_context.id as i64;
        } else { // 새 분기가 아닌 경우, 캐싱만 실행한다.

            if  ai_context_info.cache_key.is_some() {
                let _dropped = gemini_client
                    .drop_cache(&ai_context_info.cache_key.unwrap().clone())
                    .await;
            }
            let mut concated_query = before_messages.clone();
            concated_query.push(GeminiChatChunk{
                query: ai_response.discord_msg.clone(),
                is_bot: true,
                user_id: Some(calling_msg.author.id.get().to_string()),
                timestamp: chrono::Utc::now().to_string(),
                image: None,
                guild_id: calling_msg.guild_id.map(|g| g.get()),
                channel_id: Some(calling_msg.channel_id.get()),
            });
            concated_query.push(GeminiChatChunk{
                query: calling_msg.content.clone(),
                is_bot: false,
                user_id: Some(calling_msg.author.id.get().to_string()),
                timestamp: calling_msg.timestamp.to_string(),
                image: image_record_for_tx,
                guild_id: calling_msg.guild_id.map(|g| g.get()),
                channel_id: Some(calling_msg.channel_id.get()),
            });
            let created_cached = gemini_client.start_gemini_cache(
                concated_query,
                &begin_query,
                context_using_pro,
                600.0
            ).await;
            if created_cached.is_ok() {
                let created_cached = created_cached.unwrap();
                let ct = ChronoDateTime::parse_from_rfc3339(&created_cached.create_time)
                    .unwrap_or_else(|_| ChronoDateTime::from(chrono::Utc::now()));
                let cache_created_at = sea_orm::Set(ct);
                let cache_expires_at = sea_orm::Set(
                    ChronoDateTime::parse_from_rfc3339(&created_cached.expire_time)
                        .unwrap_or_else(|_| ChronoDateTime::from(chrono::Utc::now()))
                );
                let update_context = AiContextDiscordEntity::update(
                    AiContextDiscordModel {
                        id: sea_orm::Set(ai_context_info.id as i64),
                        cache_key: sea_orm::Set(Some(created_cached.name.clone())),
                        cache_created_at,
                        cache_expires_at,
                        ..Default::default()
                    }
                )
                .exec(txn)
                .await?;
                LOGGER.log(LogLevel::Debug, &format!("DB Updated Context: {:?}", update_context));
            }

        }

    tb_context_to_msg_id::Entity::insert_many(
        vec![
            tb_context_to_msg_id::ActiveModel {
                ai_msg: sea_orm::Set(inserted_context_desc[0].id as i64),
                ai_context: sea_orm::Set(current_context_id_for_relation),
                ..Default::default()
            },
            tb_context_to_msg_id::ActiveModel {
                ai_msg: sea_orm::Set(inserted_context_desc[1].id as i64),
                ai_context: sea_orm::Set(current_context_id_for_relation),
                ..Default::default()
            }
        ]).exec(txn)
        .await?;

    let ai_context_discord_messages = send_msgs.iter().map(|msg| {
        AiContextDiscordMessageModel {
            discord_message: sea_orm::Set(msg.id.get() as i64),
            ai_msg_id: sea_orm::Set(inserted_context_desc[1].id),
            ..Default::default()
        }
    }).collect::<Vec<_>>();
    let insert_user_msg_to_context =  AiContextDiscordMessageModel {
        discord_message: sea_orm::Set(calling_msg.id.get() as i64),
        ai_msg_id: sea_orm::Set(inserted_context_desc[0].id),
        ..Default::default()
    };
    AiContextDiscordMessageEntity::insert_many(ai_context_discord_messages)
        .add(insert_user_msg_to_context)
        .exec(txn)
        .await?;
        Ok(())
    })).await;
    if transaction_result.is_err() {
        LOGGER.log(LogLevel::Error, &format!("DB Transaction Error: {:?}", transaction_result));
        calling_msg.channel_id.say(_ctx, "DB Transaction Error").await.unwrap();
        return Err("DB Transaction Error".to_string());
    }
    let transaction_result = transaction_result.unwrap();
    LOGGER.log(LogLevel::Debug, &format!("DB Transaction Result: {:?}", transaction_result));

    Ok(())

}
