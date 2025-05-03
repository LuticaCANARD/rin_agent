use serenity::all::Guild;
use serenity::all::UnavailableGuild;
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::model::gateway::GatewayIntents;
use serenity::model::application::{Command, Interaction};
use std::env;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use crate::libs::thread_pipelines::AsyncThreadPipeline;

use crate::libs::logger::{LOGGER, LogLevel};

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


    }
    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: Option<bool>) {
        println!("Guild created: {:?}, is_new: {:?}", guild.id, is_new);
        
        if(is_new.unwrap_or(true)){
            // Register commands here

        }
    }
    async fn guild_delete(&self, ctx: Context, guild: UnavailableGuild, full_guild: Option<Guild>) {
        println!("Guild deleted: {:?}", guild.id);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let command_name = command.data.name.clone();
                let options = command.data.options.clone();
                println!("Received command: {}", command_name);
                // Handle the command here
                // For example, you can call a function to process the command
                // and send a response back to the user.
            }
            _ => {}
        }
    }
}

    
