use crate::models::{ProcessEvent, ProcessNode, ProcessEdge, ProcessTree, OrphanProcess, NodeColor, EdgeColor};
use crate::services::classification::ClassificationService;
use crate::error::AppError;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct ProcessTreeBuilder {
    classification_service: Arc<ClassificationService>,
}

impl ProcessTreeBuilder {
    pub fn new(classification_service: Arc<ClassificationService>) -> Self {
        Self {
            classification_service,
        }
    }

    pub async fn build_tree(&self, events: Vec<ProcessEvent>) -> Result<ProcessTree, AppError> {
        if events.is_empty() {
            return Ok(ProcessTree {
                nodes: vec![],
                edges: vec![],
                orphans: vec![],
                total_processes: 0,
            });
        }

        let mut processes: HashMap<String, ProcessNode> = HashMap::new();
        let mut parent_child_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut processed_edges: HashSet<(String, String)> = HashSet::new();

        // First pass: Create process nodes
        for event in &events {
            let process_id = event.process_id.clone();

            if !processes.contains_key(&process_id) {
                // Classify the process
                let (classification, color) = self.classification_service
                    .classify_process(event)
                    .await
                    .unwrap_or_else(|_| (
                        "Other".to_string(),
                        NodeColor {
                            background: "#9E9E9E".to_string(),
                            border: "#616161".to_string(),
                        }
                    ));

                // Parse process_id and parent_id as i64
                let pid_i64 = process_id.parse::<i64>().unwrap_or(0);
                let parent_id_i64 = event.parent_process_id.parse::<i64>().ok()
                    .filter(|&p| p != 0);

                let node = ProcessNode {
                    id: process_id.clone(),
                    label: event.process_name.clone(),
                    process_id: pid_i64,
                    parent_id: parent_id_i64,
                    process_name: event.process_name.clone(),
                    command_line: event.command_line.clone(),
                    user: event.user.clone(),
                    timestamp: event.timestamp.clone(),
                    classification,
                    color,
                };

                processes.insert(process_id.clone(), node);

                // Map parent-child relationships
                let parent_id = &event.parent_process_id;
                if !parent_id.is_empty() && parent_id != "0" {
                    parent_child_map
                        .entry(parent_id.clone())
                        .or_insert_with(Vec::new)
                        .push(process_id.clone());
                }
            }
        }

        // Second pass: Build edges
        let mut edges = Vec::new();

        for (_, node) in &processes {
            if let Some(parent_id_i64) = node.parent_id {
                let parent_id_str = parent_id_i64.to_string();
                if processes.contains_key(&parent_id_str) {
                    let edge_key = (parent_id_str.clone(), node.id.clone());
                    if !processed_edges.contains(&edge_key) {
                        edges.push(ProcessEdge {
                            from: parent_id_str.clone(),
                            to: node.id.clone(),
                            arrows: "to".to_string(),
                            color: EdgeColor {
                                color: "#848484".to_string(),
                                highlight: "#76b947".to_string(),
                            },
                            width: 2,
                        });
                        processed_edges.insert(edge_key);
                    }
                }
            }
        }

        // Find orphan processes
        let orphans = self.find_orphans(&processes);

        // Convert to vectors
        let nodes: Vec<ProcessNode> = processes.into_values().collect();
        let total_processes = nodes.len();

        Ok(ProcessTree {
            nodes,
            edges,
            orphans,
            total_processes,
        })
    }

    fn create_tooltip(&self, event: &ProcessEvent) -> String {
        // Create escaped HTML tooltip
        let process_name = html_escape(&event.process_name);
        let user_domain = html_escape(&event.user_domain);
        let user = html_escape(&event.user);
        let process_path = html_escape(&event.process_path);
        let command_line = html_escape(&event.command_line);
        let computer = html_escape(&event.computer);

        // Truncate command line if too long
        let cmd_display = if command_line.len() > 150 {
            format!("{}...", &command_line[..150])
        } else {
            command_line
        };

        format!(
            "<div style='font-family: monospace; text-align: left;'>\
                <b>Process:</b> {}<br/>\
                <b>PID:</b> {}<br/>\
                <b>Parent PID:</b> {}<br/>\
                <b>Path:</b> {}<br/>\
                <b>User:</b> {}\\{}<br/>\
                <b>Command:</b> {}<br/>\
                <b>Timestamp:</b> {}<br/>\
                <b>Computer:</b> {}<br/>\
            </div>",
            process_name,
            event.process_id,
            event.parent_process_id,
            process_path,
            user_domain,
            user,
            cmd_display,
            event.timestamp,
            computer
        )
    }

    fn find_orphans(&self, processes: &HashMap<String, ProcessNode>) -> Vec<OrphanProcess> {
        let mut orphans = Vec::new();

        for (_, node) in processes {
            if let Some(parent_id_i64) = node.parent_id {
                let parent_id_str = parent_id_i64.to_string();
                if !processes.contains_key(&parent_id_str) {
                    orphans.push(OrphanProcess {
                        process_id: node.process_id,
                        process_name: node.process_name.clone(),
                        parent_id: Some(parent_id_i64),
                        timestamp: node.timestamp.clone(),
                    });
                }
            }
        }

        orphans
    }

    pub fn get_process_chain(&self, processes: &HashMap<String, ProcessNode>, process_id: &str) -> Vec<String> {
        let mut chain = Vec::new();
        let mut current_id = process_id.to_string();
        let mut visited = HashSet::new();

        while let Some(process) = processes.get(&current_id) {
            if visited.contains(&current_id) {
                // Prevent infinite loops
                break;
            }

            chain.insert(0, current_id.clone());
            visited.insert(current_id.clone());

            match process.parent_id {
                Some(parent_id_i64) => {
                    current_id = parent_id_i64.to_string();
                }
                None => break,
            }
        }

        chain
    }

    pub fn calculate_statistics(&self, tree: &ProcessTree) -> crate::models::ProcessStatistics {
        let mut unique_users = HashSet::new();
        let mut unique_parents = HashSet::new();
        let mut suspicious_count = 0;
        let mut process_counts: HashMap<String, usize> = HashMap::new();

        for node in &tree.nodes {
            unique_users.insert(node.user.clone());
            if let Some(parent_id) = node.parent_id {
                unique_parents.insert(parent_id);
            }

            if node.classification == "Suspicious" {
                suspicious_count += 1;
            }

            *process_counts.entry(node.label.clone()).or_insert(0) += 1;
        }

        // Get top 10 processes
        let mut top_processes: Vec<_> = process_counts.into_iter().collect();
        top_processes.sort_by(|a, b| b.1.cmp(&a.1));
        top_processes.truncate(10);

        let top_processes = top_processes
            .into_iter()
            .map(|(name, count)| crate::models::TopProcess {
                name: name.clone(),
                count,
                classification: tree.nodes
                    .iter()
                    .find(|n| n.label == name)
                    .map(|n| n.classification.clone())
                    .unwrap_or_else(|| "Unknown".to_string()),
            })
            .collect();

        crate::models::ProcessStatistics {
            total_processes: tree.total_processes,
            unique_users: unique_users.len(),
            unique_parents: unique_parents.len(),
            suspicious_count,
            orphan_count: tree.orphans.len(),
            time_range_hours: 0.0, // To be calculated from timestamps
            top_processes,
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}
