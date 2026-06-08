use leptos::prelude::*;

#[component]
pub fn LandlordReservations() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Reservations"</h1>
            <p class="page-subtitle">"STR bookings across all properties."</p>
        </div>
        <div class="page-placeholder">
            <p>"Reservation data loading — connect to /api/folio/reservations"</p>
        </div>
    }
}
