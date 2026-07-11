use leptos::prelude::*;
#[component]
pub fn NotFound() -> impl IntoView {
    view! { <div class="not-found"><h1>"404"</h1><p>"Page not found."</p><a href="/dashboard">"Go to dashboard"</a></div> }
}
