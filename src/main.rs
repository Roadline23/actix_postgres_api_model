pub mod api;
pub mod emails;
pub mod error;
pub mod middlewares;
pub mod repository;
pub mod sms;
pub mod types;
pub mod utils;

use actix_cors::Cors;
use actix_web::{
    http::header,
    middleware::{DefaultHeaders, Logger},
    web::{self, Data},
    App, HttpServer,
};
use api::routes::init_auth_pro_routes;
use dotenv::dotenv;
use migration::{Migrator, MigratorTrait};
use repository::postgres_repo::PostgresRepo;
use tracing::{error, event};
use crate::middlewares::{check_auth_middleware::Auth, app_state::AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();
    dotenv().ok();
    let host = std::env::var("HOST").expect("HOST must be set");
    let port = std::env::var("PORT").expect("PORT must be set");
    let addr = format!("{}:{}", host, port);
    let connection = PostgresRepo::init().await;
    Migrator::up(&connection.db, None)
        .await
        .unwrap_or_else(|_| {
            error!("Failed to migrate");
            std::process::exit(1);
        });

    let db_data = Data::new(connection.db);
    let app_state_data = Data::new(AppState::new());
    event!(tracing::Level::INFO, "Server running on {}", addr);
    HttpServer::new(move || {
        App::new()
            .app_data(db_data.clone())
            .app_data(app_state_data.clone())
            .wrap(DefaultHeaders::new().add(("X-Powered-By", "Focus")))
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::ORIGIN])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(Logger::default())
            .service(web::scope("/auth/pro").configure(init_auth_pro_routes))
    })
    .bind(addr)?
    .run()
    .await
}
