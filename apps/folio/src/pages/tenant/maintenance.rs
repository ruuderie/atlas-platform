use leptos::prelude::*;

#[component]
pub fn MaintenanceRequests() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Maintenance"</h1>
            <p class="page-subtitle">"Submit and track maintenance requests."</p>
        </div>
        <div class="page-placeholder">
            <p>"Maintenance data loading — connect to /api/folio/maintenance"</p>
        </div>
    }
}
