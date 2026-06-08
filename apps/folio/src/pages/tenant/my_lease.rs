use leptos::prelude::*;
#[component]
pub fn UmyUlease() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"my lease"</h1>
        </div>
        <div class="page-placeholder">
            <p>"Connect to /api/folio/tenant/my_lease"</p>
        </div>
    }
}
