use actix_web::{web, HttpResponse};
use actix_session::Session;
use crate::models::{ClassificationUpdateRequest, ApiResponse};
use crate::services::database::DatabaseService;
use crate::services::classification::ClassificationService;
use crate::error::AppError;


pub async fn get_all_classifications(
    session: Session,
    db: web::Data<DatabaseService>,
) -> Result<HttpResponse, AppError> {
    // Verify session
    let _session_id = session.get::<String>("session_id")
        .map_err(|_| AppError::Unauthorized)?
        .ok_or(AppError::Unauthorized)?;

    let classification_service = ClassificationService::new(db.clone());
    let classifications = classification_service.get_all_classifications().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(classifications)))
}

pub async fn get_classification(
    session: Session,
    id: web::Path<String>,
    db: web::Data<DatabaseService>,
) -> Result<HttpResponse, AppError> {
    // Verify session
    let _session_id = session.get::<String>("session_id")
        .map_err(|_| AppError::Unauthorized)?
        .ok_or(AppError::Unauthorized)?;

    let classification = db.get_classification(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("Classification not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(classification)))
}

pub async fn create_classification(
    session: Session,
    data: web::Json<ClassificationUpdateRequest>,
    db: web::Data<DatabaseService>,
) -> Result<HttpResponse, AppError> {
    // Verify session
    let session_id = session.get::<String>("session_id")
        .map_err(|_| AppError::Unauthorized)?
        .ok_or(AppError::Unauthorized)?;

    let classification_service = ClassificationService::new(db.clone());
    
    let id = classification_service.create_classification(
        data.name.clone(),
        data.patterns.clone(),
        data.color_bg.clone(),
        data.color_border.clone(),
        data.icon.clone(),
        data.priority,
    ).await?;

    // Log audit
    db.log_audit(
        session_id,
        "create_classification".to_string(),
        Some(format!("Created classification: {}", data.name)),
        None,
        None,
    ).await?;

    tracing::info!("Created classification: {} ({})", data.name, id);

    Ok(HttpResponse::Created().json(ApiResponse::success(serde_json::json!({
        "id": id,
        "message": "Classification created successfully"
    }))))
}

pub async fn update_classification(
    session: Session,
    id: web::Path<String>,
    data: web::Json<ClassificationUpdateRequest>,
    db: web::Data<DatabaseService>,
) -> Result<HttpResponse, AppError> {
    // Verify session
    let session_id = session.get::<String>("session_id")
        .map_err(|_| AppError::Unauthorized)?
        .ok_or(AppError::Unauthorized)?;

    let classification_service = ClassificationService::new(db.clone());
    
    classification_service.update_classification(
        &id,
        data.name.clone(),
        data.patterns.clone(),
        data.color_bg.clone(),
        data.color_border.clone(),
        data.icon.clone(),
        data.priority,
        data.enabled,
    ).await?;

    // Log audit
    db.log_audit(
        session_id,
        "update_classification".to_string(),
        Some(format!("Updated classification: {}", data.name)),
        None,
        None,
    ).await?;

    tracing::info!("Updated classification: {} ({})", data.name, id);

    Ok(HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "message": "Classification updated successfully"
    }))))
}

pub async fn delete_classification(
    session: Session,
    id: web::Path<String>,
    db: web::Data<DatabaseService>,
) -> Result<HttpResponse, AppError> {
    // Verify session
    let session_id = session.get::<String>("session_id")
        .map_err(|_| AppError::Unauthorized)?
        .ok_or(AppError::Unauthorized)?;

    let classification_service = ClassificationService::new(db.clone());
    
    // Get classification name before deletion for audit
    let classification = db.get_classification(&id)
        .await?
        .ok_or_else(|| AppError::NotFound("Classification not found".to_string()))?;

    classification_service.delete_classification(&id).await?;

    // Log audit
    db.log_audit(
        session_id,
        "delete_classification".to_string(),
        Some(format!("Deleted classification: {}", classification.name)),
        None,
        None,
    ).await?;

    tracing::info!("Deleted classification: {} ({})", classification.name, id);

    Ok(HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "message": "Classification deleted successfully"
    }))))
}

pub async fn initialize_defaults(
    session: Session,
    db: web::Data<DatabaseService>,
) -> Result<HttpResponse, AppError> {
    // Verify session
    let _session_id = session.get::<String>("session_id")
        .map_err(|_| AppError::Unauthorized)?
        .ok_or(AppError::Unauthorized)?;

    let classification_service = ClassificationService::new(db.clone());
    classification_service.initialize_default_classifications().await?;

    tracing::info!("Initialized default classifications");

    Ok(HttpResponse::Ok().json(ApiResponse::success(serde_json::json!({
        "message": "Default classifications initialized"
    }))))
}
