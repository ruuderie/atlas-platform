use leptos::prelude::*;
#[component]
pub fn Uinvoices() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"invoices"</h1>
        </div>
        <div class="page-placeholder">
            <p>"Connect to /api/folio/vendor/invoices"</p>
        </div>
    }
}
