use sea_orm::{Database, DatabaseConnection};
use tokio::sync::OnceCell;

pub static DB_CONNECTION_POOL: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub async fn connect_to_db() -> &'static DatabaseConnection {
    DB_CONNECTION_POOL
        .get_or_init(|| async {
            let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
            Database::connect(&db_url)
                .await
                .expect("Failed to connect to the database")
        })
        .await
}