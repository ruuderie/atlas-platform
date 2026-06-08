use leptos::prelude::*;

#[component]
pub fn Campaigns() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Campaigns"</h1>
            <p class="page-subtitle">"Marketing campaigns and outreach."</p>
        </div>
        <div class="page-placeholder">
            <p>"Campaign data loading — connect to /api/folio/campaigns"</p>
        </div>
    }
}
