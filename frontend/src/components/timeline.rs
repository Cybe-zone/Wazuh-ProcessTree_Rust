use yew::prelude::*;
use web_sys::MouseEvent;
use wasm_bindgen::JsCast;
use chrono::{DateTime, Utc, Duration};
use crate::models::ProcessNode;

#[derive(Clone, PartialEq)]
pub enum TimelineAction {
    ZoomToNode(String),  // node_id
    ZoomToTimeRange(String, String),  // start_time, end_time
}

#[derive(Properties, PartialEq)]
pub struct Props {
    pub nodes: Vec<ProcessNode>,
    pub is_visible: bool,
    pub on_action: Callback<TimelineAction>,
    pub hidden_classifications: std::collections::HashSet<String>,
}

pub struct Timeline {
    selection_start: f64,  // Percentage 0.0 - 100.0
    selection_end: f64,    // Percentage 0.0 - 100.0
    dragging_handle: Option<DraggingHandle>,
}

#[derive(Clone, PartialEq)]
enum DraggingHandle {
    Start,
    End,
}

pub enum TimelineMsg {
    StartDragHandle(DraggingHandle, f64), // handle, initial position
    DragHandle(f64), // current position
    EndDrag,
    ApplySelection,
}

impl Component for Timeline {
    type Message = TimelineMsg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            selection_start: 0.0,
            selection_end: 100.0,
            dragging_handle: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TimelineMsg::StartDragHandle(handle, _position) => {
                self.dragging_handle = Some(handle);
                true
            }
            TimelineMsg::DragHandle(position) => {
                if let Some(handle) = &self.dragging_handle {
                    match handle {
                        DraggingHandle::Start => {
                            self.selection_start = position.max(0.0).min(self.selection_end - 5.0);
                        }
                        DraggingHandle::End => {
                            self.selection_end = position.min(100.0).max(self.selection_start + 5.0);
                        }
                    }
                }
                true
            }
            TimelineMsg::EndDrag => {
                if self.dragging_handle.is_some() {
                    self.dragging_handle = None;

                    // Parse timestamps to calculate time range
                    let mut timed_nodes: Vec<DateTime<Utc>> = Vec::new();
                    for node in ctx.props().nodes.iter() {
                        if ctx.props().hidden_classifications.contains(&node.classification) {
                            continue;
                        }
                        let normalized_timestamp = normalize_timestamp(&node.timestamp);
                        if let Ok(dt) = DateTime::parse_from_rfc3339(&normalized_timestamp) {
                            timed_nodes.push(dt.with_timezone(&Utc));
                        }
                    }

                    timed_nodes.sort();

                    if !timed_nodes.is_empty() {
                        let first_time = timed_nodes.first().unwrap();
                        let last_time = timed_nodes.last().unwrap();
                        let time_range = last_time.signed_duration_since(*first_time);

                        // Calculate time at selection positions
                        let start_offset_ms = (time_range.num_milliseconds() as f64 * self.selection_start / 100.0) as i64;
                        let end_offset_ms = (time_range.num_milliseconds() as f64 * self.selection_end / 100.0) as i64;

                        let start_time = *first_time + Duration::milliseconds(start_offset_ms);
                        let end_time = *first_time + Duration::milliseconds(end_offset_ms);

                        log::info!("Selection range: {}% to {}%, time range: {} to {}",
                            self.selection_start, self.selection_end,
                            start_time.format("%H:%M:%S"),
                            end_time.format("%H:%M:%S")
                        );

                        ctx.props().on_action.emit(TimelineAction::ZoomToTimeRange(
                            start_time.to_rfc3339(),
                            end_time.to_rfc3339()
                        ));
                    }
                }
                true
            }
            TimelineMsg::ApplySelection => {
                // Convert percentages to time range
                if let Some(nodes) = ctx.props().nodes.first() {
                    // This will be handled when user releases the drag
                    log::info!("Apply selection: {}% to {}%", self.selection_start, self.selection_end);
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        log::info!("Timeline view called - is_visible: {}, nodes count: {}",
            ctx.props().is_visible,
            ctx.props().nodes.len()
        );

        if !ctx.props().is_visible {
            log::info!("Timeline not visible, returning empty");
            return html! {};
        }

        if ctx.props().nodes.is_empty() {
            log::info!("Timeline nodes empty, returning empty");
            return html! {};
        }

        // Parse timestamps and sort nodes by time, filtering out hidden classifications
        let mut timed_nodes: Vec<(&ProcessNode, DateTime<Utc>)> = Vec::new();
        let mut parse_errors = 0;
        let mut filtered_count = 0;

        for node in ctx.props().nodes.iter() {
            // Skip nodes with hidden classifications
            if ctx.props().hidden_classifications.contains(&node.classification) {
                filtered_count += 1;
                continue;
            }

            // Normalize timestamp format: convert +0000 to +00:00 if needed
            let normalized_timestamp = normalize_timestamp(&node.timestamp);

            match DateTime::parse_from_rfc3339(&normalized_timestamp) {
                Ok(dt) => {
                    timed_nodes.push((node, dt.with_timezone(&Utc)));
                }
                Err(e) => {
                    parse_errors += 1;
                    if parse_errors <= 3 {
                        log::warn!("Failed to parse timestamp '{}' (normalized: '{}') for process {}: {}",
                            node.timestamp, normalized_timestamp, node.process_name, e);
                    }
                }
            }
        }

        if filtered_count > 0 {
            log::info!("Timeline: filtered out {} nodes with hidden classifications", filtered_count);
        }

        log::info!("Timeline: parsed {} timestamps successfully, {} parse errors",
            timed_nodes.len(), parse_errors);

        timed_nodes.sort_by_key(|(_, dt)| *dt);

        if timed_nodes.is_empty() {
            log::warn!("Timeline: no valid timestamps found, returning empty");
            return html! {};
        }

        let first_time = timed_nodes.first().unwrap().1;
        let last_time = timed_nodes.last().unwrap().1;
        let time_range = last_time.signed_duration_since(first_time);

        log::info!("Timeline: {} to {}, range: {:?}",
            first_time.format("%H:%M:%S"),
            last_time.format("%H:%M:%S"),
            time_range
        );

        // Group nodes by time slots (e.g., per second or minute)
        let slot_duration = if time_range.num_seconds() < 60 {
            1 // 1 second slots
        } else if time_range.num_minutes() < 60 {
            60 // 1 minute slots
        } else {
            3600 // 1 hour slots
        };

        html! {
            <div class="timeline-overlay">
                <div class="timeline-container">
                    <div class="timeline-header">
                        <h4>{"Process Timeline"}</h4>
                        <span class="timeline-range">
                            {format!("{} → {}",
                                first_time.format("%H:%M:%S"),
                                last_time.format("%H:%M:%S")
                            )}
                        </span>
                    </div>

                    <div class="timeline-bar">
                        <div class="timeline-start-marker">
                            {first_time.format("%H:%M:%S").to_string()}
                        </div>

                        <div
                            class="timeline-track"
                            onmousemove={
                                let link = ctx.link().clone();
                                Callback::from(move |e: MouseEvent| {
                                    if e.buttons() == 1 {  // Left mouse button is pressed
                                        let target = e.current_target().unwrap();
                                        let element: web_sys::HtmlElement = target.dyn_into().unwrap();
                                        let rect = element.get_bounding_client_rect();
                                        let mouse_x = e.client_x() as f64 - rect.left();
                                        let track_width = rect.width();
                                        let position_percent = (mouse_x / track_width * 100.0).max(0.0).min(100.0);
                                        link.send_message(TimelineMsg::DragHandle(position_percent));
                                    }
                                })
                            }
                            onmouseup={
                                let link = ctx.link().clone();
                                Callback::from(move |_e: MouseEvent| {
                                    link.send_message(TimelineMsg::EndDrag);
                                })
                            }
                            onclick={
                                let on_action = ctx.props().on_action.clone();
                                let first_time = first_time;
                                let time_range = time_range;
                                Callback::from(move |e: MouseEvent| {
                                    let target = e.target().unwrap();
                                    let element: web_sys::HtmlElement = target.dyn_into().unwrap();
                                    let rect = element.get_bounding_client_rect();
                                    let click_x = e.client_x() as f64 - rect.left();
                                    let track_width = rect.width();
                                    let position_percent = (click_x / track_width).max(0.0).min(1.0);

                                    log::info!("Timeline track clicked at position: {:.2}%", position_percent * 100.0);

                                    // Calculate time at click position
                                    let click_offset_ms = (time_range.num_milliseconds() as f64 * position_percent) as i64;
                                    let click_time = first_time + Duration::milliseconds(click_offset_ms);

                                    // Calculate time range: ±10% of total range
                                    let range_margin_ms = (time_range.num_milliseconds() as f64 * 0.1) as i64;
                                    let start_time = click_time - Duration::milliseconds(range_margin_ms);
                                    let end_time = click_time + Duration::milliseconds(range_margin_ms);

                                    log::info!("Zooming to time range: {} to {}",
                                        start_time.format("%H:%M:%S"),
                                        end_time.format("%H:%M:%S")
                                    );

                                    on_action.emit(TimelineAction::ZoomToTimeRange(
                                        start_time.to_rfc3339(),
                                        end_time.to_rfc3339()
                                    ));
                                })
                            }
                        >
                            {
                                timed_nodes.iter().map(|(node, dt)| {
                                    let elapsed = dt.signed_duration_since(first_time);
                                    let position = if time_range.num_milliseconds() > 0 {
                                        (elapsed.num_milliseconds() as f64 / time_range.num_milliseconds() as f64) * 100.0
                                    } else {
                                        0.0
                                    };

                                    let node_id = node.id.clone();
                                    let on_action = ctx.props().on_action.clone();

                                    html! {
                                        <div
                                            class="timeline-dot"
                                            style={format!("left: {}%; background-color: {}", position, node.color.background)}
                                            title={format!("{}: {} at {}",
                                                node.process_name,
                                                node.process_id,
                                                dt.format("%H:%M:%S%.3f")
                                            )}
                                            onclick={
                                                let node_id = node_id.clone();
                                                Callback::from(move |e: MouseEvent| {
                                                    e.stop_propagation(); // Prevent track click
                                                    log::info!("Timeline dot clicked - node_id: {}", node_id);
                                                    on_action.emit(TimelineAction::ZoomToNode(node_id.clone()));
                                                })
                                            }
                                        />
                                    }
                                }).collect::<Html>()
                            }

                            // Selection range overlay
                            <div
                                class="timeline-selection-range"
                                style={format!("left: {}%; width: {}%",
                                    self.selection_start,
                                    self.selection_end - self.selection_start
                                )}
                            />

                            // Start handle
                            <div
                                class="timeline-handle timeline-handle-start"
                                style={format!("left: {}%", self.selection_start)}
                                onmousedown={
                                    let link = ctx.link().clone();
                                    Callback::from(move |e: MouseEvent| {
                                        e.prevent_default();
                                        e.stop_propagation();
                                        let rect = e.current_target().unwrap()
                                            .dyn_into::<web_sys::HtmlElement>().unwrap()
                                            .get_bounding_client_rect();
                                        let pos = e.client_x() as f64 - rect.left();
                                        link.send_message(TimelineMsg::StartDragHandle(DraggingHandle::Start, pos));
                                    })
                                }
                            >
                                <div class="timeline-handle-grip"></div>
                            </div>

                            // End handle
                            <div
                                class="timeline-handle timeline-handle-end"
                                style={format!("left: {}%", self.selection_end)}
                                onmousedown={
                                    let link = ctx.link().clone();
                                    Callback::from(move |e: MouseEvent| {
                                        e.prevent_default();
                                        e.stop_propagation();
                                        let rect = e.current_target().unwrap()
                                            .dyn_into::<web_sys::HtmlElement>().unwrap()
                                            .get_bounding_client_rect();
                                        let pos = e.client_x() as f64 - rect.left();
                                        link.send_message(TimelineMsg::StartDragHandle(DraggingHandle::End, pos));
                                    })
                                }
                            >
                                <div class="timeline-handle-grip"></div>
                            </div>
                        </div>

                        <div class="timeline-end-marker">
                            {last_time.format("%H:%M:%S").to_string()}
                        </div>
                    </div>

                    <div class="timeline-legend">
                        <span>{format!("{} processes over {}",
                            timed_nodes.len(),
                            format_duration(time_range)
                        )}</span>
                    </div>
                </div>
            </div>
        }
    }
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.num_seconds();
    if secs < 60 {
        format!("{} seconds", secs)
    } else if secs < 3600 {
        format!("{} minutes", secs / 60)
    } else {
        format!("{} hours {} minutes", secs / 3600, (secs % 3600) / 60)
    }
}

/// Normalize timestamp to RFC3339 format
/// Converts timezone from +0000 to +00:00 format
pub fn normalize_timestamp(timestamp: &str) -> String {
    // Check if timestamp ends with +0000 or -0000 (without colon)
    if timestamp.len() >= 5 {
        let len = timestamp.len();
        let last_5 = &timestamp[len - 5..];

        // Check for +0000 or -0000 pattern
        if (last_5.starts_with('+') || last_5.starts_with('-')) &&
           last_5.chars().skip(1).all(|c| c.is_ascii_digit()) {
            // Insert colon: +0000 -> +00:00
            let mut normalized = timestamp[..len - 2].to_string();
            normalized.push(':');
            normalized.push_str(&timestamp[len - 2..]);
            return normalized;
        }
    }

    timestamp.to_string()
}
