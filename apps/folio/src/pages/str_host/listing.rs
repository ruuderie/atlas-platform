// apps/folio/src/pages/str_host/listing.rs
//
// STR Listing Detail — /s/listings/:id
//
// Shows and allows editing of a specific catalog entry for an STR asset.
// Data from /api/folio/catalog?asset_id={id} (active entry).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id:               Uuid,
    pub asset_id:         Uuid,
    pub listing_type:     String,
    pub headline:         Option<String>,
    pub description:      Option<String>,
    pub base_price_cents: Option<i64>,
    pub currency:         Option<String>,
    pub minimum_nights:   Option<i32>,
    pub is_active:        bool,
    pub created_at:       String,
}

#[server(FetchStrListing, "/api")]
pub async fn fetch_str_listing(asset_id: String) -> Result<Option<CatalogEntry>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/folio/catalog?asset_id={asset_id}");
    let entries = crate::atlas_client::authenticated_get::<Vec<CatalogEntry>>(&url, &token, None)
        .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(entries.into_iter().find(|e| e.is_active))
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

fn fmt_price(cents: i64, currency: &str) -> String {
    format!("${:.2} {}/night", cents as f64 / 100.0, currency)
}

#[component]
pub fn StrListingDetail() -> impl IntoView {
    let params   = use_params_map();
    let asset_id = params.get().get("id").cloned().unwrap_or_default();

    // Editable fields (local state — save wires to backend in Phase 7)
    let headline    = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let loaded      = RwSignal::new(false);
    let saved       = RwSignal::new(false);

    let listing_res = Resource::new(
        move || asset_id.clone(),
        |id| fetch_str_listing(id),
    );

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <a href="/s" class="back-link">"← STR Dashboard"</a>
                    <h1 class="page-title">"Listing Detail"</h1>
                    <p class="page-subtitle">"Manage your STR listing content and settings"</p>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-primary btn-sm"
                        disabled=move || !loaded.get()
                        on:click=move |_| saved.set(true)
                    >"Save Changes"</button>
                </div>
            </div>

            {move || if saved.get() {
                view! { <div class="alert-saved-toast">"✓ Listing saved (Phase 7 will persist to backend)"</div> }.into_any()
            } else { ().into_any() }}

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading listing…"</div> }>
                {move || listing_res.get().map(|res| {
                    match res {
                        Ok(Some(entry)) => {
                            if !loaded.get() {
                                headline.set(entry.headline.clone().unwrap_or_default());
                                description.set(entry.description.clone().unwrap_or_default());
                                loaded.set(true);
                            }
                            let price_str = entry.base_price_cents.map(|p| fmt_price(p, entry.currency.as_deref().unwrap_or("USD")));
                            view! {
                                <div class="str-listing-form">
                                    // Status
                                    <div class="str-listing-status-row">
                                        <span class=if entry.is_active { "ph-badge ph-badge--paid" } else { "ph-badge ph-badge--default" }>
                                            {if entry.is_active { "● Live" } else { "Draft" }}
                                        </span>
                                        <span class="text-xs text-on-surface-variant">"Type: " {entry.listing_type.replace('_', " ")}</span>
                                        {price_str.map(|p| view! { <span class="text-xs font-bold" style="color:var(--green)">{p}</span> })}
                                        {entry.minimum_nights.map(|m| view! {
                                            <span class="text-xs text-on-surface-variant">"Min " {m.to_string()} " nights"</span>
                                        })}
                                    </div>

                                    // Headline
                                    <div class="form-field">
                                        <label class="form-label">"Listing Headline"</label>
                                        <input type="text" class="form-input" placeholder="Cozy downtown loft…"
                                            prop:value=move || headline.get()
                                            on:input=move |ev| { headline.set(event_target_value(&ev)); saved.set(false); }
                                        />
                                    </div>

                                    // Description
                                    <div class="form-field">
                                        <label class="form-label">"Description"</label>
                                        <textarea class="form-input str-listing-textarea"
                                            placeholder="Describe your space…"
                                            on:input=move |ev| { description.set(event_target_value(&ev)); saved.set(false); }
                                        >
                                            {move || description.get()}
                                        </textarea>
                                    </div>

                                    // Metadata
                                    <div class="form-field">
                                        <label class="form-label">"Catalog Entry ID"</label>
                                        <div class="font-mono text-xs opacity-50">{entry.id.to_string()}</div>
                                    </div>
                                    <div class="form-field">
                                        <label class="form-label">"Created"</label>
                                        <div class="text-sm text-on-surface-variant">{entry.created_at.chars().take(10).collect::<String>()}</div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Ok(None) => view! {
                            <div class="listing-no-listing">
                                <div class="listing-no-icon">"📋"</div>
                                <div class="listing-no-title">"No Active Listing"</div>
                                <div class="listing-no-sub">"Create a catalog entry for this asset first."</div>
                            </div>
                        }.into_any(),
                        Err(e) => view! {
                            <div class="doc-empty text-red-400">"Error: " {e.to_string()}</div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
