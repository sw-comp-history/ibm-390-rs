use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct SidebarButton {
    pub emoji: String,
    pub label: String,
    pub onclick: Callback<MouseEvent>,
    #[allow(dead_code)]
    pub title: Option<String>,
}

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub buttons: Vec<SidebarButton>,
}

#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    html! {
        <div class="sidebar">
            { for props.buttons.iter().map(|btn| {
                let title = btn.title.clone().unwrap_or_else(|| btn.label.clone());
                let onclick = btn.onclick.clone();

                html! {
                    <button
                        {onclick}
                        title={title}
                    >
                        {&btn.emoji}{" "}{&btn.label}
                    </button>
                }
            })}
        </div>
    }
}
