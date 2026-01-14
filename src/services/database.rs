use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use crate::models::{Session, ProcessClassification};
use crate::error::AppError;
use base64::{Engine as _, engine::general_purpose};
use chrono::{Utc, Duration};
use uuid::Uuid;

pub struct DatabaseService {
    pool: SqlitePool,
}

impl DatabaseService {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(20)
            .connect(database_url)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<(), AppError> {
        // Create sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                indexer_host TEXT NOT NULL,
                indexer_port INTEGER NOT NULL,
                indexer_username TEXT NOT NULL,
                indexer_password TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL,
                last_accessed TEXT NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Create classifications table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS classifications (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                patterns TEXT NOT NULL,
                color_bg TEXT NOT NULL,
                color_border TEXT NOT NULL,
                icon TEXT,
                priority INTEGER NOT NULL DEFAULT 0,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Create audit logs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_logs (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                action TEXT NOT NULL,
                details TEXT,
                ip_address TEXT,
                user_agent TEXT,
                timestamp TEXT NOT NULL
            )
            "#
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Create index on sessions
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_sessions_expires ON sessions(expires_at)")
            .execute(&self.pool)
            .await
            .ok();

        Ok(())
    }

    // ========================================================================
    // Session Management
    // ========================================================================

    pub async fn create_session(
        &self,
        user_id: String,
        indexer_host: String,
        indexer_port: i32,
        indexer_username: String,
        indexer_password: String,
    ) -> Result<Session, AppError> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::hours(2);

        let session = Session {
            id: session_id.clone(),
            user_id,
            indexer_host,
            indexer_port,
            indexer_username,
            indexer_password: self.encrypt_password(&indexer_password),
            created_at: now,
            expires_at,
            last_accessed: now,
        };

        sqlx::query(
            r#"
            INSERT INTO sessions
            (id, user_id, indexer_host, indexer_port, indexer_username,
             indexer_password, created_at, expires_at, last_accessed)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&session.id)
        .bind(&session.user_id)
        .bind(&session.indexer_host)
        .bind(session.indexer_port)
        .bind(&session.indexer_username)
        .bind(&session.indexer_password)
        .bind(session.created_at.to_rfc3339())
        .bind(session.expires_at.to_rfc3339())
        .bind(session.last_accessed.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(session)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>, AppError> {
        let result = sqlx::query_as::<_, (String, String, String, i32, String, String, String, String, String)>(
            "SELECT id, user_id, indexer_host, indexer_port, indexer_username,
                    indexer_password, created_at, expires_at, last_accessed
             FROM sessions WHERE id = ?"
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(|(id, user_id, host, port, username, password, created, expires, accessed)| {
            Session {
                id,
                user_id,
                indexer_host: host,
                indexer_port: port,
                indexer_username: username,
                indexer_password: self.decrypt_password(&password),
                created_at: created.parse().unwrap_or(Utc::now()),
                expires_at: expires.parse().unwrap_or(Utc::now()),
                last_accessed: accessed.parse().unwrap_or(Utc::now()),
            }
        }))
    }

    pub async fn update_session_access(&self, session_id: &str) -> Result<(), AppError> {
        let now = Utc::now();

        sqlx::query("UPDATE sessions SET last_accessed = ? WHERE id = ?")
            .bind(now.to_rfc3339())
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(session_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();

        sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // Classification Management
    // ========================================================================

    pub async fn get_all_classifications(&self) -> Result<Vec<ProcessClassification>, AppError> {
        let rows = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, i32, i32, String, String)>(
            "SELECT id, name, patterns, color_bg, color_border, icon, priority, enabled,
                    created_at, updated_at
             FROM classifications
             ORDER BY priority DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|(id, name, patterns, bg, border, icon, priority, enabled, created, updated)| {
            ProcessClassification {
                id,
                name,
                patterns,
                color_bg: bg,
                color_border: border,
                icon,
                priority,
                enabled: enabled == 1,
                created_at: created.parse().unwrap_or(Utc::now()),
                updated_at: updated.parse().unwrap_or(Utc::now()),
            }
        }).collect())
    }

    pub async fn get_classification(&self, id: &str) -> Result<Option<ProcessClassification>, AppError> {
        let result = sqlx::query_as::<_, (String, String, String, String, String, Option<String>, i32, i32, String, String)>(
            "SELECT id, name, patterns, color_bg, color_border, icon, priority, enabled,
                    created_at, updated_at
             FROM classifications
             WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(result.map(|(id, name, patterns, bg, border, icon, priority, enabled, created, updated)| {
            ProcessClassification {
                id,
                name,
                patterns,
                color_bg: bg,
                color_border: border,
                icon,
                priority,
                enabled: enabled == 1,
                created_at: created.parse().unwrap_or(Utc::now()),
                updated_at: updated.parse().unwrap_or(Utc::now()),
            }
        }))
    }

    pub async fn create_classification(
        &self,
        name: String,
        patterns: Vec<String>,
        color_bg: String,
        color_border: String,
        icon: Option<String>,
        priority: i32,
    ) -> Result<String, AppError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let patterns_json = serde_json::to_string(&patterns)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO classifications
            (id, name, patterns, color_bg, color_border, icon, priority, enabled, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?, ?)
            "#
        )
        .bind(&id)
        .bind(&name)
        .bind(&patterns_json)
        .bind(&color_bg)
        .bind(&color_border)
        .bind(&icon)
        .bind(priority)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(id)
    }

    pub async fn update_classification(
        &self,
        id: &str,
        name: String,
        patterns: Vec<String>,
        color_bg: String,
        color_border: String,
        icon: Option<String>,
        priority: i32,
        enabled: bool,
    ) -> Result<(), AppError> {
        let now = Utc::now().to_rfc3339();
        let patterns_json = serde_json::to_string(&patterns)
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        sqlx::query(
            r#"
            UPDATE classifications
            SET name = ?, patterns = ?, color_bg = ?, color_border = ?,
                icon = ?, priority = ?, enabled = ?, updated_at = ?
            WHERE id = ?
            "#
        )
        .bind(&name)
        .bind(&patterns_json)
        .bind(&color_bg)
        .bind(&color_border)
        .bind(&icon)
        .bind(priority)
        .bind(if enabled { 1 } else { 0 })
        .bind(&now)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn delete_classification(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM classifications WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // Audit Logging
    // ========================================================================

    pub async fn log_audit(
        &self,
        session_id: String,
        action: String,
        details: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<(), AppError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO audit_logs
            (id, session_id, action, details, ip_address, user_agent, timestamp)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&id)
        .bind(&session_id)
        .bind(&action)
        .bind(&details)
        .bind(&ip_address)
        .bind(&user_agent)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    // ========================================================================
    // Encryption Helpers (Basic - use proper encryption in production)
    // ========================================================================

    fn encrypt_password(&self, password: &str) -> String {
        // In production, use proper encryption like AES-GCM
        // This is a simple base64 encoding for demonstration
        general_purpose::STANDARD.encode(password)
    }

    fn decrypt_password(&self, encrypted: &str) -> String {
        // In production, use proper decryption
        general_purpose::STANDARD.decode(encrypted)
           .ok()
           .and_then(|bytes| String::from_utf8(bytes).ok())
           .unwrap_or_default()
    }
}
