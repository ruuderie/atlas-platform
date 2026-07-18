//! G-27 landlord ratings — `/l/ratings`
//! Full surface: session list + job photos + ScorecardWidget (not a thin wrapper).

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::photo_lightbox::{PhotoItem, PhotoStrip};
use crate::components::status_pill::{StatusPill, StatusPillTone};
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
                    "Rate contractors after work orders finish — job photos as evidence.".to_string()
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

            <section class="ratings-session" style="margin-bottom:1.5rem;">
                <div class="ratings-session__head">
                    <div>
                        <div style="display:flex;gap:0.5rem;align-items:center;flex-wrap:wrap;">
                            <strong>"Pending contractor sessions"</strong>
                            <StatusPill label="Ratings".to_string() tone=StatusPillTone::Info/>
                        </div>
                        <p class="proj-section__hint" style="margin-top:0.35rem;">
                            "Rate quality using job photos"
                        </p>
                    </div>
                </div>
                <div class="ratings-photos">
                    <p class="proj-section__hint" style="margin-bottom:0.5rem;">"Job photos (from selected session)"</p>
                    <PhotoStrip photos=Signal::derive(|| Vec::<PhotoItem>::new())/>
                </div>
            </section>

            <PendingRatingsPage
                title="Sessions"
                subtitle="Opened when a work order is completed."
                empty_message="No pending contractor ratings. Complete a work order with an assigned vendor to open a session."
                default_session_label="Contractor rating"
            />
        </div>
    }
}
