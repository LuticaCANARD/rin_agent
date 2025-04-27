mod model; 
mod gemini; 
mod discord; 
mod api;
mod libs;
use std::thread;

use libs::logger::{self, LOGGER,LogLevel};


async fn fn_discord_thread() {

    let buffer_size = 100; // 버퍼 크기 설정
    let mut discord_manager = discord::discord_bot_manager::BotManager::new(buffer_size).await;
    LOGGER.log(LogLevel::Debug, "Starting...");
    discord_manager.run().await;

}

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    
    let discord_thread = thread::spawn(move || fn_discord_thread());
    // TODO : 감시자 쓰레드를 만들고, 다른 쓰레드가 종료되면 감시자가 다시시작하던 하도록 한다.
    let _ = discord_thread.join().unwrap().await;

    LOGGER.log(LogLevel::Debug, "Starting Discord bot thread");


}
