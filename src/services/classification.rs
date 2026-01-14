use crate::models::{ProcessClassification, NodeColor, ProcessEvent};
use crate::services::database::DatabaseService;
use crate::error::AppError;
use std::sync::Arc;
use moka::future::Cache;
use std::time::Duration;

pub struct ClassificationService {
    db: Arc<DatabaseService>,
    cache: Cache<String, Vec<ProcessClassification>>,
}

impl ClassificationService {
    pub fn new(db: Arc<DatabaseService>) -> Self {
        let cache = Cache::builder()
            .max_capacity(100)
            .time_to_live(Duration::from_secs(300))
            .build();

        Self { db, cache }
    }

    pub async fn get_all_classifications(&self) -> Result<Vec<ProcessClassification>, AppError> {
        // Try cache first
        if let Some(cached) = self.cache.get(&"all".to_string()).await {
            return Ok(cached);
        }

        // Load from database
        let classifications = self.db.get_all_classifications().await?;

        // Update cache
        self.cache.insert("all".to_string(), classifications.clone()).await;

        Ok(classifications)
    }

    pub async fn classify_process(&self, event: &ProcessEvent) -> Result<(String, NodeColor), AppError> {
        let classifications = self.get_all_classifications().await?;

        // Sort by priority (higher first)
        let mut sorted = classifications;
        sorted.sort_by(|a, b| b.priority.cmp(&a.priority));

        for classification in sorted.iter().filter(|c| c.enabled) {
            let patterns = classification.get_patterns();

            for pattern in patterns {
                let pattern_lower = pattern.to_lowercase();

                // Check process name
                if event.process_name.to_lowercase().contains(&pattern_lower) {
                    return Ok((
                        classification.name.clone(),
                        NodeColor {
                            background: classification.color_bg.clone(),
                            border: classification.color_border.clone(),
                        }
                    ));
                }

                // Check command line
                if event.command_line.to_lowercase().contains(&pattern_lower) {
                    return Ok((
                        classification.name.clone(),
                        NodeColor {
                            background: classification.color_bg.clone(),
                            border: classification.color_border.clone(),
                        }
                    ));
                }
            }
        }

        // Default classification
        Ok((
            "Other".to_string(),
            NodeColor {
                background: "#9E9E9E".to_string(),
                border: "#616161".to_string(),
            }
        ))
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
        self.db.update_classification(
            id,
            name,
            patterns,
            color_bg,
            color_border,
            icon,
            priority,
            enabled,
        ).await?;

        // Invalidate cache
        self.cache.invalidate(&"all".to_string()).await;

        Ok(())
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
        let id = self.db.create_classification(
            name,
            patterns,
            color_bg,
            color_border,
            icon,
            priority,
        ).await?;

        // Invalidate cache
        self.cache.invalidate(&"all".to_string()).await;

        Ok(id)
    }

    pub async fn delete_classification(&self, id: &str) -> Result<(), AppError> {
        self.db.delete_classification(id).await?;

        // Invalidate cache
        self.cache.invalidate(&"all".to_string()).await;

        Ok(())
    }

    pub async fn initialize_default_classifications(&self) -> Result<(), AppError> {
        let defaults = vec![
            ("System Process", vec!["system", "smss.exe", "csrss.exe", "wininit.exe", "services.exe", "lsass.exe", "svchost.exe"], "#10b981", "#059669", 100),
            ("Shell/Script", vec!["powershell", "cmd.exe", "wscript", "cscript", "bash", "sh"], "#f59e0b", "#d97706", 90),
            ("Browser", vec!["chrome", "firefox", "edge", "iexplore", "brave", "opera"], "#3b82f6", "#2563eb", 80),
            ("Suspicious", vec!["encoded", "bypass", "hidden", "downloadstring", "invoke-expression", "base64", "mimikatz"], "#ef4444", "#dc2626", 150),
            ("Development Tools", vec!["code", "devenv", "idea", "pycharm", "visual studio"], "#8b5cf6", "#7c3aed", 70),
            ("Office Apps", vec!["winword", "excel", "powerpnt", "outlook", "teams"], "#06b6d4", "#0891b2", 60),
        ];

        for (name, patterns, bg, border, priority) in defaults {
            // Check if already exists
            let existing = self.get_all_classifications().await?;
            if !existing.iter().any(|c| c.name == name) {
                self.create_classification(
                    name.to_string(),
                    patterns.iter().map(|s| s.to_string()).collect(),
                    bg.to_string(),
                    border.to_string(),
                    None,
                    priority,
                ).await?;
            }
        }

        Ok(())
    }
}