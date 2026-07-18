//! Property Owner Lite — Find a Vendor — `/po/find-vendor`
//! Wired to `GET /api/folio/marketplace/vendors`.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VendorCard {
    pub id: Uuid,
    pub business_name: String,
    pub marketplace_bio: Option<String>,
    #[serde(default)]
    pub trade_types: Vec<String>,
    pub rating_avg: Option<f64>,
    #[serde(default)]
    pub rating_count: i64,
    #[serde(default)]
    pub endorsement_count: i64,
    pub distance_km: Option<f64>,
    #[serde(default)]
    pub is_insured: bool,
    #[serde(default)]
    pub is_bonded: bool,
}

#[component]
pub fn FindVendorPage() -> impl IntoView {
    let (search, set_search) = signal(String::new());
    let vendors = Resource::new(|| (), |_| async move { list_marketplace_vendors().await });

    view! {
        <div class="page-header">
            <div>
                <h1 class="page-title">"Find a Vendor"</h1>
                <p class="page-subtitle">
                    "Browse verified contractors and service providers in the Folio network."
                </p>
            </div>
        </div>

        <div class="search-bar-wrap" style="margin-bottom:20px">
            <span class="ms search-bar__icon">"search"</span>
            <input
                type="search"
                placeholder="Search by trade or name…"
                class="form-input search-bar__input"
                on:input=move |e| set_search.set(event_target_value(&e))
            />
        </div>

        <Suspense fallback=|| view! { <div class="folio-empty"><p class="folio-empty__sub">"Loading vendors…"</p></div> }>
            {move || vendors.get().map(|result| match result {
                Err(e) => view! {
                    <div class="folio-empty">
                        <p class="folio-empty__heading">"Could not load marketplace"</p>
                        <p class="folio-empty__sub">{e.to_string()}</p>
                    </div>
                }.into_any(),
                Ok(items) => {
                    let q = search.get().to_lowercase();
                    let filtered: Vec<_> = items.into_iter().filter(|v| {
                        q.is_empty()
                            || v.business_name.to_lowercase().contains(&q)
                            || v.trade_types.iter().any(|t| t.to_lowercase().contains(&q))
                    }).collect();
                    if filtered.is_empty() {
                        view! {
                            <div class="folio-empty">
                                <span class="material-symbols-outlined folio-empty__icon">"handyman"</span>
                                <p class="folio-empty__heading">"No vendors found"</p>
                                <p class="folio-empty__sub">"Try another trade or check back as the marketplace grows."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="vendor-grid">
                                {filtered.into_iter().map(|v| {
                                    let trades = if v.trade_types.is_empty() {
                                        "General".into()
                                    } else {
                                        v.trade_types.join(" · ")
                                    };
                                    let rating = v.rating_avg
                                        .map(|r| format!("{r:.1}"))
                                        .unwrap_or_else(|| "—".into());
                                    view! {
                                        <div class="vendor-card">
                                            <div class="vendor-card__avatar">
                                                <span class="ms msf">"handyman"</span>
                                            </div>
                                            <div class="vendor-card__body">
                                                <p class="vendor-card__name">{v.business_name.clone()}</p>
                                                <p class="vendor-card__trade">{trades}</p>
                                                <div class="vendor-card__rating">
                                                    <span class="rating-stars">{format!("★ {rating}")}</span>
                                                    <span class="rating-count">{format!("({} reviews)", v.rating_count)}</span>
                                                </div>
                                                <p class="folio-empty__sub" style="margin-top:0.35rem;">
                                                    {v.marketplace_bio.clone().unwrap_or_default()}
                                                </p>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }
            })}
        </Suspense>
    }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(ListMarketplaceVendorsPo, "/api")]
pub async fn list_marketplace_vendors() -> Result<Vec<VendorCard>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<VendorCard>>(
        "/api/folio/marketplace/vendors?limit=50",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Marketplace vendors failed: {e}")))
}
