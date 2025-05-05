mod model; 
mod gemini; 
mod discord; 
mod api;
mod libs;
mod setting;
mod utils;
use std::ops::Deref;
use std::{process::Output, thread};
use crate::libs::thread_pipelines::AsyncThreadPipeline;
use crate::libs::thread_message::{DiscordToGeminiMessage};
use gemini::GEMINI_CLIENT;
use model::db::driver::connect_to_db;
use sea_orm::Database;
use tokio::sync::Mutex; // Ensure Mutex is imported
use libs::thread_pipelines::DISCORD_TO_GEMINI_PIPELINE;
use gemini::gemini_client::GeminiClientTrait; // Ensure the trait is in scope
use tokio::sync::watch::Ref;


use libs::logger::{self, LOGGER,LogLevel};
use tokio::task;


async fn fn_discord_thread() {
    
    let mut discord_manager = discord::discord_bot_manager::BotManager::new().await;
    LOGGER.log(LogLevel::Debug, "Discord > Starting...");
    discord_manager.run().await;

}
async fn fn_aspect_thread(threads: Vec<task::JoinHandle<()>>) {
    loop  {
        // TODO : 감시자 쓰레드 구현
        // 감시자 쓰레드는 다른 쓰레드가 종료되면 다시 시작하도록 한다.
        // TODO : 감시자 쓰레드는 종료된 쓰레드를 재시작하는 기능을 구현한다.
        let mut end_thread_count:usize = 0;
        for thread in threads.iter() {
            if thread.is_finished() {
                end_thread_count += 1;
                LOGGER.log(LogLevel::Debug, "Thread finished");
            }
        }
        if end_thread_count == threads.len() {
            LOGGER.log(LogLevel::Debug, "All threads finished, restarting...");
            break;
        }
        
    }
}

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    
    let discord_thread = tokio::spawn(async move { fn_discord_thread().await });

    connect_to_db().await;
    // TODO : 감시자 쓰레드를 만들고, 다른 쓰레드가 종료되면 감시자가 다시시작하던 하도록 한다.
    
    let threads_vector = vec![discord_thread];
    fn_aspect_thread(threads_vector).await;

    LOGGER.log(LogLevel::Debug, "Starting Discord bot thread");
}
