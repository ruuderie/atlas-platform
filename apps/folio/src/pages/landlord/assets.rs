//! Assets list — `/l/assets`
//!
//! Wired to `GET /api/folio/assets`. Links each row to `/l/assets/:id`.

use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

use crate::components::page_header::PageHeader;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetSummary {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub portfolio_id: Option<uuid::Uuid>,
    #[serde(default)]
    pub parent_asset_id: Option<uuid::Uuid>,
    pub asset_type: String,
    pub name: String,
    pub serial_or_folio_number: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AssetFilter {
    All,
    Properties,
    Units,
}

impl AssetFilter {
    const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Properties => "Properties",
            Self::Units => "Units",
        }
    }
}

#[component]
pub fn Assets() -> impl IntoView {
    let (filter, set_filter) = signal(AssetFilter::Properties);
    let (search, set_search) = signal(String::new());
    let assets = Resource::new(|| (), |_| async move { list_assets().await });

    let title = Signal::derive(|| "Assets".to_string());
    let subtitle = Signal::derive(|| {
        "Properties and units across your portfolio.".to_string()
    });

    view! {
        <div class="landlord-list-page">
            <PageHeader title=title subtitle=subtitle>
                <A href="/l/map" attr:class="folio-btn folio-btn--ghost">
                    <span class="material-symbols-outlined">"map"</span>
                    "Map"
                </A>
            </PageHeader>

            <div class="landlord-filter-bar">
                <div class="landlord-search-wrap">
                    <span class="material-symbols-outlined landlord-search-icon">"search"</span>
                    <input
                        class="landlord-search-input"
                        type="search"
                        placeholder="Search by name or type…"
                        on:input=move |e| set_search.set(event_target_value(&e))
                    />
                </div>
                <div class="landlord-filter-chips">
                    {[AssetFilter::Properties, AssetFilter::Units, AssetFilter::All]
                        .into_iter()
                        .map(|f| view! {
                            <button
                                type="button"
                                class=move || if filter.get() == f {
                                    "landlord-chip landlord-chip--active"
                                } else {
                                    "landlord-chip"
                                }
                                on:click=move |_| set_filter.set(f)
                            >
                                {f.label()}
                            </button>
                        })
                        .collect_view()}
                </div>
            </div>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading assets…"</p></div>
            }>
                {move || assets.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load assets"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(all) => {
                        let q = search.get().to_lowercase();
                        let f = filter.get();
                        let filtered: Vec<_> = all.into_iter().filter(|a| {
                            let hierarchy_ok = match f {
                                AssetFilter::All => true,
                                AssetFilter::Properties => a.parent_asset_id.is_none(),
                                AssetFilter::Units => a.parent_asset_id.is_some(),
                            };
                            let search_ok = q.is_empty()
                                || a.name.to_lowercase().contains(&q)
                                || a.asset_type.to_lowercase().contains(&q)
                                || a.status.to_lowercase().contains(&q);
                            hierarchy_ok && search_ok
                        }).collect();

                        if filtered.is_empty() {
                            view! {
                                <div class="folio-empty">
                                    <span class="material-symbols-outlined folio-empty__icon">"apartment"</span>
                                    <p class="folio-empty__heading">"No assets yet"</p>
                                    <p class="folio-empty__sub">
                                        "Register a property during onboarding or add one from your portfolio."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="landlord-card-grid">
                                    {filtered.into_iter().map(|a| {
                                        let href = format!("/l/assets/{}", a.id);
                                        let status_class = if a.status == "active" {
                                            "landlord-pill landlord-pill--ok"
                                        } else {
                                            "landlord-pill landlord-pill--muted"
                                        };
                                        let status_label = a.status.clone();
                                        let name = a.name.clone();
                                        let asset_type = a.asset_type.replace('_', " ");
                                        let folio = a.serial_or_folio_number.clone()
                                            .unwrap_or_else(|| "—".into());
                                        let icon = if a.parent_asset_id.is_some() {
                                            "meeting_room"
                                        } else {
                                            "domain"
                                        };
                                        view! {
                                            <A href=href attr:class="landlord-card">
                                                <div class="landlord-card__top">
                                                    <span class="material-symbols-outlined landlord-card__icon">
                                                        {icon}
                                                    </span>
                                                    <span class=status_class>{status_label}</span>
                                                </div>
                                                <h3 class="landlord-card__title">{name}</h3>
                                                <p class="landlord-card__meta">{asset_type}</p>
                                                <p class="landlord-card__meta">"Folio · "{folio}</p>
                                            </A>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(ListLandlordAssets, "/api")]
pub async fn list_assets() -> Result<Vec<AssetSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<AssetSummary>>("/api/folio/assets", &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Asset list failed: {e}")))
}
