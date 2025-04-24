mod model; 
mod gemini; 
mod discord; 
mod api;
mod libs;
use std::thread;

use libs::logger::{self, LOGGER,LogLevel};


async fn fn_discord_thread() {
    let mut discord_manager = discord::discord_bot_manager::BotManager::new().await;
    discord_manager.run().await;
}

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();

    let discord_thread = thread::spawn(fn_discord_thread);
    
    
    LOGGER.log(LogLevel::Debug, "Starting Discord bot thread");


}
