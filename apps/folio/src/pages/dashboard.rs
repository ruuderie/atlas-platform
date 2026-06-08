use leptos::prelude::*;

/// Dashboard home — portfolio summary cards.
#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div class="page-header">
            <h1 class="page-title">"Overview"</h1>
        </div>
        <div class="stat-grid">
            <StatCard label="Properties"    value="—" icon="🏠"/>
            <StatCard label="Active Leases" value="—" icon="📋"/>
            <StatCard label="Open Leads"    value="—" icon="👤"/>
            <StatCard label="Revenue MTD"   value="—" icon="💰"/>
        </div>
        // TODO: wire up to /api/folio/* endpoints
    }
}

#[component]
fn StatCard(label: &'static str, value: &'static str, icon: &'static str) -> impl IntoView {
    view! {
        <div class="stat-card">
            <span class="stat-icon">{icon}</span>
            <div class="stat-body">
                <p class="stat-label">{label}</p>
                <p class="stat-value">{value}</p>
            </div>
        </div>
    }
}
