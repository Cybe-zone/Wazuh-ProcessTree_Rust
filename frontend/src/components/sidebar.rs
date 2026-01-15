use yew::prelude::*;
use crate::components::app::SidebarSide;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub side: SidebarSide,
    pub children: Children,
    pub on_close: Callback<()>,
}

#[function_component(Sidebar)]
pub fn sidebar(props: &Props) -> Html {
    let side_class = match props.side {
        SidebarSide::Left => "left",
        SidebarSide::Right => "right",
    };

    let on_close = {
        let callback = props.on_close.clone();
        Callback::from(move |_: MouseEvent| {
            callback.emit(());
        })
    };

    html! {
        <div class={classes!("sidebar", side_class)}>
            <button class="minimize-btn" onclick={on_close}>
                {
                    match props.side {
                        SidebarSide::Left => "◀",
                        SidebarSide::Right => "▶",
                    }
                }
            </button>
            <div class="sidebar-content">
                { for props.children.iter() }
            </div>
        </div>
    }
}
