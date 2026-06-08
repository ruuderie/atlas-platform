use leptos::prelude::*;

#[component]
pub fn VendorInvoices() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Invoices"</h1>
            <p class="page-subtitle">"Your submitted invoices and payment status."</p>
        </div>
        <div class="page-placeholder">
            <p>"Invoice data loading — connect to /api/folio/vendor/invoices"</p>
        </div>
    }
}
