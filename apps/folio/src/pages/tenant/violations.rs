// apps/folio/src/pages/tenant/violations.rs
//
// Tenant Violations — /t/violations
//
// Read-only view of compliance violations on the tenant's leases.
// Shows category, description, cure deadline, cure status, and evidence notes.
//
// Endpoint: GET /api/folio/tenant/violations
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationRecord {
    pub id:               Uuid,
    pub asset_id:         Uuid,
    pub contract_id:      Option<Uuid>,
    pub category:         String,
    pub subject:          String,
    pub description:      String,
    pub cure_days:        Option<u32>,
    pub cure_deadline:    Option<String>,
    pub evidence_notes:   Option<String>,
    pub status:           String,   // "open" | "cured" | "escalated" | "dismissed"
    pub resolution_notes: Option<String>,
    pub filed_at:         String,
    pub resolved_at:      Option<String>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchTenantViolations, "/api")]
pub async fn fetch_tenant_violations() -> Result<Vec<ViolationRecord>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<ViolationRecord>>(
        "/api/folio/tenant/violations", &token, None,
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

fn viol_status_style(status: &str) -> (&'static str, &'static str) {
    match status.to_lowercase().as_str() {
        "open"      => ("viol-badge--open",      "Open"),
        "cured"     => ("viol-badge--cured",     "✓ Cured"),
        "escalated" => ("viol-badge--escalated", "⚠ Escalated"),
        "dismissed" => ("viol-badge--dismissed", "Dismissed"),
        _           => ("viol-badge--default",   "Unknown"),
    }
}

fn viol_icon(category: &str) -> &'static str {
    match category.to_lowercase().replace('_', " ").as_str() {
        c if c.contains("noise")       => "🔊",
        c if c.contains("pet")         => "🐾",
        c if c.contains("damage")      => "🔨",
        c if c.contains("unauthorized")=> "🚫",
        c if c.contains("smoke")       => "🚬",
        c if c.contains("lease")       => "📋",
        c if c.contains("parking")     => "🚗",
        c if c.contains("trash")       => "🗑",
        _                              => "⚠️",
    }
}

fn category_label(cat: &str) -> String {
    cat.replace('_', " ")
       .split_whitespace()
       .map(|w| {
           let mut c = w.chars();
           match c.next() {
               None => String::new(),
               Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
           }
       })
       .collect::<Vec<_>>()
       .join(" ")
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantViolations() -> impl IntoView {
    let refresh      = RwSignal::new(0u32);
    let status_filter= RwSignal::new("all".to_string());
    let selected     = RwSignal::new(None::<ViolationRecord>);

    let violations_res = Resource::new(
        move || refresh.get(),
        |_| fetch_tenant_violations(),
    );

    view! {
        <div class="main-area">

            // ── Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Violations"</h1>
                    <p class="page-subtitle">"Compliance violations filed against your unit — your rights and cure deadlines"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>
                        "↻ Refresh"
                    </button>
                </div>
            </div>

            // ── Info banner ──
            <div class="viol-info-banner">
                <span class="viol-info-icon">"ℹ️"</span>
                <p class="viol-info-text">
                    "If you believe a violation was filed in error, contact your property manager directly. "
                    "Open violations must be cured within the stated deadline to avoid escalation."
                </p>
            </div>

            // ── KPI row ──
            <Suspense fallback=|| ()>
                {move || violations_res.get().map(|res| {
                    match res.as_ref() {
                        Ok(viols) => {
                            let open      = viols.iter().filter(|v| v.status == "open").count();
                            let escalated = viols.iter().filter(|v| v.status == "escalated").count();
                            let cured     = viols.iter().filter(|v| v.status == "cured").count();
                            view! {
                                <div class="kpi-row" style="margin-bottom:1.25rem;">
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Open"</span>
                                        <span class="kpi-value" style="color:var(--amber)">{open.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Escalated"</span>
                                        <span class="kpi-value" style="color:var(--red)">{escalated.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Cured"</span>
                                        <span class="kpi-value" style="color:var(--green)">{cured.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Total"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">{viols.len().to_string()}</span>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(_) => ().into_any(),
                    }
                })}
            </Suspense>

            // ── Filter pills ──
            <div class="doc-filter-row">
                {
                    let pill = move |scope: &'static str, label: &'static str| {
                        view! {
                            <button
                                class=move || format!("filter-pill {}", if status_filter.get() == scope { "filter-pill--active" } else { "" })
                                on:click=move |_| status_filter.set(scope.to_string())
                            >{label}</button>
                        }
                    };
                    view! {
                        {pill("all",        "All")}
                        {pill("open",       "Open")}
                        {pill("escalated",  "Escalated")}
                        {pill("cured",      "Cured")}
                        {pill("dismissed",  "Dismissed")}
                    }
                }
            </div>

            // ── Violation list ──
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading violations…"</div> }>
                {move || violations_res.get().map(|res| {
                    match res {
                        Ok(viols) => {
                            let sf = status_filter.get();
                            let visible: Vec<_> = viols.into_iter().filter(|v| {
                                sf == "all" || v.status == sf
                            }).collect();

                            if visible.is_empty() {
                                return view! {
                                    <div class="viol-empty">
                                        <div class="viol-empty-icon">"✅"</div>
                                        <div class="viol-empty-title">"No violations found"</div>
                                        <div class="viol-empty-sub">"You're all good. Keep it up!"</div>
                                    </div>
                                }.into_any();
                            }

                            view! {
                                <div class="viol-list">
                                    <For
                                        each=move || visible.clone()
                                        key=|v| v.id
                                        children=move |viol| {
                                            let v_click = viol.clone();
                                            let icon    = viol_icon(&viol.category);
                                            let cat     = category_label(&viol.category);
                                            let subject = viol.subject.clone();
                                            let filed   = viol.filed_at.chars().take(10).collect::<String>();
                                            let deadline= viol.cure_deadline.clone();
                                            let (sc, sl)= viol_status_style(&viol.status);
                                            let sc = sc.to_string();
                                            let sl = sl.to_string();
                                            let is_open = viol.status == "open" || viol.status == "escalated";

                                            view! {
                                                <div
                                                    class=format!("viol-card {}", if is_open { "viol-card--active" } else { "" })
                                                    on:click=move |_| selected.set(Some(v_click.clone()))
                                                >
                                                    <div class="viol-card-icon">{icon}</div>
                                                    <div class="viol-card-body">
                                                        <div class="viol-card-subject">{subject}</div>
                                                        <div class="viol-card-category">{cat}</div>
                                                        <div class="viol-card-meta">
                                                            "Filed " {filed}
                                                            {deadline.map(|d| view! { " · Cure by " <strong>{d}</strong> })}
                                                        </div>
                                                    </div>
                                                    <span class=format!("viol-badge {sc}")>{sl}</span>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="doc-empty text-red-400">"Error: " {e.to_string()}</div>
                        }.into_any(),
                    }
                })}
            </Suspense>

            // ── Violation detail modal ───────────────────────────────────────
            <Show when=move || selected.get().is_some()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:34rem;">
                        {move || selected.get().map(|v| {
                            let icon    = viol_icon(&v.category);
                            let cat     = category_label(&v.category);
                            let subject = v.subject.clone();
                            let (sc, sl)= viol_status_style(&v.status);
                            let sc = sc.to_string();
                            let sl = sl.to_string();

                            view! {
                                <div class="modal-header">
                                    <h3 class="modal-title">{icon} " " {subject.clone()}</h3>
                                    <button class="modal-close" on:click=move |_| selected.set(None)>"✕"</button>
                                </div>
                                <div class="modal-body space-y-4">
                                    <div class="flex items-center gap-2">
                                        <span class=format!("viol-badge {sc}")>{sl}</span>
                                        <span class="text-xs text-on-surface-variant">{cat}</span>
                                    </div>

                                    <dl class="doc-detail-list">
                                        <dt>"Filed"</dt><dd>{v.filed_at.chars().take(10).collect::<String>()}</dd>
                                        {v.cure_days.map(|d| view! {
                                            <dt>"Cure Period"</dt><dd>{d.to_string()} " days"</dd>
                                        })}
                                        {v.cure_deadline.clone().map(|d| view! {
                                            <dt>"Deadline"</dt><dd><strong>{d}</strong></dd>
                                        })}
                                        {v.resolved_at.clone().map(|d| view! {
                                            <dt>"Resolved"</dt><dd>{d.chars().take(10).collect::<String>()}</dd>
                                        })}
                                    </dl>

                                    <div class="viol-detail-section">
                                        <div class="viol-detail-label">"Description"</div>
                                        <p class="viol-detail-body">{v.description.clone()}</p>
                                    </div>

                                    {v.evidence_notes.clone().map(|en| view! {
                                        <div class="viol-detail-section">
                                            <div class="viol-detail-label">"Evidence Notes (by property manager)"</div>
                                            <p class="viol-detail-body">{en}</p>
                                        </div>
                                    })}

                                    {v.resolution_notes.clone().map(|rn| view! {
                                        <div class="viol-detail-section">
                                            <div class="viol-detail-label">"Resolution Notes"</div>
                                            <p class="viol-detail-body">{rn}</p>
                                        </div>
                                    })}

                                    {if v.status == "open" || v.status == "escalated" {
                                        view! {
                                            <div class="viol-cure-prompt">
                                                <strong>"How to cure this violation:"</strong>
                                                <p>"Address the issue described above within the cure period and notify your property manager with documentation (photos, receipts, etc.). Contact Atlas Support if you need help."</p>
                                            </div>
                                        }.into_any()
                                    } else { ().into_any() }}
                                </div>
                                <div class="modal-footer">
                                    <button class="btn btn-ghost" on:click=move |_| selected.set(None)>"Close"</button>
                                </div>
                            }
                        })}
                    </div>
                </div>
            </Show>

        </div>
    }
}
