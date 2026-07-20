// apps/folio/src/pages/landlord/violations.rs
//
// Violations page — /l/violations
//
// Global compliance queue showing all violations across the landlord's portfolio.
// Data source: GET /api/folio/violations → [ViolationRecord]
//
// Status machine: Open → Cured | Escalated | Dismissed
// Categories: 14 (LTR + STR-specific)

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::pages::landlord::vendors::{list_assets_for_picker, AssetPickerItem};

// ── Response types ────────────────────────────────────────────────────────────

/// Mirrors ViolationRecord from the violation service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViolationRecord {
    pub id: uuid::Uuid,
    pub asset_id: Option<uuid::Uuid>,
    pub contract_id: Option<uuid::Uuid>,
    pub reservation_id: Option<uuid::Uuid>,
    pub category: String,
    pub subject: String,
    pub description: Option<String>,
    pub cure_status: String,
    pub cure_deadline: Option<chrono::NaiveDate>,
    pub filed_at: chrono::DateTime<chrono::Utc>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub resolution_notes: Option<String>,
}

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Cure status — mirrors CureStatus in violation service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CureStatus {
    Open,
    Cured,
    Escalated,
    Dismissed,
    Unknown,
}

impl CureStatus {
    /// Terminal / transition targets for Open rows (excludes Open itself).
    pub const TRANSITIONS: &'static [Self] = &[Self::Cured, Self::Escalated, Self::Dismissed];

    pub fn from_str(s: &str) -> Self {
        match s {
            "open" => Self::Open,
            "cured" => Self::Cured,
            "escalated" => Self::Escalated,
            "dismissed" => Self::Dismissed,
            _ => Self::Unknown,
        }
    }

    /// Wire value for PATCH `/api/folio/violations/{id}/status`.
    pub const fn api_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Cured => "cured",
            Self::Escalated => "escalated",
            Self::Dismissed => "dismissed",
            Self::Unknown => "open",
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Cured => "Cured",
            Self::Escalated => "Escalated",
            Self::Dismissed => "Dismissed",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Open => "vs--open",
            Self::Cured => "vs--cured",
            Self::Escalated => "vs--escalated",
            Self::Dismissed => "vs--dismissed",
            Self::Unknown => "vs--unknown",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Open => "radio_button_unchecked",
            Self::Cured => "check_circle",
            Self::Escalated => "warning",
            Self::Dismissed => "cancel",
            Self::Unknown => "help",
        }
    }
}

impl std::fmt::Display for CureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Violation category — mirrors ViolationCategory from violation service.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViolCategory {
    Noise,
    UnauthorizedOccupant,
    UnauthorizedPet,
    UnauthorizedVehicle,
    PropertyDamage,
    LeaseBreach,
    Subletting,
    FailureToMaintain,
    IllegalActivity,
    Hoarding,
    SmokingInUnit,
    UnauthorizedParty,
    OverOccupancy,
    Other,
    Unknown,
}

impl ViolCategory {
    pub const ALL: &'static [Self] = &[
        Self::Noise,
        Self::UnauthorizedOccupant,
        Self::UnauthorizedPet,
        Self::UnauthorizedVehicle,
        Self::PropertyDamage,
        Self::LeaseBreach,
        Self::Subletting,
        Self::FailureToMaintain,
        Self::IllegalActivity,
        Self::Hoarding,
        Self::SmokingInUnit,
        Self::UnauthorizedParty,
        Self::OverOccupancy,
        Self::Other,
    ];

    pub fn from_str(s: &str) -> Self {
        match s {
            "noise" => Self::Noise,
            "unauthorized_occupant" => Self::UnauthorizedOccupant,
            "unauthorized_pet" => Self::UnauthorizedPet,
            "unauthorized_vehicle" => Self::UnauthorizedVehicle,
            "property_damage" => Self::PropertyDamage,
            "lease_breach" => Self::LeaseBreach,
            "subletting" => Self::Subletting,
            "failure_to_maintain" => Self::FailureToMaintain,
            "illegal_activity" => Self::IllegalActivity,
            "hoarding" => Self::Hoarding,
            "smoking_in_unit" => Self::SmokingInUnit,
            "unauthorized_party" => Self::UnauthorizedParty,
            "over_occupancy" => Self::OverOccupancy,
            "other" => Self::Other,
            _ => Self::Unknown,
        }
    }

    pub const fn api_str(self) -> &'static str {
        match self {
            Self::Noise => "noise",
            Self::UnauthorizedOccupant => "unauthorized_occupant",
            Self::UnauthorizedPet => "unauthorized_pet",
            Self::UnauthorizedVehicle => "unauthorized_vehicle",
            Self::PropertyDamage => "property_damage",
            Self::LeaseBreach => "lease_breach",
            Self::Subletting => "subletting",
            Self::FailureToMaintain => "failure_to_maintain",
            Self::IllegalActivity => "illegal_activity",
            Self::Hoarding => "hoarding",
            Self::SmokingInUnit => "smoking_in_unit",
            Self::UnauthorizedParty => "unauthorized_party",
            Self::OverOccupancy => "over_occupancy",
            Self::Other | Self::Unknown => "other",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Noise => "Noise",
            Self::UnauthorizedOccupant => "Unauth. Occupant",
            Self::UnauthorizedPet => "Unauth. Pet",
            Self::UnauthorizedVehicle => "Unauth. Vehicle",
            Self::PropertyDamage => "Property Damage",
            Self::LeaseBreach => "Lease Breach",
            Self::Subletting => "Subletting",
            Self::FailureToMaintain => "Failure to Maintain",
            Self::IllegalActivity => "Illegal Activity",
            Self::Hoarding => "Hoarding",
            Self::SmokingInUnit => "Smoking",
            Self::UnauthorizedParty => "Unauth. Party (STR)",
            Self::OverOccupancy => "Over-Occupancy (STR)",
            Self::Other => "Other",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Noise => "volume_up",
            Self::UnauthorizedOccupant => "person_off",
            Self::UnauthorizedPet => "pets",
            Self::UnauthorizedVehicle => "directions_car",
            Self::PropertyDamage => "construction",
            Self::LeaseBreach => "description",
            Self::Subletting => "swap_horiz",
            Self::FailureToMaintain => "cleaning_services",
            Self::IllegalActivity => "gavel",
            Self::Hoarding => "inventory_2",
            Self::SmokingInUnit => "smoking_rooms",
            Self::UnauthorizedParty => "celebration",
            Self::OverOccupancy => "group",
            Self::Other => "more_horiz",
            Self::Unknown => "help",
        }
    }
}

/// Status filter for the filter bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusFilter {
    All,
    Open,
    Escalated,
    Resolved,
}

impl StatusFilter {
    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Open => "Open",
            Self::Escalated => "Escalated",
            Self::Resolved => "Resolved",
        }
    }

    pub fn matches(self, s: CureStatus) -> bool {
        match self {
            Self::All => true,
            Self::Open => s == CureStatus::Open,
            Self::Escalated => s == CureStatus::Escalated,
            Self::Resolved => matches!(s, CureStatus::Cured | CureStatus::Dismissed),
        }
    }
}

// ── Page ──────────────────────────────────────────────────────────────────────

/// Violations — portfolio-wide compliance queue.
#[component]
pub fn Violations() -> impl IntoView {
    let (status_filter, set_status) = signal(StatusFilter::All);
    let (search_query, set_search) = signal(String::new());
    let refresh = RwSignal::new(0u32);
    let show_file = RwSignal::new(false);
    let file_asset = RwSignal::new(String::new());
    let file_category = RwSignal::new(ViolCategory::Noise.api_str().to_string());
    let file_subject = RwSignal::new(String::new());
    let file_description = RwSignal::new(String::new());
    let file_cure_days = RwSignal::new(String::new());
    let filing = RwSignal::new(false);
    let file_err = RwSignal::new(None::<String>);

    let violations = Resource::new(move || refresh.get(), |_| async move { list_violations().await });
    let assets = Resource::new(
        move || show_file.get(),
        |open| async move {
            if !open {
                return Ok::<Vec<AssetPickerItem>, server_fn::error::ServerFnError>(vec![]);
            }
            list_assets_for_picker().await
        },
    );

    let on_file = move |_| {
        let asset_id = file_asset.get().trim().to_string();
        let subject = file_subject.get().trim().to_string();
        let description = file_description.get().trim().to_string();
        let category = file_category.get();
        if asset_id.is_empty() || subject.is_empty() {
            file_err.set(Some("Asset and subject are required.".into()));
            return;
        }
        let cure_days = {
            let s = file_cure_days.get().trim().to_string();
            if s.is_empty() {
                None
            } else {
                match s.parse::<u8>() {
                    Ok(n) => Some(n),
                    Err(_) => {
                        file_err.set(Some("Cure days must be a number.".into()));
                        return;
                    }
                }
            }
        };
        filing.set(true);
        file_err.set(None);
        spawn_local(async move {
            match file_violation(asset_id, category, subject, description, cure_days).await {
                Ok(_) => {
                    show_file.set(false);
                    file_subject.set(String::new());
                    file_description.set(String::new());
                    file_cure_days.set(String::new());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => file_err.set(Some(e.to_string())),
            }
            filing.set(false);
        });
    };

    view! {
        <div class="viol-page">
            // ── Header ────────────────────────────────────────────────
            <div class="viol-header">
                <div>
                    <h1 class="viol-title">"Violations"</h1>
                    <p class="viol-subtitle">"Compliance queue — cure deadlines and escalation status."</p>
                </div>
                <div style="display:flex;align-items:center;gap:0.75rem;flex-wrap:wrap;">
                    <button
                        type="button"
                        class="folio-btn folio-btn--primary press"
                        on:click=move |_| {
                            file_err.set(None);
                            show_file.set(true);
                        }
                    >
                        "+ File Violation"
                    </button>
                    // KPI badges
                    <Suspense fallback=|| view! { <div class="viol-kpi-skel"/> }>
                        {move || violations.get().map(|res| {
                            let (open, escalated) = res.as_ref().map(|v| {
                                let open = v.iter().filter(|r| CureStatus::from_str(&r.cure_status) == CureStatus::Open).count();
                                let esc  = v.iter().filter(|r| CureStatus::from_str(&r.cure_status) == CureStatus::Escalated).count();
                                (open, esc)
                            }).unwrap_or((0, 0));
                            view! {
                                <div class="viol-kpi-strip">
                                    <div class="viol-kpi">
                                        <span class="material-symbols-outlined" style="font-size:14px;color:#d97706">"radio_button_unchecked"</span>
                                        <span class="viol-kpi-val">{open}</span>
                                        <span class="viol-kpi-label">"Open"</span>
                                    </div>
                                    {(escalated > 0).then(|| view! {
                                        <div class="viol-kpi viol-kpi--escalated">
                                            <span class="material-symbols-outlined" style="font-size:14px;">"warning"</span>
                                            <span class="viol-kpi-val">{escalated}</span>
                                            <span class="viol-kpi-label">"Escalated"</span>
                                        </div>
                                    })}
                                </div>
                            }
                        })}
                    </Suspense>
                </div>
            </div>

            // ── Filter bar ────────────────────────────────────────────
            <div class="viol-filter-bar">
                <div class="viol-search-wrap">
                    <span class="material-symbols-outlined viol-search-icon">"search"</span>
                    <input
                        id="viol-search"
                        class="viol-search-input"
                        type="search"
                        placeholder="Search by subject\u{2026}"
                        on:input=move |e| set_search.set(event_target_value(&e))
                    />
                </div>
                <div class="viol-status-chips">
                    {[StatusFilter::All, StatusFilter::Open, StatusFilter::Escalated, StatusFilter::Resolved]
                        .iter().copied().map(|f| view! {
                        <button
                            class=move || if status_filter.get() == f { "viol-chip viol-chip--active" } else { "viol-chip" }
                            on:click=move |_| set_status.set(f)
                        >
                            {f.label()}
                        </button>
                    }).collect_view()}
                </div>
            </div>

            // ── Table ─────────────────────────────────────────────────
            <Suspense fallback=|| view! { <ViolSkeleton rows=8/> }>
                {move || violations.get().map(|res| match res {
                    Err(e) => view! {
                        <div class="viol-error">
                            <p class="viol-error-text">{format!("Could not load violations: {e}")}</p>
                        </div>
                    }.into_any(),
                    Ok(all) => {
                        let q  = search_query.get().to_lowercase();
                        let sf = status_filter.get();

                        let filtered: Vec<ViolationRecord> = all.into_iter().filter(|v| {
                            let status   = CureStatus::from_str(&v.cure_status);
                            let search_ok = q.is_empty()
                                || v.subject.to_lowercase().contains(&q)
                                || v.category.to_lowercase().contains(&q);
                            let status_ok = sf.matches(status);
                            search_ok && status_ok
                        }).collect();

                        if filtered.is_empty() {
                            view! {
                                <div class="viol-empty">
                                    <span class="material-symbols-outlined viol-empty-icon">"gavel"</span>
                                    <p class="viol-empty-title">"No violations"</p>
                                    <p class="viol-empty-sub">"All clear — no compliance issues match the current filter."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! { <ViolTable records=filtered refresh=refresh/> }.into_any()
                        }
                    }
                })}
            </Suspense>

            <Show when=move || show_file.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"File Violation"</h3>
                            <button type="button" class="modal-close" on:click=move |_| show_file.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="folio-field">
                                <label class="folio-field__label">"Asset *"</label>
                                <select class="folio-select" on:change=move |ev| file_asset.set(event_target_value(&ev))>
                                    <option value="">"Select asset…"</option>
                                    {move || assets.get().and_then(|r| r.ok()).unwrap_or_default().into_iter().map(|a| {
                                        let id = a.id.to_string();
                                        let label = format!(
                                            "{} ({})",
                                            a.place_label(),
                                            a.asset_type.replace('_', " ")
                                        );
                                        view! { <option value=id>{label}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Category *"</label>
                                <select class="folio-select" on:change=move |ev| file_category.set(event_target_value(&ev))>
                                    {ViolCategory::ALL.iter().copied().map(|c| {
                                        view! { <option value=c.api_str()>{c.label()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Subject *"</label>
                                <input
                                    type="text"
                                    class="folio-input"
                                    prop:value=file_subject
                                    on:input=move |ev| file_subject.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Description"</label>
                                <textarea
                                    class="folio-input"
                                    rows="3"
                                    prop:value=file_description
                                    on:input=move |ev| file_description.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Cure Days (optional)"</label>
                                <input
                                    type="number"
                                    class="folio-input"
                                    min="1"
                                    max="90"
                                    placeholder="e.g. 10"
                                    prop:value=file_cure_days
                                    on:input=move |ev| file_cure_days.set(event_target_value(&ev))
                                />
                            </div>
                            {move || file_err.get().map(|e| view! {
                                <p class="viol-error-text">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button type="button" class="folio-btn folio-btn--ghost" on:click=move |_| show_file.set(false)>"Cancel"</button>
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary"
                                disabled=move || filing.get() || file_subject.get().trim().is_empty()
                                on:click=on_file
                            >
                                {move || if filing.get() { "Filing…" } else { "File Violation" }}
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
fn ViolTable(records: Vec<ViolationRecord>, refresh: RwSignal<u32>) -> impl IntoView {
    view! {
        <div class="viol-table-wrap">
            <table class="viol-table">
                <thead>
                    <tr>
                        <th class="viol-th">"Status"</th>
                        <th class="viol-th">"Category"</th>
                        <th class="viol-th">"Subject"</th>
                        <th class="viol-th">"Asset"</th>
                        <th class="viol-th">"Type"</th>
                        <th class="viol-th">"Cure Deadline"</th>
                        <th class="viol-th">"Filed"</th>
                        <th class="viol-th">"Action"</th>
                    </tr>
                </thead>
                <tbody>
                    {records.into_iter().map(|v| {
                        let status      = CureStatus::from_str(&v.cure_status);
                        let cat         = ViolCategory::from_str(&v.category);
                        let asset       = v.asset_id.map(|id| id.to_string().split('-').next().unwrap_or("").to_uppercase()).unwrap_or_else(|| "\u{2014}".to_string());
                        let kind        = if v.reservation_id.is_some() { "STR" } else { "LTR" };
                        let deadline    = v.cure_deadline.map(|d| d.to_string()).unwrap_or_else(|| "\u{2014}".to_string());
                        let filed       = v.filed_at.format("%Y-%m-%d").to_string();
                        let is_escalated= status == CureStatus::Escalated;
                        let row_cls     = if is_escalated { "viol-row viol-row--escalated" } else { "viol-row" };
                        let is_open     = status == CureStatus::Open;
                        let vid         = v.id.to_string();
                        let pending     = RwSignal::new(false);
                        let row_err     = RwSignal::new(None::<String>);
                        view! {
                            <tr class=row_cls>
                                <td class="viol-td">
                                    <span class={format!("viol-status-badge {}", status.pill_class())}>
                                        <span class="material-symbols-outlined" style="font-size:11px;">
                                            {status.material_icon()}
                                        </span>
                                        {status.as_str()}
                                    </span>
                                </td>
                                <td class="viol-td">
                                    <span class="viol-cat-badge">
                                        <span class="material-symbols-outlined" style="font-size:12px;">
                                            {cat.material_icon()}
                                        </span>
                                        {cat.label()}
                                    </span>
                                </td>
                                <td class="viol-td viol-td--subject">{v.subject}</td>
                                <td class="viol-td viol-td--mono">{asset}</td>
                                <td class="viol-td">
                                    <span class={format!("viol-kind-badge viol-kind--{}", kind.to_lowercase())}>
                                        {kind}
                                    </span>
                                </td>
                                <td class="viol-td viol-td--deadline">{deadline}</td>
                                <td class="viol-td">{filed}</td>
                                <td class="viol-td">
                                    {if is_open {
                                        let vid2 = vid.clone();
                                        view! {
                                            <div style="display:flex;flex-direction:column;gap:0.25rem;">
                                                <select
                                                    class="folio-select"
                                                    style="min-width:8rem;font-size:0.75rem;"
                                                    prop:disabled=move || pending.get()
                                                    on:change=move |ev| {
                                                        let next = event_target_value(&ev);
                                                        if next.is_empty() { return; }
                                                        let id = vid2.clone();
                                                        pending.set(true);
                                                        row_err.set(None);
                                                        spawn_local(async move {
                                                            match update_violation_status(id, next, None).await {
                                                                Ok(_) => refresh.update(|n| *n += 1),
                                                                Err(e) => row_err.set(Some(e.to_string())),
                                                            }
                                                            pending.set(false);
                                                        });
                                                    }
                                                >
                                                    <option value="">"Update…"</option>
                                                    {CureStatus::TRANSITIONS.iter().copied().map(|s| {
                                                        view! { <option value=s.api_str()>{s.as_str()}</option> }
                                                    }).collect_view()}
                                                </select>
                                                {move || row_err.get().map(|e| view! {
                                                    <span style="color:#b91c1c;font-size:0.7rem;">{e}</span>
                                                })}
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span style="color:var(--folio-muted);">"\u{2014}"</span> }.into_any()
                                    }}
                                </td>
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
fn ViolSkeleton(rows: usize) -> impl IntoView {
    view! {
        <div class="viol-table-wrap">
            <table class="viol-table">
                <thead>
                    <tr>
                        <th class="viol-th">"Status"</th>
                        <th class="viol-th">"Category"</th>
                        <th class="viol-th">"Subject"</th>
                        <th class="viol-th">"Asset"</th>
                        <th class="viol-th">"Type"</th>
                        <th class="viol-th">"Cure Deadline"</th>
                        <th class="viol-th">"Filed"</th>
                    </tr>
                </thead>
                <tbody>
                    {(0..rows).map(|_| view! {
                        <tr class="viol-row">
                            <td class="viol-td"><div class="viol-skel viol-skel--badge"/></td>
                            <td class="viol-td"><div class="viol-skel viol-skel--badge"/></td>
                            <td class="viol-td"><div class="viol-skel viol-skel--text"/></td>
                            <td class="viol-td"><div class="viol-skel viol-skel--sm"/></td>
                            <td class="viol-td"><div class="viol-skel viol-skel--sm"/></td>
                            <td class="viol-td"><div class="viol-skel viol-skel--sm"/></td>
                            <td class="viol-td"><div class="viol-skel viol-skel--sm"/></td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

// ── Server function ───────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

/// GET /api/folio/violations
#[server(ListViolations, "/api")]
pub async fn list_violations() -> Result<Vec<ViolationRecord>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<ViolationRecord>>(
        "/api/folio/violations",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Violations list failed: {e}")))
}

#[derive(Serialize)]
struct FileViolationBody {
    asset_id: Uuid,
    contract_id: Option<Uuid>,
    reservation_id: Option<Uuid>,
    category: String,
    subject: String,
    description: String,
    cure_days: Option<u8>,
    evidence_notes: Option<String>,
}

/// POST /api/folio/violations
#[server(FileViolation, "/api")]
pub async fn file_violation(
    asset_id: String,
    category: String,
    subject: String,
    description: String,
    cure_days: Option<u8>,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let asset_id = Uuid::parse_str(asset_id.trim())
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid asset ID"))?;
    if ViolCategory::ALL
        .iter()
        .all(|c| c.api_str() != category.as_str())
    {
        return Err(server_fn::error::ServerFnError::new("Invalid category"));
    }
    if subject.trim().is_empty() {
        return Err(server_fn::error::ServerFnError::new("Subject is required"));
    }

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let body = FileViolationBody {
        asset_id,
        contract_id: None,
        reservation_id: None,
        category,
        subject: subject.trim().to_string(),
        description,
        cure_days,
        evidence_notes: None,
    };
    let resp = crate::atlas_client::authenticated_post::<FileViolationBody, ViolationRecord>(
        "/api/folio/violations",
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("File violation failed: {e}")))?;
    Ok(resp.id)
}

#[derive(Serialize)]
struct UpdateCureStatusBody {
    status: String,
    resolution_notes: Option<String>,
}

/// PATCH /api/folio/violations/{id}/status
#[server(UpdateViolationStatus, "/api")]
pub async fn update_violation_status(
    violation_id: String,
    status: String,
    resolution_notes: Option<String>,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let _ = Uuid::parse_str(violation_id.trim())
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid violation ID"))?;
    if !matches!(status.as_str(), "cured" | "escalated" | "dismissed") {
        return Err(server_fn::error::ServerFnError::new("Invalid cure status"));
    }

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let body = UpdateCureStatusBody {
        status,
        resolution_notes,
    };
    crate::atlas_client::authenticated_patch::<UpdateCureStatusBody, ViolationRecord>(
        &format!("/api/folio/violations/{violation_id}/status"),
        &token,
        body,
    )
    .await
    .map(|_| ())
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Status update failed: {e}")))
}
