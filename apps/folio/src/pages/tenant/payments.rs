use leptos::prelude::*;

#[component]
pub fn TenantPayments() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Payments"</h1>
            <p class="page-subtitle">"Your payment history and upcoming due dates."</p>
        </div>
        <div class="page-placeholder">
            <p>"Payment data loading — connect to /api/folio/billing/invoice/fiat"</p>
        </div>
    }
}
