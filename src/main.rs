mod model; 
mod gemini; 
mod discord; 
mod api;
mod lib;


#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    let mut discord_manager = discord::discord_bot_manager::BotManager::new().await;
    
    discord_manager.run().await;
    println!("Hello, world!");

}
