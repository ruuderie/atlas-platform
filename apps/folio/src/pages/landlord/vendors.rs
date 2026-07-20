// apps/folio/src/pages/landlord/vendors.rs
//
// Vendor / Contractor Network page.
//
// Lists all atlas_service_providers for the tenant via GET /api/folio/vendors.
// Each vendor row has a "Set as Default for Asset" action that:
//   1. Opens an inline asset picker (GET /api/folio/assets for the list).
//   2. On select, fires POST /api/folio/relationships with
//      relationship_type = "default_contractor".
//
// The same relationship can be set from the Asset Detail sidebar (Phase B).
// Both paths write the same G-22 record — there is no dual source of truth.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::NavIcon;
use crate::components::page_header::PageHeader;

// ── Response types (mirror backend shapes) ────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VendorSummary {
    pub id: uuid::Uuid,
    pub business_name: String,
    pub trade_type: Option<String>,
    pub status: String,
    pub is_emergency_available: bool,
    pub rating_avg: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Minimal asset list item for the asset picker.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetPickerItem {
    pub id: uuid::Uuid,
    pub name: String,
    pub asset_type: String,
    pub status: String,
    #[serde(default)]
    pub address_line_1: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub state_province: Option<String>,
}

impl AssetPickerItem {
    /// Street · name label for picker rows and `<option>`s.
    pub fn place_label(&self) -> String {
        crate::utils::format_asset_place_label(
            &self.name,
            self.address_line_1.as_deref(),
            self.city.as_deref(),
            self.state_province.as_deref(),
        )
    }
}

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Vendor status — mirrors atlas_service_providers.status values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VendorStatus {
    Active,
    Inactive,
    Suspended,
    Pending,
    Unknown,
}

impl VendorStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "inactive" => Self::Inactive,
            "suspended" => Self::Suspended,
            "pending" => Self::Pending,
            _ => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Inactive => "Inactive",
            Self::Suspended => "Suspended",
            Self::Pending => "Pending",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Active => "vendor-status-pill--active",
            Self::Inactive => "vendor-status-pill--inactive",
            Self::Suspended => "vendor-status-pill--suspended",
            Self::Pending => "vendor-status-pill--pending",
            Self::Unknown => "vendor-status-pill--unknown",
        }
    }
}

/// Trade type for create — mirrors backend `TradeType` (snake_case wire values).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VendorTradeType {
    Plumber,
    Electrician,
    Hvac,
    GeneralContractor,
    Roofer,
    Painter,
    Landscaper,
    Cleaner,
    Inspector,
    General,
}

impl VendorTradeType {
    pub const ALL: &'static [Self] = &[
        Self::Plumber,
        Self::Electrician,
        Self::Hvac,
        Self::GeneralContractor,
        Self::Roofer,
        Self::Painter,
        Self::Landscaper,
        Self::Cleaner,
        Self::Inspector,
        Self::General,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Plumber => "plumber",
            Self::Electrician => "electrician",
            Self::Hvac => "hvac",
            Self::GeneralContractor => "general_contractor",
            Self::Roofer => "roofer",
            Self::Painter => "painter",
            Self::Landscaper => "landscaper",
            Self::Cleaner => "cleaner",
            Self::Inspector => "inspector",
            Self::General => "general",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Plumber => "Plumbing",
            Self::Electrician => "Electrical",
            Self::Hvac => "HVAC",
            Self::GeneralContractor => "General Contractor",
            Self::Roofer => "Roofing",
            Self::Painter => "Painting",
            Self::Landscaper => "Landscaping",
            Self::Cleaner => "Cleaning",
            Self::Inspector => "Inspection",
            Self::General => "General",
        }
    }
}

/// Trade categories used for filtering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeFilter {
    All,
    Plumber,
    Electrician,
    Hvac,
    Roofer,
    Painter,
    Landscaper,
    Cleaner,
    Inspector,
    GeneralContractor,
    General,
}

impl TradeFilter {
    pub const ALL: &'static [Self] = &[
        Self::All,
        Self::Plumber,
        Self::Electrician,
        Self::Hvac,
        Self::Roofer,
        Self::Painter,
        Self::Landscaper,
        Self::Cleaner,
        Self::Inspector,
        Self::GeneralContractor,
        Self::General,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Plumber => "plumber",
            Self::Electrician => "electrician",
            Self::Hvac => "hvac",
            Self::Roofer => "roofer",
            Self::Painter => "painter",
            Self::Landscaper => "landscaper",
            Self::Cleaner => "cleaner",
            Self::Inspector => "inspector",
            Self::GeneralContractor => "general_contractor",
            Self::General => "general",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "All Trades",
            Self::Plumber => "Plumbing",
            Self::Electrician => "Electrical",
            Self::Hvac => "HVAC",
            Self::Roofer => "Roofing",
            Self::Painter => "Painting",
            Self::Landscaper => "Landscaping",
            Self::Cleaner => "Cleaning",
            Self::Inspector => "Inspection",
            Self::GeneralContractor => "General Contractor",
            Self::General => "General",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::All => "handyman",
            Self::Plumber => "plumbing",
            Self::Electrician => "electric_bolt",
            Self::Hvac => "thermostat",
            Self::Roofer => "roofing",
            Self::Painter => "format_paint",
            Self::Landscaper => "grass",
            Self::Cleaner => "cleaning_services",
            Self::Inspector => "fact_check",
            Self::GeneralContractor | Self::General => "build",
        }
    }
}

// ── Page component ────────────────────────────────────────────────────────────

#[component]
pub fn Vendors() -> impl IntoView {
    let (trade_filter, set_trade_filter) = signal(TradeFilter::All);
    let (search_query, set_search_query) = signal(String::new());
    let refresh = RwSignal::new(0u32);
    let show_add = RwSignal::new(false);
    let create_err = RwSignal::new(None::<String>);

    let vendors = Resource::new(move || refresh.get(), |_| async move { list_vendors().await });

    let title = Signal::derive(|| "Contractor Network".to_string());
    let subtitle = Signal::derive(|| {
        "Manage service providers. Set a vendor as default for an asset to \
         pre-fill dispatch when creating work orders."
            .to_string()
    });

    view! {
        <div class="vendors-page">
            <PageHeader title=title subtitle=subtitle>
                <button
                    type="button"
                    class="vendors-add-btn"
                    on:click=move |_| {
                        create_err.set(None);
                        show_add.set(true);
                    }
                >
                    <span class="material-symbols-outlined" style="font-size:18px;">
                        {NavIcon::Build.as_str()}
                    </span>
                    "Add vendor"
                </button>
            </PageHeader>

            // ── Filter bar ────────────────────────────────────────────────
            <div class="vendors-filter-bar">
                // Search
                <div class="vendors-search-wrap">
                    <span class="material-symbols-outlined vendors-search-icon">
                        "search"
                    </span>
                    <input
                        id="vendor-search"
                        class="vendors-search-input"
                        type="search"
                        placeholder="Search by name or trade…"
                        on:input=move |e| set_search_query.set(event_target_value(&e))
                    />
                </div>

                // Trade chips
                <div class="vendors-trade-chips">
                    {TradeFilter::ALL.iter().copied().map(|tf| {
                        view! {
                            <button
                                class=move || {
                                    if trade_filter.get() == tf {
                                        "vendor-trade-chip vendor-trade-chip--active"
                                    } else {
                                        "vendor-trade-chip"
                                    }
                                }
                                on:click=move |_| set_trade_filter.set(tf)
                            >
                                <span class="material-symbols-outlined" style="font-size:13px;">
                                    {tf.material_icon()}
                                </span>
                                {tf.label()}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            // ── Vendor grid ───────────────────────────────────────────────
            <Suspense fallback=|| view! { <VendorGridSkeleton/> }>
                {move || vendors.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="vendors-error">
                            <span class="material-symbols-outlined" style="font-size:2.5rem;color:var(--folio-muted);">
                                {NavIcon::Report.as_str()}
                            </span>
                            <p class="vendors-error-text">{format!("Could not load vendors: {e}")}</p>
                        </div>
                    }.into_any(),
                    Ok(all) => {
                        let q  = search_query.get().to_lowercase();
                        let tf = trade_filter.get();
                        let filtered: Vec<VendorSummary> = all.into_iter().filter(|v| {
                            let trade_ok = tf == TradeFilter::All || v.trade_type
                                .as_deref()
                                .map(|t| t == tf.as_str())
                                .unwrap_or(false);
                            let search_ok = q.is_empty()
                                || v.business_name.to_lowercase().contains(&q)
                                || v.trade_type.as_deref().unwrap_or("").to_lowercase().contains(&q);
                            trade_ok && search_ok
                        }).collect();

                        if filtered.is_empty() {
                            view! {
                                <div class="vendors-empty">
                                    <span class="material-symbols-outlined vendors-empty-icon">
                                        {NavIcon::Handyman.as_str()}
                                    </span>
                                    <p class="vendors-empty-title">"No vendors found"</p>
                                    <p class="vendors-empty-sub">
                                        "Try a different trade filter or search term, or add a \
                                         new contractor to your network."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="vendors-grid">
                                    {filtered.into_iter().map(|v| view! {
                                        <VendorCard vendor=v/>
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>

            <Show when=move || show_add.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Add vendor"</h3>
                            <button
                                type="button"
                                class="modal-close"
                                on:click=move |_| show_add.set(false)
                            >
                                "✕"
                            </button>
                        </div>
                        <div class="modal-body space-y-4">
                            <p class="folio-empty__sub" style="margin:0;">
                                "Vendors must already have an Atlas account. Invite-by-email is not available in this release — we won’t ask you to paste IDs."
                            </p>
                            {move || create_err.get().map(|e| view! {
                                <p class="vendor-assign-msg vendor-assign-msg--error">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button
                                type="button"
                                class="folio-btn folio-btn--ghost"
                                on:click=move |_| show_add.set(false)
                            >
                                "Close"
                            </button>
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary"
                                disabled=true
                                title="Vendor invite is not available yet"
                            >
                                "Not available"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

// ── Vendor card ───────────────────────────────────────────────────────────────

#[component]
fn VendorCard(vendor: VendorSummary) -> impl IntoView {
    let (picker_open, set_picker_open) = signal(false);
    let (assign_pending, set_assign_pending) = signal(false);
    let (assign_error, set_assign_error) = signal::<Option<String>>(None);
    let (assign_success, set_assign_success) = signal::<Option<String>>(None);

    let status = VendorStatus::from_str(&vendor.status);
    let initial = vendor
        .business_name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    let name = vendor.business_name.clone();
    let trade = vendor.trade_type.clone();
    let vendor_id_stored = StoredValue::new(vendor.id.to_string());

    let rating_stars = vendor.rating_avg.map(|r| {
        let full = r.floor() as usize;
        let frac = r - r.floor();
        let half = frac >= 0.25 && frac < 0.75;
        let empty = 5_usize.saturating_sub(full + if half { 1 } else { 0 });
        (full, half, empty, r)
    });

    let assets = Resource::new(
        move || picker_open.get(),
        |open| async move {
            if !open {
                return Ok(vec![]);
            }
            list_assets_for_picker().await
        },
    );

    view! {
        <div class="vendor-card">
            // ── Card header ──────────────────────────────────────────────
            <div class="vendor-card-header">
                <div class="vendor-avatar">{initial.clone()}</div>
                <div class="vendor-card-identity">
                    <h2 class="vendor-card-name">{name.clone()}</h2>
                    <div class="vendor-card-pills">
                        {trade.map(|t| view! {
                            <span class="vendor-trade-pill">
                                <span class="material-symbols-outlined" style="font-size:11px;">
                                    {TradeFilter::ALL.iter().copied()
                                        .find(|f| f.as_str() == t)
                                        .unwrap_or(TradeFilter::General)
                                        .material_icon()}
                                </span>
                                {t.replace('_', " ")}
                            </span>
                        })}
                        <span class={format!("vendor-status-pill {}", status.pill_class())}>
                            {status.as_str()}
                        </span>
                        {vendor.is_emergency_available.then(|| view! {
                            <span class="vendor-emergency-pill">
                                <span class="material-symbols-outlined" style="font-size:11px;">
                                    "emergency"
                                </span>
                                "Emergency"
                            </span>
                        })}
                    </div>
                </div>
            </div>

            // ── Rating ───────────────────────────────────────────────────
            {rating_stars.map(|(full, half, empty, avg)| view! {
                <div class="vendor-rating">
                    {(0..full).map(|_| view! {
                        <span class="material-symbols-outlined vendor-star vendor-star--full">
                            "star"
                        </span>
                    }).collect_view()}
                    {half.then(|| view! {
                        <span class="material-symbols-outlined vendor-star vendor-star--half">
                            "star_half"
                        </span>
                    })}
                    {(0..empty).map(|_| view! {
                        <span class="material-symbols-outlined vendor-star vendor-star--empty">
                            "star_border"
                        </span>
                    }).collect_view()}
                    <span class="vendor-rating-value">{format!("{avg:.1}")}</span>
                </div>
            })}

            // ── Actions ──────────────────────────────────────────────────
            <div class="vendor-card-actions">
                <button
                    class="vendor-action-btn vendor-action-btn--primary"
                    on:click=move |_| set_picker_open.update(|o| *o = !*o)
                    disabled=assign_pending
                >
                    <span class="material-symbols-outlined" style="font-size:16px;">
                        {NavIcon::Apartment.as_str()}
                    </span>
                    {move || if picker_open.get() { "Cancel" } else { "Set as Default for Asset" }}
                </button>
            </div>

            // ── Status messages ───────────────────────────────────────────
            {move || assign_error.get().map(|e| view! {
                <p class="vendor-assign-msg vendor-assign-msg--error">{e}</p>
            })}
            {move || assign_success.get().map(|s| view! {
                <p class="vendor-assign-msg vendor-assign-msg--success">
                    <span class="material-symbols-outlined" style="font-size:13px;">"check_circle"</span>
                    {s}
                </p>
            })}

            // ── Asset picker ──────────────────────────────────────────────
            {move || {
                if !picker_open.get() { return view! { <></> }.into_any(); }
                let assets_val = assets.get();
                let vid = vendor_id_stored.get_value();
                view! {
                    <div class="vendor-asset-picker">
                        <p class="vendor-asset-picker__label">
                            "Choose an asset — this vendor will be the default contractor for it:"
                        </p>
                        {match assets_val {
                            None => view! {
                                <p class="vendor-picker-loading">"Loading assets\u{2026}"</p>
                            }.into_any(),
                            Some(Err(e)) => view! {
                                <p class="vendor-picker-loading">{format!("Could not load assets: {e}")}</p>
                            }.into_any(),
                            Some(Ok(list)) if list.is_empty() => view! {
                                <p class="vendor-picker-loading">"No assets found for this portfolio."</p>
                            }.into_any(),
                            Some(Ok(list)) => list.into_iter().map(|asset| {
                                let asset_id  = asset.id.to_string();
                                let asset_name = asset.name.clone();
                                let vid2 = vid.clone();
                                view! {
                                    <button
                                        class="vendor-asset-option"
                                        on:click=move |_| {
                                            let a = asset_id.clone();
                                            let v = vid2.clone();
                                            let n = asset_name.clone();
                                            set_assign_pending.set(true);
                                            set_assign_error.set(None);
                                            set_assign_success.set(None);
                                            leptos::task::spawn_local(async move {
                                                match assign_vendor_to_asset(v, a).await {
                                                    Ok(_) => {
                                                        set_picker_open.set(false);
                                                        set_assign_success.set(Some(
                                                            format!("Set as default contractor for \"{n}\".")
                                                        ));
                                                    }
                                                    Err(e) => set_assign_error.set(Some(e.to_string())),
                                                }
                                                set_assign_pending.set(false);
                                            });
                                        }
                                        disabled=assign_pending
                                    >
                                        <div class="vendor-asset-option-info">
                                            <span class="material-symbols-outlined" style="font-size:16px;color:var(--folio-muted);">
                                                {NavIcon::Apartment.as_str()}
                                            </span>
                                            <div>
                                                <p class="vendor-asset-option-name">{asset.place_label()}</p>
                                                <p class="vendor-asset-option-meta">
                                                    {asset.asset_type.replace('_', " ")}
                                                    " \u{00b7} "
                                                    {asset.status}
                                                </p>
                                            </div>
                                        </div>
                                        <span class="material-symbols-outlined" style="font-size:16px;color:var(--folio-muted);">
                                            "chevron_right"
                                        </span>
                                    </button>
                                }
                            }).collect_view().into_any(),
                        }}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ── Skeleton ──────────────────────────────────────────────────────────────────

#[component]
fn VendorGridSkeleton() -> impl IntoView {
    view! {
        <div class="vendors-grid">
            {(0..6usize).map(|_| view! {
                <div class="vendor-card vendor-card--skeleton">
                    <div class="vendor-skeleton vendor-skeleton--avatar"/>
                    <div class="vendor-skeleton vendor-skeleton--name"/>
                    <div class="vendor-skeleton vendor-skeleton--pills"/>
                    <div class="vendor-skeleton vendor-skeleton--action"/>
                </div>
            }).collect_view()}
        </div>
    }
}

// ── Server functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[derive(Serialize)]
struct CreateVendorBody {
    user_id: Uuid,
    business_name: String,
    trade_type: String,
    license_number: Option<String>,
    license_state: Option<String>,
    is_emergency_available: bool,
    hourly_rate_cents: Option<i64>,
    is_insured: bool,
    is_bonded: bool,
}

#[derive(Deserialize)]
struct CreateVendorResponse {
    id: Uuid,
}

/// POST /api/folio/vendors
#[server(CreateVendor, "/api")]
pub async fn create_vendor(
    user_id: String,
    business_name: String,
    trade_type: String,
    is_emergency_available: bool,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let user_id = Uuid::parse_str(user_id.trim())
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid user ID"))?;
    if business_name.trim().is_empty() {
        return Err(server_fn::error::ServerFnError::new("Business name is required"));
    }
    if VendorTradeType::ALL
        .iter()
        .all(|t| t.as_str() != trade_type.as_str())
    {
        return Err(server_fn::error::ServerFnError::new("Invalid trade type"));
    }

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let body = CreateVendorBody {
        user_id,
        business_name: business_name.trim().to_string(),
        trade_type,
        license_number: None,
        license_state: None,
        is_emergency_available,
        hourly_rate_cents: None,
        is_insured: false,
        is_bonded: false,
    };
    let resp = crate::atlas_client::authenticated_post::<CreateVendorBody, CreateVendorResponse>(
        "/api/folio/vendors",
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Create vendor failed: {e}")))?;
    Ok(resp.id)
}

/// GET /api/folio/vendors — full vendor list for this tenant.
#[server(ListVendors, "/api")]
pub async fn list_vendors() -> Result<Vec<VendorSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<VendorSummary>>("/api/folio/vendors", &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Vendor list failed: {e}")))
}

/// GET /api/folio/assets — minimal list for the asset picker.
#[server(ListAssetsForPicker, "/api")]
pub async fn list_assets_for_picker(
) -> Result<Vec<AssetPickerItem>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    #[derive(serde::Deserialize)]
    struct RawAsset {
        id: uuid::Uuid,
        name: String,
        asset_type: String,
        status: String,
        #[serde(default)]
        address_line_1: Option<String>,
        #[serde(default)]
        city: Option<String>,
        #[serde(default)]
        state_province: Option<String>,
    }

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token_from_headers(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let raw =
        crate::atlas_client::authenticated_get::<Vec<RawAsset>>("/api/folio/assets", &token, None)
            .await
            .map_err(|e| server_fn::error::ServerFnError::new(format!("Asset list failed: {e}")))?;
    Ok(raw
        .into_iter()
        .map(|a| AssetPickerItem {
            id: a.id,
            name: a.name,
            asset_type: a.asset_type,
            status: a.status,
            address_line_1: a.address_line_1,
            city: a.city,
            state_province: a.state_province,
        })
        .collect())
}

/// Assigns this vendor as default contractor for an asset.
/// Fires POST /api/folio/relationships with relationship_type = "default_contractor".
/// Idempotent — the relationship service upserts.
#[server(AssignVendorToAsset, "/api")]
pub async fn assign_vendor_to_asset(
    vendor_id: String,
    asset_id: String,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let _ = uuid::Uuid::parse_str(&vendor_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid vendor ID"))?;
    let _ = uuid::Uuid::parse_str(&asset_id)
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid asset ID"))?;

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
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Assign failed: {e}")))
}
