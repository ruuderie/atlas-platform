use crate::auth::{ServerFnError, SessionInfo};
use leptos::prelude::*;

#[component]
pub fn PmcDashboard() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, ServerFnError>>>()
        .expect("Session context missing");
    let name = move || {
        session
            .get()
            .and_then(|r| r.ok())
            .and_then(|s| s.display_name)
            .unwrap_or_else(|| "there".into())
    };

    view! {
        <div class="page-header">
            <h1 class="page-title">{move || format!("PMC Dashboard — {}", name())}</h1>
            <p class="page-subtitle">
                "Manage your client portfolio, onboard new landlords, and view cross-client analytics."
            </p>
        </div>
        <div class="stat-grid">
            <StatCard label="Client Accounts" value="—" icon="🏢" href="/pmc/clients"/>
            <StatCard label="Managed Properties" value="—" icon="🏠" href="/pmc/clients"/>
        </div>
    }
}

#[component]
fn StatCard(
    label: &'static str,
    value: &'static str,
    icon: &'static str,
    href: &'static str,
) -> impl IntoView {
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
