use leptos::prelude::*;

#[component]
pub fn ThemeProvider(
    #[prop(into)] primary_color: Signal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <style id="atlas-platform-theme">
            {move || {
                let color = primary_color.get();
                format!(":root {{ --color-primary: {}; --brand-primary: {}; }}", color, color)
            }}
        </style>
        {children()}
    }
}
