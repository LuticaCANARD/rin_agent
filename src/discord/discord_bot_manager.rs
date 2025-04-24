use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::prelude::*;
use serenity::async_trait;
use std::env;

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
    pub async fn run(mut self) {
        if let Err(why) = &self.client.start().await {
            println!("Client error: {:?}", why);
        }
    }
}


pub struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                println!("Error sending message: {:?}", why);
            }
        }
    }
}

    
