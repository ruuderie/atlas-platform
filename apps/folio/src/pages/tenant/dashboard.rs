use leptos::prelude::*;

#[component]
pub fn TenantDashboard() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"My Dashboard"</h1>
            <p class="page-subtitle">"Your lease, payments, and requests at a glance."</p>
        </div>
        <div class="page-placeholder">
            <p>"Tenant dashboard — connecting to Folio backend."</p>
        </div>
    }
}
