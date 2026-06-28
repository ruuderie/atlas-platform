// apps/folio/src/pages/landlord/listing_preview.rs
//
// Listing Network Preview — /l/assets/:id/preview
//
// Public-facing preview of how an asset will appear on the listing network.
// Pulls from /api/folio/catalog?asset_id={id} and /api/folio/assets/{id}.
// This is what tenants and applicants see on the Atlas Network portal.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id:              Uuid,
    pub asset_id:        Uuid,
    pub listing_type:    String,   // "long_term" | "short_term"
    pub headline:        Option<String>,
    pub description:     Option<String>,
    pub base_price_cents:Option<i64>,
    pub currency:        Option<String>,
    pub minimum_nights:  Option<i32>,
    pub is_active:       bool,
    pub created_at:      String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetBrief {
    pub id:            Uuid,
    pub address_line1: String,
    pub address_city:  String,
    pub address_state: Option<String>,
    pub unit_type:     String,
    pub bedrooms:      Option<i32>,
    pub bathrooms:     Option<f32>,
    pub sqft:          Option<i32>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchListingPreview, "/api")]
pub async fn fetch_listing_preview(asset_id: String) -> Result<Option<CatalogEntry>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/folio/catalog?asset_id={asset_id}");
    let entries = crate::atlas_client::authenticated_get::<Vec<CatalogEntry>>(&url, &token, None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(entries.into_iter().find(|e| e.is_active))
}

#[server(FetchAssetBrief, "/api")]
pub async fn fetch_asset_brief(asset_id: String) -> Result<AssetBrief, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/folio/assets/{asset_id}");
    crate::atlas_client::authenticated_get::<AssetBrief>(&url, &token, None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[cfg(feature = "ssr")]
fn session_token(headers: &axum::http::HeaderMap) -> Result<String, server_fn::error::ServerFnError> {
    headers.get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find_map(|p| {
            let p = p.trim();
            p.strip_prefix("session=").map(|t| t.to_string())
        }))
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmt_price(cents: i64, currency: &str, listing_type: &str) -> String {
    let amt = format!("{:.0}", cents as f64 / 100.0);
    let suffix = if listing_type == "short_term" { "/night" } else { "/mo" };
    format!("${}{}", amt, suffix)
}

fn unit_icon(unit_type: &str) -> &'static str {
    match unit_type.to_lowercase().as_str() {
        t if t.contains("house")     => "🏡",
        t if t.contains("condo")     => "🏢",
        t if t.contains("apartment") => "🏠",
        t if t.contains("townhouse") => "🏘",
        t if t.contains("studio")    => "🛏",
        _                            => "🏠",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn ListingNetworkPreview() -> impl IntoView {
    let params   = use_params_map();
    let asset_id = params.get().get("id").unwrap_or_default();

    let aid2 = asset_id.clone();
    let listing_res = Resource::new(
        move || asset_id.clone(),
        |id| fetch_listing_preview(id),
    );
    let asset_res = Resource::new(
        move || aid2.clone(),
        |id| fetch_asset_brief(id),
    );

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <a href="/l/assets" class="back-link">"← Back to Assets"</a>
                    <h1 class="page-title">"Listing Preview"</h1>
                    <p class="page-subtitle">"How this property appears to prospective tenants on the Atlas Network"</p>
                </div>
                <div class="page-actions">
                    <span class="listing-preview-badge">"Preview Mode"</span>
                </div>
            </div>

            // ── Preview card ──
            <div class="listing-preview-wrap">
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading listing…"</div> }>
                    {move || {
                        let listing_val = listing_res.get();
                        let asset_val   = asset_res.get();

                        match (listing_val, asset_val) {
                            (Some(Ok(Some(listing))), Some(Ok(asset))) => {
                                let price_str = listing.base_price_cents
                                    .map(|p| fmt_price(p, listing.currency.as_deref().unwrap_or("USD"), &listing.listing_type))
                                    .unwrap_or_else(|| "Contact for pricing".to_string());
                                let headline = listing.headline.clone().unwrap_or_else(|| {
                                    format!("{} in {}", asset.unit_type.replace('_', " "), asset.address_city)
                                });
                                let description = listing.description.clone().unwrap_or_else(|| "No description provided.".to_string());
                                let icon = unit_icon(&asset.unit_type);
                                let full_addr = format!("{}, {}{}", asset.address_line1, asset.address_city,
                                    asset.address_state.as_deref().map(|s| format!(", {}", s)).unwrap_or_default());

                                view! {
                                    <div class="listing-preview-card">

                                        // Simulated photo placeholder
                                        <div class="listing-preview-photo">
                                            <span class="listing-preview-photo-icon">{icon}</span>
                                            <div class="listing-preview-photo-overlay">
                                                {if listing.is_active {
                                                    view! { <span class="listing-live-chip">"● Live"</span> }.into_any()
                                                } else {
                                                    view! { <span class="listing-draft-chip">"Draft"</span> }.into_any()
                                                }}
                                            </div>
                                        </div>

                                        // Content
                                        <div class="listing-preview-body">
                                            <div class="listing-preview-price">{price_str}</div>
                                            <h2 class="listing-preview-headline">{headline}</h2>
                                            <div class="listing-preview-address">"📍 " {full_addr}</div>

                                            // Spec chips
                                            <div class="listing-spec-row">
                                                {asset.bedrooms.map(|b| view! {
                                                    <span class="listing-spec-chip">{b.to_string()} " bd"</span>
                                                })}
                                                {asset.bathrooms.map(|b| view! {
                                                    <span class="listing-spec-chip">{format!("{:.1}", b)} " ba"</span>
                                                })}
                                                {asset.sqft.map(|s| view! {
                                                    <span class="listing-spec-chip">{s.to_string()} " sqft"</span>
                                                })}
                                                <span class="listing-spec-chip">{listing.listing_type.replace('_', " ")}</span>
                                                {listing.minimum_nights.map(|m| view! {
                                                    <span class="listing-spec-chip">{m.to_string()} " night min"</span>
                                                })}
                                            </div>

                                            // Description
                                            <p class="listing-preview-desc">{description}</p>

                                            // CTA (preview mode)
                                            <div class="listing-preview-cta">
                                                <div class="listing-preview-cta-inner">
                                                    <div class="text-sm text-on-surface-variant">
                                                        "This is how applicants see your property. "
                                                        "Update your catalog entry to change what's displayed."
                                                    </div>
                                                    <a href="/l/catalog" class="btn btn-ghost btn-sm">"Edit Listing →"</a>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            (Some(Ok(None)), _) => view! {
                                <div class="listing-no-listing">
                                    <div class="listing-no-icon">"📋"</div>
                                    <div class="listing-no-title">"No Active Listing"</div>
                                    <div class="listing-no-sub">"This asset doesn't have an active catalog entry yet."</div>
                                    <a href="/l/catalog" class="btn btn-primary btn-sm mt-4">"Create Listing"</a>
                                </div>
                            }.into_any(),
                            _ => view! {
                                <div class="doc-empty">"Loading preview data…"</div>
                            }.into_any(),
                        }
                    }}
                </Suspense>
            </div>

        </div>
    }
}
