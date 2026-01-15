use yew::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent, WheelEvent};
use wasm_bindgen::{JsCast, closure::Closure};
use std::rc::Rc;
use std::cell::RefCell;
use gloo::timers::callback::Interval;

use crate::models::*;
use crate::graph::GraphRenderer;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub tree_data: Option<ProcessTree>,
    pub is_light_mode: bool,
    pub on_node_select: Callback<Option<String>>,
    pub fit_view_trigger: bool,
}

pub struct GraphView {
    canvas_ref: NodeRef,
    renderer: Rc<RefCell<Option<GraphRenderer>>>,
    animation_handle: Option<Interval>,
}

pub enum Msg {
    CanvasReady,
    MouseDown(MouseEvent),
    MouseMove(MouseEvent),
    MouseUp(MouseEvent),
    Wheel(WheelEvent),
    Animate,
}

impl Component for GraphView {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_ref: NodeRef::default(),
            renderer: Rc::new(RefCell::new(None)),
            animation_handle: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::CanvasReady => {
                log::info!("Canvas ready - initializing renderer");
                if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
                    // Set canvas size to match container
                    if let Some(parent) = canvas.parent_element() {
                        let width = parent.client_width() as u32;
                        let height = parent.client_height() as u32;
                        log::info!("Canvas size: {}x{}", width, height);
                        canvas.set_width(width);
                        canvas.set_height(height);
                    }

                    match GraphRenderer::new(canvas) {
                        Ok(renderer) => {
                            log::info!("GraphRenderer created successfully");
                            *self.renderer.borrow_mut() = Some(renderer);
                            self.start_animation(ctx);
                        }
                        Err(e) => {
                            log::error!("Failed to create renderer: {}", e);
                        }
                    }
                } else {
                    log::error!("Failed to cast canvas element");
                }
                false
            }

            Msg::MouseDown(event) => {
                log::info!("GraphView: MouseDown event received");
                if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                    let old_selected = renderer.get_selected_node().map(|n| n.id.clone());
                    renderer.handle_mouse_down(&event);
                    let new_selected = renderer.get_selected_node().map(|n| n.id.clone());

                    if old_selected != new_selected {
                        log::info!("GraphView: Node selection changed");
                        ctx.props().on_node_select.emit(new_selected);
                    }
                } else {
                    log::warn!("GraphView: Renderer not initialized");
                }
                true
            }

            Msg::MouseMove(event) => {
                if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                    renderer.handle_mouse_move(&event);
                }
                true
            }

            Msg::MouseUp(event) => {
                log::info!("GraphView: MouseUp event received");
                if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                    renderer.handle_mouse_up(&event);
                }
                true
            }

            Msg::Wheel(event) => {
                log::info!("GraphView: Wheel event received");
                if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                    renderer.handle_wheel(&event);
                } else {
                    log::warn!("GraphView: Renderer not initialized for wheel event");
                }
                true
            }

            Msg::Animate => {
                if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                    renderer.render(ctx.props().is_light_mode);
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onmousedown = ctx.link().callback(Msg::MouseDown);
        let onmousemove = ctx.link().callback(Msg::MouseMove);
        let onmouseup = ctx.link().callback(Msg::MouseUp);
        let onwheel = ctx.link().callback(Msg::Wheel);

        html! {
            <canvas
                ref={self.canvas_ref.clone()}
                class="graph-canvas"
                {onmousedown}
                {onmousemove}
                {onmouseup}
                {onwheel}
            />
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link().send_message(Msg::CanvasReady);
        }

        // Update data when tree_data changes
        if let Some(tree) = &ctx.props().tree_data {
            if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                renderer.set_data(tree.nodes.clone(), tree.edges.clone());
            }
        }

        // Handle fit view trigger
        if ctx.props().fit_view_trigger {
            if let Some(renderer) = self.renderer.borrow_mut().as_mut() {
                renderer.fit_view();
            }
        }

        // Resize canvas if needed
        if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
            if let Some(parent) = canvas.parent_element() {
                let width = parent.client_width() as u32;
                let height = parent.client_height() as u32;

                if canvas.width() != width || canvas.height() != height {
                    canvas.set_width(width);
                    canvas.set_height(height);
                }
            }
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // Interval will be automatically dropped
    }
}

impl GraphView {
    fn start_animation(&mut self, ctx: &Context<Self>) {
        let link = ctx.link().clone();

        // Use Interval to render at ~60fps (16ms)
        let interval = Interval::new(16, move || {
            link.send_message(Msg::Animate);
        });

        self.animation_handle = Some(interval);
    }
}
