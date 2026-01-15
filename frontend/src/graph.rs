use crate::models::{ProcessNode, ProcessEdge};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, MouseEvent, WheelEvent};

#[derive(Clone, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug)]
pub struct NodePosition {
    pub node: ProcessNode,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

pub struct GraphRenderer {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    nodes: Vec<NodePosition>,
    edges: Vec<ProcessEdge>,

    // Pan & Zoom state
    offset_x: f64,
    offset_y: f64,
    scale: f64,

    // Interaction state
    is_dragging: bool,
    drag_start: Point,
    last_mouse_pos: Point,

    // Selection
    selected_node: Option<String>,
    hovered_node: Option<String>,
}

impl GraphRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        let context = canvas
            .get_context("2d")
            .map_err(|_| "Failed to get 2d context")?
            .ok_or("Context is null")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "Failed to cast to 2d context")?;

        Ok(Self {
            canvas,
            context,
            nodes: Vec::new(),
            edges: Vec::new(),
            offset_x: 0.0,
            offset_y: 50.0,
            scale: 1.0,
            is_dragging: false,
            drag_start: Point { x: 0.0, y: 0.0 },
            last_mouse_pos: Point { x: 0.0, y: 0.0 },
            selected_node: None,
            hovered_node: None,
        })
    }

    pub fn set_data(&mut self, nodes: Vec<ProcessNode>, edges: Vec<ProcessEdge>) {
        // Build hierarchy layout
        let positions = self.layout_hierarchy(&nodes, &edges);
        self.nodes = positions;
        self.edges = edges;

        // Center the graph
        if !self.nodes.is_empty() {
            let canvas_width = self.canvas.width() as f64;
            let canvas_height = self.canvas.height() as f64;

            // Find bounds
            let mut min_x = f64::MAX;
            let mut max_x = f64::MIN;
            let mut min_y = f64::MAX;
            let mut max_y = f64::MIN;

            for node_pos in &self.nodes {
                min_x = min_x.min(node_pos.x);
                max_x = max_x.max(node_pos.x + node_pos.width);
                min_y = min_y.min(node_pos.y);
                max_y = max_y.max(node_pos.y + node_pos.height);
            }

            let graph_width = max_x - min_x;
            let graph_height = max_y - min_y;

            // Center on first render
            self.offset_x = (canvas_width - graph_width) / 2.0 - min_x;
            self.offset_y = (canvas_height - graph_height) / 2.0 - min_y;
        }
    }

    fn layout_hierarchy(&self, nodes: &[ProcessNode], edges: &[ProcessEdge]) -> Vec<NodePosition> {
        let mut positions = Vec::new();
        let mut node_map: HashMap<String, &ProcessNode> = HashMap::new();
        let mut children_map: HashMap<String, Vec<String>> = HashMap::new();

        // Build maps
        for node in nodes {
            node_map.insert(node.id.clone(), node);
        }

        for edge in edges {
            children_map
                .entry(edge.from.clone())
                .or_insert_with(Vec::new)
                .push(edge.to.clone());
        }

        // Find root nodes (nodes with no incoming edges)
        let mut has_parent = std::collections::HashSet::new();
        for edge in edges {
            has_parent.insert(edge.to.clone());
        }

        let roots: Vec<String> = nodes
            .iter()
            .filter(|n| !has_parent.contains(&n.id))
            .map(|n| n.id.clone())
            .collect();

        // Layout parameters
        let node_width = 180.0;
        let node_height = 80.0;
        let horizontal_spacing = 40.0;
        let vertical_spacing = 100.0;

        let mut level_positions: HashMap<usize, f64> = HashMap::new();
        let mut current_x = 0.0;

        // Recursive layout
        for root_id in roots {
            current_x = self.layout_subtree(
                &root_id,
                current_x,
                0.0,
                0,
                node_width,
                node_height,
                horizontal_spacing,
                vertical_spacing,
                &node_map,
                &children_map,
                &mut positions,
                &mut level_positions,
            );
            current_x += horizontal_spacing * 2.0;
        }

        positions
    }

    fn layout_subtree(
        &self,
        node_id: &str,
        x: f64,
        _y: f64,
        level: usize,
        node_width: f64,
        node_height: f64,
        h_spacing: f64,
        v_spacing: f64,
        node_map: &HashMap<String, &ProcessNode>,
        children_map: &HashMap<String, Vec<String>>,
        positions: &mut Vec<NodePosition>,
        level_positions: &mut HashMap<usize, f64>,
    ) -> f64 {
        if let Some(node) = node_map.get(node_id) {
            let node_y = level as f64 * (node_height + v_spacing);

            // Get children
            let children = children_map.get(node_id).cloned().unwrap_or_default();

            if children.is_empty() {
                // Leaf node
                positions.push(NodePosition {
                    node: (*node).clone(),
                    x,
                    y: node_y,
                    width: node_width,
                    height: node_height,
                });
                return x + node_width + h_spacing;
            } else {
                // Layout children first
                let mut child_x = x;
                let mut child_positions = Vec::new();

                for child_id in &children {
                    let child_end_x = self.layout_subtree(
                        child_id,
                        child_x,
                        node_y + node_height + v_spacing,
                        level + 1,
                        node_width,
                        node_height,
                        h_spacing,
                        v_spacing,
                        node_map,
                        children_map,
                        positions,
                        level_positions,
                    );
                    child_positions.push((child_x, child_end_x));
                    child_x = child_end_x;
                }

                // Center parent above children
                let first_child_x = child_positions.first().map(|(x, _)| *x).unwrap_or(x);
                let last_child_x = child_positions.last().map(|(_, x)| *x).unwrap_or(x);
                let parent_x = (first_child_x + last_child_x - node_width) / 2.0;

                positions.push(NodePosition {
                    node: (*node).clone(),
                    x: parent_x,
                    y: node_y,
                    width: node_width,
                    height: node_height,
                });

                return last_child_x;
            }
        }
        x
    }

    pub fn render(&mut self, is_light_mode: bool) {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;

        // Clear canvas
        let bg_color = if is_light_mode { "#f5f5f5" } else { "#0a0a0a" };
        self.context.set_fill_style(&bg_color.into());
        self.context.fill_rect(0.0, 0.0, width, height);

        // Save context state
        self.context.save();

        // Apply transformations
        self.context.translate(self.offset_x, self.offset_y).ok();
        self.context.scale(self.scale, self.scale).ok();

        // Draw edges first
        let edge_color = if is_light_mode { "#999999" } else { "#666666" };
        self.context.set_stroke_style(&edge_color.into());
        self.context.set_line_width(2.0);

        for edge in &self.edges {
            if let (Some(from), Some(to)) = (
                self.nodes.iter().find(|n| n.node.id == edge.from),
                self.nodes.iter().find(|n| n.node.id == edge.to),
            ) {
                self.context.begin_path();
                let from_x = from.x + from.width / 2.0;
                let from_y = from.y + from.height;
                let to_x = to.x + to.width / 2.0;
                let to_y = to.y;

                // Cubic bezier for smooth curves
                let control_offset = (to_y - from_y) / 2.0;
                self.context.move_to(from_x, from_y);
                self.context.bezier_curve_to(
                    from_x,
                    from_y + control_offset,
                    to_x,
                    to_y - control_offset,
                    to_x,
                    to_y,
                );
                self.context.stroke();

                // Draw arrow
                self.draw_arrow(to_x, to_y, 0.0, -1.0);
            }
        }

        // Draw nodes
        let text_color = if is_light_mode { "#1a1a1a" } else { "#ffffff" };

        for node_pos in &self.nodes {
            let is_selected = self.selected_node.as_ref() == Some(&node_pos.node.id);
            let is_hovered = self.hovered_node.as_ref() == Some(&node_pos.node.id);

            // Draw node rectangle
            self.context.set_fill_style(&node_pos.node.color.background.clone().into());

            if is_selected || is_hovered {
                self.context.set_stroke_style(&"#76b947".into());
                self.context.set_line_width(3.0);
            } else {
                self.context.set_stroke_style(&node_pos.node.color.border.clone().into());
                self.context.set_line_width(2.0);
            }

            self.context.fill_rect(node_pos.x, node_pos.y, node_pos.width, node_pos.height);
            self.context.stroke_rect(node_pos.x, node_pos.y, node_pos.width, node_pos.height);

            // Draw text
            self.context.set_fill_style(&text_color.into());
            self.context.set_font("14px 'Segoe UI', sans-serif");
            self.context.set_text_align("center");

            // Process name (truncate if too long)
            let name = if node_pos.node.process_name.len() > 20 {
                format!("{}...", &node_pos.node.process_name[..17])
            } else {
                node_pos.node.process_name.clone()
            };

            self.context
                .fill_text(
                    &name,
                    node_pos.x + node_pos.width / 2.0,
                    node_pos.y + 25.0,
                )
                .ok();

            // PID
            self.context.set_font("12px 'Segoe UI', sans-serif");
            self.context
                .fill_text(
                    &format!("PID: {}", node_pos.node.process_id),
                    node_pos.x + node_pos.width / 2.0,
                    node_pos.y + 45.0,
                )
                .ok();

            // User
            let user = if node_pos.node.user.len() > 15 {
                format!("{}...", &node_pos.node.user[..12])
            } else {
                node_pos.node.user.clone()
            };

            self.context
                .fill_text(
                    &user,
                    node_pos.x + node_pos.width / 2.0,
                    node_pos.y + 62.0,
                )
                .ok();
        }

        // Restore context
        self.context.restore();
    }

    fn draw_arrow(&self, x: f64, y: f64, dx: f64, dy: f64) {
        let arrow_size = 8.0;
        let angle = dy.atan2(dx);

        self.context.begin_path();
        self.context.move_to(x, y);
        self.context.line_to(
            x - arrow_size * (angle - std::f64::consts::PI / 6.0).cos(),
            y - arrow_size * (angle - std::f64::consts::PI / 6.0).sin(),
        );
        self.context.move_to(x, y);
        self.context.line_to(
            x - arrow_size * (angle + std::f64::consts::PI / 6.0).cos(),
            y - arrow_size * (angle + std::f64::consts::PI / 6.0).sin(),
        );
        self.context.stroke();
    }

    pub fn handle_mouse_down(&mut self, event: &MouseEvent) {
        let x = event.offset_x() as f64;
        let y = event.offset_y() as f64;

        log::info!("Mouse down at ({}, {})", x, y);

        // Check if clicking on a node
        let clicked_node = self.get_node_at(x, y);

        if clicked_node.is_some() {
            log::info!("Clicked on node: {:?}", clicked_node);
            self.selected_node = clicked_node;
        } else {
            // Start dragging
            log::info!("Starting drag from ({}, {})", x, y);
            self.is_dragging = true;
            self.drag_start = Point { x, y };
            self.last_mouse_pos = Point { x, y };
        }
    }

    pub fn handle_mouse_move(&mut self, event: &MouseEvent) {
        let x = event.offset_x() as f64;
        let y = event.offset_y() as f64;

        if self.is_dragging {
            // Pan the view
            let dx = x - self.last_mouse_pos.x;
            let dy = y - self.last_mouse_pos.y;

            log::info!("Dragging: dx={}, dy={}, new offset=({}, {})", dx, dy, self.offset_x + dx, self.offset_y + dy);

            self.offset_x += dx;
            self.offset_y += dy;

            self.last_mouse_pos = Point { x, y };
        } else {
            // Update hover state
            self.hovered_node = self.get_node_at(x, y);
        }
    }

    pub fn handle_mouse_up(&mut self, _event: &MouseEvent) {
        log::info!("Mouse up - stopping drag");
        self.is_dragging = false;
    }

    pub fn handle_wheel(&mut self, event: &WheelEvent) {
        event.prevent_default();
        event.stop_propagation();

        let delta = event.delta_y();
        let zoom_factor = if delta < 0.0 { 1.1 } else { 0.9 };

        log::info!("Wheel event: delta={}, zoom_factor={}, current_scale={}", delta, zoom_factor, self.scale);

        let mouse_x = event.offset_x() as f64;
        let mouse_y = event.offset_y() as f64;

        // Zoom towards mouse position
        let old_scale = self.scale;
        self.scale *= zoom_factor;
        self.scale = self.scale.clamp(0.1, 5.0);

        log::info!("Zoom: old_scale={}, new_scale={}", old_scale, self.scale);

        let scale_change = self.scale / old_scale;
        self.offset_x = mouse_x - (mouse_x - self.offset_x) * scale_change;
        self.offset_y = mouse_y - (mouse_y - self.offset_y) * scale_change;
    }

    fn get_node_at(&self, screen_x: f64, screen_y: f64) -> Option<String> {
        // Transform screen coordinates to world coordinates
        let world_x = (screen_x - self.offset_x) / self.scale;
        let world_y = (screen_y - self.offset_y) / self.scale;

        for node_pos in &self.nodes {
            if world_x >= node_pos.x
                && world_x <= node_pos.x + node_pos.width
                && world_y >= node_pos.y
                && world_y <= node_pos.y + node_pos.height
            {
                return Some(node_pos.node.id.clone());
            }
        }
        None
    }

    pub fn get_selected_node(&self) -> Option<&ProcessNode> {
        self.selected_node.as_ref().and_then(|id| {
            self.nodes
                .iter()
                .find(|n| n.node.id == *id)
                .map(|n| &n.node)
        })
    }

    pub fn fit_view(&mut self) {
        if self.nodes.is_empty() {
            return;
        }

        let canvas_width = self.canvas.width() as f64;
        let canvas_height = self.canvas.height() as f64;

        // Find bounds
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;

        for node_pos in &self.nodes {
            min_x = min_x.min(node_pos.x);
            max_x = max_x.max(node_pos.x + node_pos.width);
            min_y = min_y.min(node_pos.y);
            max_y = max_y.max(node_pos.y + node_pos.height);
        }

        let graph_width = max_x - min_x;
        let graph_height = max_y - min_y;

        // Calculate scale to fit
        let padding = 50.0;
        let scale_x = (canvas_width - padding * 2.0) / graph_width;
        let scale_y = (canvas_height - padding * 2.0) / graph_height;
        self.scale = scale_x.min(scale_y).min(1.0);

        // Center
        self.offset_x = (canvas_width - graph_width * self.scale) / 2.0 - min_x * self.scale;
        self.offset_y = (canvas_height - graph_height * self.scale) / 2.0 - min_y * self.scale;
    }

    pub fn zoom_in(&mut self) {
        let canvas_width = self.canvas.width() as f64;
        let canvas_height = self.canvas.height() as f64;
        let center_x = canvas_width / 2.0;
        let center_y = canvas_height / 2.0;

        let old_scale = self.scale;
        self.scale = (self.scale * 1.2).min(5.0);

        let scale_change = self.scale / old_scale;
        self.offset_x = center_x - (center_x - self.offset_x) * scale_change;
        self.offset_y = center_y - (center_y - self.offset_y) * scale_change;
    }

    pub fn zoom_out(&mut self) {
        let canvas_width = self.canvas.width() as f64;
        let canvas_height = self.canvas.height() as f64;
        let center_x = canvas_width / 2.0;
        let center_y = canvas_height / 2.0;

        let old_scale = self.scale;
        self.scale = (self.scale / 1.2).max(0.1);

        let scale_change = self.scale / old_scale;
        self.offset_x = center_x - (center_x - self.offset_x) * scale_change;
        self.offset_y = center_y - (center_y - self.offset_y) * scale_change;
    }

    pub fn reset_zoom(&mut self) {
        self.scale = 1.0;
        let canvas_width = self.canvas.width() as f64;
        let canvas_height = self.canvas.height() as f64;

        if !self.nodes.is_empty() {
            let mut min_x = f64::MAX;
            let mut max_x = f64::MIN;
            let mut min_y = f64::MAX;
            let mut max_y = f64::MIN;

            for node_pos in &self.nodes {
                min_x = min_x.min(node_pos.x);
                max_x = max_x.max(node_pos.x + node_pos.width);
                min_y = min_y.min(node_pos.y);
                max_y = max_y.max(node_pos.y + node_pos.height);
            }

            let graph_width = max_x - min_x;
            let graph_height = max_y - min_y;

            self.offset_x = (canvas_width - graph_width) / 2.0 - min_x;
            self.offset_y = (canvas_height - graph_height) / 2.0 - min_y;
        } else {
            self.offset_x = 0.0;
            self.offset_y = 50.0;
        }
    }

    pub fn get_scale(&self) -> f64 {
        self.scale
    }
}
