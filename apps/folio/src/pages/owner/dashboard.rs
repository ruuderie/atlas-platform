use leptos::prelude::*;
use crate::auth::{SessionInfo, ServerFnError};

/// Owner portal dashboard — read-only view of the beneficial owner's portfolio.
/// Owners cannot create, edit, or delete any resource.
#[component]
pub fn OwnerDashboard() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, ServerFnError>>>()
        .expect("Session context missing");
    let name = move || session.get()
        .and_then(|r| r.ok())
        .and_then(|s| s.display_name)
        .unwrap_or_else(|| "there".into());

    view! {
        <div class="page-header">
            <h1 class="page-title">{move || format!("Owner Portal — {}", name())}</h1>
            <p class="page-subtitle">
                "Read-only view of your property portfolio. Contact your property manager to make changes."
            </p>
        </div>
        <div class="stat-grid">
            <StatCard label="Properties" value="—" icon="🏠" href="/o/properties"/>
        </div>
    }
}

#[component]
fn StatCard(label: &'static str, value: &'static str, icon: &'static str, href: &'static str) -> impl IntoView {
    view! {
        <a href=href class="stat-card stat-card--link">
            <span class="stat-icon">{icon}</span>
            <div class="stat-body">
                <p class="stat-label">{label}</p>
                <p class="stat-value">{value}</p>
            </div>
        </a>
    }
}
