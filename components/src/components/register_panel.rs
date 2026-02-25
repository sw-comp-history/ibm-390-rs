use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Register {
    pub name: String,
    pub value: String,
    pub changed: bool,
}

#[derive(Clone, PartialEq)]
pub struct LegendItem {
    pub label: String,
    pub value: String,
    pub changed: bool,
}

#[derive(Properties, PartialEq)]
pub struct RegisterPanelProps {
    pub registers: Vec<Register>,
    pub legend_items: Vec<LegendItem>,
    #[prop_or(180)]
    pub legend_width: usize,
}

#[function_component(RegisterPanel)]
pub fn register_panel(props: &RegisterPanelProps) -> Html {
    html! {
        <div class="registers-panel">
            <div class="panel-title">{"Registers & Flags"}</div>
            <div class="registers-container">
                <div class="register-grid">
                    { for props.registers.iter().map(|reg| {
                        let class = if reg.changed {
                            "register changed"
                        } else {
                            "register"
                        };

                        html! {
                            <div {class} key={reg.name.clone()}>
                                <span class="register-label">{&reg.name}</span>
                                <span class="register-value">{&reg.value}</span>
                            </div>
                        }
                    })}
                </div>
                <div class="register-legend" style={format!("width: {}px", props.legend_width)}>
                    <div class="legend-title">{"Special Registers"}</div>
                    { for props.legend_items.iter().map(|item| {
                        let value_class = if item.changed {
                            "legend-value changed"
                        } else {
                            "legend-value"
                        };

                        html! {
                            <div class="legend-item" key={item.label.clone()}>
                                <span class="legend-label">{&item.label}</span>
                                <span class={value_class}>{&item.value}</span>
                            </div>
                        }
                    })}
                </div>
            </div>
        </div>
    }
}
