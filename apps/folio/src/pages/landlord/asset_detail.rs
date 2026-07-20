// apps/folio/src/pages/landlord/asset_detail.rs
//
// Asset Detail — lets the operator inspect a single property or unit.
//
// Timeline sources:
//   G-13 atlas_cases  → GET /api/folio/assets/{id}/inspections
//     Maintenance tickets + scheduled inspections. Contractor is first-class on the case.
//   G-21 atlas_events → GET /api/folio/events?subject_entity_type=atlas_asset&subject_entity_id={id}
//     Open houses, showings, training events scheduled against this asset.
//
// Default contractor:
//   G-22 atlas_record_relationships (relationship_type = "default_contractor")
//   GET / POST / DELETE via /api/folio/assets/{id}/contractor + /api/folio/relationships

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::components::nav::{FolioRoute, NavIcon};
use crate::components::page_header::PageHeader;

// ── Response types (mirror backend shapes) ────────────────────────────────────

/// Mirrors `AssetDetail` from `GET /api/folio/assets/:id`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetDetailModel {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub portfolio_id: Option<uuid::Uuid>,
    pub asset_type: String,
    pub name: String,
    pub serial_or_folio_number: Option<String>,
    pub status: String,
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub country_code: Option<String>,
    pub postal_code: Option<String>,
    pub attributes: Option<serde_json::Value>,
    // G-10 lifecycle extension fields
    pub scheduled_service_date: Option<chrono::NaiveDate>,
    pub expiry_date: Option<chrono::NaiveDate>,
    pub condition: Option<String>,
    pub lifecycle_metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Mirrors `AssetCaseSummary` from `GET /api/folio/assets/:id/inspections`.
/// Backed by G-13 `atlas_cases` — these are *completable tasks*, not scheduled occurrences.
/// Contractor is first-class: `assigned_vendor_name` is denormalized by the backend.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetCaseSummary {
    pub id: uuid::Uuid,
    pub case_type: String,
    pub subject: String,
    pub status: String,
    pub priority: String,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_cost_cents: Option<i64>,
    pub actual_cost_cents: Option<i64>,
    pub assigned_vendor_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Mirrors G-21 event shape from `GET /api/folio/events?subject_entity_type=atlas_asset&…`.
/// Backed by G-21 `atlas_events` — these are *scheduled occurrences* (open houses, showings).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetEventSummary {
    pub id: uuid::Uuid,
    pub name: String,
    pub event_type: String,
    pub status: String,
    pub starts_at: chrono::DateTime<chrono::Utc>,
    pub ends_at: chrono::DateTime<chrono::Utc>,
    pub venue_name: Option<String>,
}

/// Mirrors `AssetContractorSummary` from `GET /api/folio/assets/:id/contractor`.
/// This is the *default dispatch suggestion* for this asset — not ownership.
/// The actual contractor per job lives on `atlas_cases.assigned_service_provider_id`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetContractorSummary {
    pub vendor_id: uuid::Uuid,
    pub business_name: String,
    pub primary_trade: Option<String>,
    pub relationship_type: String,
}

/// Minimal vendor list item for the contractor selector.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VendorListItem {
    pub id: uuid::Uuid,
    pub business_name: String,
    pub primary_trade: Option<String>,
}

// ── Finite value sets as enums ────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetStatus {
    Active,
    Inactive,
    Pending,
    Archived,
    Unknown,
}

impl AssetStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "inactive" => Self::Inactive,
            "pending" => Self::Pending,
            "archived" => Self::Archived,
            _ => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Inactive => "Inactive",
            Self::Pending => "Pending",
            Self::Archived => "Archived",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Active => "asset-status-pill--active",
            Self::Inactive => "asset-status-pill--inactive",
            Self::Pending => "asset-status-pill--pending",
            Self::Archived => "asset-status-pill--archived",
            Self::Unknown => "asset-status-pill--unknown",
        }
    }
}

/// G-10 condition values: "excellent" | "good" | "fair" | "poor" | "retired"
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetCondition {
    Excellent,
    Good,
    Fair,
    Poor,
    Retired,
    Unknown,
}

impl AssetCondition {
    pub fn from_str(s: &str) -> Self {
        match s {
            "excellent" => Self::Excellent,
            "good" => Self::Good,
            "fair" => Self::Fair,
            "poor" => Self::Poor,
            "retired" => Self::Retired,
            _ => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::Poor => "Poor",
            Self::Retired => "Retired",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Excellent => "asset-cond-pill--excellent",
            Self::Good => "asset-cond-pill--good",
            Self::Fair => "asset-cond-pill--fair",
            Self::Poor => "asset-cond-pill--poor",
            Self::Retired => "asset-cond-pill--retired",
            Self::Unknown => "asset-cond-pill--unknown",
        }
    }
}

/// G-13 PmCaseType values relevant to asset history.
/// Icon drives visual category; label is the subtitle shown under the case subject.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetCaseType {
    Maintenance,
    ScheduledInspection,
    ComplianceViolation,
    LeaseRenewal,
    MoveOut,
    ApplicationReview,
    ReportRequest,
    Unknown,
}

impl AssetCaseType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "maintenance" => Self::Maintenance,
            "scheduled_inspection" => Self::ScheduledInspection,
            "compliance_violation" => Self::ComplianceViolation,
            "lease_renewal" => Self::LeaseRenewal,
            "move_out" => Self::MoveOut,
            "application_review" => Self::ApplicationReview,
            "report_request" => Self::ReportRequest,
            _ => Self::Unknown,
        }
    }

    pub const fn display_label(self) -> &'static str {
        match self {
            Self::Maintenance => "Maintenance",
            Self::ScheduledInspection => "Inspection",
            Self::ComplianceViolation => "Compliance",
            Self::LeaseRenewal => "Lease Renewal",
            Self::MoveOut => "Move Out",
            Self::ApplicationReview => "Application",
            Self::ReportRequest => "Report Request",
            Self::Unknown => "Work Item",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Maintenance => "build",
            Self::ScheduledInspection => "fact_check",
            Self::ComplianceViolation => "gavel",
            Self::LeaseRenewal => "autorenew",
            Self::MoveOut => "move_item",
            Self::ApplicationReview => "person_search",
            Self::ReportRequest => "description",
            Self::Unknown => "work",
        }
    }

    pub const fn icon_class(self) -> &'static str {
        match self {
            Self::Maintenance | Self::ScheduledInspection => "asset-tl-icon--maintenance",
            Self::ComplianceViolation => "asset-tl-icon--compliance",
            Self::LeaseRenewal | Self::MoveOut => "asset-tl-icon--lease",
            Self::ApplicationReview | Self::ReportRequest | Self::Unknown => {
                "asset-tl-icon--scheduled"
            }
        }
    }
}

/// G-21 EventType values that may appear for a PM asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetScheduledEventType {
    OpenHouse,
    Training,
    Conference,
    Meetup,
    VenueBooking,
    Unknown,
}

impl AssetScheduledEventType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "open_house" => Self::OpenHouse,
            "training" => Self::Training,
            "conference" => Self::Conference,
            "meetup" => Self::Meetup,
            "venue_booking" => Self::VenueBooking,
            _ => Self::Unknown,
        }
    }

    pub const fn display_label(self) -> &'static str {
        match self {
            Self::OpenHouse => "Open House",
            Self::Training => "Training",
            Self::Conference => "Conference",
            Self::Meetup => "Meetup",
            Self::VenueBooking => "Venue Booking",
            Self::Unknown => "Scheduled Event",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::OpenHouse => "home",
            Self::Training => "school",
            Self::Conference => "groups",
            Self::Meetup => "handshake",
            Self::VenueBooking => "event_seat",
            Self::Unknown => "event",
        }
    }
}

/// Status for G-13 cases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseStatus {
    Open,
    InProgress,
    Scheduled,
    Completed,
    Cancelled,
    Unknown,
}

impl CaseStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "open" => Self::Open,
            "in_progress" => Self::InProgress,
            "scheduled" => Self::Scheduled,
            "completed" => Self::Completed,
            "cancelled" => Self::Cancelled,
            _ => Self::Unknown,
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Completed => "asset-event-pill--complete",
            Self::Cancelled => "asset-event-pill--cancelled",
            Self::InProgress => "asset-event-pill--active",
            Self::Open | Self::Scheduled | Self::Unknown => "asset-event-pill--scheduled",
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::InProgress => "In Progress",
            Self::Scheduled => "Scheduled",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn tl_icon_class(self) -> &'static str {
        match self {
            Self::Completed => "asset-tl-icon--complete",
            Self::Cancelled => "asset-tl-icon--cancelled",
            Self::InProgress | Self::Open | Self::Scheduled | Self::Unknown => {
                "asset-tl-icon--scheduled"
            }
        }
    }
}

// ── Page component ────────────────────────────────────────────────────────────

#[component]
pub fn AssetDetail() -> impl IntoView {
    let params = use_params_map();
    let asset_id = move || params.get().get("id").unwrap_or_default().to_string();

    let asset = Resource::new(asset_id.clone(), |id| async move {
        if id.is_empty() {
            return Err(server_fn::error::ServerFnError::new("No asset ID in URL"));
        }
        get_asset_detail(id).await
    });

    // G-13 cases (maintenance + inspections) — loads in parallel with events
    let cases = Resource::new(asset_id.clone(), |id| async move {
        if id.is_empty() {
            return Ok(vec![]);
        }
        get_asset_cases(id).await
    });

    // G-21 events (open houses, showings, etc.) — loads in parallel with cases
    let events = Resource::new(asset_id.clone(), |id| async move {
        if id.is_empty() {
            return Ok(vec![]);
        }
        get_asset_events(id).await
    });

    // Default contractor — used only to trigger initial contractor_managed load.
    let _contractor = Resource::new(asset_id.clone(), |id| async move {
        if id.is_empty() {
            return Ok(None);
        }
        get_asset_contractor(id).await
    });

    // Signal to refetch contractor after set/remove actions
    let (contractor_refresh, set_contractor_refresh) = signal(0u32);
    let asset_id_for_contractor = asset_id.clone();
    let contractor_managed = Resource::new(
        move || (asset_id_for_contractor(), contractor_refresh.get()),
        |(id, _)| async move {
            if id.is_empty() {
                return Ok(None);
            }
            get_asset_contractor(id).await
        },
    );

    view! {
        <div class="asset-detail-page">
            <Suspense fallback=|| view! { <AssetDetailSkeleton/> }>
                {move || asset.get().map(|result| match result {
                    Ok(detail) => view! {
                        <AssetDetailContent
                            detail=detail
                            cases=cases
                            events=events
                            contractor=contractor_managed
                            set_contractor_refresh=set_contractor_refresh
                        />
                    }.into_any(),
                    Err(e) => view! { <AssetDetailError message=e.to_string()/> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

// ── Content component ─────────────────────────────────────────────────────────

#[component]
fn AssetDetailContent(
    detail: AssetDetailModel,
    cases: Resource<Result<Vec<AssetCaseSummary>, server_fn::error::ServerFnError>>,
    events: Resource<Result<Vec<AssetEventSummary>, server_fn::error::ServerFnError>>,
    contractor: Resource<Result<Option<AssetContractorSummary>, server_fn::error::ServerFnError>>,
    set_contractor_refresh: WriteSignal<u32>,
) -> impl IntoView {
    let status = AssetStatus::from_str(&detail.status);
    let condition = detail.condition.as_deref().map(AssetCondition::from_str);
    let asset_id = detail.id.to_string();

    let address_display = {
        let parts: Vec<&str> = [
            detail.address_line_1.as_deref(),
            detail.city.as_deref(),
            detail.state_province.as_deref(),
            detail.country_code.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect();
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(", "))
        }
    };

    let attribute_pairs: Vec<(String, String)> = detail
        .attributes
        .as_ref()
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| {
                    let display_key = k.replace('_', " ");
                    let display_val = match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Number(n) => n.to_string(),
                        serde_json::Value::Bool(b) => {
                            if *b {
                                "Yes".into()
                            } else {
                                "No".into()
                            }
                        }
                        _ => v.to_string(),
                    };
                    (display_key, display_val)
                })
                .collect()
        })
        .unwrap_or_default();

    let asset_name = detail.name.clone();
    let asset_type = detail.asset_type.clone();
    let folio_number = detail.serial_or_folio_number.clone();
    let svc_date = detail
        .scheduled_service_date
        .map(|d| d.format("%b %-d, %Y").to_string());
    let exp_date = detail
        .expiry_date
        .map(|d| d.format("%b %-d, %Y").to_string());
    let created = detail.created_at.format("%b %-d, %Y").to_string();
    let has_lifecycle = svc_date.is_some() || exp_date.is_some() || condition.is_some();

    view! {
        <div class="asset-detail-layout">

            // ── Breadcrumb ──────────────────────────────────────────────────
            <nav class="asset-breadcrumb" aria-label="Breadcrumb">
                <a href=FolioRoute::LandlordAssets.path() class="asset-breadcrumb__link">
                    <span class="material-symbols-outlined asset-breadcrumb__icon">
                        {NavIcon::Apartment.as_str()}
                    </span>
                    "Assets"
                </a>
                <span class="material-symbols-outlined asset-breadcrumb__sep">
                    {NavIcon::ChevronRight.as_str()}
                </span>
                <span class="asset-breadcrumb__current">{asset_name.clone()}</span>
            </nav>

            <div class="asset-detail-body">

                // ── LEFT COLUMN ─────────────────────────────────────────────
                <div class="asset-detail-main">

                    {
                        let title = Signal::derive({
                            let n = asset_name.clone();
                            move || n.clone()
                        });
                        let subtitle_text = address_display
                            .clone()
                            .unwrap_or_else(|| asset_type.replace('_', " "));
                        let subtitle = Signal::derive(move || subtitle_text.clone());
                        view! {
                            <PageHeader title=title subtitle=subtitle />
                        }
                    }
                    <div class="asset-header-pills">
                        <span class="asset-type-pill">{asset_type.replace('_', " ")}</span>
                        <span class={format!("asset-status-pill {}", status.pill_class())}>
                            {status.as_str()}
                        </span>
                        {condition.map(|c| view! {
                            <span class={format!("asset-status-pill {}", c.pill_class())}>
                                {c.as_str()}
                            </span>
                        })}
                    </div>

                    // Lifecycle status grid (G-10 fields)
                    {has_lifecycle.then(|| view! {
                        <div class="asset-detail-card">
                            <p class="asset-section-label">"Lifecycle Status"</p>
                            <div class="asset-lifecycle-grid">
                                {svc_date.map(|d| view! {
                                    <div class="asset-lifecycle-cell">
                                        <p class="asset-lifecycle-cell__label">"Next Service"</p>
                                        <p class="asset-lifecycle-cell__value">{d}</p>
                                    </div>
                                })}
                                {exp_date.map(|d| view! {
                                    <div class="asset-lifecycle-cell">
                                        <p class="asset-lifecycle-cell__label">"Expiry"</p>
                                        <p class="asset-lifecycle-cell__value">{d}</p>
                                    </div>
                                })}
                                {condition.map(|c| view! {
                                    <div class="asset-lifecycle-cell">
                                        <p class="asset-lifecycle-cell__label">"Condition"</p>
                                        <p class="asset-lifecycle-cell__value">{c.as_str()}</p>
                                    </div>
                                })}
                            </div>
                        </div>
                    })}

                    // Unified timeline — merges G-13 cases + G-21 events
                    <div class="asset-detail-card">
                        <div class="asset-section-header">
                            <p class="asset-section-label">"Activity & Events"</p>
                            <a href=FolioRoute::LandlordMaintenance.path() class="asset-section-action">
                                <span class="material-symbols-outlined" style="font-size:14px;">
                                    {NavIcon::Build.as_str()}
                                </span>
                                "New Work Order"
                            </a>
                        </div>

                        // G-13 Cases section
                        <Suspense fallback=|| view! { <div class="asset-timeline-loading"/> }>
                            {move || cases.get().map(|result| match result {
                                Ok(cs) if cs.is_empty() => view! {
                                    <div class="asset-empty-timeline">
                                        <span class="material-symbols-outlined asset-empty-icon">
                                            {NavIcon::Build.as_str()}
                                        </span>
                                        <p class="asset-empty-text">"No maintenance or inspections recorded."</p>
                                        <p class="asset-empty-sub">
                                            "Schedule an inspection at "
                                            <a href=FolioRoute::LandlordMaintenance.path() class="asset-inline-link">
                                                "Maintenance"
                                            </a>
                                            ", or file a ticket via "
                                            <code class="asset-inline-code">
                                                "POST /api/folio/maintenance"
                                            </code>
                                            "."
                                        </p>
                                    </div>
                                }.into_any(),
                                Ok(cs) => view! {
                                    <div class="asset-timeline-section-label">"Work Orders & Inspections"</div>
                                    <div class="asset-timeline">
                                        {cs.into_iter().map(|c| {
                                            let case_type = AssetCaseType::from_str(&c.case_type);
                                            let case_status = CaseStatus::from_str(&c.status);
                                            let date_str = c.scheduled_at
                                                .or(c.completed_at)
                                                .map(|dt| dt.format("%b %-d, %Y").to_string())
                                                .unwrap_or_else(|| c.created_at.format("%b %-d, %Y").to_string());
                                            let cost_str = c.actual_cost_cents
                                                .or(c.estimated_cost_cents)
                                                .map(|cents| format!("${:.2}", cents as f64 / 100.0));
                                            view! {
                                                <div class="asset-timeline-item">
                                                    <div class={format!(
                                                        "asset-tl-icon {} {}",
                                                        case_type.icon_class(),
                                                        case_status.tl_icon_class()
                                                    )}>
                                                        <span class="material-symbols-outlined" style="font-size:14px;">
                                                            {case_type.material_icon()}
                                                        </span>
                                                    </div>
                                                    <div class="asset-tl-body">
                                                        <div class="asset-tl-header">
                                                            <div>
                                                                <p class="asset-tl-name">{c.subject}</p>
                                                                <p class="asset-tl-meta">
                                                                    {case_type.display_label()}
                                                                    {format!(" \u{00b7} {date_str}")}
                                                                    {c.assigned_vendor_name.map(|v|
                                                                        format!(" \u{00b7} {v}")
                                                                    )}
                                                                </p>
                                                            </div>
                                                            <div class="asset-tl-badges">
                                                                <span class={format!(
                                                                    "asset-event-pill {}",
                                                                    case_status.pill_class()
                                                                )}>
                                                                    {case_status.as_str()}
                                                                </span>
                                                                {cost_str.map(|c| view! {
                                                                    <span class="asset-cost-badge">{c}</span>
                                                                })}
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <p class="asset-meta-empty">
                                        {format!("Could not load work orders: {e}")}
                                    </p>
                                }.into_any(),
                            })}
                        </Suspense>

                        // G-21 Events section
                        <Suspense fallback=|| view! { <div class="asset-timeline-loading" style="margin-top:.75rem;"/> }>
                            {move || events.get().map(|result| match result {
                                Ok(evs) if !evs.is_empty() => view! {
                                    <div class="asset-timeline-section-label" style="margin-top:1.25rem;">
                                        "Scheduled Events"
                                    </div>
                                    <div class="asset-timeline">
                                        {evs.into_iter().map(|ev| {
                                            let ev_type = AssetScheduledEventType::from_str(&ev.event_type);
                                            let starts = ev.starts_at.format("%b %-d, %Y \u{00b7} %H:%M").to_string();
                                            view! {
                                                <div class="asset-timeline-item">
                                                    <div class="asset-tl-icon asset-tl-icon--event">
                                                        <span class="material-symbols-outlined" style="font-size:14px;">
                                                            {ev_type.material_icon()}
                                                        </span>
                                                    </div>
                                                    <div class="asset-tl-body">
                                                        <div class="asset-tl-header">
                                                            <div>
                                                                <p class="asset-tl-name">{ev.name}</p>
                                                                <p class="asset-tl-meta">
                                                                    {ev_type.display_label()}
                                                                    {format!(" \u{00b7} {starts}")}
                                                                    {ev.venue_name.map(|vn| format!(" \u{00b7} {vn}"))}
                                                                </p>
                                                            </div>
                                                            <span class="asset-event-pill asset-event-pill--scheduled">
                                                                {ev.status}
                                                            </span>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any(),
                                Ok(_) => view! { <></> }.into_any(),
                                Err(_) => view! { <></> }.into_any(),
                            })}
                        </Suspense>
                    </div>

                </div>

                // ── RIGHT COLUMN ────────────────────────────────────────────
                <aside class="asset-detail-sidebar">

                    // Quick actions
                    <div class="asset-detail-card asset-actions-card">
                        <p class="asset-section-label">"Quick Actions"</p>
                        <div class="asset-actions-list">
                            <a href=FolioRoute::LandlordMaintenance.path()
                               class="asset-action-btn asset-action-btn--primary">
                                <span class="material-symbols-outlined" style="font-size:18px;">
                                    {NavIcon::Build.as_str()}
                                </span>
                                "Create Work Order"
                            </a>
                            <a href=FolioRoute::LandlordLeases.path()
                               class="asset-action-btn asset-action-btn--secondary">
                                <span class="material-symbols-outlined" style="font-size:18px;">
                                    {NavIcon::Description.as_str()}
                                </span>
                                "View Leases"
                            </a>
                        </div>
                    </div>

                    // Default Contractor (G-22)
                    <DefaultContractorPanel
                        asset_id=asset_id.clone()
                        contractor=contractor
                        set_refresh=set_contractor_refresh
                    />

                    // Specifications
                    <div class="asset-detail-card">
                        <p class="asset-section-label">"Specifications"</p>
                        {folio_number.map(|n| view! {
                            <div class="asset-meta-row">
                                <p class="asset-meta-key">"Folio / Serial"</p>
                                <p class="asset-meta-val asset-meta-mono">{n}</p>
                            </div>
                        })}
                        <div class="asset-meta-row">
                            <p class="asset-meta-key">"Registered"</p>
                            <p class="asset-meta-val">{created}</p>
                        </div>
                        {if attribute_pairs.is_empty() {
                            view! {
                                <p class="asset-meta-empty">"No additional specifications recorded."</p>
                            }.into_any()
                        } else {
                            attribute_pairs.into_iter().map(|(key, val)| view! {
                                <div class="asset-meta-row">
                                    <p class="asset-meta-key">{key}</p>
                                    <p class="asset-meta-val">{val}</p>
                                </div>
                            }).collect_view().into_any()
                        }}
                    </div>

                    <AssetArchivePanel asset_id=asset_id.clone()/>

                </aside>
            </div>
        </div>
    }
}

/// Soft-archive (type DELETE) for leaf assets — muted trigger + modal, not a danger-zone card.
#[component]
fn AssetArchivePanel(asset_id: String) -> impl IntoView {
    use crate::pages::landlord::asset_api::{archive_folio_asset, ArchiveBlockerDto};

    let show_archive = RwSignal::new(false);
    let archive_confirm = RwSignal::new(String::new());
    let archive_pending = RwSignal::new(false);
    let archive_err = RwSignal::new(None::<String>);
    let archive_blockers = RwSignal::new(Vec::<ArchiveBlockerDto>::new());
    let archived_ok = RwSignal::new(false);
    let aid = StoredValue::new(asset_id);

    let on_archive = move |_| {
        if archive_confirm.get().trim() != "DELETE" {
            archive_err.set(Some("Type DELETE to confirm.".into()));
            return;
        }
        archive_pending.set(true);
        archive_err.set(None);
        archive_blockers.set(vec![]);
        let id = aid.get_value();
        spawn_local(async move {
            match archive_folio_asset(id).await {
                Ok(outcome) => {
                    if outcome.archived {
                        archived_ok.set(true);
                        show_archive.set(false);
                    } else {
                        archive_blockers.set(outcome.blockers);
                        archive_err.set(Some(
                            "This asset cannot be archived until the items below are resolved."
                                .into(),
                        ));
                    }
                }
                Err(e) => archive_err.set(Some(e.to_string())),
            }
            archive_pending.set(false);
        });
    };

    view! {
        <div class="hub-archive-foot">
            {move || if archived_ok.get() {
                view! {
                    <p class="hub-archive-foot__ok">"Asset archived."</p>
                }.into_any()
            } else {
                view! {
                    <button
                        type="button"
                        class="hub-archive-foot__link"
                        on:click=move |_| {
                            archive_err.set(None);
                            archive_blockers.set(vec![]);
                            archive_confirm.set(String::new());
                            show_archive.set(true);
                        }
                    >
                        "Archive asset…"
                    </button>
                }.into_any()
            }}
        </div>

        <Show when=move || show_archive.get()>
            <div class="modal-backdrop">
                <div class="modal-card" style="max-width:28rem;">
                    <div class="modal-header">
                        <h3 class="modal-title">"Archive asset"</h3>
                        <button
                            class="modal-close"
                            on:click=move |_| show_archive.set(false)
                        >
                            <span class="material-symbols-outlined">"close"</span>
                        </button>
                    </div>
                    <div class="modal-body space-y-4">
                        <p class="proj-section__hint">
                            "Archive hides this asset from the default Assets list. Type DELETE to confirm."
                        </p>
                        <label class="folio-field__label">
                            "Type DELETE"
                            <input
                                class="folio-input"
                                type="text"
                                autocomplete="off"
                                prop:value=move || archive_confirm.get()
                                on:input=move |ev| archive_confirm.set(event_target_value(&ev))
                            />
                        </label>
                        {move || archive_err.get().map(|e| view! {
                            <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                        })}
                        {move || {
                            let blockers = archive_blockers.get();
                            if blockers.is_empty() {
                                return ().into_any();
                            }
                            view! {
                                <ul style="margin:0;padding-left:1.1rem;font-size:0.85rem;">
                                    {blockers.into_iter().map(|b| view! {
                                        <li>{b.message}</li>
                                    }).collect_view()}
                                </ul>
                            }.into_any()
                        }}
                    </div>
                    <div class="modal-footer">
                        <button
                            class="folio-btn folio-btn--ghost"
                            on:click=move |_| show_archive.set(false)
                        >
                            "Cancel"
                        </button>
                        <button
                            type="button"
                            class="folio-btn folio-btn--primary"
                            style="background:#b91c1c;border-color:#b91c1c;"
                            prop:disabled=move || {
                                archive_pending.get()
                                    || archive_confirm.get().trim() != "DELETE"
                            }
                            on:click=on_archive
                        >
                            {move || if archive_pending.get() { "Archiving…" } else { "Archive asset" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

// ── Default Contractor panel (Phase B) ────────────────────────────────────────

/// Sidebar panel for the asset's default contractor.
/// Provides inline set / change / remove via G-22 `POST|DELETE /api/folio/relationships`.
#[component]
fn DefaultContractorPanel(
    asset_id: String,
    contractor: Resource<Result<Option<AssetContractorSummary>, server_fn::error::ServerFnError>>,
    set_refresh: WriteSignal<u32>,
) -> impl IntoView {
    let (selector_open, set_selector_open) = signal(false);
    let (action_pending, set_action_pending) = signal(false);
    let (action_error, set_action_error) = signal::<Option<String>>(None);

    // Wrap non-Copy values in StoredValue so closures can borrow without moving.
    let aid_stored = StoredValue::new(asset_id);

    let vendors = Resource::new(
        move || selector_open.get(),
        |open| async move {
            if !open {
                return Ok(vec![]);
            }
            get_vendor_list().await
        },
    );

    view! {
        <div class="asset-detail-card">
            <div class="asset-section-header">
                <p class="asset-section-label">"Default Contractor"</p>
                <button
                    class="asset-section-action"
                    on:click=move |_| set_selector_open.update(|o| *o = !*o)
                    disabled=action_pending
                >
                    <span class="material-symbols-outlined" style="font-size:14px;">
                        {NavIcon::Handyman.as_str()}
                    </span>
                    {move || if selector_open.get() { "Cancel" } else { "Change" }}
                </button>
            </div>
            <p class="asset-contractor-hint">
                "Pre-filled when scheduling maintenance for this asset."
            </p>

            {move || action_error.get().map(|e| view! {
                <p class="asset-meta-empty" style="color:var(--folio-error);">{e}</p>
            })}

            // Contractor display — bare move|| reads contractor.get() for all states
            {move || {
                let c_val = contractor.get();
                match c_val {
                    None => view! {
                        <div class="asset-contractor-loading"/>
                    }.into_any(),
                    Some(Err(e)) => view! {
                        <p class="asset-meta-empty">{format!("Could not load contractor: {e}")}</p>
                    }.into_any(),
                    Some(Ok(None)) => view! {
                        <div class="asset-empty-inline">
                            <span class="material-symbols-outlined asset-empty-icon asset-empty-icon--sm">
                                {NavIcon::Handyman.as_str()}
                            </span>
                            <p class="asset-empty-text asset-empty-text--sm">"No default set."</p>
                        </div>
                        <button
                            class="asset-action-btn asset-action-btn--secondary"
                            style="width:100%; margin-top:.5rem;"
                            on:click=move |_| set_selector_open.set(true)
                            disabled=action_pending
                        >
                            <span class="material-symbols-outlined" style="font-size:16px;">
                                {NavIcon::Handyman.as_str()}
                            </span>
                            "Set Default Contractor"
                        </button>
                    }.into_any(),
                    Some(Ok(Some(c))) => {
                        let vid = c.vendor_id.to_string();
                        let name = c.business_name.clone();
                        let trade = c.primary_trade.clone();
                        let initial = name.chars().next().unwrap_or('?').to_uppercase().to_string();
                        let vid_rm = vid.clone();
                        let aid_rm = aid_stored.get_value();
                        view! {
                            <div class="asset-contractor-card">
                                <div class="asset-contractor-avatar">{initial}</div>
                                <div class="asset-contractor-info">
                                    <p class="asset-contractor-name">{name}</p>
                                    {trade.map(|t| view! {
                                        <span class="asset-type-pill" style="font-size:9px;">{t}</span>
                                    })}
                                </div>
                                <button
                                    class="asset-contractor-remove"
                                    title="Remove default contractor"
                                    on:click=move |_| {
                                        let v = vid_rm.clone();
                                        let a = aid_rm.clone();
                                        set_action_pending.set(true);
                                        set_action_error.set(None);
                                        leptos::task::spawn_local(async move {
                                            match remove_default_contractor(a, v).await {
                                                Ok(_) => set_refresh.update(|n| *n += 1),
                                                Err(e) => set_action_error.set(Some(e.to_string())),
                                            }
                                            set_action_pending.set(false);
                                        });
                                    }
                                    disabled=action_pending
                                >
                                    <span class="material-symbols-outlined" style="font-size:14px;">
                                        "close"
                                    </span>
                                </button>
                            </div>
                        }.into_any()
                    }
                }
            }}

            // Vendor selector
            {move || {
                if !selector_open.get() { return view! { <></> }.into_any(); }
                let vendors_val = vendors.get();
                let aid_sel = aid_stored.get_value();
                view! {
                    <div class="asset-vendor-selector">
                        <p class="asset-vendor-selector__label">"Select a contractor:"</p>
                        {match vendors_val {
                            None => view! {
                                <p class="asset-meta-empty">"Loading vendors\u{2026}"</p>
                            }.into_any(),
                            Some(Ok(vs)) if vs.is_empty() => view! {
                                <p class="asset-meta-empty">"No vendors found."</p>
                            }.into_any(),
                            Some(Ok(vs)) => vs.into_iter().map(|v| {
                                let vid = v.id.to_string();
                                let vname = v.business_name.clone();
                                let vtrade = v.primary_trade.clone();
                                let aid2 = aid_sel.clone();
                                view! {
                                    <button
                                        class="asset-vendor-option"
                                        on:click=move |_| {
                                            let v2 = vid.clone();
                                            let a2 = aid2.clone();
                                            set_action_pending.set(true);
                                            set_action_error.set(None);
                                            leptos::task::spawn_local(async move {
                                                match set_default_contractor(a2, v2).await {
                                                    Ok(_) => {
                                                        set_selector_open.set(false);
                                                        set_refresh.update(|n| *n += 1);
                                                    }
                                                    Err(e) => set_action_error.set(Some(e.to_string())),
                                                }
                                                set_action_pending.set(false);
                                            });
                                        }
                                        disabled=action_pending
                                    >
                                        <div class="asset-contractor-avatar" style="width:1.75rem;height:1.75rem;font-size:.75rem;">
                                            {vname.chars().next().unwrap_or('?').to_uppercase().to_string()}
                                        </div>
                                        <div>
                                            <p class="asset-contractor-name" style="font-size:.8rem;">{vname}</p>
                                            {vtrade.map(|t| view! {
                                                <span class="asset-type-pill" style="font-size:8px;">{t}</span>
                                            })}
                                        </div>
                                    </button>
                                }
                            }).collect_view().into_any(),
                            Some(Err(e)) => view! {
                                <p class="asset-meta-empty">{format!("Could not load vendors: {e}")}</p>
                            }.into_any(),
                        }}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ── Skeleton ──────────────────────────────────────────────────────────────────

#[component]
fn AssetDetailSkeleton() -> impl IntoView {
    view! {
        <div class="asset-detail-page asset-detail-page--loading">
            <div class="asset-skeleton asset-skeleton--breadcrumb"/>
            <div class="asset-detail-body">
                <div class="asset-detail-main">
                    <div class="asset-skeleton asset-skeleton--header"/>
                    <div class="asset-skeleton asset-skeleton--timeline"/>
                </div>
                <aside class="asset-detail-sidebar">
                    <div class="asset-skeleton asset-skeleton--card"/>
                    <div class="asset-skeleton asset-skeleton--card"/>
                    <div class="asset-skeleton asset-skeleton--card"/>
                </aside>
            </div>
        </div>
    }
}

// ── Error state ───────────────────────────────────────────────────────────────

#[component]
fn AssetDetailError(message: String) -> impl IntoView {
    view! {
        <div class="asset-detail-error">
            <span class="material-symbols-outlined asset-detail-error-icon">
                {NavIcon::Report.as_str()}
            </span>
            <h2 class="asset-detail-error-title">"Could not load asset"</h2>
            <p class="asset-detail-error-body">{message}</p>
            <a href=FolioRoute::LandlordAssets.path() class="asset-detail-back-link">
                <span class="material-symbols-outlined" style="font-size:16px;">
                    {NavIcon::ArrowBack.as_str()}
                </span>
                "Back to Assets"
            </a>
        </div>
    }
}

// ── Server functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(GetAssetDetail, "/api")]
pub async fn get_asset_detail(
    asset_id: String,
) -> Result<AssetDetailModel, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let _ = uuid::Uuid::parse_str(&asset_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Asset ID is not a valid UUID"))?;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<AssetDetailModel>(
        &format!("/api/folio/assets/{asset_id}"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Asset fetch failed: {e}")))
}

/// Fetches G-13 cases (maintenance + inspections) for this asset.
/// Calls GET /api/folio/assets/{id}/inspections (maintenance.rs handler).
#[server(GetAssetCases, "/api")]
pub async fn get_asset_cases(
    asset_id: String,
) -> Result<Vec<AssetCaseSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let _ = uuid::Uuid::parse_str(&asset_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Asset ID is not a valid UUID"))?;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<AssetCaseSummary>>(
        &format!("/api/folio/assets/{asset_id}/inspections"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Cases fetch failed: {e}")))
}

/// Fetches G-21 events for this asset.
/// Calls GET /api/folio/events?subject_entity_type=atlas_asset&subject_entity_id={id}
#[server(GetAssetEvents, "/api")]
pub async fn get_asset_events(
    asset_id: String,
) -> Result<Vec<AssetEventSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let _ = uuid::Uuid::parse_str(&asset_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Asset ID is not a valid UUID"))?;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<AssetEventSummary>>(
        &format!("/api/folio/events?subject_entity_type=atlas_asset&subject_entity_id={asset_id}"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Events fetch failed: {e}")))
}

/// Fetches the default contractor (G-22) for this asset.
#[server(GetAssetContractor, "/api")]
pub async fn get_asset_contractor(
    asset_id: String,
) -> Result<Option<AssetContractorSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let _ = uuid::Uuid::parse_str(&asset_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Asset ID is not a valid UUID"))?;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Option<AssetContractorSummary>>(
        &format!("/api/folio/assets/{asset_id}/contractor"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Contractor fetch failed: {e}")))
}

/// Fetches the vendor list for the contractor selector.
#[server(GetVendorList, "/api")]
pub async fn get_vendor_list() -> Result<Vec<VendorListItem>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    #[derive(serde::Deserialize)]
    struct RawVendor {
        id: uuid::Uuid,
        business_name: String,
        trade_type: Option<String>,
    }

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let raw = crate::atlas_client::authenticated_get::<Vec<RawVendor>>(
        "/api/folio/vendors",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Vendor list fetch failed: {e}")))?;
    Ok(raw
        .into_iter()
        .map(|v| VendorListItem {
            id: v.id,
            business_name: v.business_name,
            primary_trade: v.trade_type,
        })
        .collect())
}

/// Sets the default contractor for this asset via POST /api/folio/relationships.
/// Idempotent — the relationship service upserts.
#[server(SetDefaultContractor, "/api")]
pub async fn set_default_contractor(
    asset_id: String,
    vendor_id: String,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let _ = uuid::Uuid::parse_str(&asset_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid asset ID"))?;
    let _ = uuid::Uuid::parse_str(&vendor_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid vendor ID"))?;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let payload = serde_json::json!({
        "source_entity_type": "atlas_asset",
        "source_entity_id": asset_id,
        "target_entity_type": "atlas_service_providers",
        "target_entity_id": vendor_id,
        "relationship_type": "default_contractor"
    });

    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/folio/relationships",
        &token,
        None,
        &payload,
    )
    .await
    .map(|_| ())
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Set contractor failed: {e}")))
}

/// Removes the default contractor relationship for this asset.
#[server(RemoveDefaultContractor, "/api")]
pub async fn remove_default_contractor(
    asset_id: String,
    vendor_id: String,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let _ = uuid::Uuid::parse_str(&asset_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid asset ID"))?;
    let _ = uuid::Uuid::parse_str(&vendor_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid vendor ID"))?;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let _payload = serde_json::json!({
        "source_entity_type": "atlas_asset",
        "source_entity_id": asset_id,
        "target_entity_type": "atlas_service_providers",
        "target_entity_id": vendor_id,
        "relationship_type": "default_contractor"
    });

    // authenticated_delete takes no body; pass the relationship identifiers as query params.
    let path = format!(
        "/api/folio/relationships?source_entity_type=atlas_asset&source_entity_id={}&target_entity_type=atlas_service_providers&target_entity_id={}&relationship_type=default_contractor",
        asset_id, vendor_id
    );
    crate::atlas_client::authenticated_delete(&path, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Remove contractor failed: {e}")))
}
