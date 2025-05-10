mod model; 
mod gemini; 
mod discord; 
mod api;
mod libs;
mod setting;
mod utils;
#[macro_use] extern crate rocket;
use api::routers::get_rocket;
use model::db::driver::connect_to_db;

use libs::logger::{self, LOGGER,LogLevel};
use tokio::task;
use tokio::signal;
use std::sync::Arc;
use tokio::sync::Notify;

async fn fn_discord_thread() {
    
    let mut discord_manager = discord::discord_bot_manager::BotManager::new().await;
    LOGGER.log(LogLevel::Debug, "Discord > Starting...");
    discord_manager.run().await;

}
async fn fn_aspect_thread(threads: Vec<task::JoinHandle<()>>) {

    let sigint_notify = Arc::new(Notify::new());
    let sigint_notify_clone = sigint_notify.clone();

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for SIGINT");
        LOGGER.log(LogLevel::Debug, "SIGINT received, shutting down...");
        sigint_notify_clone.notify_one();
    });

    sigint_notify.notified().await;
    LOGGER.log(LogLevel::Debug, "Waiting for all threads to finish...");
    for thread in threads {
        let _ = thread.await;
    }
    LOGGER.log(LogLevel::Debug, "All threads have finished.");
}

async fn fn_web_server_thread() {
    let _ = dotenv::dotenv();
    let _db_init_ = connect_to_db().await;

    LOGGER.log(LogLevel::Debug, "Web server > Starting...");
    get_rocket().launch().await.unwrap();
    LOGGER.log(LogLevel::Debug, "Web server > Stopped");

}
#[cfg(target_os = "linux")]
fn set_process_name(name: &str) {
    use std::ffi::CString;
    use libc;
    let cname = CString::new(name).unwrap();
    unsafe {
        libc::prctl(libc::PR_SET_NAME, cname.as_ptr() as usize, 0, 0, 0);
    }
}
#[tokio::main]
async fn main() {
    #[cfg(target_os = "linux")]
    set_process_name("rin_agent_main_server");

    let _ = dotenv::dotenv();
    let discord_thread = tokio::spawn(async move { fn_discord_thread().await });
    let web_server_thread = tokio::spawn(async move { fn_web_server_thread().await });

    let _db_init_ = connect_to_db().await;

    // TODO : 감시자 쓰레드를 만들고, 다른 쓰레드가 종료되면 감시자가 다시시작하던 하도록 한다.
    
    let threads_vector = vec![
        discord_thread,
        web_server_thread,
    ];
    fn_aspect_thread(threads_vector).await;

    LOGGER.log(LogLevel::Debug, "Starting Discord bot thread");
}
