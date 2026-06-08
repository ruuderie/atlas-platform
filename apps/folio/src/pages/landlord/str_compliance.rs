use leptos::prelude::*;

#[component]
pub fn StrCompliance() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"STR Compliance"</h1>
            <p class="page-subtitle">"Short-term rental permits and regulatory status."</p>
        </div>
        <div class="page-placeholder">
            <p>"STR data loading — connect to /api/folio/str/permits"</p>
        </div>
    }
}
