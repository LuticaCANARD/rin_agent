mod model; 
mod gemini; 
mod discord; 
mod api;
mod libs;
mod setting;
mod utils;
use model::db::driver::connect_to_db;

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

    let db_cnn = connect_to_db().await;
    let _pool = db_cnn.get_postgres_connection_pool();


    // TODO : 감시자 쓰레드를 만들고, 다른 쓰레드가 종료되면 감시자가 다시시작하던 하도록 한다.
    
    let threads_vector = vec![discord_thread];
    fn_aspect_thread(threads_vector).await;

    LOGGER.log(LogLevel::Debug, "Starting Discord bot thread");
}
