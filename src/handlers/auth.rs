use actix_web::{web, HttpResponse, HttpRequest};
use actix_session::Session;
use crate::models::{LoginRequest, LoginResponse, ApiResponse};
use crate::services::database::DatabaseService;
use crate::services::wazuh::WazuhClient;
use crate::error::AppError;
use uuid::Uuid;
use std::sync::Arc;

pub async fn login(
    req: HttpRequest,
    session: Session,
    data: web::Json<LoginRequest>,
    db: web::Data<Arc<DatabaseService>>,
) -> Result<HttpResponse, AppError> {
    let login_data = data.into_inner();

    // Test Wazuh connection
    let wazuh_client = WazuhClient::new(
        &login_data.indexer_host,
        login_data.indexer_port,
        login_data.username.clone(),
        login_data.password.clone(),
    )?;

    if !wazuh_client.test_connection().await? {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            "Failed to connect to Wazuh Indexer. Please check credentials.".to_string()
        )));
    }

    // Create session in database
    let user_id = Uuid::new_v4().to_string();
    let db_session = db.create_session(
        user_id,
        login_data.indexer_host.clone(),
        login_data.indexer_port as i32,
        login_data.username.clone(),
        login_data.password.clone(),
    ).await?;

    // Set session cookie
    session.insert("session_id", db_session.id.clone())
        .map_err(|e| AppError::Internal(format!("Session error: {}", e)))?;

    // Log audit
    let ip = req.connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    db.log_audit(
        db_session.id.clone(),
        "login".to_string(),
        Some(format!("Connected to {}", login_data.indexer_host)),
        ip,
        user_agent,
    ).await?;

    tracing::info!("User logged in with session: {}", db_session.id);

    Ok(HttpResponse::Ok().json(ApiResponse::success(LoginResponse {
        success: true,
        session_id: db_session.id,
        message: "Successfully connected to Wazuh Indexer".to_string(),
    })))
}

pub async fn logout(
    session: Session,
    db: web::Data<Arc<DatabaseService>>,
    req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    if let Ok(Some(session_id)) = session.get::<String>("session_id") {
        // Log audit before deleting session
        let ip = req.connection_info()
            .realip_remote_addr()
            .map(|s| s.to_string());

        let user_agent = req.headers()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        db.log_audit(
            session_id.clone(),
            "logout".to_string(),
            None,
            ip,
            user_agent,
        ).await.ok();

        // Delete session from database
        db.delete_session(&session_id).await?;

        tracing::info!("User logged out, session: {}", session_id);
    }

    // Clear session cookie
    session.purge();

    Ok(HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "message": "Logged out successfully"
    }))))
}

pub async fn check_session(
    session: Session,
    db: web::Data<Arc<DatabaseService>>,
) -> Result<HttpResponse, AppError> {
    if let Ok(Some(session_id)) = session.get::<String>("session_id") {
        if let Some(db_session) = db.get_session(&session_id).await? {
            // Check if session is expired
            if db_session.expires_at < chrono::Utc::now() {
                db.delete_session(&session_id).await?;
                session.purge();
                return Err(AppError::SessionExpired);
            }

            // Update last accessed time
            db.update_session_access(&session_id).await?;

            return Ok(HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
                "authenticated": true,
                "indexer_host": db_session.indexer_host
            }))));
        }
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "authenticated": false
    }))))
}