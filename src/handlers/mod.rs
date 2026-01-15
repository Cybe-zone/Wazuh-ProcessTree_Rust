pub mod auth;
pub mod process;
pub mod classification;

use actix_web::{web, HttpResponse};
use actix_files::NamedFile;
use std::path::PathBuf;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Health check
        .route("/api/health", web::get().to(health_check))
        
        // Authentication routes
        .route("/api/auth/login", web::post().to(auth::login))
        .route("/api/auth/logout", web::post().to(auth::logout))
        .route("/api/auth/check", web::get().to(auth::check_session))
        
        // Process analysis routes
        .route("/api/process/analyze", web::post().to(process::analyze_process_tree))
        .route("/api/process/export", web::post().to(process::export_process_tree))
        .route("/api/process/agent/{id}", web::get().to(process::get_agent_info))
        
        // Classification routes
        .route("/api/classifications", web::get().to(classification::get_all_classifications))
        .route("/api/classifications", web::post().to(classification::create_classification))
        .route("/api/classifications/{id}", web::get().to(classification::get_classification))
        .route("/api/classifications/{id}", web::put().to(classification::update_classification))
        .route("/api/classifications/{id}", web::delete().to(classification::delete_classification))
        .route("/api/classifications/defaults", web::post().to(classification::initialize_defaults))
        
        // Frontend routes
        .route("/", web::get().to(index))
        .route("/login", web::get().to(login_page))
        .route("/dashboard", web::get().to(dashboard_page))
        .route("/settings", web::get().to(settings_page));
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": "3.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn index() -> actix_web::Result<NamedFile> {
    let path: PathBuf = "templates/login.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn login_page() -> actix_web::Result<NamedFile> {
    let path: PathBuf = "templates/login.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn dashboard_page() -> actix_web::Result<NamedFile> {
    // Serve the WASM frontend
    let path: PathBuf = "static/wasm/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn settings_page() -> actix_web::Result<NamedFile> {
    let path: PathBuf = "templates/settings.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}
