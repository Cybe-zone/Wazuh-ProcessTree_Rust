use actix_web::{web, App, HttpServer};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_files::Files;
use actix_cors::Cors;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod models;
mod handlers;
mod services;
mod middleware;
use crate::middleware as custom_middleware;
mod error;
mod utils;

use config::AppConfig;
use services::database::DatabaseService;
use services::classification::ClassificationService;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "wazuh_process_tree_viewer=info,actix_web=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Wazuh Process Tree Viewer v3.0");

    // Load configuration
    let config = AppConfig::from_env()
        .expect("Failed to load configuration");

    // Initialize database
    let db = DatabaseService::new(&config.database_url)
        .await
        .expect("Failed to initialize database");

    db.run_migrations()
        .await
        .expect("Failed to run migrations");

    let db_arc = Arc::new(db);
    let db_data = web::Data::new(db_arc.clone());

    // Initialize default classifications
    tracing::info!("Initializing default process classifications...");
    let classification_service = ClassificationService::new(db_arc.clone());
    classification_service
        .initialize_default_classifications()
        .await
        .expect("Failed to initialize default classifications");
    tracing::info!("Default classifications loaded successfully");

    // Generate secure session key
    let secret_key = actix_web::cookie::Key::from(config.secret_key.as_bytes());

    let bind_address = format!("{}:{}", config.host, config.port);
    tracing::info!("Server starting on https://{}", bind_address);

    // Start HTTP server
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&format!("https://{}:{}", config.host, config.port))
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .max_age(3600);

        App::new()
            .app_data(db_data.clone())
            .app_data(web::Data::new(config.clone()))
            // Session middleware
            .wrap(
                SessionMiddleware::new(
                    CookieSessionStore::default(),
                    secret_key.clone()
                )
            )            
            // Middlewares
            .wrap(cors)
            .wrap(actix_web::middleware::Logger::default())
            .wrap(actix_web::middleware::Compress::default())
            .wrap(custom_middleware::SecurityHeaders)
            // Static files with proper MIME types
            .service(
                Files::new("/wasm", "./static/wasm")
                    .use_last_modified(true)
                    .prefer_utf8(true)
            )
            .service(
                Files::new("/static", "./static")
                    .use_last_modified(true)
                    .prefer_utf8(true)
            )
            // API routes
            .configure(handlers::configure_routes)
    })
    .bind(&bind_address)?
    .workers(4)
    .run()
    .await
}
