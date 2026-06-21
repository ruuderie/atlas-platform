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
    let avatar_style = format!(
        "display:flex;align-items:center;justify-content:center;\
         width:30px;height:30px;border-radius:6px;\
         font-size:11px;font-weight:700;flex-shrink:0;\
         background:{};color:{};",
        bg, color
    );
    view! {
        <div class="con-cell" style="display:flex;align-items:center;gap:10px;">
            <div class="con-avatar" style=avatar_style>
                {initials}
            </div>
            <div style="min-width:0;">
                <div
                    class="con-name-text"
                    style="font-size:12.5px;font-weight:500;color:var(--text-primary);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:200px;"
                >{name}</div>
                {(!sub.is_empty()).then(|| view! {
                    <div
                        class="con-title-sub"
                        style="font-size:11px;color:var(--text-muted);margin-top:1px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:200px;"
                    >{sub}</div>
                })}
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
