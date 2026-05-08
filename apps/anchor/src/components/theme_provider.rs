use leptos::prelude::*;

#[component]
pub fn ThemeProvider(
    #[prop(into)] primary_color: Signal<String>,
    children: Children,
) -> impl IntoView {
    let color = primary_color.get_untracked();
    view! {
        <style id="atlas-platform-theme">
            {format!(":root {{ --color-primary: {}; --brand-primary: {}; }}", color, color)}
        </style>
        {children()}
    }
}
