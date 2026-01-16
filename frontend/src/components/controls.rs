use yew::prelude::*;
use crate::components::app::Theme;
use std::collections::HashMap;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub theme: Theme,
    pub on_toggle_theme: Callback<()>,
    pub on_fit_view: Callback<()>,
    pub on_logout: Callback<()>,
    pub classification_stats: HashMap<String, usize>,
    pub total_processes: usize,
    pub orphan_count: usize,
}

#[function_component(Controls)]
pub fn controls(props: &Props) -> Html {
    let theme_icon = match props.theme {
        Theme::Light => "🌙",
        Theme::Dark => "☀️",
    };

    let on_theme_click = {
        let callback = props.on_toggle_theme.clone();
        Callback::from(move |_: MouseEvent| callback.emit(()))
    };

    let on_fit_click = {
        let callback = props.on_fit_view.clone();
        Callback::from(move |_: MouseEvent| callback.emit(()))
    };

    let on_logout_click = {
        let callback = props.on_logout.clone();
        Callback::from(move |_: MouseEvent| callback.emit(()))
    };

    html! {
        <div class="controls-bar">
            <div class="controls-left">
                <h1 class="app-title">{"Wazuh Process Tree Viewer"}</h1>
            </div>

            <div class="controls-stats">
                <div class="stat-badge">
                    <span class="stat-label">{"Total Processes:"}</span>
                    <span class="stat-value">{props.total_processes}</span>
                </div>
                <div class="stat-badge">
                    <span class="stat-label">{"Orphan Processes:"}</span>
                    <span class="stat-value">{props.orphan_count}</span>
                </div>
                {
                    props.classification_stats.iter().map(|(classification, count)| {
                        let class_name = match classification.as_str() {
                            "Critical" => "stat-critical",
                            "Suspicious" => "stat-suspicious",
                            _ => "stat-normal"
                        };
                        html! {
                            <div class="stat-badge">
                                <span class="stat-label">{format!("{}:", classification)}</span>
                                <span class={classes!("stat-value", class_name)}>{count}</span>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>

            <div class="controls-right">
                <button
                    class="btn-icon"
                    onclick={on_fit_click}
                    title="Fit View"
                >
                    {"🎯"}
                </button>

                <button
                    class="theme-toggle-btn"
                    onclick={on_theme_click}
                    title="Toggle Theme"
                >
                    {theme_icon}
                </button>

                <button
                    class="btn-icon"
                    onclick={on_logout_click}
                    title="Logout"
                >
                    {"⎋"}
                </button>
            </div>
        </div>
    }
}
