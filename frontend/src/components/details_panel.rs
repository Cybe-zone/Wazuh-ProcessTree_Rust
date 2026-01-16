use yew::prelude::*;
use crate::models::ProcessNode;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub node: ProcessNode,
}

#[function_component(DetailsPanel)]
pub fn details_panel(props: &Props) -> Html {
    let node = &props.node;

    html! {
        <div class="details-panel">
            <div class="sidebar-section">
                <h3>{"Process Details"}</h3>

                <div class="detail-group">
                    <div class="detail-label">{"Process Name"}</div>
                    <div class="detail-value">{&node.process_name}</div>
                </div>

                <div class="detail-group">
                    <div class="detail-label">{"Process ID (PID)"}</div>
                    <div class="detail-value">{node.process_id}</div>
                </div>

                {
                    if let Some(ppid) = node.parent_id {
                        html! {
                            <div class="detail-group">
                                <div class="detail-label">{"Parent ID (PPID)"}</div>
                                <div class="detail-value">{ppid}</div>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                <div class="detail-group">
                    <div class="detail-label">{"User"}</div>
                    <div class="detail-value">{&node.user}</div>
                </div>

                <div class="detail-group">
                    <div class="detail-label">{"Classification"}</div>
                    <div class="detail-value classification-badge"
                         style={format!("background-color: {}; border-color: {}",
                                      node.color.background, node.color.border)}>
                        {&node.classification}
                    </div>
                </div>

                <div class="detail-group">
                    <div class="detail-label">{"Timestamp"}</div>
                    <div class="detail-value detail-timestamp">{&node.timestamp}</div>
                </div>

                <div class="detail-group">
                    <div class="detail-label">{"Command Line"}</div>
                    <div class="detail-value detail-command">
                        <code>{&node.command_line}</code>
                    </div>
                </div>
            </div>

            <div class="sidebar-section">
                <h3>{"Actions"}</h3>
                <div class="action-buttons">
                    <button class="btn-secondary" onclick={
                        let cmd = node.command_line.clone();
                        Callback::from(move |_| {
                            // Log command to console for now
                            // Clipboard API requires additional web-sys features
                            log::info!("Command: {}", cmd);
                            web_sys::console::log_1(&format!("Command copied: {}", cmd).into());
                        })
                    }>
                        {"Copy Command"}
                    </button>
                </div>
            </div>
        </div>
    }
}
