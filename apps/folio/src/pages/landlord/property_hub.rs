//! Property hub / unit / leaf dispatch — `/l/assets/:id`

use crate::components::activity_rail::{ActivityRail, ActivityRailItem};
use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::property_tab_bar::{PropertyTab, PropertyTabBar};
use crate::components::stat_card::StatCard;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::asset_api::{
    get_asset_children, get_asset_for_dispatch, get_projects_for_asset, AssetChildDto,
    AssetDetailDto,
};
use crate::pages::landlord::asset_detail::AssetDetail as LeafAssetDetail;
use crate::pages::landlord::maintenance_queue::{
    list_maintenance_tickets, CaseStatus, MaintenanceSummary,
};
use crate::pages::landlord::unit_detail::UnitDetailPage;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;

fn is_multi_unit_parent(a: &AssetDetailDto, children: &[AssetChildDto]) -> bool {
    // Hierarchy is authoritative. Onboarding stores PropertyType strings
    // (`multi_family`, …) on both parent and children — not `*property*` / `*unit*`.
    a.parent_asset_id.is_none() && !children.is_empty()
}

fn is_unit(a: &AssetDetailDto) -> bool {
    a.parent_asset_id.is_some()
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

    let units: Vec<AssetChildDto> = children;
    let mut scope = units.iter().map(|u| u.id).collect::<Vec<_>>();
    scope.push(asset_id);
    let scope_ids_sig = RwSignal::new(scope);
    let units_sig = RwSignal::new(units);
    let tab = RwSignal::new(PropertyTab::Overview);

    let projects = Resource::new(
        move || asset_id,
        |aid| async move { get_projects_for_asset(aid).await.unwrap_or_default() },
    );

    let tickets = Resource::new(|| (), |_| async move { list_maintenance_tickets().await });

    let activity_items = Signal::derive(move || {
        let scope = scope_ids_sig.get();
        tickets
            .get()
            .and_then(|r| r.ok())
            .map(|items: Vec<MaintenanceSummary>| {
                items
                    .into_iter()
                    .filter(|t| {
                        t.asset_id
                            .map(|aid| scope.contains(&aid))
                            .unwrap_or(false)
                    })
                    .filter(|t| {
                        matches!(
                            CaseStatus::from_str(&t.status),
                            CaseStatus::Open | CaseStatus::InProgress
                        )
                    })
                    .take(8)
                    .map(|t| ActivityRailItem {
                        id: t.id.to_string(),
                        kind_label: "WO".into(),
                        title: t.subject,
                        meta: format!("{} · {}", t.status, t.priority),
                        href: FolioRoute::LandlordMaintenanceDetail
                            .path()
                            .replace(":id", &t.id.to_string()),
                        tone: StatusPillTone::Warn,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

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
                <a class="folio-btn folio-btn--primary press" href=FolioRoute::LandlordMaintenanceNew.path()>
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
                                    <p class="proj-section__hint">"Renovation projects"</p>
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
                        items=activity_items
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

#[cfg(test)]
mod dispatch_tests {
    use super::{is_multi_unit_parent, is_unit};
    use crate::pages::landlord::asset_api::{AssetChildDto, AssetDetailDto};
    use uuid::Uuid;

    fn parent(asset_type: &str) -> AssetDetailDto {
        AssetDetailDto {
            id: Uuid::nil(),
            parent_asset_id: None,
            asset_type: asset_type.into(),
            name: "Bristol".into(),
            status: "active".into(),
            city: None,
            state_province: None,
            str_eligible: false,
            str_listing_active: false,
        }
    }

    fn child(asset_type: &str) -> AssetChildDto {
        AssetChildDto {
            id: Uuid::from_u128(1),
            name: "1".into(),
            asset_type: asset_type.into(),
            status: "active".into(),
        }
    }

    #[test]
    fn multi_family_parent_with_children_opens_hub() {
        let kids = vec![child("multi_family"), child("multi_family")];
        assert!(is_multi_unit_parent(&parent("multi_family"), &kids));
    }

    #[test]
    fn leaf_property_without_children_is_not_hub() {
        assert!(!is_multi_unit_parent(&parent("multi_family"), &[]));
    }

    #[test]
    fn nested_row_is_unit_regardless_of_type_string() {
        let mut unit = parent("multi_family");
        unit.parent_asset_id = Some(Uuid::from_u128(9));
        assert!(is_unit(&unit));
    }
}
