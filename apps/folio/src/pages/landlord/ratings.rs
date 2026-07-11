//! G-27 landlord pending ratings — `/l/ratings`
//!
//! After vendor work-order complete (`case_resolved`), sessions appear here
//! for the landlord/PM (`assigned_user_id`) to rate the contractor.

use leptos::prelude::*;
use crate::pages::tenant::ratings::PendingRatingsPage;

#[component]
pub fn LandlordRatings() -> impl IntoView {
    view! {
        <PendingRatingsPage
            title="Rate contractors"
            subtitle="Pending ratings opened when a work order is completed."
            empty_message="No pending contractor ratings. Complete a vendor work order to get a nudge here."
            default_session_label="Contractor rating"
        />
    }
}
