use leptos::prelude::*;
#[component]
pub fn Upayments() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"payments"</h1>
        </div>
        <div class="page-placeholder">
            <p>"Connect to /api/folio/tenant/payments"</p>
        </div>
    }
}
