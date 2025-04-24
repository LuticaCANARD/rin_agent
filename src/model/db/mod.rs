use sea_orm::{Database, DatabaseConnection}; 
pub mod entity; 

pub async fn connect_to_db() -> DatabaseConnection {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Database::connect(&db_url)
    .await
    .expect("Failed to connect to the database")
}
