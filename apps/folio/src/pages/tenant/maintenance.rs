use leptos::prelude::*;
#[component]
pub fn Umaintenance() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"maintenance"</h1>
        </div>
        <div class="page-placeholder">
            <p>"Connect to /api/folio/tenant/maintenance"</p>
        </div>
    }
}
