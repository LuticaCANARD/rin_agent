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
use crate::libs::thread_pipelines::AsyncThreadPipeline;
use serenity::model::prelude::*;
use crate::libs::logger::{LOGGER, LogLevel};

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
                                eprintln!("Error executing command {}: {:?}", stringify!($module), err);
                            }
                        }
                    )*
                    _ => {
                        println!("Unknown command: {}", name);
                    }
                }
            })
        };
    };
}

async fn register_commands(ctx: Context, guild_id: GuildId) {
    // Register commands here
    let commands = register_commands_module!{
        ping
    };
    guild_id.set_commands(ctx.http.clone(), commands.clone()).await.unwrap();
}
pub struct BotManager {
    client: Client,
    thread_message_pipeline_to_ai:AsyncThreadPipeline<String>,
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
            thread_message_pipeline_to_ai: AsyncThreadPipeline::new(), // 버퍼 크기 설정
        }
    }
    pub async fn run(&mut self) {
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
        
        println!("Bot is connected and ready!");

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


    }
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        println!("Guild created: {:?}, is_new: {:?}", guild.id, is_new);
        
        if is_new.unwrap_or(true) {
            // Register commands here
            register_commands(ctx.clone(), guild.id).await;
        }
    }


    async fn guild_delete(&self, ctx: Context, guild: UnavailableGuild, full_guild: Option<Guild>) {
        println!("Guild deleted: {:?}", guild.id);
        let guild_id = guild.id;
        let commands = ctx.http.get_guild_commands(guild_id).await.unwrap();
        for command in commands {
            ctx.http.delete_guild_command(guild_id, command.id).await.unwrap();
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
                let command_future = get_command_function! {
                    ping
                };
            
                // 클로저 호출 후 `.await`
                command_future(command_name, &ctx, &command).await;

                
            }
            _ => {}
        }
    }
    
}

    
