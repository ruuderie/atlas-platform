use leptos::prelude::*;

#[component]
pub fn Leases() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Leases"</h1>
            <p class="page-subtitle">"Active lease contracts."</p>
        </div>
        <div class="page-placeholder">
            <p>"Lease data loading — connect to /api/folio/leases"</p>
        </div>
    }
}
