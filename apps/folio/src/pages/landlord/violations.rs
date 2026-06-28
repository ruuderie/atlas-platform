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
use serde::{Deserialize, Serialize};

// ── Response types ────────────────────────────────────────────────────────────

/// Mirrors ViolationRecord from the violation service.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ViolationRecord {
    pub id:               uuid::Uuid,
    pub asset_id:         Option<uuid::Uuid>,
    pub contract_id:      Option<uuid::Uuid>,
    pub reservation_id:   Option<uuid::Uuid>,
    pub category:         String,
    pub subject:          String,
    pub description:      Option<String>,
    pub cure_status:      String,
    pub cure_deadline:    Option<chrono::NaiveDate>,
    pub filed_at:         chrono::DateTime<chrono::Utc>,
    pub resolved_at:      Option<chrono::DateTime<chrono::Utc>>,
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
    pub fn from_str(s: &str) -> Self {
        match s {
            "open"      => Self::Open,
            "cured"     => Self::Cured,
            "escalated" => Self::Escalated,
            "dismissed" => Self::Dismissed,
            _           => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open      => "Open",
            Self::Cured     => "Cured",
            Self::Escalated => "Escalated",
            Self::Dismissed => "Dismissed",
            Self::Unknown   => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Open      => "vs--open",
            Self::Cured     => "vs--cured",
            Self::Escalated => "vs--escalated",
            Self::Dismissed => "vs--dismissed",
            Self::Unknown   => "vs--unknown",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Open      => "radio_button_unchecked",
            Self::Cured     => "check_circle",
            Self::Escalated => "warning",
            Self::Dismissed => "cancel",
            Self::Unknown   => "help",
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
    pub fn from_str(s: &str) -> Self {
        match s {
            "noise"                 => Self::Noise,
            "unauthorized_occupant" => Self::UnauthorizedOccupant,
            "unauthorized_pet"      => Self::UnauthorizedPet,
            "unauthorized_vehicle"  => Self::UnauthorizedVehicle,
            "property_damage"       => Self::PropertyDamage,
            "lease_breach"          => Self::LeaseBreach,
            "subletting"            => Self::Subletting,
            "failure_to_maintain"   => Self::FailureToMaintain,
            "illegal_activity"      => Self::IllegalActivity,
            "hoarding"              => Self::Hoarding,
            "smoking_in_unit"       => Self::SmokingInUnit,
            "unauthorized_party"    => Self::UnauthorizedParty,
            "over_occupancy"        => Self::OverOccupancy,
            "other"                 => Self::Other,
            _                       => Self::Unknown,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Noise                => "Noise",
            Self::UnauthorizedOccupant => "Unauth. Occupant",
            Self::UnauthorizedPet      => "Unauth. Pet",
            Self::UnauthorizedVehicle  => "Unauth. Vehicle",
            Self::PropertyDamage       => "Property Damage",
            Self::LeaseBreach          => "Lease Breach",
            Self::Subletting           => "Subletting",
            Self::FailureToMaintain    => "Failure to Maintain",
            Self::IllegalActivity      => "Illegal Activity",
            Self::Hoarding             => "Hoarding",
            Self::SmokingInUnit        => "Smoking",
            Self::UnauthorizedParty    => "Unauth. Party (STR)",
            Self::OverOccupancy        => "Over-Occupancy (STR)",
            Self::Other                => "Other",
            Self::Unknown              => "Unknown",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Noise                => "volume_up",
            Self::UnauthorizedOccupant => "person_off",
            Self::UnauthorizedPet      => "pets",
            Self::UnauthorizedVehicle  => "directions_car",
            Self::PropertyDamage       => "construction",
            Self::LeaseBreach          => "description",
            Self::Subletting           => "swap_horiz",
            Self::FailureToMaintain    => "cleaning_services",
            Self::IllegalActivity      => "gavel",
            Self::Hoarding             => "inventory_2",
            Self::SmokingInUnit        => "smoking_rooms",
            Self::UnauthorizedParty    => "celebration",
            Self::OverOccupancy        => "group",
            Self::Other                => "more_horiz",
            Self::Unknown              => "help",
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
            Self::All       => "All",
            Self::Open      => "Open",
            Self::Escalated => "Escalated",
            Self::Resolved  => "Resolved",
        }
    }

    pub fn matches(self, s: CureStatus) -> bool {
        match self {
            Self::All       => true,
            Self::Open      => s == CureStatus::Open,
            Self::Escalated => s == CureStatus::Escalated,
            Self::Resolved  => matches!(s, CureStatus::Cured | CureStatus::Dismissed),
        }
    }
}

// ── Page ──────────────────────────────────────────────────────────────────────

/// Violations — portfolio-wide compliance queue.
#[component]
pub fn Violations() -> impl IntoView {
    let (status_filter, set_status) = signal(StatusFilter::All);
    let (search_query, set_search) = signal(String::new());

    let violations = Resource::new(
        || (),
        |_| async move { list_violations().await },
    );

    view! {
        <div class="viol-page">
            // ── Header ────────────────────────────────────────────────
            <div class="viol-header">
                <div>
                    <h1 class="viol-title">"Violations"</h1>
                    <p class="viol-subtitle">"Compliance queue — cure deadlines and escalation status."</p>
                </div>
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
                            view! { <ViolTable records=filtered/> }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

// ── Table component ───────────────────────────────────────────────────────────

#[component]
fn ViolTable(records: Vec<ViolationRecord>) -> impl IntoView {
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
                    {records.into_iter().map(|v| {
                        let status      = CureStatus::from_str(&v.cure_status);
                        let cat         = ViolCategory::from_str(&v.category);
                        let asset       = v.asset_id.map(|id| id.to_string().split('-').next().unwrap_or("").to_uppercase()).unwrap_or_else(|| "\u{2014}".to_string());
                        let kind        = if v.reservation_id.is_some() { "STR" } else { "LTR" };
                        let deadline    = v.cure_deadline.map(|d| d.to_string()).unwrap_or_else(|| "\u{2014}".to_string());
                        let filed       = v.filed_at.format("%Y-%m-%d").to_string();
                        let is_escalated= status == CureStatus::Escalated;
                        let row_cls     = if is_escalated { "viol-row viol-row--escalated" } else { "viol-row" };
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

/// GET /api/folio/violations
#[server(ListViolations, "/api")]
pub async fn list_violations(
) -> Result<Vec<ViolationRecord>, server_fn::error::ServerFnError> {
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
