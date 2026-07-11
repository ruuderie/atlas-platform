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

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchVendors, "/api")]
pub async fn fetch_vendors() -> Result<Vec<VendorSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<VendorSummary>>("/api/folio/vendors", &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[cfg(feature = "ssr")]
fn session_token(
    headers: &axum::http::HeaderMap,
) -> Result<String, server_fn::error::ServerFnError> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            })
        })
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
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
    let show_add = RwSignal::new(false);

    // Add vendor form state
    let new_biz = RwSignal::new(String::new());
    let new_trade = RwSignal::new("general_contractor".to_string());
    let new_emerg = RwSignal::new(false);
    let adding = RwSignal::new(false);

    let vendors_res = Resource::new(move || refresh.get(), |_| fetch_vendors());

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <h1 class="page-title">"Contractor Marketplace"</h1>
                    <p class="page-subtitle">"Your vetted vendor network — find, add, and manage contractors by trade"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-primary btn-sm" on:click=move |_| show_add.set(true)>
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
                            {pill("plumbing",           "Plumbing")}
                            {pill("electrical",         "Electrical")}
                            {pill("hvac",               "HVAC")}
                            {pill("general_contractor", "General")}
                            {pill("cleaning",           "Cleaning")}
                        }
                    }
                </div>
                <label class="mkt-emerg-toggle">
                    <input type="checkbox" on:change=move |ev: web_sys::Event| {
                        let el = Some(event_target::<web_sys::HtmlInputElement>(&ev));
                        if let Some(el) = el { new_emerg.set(el.checked()); }
                    }/>
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
                            let emerg_only = new_emerg.get();

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
                                    <option value="general_contractor">"General Contractor"</option>
                                    <option value="plumbing">"Plumbing"</option>
                                    <option value="electrical">"Electrical"</option>
                                    <option value="hvac">"HVAC"</option>
                                    <option value="painting">"Painting"</option>
                                    <option value="roofing">"Roofing"</option>
                                    <option value="landscaping">"Landscaping"</option>
                                    <option value="cleaning">"Cleaning"</option>
                                    <option value="pest_control">"Pest Control"</option>
                                    <option value="locksmith">"Locksmith"</option>
                                </select>
                            </div>
                            <div class="form-field">
                                <label class="form-label flex items-center gap-2">
                                    <input type="checkbox" class="form-checkbox"
                                        on:change=move |ev: web_sys::Event| {
                                            let el = Some(event_target::<web_sys::HtmlInputElement>(&ev));
                                            if let Some(el) = el { new_emerg.set(el.checked()); }
                                        }
                                    />
                                    "Available for emergency calls"
                                </label>
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                disabled=move || adding.get() || new_biz.get().trim().is_empty()
                                on:click=move |_| {
                                    // Vendor creation requires a user_id (backend FK constraint).
                                    // This is orchestrated via the vendor onboarding flow.
                                    // For now show a notification and close modal.
                                    show_add.set(false);
                                }
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
