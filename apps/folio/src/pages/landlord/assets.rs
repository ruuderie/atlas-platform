//! Assets gallery — `/l/assets`
//!
//! Operator portfolio view: properties you own, open a building, or jump to a unit.
//! Wired to `GET /api/folio/assets` (children inferred from `parent_asset_id`).

use std::collections::HashMap;

use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::related_links::{RelatedLink, RelatedLinks};

const ASSET_RELATED: &[RelatedLink] = &[
    RelatedLink {
        route: FolioRoute::LandlordVault,
        label: "Vault",
    },
    RelatedLink {
        route: FolioRoute::LandlordSystems,
        label: "Systems",
    },
    RelatedLink {
        route: FolioRoute::LandlordAppliances,
        label: "Appliances",
    },
    RelatedLink {
        route: FolioRoute::LandlordCatalog,
        label: "Catalog",
    },
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetSummary {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub portfolio_id: Option<Uuid>,
    #[serde(default)]
    pub parent_asset_id: Option<Uuid>,
    pub asset_type: String,
    pub name: String,
    pub serial_or_folio_number: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub address_line_1: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub state_province: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub str_eligible: bool,
    #[serde(default)]
    pub str_listing_active: bool,
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

#[derive(Clone, Debug)]
struct UnitPeek {
    id: Uuid,
    name: String,
    status: String,
}

#[component]
pub fn Assets() -> impl IntoView {
    let (filter, set_filter) = signal(AssetFilter::Properties);
    let (search, set_search) = signal(String::new());
    let open_menu = RwSignal::new(Option::<Uuid>::None);
    let assets = Resource::new(|| (), |_| async move { list_assets().await });

    let title = Signal::derive(|| "Assets".to_string());
    let subtitle = Signal::derive(|| {
        "Your buildings and units.".to_string()
    });

    view! {
        <div class="landlord-list-page assets-page">
            <PageHeader title=title subtitle=subtitle>
                <A href=FolioRoute::LandlordMap.path() attr:class="folio-btn folio-btn--ghost press">
                    <span class="material-symbols-outlined">"map"</span>
                    "Map"
                </A>
                <A href=FolioRoute::LandlordAssetsCreate.path() attr:class="folio-btn folio-btn--primary press">
                    <span class="material-symbols-outlined">"add"</span>
                    "Add property"
                </A>
            </PageHeader>
            <RelatedLinks links=ASSET_RELATED />

            <div class="landlord-filter-bar">
                <div class="landlord-search-wrap">
                    <span class="material-symbols-outlined landlord-search-icon">"search"</span>
                    <input
                        class="landlord-search-input"
                        type="search"
                        placeholder="Search properties or units…"
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

                        let mut children_by_parent: HashMap<Uuid, Vec<UnitPeek>> = HashMap::new();
                        for a in &all {
                            if let Some(pid) = a.parent_asset_id {
                                children_by_parent.entry(pid).or_default().push(UnitPeek {
                                    id: a.id,
                                    name: a.name.clone(),
                                    status: a.status.clone(),
                                });
                            }
                        }

                        let property_count = all.iter().filter(|a| a.parent_asset_id.is_none()).count();
                        let unit_count = all.iter().filter(|a| a.parent_asset_id.is_some()).count();

                        let filtered: Vec<_> = all.into_iter().filter(|a| {
                            let hierarchy_ok = match f {
                                AssetFilter::All => true,
                                AssetFilter::Properties => a.parent_asset_id.is_none(),
                                AssetFilter::Units => a.parent_asset_id.is_some(),
                            };
                            let search_ok = q.is_empty()
                                || a.name.to_lowercase().contains(&q)
                                || a.asset_type.to_lowercase().contains(&q)
                                || a.status.to_lowercase().contains(&q)
                                || children_by_parent
                                    .get(&a.id)
                                    .map(|kids| kids.iter().any(|k| k.name.to_lowercase().contains(&q)))
                                    .unwrap_or(false);
                            hierarchy_ok && search_ok
                        }).collect();

                        if filtered.is_empty() {
                            view! {
                                <div class="folio-empty">
                                    <span class="material-symbols-outlined folio-empty__icon">"apartment"</span>
                                    <p class="folio-empty__heading">"No assets yet"</p>
                                    <p class="folio-empty__sub">
                                        "Add a property to start your holdings list."
                                    </p>
                                    <div style="margin-top:1rem">
                                        <A href=FolioRoute::LandlordAssetsCreate.path() attr:class="folio-btn folio-btn--primary press">
                                            "Add property"
                                        </A>
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="assets-kpi-strip">
                                    <div class="assets-kpi">
                                        <p class="assets-kpi__label">"Properties"</p>
                                        <p class="assets-kpi__value">{property_count.to_string()}</p>
                                    </div>
                                    <div class="assets-kpi">
                                        <p class="assets-kpi__label">"Units"</p>
                                        <p class="assets-kpi__value">{unit_count.to_string()}</p>
                                    </div>
                                </div>
                                <div class="assets-gallery">
                                    {filtered.into_iter().map(|a| {
                                        let is_unit = a.parent_asset_id.is_some();
                                        let href = FolioRoute::LandlordAssetDetail
                                            .path()
                                            .replace(":id", &a.id.to_string());
                                        let status_class = if a.status == "active" || a.status == "occupied" {
                                            "landlord-pill landlord-pill--ok"
                                        } else if a.status == "vacant" {
                                            "landlord-pill landlord-pill--warn"
                                        } else {
                                            "landlord-pill landlord-pill--muted"
                                        };
                                        let status_label = a.status.clone();
                                        let name = a.name.clone();
                                        let street = a
                                            .address_line_1
                                            .clone()
                                            .filter(|s| !s.trim().is_empty())
                                            .unwrap_or_else(|| name.clone());
                                        let place_meta = {
                                            let mut parts = Vec::new();
                                            if let Some(c) = a.city.as_ref().filter(|s| !s.is_empty()) {
                                                parts.push(c.clone());
                                            }
                                            if let Some(st) = a.state_province.as_ref().filter(|s| !s.is_empty()) {
                                                parts.push(st.clone());
                                            }
                                            if parts.is_empty() {
                                                a.asset_type.replace('_', " ")
                                            } else {
                                                parts.join(", ")
                                            }
                                        };
                                        let asset_type = a.asset_type.replace('_', " ");
                                        let kids = children_by_parent.get(&a.id).cloned().unwrap_or_default();
                                        let unit_n = kids.len();
                                        let aid = a.id;
                                        let menu_open = Signal::derive(move || open_menu.get() == Some(aid));
                                        let nick = if street != name && !name.is_empty() {
                                            Some(name.clone())
                                        } else {
                                            None
                                        };

                                        if is_unit {
                                            view! {
                                                <article class="assets-card">
                                                    <A href=href.clone() attr:class="assets-card__hero press">
                                                        <div class="assets-card__banner assets-card__banner--unit">
                                                            <span class="landlord-pill landlord-pill--muted" style="background:rgba(255,255,255,.15);color:#fff;border:none;">
                                                                "Unit"
                                                            </span>
                                                        </div>
                                                        <div class="assets-card__body">
                                                            <h3 class="assets-card__title">{street.clone()}</h3>
                                                            <p class="assets-card__meta">{place_meta.clone()}</p>
                                                            <div class="assets-card__pills">
                                                                <span class=status_class>{status_label.clone()}</span>
                                                            </div>
                                                        </div>
                                                    </A>
                                                    <div class="assets-card__actions">
                                                        <A href=href attr:class="folio-btn folio-btn--ghost press assets-card__cta">
                                                            "Open unit"
                                                        </A>
                                                    </div>
                                                </article>
                                            }.into_any()
                                        } else {
                                            let kids_for_menu = kids.clone();
                                            view! {
                                                <article class="assets-card">
                                                    <A href=href.clone() attr:class="assets-card__hero press">
                                                        <div class="assets-card__banner">
                                                            <span class="landlord-pill landlord-pill--muted" style="background:rgba(255,255,255,.15);color:#fff;border:none;">
                                                                {asset_type.clone()}
                                                            </span>
                                                        </div>
                                                        <div class="assets-card__body">
                                                            <h3 class="assets-card__title">{street.clone()}</h3>
                                                            <p class="assets-card__meta">
                                                                {format!(
                                                                    "{} · {}",
                                                                    place_meta,
                                                                    if unit_n > 0 {
                                                                        format!("{unit_n} units")
                                                                    } else {
                                                                        "Property".to_string()
                                                                    }
                                                                )}
                                                            </p>
                                                            {nick.clone().map(|n| view! {
                                                                <p class="assets-card__meta" style="opacity:0.75;">"Nickname: "{n}</p>
                                                            })}
                                                            <div class="assets-card__pills">
                                                                <span class=status_class>{status_label.clone()}</span>
                                                            </div>
                                                        </div>
                                                    </A>
                                                    <div class="assets-card__actions">
                                                        <A href=href.clone() attr:class="folio-btn folio-btn--ghost press assets-card__cta">
                                                            {if unit_n > 0 { "Open building" } else { "Open property" }}
                                                        </A>
                                                        {if unit_n > 0 {
                                                            view! {
                                                                <div class="assets-jump">
                                                                    <button
                                                                        type="button"
                                                                        class="folio-btn folio-btn--ghost press assets-card__cta"
                                                                        on:click=move |ev| {
                                                                            ev.prevent_default();
                                                                            open_menu.update(|cur| {
                                                                                *cur = if *cur == Some(aid) { None } else { Some(aid) };
                                                                            });
                                                                        }
                                                                    >
                                                                        "Jump to unit"
                                                                        <span class="material-symbols-outlined" style="font-size:16px;">"expand_more"</span>
                                                                    </button>
                                                                    <Show when=move || menu_open.get()>
                                                                        <div class="assets-unit-menu">
                                                                            {kids_for_menu.iter().map(|u| {
                                                                                let uh = FolioRoute::LandlordAssetDetail
                                                                                    .path()
                                                                                    .replace(":id", &u.id.to_string());
                                                                                let uname = u.name.clone();
                                                                                let ustatus = u.status.clone();
                                                                                view! {
                                                                                    <A
                                                                                        href=uh
                                                                                        attr:class="assets-unit-opt press"
                                                                                        on:click=move |_| open_menu.set(None)
                                                                                    >
                                                                                        <span class="assets-unit-opt__name">{uname}</span>
                                                                                        <span class="assets-unit-opt__meta">{ustatus}</span>
                                                                                    </A>
                                                                                }
                                                                            }).collect_view()}
                                                                        </div>
                                                                    </Show>
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! { <span></span> }.into_any()
                                                        }}
                                                    </div>
                                                </article>
                                            }.into_any()
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
