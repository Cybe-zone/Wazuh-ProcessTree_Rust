use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub secret_key: String,
    pub log_level: String,
    pub max_connections: u32,
    pub rate_limit_requests: u64,
    pub rate_limit_duration_secs: u64,
    pub cache_ttl_secs: u64,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        Ok(Self {
            host: std::env::var("HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "5443".to_string())
                .parse()
                .context("Invalid PORT")?,
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://wptv.db".to_string()),
            secret_key: std::env::var("SECRET_KEY")
                .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string()),
            log_level: std::env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
            max_connections: std::env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "100".to_string())
                .parse()
                .unwrap_or(100),
            rate_limit_requests: std::env::var("RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
            rate_limit_duration_secs: std::env::var("RATE_LIMIT_DURATION")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            cache_ttl_secs: std::env::var("CACHE_TTL")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .unwrap_or(300),
        })
    }
}
