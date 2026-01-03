mod libs;
mod command;
mod service;

use contract::config::{EnvConfigBuilder, ManagerConfig};
use service::manager_service::ManagerService;

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

    println!("Starting RinAgent Manager Service...");

    // Build configuration from environment variables using contract
    let config = ManagerConfig::from_env()?;

    // Initialize the manager service
    let service = ManagerService::new(config).await?;

    println!("Manager service initialized successfully");

    // Start the service
    service.start().await?;

    println!("Manager service started. Press Ctrl+C to stop.");

    // Keep the service running
    tokio::signal::ctrl_c().await?;

    println!("Shutting down manager service...");

    Ok(())
}

