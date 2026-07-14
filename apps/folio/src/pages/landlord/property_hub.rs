//! Property hub / unit / leaf dispatch — `/l/assets/:id`

use crate::components::activity_rail::{ActivityRail, ActivityRailItem};
use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::property_tab_bar::{PropertyTab, PropertyTabBar};
use crate::components::stat_card::StatCard;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::asset_detail::AssetDetail as LeafAssetDetail;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetDetailDto {
    id: Uuid,
    parent_asset_id: Option<Uuid>,
    asset_type: String,
    name: String,
    status: String,
    city: Option<String>,
    state_province: Option<String>,
    str_eligible: bool,
    #[serde(default)]
    str_listing_active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetChildDto {
    id: Uuid,
    name: String,
    asset_type: String,
    status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSummaryDto {
    id: Uuid,
    title: String,
    status: String,
    estimated_cost_cents: Option<i64>,
    actual_spent_cents: i64,
    child_count: usize,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(GetAssetForDispatch, "/api")]
pub async fn get_asset_for_dispatch(id: Uuid) -> Result<AssetDetailDto, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get(&format!("/api/folio/assets/{id}"), &token, None)
        .await
        .map_err(ServerFnError::new)
}

#[server(GetAssetChildren, "/api")]
pub async fn get_asset_children(id: Uuid) -> Result<Vec<AssetChildDto>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get(
        &format!("/api/folio/assets/{id}/children"),
        &token,
        None,
    )
    .await
    .map_err(ServerFnError::new)
}

#[server(GetProjectsForAsset, "/api")]
pub async fn get_projects_for_asset(asset_id: Uuid) -> Result<Vec<ProjectSummaryDto>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get(
        &format!("/api/folio/projects?asset_id={asset_id}"),
        &token,
        None,
    )
    .await
    .map_err(ServerFnError::new)
}

fn is_multi_unit_parent(a: &AssetDetailDto, children: &[AssetChildDto]) -> bool {
    a.parent_asset_id.is_none()
        && (a.asset_type.contains("property") || a.asset_type.contains("building"))
        && children.iter().any(|c| c.asset_type.contains("unit"))
}

fn is_unit(a: &AssetDetailDto) -> bool {
    a.parent_asset_id.is_some() && a.asset_type.contains("unit")
}

#[component]
pub fn AssetRouteDispatch() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| {
        params
            .get()
            .get("id")
            .and_then(|s| Uuid::parse_str(&s).ok())
    });

    let asset = Resource::new(
        move || id.get(),
        |maybe| async move {
            match maybe {
                Some(aid) => get_asset_for_dispatch(aid).await,
                None => Err(ServerFnError::new("Missing asset id")),
            }
        },
    );

    let children = Resource::new(
        move || id.get(),
        |maybe| async move {
            match maybe {
                Some(aid) => get_asset_children(aid).await.unwrap_or_default(),
                None => vec![],
            }
        },
    );

    view! {
        <Suspense fallback=move || view! { <div class="folio-empty">"Loading property…"</div> }>
            {move || {
                let a = asset.get();
                let kids = children.get().unwrap_or_default();
                match a {
                    Some(Ok(ref asset_dto)) if is_multi_unit_parent(asset_dto, &kids) => {
                        view! { <PropertyHub asset=asset_dto.clone() children=kids/> }.into_any()
                    }
                    Some(Ok(ref asset_dto)) if is_unit(asset_dto) => {
                        view! { <UnitDetailPage asset=asset_dto.clone()/> }.into_any()
                    }
                    Some(Ok(_)) => view! { <LeafAssetDetail/> }.into_any(),
                    Some(Err(e)) => view! {
                        <div class="folio-empty"><p>{e.to_string()}</p></div>
                    }.into_any(),
                    None => view! { <div class="folio-empty">"Loading…"</div> }.into_any(),
                }
            }}
        </Suspense>
    }
}

#[component]
fn PropertyHub(asset: AssetDetailDto, children: Vec<AssetChildDto>) -> impl IntoView {
    let asset_id = asset.id;
    let asset_name = asset.name.clone();
    let asset_status = asset.status.clone();
    let str_eligible = asset.str_eligible;
    let loc = [
        asset.city.clone().unwrap_or_default(),
        asset.state_province.clone().unwrap_or_default(),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join(", ");

    let units: Vec<AssetChildDto> = children
        .into_iter()
        .filter(|c| c.asset_type.contains("unit"))
        .collect();
    let units_sig = RwSignal::new(units);
    let tab = RwSignal::new(PropertyTab::Overview);

    let projects = Resource::new(
        move || asset_id,
        |aid| async move { get_projects_for_asset(aid).await.unwrap_or_default() },
    );

    let maint_href = FolioRoute::LandlordMaintenance.path().to_string();
    let title_sig = Signal::derive({
        let name = asset_name.clone();
        move || name.clone()
    });
    let subtitle_sig = Signal::derive({
        let loc = loc.clone();
        move || {
            if loc.is_empty() {
                "Property hub".to_string()
            } else {
                loc.clone()
            }
        }
    });
    let status_sig = Signal::derive({
        let status = asset_status.clone();
        move || status.clone()
    });
    let rental_sig = Signal::derive(move || {
        if str_eligible {
            "Short-term eligible".to_string()
        } else {
            "Long-term".to_string()
        }
    });

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=title_sig
                subtitle=subtitle_sig
            >
                <a class="folio-btn folio-btn--primary" href=FolioRoute::LandlordMaintenanceNew.path()>
                    "New work order"
                </a>
            </PageHeader>

            <PropertyTabBar
                asset_id=asset_id
                active=tab
                on_overview=Callback::new(move |_| tab.set(PropertyTab::Overview))
                on_units=Callback::new(move |_| tab.set(PropertyTab::Units))
            />

            <Show when=move || matches!(tab.get(), PropertyTab::Overview)>
                <div class="hub-overview">
                    <div>
                        <div class="landlord-card-grid" style="margin-bottom:1.5rem;">
                            <StatCard label="Units" value=Signal::derive(move || units_sig.get().len().to_string()) icon="apartment"/>
                            <StatCard
                                label="Status"
                                value=status_sig
                                icon="info"
                            />
                            <StatCard
                                label="Rental"
                                value=rental_sig
                                icon="home"
                            />
                        </div>

                        <section class="proj-section" style="margin-bottom:1.5rem;">
                            <div class="proj-section__head">
                                <h3 class="proj-section__title">"Units"</h3>
                                <button
                                    type="button"
                                    class="hub-activity-rail__all"
                                    on:click=move |_| tab.set(PropertyTab::Units)
                                >
                                    "View all"
                                </button>
                            </div>
                            <For
                                each=move || units_sig.get()
                                key=|u| u.id
                                children=move |u| {
                                    let href = FolioRoute::LandlordAssetDetail
                                        .path()
                                        .replace(":id", &u.id.to_string());
                                    view! {
                                        <a class="hub-activity-rail__row press" href=href>
                                            <StatusPill label="Unit".to_string() tone=StatusPillTone::Info/>
                                            <div class="hub-activity-rail__body">
                                                <p class="hub-activity-rail__row-title">{u.name}</p>
                                                <p class="hub-activity-rail__row-meta">{u.status}</p>
                                            </div>
                                        </a>
                                    }
                                }
                            />
                        </section>

                        <section class="proj-section">
                            <div class="proj-section__head">
                                <div>
                                    <h3 class="proj-section__title">"Projects"</h3>
                                    <p class="proj-section__hint">"G-13 renovation_project"</p>
                                </div>
                            </div>
                            <Suspense fallback=|| view! { <div class="folio-empty--compact">"Loading…"</div> }>
                                {move || {
                                    let list = projects.get().unwrap_or_default();
                                    if list.is_empty() {
                                        return view! {
                                            <div class="folio-empty--compact">"No renovation projects yet."</div>
                                        }.into_any();
                                    }
                                    view! {
                                        <For
                                            each=move || list.clone()
                                            key=|p| p.id
                                            children=move |p| {
                                                let href = FolioRoute::LandlordProjectDetail
                                                    .path()
                                                    .replace(":id", &p.id.to_string());
                                                let spent = format!("${:.0}", p.actual_spent_cents as f64 / 100.0);
                                                let budget = p
                                                    .estimated_cost_cents
                                                    .map(|c| format!("${:.0}", c as f64 / 100.0))
                                                    .unwrap_or_else(|| "—".into());
                                                view! {
                                                    <a class="hub-activity-rail__row press" href=href>
                                                        <StatusPill label=p.status tone=StatusPillTone::Warn/>
                                                        <div class="hub-activity-rail__body">
                                                            <p class="hub-activity-rail__row-title">{p.title}</p>
                                                            <p class="hub-activity-rail__row-meta">
                                                                {format!("{spent} / {budget} · {} WOs", p.child_count)}
                                                            </p>
                                                        </div>
                                                    </a>
                                                }
                                            }
                                        />
                                    }.into_any()
                                }}
                            </Suspense>
                        </section>
                    </div>
                    <ActivityRail
                        items=Signal::derive(|| Vec::<ActivityRailItem>::new())
                        see_all_href=maint_href.clone()
                    />
                </div>
            </Show>

            <Show when=move || matches!(tab.get(), PropertyTab::Units)>
                <section class="proj-section">
                    <div class="proj-section__head">
                        <h3 class="proj-section__title">"All units"</h3>
                    </div>
                    <For
                        each=move || units_sig.get()
                        key=|u| u.id
                        children=move |u| {
                            let href = FolioRoute::LandlordAssetDetail
                                .path()
                                .replace(":id", &u.id.to_string());
                            view! {
                                <a class="hub-activity-rail__row press" href=href>
                                    <StatusPill label="Unit".to_string() tone=StatusPillTone::Info/>
                                    <div class="hub-activity-rail__body">
                                        <p class="hub-activity-rail__row-title">{u.name}</p>
                                        <p class="hub-activity-rail__row-meta">{u.status}</p>
                                    </div>
                                </a>
                            }
                        }
                    />
                </section>
            </Show>
        </div>
    }
}

#[component]
fn UnitDetailPage(asset: AssetDetailDto) -> impl IntoView {
    let name = asset.name.clone();
    let str_mode = asset.str_eligible;
    let parent = asset.parent_asset_id;
    let wo_new = FolioRoute::LandlordMaintenanceNew.path();

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(move || name.clone())
                subtitle=Signal::derive(move || -> String {
                    if str_mode {
                        "Unit · short-term rental".into()
                    } else {
                        "Unit · long-term rental".into()
                    }
                })
            >
                <a class="folio-btn folio-btn--primary" href=wo_new>"Create WO"</a>
            </PageHeader>
            {parent.map(|pid| {
                view! {
                    <PropertyTabBar
                        asset_id=pid
                        active=Signal::derive(|| PropertyTab::Units)
                    />
                }
            })}
            <section class="proj-section">
                <div class="proj-section__head">
                    <div>
                        <h3 class="proj-section__title">"Spaces"</h3>
                        <p class="proj-section__hint">"G-10 unit_space — spaces API"</p>
                    </div>
                </div>
                <div class="folio-empty--compact">
                    "Add kitchens, bathrooms, and other spaces to target work orders."
                </div>
            </section>
        </div>
    }
}
