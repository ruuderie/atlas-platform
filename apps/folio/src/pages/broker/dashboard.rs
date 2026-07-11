use crate::auth::{ServerFnError, SessionInfo};
use leptos::prelude::*;

#[component]
pub fn BrokerDashboard() -> impl IntoView {
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
            <h1 class="page-title">{move || format!("Broker Office — {}", name())}</h1>
            <p class="page-subtitle">
                "Office-level oversight: agent performance, listing inventory, compliance, and revenue."
            </p>
        </div>
        <div class="stat-grid">
            <StatCard label="Active Agents"     value="—" icon="👥" href="/b/agents"/>
            <StatCard label="Office Listings"   value="—" icon="🏠" href="/b/listings"/>
            <StatCard label="Compliance Items"  value="—" icon="📋" href="/b/compliance"/>
            <StatCard label="Monthly GCI"       value="—" icon="💰" href="/b/revenue"/>
        </div>
    }
}

#[component]
pub fn BrokerAgents() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Agent Roster"</h1>
            <p class="page-subtitle">"Manage licensed agents, assign territories, and review performance."</p>
        </div>
        <div class="empty-state">
            <p>"No agents onboarded. Invite agents to join your office."</p>
        </div>
    }
}

#[component]
pub fn BrokerListings() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"All Office Listings"</h1>
            <p class="page-subtitle">"Complete inventory across all agents in this brokerage."</p>
        </div>
        <div class="empty-state">
            <p>"No listings yet. Listings appear here once agents create them."</p>
        </div>
    }
}

#[component]
pub fn BrokerCompliance() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Compliance"</h1>
            <p class="page-subtitle">"License renewals, disclosure deadlines, and regulatory filings."</p>
        </div>
        <div class="empty-state">
            <p>"No open compliance items."</p>
        </div>
    }
}

#[component]
pub fn BrokerRevenue() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Revenue"</h1>
            <p class="page-subtitle">"GCI, commission splits, and agent production reports."</p>
        </div>
        <div class="empty-state">
            <p>"No closed transactions recorded yet."</p>
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
