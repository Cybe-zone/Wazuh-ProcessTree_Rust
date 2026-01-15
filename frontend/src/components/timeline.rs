use yew::prelude::*;
use chrono::{DateTime, Utc, Duration};
use crate::models::ProcessNode;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub nodes: Vec<ProcessNode>,
    pub is_visible: bool,
}

pub struct Timeline;

impl Component for Timeline {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
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

        // Parse timestamps and sort nodes by time
        let mut timed_nodes: Vec<(&ProcessNode, DateTime<Utc>)> = Vec::new();
        let mut parse_errors = 0;

        for node in ctx.props().nodes.iter() {
            match DateTime::parse_from_rfc3339(&node.timestamp) {
                Ok(dt) => {
                    timed_nodes.push((node, dt.with_timezone(&Utc)));
                }
                Err(e) => {
                    parse_errors += 1;
                    if parse_errors <= 3 {
                        log::warn!("Failed to parse timestamp '{}' for process {}: {}",
                            node.timestamp, node.process_name, e);
                    }
                }
            }
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

                        <div class="timeline-track">
                            {
                                timed_nodes.iter().map(|(node, dt)| {
                                    let elapsed = dt.signed_duration_since(first_time);
                                    let position = if time_range.num_milliseconds() > 0 {
                                        (elapsed.num_milliseconds() as f64 / time_range.num_milliseconds() as f64) * 100.0
                                    } else {
                                        0.0
                                    };

                                    html! {
                                        <div
                                            class="timeline-dot"
                                            style={format!("left: {}%; background-color: {}", position, node.color.background)}
                                            title={format!("{}: {} at {}",
                                                node.process_name,
                                                node.process_id,
                                                dt.format("%H:%M:%S%.3f")
                                            )}
                                        />
                                    }
                                }).collect::<Html>()
                            }
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
