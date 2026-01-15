mod api;
mod models;
mod graph;
mod components;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run_app() {
    // Initialize logging
    wasm_logger::init(wasm_logger::Config::default());

    log::info!("Starting Wazuh Process Tree Viewer WASM Frontend");

    // Mount the app
    yew::Renderer::<components::app::App>::new().render();
}
