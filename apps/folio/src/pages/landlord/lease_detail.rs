// apps/folio/src/pages/landlord/lease_detail.rs
//
// Lease Detail page — /l/leases/:id
//
// Displays the full lease contract for the operator:
//   - Financial terms (rent, interval, currency, guarantee)
//   - Timeline (start / end / signed / terminated)
//   - Linked household occupants (GET /api/folio/leases/:id/occupants)
//   - Registered vehicles (GET /api/folio/leases/:id/vehicles)
//   - Payment history (GET /api/folio/leases/:id/invoices)
//
// All non-trivial data fetches are parallel Resources keyed off the lease ID.
// The route param is read via leptos_router::hooks::use_params_map().

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::components::nav::{FolioRoute, NavIcon};
use crate::pages::landlord::leases::LeaseStatus;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaseDetail {
    pub id: uuid::Uuid,
    pub asset_id: Option<uuid::Uuid>,
    pub counterparty_user_id: Option<uuid::Uuid>,
    pub contract_type: String,
    pub recurring_amount_cents: Option<i64>,
    pub currency: String,
    pub billing_interval: String,
    pub status: String,
    pub guarantee_type: Option<String>,
    pub auto_renew: bool,
    pub start_date: chrono::NaiveDate,
    pub end_date: Option<chrono::NaiveDate>,
    pub signed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub terminated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub termination_reason: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OccupantRecord {
    pub id: uuid::Uuid,
    pub full_name: String,
    pub kind: String,
    pub relationship: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VehicleRecord {
    pub id: uuid::Uuid,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: String,
    pub license_plate: String,
    pub parking_spot: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaseInvoiceSummary {
    pub id: uuid::Uuid,
    pub gross_amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub payment_rail: Option<String>,
    pub due_date: Option<chrono::NaiveDate>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Occupants API wraps active list.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OccupantsResponse {
    pub active: Vec<OccupantRecord>,
    pub former: Vec<OccupantRecord>,
}

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Billing interval — mirrors atlas_contract.billing_interval values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BillingInterval {
    Monthly,
    Weekly,
    BiWeekly,
    Annual,
    Unknown,
}

impl BillingInterval {
    pub fn from_str(s: &str) -> Self {
        match s {
            "monthly" => Self::Monthly,
            "weekly" => Self::Weekly,
            "bi_weekly" => Self::BiWeekly,
            "annual" => Self::Annual,
            _ => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Monthly => "Monthly",
            Self::Weekly => "Weekly",
            Self::BiWeekly => "Bi-weekly",
            Self::Annual => "Annual",
            Self::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for BillingInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Invoice payment status — mirrors atlas_ledger_entries.status values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvoiceStatus {
    Pending,
    Processing,
    Paid,
    Failed,
    Refunded,
    Unknown,
}

impl InvoiceStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "pending" => Self::Pending,
            "processing" => Self::Processing,
            "paid" => Self::Paid,
            "failed" => Self::Failed,
            "refunded" => Self::Refunded,
            _ => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Processing => "Processing",
            Self::Paid => "Paid",
            Self::Failed => "Failed",
            Self::Refunded => "Refunded",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Paid => "inv-status--paid",
            Self::Pending => "inv-status--pending",
            Self::Processing => "inv-status--processing",
            Self::Failed => "inv-status--failed",
            Self::Refunded => "inv-status--refunded",
            Self::Unknown => "inv-status--unknown",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Paid => "check_circle",
            Self::Pending => "schedule",
            Self::Processing => "sync",
            Self::Failed => "cancel",
            Self::Refunded => "undo",
            Self::Unknown => "help",
        }
    }
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Page ──────────────────────────────────────────────────────────────────────

/// Full lease contract view — financial terms, household, vehicles, payment history.
#[component]
pub fn LeaseDetail() -> impl IntoView {
    let params = use_params_map();
    let lease_id = move || params.with(|p| p.get("id").unwrap_or_default());

    let detail = Resource::new(lease_id, |id| async move { get_lease_detail(id).await });

    let occupants = Resource::new(lease_id, |id| async move { get_lease_occupants(id).await });

    let vehicles = Resource::new(lease_id, |id| async move { get_lease_vehicles(id).await });

    let invoices = Resource::new(lease_id, |id| async move { get_lease_invoices(id).await });

    view! {
        <div class="ld-page">
            // ── Back nav ──────────────────────────────────────────────
            <a href=FolioRoute::LandlordLeases.path() class="ld-back-link">
                <span class="material-symbols-outlined" style="font-size:16px;">
                    {NavIcon::ArrowBack.as_str()}
                </span>
                "Back to Leases"
            </a>

            // ── Header ────────────────────────────────────────────────
            <Suspense fallback=|| view! { <div class="ld-header-skeleton"/> }>
                {move || detail.get().map(|res| match res {
                    Err(e) => view! {
                        <div class="ld-error">
                            <p class="ld-error-text">{format!("Could not load lease: {e}")}</p>
                        </div>
                    }.into_any(),
                    Ok(d) => {
                        let status = LeaseStatus::from_str(&d.status);
                        let rent = d.recurring_amount_cents.map(|c| {
                            format!("{} {:.2}", d.currency, c as f64 / 100.0)
                        }).unwrap_or_else(|| "\u{2014}".to_string());
                        let interval = BillingInterval::from_str(&d.billing_interval);
                        let end_str = d.end_date
                            .map(|dt| dt.to_string())
                            .unwrap_or_else(|| "Open-ended".to_string());
                        let signed = d.signed_at
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_else(|| "Unsigned".to_string());

                        view! {
                            <div class="ld-header">
                                <div class="ld-header-left">
                                    <div class="ld-avatar">
                                        <span class="material-symbols-outlined" style="font-size:1.6rem;">
                                            {NavIcon::Description.as_str()}
                                        </span>
                                    </div>
                                    <div>
                                        <div class="ld-header-title-row">
                                            <h1 class="ld-title">
                                                "Lease "
                                                <span class="ld-title-id">
                                                    {d.id.to_string().split('-').next().unwrap_or("").to_uppercase()}
                                                </span>
                                            </h1>
                                            <span class={format!("lease-status-badge {}", status.pill_class())}>
                                                <span class="material-symbols-outlined" style="font-size:11px;">
                                                    {status.material_icon()}
                                                </span>
                                                {status.as_str()}
                                            </span>
                                        </div>
                                        <p class="ld-subtitle">
                                            {format!("{} \u{00b7} Signed {}", d.contract_type.replace('_', " "), signed)}
                                        </p>
                                    </div>
                                </div>
                                <div class="ld-header-kpis">
                                    <div class="ld-kpi">
                                        <span class="ld-kpi-label">"Monthly Rent"</span>
                                        <span class="ld-kpi-value">{rent}</span>
                                        <span class="ld-kpi-sub">{format!("billed {}", interval)}</span>
                                    </div>
                                    <div class="ld-kpi">
                                        <span class="ld-kpi-label">"Start Date"</span>
                                        <span class="ld-kpi-value">{d.start_date.to_string()}</span>
                                    </div>
                                    <div class="ld-kpi">
                                        <span class="ld-kpi-label">"End Date"</span>
                                        <span class="ld-kpi-value">{end_str}</span>
                                        {d.auto_renew.then(|| view! {
                                            <span class="ld-kpi-sub ld-kpi-sub--green">
                                                <span class="material-symbols-outlined" style="font-size:11px;">"autorenew"</span>
                                                "Auto-renews"
                                            </span>
                                        })}
                                    </div>
                                    {d.guarantee_type.map(|gt| view! {
                                        <div class="ld-kpi">
                                            <span class="ld-kpi-label">"Guarantee"</span>
                                            <span class="ld-kpi-value ld-kpi-value--sm">
                                                {gt.replace('_', " ")}
                                            </span>
                                        </div>
                                    })}
                                </div>
                            </div>

                            // Termination banner
                            {d.terminated_at.map(|_| view! {
                                <div class="ld-termination-banner">
                                    <span class="material-symbols-outlined" style="font-size:16px;">"cancel"</span>
                                    <span>
                                        "Lease terminated"
                                        {d.termination_reason.as_deref().map(|r| format!(": {r}")).unwrap_or_default()}
                                    </span>
                                </div>
                            })}
                        }.into_any()
                    }
                })}
            </Suspense>

            // ── Two-column body ───────────────────────────────────────
            <div class="ld-body">
                // Left column
                <div class="ld-col-main">

                    // ── Invoice / payment history ─────────────────────
                    <div class="ld-card">
                        <h2 class="ld-card-title">
                            <span class="material-symbols-outlined" style="font-size:18px;">
                                {NavIcon::ReceiptLong.as_str()}
                            </span>
                            "Payment History"
                        </h2>
                        <Suspense fallback=|| view! { <div class="ld-skel ld-skel--table"/> }>
                            {move || invoices.get().map(|res| match res {
                                Err(e) => view! {
                                    <p class="ld-meta-empty">{format!("Could not load invoices: {e}")}</p>
                                }.into_any(),
                                Ok(inv) if inv.is_empty() => view! {
                                    <div class="ld-empty-inline">
                                        <span class="material-symbols-outlined ld-empty-icon">
                                            {NavIcon::ReceiptLong.as_str()}
                                        </span>
                                        <p class="ld-empty-text">
                                            "No invoices yet. They appear here once billing begins."
                                        </p>
                                    </div>
                                }.into_any(),
                                Ok(inv) => view! {
                                    <table class="ld-inv-table">
                                        <thead>
                                            <tr>
                                                <th class="ld-inv-th">"Status"</th>
                                                <th class="ld-inv-th">"Amount"</th>
                                                <th class="ld-inv-th">"Rail"</th>
                                                <th class="ld-inv-th">"Due"</th>
                                                <th class="ld-inv-th">"Paid"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {inv.into_iter().map(|inv| {
                                                let status = InvoiceStatus::from_str(&inv.status);
                                                let amount = format!(
                                                    "{} {:.2}",
                                                    inv.currency,
                                                    inv.gross_amount_cents as f64 / 100.0
                                                );
                                                let due = inv.due_date.map(|d| d.to_string()).unwrap_or_else(|| "\u{2014}".to_string());
                                                let paid = inv.paid_at.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "\u{2014}".to_string());
                                                let rail = inv.payment_rail.unwrap_or_else(|| "\u{2014}".to_string());
                                                view! {
                                                    <tr class="ld-inv-row">
                                                        <td class="ld-inv-td">
                                                            <span class={format!("inv-status-badge {}", status.pill_class())}>
                                                                <span class="material-symbols-outlined" style="font-size:10px;">
                                                                    {status.material_icon()}
                                                                </span>
                                                                {status.as_str()}
                                                            </span>
                                                        </td>
                                                        <td class="ld-inv-td ld-inv-td--amount">{amount}</td>
                                                        <td class="ld-inv-td ld-inv-td--rail">{rail}</td>
                                                        <td class="ld-inv-td">{due}</td>
                                                        <td class="ld-inv-td">{paid}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                }.into_any(),
                            })}
                        </Suspense>
                    </div>

                    // ── Occupants ─────────────────────────────────────
                    <div class="ld-card">
                        <h2 class="ld-card-title">
                            <span class="material-symbols-outlined" style="font-size:18px;">
                                {NavIcon::People.as_str()}
                            </span>
                            "Household"
                        </h2>
                        <Suspense fallback=|| view! { <div class="ld-skel ld-skel--list"/> }>
                            {move || occupants.get().map(|res| match res {
                                Err(e) => view! {
                                    <p class="ld-meta-empty">{format!("Could not load occupants: {e}")}</p>
                                }.into_any(),
                                Ok(occ) if occ.active.is_empty() => view! {
                                    <div class="ld-empty-inline">
                                        <span class="material-symbols-outlined ld-empty-icon">
                                            {NavIcon::People.as_str()}
                                        </span>
                                        <p class="ld-empty-text">"No occupants registered on this lease."</p>
                                    </div>
                                }.into_any(),
                                Ok(occ) => view! {
                                    <ul class="ld-occupant-list">
                                        {occ.active.into_iter().map(|o| view! {
                                            <li class="ld-occupant-row">
                                                <div class="ld-occupant-avatar">
                                                    {o.full_name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                                                </div>
                                                <div class="ld-occupant-info">
                                                    <p class="ld-occupant-name">{o.full_name}</p>
                                                    <p class="ld-occupant-meta">
                                                        {o.kind.replace('_', " ")}
                                                        " \u{00b7} "
                                                        {o.relationship.replace('_', " ")}
                                                    </p>
                                                </div>
                                            </li>
                                        }).collect_view()}
                                    </ul>
                                }.into_any(),
                            })}
                        </Suspense>
                    </div>
                </div>

                // Right column
                <div class="ld-col-side">

                    // ── Vehicles ──────────────────────────────────────
                    <div class="ld-card">
                        <h2 class="ld-card-title">
                            <span class="material-symbols-outlined" style="font-size:18px;">
                                "directions_car"
                            </span>
                            "Registered Vehicles"
                        </h2>
                        <Suspense fallback=|| view! { <div class="ld-skel ld-skel--list"/> }>
                            {move || vehicles.get().map(|res| match res {
                                Err(e) => view! {
                                    <p class="ld-meta-empty">{format!("Could not load vehicles: {e}")}</p>
                                }.into_any(),
                                Ok(vs) if vs.is_empty() => view! {
                                    <div class="ld-empty-inline">
                                        <span class="material-symbols-outlined ld-empty-icon">
                                            "directions_car"
                                        </span>
                                        <p class="ld-empty-text">"No vehicles registered."</p>
                                    </div>
                                }.into_any(),
                                Ok(vs) => view! {
                                    <ul class="ld-vehicle-list">
                                        {vs.into_iter().map(|v| view! {
                                            <li class="ld-vehicle-row">
                                                <span class="material-symbols-outlined" style="font-size:20px;color:var(--folio-muted);">
                                                    "directions_car"
                                                </span>
                                                <div class="ld-vehicle-info">
                                                    <p class="ld-vehicle-name">
                                                        {format!("{} {} {} {}", v.year, v.color, v.make, v.model)}
                                                    </p>
                                                    <p class="ld-vehicle-plate">
                                                        {v.license_plate}
                                                        {v.parking_spot.map(|s| format!(" \u{00b7} Spot {s}"))}
                                                    </p>
                                                </div>
                                            </li>
                                        }).collect_view()}
                                    </ul>
                                }.into_any(),
                            })}
                        </Suspense>
                    </div>

                    // ── Quick links ───────────────────────────────────
                    <div class="ld-card">
                        <h2 class="ld-card-title">
                            <span class="material-symbols-outlined" style="font-size:18px;">
                                {NavIcon::Assignment.as_str()}
                            </span>
                            "Quick Actions"
                        </h2>
                        <div class="ld-quick-links">
                            <a
                                href=FolioRoute::LandlordMaintenance.path()
                                class="ld-quick-link"
                            >
                                <span class="material-symbols-outlined" style="font-size:18px;">
                                    {NavIcon::Build.as_str()}
                                </span>
                                "Create Work Order"
                            </a>
                            <a
                                href=FolioRoute::LandlordBilling.path()
                                class="ld-quick-link"
                            >
                                <span class="material-symbols-outlined" style="font-size:18px;">
                                    {NavIcon::ReceiptLong.as_str()}
                                </span>
                                "Billing Centre"
                            </a>
                        </div>
                    </div>
                </div>
            </div>
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

/// GET /api/folio/leases/:id
#[server(GetLeaseDetail, "/api")]
pub async fn get_lease_detail(
    lease_id: String,
) -> Result<LeaseDetail, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    if lease_id.is_empty() {
        return Err(server_fn::error::ServerFnError::new("Missing lease ID"));
    }
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<LeaseDetail>(
        &format!("/api/folio/leases/{lease_id}"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Lease detail failed: {e}")))
}

/// GET /api/folio/leases/:id/occupants
#[server(GetLeaseOccupants, "/api")]
pub async fn get_lease_occupants(
    lease_id: String,
) -> Result<OccupantsResponse, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    if lease_id.is_empty() {
        return Err(server_fn::error::ServerFnError::new("Missing lease ID"));
    }
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<OccupantsResponse>(
        &format!("/api/folio/leases/{lease_id}/occupants"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Occupants failed: {e}")))
}

/// GET /api/folio/leases/:id/vehicles
#[server(GetLeaseVehicles, "/api")]
pub async fn get_lease_vehicles(
    lease_id: String,
) -> Result<Vec<VehicleRecord>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    if lease_id.is_empty() {
        return Err(server_fn::error::ServerFnError::new("Missing lease ID"));
    }
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<VehicleRecord>>(
        &format!("/api/folio/leases/{lease_id}/vehicles"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Vehicles failed: {e}")))
}

/// GET /api/folio/leases/:id/invoices
#[server(GetLeaseInvoices, "/api")]
pub async fn get_lease_invoices(
    lease_id: String,
) -> Result<Vec<LeaseInvoiceSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    if lease_id.is_empty() {
        return Err(server_fn::error::ServerFnError::new("Missing lease ID"));
    }
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<LeaseInvoiceSummary>>(
        &format!("/api/folio/leases/{lease_id}/invoices"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Invoices failed: {e}")))
}
