// apps/folio/src/pages/landlord/contractor_marketplace.rs
//
// Contractor Marketplace — /l/marketplace
//
// Allows landlords to view, search, and onboard vendors/contractors.
// Powered by /api/folio/vendors.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::get_session;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorSummary {
    pub id: Uuid,
    pub business_name: String,
    pub trade_type: Option<String>,
    pub status: String,
    pub is_emergency_available: bool,
    pub rating_avg: Option<f64>,
    pub created_at: String,
}

/// Trade type — mirrors backend `TradeType` snake_case values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarketplaceTradeType {
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

impl MarketplaceTradeType {
    pub const ALL: &'static [Self] = &[
        Self::GeneralContractor,
        Self::Plumber,
        Self::Electrician,
        Self::Hvac,
        Self::Painter,
        Self::Roofer,
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

// ── Server functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(FetchVendors, "/api")]
pub async fn fetch_vendors() -> Result<Vec<VendorSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<VendorSummary>>("/api/folio/vendors", &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
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

/// POST /api/folio/vendors — uses session user_id when `user_id` is empty.
#[server(CreateMarketplaceVendor, "/api")]
pub async fn create_marketplace_vendor(
    business_name: String,
    trade_type: String,
    is_emergency_available: bool,
    user_id: Option<String>,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    if business_name.trim().is_empty() {
        return Err(server_fn::error::ServerFnError::new("Business name is required"));
    }
    if MarketplaceTradeType::ALL
        .iter()
        .all(|t| t.as_str() != trade_type.as_str())
    {
        return Err(server_fn::error::ServerFnError::new("Invalid trade type"));
    }

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let user_id = match user_id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => Uuid::parse_str(s)
            .map_err(|_| server_fn::error::ServerFnError::new("Invalid user ID"))?,
        None => get_session().await?.user_id,
    };

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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn trade_icon(trade: Option<&str>) -> &'static str {
    match trade {
        Some(t) if t.contains("plumb") => "🔧",
        Some(t) if t.contains("electric") => "⚡",
        Some(t) if t.contains("hvac") => "❄️",
        Some(t) if t.contains("paint") => "🎨",
        Some(t) if t.contains("roof") => "🏠",
        Some(t) if t.contains("landscape") => "🌿",
        Some(t) if t.contains("pest") => "🐜",
        Some(t) if t.contains("locksmith") => "🔑",
        Some(t) if t.contains("clean") => "🧹",
        Some(t) if t.contains("general") => "🛠",
        _ => "🏗",
    }
}

fn stars(rating: f64) -> String {
    let full = rating.floor() as usize;
    let empty = 5 - full.min(5);
    "★".repeat(full) + &"☆".repeat(empty)
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn ContractorMarketplace() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let trade_filter = RwSignal::new("all".to_string());
    let search_q = RwSignal::new(String::new());
    let filter_emerg = RwSignal::new(false);
    let show_add = RwSignal::new(false);

    // Add vendor form state
    let new_biz = RwSignal::new(String::new());
    let new_trade = RwSignal::new(MarketplaceTradeType::GeneralContractor.as_str().to_string());
    let new_user_id = RwSignal::new(String::new());
    let new_emerg = RwSignal::new(false);
    let adding = RwSignal::new(false);
    let add_err = RwSignal::new(None::<String>);

    let vendors_res = Resource::new(move || refresh.get(), |_| fetch_vendors());

    let on_add = move |_| {
        let biz = new_biz.get().trim().to_string();
        if biz.is_empty() {
            return;
        }
        let trade = new_trade.get();
        let emerg = new_emerg.get();
        let uid = {
            let s = new_user_id.get().trim().to_string();
            if s.is_empty() { None } else { Some(s) }
        };
        adding.set(true);
        add_err.set(None);
        spawn_local(async move {
            match create_marketplace_vendor(biz, trade, emerg, uid).await {
                Ok(_) => {
                    show_add.set(false);
                    new_biz.set(String::new());
                    new_user_id.set(String::new());
                    new_emerg.set(false);
                    refresh.update(|n| *n += 1);
                }
                Err(e) => add_err.set(Some(e.to_string())),
            }
            adding.set(false);
        });
    };

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <h1 class="page-title">"Contractor Marketplace"</h1>
                    <p class="page-subtitle">"Your vetted vendor network — find, add, and manage contractors by trade"</p>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-primary btn-sm"
                        on:click=move |_| {
                            add_err.set(None);
                            show_add.set(true);
                        }
                    >
                        "+ Add Vendor"
                    </button>
                </div>
            </div>

            // ── Search + filter bar ──
            <div class="mkt-controls">
                <input
                    type="text"
                    class="mkt-search"
                    placeholder="Search vendors…"
                    prop:value=search_q
                    on:input=move |ev| search_q.set(event_target_value(&ev))
                />
                <div class="mkt-filters">
                    {
                        let pill = move |scope: &'static str, label: &'static str| view! {
                            <button
                                class=move || format!("filter-pill {}", if trade_filter.get() == scope { "filter-pill--active" } else { "" })
                                on:click=move |_| trade_filter.set(scope.to_string())
                            >{label}</button>
                        };
                        view! {
                            {pill("all",                "All")}
                            {pill("plumber",            "Plumbing")}
                            {pill("electrician",        "Electrical")}
                            {pill("hvac",               "HVAC")}
                            {pill("general_contractor", "General")}
                            {pill("cleaner",            "Cleaning")}
                        }
                    }
                </div>
                <label class="mkt-emerg-toggle">
                    <input
                        type="checkbox"
                        prop:checked=move || filter_emerg.get()
                        on:change=move |ev: web_sys::Event| {
                            let el = event_target::<web_sys::HtmlInputElement>(&ev);
                            filter_emerg.set(el.checked());
                        }
                    />
                    "🚨 Emergency Available Only"
                </label>
            </div>

            // ── Vendor grid ──
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading vendors…"</div> }>
                {move || vendors_res.get().map(|res| {
                    match res {
                        Ok(vendors) => {
                            let q  = search_q.get().to_lowercase();
                            let tf = trade_filter.get();
                            let emerg_only = filter_emerg.get();

                            let visible: Vec<_> = vendors.into_iter().filter(|v| {
                                let trade_match = tf == "all" || v.trade_type.as_deref().unwrap_or("").contains(&tf);
                                let q_match     = q.is_empty() || v.business_name.to_lowercase().contains(&q);
                                let emerg_match = !emerg_only || v.is_emergency_available;
                                trade_match && q_match && emerg_match
                            }).collect();

                            if visible.is_empty() {
                                return view! {
                                    <div class="doc-empty">"No vendors match your filters."</div>
                                }.into_any();
                            }

                            view! {
                                <div class="mkt-grid">
                                    <For
                                        each=move || visible.clone()
                                        key=|v| v.id
                                        children=move |vendor| {
                                            let icon  = trade_icon(vendor.trade_type.as_deref());
                                            let trade = vendor.trade_type.clone().unwrap_or_else(|| "General".to_string()).replace('_', " ");
                                            let name  = vendor.business_name.clone();
                                            let emerg = vendor.is_emergency_available;
                                            let rating_str = vendor.rating_avg.map(|r| format!("{:.1} {}", r, stars(r)));

                                            view! {
                                                <div class="mkt-card">
                                                    {if emerg {
                                                        view! { <span class="mkt-emerg-chip">"🚨 Emergency"</span> }.into_any()
                                                    } else { ().into_any() }}

                                                    <div class="mkt-card-icon">{icon}</div>
                                                    <div class="mkt-card-name">{name}</div>
                                                    <div class="mkt-card-trade">{trade}</div>

                                                    {rating_str.map(|r| view! {
                                                        <div class="mkt-card-rating">{r}</div>
                                                    })}

                                                    <div class="mkt-card-status">
                                                        <span class=format!("mkt-status {}",
                                                            if vendor.status.to_lowercase() == "active" { "mkt-status--active" } else { "" }
                                                        )>
                                                            {vendor.status.clone()}
                                                        </span>
                                                    </div>

                                                    <div class="mkt-card-actions">
                                                        <a href="/l/maintenance" class="btn btn-primary btn-sm" style="width:100%;justify-content:center;">"Assign to Job"</a>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="doc-empty text-red-400">"Error: " {e.to_string()}</div>
                        }.into_any(),
                    }
                })}
            </Suspense>

            // ── Add Vendor Modal ─────────────────────────────────────────────
            <Show when=move || show_add.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:30rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Add a Vendor"</h3>
                            <button class="modal-close" on:click=move |_| show_add.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="form-field">
                                <label class="form-label">"Business Name *"</label>
                                <input type="text" class="form-input" placeholder="Acme Plumbing LLC"
                                    prop:value=new_biz
                                    on:input=move |ev| new_biz.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Trade Type"</label>
                                <select class="form-select" on:change=move |ev| new_trade.set(event_target_value(&ev))>
                                    {MarketplaceTradeType::ALL.iter().copied().map(|t| {
                                        view! { <option value=t.as_str()>{t.label()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"User ID (optional)"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    placeholder="Defaults to your session user"
                                    prop:value=new_user_id
                                    on:input=move |ev| new_user_id.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox"
                                        prop:checked=move || new_emerg.get()
                                        on:change=move |ev: web_sys::Event| {
                                            let el = event_target::<web_sys::HtmlInputElement>(&ev);
                                            new_emerg.set(el.checked());
                                        }
                                    />
                                    "Available for emergency calls"
                                </label>
                            </div>
                            {move || add_err.get().map(|e| view! {
                                <p class="text-red-400" style="font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                disabled=move || adding.get() || new_biz.get().trim().is_empty()
                                on:click=on_add
                            >
                                {move || if adding.get() { "Adding…" } else { "Add Vendor" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

        </div>
    }
}
