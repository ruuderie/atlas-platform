use leptos::prelude::*;

#[component]
pub fn Billing() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Billing"</h1>
            <p class="page-subtitle">"Invoices, payments, and ledger."</p>
        </div>
        <div class="page-placeholder">
            <p>"Billing data loading — connect to /api/folio/billing/invoice/fiat"</p>
        </div>
    }
}
