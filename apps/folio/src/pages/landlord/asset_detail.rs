// apps/folio/src/pages/landlord/asset_detail.rs
//
// Asset Detail — lets the operator inspect a single property or unit:
// address, type, status, lifecycle dates, folio/serial number, attribute specs,
// event/inspection history (G-21), and assigned contractor (G-22).

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::components::nav::{FolioRoute, NavIcon};

// ── Response types (mirror backend shapes in handlers/folio/assets.rs) ────────

/// Mirrors `AssetDetail` returned by `GET /api/folio/assets/:id`.
///
/// The G-10 lifecycle fields (`scheduled_service_date`, `expiry_date`,
/// `condition`, `lifecycle_metadata`) were added in migration m20260900 and are
/// always present in the response \u2014 `Option` because older assets may lack them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetDetailModel {
    pub id:                      uuid::Uuid,
    pub tenant_id:               uuid::Uuid,
    pub portfolio_id:            Option<uuid::Uuid>,
    pub asset_type:              String,
    pub name:                    String,
    pub serial_or_folio_number:  Option<String>,
    pub status:                  String,
    pub address_line_1:          Option<String>,
    pub address_line_2:          Option<String>,
    pub city:                    Option<String>,
    pub state_province:          Option<String>,
    pub country_code:            Option<String>,
    pub postal_code:             Option<String>,
    /// Free-form JSON attributes (make, model, fuel type, etc.)
    pub attributes:              Option<serde_json::Value>,
    // G-10 lifecycle extension fields
    pub scheduled_service_date:  Option<chrono::NaiveDate>,
    pub expiry_date:             Option<chrono::NaiveDate>,
    pub condition:               Option<String>,
    pub lifecycle_metadata:      Option<serde_json::Value>,
    pub created_at:              chrono::DateTime<chrono::Utc>,
}

/// Mirrors `AssetEventSummary` from `GET /api/folio/assets/:id/events`.
/// Backed by G-21 `atlas_events` via `EventService::find_by_subject`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetEventSummary {
    pub id:         uuid::Uuid,
    pub name:       String,
    pub event_type: String,
    pub status:     String,
    pub starts_at:  chrono::DateTime<chrono::Utc>,
    pub ends_at:    chrono::DateTime<chrono::Utc>,
    pub venue_name: Option<String>,
}

/// Mirrors `AssetContractorSummary` from `GET /api/folio/assets/:id/contractor`.
/// Backed by G-22 `atlas_record_relationships` (relationship_type = "assigned_contractor")
/// + G-12 `atlas_service_providers` lookup.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetContractorSummary {
    pub vendor_id:         uuid::Uuid,
    pub business_name:     String,
    pub primary_trade:     Option<String>,
    pub relationship_type: String,
}

// ── Finite value sets as enums (Rule 1) ───────────────────────────────────────

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
            "active"   => Self::Active,
            "inactive" => Self::Inactive,
            "pending"  => Self::Pending,
            "archived" => Self::Archived,
            _          => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active   => "Active",
            Self::Inactive => "Inactive",
            Self::Pending  => "Pending",
            Self::Archived => "Archived",
            Self::Unknown  => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Active   => "asset-status-pill--active",
            Self::Inactive => "asset-status-pill--inactive",
            Self::Pending  => "asset-status-pill--pending",
            Self::Archived => "asset-status-pill--archived",
            Self::Unknown  => "asset-status-pill--unknown",
        }
    }
}

impl std::fmt::Display for AssetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Condition values defined in the G-10 entity comment:
/// "excellent" | "good" | "fair" | "poor" | "retired"
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
            "good"      => Self::Good,
            "fair"      => Self::Fair,
            "poor"      => Self::Poor,
            "retired"   => Self::Retired,
            _           => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::Good      => "Good",
            Self::Fair      => "Fair",
            Self::Poor      => "Poor",
            Self::Retired   => "Retired",
            Self::Unknown   => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Excellent => "asset-cond-pill--excellent",
            Self::Good      => "asset-cond-pill--good",
            Self::Fair      => "asset-cond-pill--fair",
            Self::Poor      => "asset-cond-pill--poor",
            Self::Retired   => "asset-cond-pill--retired",
            Self::Unknown   => "asset-cond-pill--unknown",
        }
    }
}

/// G-21 event status \u2014 must cover the full EventStatus state machine:
/// Draft \u2192 Published \u2192 Active \u2192 RegistrationClosed \u2192 InProgress \u2192 Completed | Cancelled
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetEventStatus {
    Draft,
    Published,
    Active,
    RegistrationClosed,
    InProgress,
    Completed,
    Cancelled,
    Unknown,
}

impl AssetEventStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "draft"               => Self::Draft,
            "published"           => Self::Published,
            "active"              => Self::Active,
            "registration_closed" => Self::RegistrationClosed,
            "in_progress"         => Self::InProgress,
            "completed"           => Self::Completed,
            "cancelled"           => Self::Cancelled,
            _                     => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Draft               => "Draft",
            Self::Published           => "Published",
            Self::Active              => "Active",
            Self::RegistrationClosed  => "Registration Closed",
            Self::InProgress          => "In Progress",
            Self::Completed           => "Completed",
            Self::Cancelled           => "Cancelled",
            Self::Unknown             => "Unknown",
        }
    }

    pub const fn timeline_class(self) -> &'static str {
        match self {
            Self::Completed           => "asset-tl-icon--complete",
            Self::Cancelled           => "asset-tl-icon--cancelled",
            Self::Draft
            | Self::Published
            | Self::Active
            | Self::RegistrationClosed
            | Self::InProgress
            | Self::Unknown           => "asset-tl-icon--scheduled",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Completed           => "asset-event-pill--complete",
            Self::Cancelled           => "asset-event-pill--cancelled",
            Self::Active
            | Self::InProgress        => "asset-event-pill--active",
            Self::Draft
            | Self::Published
            | Self::RegistrationClosed
            | Self::Unknown           => "asset-event-pill--scheduled",
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

    // Events and contractor are independent resources so they load in parallel
    // and don't block each other or the main asset fetch.
    let events = Resource::new(asset_id.clone(), |id| async move {
        if id.is_empty() { return Ok(vec![]); }
        get_asset_events(id).await
    });

    let contractor = Resource::new(asset_id, |id| async move {
        if id.is_empty() { return Ok(None); }
        get_asset_contractor(id).await
    });

    view! {
        <div class="asset-detail-page">
            <Suspense fallback=|| view! { <AssetDetailSkeleton/> }>
                {move || asset.get().map(|result| match result {
                    Ok(detail) => view! {
                        <AssetDetailContent
                            detail=detail
                            events=events
                            contractor=contractor
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
    events: Resource<Result<Vec<AssetEventSummary>, server_fn::error::ServerFnError>>,
    contractor: Resource<Result<Option<AssetContractorSummary>, server_fn::error::ServerFnError>>,
) -> impl IntoView {
    let status    = AssetStatus::from_str(&detail.status);
    let condition = detail.condition.as_deref().map(AssetCondition::from_str);

    let address_display = {
        let parts: Vec<&str> = [
            detail.address_line_1.as_deref(),
            detail.city.as_deref(),
            detail.state_province.as_deref(),
            detail.country_code.as_deref(),
        ]
        .into_iter().flatten().collect();
        if parts.is_empty() { None } else { Some(parts.join(", ")) }
    };

    let attribute_pairs: Vec<(String, String)> = detail
        .attributes.as_ref()
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter().map(|(k, v)| {
                let display_key = k.replace('_', " ");
                let display_val = match v {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b)   => if *b { "Yes".to_string() } else { "No".to_string() },
                    _                            => v.to_string(),
                };
                (display_key, display_val)
            }).collect()
        })
        .unwrap_or_default();

    let asset_name   = detail.name.clone();
    let asset_type   = detail.asset_type.clone();
    let folio_number = detail.serial_or_folio_number.clone();

    // Lifecycle date display helpers (NaiveDate \u2192 "Jun 18, 2026")
    let svc_date  = detail.scheduled_service_date.map(|d| d.format("%b %-d, %Y").to_string());
    let exp_date  = detail.expiry_date.map(|d| d.format("%b %-d, %Y").to_string());
    let created   = detail.created_at.format("%b %-d, %Y").to_string();

    // Has any lifecycle data to show?
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

                    // Header card
                    <div class="asset-detail-card asset-header-card">
                        <div class="asset-type-icon-wrap">
                            <span class="material-symbols-outlined asset-type-icon">
                                {NavIcon::Apartment.as_str()}
                            </span>
                        </div>
                        <div class="asset-header-info">
                            <h1 class="asset-title">{asset_name.clone()}</h1>
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
                            {address_display.map(|addr| view! {
                                <p class="asset-header-address">
                                    <span class="material-symbols-outlined asset-header-address-icon">
                                        {NavIcon::Map.as_str()}
                                    </span>
                                    {addr}
                                </p>
                            })}
                        </div>
                    </div>

                    // Lifecycle status grid (G-10 fields: scheduled_service_date, expiry_date, condition)
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

                    // Event history — backed by G-21 atlas_events
                    <div class="asset-detail-card">
                        <div class="asset-section-header">
                            <p class="asset-section-label">"Event History"</p>
                            <a href=FolioRoute::LandlordMaintenance.path() class="asset-section-action">
                                <span class="material-symbols-outlined" style="font-size:14px;">
                                    {NavIcon::Build.as_str()}
                                </span>
                                "Maintenance"
                            </a>
                        </div>
                        <Suspense fallback=|| view! { <div class="asset-timeline-loading"/> }>
                            {move || events.get().map(|result| match result {
                                Ok(evs) if evs.is_empty() => view! {
                                    <div class="asset-empty-timeline">
                                        <span class="material-symbols-outlined asset-empty-icon">
                                            {NavIcon::EventAvailable.as_str()}
                                        </span>
                                        <p class="asset-empty-text">"No events recorded yet."</p>
                                        <p class="asset-empty-sub">
                                            "Inspection and maintenance events will appear here. "
                                            "Create one via "
                                            <a href=FolioRoute::LandlordMaintenance.path() class="asset-inline-link">
                                                "Maintenance"
                                            </a>
                                            " or by posting to "
                                            <code class="asset-inline-code">
                                                "POST /api/folio/events"
                                            </code>
                                            " with "
                                            <code class="asset-inline-code">
                                                {format!("subject_entity_type=\"atlas_asset\"")}
                                            </code>
                                            "."
                                        </p>
                                    </div>
                                }.into_any(),
                                Ok(evs) => view! {
                                    <div class="asset-timeline">
                                        {evs.into_iter().map(|ev| {
                                            let ev_status = AssetEventStatus::from_str(&ev.status);
                                            let starts = ev.starts_at.format("%b %-d, %Y \u{00b7} %H:%M").to_string();
                                            view! {
                                                <div class="asset-timeline-item">
                                                    <div class={format!("asset-tl-icon {}", ev_status.timeline_class())}>
                                                        <span class="material-symbols-outlined" style="font-size:14px;">
                                                            {if ev_status == AssetEventStatus::Completed {
                                                                "check_circle"
                                                            } else if ev_status == AssetEventStatus::Cancelled {
                                                                "cancel"
                                                            } else {
                                                                "schedule"
                                                            }}
                                                        </span>
                                                    </div>
                                                    <div class="asset-tl-body">
                                                        <div class="asset-tl-header">
                                                            <div>
                                                                <p class="asset-tl-name">{ev.name}</p>
                                                                <p class="asset-tl-meta">
                                                                    {starts}
                                                                    {ev.venue_name.map(|vn| format!(" \u{00b7} {vn}"))}
                                                                </p>
                                                            </div>
                                                            <span class={format!(
                                                                "asset-event-pill {}",
                                                                ev_status.pill_class()
                                                            )}>
                                                                {ev_status.as_str()}
                                                            </span>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any(),
                                Err(e) => view! {
                                    <p class="asset-meta-empty">{format!("Could not load events: {e}")}</p>
                                }.into_any(),
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
                            <a
                                href=FolioRoute::LandlordMaintenance.path()
                                class="asset-action-btn asset-action-btn--primary"
                            >
                                <span class="material-symbols-outlined" style="font-size:18px;">
                                    {NavIcon::Build.as_str()}
                                </span>
                                "Create Work Order"
                            </a>
                            <a
                                href=FolioRoute::LandlordLeases.path()
                                class="asset-action-btn asset-action-btn--secondary"
                            >
                                <span class="material-symbols-outlined" style="font-size:18px;">
                                    {NavIcon::Description.as_str()}
                                </span>
                                "View Leases"
                            </a>
                        </div>
                    </div>

                    // Assigned contractor — backed by G-22 atlas_record_relationships
                    <div class="asset-detail-card">
                        <p class="asset-section-label">"Assigned Contractor"</p>
                        <Suspense fallback=|| view! { <div class="asset-contractor-loading"/> }>
                            {move || contractor.get().map(|result| match result {
                                Ok(Some(c)) => view! {
                                    <div class="asset-contractor-card">
                                        <div class="asset-contractor-avatar">
                                            {c.business_name.chars().next().unwrap_or('?').to_uppercase().to_string()}
                                        </div>
                                        <div class="asset-contractor-info">
                                            <p class="asset-contractor-name">{c.business_name}</p>
                                            {c.primary_trade.map(|t| view! {
                                                <span class="asset-type-pill" style="font-size:9px;">{t}</span>
                                            })}
                                        </div>
                                    </div>
                                    <a
                                        href=FolioRoute::LandlordVendors.path()
                                        class="asset-vendor-link"
                                    >
                                        <span class="material-symbols-outlined" style="font-size:14px;">
                                            {NavIcon::Handyman.as_str()}
                                        </span>
                                        "View in Contractor Network"
                                    </a>
                                }.into_any(),
                                Ok(None) => view! {
                                    <div class="asset-empty-inline">
                                        <span class="material-symbols-outlined asset-empty-icon asset-empty-icon--sm">
                                            {NavIcon::Handyman.as_str()}
                                        </span>
                                        <p class="asset-empty-text asset-empty-text--sm">
                                            "No contractor assigned."
                                        </p>
                                    </div>
                                    <p class="asset-meta-empty" style="text-align:left; padding-top:.5rem;">
                                        "To assign, create a record relationship with "
                                        <code class="asset-inline-code">
                                            "relationship_type=\"assigned_contractor\""
                                        </code>
                                        " via "
                                        <code class="asset-inline-code">
                                            "POST /api/folio/relationships"
                                        </code>
                                        "."
                                    </p>
                                    <a
                                        href=FolioRoute::LandlordVendors.path()
                                        class="asset-vendor-link"
                                    >
                                        <span class="material-symbols-outlined" style="font-size:14px;">
                                            {NavIcon::Handyman.as_str()}
                                        </span>
                                        "Browse Contractor Network"
                                    </a>
                                }.into_any(),
                                Err(e) => view! {
                                    <p class="asset-meta-empty">
                                        {format!("Could not load contractor: {e}")}
                                    </p>
                                }.into_any(),
                            })}
                        </Suspense>
                    </div>

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
                                <p class="asset-meta-empty">
                                    "No additional specifications recorded."
                                </p>
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

                </aside>
            </div>
        </div>
    }
}

// ── Skeleton ─────────────────────────────────────────────────────────────────

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

fn extract_token_from_headers(
    headers: &axum::http::HeaderMap,
) -> Option<String> {
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

#[server(GetAssetDetail, "/api")]
pub async fn get_asset_detail(
    asset_id: String,
) -> Result<AssetDetailModel, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    if asset_id.len() > 36 || asset_id.is_empty() {
        return Err(server_fn::error::ServerFnError::new("Invalid asset ID format"));
    }
    let _ = uuid::Uuid::parse_str(&asset_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Asset ID is not a valid UUID"))?;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let path = format!("/api/folio/assets/{asset_id}");
    crate::atlas_client::authenticated_get::<AssetDetailModel>(&path, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Asset fetch failed: {e}")))
}

#[server(GetAssetEvents, "/api")]
pub async fn get_asset_events(
    asset_id: String,
) -> Result<Vec<AssetEventSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    if asset_id.len() > 36 || asset_id.is_empty() {
        return Err(server_fn::error::ServerFnError::new("Invalid asset ID format"));
    }
    let _ = uuid::Uuid::parse_str(&asset_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Asset ID is not a valid UUID"))?;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let path = format!("/api/folio/assets/{asset_id}/events");
    crate::atlas_client::authenticated_get::<Vec<AssetEventSummary>>(&path, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Events fetch failed: {e}")))
}

#[server(GetAssetContractor, "/api")]
pub async fn get_asset_contractor(
    asset_id: String,
) -> Result<Option<AssetContractorSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    if asset_id.len() > 36 || asset_id.is_empty() {
        return Err(server_fn::error::ServerFnError::new("Invalid asset ID format"));
    }
    let _ = uuid::Uuid::parse_str(&asset_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Asset ID is not a valid UUID"))?;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let path = format!("/api/folio/assets/{asset_id}/contractor");
    // Backend returns `null` (JSON null) when no contractor is assigned.
    // reqwest deserializes JSON null as `None` for `Option<T>`.
    crate::atlas_client::authenticated_get::<Option<AssetContractorSummary>>(&path, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Contractor fetch failed: {e}")))
}
