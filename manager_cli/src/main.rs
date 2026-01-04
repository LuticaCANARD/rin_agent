use contract::config::{EnvConfigBuilder, ManagerConfig};
use contract::{ ManagerResponse };
use futures_util::StreamExt;
use futures_util::lock::Mutex;
use redis::aio::{MultiplexedConnection, PubSub};
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use colored::Colorize;

mod command;

const COMMAND_CHANNEL: &str = "manager:commands";
const RESPONSE_CHANNEL: &str = "manager:responses";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments for environment loading strategy
    let strategy = contract::config::parse_env_strategy_from_args();
    let dotenv_path = contract::config::parse_dotenv_path_from_args();
    
    // Load environment variables with specified strategy
    let mut builder = EnvConfigBuilder::new()
        .strategy(strategy)
        .ignore_missing(true);
    
    if let Some(path) = dotenv_path {
        builder = builder.dotenv_path(path);
    }
    
    builder.load()?;
    
    // Build configuration from environment variables
    let config = ManagerConfig::from_env()?;
    
    println!("{}", "╔════════════════════════════════════════╗".bright_cyan());
    println!("{}", "║   RinAgent Manager CLI - Client Mode   ║".bright_cyan());
    println!("{}", "╚════════════════════════════════════════╝".bright_cyan());
    println!();
    println!("{}", "Connecting to Valkey/Redis...".yellow());
    
    // Initialize Redis client
    let redis_client = redis::Client::open(config.common.redis_url.clone())?;
    let redis_conn = redis_client.get_multiplexed_async_connection().await?;
    let redis_conn = Arc::new(tokio::sync::Mutex::new(redis_conn));
    
    println!("{}", "✓ Connected to manager service\n".green());
    
    // Channel for communication between CLI and Redis handler
    let (tx, mut rx) = mpsc::channel::<ManagerResponse>(100);
    
    // Spawn Redis pubsub listener
    let redis_url = config.common.redis_url.clone();
    tokio::spawn(async move {
        handle_redis_responses(redis_url, tx).await;
    });
    
    // Spawn response printer
    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            print_response(&response);
        }
    });
    
    // Run interactive CLI
    run_interactive_cli(redis_conn).await?;
    
    Ok(())
}

async fn handle_redis_responses(
    redis: Arc<Mutex<PubSub>>,
    tx: mpsc::Sender<ManagerResponse>,
) {
    // 채널 구독
    let mut pubsub_conn = redis.lock().await;

    if let Err(e) = pubsub_conn.subscribe(RESPONSE_CHANNEL).await {
        eprintln!("{} {}", "Redis subscribe error:".red(), e);
        return;
    }
    
    println!("{}", format!("✓ Subscribed to '{}'", RESPONSE_CHANNEL).bright_black());// 메시지 스트림 처리
    loop {
        let msg = pubsub_conn.on_message().next().await;
        if let Some(msg) = msg {
            
        }
    }
    
}

fn print_response(response: &ManagerResponse) {
    match response {
        ManagerResponse::HealthReport {
            cpu_usage,
            memory_usage_percent,
            total_memory_mb,
            used_memory_mb,
            timestamp,
        } => {
            println!("\n{}", "=== System Health Report ===".bright_blue());
            println!("  CPU Usage: {}%", format!("{:.2}", cpu_usage).yellow());
            println!(
                "  Memory: {}% ({} MB / {} MB)",
                format!("{:.2}", memory_usage_percent).yellow(),
                used_memory_mb,
                total_memory_mb
            );
            println!("  Timestamp: {}", timestamp);
        }
        ManagerResponse::ProcessStatus {
            process_name,
            is_running,
            pid,
            timestamp,
        } => {
            if *is_running {
                println!(
                    "{} Process '{}' is running (PID: {:?})",
                    "✓".green(),
                    process_name.bright_white(),
                    pid
                );
            } else {
                println!(
                    "{} Process '{}' is not running",
                    "✗".red(),
                    process_name.bright_white()
                );
            }
        }
        ManagerResponse::Success {
            command,
            message,
            data,
        } => {
            println!("{} {}: {}", "✓".green(), command.bright_white(), message);
            if let Some(data) = data {
                println!("  Data: {}", serde_json::to_string_pretty(data).unwrap());
            }
        }
        ManagerResponse::Error { command, error } => {
            eprintln!("{} {}: {}", "✗".red(), command.bright_white(), error);
        }
    }
}

async fn run_interactive_cli(
    redis_conn: Arc<tokio::sync::Mutex<MultiplexedConnection>>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Type 'help' for available commands\n".bright_black());
    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    loop {
        // Print prompt
        print!("{} ", "manager>".bright_green().bold());
        stdout.flush()?;
        
        // Read input
        let mut input = String::new();
        stdin.read_line(&mut input)?;
        
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        
        // Parse command
        let parts: Vec<&str> = input.split_whitespace().collect();
        let command = parts[0].to_lowercase();
        
        match command.as_str() {
            "help" | "h" | "?" => {
                print_help();
            }
            "exit" | "quit" | "q" => {
                println!("{}", "Disconnecting from manager...".yellow());
                break;
            }
            _ => {
                eprintln!(
                    "{} Unknown command: '{}'. Type 'help' for available commands.",
                    "✗".red(),
                    input
                );
            }
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("\n{}", "=== Available Commands ===".bright_blue());
    println!("  {} - Show this help message", "help, h, ?".bright_white());
    println!("  {} - Request system health status", "status, s".bright_white());
    println!("  {} - Request system information", "info, i".bright_white());
    println!(
        "  {} - Restart a process",
        "restart, r <name>".bright_white()
    );
    println!(
        "  {} - Start monitoring a process",
        "monitor, m <name> [interval]".bright_white()
    );
    println!(
        "  {} - Stop monitoring a process",
        "unmonitor, u <name>".bright_white()
    );
    println!("  {} - Exit the CLI", "exit, quit, q".bright_white());
    println!();
}