// apps/folio/src/pages/pmc/client_detail.rs
//
// PMC Client Detail — /pmc/clients/:id
//
// Full portfolio snapshot for a single client account.
// Shows all properties, active leases, and quick-action links.
// Data from /api/folio/pm/clients/{account_id}.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientDetail {
    pub account_id:    Uuid,
    pub display_name:  String,
    pub portfolio_ids: Vec<Uuid>,
    pub active_leases: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSummary {
    pub account_id:         Uuid,
    pub display_name:       String,
    pub contact_name:       Option<String>,
    pub contact_email:      Option<String>,
    pub property_count:     Option<i64>,
    pub unit_count:         Option<i64>,
    pub active_lease_count: Option<i64>,
    pub occupancy_pct:      Option<f64>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchPmcClientDetail, "/api")]
pub async fn fetch_pmc_client_detail(account_id: String) -> Result<ClientDetail, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/folio/pm/clients/{account_id}");
    crate::atlas_client::authenticated_get::<ClientDetail>(&url, &token, None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchPmcClients, "/api")]
pub async fn fetch_pmc_clients() -> Result<Vec<ClientSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<ClientSummary>>(
        "/api/folio/pm/clients", &token, None,
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

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn PmcClientDetail() -> impl IntoView {
    let params     = use_params_map();
    let account_id = params.get().get("id").cloned().unwrap_or_default();
    let aid2       = account_id.clone();
    let aid3       = account_id.clone();

    let detail_res  = Resource::new(
        move || account_id.clone(),
        |id| fetch_pmc_client_detail(id),
    );
    let clients_res = Resource::new(|| (), |_| fetch_pmc_clients());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <a href="/pmc/clients" class="back-link">"← Client Book"</a>
                    <h1 class="page-title">"Client Detail"</h1>
                    <p class="page-subtitle">"Full portfolio snapshot for this managed account"</p>
                </div>
                <div class="page-actions">
                    <a href="/pmc/maintenance" class="btn btn-ghost btn-sm">"🔧 Maintenance"</a>
                    <button class="btn btn-primary btn-sm" disabled=true title="Owner link requires PMC admin">"+ Add Owner Link"</button>
                </div>
            </div>

            // ── Client info header ──
            <Suspense fallback=|| ()>
                {move || clients_res.get().map(|res| {
                    if let Ok(clients) = res {
                        let id_str = aid2.clone();
                        let client = Uuid::parse_str(&id_str).ok()
                            .and_then(|uid| clients.into_iter().find(|c| c.account_id == uid));
                        if let Some(c) = client {
                            let name  = c.display_name.clone();
                            let email = c.contact_email.clone();
                            let contact_name = c.contact_name.clone();
                            return view! {
                                <div class="pmc-client-header">
                                    <div class="pmc-client-avatar">{
                                        name.chars().next().map(|ch| ch.to_string()).unwrap_or_else(|| "?".to_string())
                                    }</div>
                                    <div class="pmc-client-info">
                                        <div class="pmc-client-name">{name}</div>
                                        {contact_name.map(|n| view! { <div class="pmc-client-meta">"Contact: " {n}</div> })}
                                        {email.map(|e| view! { <div class="pmc-client-meta">"✉ " {e}</div> })}
                                    </div>
                                </div>
                                <div class="kpi-row" style="margin-bottom:1.25rem;">
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Properties"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">
                                            {c.property_count.unwrap_or(0).to_string()}
                                        </span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Units"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">
                                            {c.unit_count.unwrap_or(0).to_string()}
                                        </span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Active Leases"</span>
                                        <span class="kpi-value" style="color:var(--green)">
                                            {c.active_lease_count.unwrap_or(0).to_string()}
                                        </span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Occupancy"</span>
                                        <span class="kpi-value" style="color:var(--green)">
                                            {c.occupancy_pct.map(|p| format!("{:.0}%", p)).unwrap_or_else(|| "—".to_string())}
                                        </span>
                                    </div>
                                </div>
                            }.into_any();
                        }
                    }
                    ().into_any()
                })}
            </Suspense>

            // ── Portfolio IDs ──
            <div class="owner-section">
                <div class="owner-section-title">"Managed Portfolios"</div>
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading…"</div> }>
                    {move || detail_res.get().map(|res| {
                        match res {
                            Ok(detail) if !detail.portfolio_ids.is_empty() => view! {
                                <div class="pmc-portfolio-grid">
                                    <For
                                        each=move || detail.portfolio_ids.clone()
                                        key=|id| *id
                                        children=move |pid| {
                                            let pid_str = pid.to_string();
                                            let short   = format!("…{}", &pid_str[24..]);
                                            view! {
                                                <div class="pmc-portfolio-chip">
                                                    <span class="pmc-portfolio-chip-icon">"🏘"</span>
                                                    <span class="pmc-portfolio-chip-id font-mono text-xs">{short}</span>
                                                    <a href=format!("/l/portfolio") class="btn btn-ghost btn-sm">"View"</a>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any(),
                            Ok(_) => view! {
                                <div class="doc-empty">
                                    "No portfolios linked. Use '+ Add Owner Link' to associate properties."
                                </div>
                            }.into_any(),
                            Err(e) => view! {
                                <div class="doc-empty text-red-400">"Error: " {e.to_string()}</div>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </div>

            // ── Quick actions ──
            <div class="owner-section">
                <div class="owner-section-title">"Actions"</div>
                <div class="pmc-action-row">
                    <a href=format!("/pmc/statements") class="pmc-action-card">
                        <span class="pmc-action-icon">"📄"</span>
                        <span>"Owner Statements"</span>
                    </a>
                    <a href=format!("/pmc/maintenance") class="pmc-action-card">
                        <span class="pmc-action-icon">"🔧"</span>
                        <span>"Maintenance"</span>
                    </a>
                    <a href=format!("/l/ledger") class="pmc-action-card">
                        <span class="pmc-action-icon">"💰"</span>
                        <span>"Ledger"</span>
                    </a>
                </div>
            </div>
        </div>
    }
}
