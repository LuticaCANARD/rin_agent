mod model; 
mod gemini; 
mod discord; 
mod api;
mod libs;
mod setting;
mod utils;
mod web;
mod service;
#[cfg(test)] mod tests;

use api::instances::init_rin_services;
use contract::config::{EnvConfigBuilder, RinAgentConfig};
use discord::discord_bot_manager::{get_discord_service, BotManager};
use service::discord_error_msg::send_additional_log;
use web::server::server::get_rocket;
use model::db::driver::connect_to_db;
use libs::logger::{self, LOGGER,LogLevel};
use tokio::task;
use tokio::signal;
use std::sync::Arc;
use tokio::sync::Notify;

async fn fn_discord_thread() {
    let discord_service = get_discord_service().await;
    let bot_manager_result = discord_service.call::<BotManager>();
    
    if let Some(bot_manager_mutex) = bot_manager_result {
        let mut bot_manager = bot_manager_mutex.lock().await;
        
        LOGGER.log(LogLevel::Debug, "Discord > Starting...");
        
        match bot_manager.run().await {
            Ok(_) => {
                LOGGER.log(LogLevel::Info, "Discord > Bot finished normally");
            },
            Err(e) => {
                LOGGER.log(LogLevel::Error, &format!("Discord > Bot run error: {:?}", e));
                send_additional_log(format!("Discord bot run error: {:?}", e), None).await;
            }
        }
    } else {
        LOGGER.log(LogLevel::Error, "Discord > Failed to get BotManager");
        send_additional_log("Failed to get Discord BotManager".to_string(), None).await;
    }
}
async fn fn_aspect_thread(threads: Vec<task::JoinHandle<()>>) {

    let sigint_notify = Arc::new(Notify::new());
    let sigint_notify_clone = sigint_notify.clone();

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for SIGINT");
        LOGGER.log(LogLevel::Debug, "SIGINT received, shutting down...");
        let is_dev = if cfg!(debug_assertions) {
            "dev"
        } else {
            "prod"
        };
        tokio::task::spawn_blocking(move || send_additional_log(format!("{} > SIGINT received, shutting down...",is_dev).to_string(),None));
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

    // Load environment variables with CLI-specified strategy
    let strategy = contract::config::parse_env_strategy_from_args();
    let dotenv_path = contract::config::parse_dotenv_path_from_args();
    
    let mut builder = EnvConfigBuilder::new()
        .strategy(strategy)
        .ignore_missing(true);
    
    if let Some(path) = dotenv_path {
        builder = builder.dotenv_path(path);
    }
    
    if let Err(e) = builder.load() {
        eprintln!("Failed to load environment variables: {}", e);
        std::process::exit(1);
    }

    // Load RinAgent configuration from environment
    let config = match RinAgentConfig::from_env() {
        Ok(cfg) => {
            LOGGER.log(LogLevel::Info, "Configuration loaded successfully");
            cfg
        }
        Err(e) => {
            eprintln!("Failed to load RinAgent configuration: {}", e);
            LOGGER.log(LogLevel::Error, &format!("Configuration load failed: {}", e));
            std::process::exit(1);
        }
    };

    // Initialize database (once)
    let _db_init_ = connect_to_db().await;
    
    // Initialize services
    init_rin_services().await;
    
    let startup_msg = "Rin Agent Main Server started";
    LOGGER.log(LogLevel::Info, startup_msg);
    send_additional_log(startup_msg.to_string(), None).await;
    let discord_thread = tokio::spawn(async move { fn_discord_thread().await });
    let web_server_thread = tokio::spawn(async move { fn_web_server_thread().await });


    // TODO : 감시자 쓰레드를 만들고, 다른 쓰레드가 종료되면 감시자가 다시시작하던 하도록 한다.
    
    let threads_vector = vec![
        discord_thread,
        web_server_thread,
    ];
    fn_aspect_thread(threads_vector).await;

    LOGGER.log(LogLevel::Debug, "Starting Discord bot thread");
}
