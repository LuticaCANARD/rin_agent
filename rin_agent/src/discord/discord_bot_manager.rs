use entity::tb_alarm_model;
use entity::tb_discord_guilds;
use rs_ervice::RSContext;
use rs_ervice::RSContextBuilder;
use rs_ervice::RSContextService;
use sea_orm::EntityOrSelect;
use sea_orm::EntityTrait;
use sea_orm::QuerySelect;
use serenity::all::CreateAttachment;
use serenity::all::CreateCommand;
use serenity::all::CreateEmbed;
use serenity::all::CreateEmbedFooter;
use serenity::all::CreateInteractionResponse;
use serenity::all::CreateInteractionResponseMessage;
use serenity::all::CreateMessage;
use serenity::all::EditAttachments;
use serenity::all::EditMessage;
use serenity::all::Guild;
use serenity::all::GuildId;
use serenity::all::Http;
use serenity::all::UnavailableGuild;
use serenity::client::Context;
use serenity::prelude::*;
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::gateway::GatewayIntents;
use serenity::model::application::{Command, Interaction};
use songbird::SerenityInit;
use sqlx::types::chrono;
use tokio::sync::watch::Receiver;
use std::collections::HashMap;
use std::env;
use std::hash::Hasher;
use std::panic;
use std::sync::Arc;
use std::sync::OnceLock;
use serenity::model::prelude::*;
use std::pin::Pin;
use std::future::Future;
use crate::discord::voice_handler::voice_handler::VoiceHandler;
use crate::libs::logger::{LOGGER, LogLevel};
use crate::libs::thread_message::GeminiFunctionAlarm;
use crate::libs::thread_pipelines::AsyncThreadPipeline;
use crate::libs::thread_pipelines::GeminiChannelResult;
use crate::libs::thread_pipelines::GEMINI_FUNCTION_EXECUTION_ALARM;
use crate::libs::thread_pipelines::SCHEDULE_TO_DISCORD_PIPELINE;
use crate::model::db::driver::DB_CONNECTION_POOL;
use crate::service::discord_error_msg::send_additional_log;
use crate::service::discord_error_msg::send_debug_error_log;
use crate::service::discord_message_service::{MessageSendReceiver, MessageSendSender, create_message_channel, init_message_sender};
use crate::service::voice_session_manager;
use std::sync::LazyLock;
use tokio::signal;

use super::commands::gemini_query;
use super::constant::DISCORD_DB_ERROR;

macro_rules! register_commands_module {
    ($($module:ident),*) => {
        vec![
            $(
                crate::discord::commands::$module::register(),
            )*
        ]
    };
}
macro_rules! get_command_function {
    // TODO : 차후에 run에서 다 처리하도록 정정/ string으로 return하는 부분 제거
    
    ($($module:ident),*) => {
        |name: String, context: &Context, interaction: &CommandInteraction| {
            let name = name.clone(); // Clone the name to avoid lifetime issues
            let context = context.clone(); // Clone the context if necessary
            let interaction = interaction.clone(); // Clone the interaction if necessary

            Box::pin(async move {
                match name.as_str() {
                    $(
                        stringify!($module) => {
                            let response = crate::discord::commands::$module::run(&context, &interaction).await;
                            if let Err(err) = response {
                                LOGGER.log(LogLevel::Error, &format!("Discord > Error executing command {}: {:?}", stringify!($module), err));
                                return Err(err);
                            }
                            let response = response.unwrap();
                            if response.clone().do_not_send {
                                return Ok("Command executed successfully, but response was suppressed.".to_string());
                            }
                            interaction.create_response(context, response.content).await?;
                            let response = format!("Discord > Command {} executed successfully!", stringify!($module));
                            LOGGER.log(LogLevel::Info, &response);
                            Ok(response)
                        }
                    )*
                    _ => {
                        LOGGER.log(LogLevel::Error, &format!("Discord > Unknown command: {}", name));
                        Ok("Unknown command".to_string())
                    }
                }
            })
        }
    };
}
macro_rules! define_lazy_static {
    ($commands:ident, $activate_commands:ident, [$($module:ident),*]) => {
        static $commands: LazyLock<Vec<CreateCommand>> = LazyLock::new(|| {
            register_commands_module! {
                $($module),*
            }
        });

        static $activate_commands: LazyLock<Box<dyn Fn(String, &Context, &CommandInteraction) -> 
            Pin<Box<dyn Future<Output = Result<String, serenity::Error>> + Send>> + Send + Sync>> = LazyLock::new(|| {
            Box::new(get_command_function! {
                $($module),*
            })
        });
    };
}

define_lazy_static!(USING_COMMANDS, USING_ACTIVATE_COMMANDS, 
    [
        lping,
        gemini_query,
        lutica_repo,
        join_voice,
        leave_voice
    ]
);


static MESSAGE_PROCESS_MAP: LazyLock<Mutex<HashMap<u64, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn remove_message_process_map_entry(message_id: u64) {
    MESSAGE_PROCESS_MAP.lock().await.remove(&message_id);
}

static CLIENT_ID: LazyLock<Option<UserId>> = LazyLock::new(|| (std::env::var("DISCORD_CLIENT_ID").ok()).and_then(|id| id.parse::<u64>().ok()).map(UserId::new));

fn make_message_embed(title: &str, description: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title(title)
        .description(description)
}

async fn register_commands(ctx: Context, guild_id: GuildId) {
    // Register commands here
    let commands = USING_COMMANDS.clone();
    LOGGER.log(LogLevel::Info, &format!("Registering commands for guild: {}", guild_id));
    if let Err(err) = ctx.http.create_guild_commands(guild_id, &commands.clone()).await {
        LOGGER.log(LogLevel::Error, &format!("Failed to register commands for guild {}: {:?}", guild_id, err));
        return;
    }
}
pub struct BotManager {
    client: Client,
    gemini_function_channel: &'static Receiver<GeminiChannelResult>,
    alarm_channel: &'static Receiver<GeminiFunctionAlarm<Option<tb_alarm_model::Model>>>,
    pub message_sender: Option<MessageSendSender>,
    message_receiver: Option<MessageSendReceiver>,
}
static DISCORD_SERVICE: OnceLock<rs_ervice::RSContext> = OnceLock::new();
static MESSAGE_RECEIVER: OnceLock<std::sync::Mutex<Option<MessageSendReceiver>>> = OnceLock::new();


pub fn get_message_receiver() -> Option<MessageSendReceiver> {
    MESSAGE_RECEIVER.get()?.lock().ok()?.take()
}

pub async fn get_discord_service() -> &'static RSContext {
    if let Some(ctx) = DISCORD_SERVICE.get() {
        ctx
    } else {
        let ctx = RSContextBuilder::new()
            .register::<BotManager>()
            .await
            .expect("Failed to register BotManager service")
            .build()
            .await
            .expect("Failed to build RSContext with BotManager service");
        DISCORD_SERVICE.set(ctx).ok();
        DISCORD_SERVICE.get().unwrap()
    }
}

impl BotManager{
    pub async fn new() -> (Self, MessageSendReceiver) {
        let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
        let intents = GatewayIntents::GUILDS
            |GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::GUILD_VOICE_STATES // 음성 상태를 위해 필수!
            | GatewayIntents::MESSAGE_CONTENT;
        let gemini_function_channel = &GEMINI_FUNCTION_EXECUTION_ALARM.receiver;
        let alarm_channel = &SCHEDULE_TO_DISCORD_PIPELINE.receiver;
        let client = Client::builder(token, intents)
                .event_handler(Handler)
                .voice_manager(
                    VoiceHandler {}
                )
                .register_songbird()
                .await
                .expect("Error creating client");
        {
            let mut data = client.data.write().await;
            if let Some(voice_manager) = client.voice_manager.as_ref() {
                data.insert::<VoiceHandler>(
                    Arc::new(Mutex::new(Arc::clone(voice_manager)))
                )
            }
        }
        let (message_sender, message_receiver) = create_message_channel();
        
        // 전역 메시지 리시버 저장소 초기화 (빈 상태로)
        MESSAGE_RECEIVER.get_or_init(|| std::sync::Mutex::new(None));
        init_message_sender(message_sender.clone());
        
        (Self {
            client,
            gemini_function_channel,
            alarm_channel,
            message_sender: Some(message_sender),
            message_receiver: Some(message_receiver),
        }, MESSAGE_RECEIVER.get().unwrap().lock().unwrap().take().unwrap_or_else(|| {
            let (_, rx) = create_message_channel();
            rx
        }))
    }
    pub async fn send_message(&self, channel_id: ChannelId, content: CreateMessage) -> Result<Message, serenity::Error> {
        channel_id.send_message(&self.client.http, content).await
    }
    pub async fn run(&mut self) -> Result<(), serenity::Error> {
        let mut message_receiver = self.message_receiver.take()
            .ok_or_else(|| serenity::Error::Other("Message receiver not available"))?;
        let mut fun_alarm_receiver = self.gemini_function_channel.to_owned();
        let mut alarm_channel = self.alarm_channel.clone();
        let client_control = self.client.http.clone();
        
        // 메시지 전송 채널 처리를 위한 태스크 생성
        let http_for_messages = self.client.http.clone();
        tokio::spawn(async move {
            while let Some(request) = message_receiver.recv().await {
                let result = request.channel_id
                    .send_message(&http_for_messages, request.content)
                    .await;
                
                let _ = request.response_sender.send(result);
            }
            LOGGER.log(LogLevel::Debug, "Message sender channel closed");
        });
        
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    res = fun_alarm_receiver.changed() => {
                        match res {
                            Ok(_) => {
                                let message = {
                                    let alarm = fun_alarm_receiver.borrow_and_update();
                                    alarm.clone()
                                };
                                let u64_msg_id = message.message_id.parse::<u64>().unwrap();
                                let mut message_process_map = MESSAGE_PROCESS_MAP.lock().await;
                                let mut sending = if message_process_map.contains_key(&u64_msg_id) {
                                    message_process_map.get(&u64_msg_id).unwrap().clone()
                                } else {
                                    String::from("Processing your request, please wait...")
                                };

                                LOGGER.log(LogLevel::Debug, &format!("Gemini function alarm received. {}", message.message_id));
                                let channel_id = ChannelId::new(message.channel_id.parse::<u64>().unwrap());
                                let target_user = UserId::new(message.sender.parse::<u64>().unwrap());
                                let msg = message.message.clone().result_message;
                                sending = format!("{} \n {}", sending.clone(), msg.clone()).to_string();
                                let sending_msg = if sending.len() > 1024 {
                                    sending.chars().rev().take(1024).collect::<String>().chars().rev().collect()
                                } else {
                                    sending.clone()
                                };
                                message_process_map.insert(u64_msg_id, sending_msg.clone());
                                if !message.need_send {
                                    LOGGER.log(LogLevel::Debug, "No need to send message, skipping...");
                                    continue;
                                }
                                let mut edit_cmd = EditMessage::new()
                                    .content(format!("{} \n {}", target_user.mention(), msg))
                                    .embed(
                                        CreateEmbed::new()
                                            .title("Gemini Function Alarm")
                                            .description(sending_msg.clone())
                                            .footer(CreateEmbedFooter::new("time... : ".to_string() + &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))
                                    );

                                if let Some(image_data) = message.message.image {
                                    let mime = message.message.result.get("mime").and_then(|m| m.as_str()).unwrap_or("image/png").to_string();
                                    let ext = if mime == "image/png" {
                                        "png"
                                    } else if mime == "image/jpeg" {
                                        "jpg"
                                    } else {
                                        "bin"
                                    };
                                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                                    hasher.write(image_data.as_slice());
                                    let hash_img = hasher.finish();
                                    let filename = format!("image_{}_{}.{}", hash_img, chrono::Local::now().format("%Y-%m-%d"), ext);
                                    let msgid = MessageId::new(u64_msg_id);
                                    let msg = client_control.clone().get_message(channel_id.clone(),msgid).await.unwrap();
                                    edit_cmd = edit_cmd.attachments(
                                        EditAttachments::keep_all(
                                            &msg
                                        ).add(
                                            CreateAttachment::bytes(
                                                image_data, filename
                                            )
                                        )
                                    );
                                
                                }

                                channel_id.edit_message(
                                    client_control.clone(), 
                                    u64_msg_id,
                                    edit_cmd
                                ).await.unwrap();
                            }
                            Err(_) => {
                                LOGGER.log(LogLevel::Error, "Gemini function alarm receiver has been closed.");
                                break;
                            }
                        }
                    }
                    res = alarm_channel.changed() => {
                        match res {
                            Ok(_) => {
                                let message = {
                                    let alarm = alarm_channel.borrow_and_update();
                                    alarm.clone()
                                };
                                LOGGER.log(LogLevel::Debug, &format!("Alarm received. {}", message.message_id));
                                let channel_id = ChannelId::new(message.channel_id.parse::<u64>().unwrap());
                                let target_user = UserId::new(message.sender.parse::<u64>().unwrap());
                                let msg = message.message.clone();
                                let msg = match msg {
                                    Some(m) => m.message,
                                    None => "No message provided".to_string(),
                                };
                                LOGGER.log(LogLevel::Debug, &format!("Sending alarm message to {},{}: {}", target_user, channel_id, msg));
                                let _ = send_message_for_alarm::<Arc<Http>>(client_control.clone(), channel_id, target_user, msg.clone())
                                .await.map_err(|e| {
                                    LOGGER.log(LogLevel::Error, &format!("Failed to send alarm message: {:?}", e));
                                });
                            }
                            Err(_) => {
                                LOGGER.log(LogLevel::Error, "Alarm receiver has been closed.");
                                break;
                            }
                        }
                    }

                res = signal::ctrl_c() => {
                        match res {
                            Ok(_) => {
                                LOGGER.log(LogLevel::Info, "SIGINT received, shutting down...");
                                std::process::exit(0);
                            }
                            Err(err) => {
                                LOGGER.log(LogLevel::Error, &format!("Failed to listen for SIGINT: {:?}", err));
                            }
                        }
                    }
                }
            }
        });
        
        // Discord 봇 시작
        self.client.start().await
    }

    
}

async fn send_message_for_alarm<T: CacheHttp>(client:T, channel_id: ChannelId,target_user:UserId, memo: String)->Result<serenity::all::Message, serenity::Error> {
        channel_id.send_message(client, CreateMessage::new()
            .content(format!("{} \n {}", target_user.mention(), memo))
            .embed(
                CreateEmbed::new()
                    .title("Alarm")
                    .description(memo)
                    .footer(CreateEmbedFooter::new("time... : ".to_string() + &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))
            )
        ).await
    }


pub struct Handler;

impl RSContextService for BotManager {
    async fn on_all_services_built(&self, context: &rs_ervice::RSContext) -> rs_ervice::AsyncHooksResult {
        LOGGER.log(LogLevel::Info, "Discord > BotManager service is built and ready to use.");
        // Register the bot manager in the RSContext
        Ok(())
    }
    fn on_register_crate_instance() -> impl Future<Output = Self> where Self: Sized {
        async { 
            let (bot_manager, message_receiver) = BotManager::new().await;
            // 메시지 리시버를 전역 저장소에 다시 저장
            if let Some(storage) = MESSAGE_RECEIVER.get() {
                if let Ok(mut guard) = storage.lock() {
                    *guard = Some(message_receiver);
                }
            }
            bot_manager
        }
    }
    async fn on_service_created(&mut self, builder: &RSContextBuilder) -> rs_ervice::AsyncHooksResult {
        LOGGER.log(LogLevel::Info, "Discord > BotManager service is created.");
        // Register the bot manager in the RSContext
        Ok(())
    }


}

#[async_trait]
impl EventHandler for Handler {
    //https://github.com/serenity-rs/serenity/blob/current/examples/e14_slash_commands/src/main.rs
    async fn ready(&self, ctx: Context, _ready: Ready) {
        // Delete remaining commands and register new ones
        let db = DB_CONNECTION_POOL.get().expect("Database connection not initialized");
        let guilds = ctx.cache.guilds().len();

        println!("Guilds in the Cache: {}", guilds);
        let gids = tb_discord_guilds::Entity::find()
            .select()
            .column(tb_discord_guilds::Column::GuildId)
            .into_model::<tb_discord_guilds::Model>()
            .all(db).await.unwrap();

        for guild in &gids {
            let guild_id = GuildId::new(guild.guild_id as u64);
            let commands = ctx.http.get_guild_commands(guild_id).await.unwrap();
            for command in commands {
                ctx.http.delete_guild_command(guild_id, command.id).await.unwrap();
            }
        }
        for guild in &gids {
            let guild_id = GuildId::new(guild.guild_id as u64);
            register_commands(ctx.clone(), guild_id).await;
        }
        LOGGER.log(LogLevel::Info, "Discord > Bot is ready!");
    }
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {

        LOGGER.log(LogLevel::Info, &format!("Guild created: {:?}", guild.id));
        if is_new.unwrap_or(true) {
            // Register commands here
            register_commands(ctx.clone(), guild.id).await;
            LOGGER.log(LogLevel::Info, &format!("Commands registered for guild: {:?}", guild.id));
        }
    }


    async fn guild_delete(&self, ctx: Context, guild: UnavailableGuild, full_guild: Option<Guild>) {
        LOGGER.log(LogLevel::Info, &format!("Guild deleted: {:?}", guild.id));
        let guild_id = guild.id;
        let commands: Vec<Command> = ctx.http.get_guild_commands(guild_id).await.unwrap();
        for command in commands {
            ctx.http.delete_guild_command(guild_id, command.id).await.unwrap();
        }
    }
    async fn message(&self, ctx: Context, msg: Message) {
        // Handle messages here
        if msg.author.bot {
            return;
        }
        LOGGER.log(LogLevel::Info, &format!("Received message: {}", msg.content));
        // 기능 1: 동일 링크 3회 이상 전송 시 차단


        // 기능 2: reply시 쿼리 탐색
        let cpy_msg = msg.clone();
        let parent_id = cpy_msg.referenced_message;
        if let Some(parent) = parent_id {
            if parent.author.id == *CLIENT_ID.as_ref().unwrap() {
                // Check if the message is a reply to another message

                let content = cpy_msg.content.clone();
                if content.len() > 0 || cpy_msg.attachments.len() > 0 {
                    // Handle the query here
                    LOGGER.log(LogLevel::Info, &format!("Query found: {:?}", content));
                    // Directly call the async function without catch_unwind
                    let cq = gemini_query::continue_query(&ctx, &msg, &msg.author).await;
                    if let Err(err) = cq {
                        LOGGER.log(LogLevel::Error, &format!("Error processing query: {:?}", err));
                        msg.channel_id.send_message(ctx, 
                            CreateMessage::new().embed(
                                make_message_embed("Error", "An error occurred while processing your query. Please try again later.")
                                .color(0xFF0000)
                                .footer(CreateEmbedFooter::new("time... : ".to_string() + &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))
                            )                         
                        ).await.unwrap();
                    }
                }
            }
        }


    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let command_name = command.data.name.clone();
                // Handle the command here
                // For example, you can call a function to process the command
                // and send a response back to the user.
                let command_future = &USING_ACTIVATE_COMMANDS;

                if let Err(err) = command_future(command_name.clone(), &ctx, &command).await {
                    LOGGER.log(LogLevel::Error, &format!("Discord > Error executing command {}: {:?}", command_name, err));
                    if matches!(err, serenity::Error::Other("Gemini API Error")) {
                        LOGGER.log(LogLevel::Error,  err.to_string().as_str());
                        command.channel_id.send_message(ctx, 
                            CreateMessage::new().embed(
                                make_message_embed("Gemini API Error", "Gemini API Error - Please contact the administrator: @lutica_canard")
                                .footer(
                                    CreateEmbedFooter::new("time... : ".to_string() + &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
                                )
                                .color(0xFF0000)
                            )                         
                        ).await.unwrap();
                    } else if matches!(err, serenity::Error::Other(DISCORD_DB_ERROR)) {
                        LOGGER.log(LogLevel::Error, err.to_string().as_str());
                        command.channel_id.send_message(ctx, 
                            CreateMessage::new().embed(
                                make_message_embed("DB Error", "DB Error - Please contact the administrator: @lutica_canard")
                                .color(0xF00000)
                                .footer(CreateEmbedFooter::new("time... : ".to_string() + &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))
                            )                         
                        ).await.unwrap();

                    } else if matches!(err, serenity::Error::Http(_)) {
                        LOGGER.log(LogLevel::Error,  err.to_string().as_str());
                        command.channel_id.send_message(ctx, 
                            CreateMessage::new().embed(
                                make_message_embed("HTTP Error", "HTTP Error - Please contact the administrator: @lutica_canard").color(0xFF0000).footer(CreateEmbedFooter::new("time... : ".to_string() + &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))
                            )                         
                        ).await.unwrap();
                    } else {
                        LOGGER.log(LogLevel::Error,  err.to_string().as_str());
                        if let Err(e) = command.create_response(ctx, 
                            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                            .content("General Discord Bot Error")
                            .add_embed(
                                make_message_embed("Discord Bot Error", "General Discord Bot Error - Please contact the administrator: @lutica_canard").color(0xFF0000)
                            )
                        )).await {
                            LOGGER.log(LogLevel::Error, &format!("Failed to send error response (interaction may already be acknowledged): {:?}", e));
                        }
                    }
                    return;
                }

            }
            Interaction::Ping(ping) =>{
                LOGGER.log(LogLevel::Debug, "Discord > Ping interaction received");
            }
            _ => {
                
            }
        }
    }
    
}

    
