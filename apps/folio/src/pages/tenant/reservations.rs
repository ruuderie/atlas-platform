use leptos::prelude::*;
#[component]
pub fn Ureservations() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"reservations"</h1>
        </div>
        <div class="page-placeholder">
            <p>"Connect to /api/folio/tenant/reservations"</p>
        </div>
    }
}
