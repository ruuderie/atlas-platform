use leptos::prelude::*;

#[component]
pub fn WorkOrders() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Work Orders"</h1>
            <p class="page-subtitle">"Assigned work orders — open and in progress."</p>
        </div>
        <div class="page-placeholder">
            <p>"Work order data loading — connect to /api/folio/vendor/work-orders"</p>
        </div>
    }
}
