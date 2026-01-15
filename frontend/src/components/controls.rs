use yew::prelude::*;
use crate::components::app::Theme;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub theme: Theme,
    pub on_toggle_theme: Callback<()>,
    pub on_fit_view: Callback<()>,
    pub on_logout: Callback<()>,
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

            <div class="controls-right">
                <button
                    class="btn-icon"
                    onclick={on_fit_click}
                    title="Fit View"
                >
                    {"⊡"}
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
