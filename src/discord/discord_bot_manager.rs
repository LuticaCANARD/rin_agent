use serenity::all::CreateCommand;
use serenity::all::Guild;
use serenity::all::GuildId;
use serenity::all::UnavailableGuild;
use serenity::client::Context;
use serenity::prelude::*;
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::gateway::GatewayIntents;
use serenity::model::application::{Command, Interaction};

use std::env;
use serenity::model::prelude::*;
use std::pin::Pin;
use std::future::Future;
use crate::libs::logger::{LOGGER, LogLevel};
use std::sync::LazyLock;
use tokio::signal;

use super::commands::gemini_query;

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
                            }
                            let response = format!("Discord > Command {} executed successfully!", stringify!($module));
                            LOGGER.log(LogLevel::Info, &response);
                            response
                        }
                    )*
                    _ => {
                        LOGGER.log(LogLevel::Error, &format!("Discord > Unknown command: {}", name));
                        "Unknown command".to_string()
                    }
                }
            })
        }
    };
}

static USING_COMMANDS: LazyLock<Vec<CreateCommand>> = LazyLock::new(|| {
    register_commands_module!{
        ping,
        gemini_query
    }
});

static USING_ACTIVATE_COMMANDS: LazyLock<Box<dyn Fn(String, &Context, &CommandInteraction) -> Pin<Box<dyn Future<Output = String> + Send>> + Send + Sync>> = LazyLock::new(|| {
    Box::new(get_command_function!(
        ping,
        gemini_query))
});




static CLIENT_ID: LazyLock<Option<UserId>> = LazyLock::new(|| (std::env::var("DISCORD_CLIENT_ID").ok()).and_then(|id| id.parse::<u64>().ok()).map(UserId::new));

async fn register_commands(ctx: Context, guild_id: GuildId) {
    // Register commands here
    let commands = USING_COMMANDS.clone();
    guild_id.set_commands(&ctx.http, commands.clone()).await.unwrap();
}
pub struct BotManager {
    client: Client,
}


impl BotManager {
    pub async fn new() -> Self {
        let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;
        Self {
            client: Client::builder(token, intents)
                .event_handler(Handler)
                .await
                .expect("Error creating client"),
        }
    }
    pub async fn run(&mut self) {

        tokio::spawn(async move {
            if let Err(err) = signal::ctrl_c().await {
                LOGGER.log(LogLevel::Error, &format!("Failed to listen for SIGINT: {:?}", err));
            } else {
                LOGGER.log(LogLevel::Info, "SIGINT received, shutting down...");
                std::process::exit(0);
            }
        });
        if let Err(why) = self.client.start().await {
            println!("Client error: {:?}", why);
        }
    }
}
pub struct Handler;
#[async_trait]
impl EventHandler for Handler {
    //https://github.com/serenity-rs/serenity/blob/current/examples/e14_slash_commands/src/main.rs
    async fn ready(&self, ctx: Context, _ready: Ready) {
        // Delete remaining commands and register new ones
        
        // Register commands here
        let _guild_id:u64 = 1026747872508653568; // Replace with your guild ID
        
        let guild_id = GuildId::new(_guild_id);
        // let guild = ctx.http.get_guild(guild_id).await.unwrap();
        let commands = ctx.http.get_guild_commands(guild_id).await.unwrap();
        for command in commands {
            ctx.http.delete_guild_command(guild_id, command.id).await.unwrap();
        }
        // Register commands
        register_commands(ctx.clone(), guild_id).await;
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
        let commands = ctx.http.get_guild_commands(guild_id).await.unwrap();
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
                    gemini_query::continue_query(&ctx, &msg,&msg.author).await;
                }
            }
        }


    }
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let command_name = command.data.name.clone();
                println!("Received command: {}", command_name);
                // Handle the command here
                // For example, you can call a function to process the command
                // and send a response back to the user.
                let command_future = &USING_ACTIVATE_COMMANDS;
            
                // 클로저 호출 후 `.await`
                let _ = command_future(command_name, &ctx, &command).await;

            }
            Interaction::Ping(ping) =>{
                LOGGER.log(LogLevel::Debug, "Discord > Ping interaction received");
            }
            _ => {
                
            }
        }
    }
    
}

    
