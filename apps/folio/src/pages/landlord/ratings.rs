//! G-27 landlord ratings — `/l/ratings`
//! Full surface: session list + job photos + ScorecardWidget (not a thin wrapper).

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::pages::tenant::ratings::PendingRatingsPage;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component]
pub fn LandlordRatings() -> impl IntoView {
    let q = use_query_map();
    let project_note = Memo::new(move |_| q.get().get("project"));

    view! {
        <div class="ratings-page landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Rate contractors".to_string())
                subtitle=Signal::derive(|| {
                    "Rate contractors after work orders finish. Job photos appear on each session when available."
                        .to_string()
                })
            />
            {move || project_note.get().map(|pid| {
                let href = FolioRoute::LandlordProjectDetail.path().replace(":id", &pid);
                view! {
                    <p class="proj-section__hint" style="margin-bottom:1rem;">
                        "Showing sessions for project · "
                        <a class="hub-activity-rail__all" href=href>"Open project"</a>
                        " · "
                        <a class="hub-activity-rail__all" href=FolioRoute::LandlordRatings.path()>"Clear filter"</a>
                    </p>
                }
            })}

            <PendingRatingsPage
                title="Sessions"
                subtitle="Opened when a work order is completed."
                empty_message="No pending contractor ratings. Complete a work order with an assigned vendor to open a session."
                default_session_label="Contractor rating"
            />
        </div>
    }
}
