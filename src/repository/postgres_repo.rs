use tracing::error;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::{env, time::Duration};

use dotenv::dotenv;

pub struct PostgresRepo {
    pub db: DatabaseConnection,
}

impl PostgresRepo {
    pub async fn init() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            error!("Error loading env variable DATABASE_URL");
            std::process::exit(1);
        });
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Info)
            .set_schema_search_path("public");

        let db = Database::connect(opt).await.unwrap_or_else(|_| {
            error!("Failed to initialize client");
            std::process::exit(1);
        });

        PostgresRepo { db }
    }
}
