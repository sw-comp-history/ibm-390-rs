use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct HeaderProps {
    pub title: String,
    #[prop_or_default]
    pub subtitle: Option<String>,
}

#[function_component(Header)]
pub fn header(props: &HeaderProps) -> Html {
    html! {
        <header>
            <h1>{&props.title}</h1>
            if let Some(subtitle) = &props.subtitle {
                <div class="subtitle">{subtitle}</div>
            }
        </header>
    }
}
