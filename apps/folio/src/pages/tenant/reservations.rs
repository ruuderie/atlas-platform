use leptos::prelude::*;

#[component]
pub fn TenantReservations() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Reservations"</h1>
            <p class="page-subtitle">"Your STR bookings and upcoming stays."</p>
        </div>
        <div class="page-placeholder">
            <p>"Reservation data loading — connect to /api/folio/reservations"</p>
        </div>
    }
}
