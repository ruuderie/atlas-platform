use leptos::prelude::*;

#[component]
pub fn MyLease() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"My Lease"</h1>
            <p class="page-subtitle">"Your current lease agreement."</p>
        </div>
        <div class="page-placeholder">
            <p>"Lease data loading — connect to /api/folio/leases"</p>
        </div>
    }
}
