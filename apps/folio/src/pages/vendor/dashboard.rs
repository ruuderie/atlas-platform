use leptos::prelude::*;

#[component]
pub fn VendorDashboard() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Vendor Dashboard"</h1>
            <p class="page-subtitle">"Your work orders and invoices at a glance."</p>
        </div>
        <div class="page-placeholder">
            <p>"Vendor dashboard — connecting to Folio backend."</p>
        </div>
    }
}
