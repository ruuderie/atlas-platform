use leptos::prelude::*;
#[component]
pub fn Uvendors() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"vendors"</h1>
        </div>
        <div class="page-placeholder">
            <p>"Connect to /api/folio/vendors"</p>
        </div>
    }
}
