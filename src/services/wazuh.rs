use crate::models::{ProcessEvent, ProcessQueryRequest};
use crate::error::AppError;
use reqwest::{Client, ClientBuilder};
use serde_json::{json, Value};
use std::time::Duration;
use tracing::{info, error};

pub struct WazuhClient {
    client: Client,
    base_url: String,
    username: String,
    password: String,
}

impl WazuhClient {
    pub fn new(host: &str, port: u16, username: String, password: String) -> Result<Self, AppError> {
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(true) // For self-signed certs
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        let base_url = format!("https://{}:{}", host, port);

        Ok(Self {
            client,
            base_url,
            username,
            password,
        })
    }

    pub async fn test_connection(&self) -> Result<bool, AppError> {
        let url = format!("{}/_cluster/health", self.base_url);
        
        let response = self.client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| AppError::WazuhConnection(e.to_string()))?;

        Ok(response.status().is_success())
    }

    pub async fn get_process_events(
        &self,
        request: &ProcessQueryRequest,
    ) -> Result<Vec<ProcessEvent>, AppError> {
        let url = format!("{}/wazuh-alerts-*/_search", self.base_url);
        
        // Build query with flexible event ID matching
        let mut must_conditions = vec![
            json!({"term": {"agent.id": request.agent_id}})
        ];

        // Add time range if provided
        if let (Some(start), Some(end)) = (&request.start_time, &request.end_time) {
            must_conditions.push(json!({
                "range": {
                    "timestamp": {
                        "gte": start,
                        "lte": end,
                        "format": "strict_date_optional_time"
                    }
                }
            }));
        }

        let query = json!({
            "size": 10000,
            "sort": [{"timestamp": {"order": "asc"}}],
            "query": {
                "bool": {
                    "must": must_conditions,
                    "should": [
                        {"term": {"data.win.system.eventID": "4688"}},
                        {"term": {"data.win.eventdata.eventID": "4688"}},
                        {"term": {"win.system.eventID": "4688"}},
                        {"term": {"data.eventID": "4688"}},
                        {"match": {"rule.id": "60103"}},
                        {"match": {"rule.id": "18105"}},
                    ],
                    "minimum_should_match": 1
                }
            }
        });

        info!("Querying Wazuh for agent {} with query: {}", request.agent_id, query);

        let response = self.client
            .post(&url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&query)
            .send()
            .await
            .map_err(|e| AppError::WazuhQuery(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Wazuh query failed: {} - {}", status, text);
            return Err(AppError::WazuhQuery(format!("Query failed: {}", status)));
        }

        let data: Value = response.json()
            .await
            .map_err(|e| AppError::ParseError(e.to_string()))?;

        let hits = data["hits"]["hits"]
            .as_array()
            .ok_or_else(|| AppError::ParseError("Invalid response format".to_string()))?;

        info!("Retrieved {} raw events from Wazuh", hits.len());

        let mut events = Vec::new();
        for hit in hits {
            if let Some(event) = self.normalize_event(&hit["_source"]) {
                events.push(event);
            }
        }

        info!("Successfully normalized {} events", events.len());

        // Apply filters if provided
        if let Some(filters) = &request.filters {
            events = self.apply_filters(events, filters);
        }

        Ok(events)
    }

    fn normalize_event(&self, source: &Value) -> Option<ProcessEvent> {
        // Try multiple possible field paths
        let win_data = source.get("data")?.get("win")
            .or_else(|| source.get("win"))?;

        let event_data = win_data.get("eventdata")?;
        let system_data = win_data.get("system")?;

        // Get event ID
        let event_id = system_data.get("eventID")
            .or_else(|| event_data.get("eventID"))
            .or_else(|| source.get("data")?.get("eventID"))
            .and_then(|v| v.as_str().or_else(|| v.as_i64().map(|n| {
                if n == 4688 { "4688" } else { "" }
            })))?;

        if event_id != "4688" {
            return None;
        }

        // Extract process IDs
        let mut process_id = event_data.get("newProcessId")
            .or_else(|| event_data.get("processId"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        let mut parent_process_id = event_data.get("processId")
            .or_else(|| event_data.get("parentProcessId"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        // Convert hex to decimal
        if process_id.starts_with("0x") {
            process_id = self.hex_to_decimal(&process_id);
        }
        if parent_process_id.starts_with("0x") {
            parent_process_id = self.hex_to_decimal(&parent_process_id);
        }

        let process_path = event_data.get("newProcessName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let process_name = process_path.split('\\')
            .last()
            .unwrap_or(&process_path)
            .to_string();

        if process_id.is_empty() || process_name.is_empty() {
            return None;
        }

        Some(ProcessEvent {
            timestamp: source.get("timestamp")
                .or_else(|| source.get("@timestamp"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            agent_id: source.get("agent")?.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            agent_name: source.get("agent")?.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            process_id,
            parent_process_id: if parent_process_id.is_empty() { "0".to_string() } else { parent_process_id },
            process_name,
            process_path,
            command_line: event_data.get("commandLine")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            user: event_data.get("subjectUserName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            user_domain: event_data.get("subjectDomainName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            event_id: "4688".to_string(),
            computer: system_data.get("computer")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            rule_id: source.get("rule")?.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            rule_description: source.get("rule")?.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
    }

    fn hex_to_decimal(&self, hex_str: &str) -> String {
        if let Some(hex) = hex_str.strip_prefix("0x") {
            if let Ok(num) = i64::from_str_radix(hex, 16) {
                return num.to_string();
            }
        }
        hex_str.to_string()
    }

    fn apply_filters(&self, mut events: Vec<ProcessEvent>, filters: &crate::models::ProcessFilters) -> Vec<ProcessEvent> {
        if let Some(name) = &filters.process_name {
            events.retain(|e| e.process_name.to_lowercase().contains(&name.to_lowercase()));
        }

        if let Some(user) = &filters.user {
            events.retain(|e| e.user.to_lowercase().contains(&user.to_lowercase()));
        }

        if let Some(cmd) = &filters.command_line_contains {
            events.retain(|e| e.command_line.to_lowercase().contains(&cmd.to_lowercase()));
        }

        if filters.exclude_system.unwrap_or(false) {
            let system_processes = vec!["system", "smss.exe", "csrss.exe", "wininit.exe"];
            events.retain(|e| {
                !system_processes.iter().any(|sp| e.process_name.to_lowercase() == sp.to_lowercase())
            });
        }

        events
    }
}
