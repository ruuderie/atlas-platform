// apps/folio/src/pages/landlord/ledger.rs
//
// Ledger page — /l/ledger
//
// Master audit trail for every billable event across the landlord's portfolio:
//   - Rent payments        (billable_entity_type = "atlas_contract")
//   - Late fees            ([late_fee] prefix in description)
//   - Maintenance bills    ([maintenance_reimbursement] prefix)
//   - Incidentals          ([incidental] prefix)
//   - Deposit deductions   ([security_deposit_deduction] prefix)
//   - Utility chargebacks  ([utility_chargeback] prefix)
//   - STR booking charges  (billable_entity_type = "atlas_reservation")
//   - Violation fines      (billable_entity_type = "atlas_violation")
//
// Data source: GET /api/folio/ledger → all atlas_ledger_entries for tenant.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::nav::NavIcon;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerEntrySummary {
    pub id:                   uuid::Uuid,
    pub billable_entity_type: String,
    pub billable_entity_id:   uuid::Uuid,
    pub description:          Option<String>,
    pub gross_amount_cents:   i64,
    pub fee_amount_cents:     i64,
    pub net_amount_cents:     i64,
    pub currency:             String,
    pub payment_rail:         Option<String>,
    pub status:               String,
    pub due_date:             Option<chrono::NaiveDate>,
    pub paid_at:              Option<chrono::DateTime<chrono::Utc>>,
    pub reconciled_at:        Option<chrono::DateTime<chrono::Utc>>,
    pub reconciliation_note:  Option<String>,
    pub created_at:           chrono::DateTime<chrono::Utc>,
}

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Ledger entry payment/settlement status — mirrors PmLedgerService state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryStatus {
    Pending,
    Processing,
    Paid,
    Failed,
    Refunded,
    Waived,
    Unknown,
}

impl EntryStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "pending"    => Self::Pending,
            "processing" => Self::Processing,
            "paid"       => Self::Paid,
            "failed"     => Self::Failed,
            "refunded"   => Self::Refunded,
            "waived"     => Self::Waived,
            _            => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending    => "Pending",
            Self::Processing => "Processing",
            Self::Paid       => "Paid",
            Self::Failed     => "Failed",
            Self::Refunded   => "Refunded",
            Self::Waived     => "Waived",
            Self::Unknown    => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Pending    => "le-status--pending",
            Self::Processing => "le-status--processing",
            Self::Paid       => "le-status--paid",
            Self::Failed     => "le-status--failed",
            Self::Refunded   => "le-status--refunded",
            Self::Waived     => "le-status--waived",
            Self::Unknown    => "le-status--unknown",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Pending    => "schedule",
            Self::Processing => "sync",
            Self::Paid       => "check_circle",
            Self::Failed     => "cancel",
            Self::Refunded   => "undo",
            Self::Waived     => "remove_circle",
            Self::Unknown    => "help",
        }
    }
}

impl std::fmt::Display for EntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Charge type — parsed from the `[charge_type]` prefix in the description.
/// Rent payments (no prefix) map to `Rent`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChargeType {
    Rent,
    LateFee,
    MaintenanceReimbursement,
    Incidental,
    SecurityDepositDeduction,
    UtilityChargeback,
    BookingCharge,
    ViolationFine,
    Other,
    Unknown,
}

impl ChargeType {
    /// Parse from billable_entity_type + description prefix.
    pub fn classify(entity_type: &str, description: Option<&str>) -> Self {
        // Entity type takes priority for non-contract entities.
        match entity_type {
            "atlas_reservation" => return Self::BookingCharge,
            "atlas_violation"   => return Self::ViolationFine,
            _                   => {}
        }
        // For atlas_contract entries, inspect the [prefix] tag.
        let desc = description.unwrap_or("");
        if desc.starts_with("[late_fee]") {
            Self::LateFee
        } else if desc.starts_with("[maintenance_reimbursement]") {
            Self::MaintenanceReimbursement
        } else if desc.starts_with("[incidental]") {
            Self::Incidental
        } else if desc.starts_with("[security_deposit_deduction]") {
            Self::SecurityDepositDeduction
        } else if desc.starts_with("[utility_chargeback]") {
            Self::UtilityChargeback
        } else if desc.starts_with("[other]") {
            Self::Other
        } else if entity_type == "atlas_contract" {
            Self::Rent
        } else {
            Self::Unknown
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Rent                       => "Rent",
            Self::LateFee                    => "Late Fee",
            Self::MaintenanceReimbursement   => "Maintenance",
            Self::Incidental                 => "Incidental",
            Self::SecurityDepositDeduction   => "Deposit Deduction",
            Self::UtilityChargeback          => "Utility Chargeback",
            Self::BookingCharge              => "Booking",
            Self::ViolationFine              => "Violation Fine",
            Self::Other                      => "Other",
            Self::Unknown                    => "Charge",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Rent                       => "ct--rent",
            Self::LateFee                    => "ct--late-fee",
            Self::MaintenanceReimbursement   => "ct--maintenance",
            Self::Incidental                 => "ct--incidental",
            Self::SecurityDepositDeduction   => "ct--deposit",
            Self::UtilityChargeback          => "ct--utility",
            Self::BookingCharge              => "ct--booking",
            Self::ViolationFine              => "ct--violation",
            Self::Other                      => "ct--other",
            Self::Unknown                    => "ct--other",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Rent                       => "home",
            Self::LateFee                    => "pending_actions",
            Self::MaintenanceReimbursement   => "handyman",
            Self::Incidental                 => "receipt_long",
            Self::SecurityDepositDeduction   => "lock_open",
            Self::UtilityChargeback          => "bolt",
            Self::BookingCharge              => "hotel",
            Self::ViolationFine              => "gavel",
            Self::Other                      => "more_horiz",
            Self::Unknown                    => "help",
        }
    }
}

/// Status filter used in the filter bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusFilter {
    All,
    Outstanding,
    Paid,
    Failed,
}

impl StatusFilter {
    pub const fn label(self) -> &'static str {
        match self {
            Self::All         => "All",
            Self::Outstanding => "Outstanding",
            Self::Paid        => "Paid",
            Self::Failed      => "Failed",
        }
    }

    pub fn matches(self, status: EntryStatus) -> bool {
        match self {
            Self::All         => true,
            Self::Outstanding => matches!(status, EntryStatus::Pending | EntryStatus::Processing),
            Self::Paid        => status == EntryStatus::Paid,
            Self::Failed      => matches!(status, EntryStatus::Failed | EntryStatus::Refunded),
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Strip the `[charge_type] ` prefix tag to get the display description.
fn strip_tag(description: Option<&str>) -> String {
    let Some(d) = description else {
        return "\u{2014}".to_string();
    };
    if d.starts_with('[') {
        // Skip past the closing `] ` — e.g. "[late_fee] Payment 7 days past due"
        d.find("] ")
            .map(|i| d[i + 2..].to_string())
            .unwrap_or_else(|| d.to_string())
    } else {
        d.to_string()
    }
}

fn format_amount(cents: i64, currency: &str) -> String {
    format!("{} {:.2}", currency, cents as f64 / 100.0)
}

// ── Page ──────────────────────────────────────────────────────────────────────

/// Ledger — master audit trail for all billable events.
#[component]
pub fn Ledger() -> impl IntoView {
    let (status_filter, set_status) = signal(StatusFilter::All);
    let (type_filter, set_type) = signal(Option::<ChargeType>::None);
    let (search_query, set_search) = signal(String::new());

    let entries = Resource::new(
        || (),
        |_| async move { list_ledger_entries().await },
    );

    view! {
        <div class="le-page">
            // ── Header ────────────────────────────────────────────────
            <div class="le-header">
                <div>
                    <h1 class="le-title">"Ledger"</h1>
                    <p class="le-subtitle">"All billable events across your portfolio — rent, fees, reimbursements."</p>
                </div>
                // KPI strip — totals derived from the entry list
                <Suspense fallback=|| view! { <div class="le-kpi-skel"/> }>
                    {move || entries.get().map(|res| {
                        let (outstanding, total_paid) = res.as_ref().map(|v| {
                            let out = v.iter()
                                .filter(|e| matches!(EntryStatus::from_str(&e.status), EntryStatus::Pending | EntryStatus::Processing))
                                .map(|e| e.gross_amount_cents)
                                .sum::<i64>();
                            let paid = v.iter()
                                .filter(|e| EntryStatus::from_str(&e.status) == EntryStatus::Paid)
                                .map(|e| e.net_amount_cents)
                                .sum::<i64>();
                            (out, paid)
                        }).unwrap_or((0, 0));
                        let currency = res.as_ref().ok()
                            .and_then(|v| v.first().map(|e| e.currency.clone()))
                            .unwrap_or_else(|| "USD".to_string());
                        view! {
                            <div class="le-kpi-strip">
                                <div class="le-kpi">
                                    <span class="le-kpi-label">"Outstanding"</span>
                                    <span class="le-kpi-val le-kpi-val--alert">
                                        {format_amount(outstanding, &currency)}
                                    </span>
                                </div>
                                <div class="le-kpi">
                                    <span class="le-kpi-label">"Collected (net)"</span>
                                    <span class="le-kpi-val le-kpi-val--green">
                                        {format_amount(total_paid, &currency)}
                                    </span>
                                </div>
                            </div>
                        }
                    })}
                </Suspense>
            </div>

            // ── Filter bar ────────────────────────────────────────────
            <div class="le-filter-bar">
                <div class="le-search-wrap">
                    <span class="material-symbols-outlined le-search-icon">"search"</span>
                    <input
                        id="le-search"
                        class="le-search-input"
                        type="search"
                        placeholder="Search by description\u{2026}"
                        on:input=move |e| set_search.set(event_target_value(&e))
                    />
                </div>
                // Status chips
                <div class="le-status-chips">
                    {[StatusFilter::All, StatusFilter::Outstanding, StatusFilter::Paid, StatusFilter::Failed]
                        .iter().copied().map(|f| view! {
                        <button
                            class=move || if status_filter.get() == f { "le-chip le-chip--active" } else { "le-chip" }
                            on:click=move |_| set_status.set(f)
                        >
                            {f.label()}
                        </button>
                    }).collect_view()}
                </div>
                // Charge type filter
                <div class="le-type-chips">
                    <button
                        class=move || if type_filter.get().is_none() { "le-type-chip le-type-chip--active" } else { "le-type-chip" }
                        on:click=move |_| set_type.set(None)
                    >"All types"</button>
                    {[
                        ChargeType::Rent,
                        ChargeType::LateFee,
                        ChargeType::MaintenanceReimbursement,
                        ChargeType::Incidental,
                        ChargeType::SecurityDepositDeduction,
                        ChargeType::UtilityChargeback,
                        ChargeType::BookingCharge,
                        ChargeType::ViolationFine,
                    ].iter().copied().map(|ct| view! {
                        <button
                            class=move || {
                                if type_filter.get() == Some(ct) {
                                    "le-type-chip le-type-chip--active"
                                } else {
                                    "le-type-chip"
                                }
                            }
                            on:click=move |_| set_type.set(Some(ct))
                        >
                            <span class="material-symbols-outlined" style="font-size:12px;">
                                {ct.material_icon()}
                            </span>
                            {ct.label()}
                        </button>
                    }).collect_view()}
                </div>
            </div>

            // ── Table ─────────────────────────────────────────────────
            <Suspense fallback=|| view! { <LedgerSkeleton rows=10/> }>
                {move || entries.get().map(|res| match res {
                    Err(e) => view! {
                        <div class="le-error">
                            <p class="le-error-text">{format!("Could not load ledger: {e}")}</p>
                        </div>
                    }.into_any(),
                    Ok(all) => {
                        let q    = search_query.get().to_lowercase();
                        let sf   = status_filter.get();
                        let tf   = type_filter.get();

                        let filtered: Vec<LedgerEntrySummary> = all.into_iter().filter(|e| {
                            let status   = EntryStatus::from_str(&e.status);
                            let ct       = ChargeType::classify(&e.billable_entity_type, e.description.as_deref());
                            let desc_raw = e.description.as_deref().unwrap_or("");
                            let search_ok = q.is_empty()
                                || strip_tag(Some(desc_raw)).to_lowercase().contains(&q)
                                || e.billable_entity_id.to_string().contains(&q);
                            let status_ok = sf.matches(status);
                            let type_ok   = tf.map(|t| t == ct).unwrap_or(true);
                            search_ok && status_ok && type_ok
                        }).collect();

                        if filtered.is_empty() {
                            view! {
                                <div class="le-empty">
                                    <span class="material-symbols-outlined le-empty-icon">
                                        {NavIcon::AccountBalance.as_str()}
                                    </span>
                                    <p class="le-empty-title">"No entries"</p>
                                    <p class="le-empty-sub">"Adjust filters or post a new charge."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! { <LedgerTable entries=filtered/> }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

// ── Table component ───────────────────────────────────────────────────────────

#[component]
fn LedgerTable(entries: Vec<LedgerEntrySummary>) -> impl IntoView {
    view! {
        <div class="le-table-wrap">
            <table class="le-table">
                <thead>
                    <tr>
                        <th class="le-th">"Type"</th>
                        <th class="le-th">"Status"</th>
                        <th class="le-th">"Description"</th>
                        <th class="le-th">"Entity"</th>
                        <th class="le-th">"Gross"</th>
                        <th class="le-th">"Net"</th>
                        <th class="le-th">"Rail"</th>
                        <th class="le-th">"Due"</th>
                        <th class="le-th">"Paid"</th>
                    </tr>
                </thead>
                <tbody>
                    {entries.into_iter().map(|e| {
                        let status  = EntryStatus::from_str(&e.status);
                        let ct      = ChargeType::classify(&e.billable_entity_type, e.description.as_deref());
                        let desc    = strip_tag(e.description.as_deref());
                        let entity  = e.billable_entity_id.to_string().split('-').next().unwrap_or("").to_uppercase();
                        let gross   = format_amount(e.gross_amount_cents, &e.currency);
                        let net     = format_amount(e.net_amount_cents, &e.currency);
                        let rail    = e.payment_rail.as_deref().unwrap_or("\u{2014}").to_string();
                        let due     = e.due_date.map(|d| d.to_string()).unwrap_or_else(|| "\u{2014}".to_string());
                        let paid    = e.paid_at.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "\u{2014}".to_string());
                        let outstanding = matches!(status, EntryStatus::Pending | EntryStatus::Processing);
                        view! {
                            <tr class=if outstanding { "le-row le-row--outstanding" } else { "le-row" }>
                                <td class="le-td">
                                    <span class={format!("le-charge-badge {}", ct.pill_class())}>
                                        <span class="material-symbols-outlined" style="font-size:11px;">
                                            {ct.material_icon()}
                                        </span>
                                        {ct.label()}
                                    </span>
                                </td>
                                <td class="le-td">
                                    <span class={format!("le-status-badge {}", status.pill_class())}>
                                        <span class="material-symbols-outlined" style="font-size:11px;">
                                            {status.material_icon()}
                                        </span>
                                        {status.as_str()}
                                    </span>
                                </td>
                                <td class="le-td le-td--desc">{desc}</td>
                                <td class="le-td le-td--mono">{entity}</td>
                                <td class="le-td le-td--amount">{gross}</td>
                                <td class="le-td le-td--net">{net}</td>
                                <td class="le-td le-td--rail">{rail}</td>
                                <td class="le-td">{due}</td>
                                <td class="le-td">{paid}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

// ── Skeleton ──────────────────────────────────────────────────────────────────

#[component]
fn LedgerSkeleton(rows: usize) -> impl IntoView {
    view! {
        <div class="le-table-wrap">
            <table class="le-table">
                <thead>
                    <tr>
                        <th class="le-th">"Type"</th>
                        <th class="le-th">"Status"</th>
                        <th class="le-th">"Description"</th>
                        <th class="le-th">"Entity"</th>
                        <th class="le-th">"Gross"</th>
                        <th class="le-th">"Net"</th>
                        <th class="le-th">"Rail"</th>
                        <th class="le-th">"Due"</th>
                        <th class="le-th">"Paid"</th>
                    </tr>
                </thead>
                <tbody>
                    {(0..rows).map(|_| view! {
                        <tr class="le-row">
                            <td class="le-td"><div class="le-skel le-skel--badge"/></td>
                            <td class="le-td"><div class="le-skel le-skel--badge"/></td>
                            <td class="le-td"><div class="le-skel le-skel--text"/></td>
                            <td class="le-td"><div class="le-skel le-skel--sm"/></td>
                            <td class="le-td"><div class="le-skel le-skel--sm"/></td>
                            <td class="le-td"><div class="le-skel le-skel--sm"/></td>
                            <td class="le-td"><div class="le-skel le-skel--sm"/></td>
                            <td class="le-td"><div class="le-skel le-skel--sm"/></td>
                            <td class="le-td"><div class="le-skel le-skel--sm"/></td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

// ── Server functions ──────────────────────────────────────────────────────────

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

/// GET /api/folio/ledger
#[server(ListLedgerEntries, "/api")]
pub async fn list_ledger_entries(
) -> Result<Vec<LedgerEntrySummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<LedgerEntrySummary>>(
        "/api/folio/ledger",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Ledger list failed: {e}")))
}
