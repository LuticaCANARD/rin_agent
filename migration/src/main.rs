use migration::{Migrator, MigratorTrait};

#[tokio::main]
async fn main(){
    dotenv::dotenv().ok(); // Load environment variables from .env file

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connection = sea_orm::Database::connect(&database_url).await.expect("Failed to connect to the database");
    Migrator::up(&connection, None).await;
}