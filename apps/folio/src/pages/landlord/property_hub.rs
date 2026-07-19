//! Property hub / unit / leaf dispatch — `/l/assets/:id`

use crate::components::activity_rail::{ActivityRail, ActivityRailItem};
use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::property_tab_bar::{PropertyTab, PropertyTabBar};
use crate::components::stat_card::StatCard;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::asset_api::{
    archive_folio_asset, create_child_asset, create_project_for_asset, get_asset_children,
    get_asset_for_dispatch, get_projects_for_asset, ArchiveBlockerDto, AssetChildDto,
    AssetDetailDto,
};
use crate::pages::landlord::asset_detail::AssetDetail as LeafAssetDetail;
use crate::pages::landlord::maintenance_queue::{
    list_maintenance_tickets, CaseStatus, MaintenanceSummary,
};
use crate::pages::landlord::unit_detail::UnitDetailPage;
use leptos::prelude::*;
use leptos::task::spawn_local;
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

    let projects_refresh = RwSignal::new(0u32);
    let projects = Resource::new(
        move || (asset_id, projects_refresh.get()),
        |(aid, _)| async move { get_projects_for_asset(aid).await.unwrap_or_default() },
    );

    let tickets = Resource::new(|| (), |_| async move { list_maintenance_tickets().await });

    let show_add_unit = RwSignal::new(false);
    let new_unit_name = RwSignal::new(String::new());
    let add_unit_err = RwSignal::new(None::<String>);
    let add_unit_pending = RwSignal::new(false);

    let show_add_project = RwSignal::new(false);
    let new_project_title = RwSignal::new(String::new());
    let new_project_budget = RwSignal::new(String::new());
    let add_project_err = RwSignal::new(None::<String>);
    let add_project_pending = RwSignal::new(false);

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

    let archive_confirm = RwSignal::new(String::new());
    let archive_pending = RwSignal::new(false);
    let archive_err = RwSignal::new(None::<String>);
    let archive_blockers = RwSignal::new(Vec::<ArchiveBlockerDto>::new());
    let archived_ok = RwSignal::new(false);
    let on_archive = move |_| {
        if archive_confirm.get().trim() != "DELETE" {
            archive_err.set(Some("Type DELETE to confirm archive.".into()));
            return;
        }
        archive_pending.set(true);
        archive_err.set(None);
        archive_blockers.set(vec![]);
        spawn_local(async move {
            match archive_folio_asset(asset_id.to_string()).await {
                Ok(outcome) => {
                    if outcome.archived {
                        archived_ok.set(true);
                    } else {
                        archive_blockers.set(outcome.blockers);
                        archive_err.set(Some(
                            "This property cannot be archived until the items below are resolved."
                                .into(),
                        ));
                    }
                }
                Err(e) => archive_err.set(Some(e.to_string())),
            }
            archive_pending.set(false);
        });
    };

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
                                <div class="unit-actions">
                                    <button
                                        type="button"
                                        class="hub-activity-rail__all"
                                        on:click=move |_| show_add_unit.set(true)
                                    >
                                        "Add unit"
                                    </button>
                                    <button
                                        type="button"
                                        class="hub-activity-rail__all"
                                        on:click=move |_| tab.set(PropertyTab::Units)
                                    >
                                        "View all"
                                    </button>
                                </div>
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
                                <button
                                    type="button"
                                    class="hub-activity-rail__all"
                                    on:click=move |_| show_add_project.set(true)
                                >
                                    "New project"
                                </button>
                            </div>
                            <Suspense fallback=|| view! { <div class="folio-empty--compact">"Loading…"</div> }>
                                {move || {
                                    let list = projects.get().unwrap_or_default();
                                    if list.is_empty() {
                                        return view! {
                                            <div class="folio-empty--compact">
                                                <p>"No renovation projects yet."</p>
                                                <button
                                                    type="button"
                                                    class="folio-btn folio-btn--primary press"
                                                    style="margin-top:0.75rem;"
                                                    on:click=move |_| show_add_project.set(true)
                                                >
                                                    "New project"
                                                </button>
                                            </div>
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
                        <button
                            type="button"
                            class="folio-btn folio-btn--primary press"
                            on:click=move |_| show_add_unit.set(true)
                        >
                            "Add unit"
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
            </Show>

            <Show when=move || show_add_unit.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:24rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Add unit"</h3>
                            <button class="modal-close" on:click=move |_| show_add_unit.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <label class="form-label">
                                "Unit name"
                                <input
                                    class="form-input"
                                    type="text"
                                    placeholder="Unit 2B"
                                    prop:value=move || new_unit_name.get()
                                    on:input=move |ev| new_unit_name.set(event_target_value(&ev))
                                />
                            </label>
                            {move || add_unit_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add_unit.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                disabled=move || add_unit_pending.get()
                                on:click=move |_| {
                                    let name = new_unit_name.get().trim().to_string();
                                    if name.is_empty() {
                                        add_unit_err.set(Some("Name is required.".into()));
                                        return;
                                    }
                                    add_unit_pending.set(true);
                                    add_unit_err.set(None);
                                    spawn_local(async move {
                                        match create_child_asset(asset_id, name, "condo".into()).await {
                                            Ok(id) => {
                                                units_sig.update(|list| {
                                                    list.push(AssetChildDto {
                                                        id,
                                                        name: new_unit_name.get(),
                                                        asset_type: "condo".into(),
                                                        status: "active".into(),
                                                    });
                                                });
                                                scope_ids_sig.update(|s| s.push(id));
                                                new_unit_name.set(String::new());
                                                show_add_unit.set(false);
                                                add_unit_pending.set(false);
                                            }
                                            Err(e) => {
                                                add_unit_err.set(Some(e.to_string()));
                                                add_unit_pending.set(false);
                                            }
                                        }
                                    });
                                }
                            >
                                "Create unit"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_add_project.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:24rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"New project"</h3>
                            <button class="modal-close" on:click=move |_| show_add_project.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <label class="form-label">
                                "Title"
                                <input
                                    class="form-input"
                                    type="text"
                                    placeholder="Kitchen renovation"
                                    prop:value=move || new_project_title.get()
                                    on:input=move |ev| new_project_title.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="form-label">
                                "Budget (optional)"
                                <input
                                    class="form-input"
                                    type="text"
                                    inputmode="decimal"
                                    placeholder="25000"
                                    prop:value=move || new_project_budget.get()
                                    on:input=move |ev| new_project_budget.set(event_target_value(&ev))
                                />
                            </label>
                            {move || add_project_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add_project.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                disabled=move || add_project_pending.get()
                                on:click=move |_| {
                                    let title = new_project_title.get().trim().to_string();
                                    if title.is_empty() {
                                        add_project_err.set(Some("Title is required.".into()));
                                        return;
                                    }
                                    let budget = {
                                        let s = new_project_budget.get().trim().to_string();
                                        if s.is_empty() {
                                            None
                                        } else {
                                            match s.parse::<f64>() {
                                                Ok(v) if v >= 0.0 => Some((v * 100.0).round() as i64),
                                                _ => {
                                                    add_project_err.set(Some("Invalid budget.".into()));
                                                    return;
                                                }
                                            }
                                        }
                                    };
                                    add_project_pending.set(true);
                                    add_project_err.set(None);
                                    spawn_local(async move {
                                        match create_project_for_asset(asset_id, title, budget).await {
                                            Ok(_) => {
                                                new_project_title.set(String::new());
                                                new_project_budget.set(String::new());
                                                show_add_project.set(false);
                                                projects_refresh.update(|n| *n += 1);
                                                add_project_pending.set(false);
                                            }
                                            Err(e) => {
                                                add_project_err.set(Some(e.to_string()));
                                                add_project_pending.set(false);
                                            }
                                        }
                                    });
                                }
                            >
                                "Create project"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <section class="proj-section" style="margin-top:2rem;border-color:#fecaca;">
                <div class="proj-section__head">
                    <div>
                        <h3 class="proj-section__title" style="color:#b91c1c;">"Danger zone"</h3>
                        <p class="proj-section__hint">
                            "Archive hides this property from the Assets list. Active units block archive until they are archived first. Type DELETE to confirm."
                        </p>
                    </div>
                </div>
                {move || if archived_ok.get() {
                    view! {
                        <p style="color:#15803d;font-size:0.875rem;padding:0 1.25rem 1rem;">"Property archived."</p>
                    }.into_any()
                } else {
                    view! {
                        <div style="padding:0 1.25rem 1.25rem;display:flex;flex-direction:column;gap:0.75rem;max-width:28rem;">
                            <label class="form-label">
                                "Type DELETE"
                                <input
                                    class="form-input"
                                    type="text"
                                    autocomplete="off"
                                    prop:value=move || archive_confirm.get()
                                    on:input=move |ev| archive_confirm.set(event_target_value(&ev))
                                />
                            </label>
                            <button
                                type="button"
                                class="folio-btn"
                                style="background:#b91c1c;color:#fff;"
                                prop:disabled=move || {
                                    archive_pending.get()
                                        || archive_confirm.get().trim() != "DELETE"
                                }
                                on:click=on_archive
                            >
                                {move || if archive_pending.get() { "Archiving…" } else { "Archive property" }}
                            </button>
                            {move || archive_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                            {move || {
                                let blockers = archive_blockers.get();
                                if blockers.is_empty() {
                                    return ().into_any();
                                }
                                view! {
                                    <ul style="margin:0;padding-left:1.1rem;font-size:0.85rem;">
                                        {blockers.into_iter().map(|b| view! {
                                            <li>{b.message}</li>
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }}
                        </div>
                    }.into_any()
                }}
            </section>
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
            portfolio_id: None,
            parent_asset_id: None,
            asset_type: asset_type.into(),
            name: "Bristol".into(),
            status: "active".into(),
            address_line_1: None,
            address_line_2: None,
            city: None,
            state_province: None,
            postal_code: None,
            country_code: None,
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
