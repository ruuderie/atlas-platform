use leptos::prelude::*;

/// Stub placeholder for the PMC client book page.
/// Will list all managed landlord accounts with filtering and search.
#[component]
pub fn ClientBook() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Client Book"</h1>
            <p class="page-subtitle">"All managed landlord accounts."</p>
        </div>
        <div class="empty-state">
            <p>"No clients yet. Invite a landlord to get started."</p>
        </div>
    }
}
