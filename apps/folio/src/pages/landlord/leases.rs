// apps/folio/src/pages/landlord/leases.rs
//
// Leases list page.
//
// Lists all atlas_contract records for the tenant via GET /api/folio/leases.
// Each row links to the Lease Detail page (/l/leases/:id).

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::nav::{FolioRoute, NavIcon};

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaseSummary {
    pub id: uuid::Uuid,
    pub asset_id: Option<uuid::Uuid>,
    pub counterparty_user_id: Option<uuid::Uuid>,
    pub currency: String,
    pub status: String,
    pub monthly_rent_cents: Option<i64>,
    pub start_date: Option<chrono::NaiveDate>,
    pub end_date: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Lease contract status — mirrors atlas_contract.status values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeaseStatus {
    Active,
    Draft,
    Pending,
    Expired,
    Terminated,
    Unknown,
}

impl LeaseStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "draft" => Self::Draft,
            "pending" => Self::Pending,
            "expired" => Self::Expired,
            "terminated" => Self::Terminated,
            _ => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Draft => "Draft",
            Self::Pending => "Pending",
            Self::Expired => "Expired",
            Self::Terminated => "Terminated",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Active => "lease-status--active",
            Self::Draft => "lease-status--draft",
            Self::Pending => "lease-status--pending",
            Self::Expired => "lease-status--expired",
            Self::Terminated => "lease-status--terminated",
            Self::Unknown => "lease-status--unknown",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Active => "verified",
            Self::Draft => "edit_document",
            Self::Pending => "pending",
            Self::Expired => "event_busy",
            Self::Terminated => "cancel",
            Self::Unknown => "help",
        }
    }
}

// ── Page ─────────────────────────────────────────────────────────────────────

/// Lease list — gives the landlord a snapshot of every active rental contract,
/// monthly rent, tenant, and quick-links into the detail page.
#[component]
pub fn Leases() -> impl IntoView {
    let (status_filter, set_status_filter) = signal(LeaseStatus::Active);
    let (search_query, set_search_query) = signal(String::new());

    let leases = Resource::new(|| (), |_| async move { list_leases().await });

    view! {
        <div class="leases-page">
            <div class="leases-header">
                <div>
                    <h1 class="leases-title">"Leases"</h1>
                    <p class="leases-subtitle">"Rental contracts across your portfolio."</p>
                </div>
            </div>

            // ── Filter bar ────────────────────────────────────────────
            <div class="leases-filter-bar">
                <div class="leases-search-wrap">
                    <span class="material-symbols-outlined leases-search-icon">"search"</span>
                    <input
                        id="lease-search"
                        class="leases-search-input"
                        type="search"
                        placeholder="Search by asset or status\u{2026}"
                        on:input=move |e| set_search_query.set(event_target_value(&e))
                    />
                </div>
                <div class="leases-status-chips">
                    {[
                        LeaseStatus::Active,
                        LeaseStatus::Draft,
                        LeaseStatus::Pending,
                        LeaseStatus::Expired,
                        LeaseStatus::Terminated,
                    ].iter().copied().map(|s| view! {
                        <button
                            class=move || {
                                if status_filter.get() == s {
                                    "lease-status-chip lease-status-chip--active"
                                } else {
                                    "lease-status-chip"
                                }
                            }
                            on:click=move |_| set_status_filter.set(s)
                        >
                            <span class="material-symbols-outlined" style="font-size:13px;">
                                {s.material_icon()}
                            </span>
                            {s.as_str()}
                        </button>
                    }).collect_view()}
                </div>
            </div>

            // ── Table / list ──────────────────────────────────────────
            <Suspense fallback=|| view! { <LeasesTableSkeleton/> }>
                {move || leases.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="leases-error">
                            <span class="material-symbols-outlined" style="font-size:2.5rem;color:var(--folio-muted);">
                                {NavIcon::Report.as_str()}
                            </span>
                            <p class="leases-error-text">{format!("Could not load leases: {e}")}</p>
                        </div>
                    }.into_any(),
                    Ok(all) => {
                        let q  = search_query.get().to_lowercase();
                        let sf = status_filter.get();
                        let filtered: Vec<LeaseSummary> = all.into_iter().filter(|l| {
                            let status_ok = LeaseStatus::from_str(&l.status) == sf;
                            let search_ok = q.is_empty()
                                || l.status.to_lowercase().contains(&q)
                                || l.id.to_string().contains(&q);
                            status_ok && search_ok
                        }).collect();

                        if filtered.is_empty() {
                            view! {
                                <div class="leases-empty">
                                    <span class="material-symbols-outlined leases-empty-icon">
                                        {NavIcon::Description.as_str()}
                                    </span>
                                    <p class="leases-empty-title">"No leases found"</p>
                                    <p class="leases-empty-sub">
                                        "Try a different status filter or create a new lease."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="leases-table-wrap">
                                    <table class="leases-table">
                                        <thead>
                                            <tr>
                                                <th class="leases-th">"Status"</th>
                                                <th class="leases-th">"Asset"</th>
                                                <th class="leases-th">"Monthly Rent"</th>
                                                <th class="leases-th">"Start"</th>
                                                <th class="leases-th">"End"</th>
                                                <th class="leases-th">"Created"</th>
                                                <th class="leases-th"></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {filtered.into_iter().map(|l| {
                                                let status = LeaseStatus::from_str(&l.status);
                                                let id_str = l.id.to_string();
                                                let detail_href = format!("/l/leases/{id_str}");
                                                let rent = l.monthly_rent_cents.map(|c| {
                                                    format!("{} {:.2}", l.currency, c as f64 / 100.0)
                                                }).unwrap_or_else(|| "\u{2014}".to_string());
                                                let start = l.start_date
                                                    .map(|d| d.to_string())
                                                    .unwrap_or_else(|| "\u{2014}".to_string());
                                                let end = l.end_date
                                                    .map(|d| d.to_string())
                                                    .unwrap_or_else(|| "Open".to_string());
                                                let created = l.created_at.format("%Y-%m-%d").to_string();
                                                view! {
                                                    <tr class="leases-row">
                                                        <td class="leases-td">
                                                            <span class={format!("lease-status-badge {}", status.pill_class())}>
                                                                <span class="material-symbols-outlined" style="font-size:11px;">
                                                                    {status.material_icon()}
                                                                </span>
                                                                {status.as_str()}
                                                            </span>
                                                        </td>
                                                        <td class="leases-td leases-td--mono">
                                                            {l.asset_id.map(|id| id.to_string()).unwrap_or_else(|| "\u{2014}".to_string())}
                                                        </td>
                                                        <td class="leases-td leases-td--amount">{rent}</td>
                                                        <td class="leases-td">{start}</td>
                                                        <td class="leases-td">{end}</td>
                                                        <td class="leases-td">{created}</td>
                                                        <td class="leases-td">
                                                            <a href=detail_href class="leases-detail-link">
                                                                "View"
                                                                <span class="material-symbols-outlined" style="font-size:14px;">
                                                                    {NavIcon::ChevronRight.as_str()}
                                                                </span>
                                                            </a>
                                                        </td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

// ── Skeleton ──────────────────────────────────────────────────────────────────

#[component]
fn LeasesTableSkeleton() -> impl IntoView {
    view! {
        <div class="leases-table-wrap">
            <table class="leases-table">
                <thead>
                    <tr>
                        <th class="leases-th">"Status"</th>
                        <th class="leases-th">"Asset"</th>
                        <th class="leases-th">"Monthly Rent"</th>
                        <th class="leases-th">"Start"</th>
                        <th class="leases-th">"End"</th>
                        <th class="leases-th">"Created"</th>
                        <th class="leases-th"></th>
                    </tr>
                </thead>
                <tbody>
                    {(0..6usize).map(|_| view! {
                        <tr class="leases-row">
                            <td class="leases-td"><div class="leases-skel leases-skel--badge"/></td>
                            <td class="leases-td"><div class="leases-skel leases-skel--text"/></td>
                            <td class="leases-td"><div class="leases-skel leases-skel--text"/></td>
                            <td class="leases-td"><div class="leases-skel leases-skel--text"/></td>
                            <td class="leases-td"><div class="leases-skel leases-skel--text"/></td>
                            <td class="leases-td"><div class="leases-skel leases-skel--text"/></td>
                            <td class="leases-td"><div class="leases-skel leases-skel--link"/></td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

// ── Server functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get(axum::http::header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies.split(';').find_map(|part| {
                        part.trim()
                            .strip_prefix("atlas_session=")
                            .map(|t| t.to_string())
                    })
                })
        })
}

/// GET /api/folio/leases
#[server(ListLeases, "/api")]
pub async fn list_leases() -> Result<Vec<LeaseSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<LeaseSummary>>("/api/folio/leases", &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Lease list failed: {e}")))
}
