use leptos::prelude::*;

/// Shared avatar + name + subtitle cell used in every CRM table row.
#[component]
pub fn RecordRow(
    #[prop(into)] initials: String,
    #[prop(into)] name: String,
    #[prop(into, default = String::new())] sub: String,
    #[prop(default = "var(--cobalt-dim)")] bg: &'static str,
    #[prop(default = "var(--cobalt)")] color: &'static str,
) -> impl IntoView {
    view! {
        <div class="con-cell">
            <div class="con-avatar" style=format!("background:{};color:{}", bg, color)>
                {initials}
            </div>
            <div>
                <div class="con-name-text">{name}</div>
                {(!sub.is_empty()).then(|| view! { <div class="con-title-sub">{sub}</div> })}
            </div>
        </div>
    }
}

/// Generate 1–2 uppercase initials from a name string.
pub fn initials(name: &str) -> String {
    name.split_whitespace()
        .map(|w| w.chars().next().unwrap_or('?'))
        .collect::<String>()
        .chars()
        .take(2)
        .collect::<String>()
        .to_uppercase()
}
