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
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::NavIcon;
use crate::components::page_header::PageHeader;
use crate::pages::landlord::leases::list_leases;

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LedgerEntrySummary {
    pub id: uuid::Uuid,
    pub billable_entity_type: String,
    pub billable_entity_id: uuid::Uuid,
    pub description: Option<String>,
    pub gross_amount_cents: i64,
    pub fee_amount_cents: i64,
    pub net_amount_cents: i64,
    pub currency: String,
    pub payment_rail: Option<String>,
    pub status: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reconciled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reconciliation_note: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
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
            "pending" => Self::Pending,
            "processing" => Self::Processing,
            "paid" => Self::Paid,
            "failed" => Self::Failed,
            "refunded" => Self::Refunded,
            "waived" => Self::Waived,
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
            Self::Waived => "Waived",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Pending => "le-status--pending",
            Self::Processing => "le-status--processing",
            Self::Paid => "le-status--paid",
            Self::Failed => "le-status--failed",
            Self::Refunded => "le-status--refunded",
            Self::Waived => "le-status--waived",
            Self::Unknown => "le-status--unknown",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Pending => "schedule",
            Self::Processing => "sync",
            Self::Paid => "check_circle",
            Self::Failed => "cancel",
            Self::Refunded => "undo",
            Self::Waived => "remove_circle",
            Self::Unknown => "help",
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
            "atlas_violation" => return Self::ViolationFine,
            _ => {}
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
            Self::Rent => "Rent",
            Self::LateFee => "Late Fee",
            Self::MaintenanceReimbursement => "Maintenance",
            Self::Incidental => "Incidental",
            Self::SecurityDepositDeduction => "Deposit Deduction",
            Self::UtilityChargeback => "Utility Chargeback",
            Self::BookingCharge => "Booking",
            Self::ViolationFine => "Violation Fine",
            Self::Other => "Other",
            Self::Unknown => "Charge",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Rent => "ct--rent",
            Self::LateFee => "ct--late-fee",
            Self::MaintenanceReimbursement => "ct--maintenance",
            Self::Incidental => "ct--incidental",
            Self::SecurityDepositDeduction => "ct--deposit",
            Self::UtilityChargeback => "ct--utility",
            Self::BookingCharge => "ct--booking",
            Self::ViolationFine => "ct--violation",
            Self::Other => "ct--other",
            Self::Unknown => "ct--other",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Rent => "home",
            Self::LateFee => "pending_actions",
            Self::MaintenanceReimbursement => "handyman",
            Self::Incidental => "receipt_long",
            Self::SecurityDepositDeduction => "lock_open",
            Self::UtilityChargeback => "bolt",
            Self::BookingCharge => "hotel",
            Self::ViolationFine => "gavel",
            Self::Other => "more_horiz",
            Self::Unknown => "help",
        }
    }
}

/// Ad-hoc charge types accepted by `POST /api/folio/ledger/charge`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdHocChargeType {
    LateFee,
    MaintenanceReimbursement,
    Incidental,
    SecurityDepositDeduction,
    UtilityChargeback,
    Other,
}

impl AdHocChargeType {
    const ALL: &'static [Self] = &[
        Self::LateFee,
        Self::MaintenanceReimbursement,
        Self::Incidental,
        Self::SecurityDepositDeduction,
        Self::UtilityChargeback,
        Self::Other,
    ];

    const fn as_str(self) -> &'static str {
        match self {
            Self::LateFee => "late_fee",
            Self::MaintenanceReimbursement => "maintenance_reimbursement",
            Self::Incidental => "incidental",
            Self::SecurityDepositDeduction => "security_deposit_deduction",
            Self::UtilityChargeback => "utility_chargeback",
            Self::Other => "other",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::LateFee => "Late fee",
            Self::MaintenanceReimbursement => "Maintenance reimbursement",
            Self::Incidental => "Incidental",
            Self::SecurityDepositDeduction => "Security deposit deduction",
            Self::UtilityChargeback => "Utility chargeback",
            Self::Other => "Other",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|t| t.as_str() == s)
    }
}

/// ISO currency for ad-hoc charges — mirrors backend `Currency`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ChargeCurrency {
    Usd,
    Brl,
    Dop,
    Htg,
}

impl ChargeCurrency {
    const ALL: &'static [Self] = &[Self::Usd, Self::Brl, Self::Dop, Self::Htg];

    const fn as_str(self) -> &'static str {
        match self {
            Self::Usd => "USD",
            Self::Brl => "BRL",
            Self::Dop => "DOP",
            Self::Htg => "HTG",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|c| c.as_str().eq_ignore_ascii_case(s))
    }
}

/// Billable entity for landlord ad-hoc charges (lease/contract today).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BillableEntityType {
    AtlasContract,
}

impl BillableEntityType {
    const fn as_str(self) -> &'static str {
        match self {
            Self::AtlasContract => "atlas_contract",
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
            Self::All => "All",
            Self::Outstanding => "Outstanding",
            Self::Paid => "Paid",
            Self::Failed => "Failed",
        }
    }

    pub fn matches(self, status: EntryStatus) -> bool {
        match self {
            Self::All => true,
            Self::Outstanding => matches!(status, EntryStatus::Pending | EntryStatus::Processing),
            Self::Paid => status == EntryStatus::Paid,
            Self::Failed => matches!(status, EntryStatus::Failed | EntryStatus::Refunded),
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

    let refresh = RwSignal::new(0u32);
    let entries = Resource::new(
        move || refresh.get(),
        |_| async move { list_ledger_entries().await },
    );

    let show_charge = RwSignal::new(false);
    let lease_id = RwSignal::new(String::new());
    let charge_type = RwSignal::new(AdHocChargeType::LateFee.as_str().to_string());
    let description = RwSignal::new(String::new());
    let amount = RwSignal::new(String::new());
    let currency = RwSignal::new(ChargeCurrency::Usd.as_str().to_string());
    let due_date = RwSignal::new(String::new());
    let creating = RwSignal::new(false);
    let create_err = RwSignal::new(None::<String>);

    let leases = Resource::new(
        move || show_charge.get(),
        |open| async move {
            if !open {
                return Ok(vec![]);
            }
            list_leases().await
        },
    );

    let on_post_charge = move |_| {
        let lid = lease_id.get().trim().to_string();
        let desc = description.get().trim().to_string();
        let amt_raw = amount.get().trim().to_string();
        if lid.is_empty() || desc.is_empty() || amt_raw.is_empty() {
            create_err.set(Some("Lease, description, and amount are required.".into()));
            return;
        }
        let Ok(lease_uuid) = Uuid::parse_str(&lid) else {
            create_err.set(Some("Invalid lease.".into()));
            return;
        };
        let Some(ct) = AdHocChargeType::from_str(&charge_type.get()) else {
            create_err.set(Some("Invalid charge type.".into()));
            return;
        };
        let Some(cur) = ChargeCurrency::from_str(&currency.get()) else {
            create_err.set(Some("Invalid currency.".into()));
            return;
        };
        let Ok(dollars) = amt_raw.parse::<f64>() else {
            create_err.set(Some("Amount must be a number.".into()));
            return;
        };
        let cents = (dollars * 100.0).round() as i64;
        if cents <= 0 {
            create_err.set(Some("Amount must be greater than zero.".into()));
            return;
        }
        let due = {
            let d = due_date.get().trim().to_string();
            if d.is_empty() {
                None
            } else {
                Some(d)
            }
        };
        creating.set(true);
        create_err.set(None);
        spawn_local(async move {
            match create_ad_hoc_charge(
                BillableEntityType::AtlasContract.as_str().to_string(),
                lease_uuid,
                desc,
                ct.as_str().to_string(),
                cents,
                cur.as_str().to_string(),
                due,
            )
            .await
            {
                Ok(_) => {
                    show_charge.set(false);
                    description.set(String::new());
                    amount.set(String::new());
                    due_date.set(String::new());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => create_err.set(Some(e.to_string())),
            }
            creating.set(false);
        });
    };

    let title = Signal::derive(|| "Ledger".to_string());
    let subtitle = Signal::derive(|| {
        "All billable events across your portfolio — rent, fees, reimbursements.".to_string()
    });

    view! {
        <div class="le-page">
            <PageHeader title=title subtitle=subtitle>
                <button
                    type="button"
                    class="folio-btn folio-btn--primary press"
                    on:click=move |_| {
                        create_err.set(None);
                        show_charge.set(true);
                    }
                >
                    "Post charge"
                </button>
            </PageHeader>
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
                                || e.billable_entity_type.to_lowercase().contains(&q);
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
                                    <button
                                        type="button"
                                        class="folio-btn folio-btn--primary press"
                                        style="margin-top:1rem;"
                                        on:click=move |_| {
                                            create_err.set(None);
                                            show_charge.set(true);
                                        }
                                    >
                                        "Post charge"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <LedgerTable entries=filtered/> }.into_any()
                        }
                    }
                })}
            </Suspense>

            <Show when=move || show_charge.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Post charge"</h3>
                            <button type="button" class="modal-close" on:click=move |_| show_charge.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="folio-field">
                                <label class="folio-field__label">"Lease *"</label>
                                <select
                                    class="folio-select"
                                    on:change=move |ev| lease_id.set(event_target_value(&ev))
                                >
                                    <option value="">"Select lease…"</option>
                                    {move || leases.get().and_then(|r| r.ok()).unwrap_or_default().into_iter().map(|l| {
                                        let id = l.id.to_string();
                                        let label = format!(
                                            "{} · {} · {}",
                                            &id[..8.min(id.len())],
                                            l.status,
                                            format_amount(l.monthly_rent_cents.unwrap_or(0), &l.currency),
                                        );
                                        view! { <option value=id>{label}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Charge type *"</label>
                                <select
                                    class="folio-select"
                                    on:change=move |ev| charge_type.set(event_target_value(&ev))
                                >
                                    {AdHocChargeType::ALL.iter().copied().map(|t| {
                                        view! { <option value=t.as_str()>{t.label()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Description *"</label>
                                <input
                                    type="text"
                                    class="folio-input"
                                    placeholder="Payment 7 days past due"
                                    prop:value=description
                                    on:input=move |ev| description.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Amount *"</label>
                                <input
                                    type="number"
                                    class="folio-input"
                                    min="0"
                                    step="0.01"
                                    prop:value=amount
                                    on:input=move |ev| amount.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Currency *"</label>
                                <select
                                    class="folio-select"
                                    on:change=move |ev| currency.set(event_target_value(&ev))
                                >
                                    {ChargeCurrency::ALL.iter().copied().map(|c| {
                                        view! { <option value=c.as_str()>{c.as_str()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Due date"</label>
                                <input
                                    type="date"
                                    class="folio-input"
                                    prop:value=due_date
                                    on:input=move |ev| due_date.set(event_target_value(&ev))
                                />
                            </div>
                            {move || create_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button type="button" class="folio-btn folio-btn--ghost" on:click=move |_| show_charge.set(false)>
                                "Cancel"
                            </button>
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary"
                                disabled=move || creating.get()
                                on:click=on_post_charge
                            >
                                {move || if creating.get() { "Posting…" } else { "Post charge" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
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
                        let entity = {
                            let t = e.billable_entity_type.replace('_', " ");
                            if t.is_empty() { "—".into() } else { t }
                        };
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

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
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

#[derive(Serialize)]
struct CreateAdHocChargeBody {
    billable_entity_type: String,
    billable_entity_id: Uuid,
    description: String,
    charge_type: String,
    gross_amount_cents: i64,
    currency: String,
    due_date: Option<String>,
}

#[derive(Deserialize)]
struct CreateAdHocChargeResponse {
    ledger_entry_id: Uuid,
}

/// POST /api/folio/ledger/charge
#[server(CreateAdHocCharge, "/api")]
pub async fn create_ad_hoc_charge(
    billable_entity_type: String,
    billable_entity_id: Uuid,
    description: String,
    charge_type: String,
    gross_amount_cents: i64,
    currency: String,
    due_date: Option<String>,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    if billable_entity_type != BillableEntityType::AtlasContract.as_str() {
        return Err(server_fn::error::ServerFnError::new(
            "Unsupported billable entity type",
        ));
    }
    if AdHocChargeType::from_str(&charge_type).is_none() {
        return Err(server_fn::error::ServerFnError::new("Invalid charge type"));
    }
    if ChargeCurrency::from_str(&currency).is_none() {
        return Err(server_fn::error::ServerFnError::new("Invalid currency"));
    }
    if description.trim().is_empty() {
        return Err(server_fn::error::ServerFnError::new("Description is required"));
    }
    if gross_amount_cents <= 0 {
        return Err(server_fn::error::ServerFnError::new(
            "Amount must be greater than zero",
        ));
    }

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let body = CreateAdHocChargeBody {
        billable_entity_type,
        billable_entity_id,
        description: description.trim().to_string(),
        charge_type,
        gross_amount_cents,
        currency,
        due_date,
    };
    let resp =
        crate::atlas_client::authenticated_post::<CreateAdHocChargeBody, CreateAdHocChargeResponse>(
            "/api/folio/ledger/charge",
            &token,
            None,
            &body,
        )
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Post charge failed: {e}")))?;
    Ok(resp.ledger_entry_id)
}
