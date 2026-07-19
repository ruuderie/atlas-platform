// apps/folio/src/pages/str_host/reviews.rs
//
// STR Reviews — /s/reviews
// Honest empty until /api/folio/reviews is available.

use leptos::prelude::*;

#[component]
pub fn StrReviews() -> impl IntoView {
    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Guest Reviews"</h1>
                    <p class="page-subtitle">"Guest ratings will appear here when the reviews API is available."</p>
                </div>
            </div>

            <div class="doc-empty">
                <p>"No guest reviews yet."</p>
                <p class="proj-section__hint" style="margin-top:0.5rem;">
                    "Review sync and host responses are not available in this release."
                </p>
            </div>
        </div>
    }
}
