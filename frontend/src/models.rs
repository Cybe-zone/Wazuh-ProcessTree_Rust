use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeColor {
    pub background: String,
    pub border: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessNode {
    pub id: String,
    pub label: String,
    pub process_id: i64,
    pub parent_id: Option<i64>,
    pub process_name: String,
    pub command_line: String,
    pub user: String,
    pub timestamp: String,
    pub classification: String,
    pub color: NodeColor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrphanProcess {
    pub process_id: i64,
    pub process_name: String,
    pub parent_id: Option<i64>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessTree {
    pub nodes: Vec<ProcessNode>,
    pub edges: Vec<ProcessEdge>,
    pub orphans: Vec<OrphanProcess>,
    pub total_processes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statistics {
    pub total_processes: usize,
    pub suspicious_count: usize,
    pub orphan_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub success: bool,
    pub data: Option<AnalysisData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisData {
    pub tree: ProcessTree,
    pub statistics: Statistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub agent_id: String,
    pub start_time: String,
    pub end_time: String,
    pub filters: Option<ProcessFilters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessFilters {
    pub process_name: Option<String>,
    pub user: Option<String>,
    pub command_line_contains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessClassification {
    pub id: String,
    pub name: String,
    pub patterns: String,
    pub color_bg: String,
    pub color_border: String,
    pub icon: Option<String>,
    pub priority: i32,
    pub enabled: bool,
}

impl ProcessClassification {
    pub fn get_patterns(&self) -> Vec<String> {
        self.patterns
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationsResponse {
    pub success: bool,
    pub data: Option<Vec<ProcessClassification>>,
}
