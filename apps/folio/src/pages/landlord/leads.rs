use leptos::prelude::*;

#[component]
pub fn Leads() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Leads"</h1>
            <p class="page-subtitle">"Prospective tenants and buyers."</p>
        </div>
        <div class="page-placeholder">
            <p>"Lead data loading — connect to /api/folio/leads"</p>
        </div>
    }
}
