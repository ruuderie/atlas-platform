//! Owner portal dashboard — `/o`
//! Wired to `GET /api/folio/owner/summary`.

use crate::auth::{ServerFnError, SessionInfo};
use crate::components::nav::FolioRoute;
use crate::components::network_invite_panel::{AngleCard, NetworkInvitePanel};
use crate::pages::owner::statements::{fetch_owner_summary_for_statements, OwnerPortfolioSummary};
use leptos::prelude::*;

#[component]
pub fn OwnerDashboard() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, ServerFnError>>>()
        .expect("Session context missing");
    let name = move || {
        session
            .get()
            .and_then(|r| r.ok())
            .and_then(|s| s.display_name)
            .unwrap_or_else(|| "there".into())
    };

    let summary = Resource::new(|| (), |_| async move { fetch_owner_summary_for_statements().await });

    let props = Signal::derive(move || {
        summary
            .get()
            .and_then(|r| r.ok())
            .map(|s: OwnerPortfolioSummary| s.total_properties.to_string())
            .unwrap_or_else(|| "—".into())
    });
    let occ = Signal::derive(move || {
        summary
            .get()
            .and_then(|r| r.ok())
            .map(|s| format!("{:.0}%", s.occupancy_pct))
            .unwrap_or_else(|| "—".into())
    });
    let open_wo = Signal::derive(move || {
        summary
            .get()
            .and_then(|r| r.ok())
            .map(|s| s.open_maintenance_cases.to_string())
            .unwrap_or_else(|| "—".into())
    });
    let revenue = Signal::derive(move || {
        summary
            .get()
            .and_then(|r| r.ok())
            .map(|s| format!("${:.0}", s.revenue_this_month_cents as f64 / 100.0))
            .unwrap_or_else(|| "—".into())
    });

    view! {
        <div class="landlord-list-page">
            <div class="page-header">
                <h1 class="page-title">{move || format!("Owner Portal - {}", name())}</h1>
                <p class="page-subtitle">
                    "Your properties. Contact your manager to make changes."
                </p>
            </div>

            <Suspense fallback=|| view! { <div class="folio-empty"><p class="folio-empty__sub">"Loading…"</p></div> }>
                {move || match summary.get() {
                    Some(Err(e)) => view! {
                        <div class="folio-empty">
                            <p class="folio-empty__heading">"Could not load portfolio summary"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <div class="stat-grid">
                            <a href=FolioRoute::OwnerProperties.path() class="stat-card stat-card--link">
                                <span class="stat-icon">"🏠"</span>
                                <div class="stat-body">
                                    <p class="stat-label">"Properties"</p>
                                    <p class="stat-value">{move || props.get()}</p>
                                </div>
                            </a>
                            <a href=FolioRoute::OwnerProperties.path() class="stat-card stat-card--link">
                                <span class="stat-icon">"📊"</span>
                                <div class="stat-body">
                                    <p class="stat-label">"Occupancy"</p>
                                    <p class="stat-value">{move || occ.get()}</p>
                                </div>
                            </a>
                            <a href=FolioRoute::OwnerMaintenance.path() class="stat-card stat-card--link">
                                <span class="stat-icon">"🔧"</span>
                                <div class="stat-body">
                                    <p class="stat-label">"Open maintenance"</p>
                                    <p class="stat-value">{move || open_wo.get()}</p>
                                </div>
                            </a>
                            <a href=FolioRoute::OwnerStatements.path() class="stat-card stat-card--link">
                                <span class="stat-icon">"💵"</span>
                                <div class="stat-body">
                                    <p class="stat-label">"Revenue MTD"</p>
                                    <p class="stat-value">{move || revenue.get()}</p>
                                </div>
                            </a>
                        </div>
                    }.into_any(),
                }}
            </Suspense>

            <div style="margin-top:20px;">
                <h2 style="font-size:18px;font-weight:700;margin:0 0 6px;">"Invite an owner"</h2>
                <p style="font-size:14px;color:#64748b;margin:0 0 14px;line-height:1.5;max-width:560px;">
                    "Share this portal with fellow investors, or introduce Folio to a self-managed landlord in your circle."
                </p>
                <NetworkInvitePanel
                    actor_role="owner"
                    preferred_slug="owner_invite_peers"
                    angles=vec![
                        AngleCard {
                            icon: "star",
                            title: "Other managed owners",
                            body: "Fellow investors get the same visibility into statements and approvals.",
                            benefit_icon: None,
                            benefit_label: None,
                        },
                        AngleCard {
                            icon: "apartment",
                            title: "Self-managed landlords",
                            body: "Share Folio with landlords who still track rent in spreadsheets.",
                            benefit_icon: None,
                            benefit_label: None,
                        },
                    ]
                    allow_multi=false
                    send_label="Send invite".to_string()
                    show_history=true
                />
            </div>
        </div>
    }
}
