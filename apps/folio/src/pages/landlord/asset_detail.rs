// apps/folio/src/pages/landlord/asset_detail.rs
//
// Asset Detail — lets the operator inspect a single property or unit:
// its address, type, status, folio number, and JSON attributes (specs).
// Event history and assigned contractor panels are stubbed pending
// dedicated backend endpoints (see TODO comments below).

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::components::nav::{FolioRoute, NavIcon};

// ── Response type (mirrors AssetDetail in backend/src/handlers/folio/assets.rs) ──

/// Mirrors `AssetDetail` returned by `GET /api/folio/assets/:id`.
///
/// All monetary and measurement values that arrive as strings from the
/// `attributes` JSON blob are treated as display-only strings — never
/// parsed as f64 (Rule 24: no float arithmetic for domain values).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetDetailModel {
    pub id:                     uuid::Uuid,
    pub tenant_id:              uuid::Uuid,
    pub portfolio_id:           Option<uuid::Uuid>,
    pub asset_type:             String,
    pub name:                   String,
    pub serial_or_folio_number: Option<String>,
    pub status:                 String,
    pub address_line_1:         Option<String>,
    pub address_line_2:         Option<String>,
    pub city:                   Option<String>,
    pub state_province:         Option<String>,
    pub country_code:           Option<String>,
    pub postal_code:            Option<String>,
    /// Free-form JSON attributes (make, model, fuel type, etc.)
    pub attributes:             Option<serde_json::Value>,
    pub created_at:             chrono::DateTime<chrono::Utc>,
}

// ── Asset status — finite set, must be an enum (Rule 1) ──────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetStatus {
    Active,
    Inactive,
    Pending,
    Archived,
    Unknown,
}

impl AssetStatus {
    /// Parse from the `status` string returned by the backend.
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

    /// CSS class suffix used by the status pill.
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

// ── Page component ────────────────────────────────────────────────────────────

/// Enables the landlord to inspect a single property or unit in full detail:
/// address, type, operational status, folio/serial number, and attribute specs.
#[component]
pub fn AssetDetail() -> impl IntoView {
    let params = use_params_map();

    // Extract `:id` from the URL — use_params_map() returns a reactive signal.
    let asset_id = move || {
        params.get().get("id").unwrap_or_default().to_string()
    };

    let asset = Resource::new(asset_id, |id| async move {
        if id.is_empty() {
            return Err(server_fn::error::ServerFnError::new("No asset ID in URL"));
        }
        get_asset_detail(id).await
    });

    view! {
        <div class="asset-detail-page">
            <Suspense fallback=|| view! { <AssetDetailSkeleton/> }>
                {move || asset.get().map(|result| match result {
                    Ok(detail) => view! { <AssetDetailContent detail=detail/> }.into_any(),
                    Err(e)     => view! { <AssetDetailError message=e.to_string()/> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

// ── Content component — receives the loaded AssetDetailModel ─────────────────

#[component]
fn AssetDetailContent(detail: AssetDetailModel) -> impl IntoView {
    let status = AssetStatus::from_str(&detail.status);

    // Build a one-line address string for the header subtitle.
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

    // `attributes` is a serde_json::Value blob — extract key/value pairs for
    // display. We render them as strings; no numeric parsing (Rule 24).
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
                        serde_json::Value::Bool(b)   => if *b { "Yes".to_string() } else { "No".to_string() },
                        _                            => v.to_string(),
                    };
                    (display_key, display_val)
                })
                .collect()
        })
        .unwrap_or_default();

    let asset_name   = detail.name.clone();
    let asset_type   = detail.asset_type.clone();
    let folio_number = detail.serial_or_folio_number.clone();

    view! {
        <div class="asset-detail-layout">

            // ── Breadcrumb (desktop) ──────────────────────────────────────────
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

            // ── Two-column body ───────────────────────────────────────────────
            <div class="asset-detail-body">

                // ── LEFT COLUMN ───────────────────────────────────────────────
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

                    // Event history — TODO: needs GET /api/folio/assets/:id/events
                    // Backend endpoint not yet available; renders empty state.
                    <div class="asset-detail-card">
                        <div class="asset-section-header">
                            <p class="asset-section-label">"Event History"</p>
                            <a
                                href=FolioRoute::LandlordMaintenance.path()
                                class="asset-section-action"
                            >
                                <span class="material-symbols-outlined" style="font-size:14px;">
                                    {NavIcon::Build.as_str()}
                                </span>
                                "Maintenance"
                            </a>
                        </div>
                        // TODO: needs backend endpoint GET /api/folio/assets/:id/events
                        <div class="asset-empty-timeline">
                            <span class="material-symbols-outlined asset-empty-icon">
                                {NavIcon::EventAvailable.as_str()}
                            </span>
                            <p class="asset-empty-text">"No event history available yet."</p>
                            <p class="asset-empty-sub">"Inspection records will appear here once the events endpoint is wired."</p>
                        </div>
                    </div>

                </div>

                // ── RIGHT COLUMN ──────────────────────────────────────────────
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

                    // Specifications (from attributes JSON)
                    <div class="asset-detail-card">
                        <p class="asset-section-label">"Specifications"</p>

                        // Folio / serial number if present
                        {folio_number.map(|n| view! {
                            <div class="asset-meta-row">
                                <p class="asset-meta-key">"Folio / Serial"</p>
                                <p class="asset-meta-val asset-meta-mono">{n}</p>
                            </div>
                        })}

                        // Created date
                        <div class="asset-meta-row">
                            <p class="asset-meta-key">"Registered"</p>
                            <p class="asset-meta-val">
                                {detail.created_at.format("%b %-d, %Y").to_string()}
                            </p>
                        </div>

                        // Dynamic attribute pairs from the JSON blob
                        {if attribute_pairs.is_empty() {
                            view! {
                                <p class="asset-meta-empty">
                                    "No additional specifications recorded."
                                </p>
                            }.into_any()
                        } else {
                            attribute_pairs
                                .into_iter()
                                .map(|(key, val)| view! {
                                    <div class="asset-meta-row">
                                        <p class="asset-meta-key">{key}</p>
                                        <p class="asset-meta-val">{val}</p>
                                    </div>
                                })
                                .collect_view()
                                .into_any()
                        }}
                    </div>

                    // Assigned contractor — TODO: needs GET /api/folio/assets/:id/contractor
                    <div class="asset-detail-card">
                        <p class="asset-section-label">"Assigned Contractor"</p>
                        // TODO: needs backend endpoint GET /api/folio/assets/:id/contractor
                        <div class="asset-empty-inline">
                            <span class="material-symbols-outlined asset-empty-icon asset-empty-icon--sm">
                                {NavIcon::Handyman.as_str()}
                            </span>
                            <p class="asset-empty-text asset-empty-text--sm">"No contractor assigned."</p>
                        </div>
                        <a
                            href=FolioRoute::LandlordVendors.path()
                            class="asset-vendor-link"
                        >
                            <span class="material-symbols-outlined" style="font-size:14px;">
                                {NavIcon::Handyman.as_str()}
                            </span>
                            "Browse Contractor Network"
                        </a>
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

// ── Server function ───────────────────────────────────────────────────────────

#[server(GetAssetDetail, "/api")]
pub async fn get_asset_detail(asset_id: String) -> Result<AssetDetailModel, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    // Input validation — asset_id must be a valid UUID string (Rule 21).
    if asset_id.len() > 36 || asset_id.is_empty() {
        return Err(server_fn::error::ServerFnError::new("Invalid asset ID format"));
    }
    // Verify it parses as UUID before making the network call.
    let _ = uuid::Uuid::parse_str(&asset_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Asset ID is not a valid UUID"))?;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = {
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
            .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?
    };

    let path = format!("/api/folio/assets/{asset_id}");
    crate::atlas_client::authenticated_get::<AssetDetailModel>(&path, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Asset fetch failed: {e}")))
}
