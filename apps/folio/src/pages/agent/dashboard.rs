use leptos::prelude::*;
use crate::auth::{SessionInfo, ServerFnError};

#[component]
pub fn AgentDashboard() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, ServerFnError>>>()
        .expect("Session context missing");
    let name = move || session.get()
        .and_then(|r| r.ok())
        .and_then(|s| s.display_name)
        .unwrap_or_else(|| "there".into());

    view! {
        <div class="page-header">
            <h1 class="page-title">{move || format!("Agent Dashboard — {}", name())}</h1>
            <p class="page-subtitle">
                "Manage your client files, active listings, and upcoming appointments."
            </p>
        </div>
        <div class="stat-grid">
            <StatCard label="Active Clients"   value="—" icon="👤" href="/a/clients"/>
            <StatCard label="My Listings"      value="—" icon="🏠" href="/a/listings"/>
            <StatCard label="Open Deals"       value="—" icon="🤝" href="/a/deals"/>
            <StatCard label="Upcoming Showings" value="—" icon="📅" href="/a/schedule"/>
        </div>
    }
}

#[component]
pub fn AgentClients() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"My Clients"</h1>
            <p class="page-subtitle">"Buyer and seller client files managed by you."</p>
        </div>
        <div class="empty-state">
            <p>"No clients yet. Add your first client to get started."</p>
        </div>
    }
}

#[component]
pub fn AgentListings() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"My Listings"</h1>
            <p class="page-subtitle">"Active, pending, and sold listings you manage."</p>
        </div>
        <div class="empty-state">
            <p>"No listings assigned to you yet."</p>
        </div>
    }
}

#[component]
pub fn AgentDeals() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Deals"</h1>
            <p class="page-subtitle">"Track offers, counter-offers, and closing timelines."</p>
        </div>
        <div class="empty-state">
            <p>"No open deals. Submit or receive an offer to create a deal file."</p>
        </div>
    }
}

#[component]
pub fn AgentSchedule() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Schedule"</h1>
            <p class="page-subtitle">"Showings, open houses, and client appointments."</p>
        </div>
        <div class="empty-state">
            <p>"No upcoming appointments scheduled."</p>
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
