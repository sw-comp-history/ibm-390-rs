use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ModalProps {
    pub id: String,
    pub title: String,
    pub active: bool,
    pub on_close: Callback<MouseEvent>,
    pub children: Children,
}

#[function_component(Modal)]
pub fn modal(props: &ModalProps) -> Html {
    let modal_class = classes!("modal", props.active.then_some("active"));

    let on_overlay_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |e: MouseEvent| {
            // Only close if clicking the overlay itself, not the content
            #[allow(clippy::collapsible_if)]
            if let Some(target) = e.target() {
                if let Some(element) = target.dyn_ref::<web_sys::HtmlElement>() {
                    if element.class_name().contains("modal") {
                        on_close.emit(e);
                    }
                }
            }
        })
    };

    html! {
        <div id={props.id.clone()} class={modal_class} onclick={on_overlay_click}>
            <div class="modal-content" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                <span class="modal-close" onclick={props.on_close.clone()}>
                    {"×"}
                </span>
                <h2 class="modal-title">{&props.title}</h2>
                <div class="modal-body">
                    {props.children.clone()}
                </div>
            </div>
        </div>
    }
}
