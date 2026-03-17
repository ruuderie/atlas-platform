use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BadgeIntent {
    Success,
    Warning,
    Error,
    Default,
    Primary,
}

impl BadgeIntent {
    pub fn as_class(&self) -> &'static str {
        match self {
            BadgeIntent::Success => "success",
            BadgeIntent::Warning => "warning",
            BadgeIntent::Error => "error",
            BadgeIntent::Default => "default",
            BadgeIntent::Primary => "primary",
        }
    }
}

#[component]
pub fn Badge(
    children: Children,
    #[prop(optional)] intent: Option<BadgeIntent>,
) -> impl IntoView {
    let intent_val = intent.unwrap_or(BadgeIntent::Default);
    let class_name = format!("badge badge-{}", intent_val.as_class());
    view! {
        <span class=class_name>
            {children()}
        </span>
    }
}
