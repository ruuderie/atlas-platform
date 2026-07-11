use crate::auth::{ServerFnError, SessionInfo};
use leptos::prelude::*;

#[component]
pub fn LandlordDashboard() -> impl IntoView {
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
            <h1 class="page-title">{move || format!("Welcome back, {}!", name())}</h1>
            <p class="page-subtitle">"Here's what's happening across your portfolio today."</p>
        </div>
        <div class="stat-grid">
            <StatCard label="Properties"       value="—" icon="🏠" href="/l/portfolio"/>
            <StatCard label="Active Leases"    value="—" icon="📋" href="/l/leases"/>
            <StatCard label="Open Leads"       value="—" icon="👤" href="/l/leads"/>
            <StatCard label="Revenue MTD"      value="—" icon="💰" href="/l/billing"/>
            <StatCard label="Open Work Orders" value="—" icon="🔧" href="/l/vendors"/>
            <StatCard label="STR Reservations" value="—" icon="📅" href="/l/reservations"/>
        </div>
        <div class="section-row">
            <QuickLinks/>
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

#[component]
fn QuickLinks() -> impl IntoView {
    view! {
        <div class="quick-links">
            <h2 class="quick-links-title">"Quick actions"</h2>
            <div class="quick-links-grid">
                <a href="/l/leads"         class="quick-link">"+ New Lead"</a>
                <a href="/l/leases"        class="quick-link">"+ New Lease"</a>
                <a href="/l/vendors"       class="quick-link">"Dispatch Vendor"</a>
                <a href="/l/campaigns"     class="quick-link">"Launch Campaign"</a>
                <a href="/l/str"           class="quick-link">"STR Compliance"</a>
                <a href="/l/catalog"       class="quick-link">"Manage Catalog"</a>
            </div>
        </div>
    }
}
