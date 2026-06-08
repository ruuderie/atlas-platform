use leptos::prelude::*;

#[component]
pub fn Assets() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Assets"</h1>
            <p class="page-subtitle">"Properties and units across your portfolio."</p>
        </div>
        <div class="page-placeholder">
            <p>"Asset data loading — connect to /api/folio/assets"</p>
        </div>
    }
}
