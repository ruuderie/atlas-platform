// apps/folio/src/pages/owner/property.rs
//
// Owner Property Detail — /o/properties/:id
//
// Read-only detail view for a single owned property.
// Shows revenue, leases, maintenance, and violations for the asset.
// Data from /api/folio/owner/properties (filtered client-side by asset_id).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerPropertySummary {
    pub asset_id:                  Uuid,
    pub asset_name:                String,
    pub asset_type:                String,
    pub address_line_1:            Option<String>,
    pub active_leases:             usize,
    pub open_maintenance:          usize,
    pub open_violations:           usize,
    pub revenue_this_month_cents:  i64,
    pub outstanding_balance_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerLeaseEntry {
    pub contract_id:          Uuid,
    pub asset_id:             Option<Uuid>,
    pub start_date:           String,
    pub end_date:             Option<String>,
    pub status:               String,
    pub monthly_rent_cents:   Option<i64>,
    pub currency:             String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerMaintenanceSummary {
    pub case_id:      Uuid,
    pub asset_id:     Option<Uuid>,
    pub subject:      String,
    pub priority:     String,
    pub status:       String,
    pub created_at:   String,
    pub completed_at: Option<String>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchOwnerProperties, "/api")]
pub async fn fetch_owner_properties() -> Result<Vec<OwnerPropertySummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<OwnerPropertySummary>>(
        "/api/folio/owner/properties", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchOwnerLeases, "/api")]
pub async fn fetch_owner_leases() -> Result<Vec<OwnerLeaseEntry>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<OwnerLeaseEntry>>(
        "/api/folio/owner/leases", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchOwnerMaintenance, "/api")]
pub async fn fetch_owner_maintenance() -> Result<Vec<OwnerMaintenanceSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<OwnerMaintenanceSummary>>(
        "/api/folio/owner/maintenance", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[cfg(feature = "ssr")]
fn session_token(headers: &axum::http::HeaderMap) -> Result<String, server_fn::error::ServerFnError> {
    headers.get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find_map(|p| {
            let p = p.trim();
            p.strip_prefix("session=").map(|t| t.to_string())
        }))
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmt_k(cents: i64) -> String {
    format!("${:.0}", cents as f64 / 100.0)
}

fn status_chip_class(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "active" | "paid" | "resolved"  => "owner-chip--green",
        "pending" | "open" | "submitted"=> "owner-chip--amber",
        "overdue" | "escalated"         => "owner-chip--red",
        _                               => "owner-chip--grey",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn OwnerPropertyDetail() -> impl IntoView {
    let params   = use_params_map();
    let asset_id = params.get().get("id")
        .and_then(|s| Uuid::parse_str(s).ok());

    let props_res = Resource::new(|| (), |_| fetch_owner_properties());
    let leases_res= Resource::new(|| (), |_| fetch_owner_leases());
    let maint_res = Resource::new(|| (), |_| fetch_owner_maintenance());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <a href="/o" class="back-link">"← Owner Dashboard"</a>
                    <h1 class="page-title">"Property Detail"</h1>
                    <p class="page-subtitle">"Read-only view — contact your property manager for changes"</p>
                </div>
                <div class="page-actions">
                    <span class="owner-readonly-badge">"👁 Read Only"</span>
                </div>
            </div>

            // ── Property KPIs ──
            <Suspense fallback=|| ()>
                {move || props_res.get().map(|res| {
                    if let Ok(props) = res {
                        let prop = asset_id.and_then(|aid| props.into_iter().find(|p| p.asset_id == aid));
                        if let Some(p) = prop {
                            let name    = p.asset_name.clone();
                            let addr    = p.address_line_1.clone().unwrap_or_else(|| "—".to_string());
                            let atype   = p.asset_type.replace('_', " ");
                            return view! {
                                <div class="owner-prop-header">
                                    <div class="owner-prop-icon">"🏠"</div>
                                    <div>
                                        <div class="owner-prop-name">{name}</div>
                                        <div class="owner-prop-meta">{atype} " · " {addr}</div>
                                    </div>
                                </div>
                                <div class="kpi-row" style="margin-bottom:1.25rem;">
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Revenue This Month"</span>
                                        <span class="kpi-value" style="color:var(--green)">{fmt_k(p.revenue_this_month_cents)}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Outstanding"</span>
                                        <span class="kpi-value" style="color:var(--amber)">{fmt_k(p.outstanding_balance_cents)}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Active Leases"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">{p.active_leases.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Open Maintenance"</span>
                                        <span class="kpi-value" style="color:var(--amber)">{p.open_maintenance.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Open Violations"</span>
                                        <span class="kpi-value" style="color:var(--red)">{p.open_violations.to_string()}</span>
                                    </div>
                                </div>
                            }.into_any();
                        }
                    }
                    ().into_any()
                })}
            </Suspense>

            // ── Leases ──
            <div class="owner-section">
                <div class="owner-section-title">"Active Leases"</div>
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading…"</div> }>
                    {move || leases_res.get().map(|res| {
                        match res {
                            Ok(leases) => {
                                let filtered: Vec<_> = leases.into_iter()
                                    .filter(|l| asset_id.map(|aid| l.asset_id == Some(aid)).unwrap_or(true))
                                    .collect();
                                if filtered.is_empty() {
                                    return view! { <div class="doc-empty">"No leases for this property."</div> }.into_any();
                                }
                                view! {
                                    <div class="owner-table-wrap">
                                        <table class="ph-table">
                                            <thead><tr>
                                                <th>"Start"</th><th>"End"</th>
                                                <th>"Rent/mo"</th><th>"Status"</th>
                                            </tr></thead>
                                            <tbody>
                                                <For each=move || filtered.clone() key=|l| l.contract_id children=move |l| {
                                                    let sc = status_chip_class(&l.status);
                                                    let rent = l.monthly_rent_cents.map(|r| fmt_k(r)).unwrap_or_else(|| "—".to_string());
                                                    view! {
                                                        <tr class="ph-row">
                                                            <td class="ph-date">{l.start_date.chars().take(10).collect::<String>()}</td>
                                                            <td class="ph-date">{l.end_date.map(|d| d.chars().take(10).collect::<String>()).unwrap_or_else(|| "—".to_string())}</td>
                                                            <td class="ph-amount">{rent}</td>
                                                            <td><span class=format!("owner-chip {sc}")>{l.status}</span></td>
                                                        </tr>
                                                    }
                                                }/>
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                            Err(_) => view! { <div class="doc-empty text-red-400">"Error loading leases."</div> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>

            // ── Maintenance ──
            <div class="owner-section">
                <div class="owner-section-title">"Maintenance Cases"</div>
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading…"</div> }>
                    {move || maint_res.get().map(|res| {
                        match res {
                            Ok(cases) => {
                                let filtered: Vec<_> = cases.into_iter()
                                    .filter(|c| asset_id.map(|aid| c.asset_id == Some(aid)).unwrap_or(true))
                                    .collect();
                                if filtered.is_empty() {
                                    return view! { <div class="doc-empty">"No maintenance cases. 🎉"</div> }.into_any();
                                }
                                view! {
                                    <div class="viol-list">
                                        <For each=move || filtered.clone() key=|c| c.case_id children=move |c| {
                                            let sc = status_chip_class(&c.status);
                                            let date = c.created_at.chars().take(10).collect::<String>();
                                            view! {
                                                <div class="viol-card">
                                                    <div class="viol-card-icon">"🔧"</div>
                                                    <div class="viol-card-body">
                                                        <div class="viol-card-subject">{c.subject}</div>
                                                        <div class="viol-card-meta">"Filed " {date} " · Priority: " {c.priority}</div>
                                                    </div>
                                                    <span class=format!("owner-chip {sc}")>{c.status}</span>
                                                </div>
                                            }
                                        }/>
                                    </div>
                                }.into_any()
                            }
                            Err(_) => view! { <div class="doc-empty text-red-400">"Error loading maintenance."</div> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}
