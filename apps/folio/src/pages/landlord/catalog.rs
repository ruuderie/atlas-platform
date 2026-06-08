use leptos::prelude::*;

#[component]
pub fn Catalog() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Catalog"</h1>
            <p class="page-subtitle">"Product and service pricebook."</p>
        </div>
        <div class="page-placeholder">
            <p>"Catalog data loading — connect to /api/folio/catalog"</p>
        </div>
    }
}
