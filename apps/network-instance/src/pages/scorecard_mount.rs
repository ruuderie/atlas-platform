//! Mount shared-ui ScorecardWidget / Configurator against the same tenant APIs as Folio.
//!
//! Route: `/dashboard/scorecards`
//!
//! APIs (no Folio-only handlers):
//!   GET  /api/scorecard-templates?app_instance_id=
//!   POST /api/scorecards/get-or-create
//!   POST /api/scorecards/{id}/sessions
//!   POST /api/scorecard-sessions/{sid}/entries
//!
//! See: `docs/architecture/g27/g27_app_instance_runtime.md`

use leptos::prelude::*;

#[component]
pub fn ScorecardMountStub() -> impl IntoView {
    view! {
        <div class="w-full">
            <h1>"Scorecards"</h1>
            <p>
                "Wire ScorecardWidget here for listing/profile subjects. "
                "Use GET /api/scorecard-templates (deployed-only) — same contract as Folio."
            </p>
        </div>
    }
}
