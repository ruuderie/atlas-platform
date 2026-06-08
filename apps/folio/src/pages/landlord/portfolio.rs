use leptos::prelude::*;

#[component]
pub fn Portfolio() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Portfolio"</h1>
            <p class="page-subtitle">"Manage your property portfolio."</p>
        </div>
        <div class="page-placeholder">
            <p>"Portfolio data loading — connect to /api/folio/portfolios"</p>
        </div>
    }
}
