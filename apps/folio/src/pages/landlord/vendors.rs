use leptos::prelude::*;

#[component]
pub fn Vendors() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Vendors"</h1>
            <p class="page-subtitle">"Contractors and service providers."</p>
        </div>
        <div class="page-placeholder">
            <p>"Vendor data loading — connect to /api/folio/vendors"</p>
        </div>
    }
}
