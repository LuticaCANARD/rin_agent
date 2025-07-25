use entity::tb_alarm_model;
use entity::tb_discord_guilds;
use rs_ervice::RSContext;
use rs_ervice::RSContextBuilder;
use rs_ervice::RSContextService;
use sea_orm::EntityOrSelect;
use sea_orm::EntityTrait;
use sea_orm::ModelTrait;
use sea_orm::QuerySelect;
use serenity::all::CreateCommand;
use serenity::all::CreateEmbed;
use serenity::all::CreateEmbedFooter;
use serenity::all::CreateInteractionResponse;
use serenity::all::CreateInteractionResponseMessage;
use serenity::all::CreateMessage;
use serenity::all::Guild;
use serenity::all::GuildId;
use serenity::all::Http;
use serenity::all::UnavailableGuild;
use serenity::client;
use serenity::client::Context;
use serenity::prelude::*;
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::gateway::GatewayIntents;
use serenity::model::application::{Command, Interaction};
use sqlx::types::chrono;
use tokio::sync::watch::Receiver;
use std::cell::OnceCell;


use std::env;
use std::panic;
use std::sync::Arc;
use std::sync::OnceLock;
use serenity::model::prelude::*;
use std::pin::Pin;
use std::future::Future;
use crate::discord::voice_handler::voice_handler::VoiceHandler;
use crate::gemini::types::GeminiActionResult;
use crate::libs::logger::{LOGGER, LogLevel};
use crate::libs::thread_message::GeminiFunctionAlarm;
use crate::libs::thread_pipelines::AsyncThreadPipeline;
use crate::libs::thread_pipelines::GeminiChannelResult;
use crate::libs::thread_pipelines::GEMINI_FUNCTION_EXECUTION_ALARM;
use crate::libs::thread_pipelines::SCHEDULE_TO_DISCORD_PIPELINE;
use crate::model::db::driver::DB_CONNECTION_POOL;
use crate::service::discord_error_msg::send_additional_log;
use crate::service::discord_error_msg::send_debug_error_log;
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
                            if let Err(err) = crate::discord::commands::$module::run(&context, &interaction).await {
                                LOGGER.log(LogLevel::Error, &format!("Discord > Error executing command {}: {:?}", stringify!($module), err));
                                return Err(err);
                            }
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

        static $activate_commands: LazyLock<Box<dyn Fn(String, &Context, &CommandInteraction) -> Pin<Box<dyn Future<Output = Result<String, serenity::Error>> + Send>> + Send + Sync>> = LazyLock::new(|| {
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
}
static DISCORD_SERVICE: OnceLock<rs_ervice::RSContext> = OnceLock::new();


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
    pub async fn new() -> Self {
        let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;
        let gemini_function_channel = &GEMINI_FUNCTION_EXECUTION_ALARM.receiver;
        let alarm_channel = &SCHEDULE_TO_DISCORD_PIPELINE.receiver;
        let client = Client::builder(token, intents)
                .event_handler(Handler)
                .voice_manager(
                    VoiceHandler {}
                )
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
        Self {
            client,
            gemini_function_channel,
            alarm_channel,
        }
    }
    pub async fn run(&mut self) {
        let mut fun_alarm_receiver = self.gemini_function_channel.clone();
        let mut alarm_channel = self.alarm_channel.clone();
        let client_control = self.client.http.clone();
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
                                LOGGER.log(LogLevel::Debug, &format!("Gemini function alarm received. {}", message.message_id));
                                let channel_id = ChannelId::new(message.channel_id.parse::<u64>().unwrap());
                                let target_user = UserId::new(message.sender.parse::<u64>().unwrap());
                                let msg = message.message.clone().result_message;
                                channel_id.send_message(client_control.clone(), CreateMessage::new()
                                    .content(format!("{} \n {}", target_user.mention(), msg))
                                    .embed(
                                        CreateEmbed::new()
                                            .title("Gemini Function Alarm")
                                            .description(msg)
                                            .footer(CreateEmbedFooter::new("time... : ".to_string() + &chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()))
                                    )
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
        if let Err(why) = self.client.start().await {
            println!("Client error: {:?}", why);
        }
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
        async { BotManager::new().await }
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
                        command.create_response(ctx, 
                            CreateInteractionResponse::Message(CreateInteractionResponseMessage::new()
                            .content("General Discord Bot Error")
                            .add_embed(
                                make_message_embed("Discord Bot Error", "General Discord Bot Error - Please contact the administrator: @lutica_canard").color(0xFF0000)
                            )
                    )).await.unwrap();
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

    
