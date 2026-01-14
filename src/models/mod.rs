use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};


// ============================================================================
// Authentication Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub indexer_host: String,
    pub indexer_port: i32,
    pub indexer_username: String,
    #[serde(skip_serializing)]
    pub indexer_password: String, // Encrypted in DB
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub indexer_host: String,
    pub indexer_port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub session_id: String,
    pub message: String,
}

// ============================================================================
// Process Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessEvent {
    pub timestamp: String,
    pub agent_id: String,
    pub agent_name: String,
    pub process_id: String,
    pub parent_process_id: String,
    pub process_name: String,
    pub process_path: String,
    pub command_line: String,
    pub user: String,
    pub user_domain: String,
    pub event_id: String,
    pub computer: String,
    pub rule_id: String,
    pub rule_description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessNode {
    pub id: String,
    pub label: String,
    pub title: String, // HTML tooltip - now properly escaped
    pub process_path: String,
    pub command_line: String,
    pub user: String,
    pub timestamp: String,
    pub parent_id: String,
    pub color: NodeColor,
    pub shape: String,
    pub font: NodeFont,
    pub classification: String, // Added for custom classification
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeColor {
    pub background: String,
    pub border: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeFont {
    pub size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessEdge {
    pub from: String,
    pub to: String,
    pub arrows: String,
    pub color: EdgeColor,
    pub width: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EdgeColor {
    pub color: String,
    pub highlight: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessTree {
    pub nodes: Vec<ProcessNode>,
    pub edges: Vec<ProcessEdge>,
    pub orphans: Vec<OrphanProcess>,
    pub total_processes: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrphanProcess {
    pub pid: String,
    pub name: String,
    pub missing_parent: String,
}

// ============================================================================
// Classification Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProcessClassification {
    pub id: String,
    pub name: String,
    pub patterns: String, // JSON array of patterns
    pub color_bg: String,
    pub color_border: String,
    pub icon: Option<String>,
    pub priority: i32, // Higher priority = checked first
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ProcessClassification {
    pub fn get_patterns(&self) -> Vec<String> {
        serde_json::from_str(&self.patterns).unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
pub struct ClassificationUpdateRequest {
    pub name: String,
    pub patterns: Vec<String>,
    pub color_bg: String,
    pub color_border: String,
    pub icon: Option<String>,
    pub priority: i32,
    pub enabled: bool,
}

// ============================================================================
// Filter Models
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct ProcessQueryRequest {
    pub agent_id: String,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub filters: Option<ProcessFilters>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessFilters {
    pub process_name: Option<String>,
    pub user: Option<String>,
    pub parent_process: Option<String>,
    pub command_line_contains: Option<String>,
    pub classification: Option<String>,
    pub exclude_system: Option<bool>,
}

// ============================================================================
// API Response Models
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: Utc::now(),
        }
    }
}

// ============================================================================
// Statistics Models
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ProcessStatistics {
    pub total_processes: usize,
    pub unique_users: usize,
    pub unique_parents: usize,
    pub suspicious_count: usize,
    pub time_range_hours: f64,
    pub top_processes: Vec<TopProcess>,
}

#[derive(Debug, Serialize)]
pub struct TopProcess {
    pub name: String,
    pub count: usize,
    pub classification: String,
}
