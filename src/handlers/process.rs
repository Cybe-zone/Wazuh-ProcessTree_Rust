use actix_web::{web, HttpResponse, HttpRequest};
use actix_session::Session;
use crate::models::{ProcessQueryRequest, ApiResponse};
use crate::services::database::DatabaseService;
use crate::services::wazuh::WazuhClient;
use crate::services::process_tree::ProcessTreeBuilder;
use crate::services::classification::ClassificationService;
use crate::error::AppError;
use std::sync::Arc;


pub async fn analyze_process_tree(
    req: HttpRequest,
    session: Session,
    query: web::Json<ProcessQueryRequest>,
    db: web::Data<Arc<DatabaseService>>,
) -> Result<HttpResponse, AppError> {
    // Get session
    let session_id = session.get::<String>("session_id")
        .map_err(|_| AppError::Unauthorized)?
        .ok_or(AppError::Unauthorized)?;

    let db_session = db.get_session(&session_id)
        .await?
        .ok_or(AppError::SessionExpired)?;

    // Check session expiration
    if db_session.expires_at < chrono::Utc::now() {
        db.delete_session(&session_id).await?;
        session.purge();
        return Err(AppError::SessionExpired);
    }

    // Update session access time
    db.update_session_access(&session_id).await?;

    // Create Wazuh client with stored credentials
    let wazuh_client = WazuhClient::new(
        &db_session.indexer_host,
        db_session.indexer_port as u16,
        db_session.indexer_username.clone(),
        db_session.indexer_password.clone(),
    )?;

    // Log audit
    let ip = req.connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());
    
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    db.log_audit(
        session_id.clone(),
        "analyze_agent".to_string(),
        Some(format!(
            "Agent: {}, Time range: {:?} to {:?}",
            query.agent_id, query.start_time, query.end_time
        )),
        ip,
        user_agent,
    ).await?;

    tracing::info!("Analyzing agent {} for session {}", query.agent_id, session_id);

    // Fetch events from Wazuh
    let events = wazuh_client.get_process_events(&query).await?;

    tracing::info!("Retrieved {} events for agent {}", events.len(), query.agent_id);

    // Build process tree
    let classification_service = ClassificationService::new(Arc::clone(db.get_ref()));
    let tree_builder = ProcessTreeBuilder::new(Arc::new(classification_service));
    
    let process_tree = tree_builder.build_tree(events).await?;
    
    // Calculate statistics
    let statistics = tree_builder.calculate_statistics(&process_tree);

    tracing::info!(
        "Process tree built: {} nodes, {} edges, {} orphans",
        process_tree.nodes.len(),
        process_tree.edges.len(),
        process_tree.orphans.len()
    );

    // Debug logging to identify type mismatches
    if !process_tree.nodes.is_empty() {
        tracing::debug!("First node sample: {:?}", &process_tree.nodes[0]);
    }
    if !process_tree.orphans.is_empty() {
        tracing::debug!("First orphan sample: {:?}", &process_tree.orphans[0]);
    }

    let response_json = serde_json::json!({
        "tree": process_tree,
        "statistics": statistics
    });

    tracing::debug!("Response JSON (first 500 chars): {}",
        serde_json::to_string(&response_json)
            .unwrap_or_default()
            .chars()
            .take(500)
            .collect::<String>()
    );

    Ok(HttpResponse::Ok().json(ApiResponse::success(response_json)))
}

pub async fn export_process_tree(
    session: Session,
    data: web::Json<serde_json::Value>,
    db: web::Data<Arc<DatabaseService>>,
) -> Result<HttpResponse, AppError> {
    // Get session
    let session_id = session.get::<String>("session_id")
        .map_err(|_| AppError::Unauthorized)?
        .ok_or(AppError::Unauthorized)?;

    let db_session = db.get_session(&session_id)
        .await?
        .ok_or(AppError::SessionExpired)?;

    // Check session expiration
    if db_session.expires_at < chrono::Utc::now() {
        db.delete_session(&session_id).await?;
        session.purge();
        return Err(AppError::SessionExpired);
    }

    let export_data = data.into_inner();
    let format = export_data["format"].as_str().unwrap_or("json");

    match format {
        "json" => {
            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .json(ApiResponse::success(export_data["tree_data"].clone())))
        }
        "csv" => {
            // Convert tree data to CSV
            let nodes = export_data["tree_data"]["nodes"].as_array()
                .ok_or_else(|| AppError::BadRequest("Invalid tree data".to_string()))?;

            let mut csv_content = "ID,Label,User,Timestamp,Classification,Parent_ID\n".to_string();
            
            for node in nodes {
                csv_content.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    node["id"].as_str().unwrap_or(""),
                    node["label"].as_str().unwrap_or(""),
                    node["user"].as_str().unwrap_or(""),
                    node["timestamp"].as_str().unwrap_or(""),
                    node["classification"].as_str().unwrap_or(""),
                    node["parent_id"].as_str().unwrap_or("")
                ));
            }

            Ok(HttpResponse::Ok()
                .content_type("text/csv")
                .insert_header(("Content-Disposition", "attachment; filename=process_tree.csv"))
                .body(csv_content))
        }
        _ => Err(AppError::BadRequest("Unsupported export format".to_string()))
    }
}

pub async fn get_agent_info(
    session: Session,
    agent_id: web::Path<String>,
    db: web::Data<Arc<DatabaseService>>,
) -> Result<HttpResponse, AppError> {
    // Get session
    let session_id = session.get::<String>("session_id")
        .map_err(|_| AppError::Unauthorized)?
        .ok_or(AppError::Unauthorized)?;

    let db_session = db.get_session(&session_id)
        .await?
        .ok_or(AppError::SessionExpired)?;

    // Check session expiration
    if db_session.expires_at < chrono::Utc::now() {
        db.delete_session(&session_id).await?;
        session.purge();
        return Err(AppError::SessionExpired);
    }

    // Create Wazuh client
    let _wazuh_client = WazuhClient::new(
        &db_session.indexer_host,
        db_session.indexer_port as u16,
        db_session.indexer_username.clone(),
        db_session.indexer_password.clone(),
    )?;

    // Query for agent info (implement in WazuhClient if needed)
    // For now, return basic info
    Ok(HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "agent_id": agent_id.into_inner(),
        "connected": true
    }))))
}
