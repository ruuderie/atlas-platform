// apps/folio/src/pages/str_host/pricing.rs
//
// STR Pricing Rules — /s/pricing
//
// Lists and configures pricing rules per catalog entry.
// Uses /api/folio/catalog/{id}/rate-rules (POST to add) and
// /api/folio/catalog/{id}/availability (GET to show effective price).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct RuleRow {
    name: &'static str,
    trigger: &'static str,
    adjustment: &'static str,
    category: &'static str,
}

fn default_rules() -> Vec<RuleRow> {
    vec![
        RuleRow {
            name: "Weekend Premium",
            trigger: "Fri–Sun",
            adjustment: "+20%",
            category: "Seasonal",
        },
        RuleRow {
            name: "Last-Minute Discount",
            trigger: "< 3 days away",
            adjustment: "−15%",
            category: "Dynamic",
        },
        RuleRow {
            name: "Weekly Stay",
            trigger: "7+ nights",
            adjustment: "−10%",
            category: "Length",
        },
        RuleRow {
            name: "Monthly Stay",
            trigger: "28+ nights",
            adjustment: "−25%",
            category: "Length",
        },
        RuleRow {
            name: "High Season",
            trigger: "Jul–Aug",
            adjustment: "+30%",
            category: "Seasonal",
        },
        RuleRow {
            name: "Early Bird",
            trigger: "> 60 days out",
            adjustment: "−5%",
            category: "Dynamic",
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub listing_type: String,
    pub base_price_cents: Option<i64>,
    pub currency: Option<String>,
    pub is_active: bool,
}

#[server(FetchStrCatalog, "/api")]
pub async fn fetch_str_catalog() -> Result<Vec<CatalogEntry>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<CatalogEntry>>("/api/folio/catalog", &token, None)
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

fn fmt_price(cents: i64) -> String {
    format!("${:.0}/night", cents as f64 / 100.0)
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StrPricingRules() -> impl IntoView {
    let rules = default_rules();
    let show_add = RwSignal::new(false);
    let saved = RwSignal::new(false);

    let catalog_res = Resource::new(|| (), |_| fetch_str_catalog());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Pricing Rules"</h1>
                    <p class="page-subtitle">"Configure dynamic pricing adjustments across your STR listings"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-primary btn-sm" on:click=move |_| show_add.set(true)>
                        "+ Add Rule"
                    </button>
                </div>
            </div>

            {move || if saved.get() {
                view! { <div class="alert-saved-toast">"✓ Rule saved (POST to /api/folio/catalog/:id/rate-rules in Phase 7)"</div> }.into_any()
            } else { ().into_any() }}

            // ── Base prices per listing ──
            <div class="owner-section">
                <div class="owner-section-title">"Base Prices by Listing"</div>
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading catalog…"</div> }>
                    {move || catalog_res.get().map(|res| {
                        match res {
                            Ok(entries) if !entries.is_empty() => view! {
                                <div class="pricing-listing-grid">
                                    <For
                                        each=move || entries.clone()
                                        key=|e| e.id
                                        children=move |e| {
                                            let price = e.base_price_cents.map(fmt_price).unwrap_or_else(|| "—".to_string());
                                            let ltype = e.listing_type.replace('_', " ");
                                            view! {
                                                <div class="pricing-listing-card">
                                                    <div class="pricing-listing-type">{ltype}</div>
                                                    <div class="pricing-listing-base">{price}</div>
                                                    <div class="text-xs opacity-50 font-mono">{e.currency.unwrap_or_else(|| "USD".to_string())}</div>
                                                    {if e.is_active {
                                                        view! { <span class="ph-badge ph-badge--paid">"Live"</span> }.into_any()
                                                    } else { view! { <span class="ph-badge ph-badge--default">"Draft"</span> }.into_any() }}
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any(),
                            _ => view! { <div class="doc-empty">"No listings found."</div> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>

            // ── Pricing rules table ──
            <div class="owner-section">
                <div class="owner-section-title">"Configured Rules"</div>
                <div class="pricing-rules-list">
                    {rules.iter().map(|rule| {
                        let adj_color = if rule.adjustment.starts_with('+') { "var(--green)" } else { "var(--red)" };
                        view! {
                            <div class="pricing-rule-row">
                                <div class="pricing-rule-info">
                                    <div class="pricing-rule-name">{rule.name}</div>
                                    <div class="pricing-rule-trigger">{rule.trigger}</div>
                                </div>
                                <span class="pricing-rule-category">{rule.category}</span>
                                <span class="pricing-rule-adj" style=format!("color:{adj_color};font-weight:800;font-size:1rem;")>{rule.adjustment}</span>
                                <button class="btn btn-ghost btn-sm" disabled=true>"Edit"</button>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            // ── Add Rule Modal ──
            <Show when=move || show_add.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"New Pricing Rule"</h3>
                            <button class="modal-close" on:click=move |_| show_add.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="form-field">
                                <label class="form-label">"Rule Name"</label>
                                <input type="text" class="form-input" placeholder="Weekend Premium" />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Trigger"</label>
                                <select class="form-select">
                                    <option>"Weekend (Fri–Sun)"</option>
                                    <option>"Last Minute (< 3 days)"</option>
                                    <option>"Weekly (7+ nights)"</option>
                                    <option>"Monthly (28+ nights)"</option>
                                    <option>"High Season (custom dates)"</option>
                                </select>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Adjustment (%)"</label>
                                <input type="number" class="form-input" placeholder="20 (positive = increase)" />
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add.set(false)>"Cancel"</button>
                            <button class="btn btn-primary" on:click=move |_| {
                                show_add.set(false); saved.set(true);
                            }>"Save Rule"</button>
                        </div>
                    </div>
                </div>
            </Show>

        </div>
    }
}
