//! First-class unit workspace — `/l/assets/:id` when the asset is a unit child.
//!
//! Not a building tab: own breadcrumb + UnitTabBar + lease/WO/spaces/history.

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::asset_api::{
    archive_folio_asset as archive_unit_asset, create_child_asset, get_asset_children,
    get_asset_for_dispatch, ArchiveBlockerDto, AssetDetailDto,
};
use crate::pages::landlord::lease_detail::{
    get_lease_occupants, get_lease_vehicles, OccupantRecord, VehicleRecord,
};
use crate::pages::landlord::leases::{list_leases, LeaseStatus};
use crate::pages::landlord::ledger::list_ledger_entries;
use crate::pages::landlord::maintenance_queue::{
    list_maintenance_tickets, CaseStatus, MaintenanceSummary,
};
use crate::pages::tenant::household::{hh_add_occupant, hh_add_vehicle};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_location;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitTab {
    Overview,
    LeaseHousehold,
    Maintenance,
    Spaces,
    History,
}

impl UnitTab {
    const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::LeaseHousehold => "Lease & household",
            Self::Maintenance => "Maintenance",
            Self::Spaces => "Spaces",
            Self::History => "History",
        }
    }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(DepartLeaseOccupant, "/api")]
async fn depart_lease_occupant(
    lease_id: Uuid,
    entry_id: Uuid,
) -> Result<(), ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let _: serde_json::Value = crate::atlas_client::authenticated_post(
        &format!("/api/folio/leases/{lease_id}/occupants/{entry_id}/depart"),
        &token,
        None,
        &serde_json::json!({ "reason": "moved_out" }),
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(())
}

#[component]
pub fn UnitDetailPage(asset: AssetDetailDto) -> impl IntoView {
    let unit_id = asset.id;
    let parent_id = asset.parent_asset_id;
    let name = asset.name.clone();
    let status = asset.status.clone();
    let str_mode = asset.str_eligible;
    let tab = RwSignal::new(UnitTab::Overview);

    let location = use_location();
    Effect::new(move |_| {
        let path = location.pathname.get();
        if path.ends_with("/history") || path.ends_with("/archive") {
            tab.set(UnitTab::History);
        }
    });

    let parent = Resource::new(
        move || parent_id,
        |maybe| async move {
            match maybe {
                Some(pid) => get_asset_for_dispatch(pid).await.ok(),
                None => None,
            }
        },
    );

    let spaces_refresh = RwSignal::new(0u32);
    let spaces = Resource::new(
        move || (unit_id, spaces_refresh.get()),
        |(id, _)| async move {
            get_asset_children(id).await.unwrap_or_default()
        },
    );

    let leases = Resource::new(|| (), |_| async move { list_leases().await });
    let tickets = Resource::new(|| (), |_| async move { list_maintenance_tickets().await });
    let ledger = Resource::new(|| (), |_| async move { list_ledger_entries().await });

    let show_add_space = RwSignal::new(false);
    let new_space_name = RwSignal::new(String::new());
    let add_space_err = RwSignal::new(None::<String>);
    let add_space_pending = RwSignal::new(false);

    let show_add_occupant = RwSignal::new(false);
    let occ_name = RwSignal::new(String::new());
    let occ_rel = RwSignal::new("roommate".to_string());
    let occ_err = RwSignal::new(None::<String>);
    let occ_pending = RwSignal::new(false);
    let occ_refresh = RwSignal::new(0u32);

    let show_add_vehicle = RwSignal::new(false);
    let veh_make = RwSignal::new(String::new());
    let veh_model = RwSignal::new(String::new());
    let veh_year = RwSignal::new("2020".to_string());
    let veh_color = RwSignal::new(String::new());
    let veh_plate = RwSignal::new(String::new());
    let veh_state = RwSignal::new("FL".to_string());
    let veh_err = RwSignal::new(None::<String>);
    let veh_pending = RwSignal::new(false);
    let veh_refresh = RwSignal::new(0u32);

    let unit_lease = Signal::derive(move || {
        leases
            .get()
            .and_then(|r| r.ok())
            .and_then(|items| {
                items.into_iter().find(|l| {
                    l.asset_id == Some(unit_id)
                        && LeaseStatus::from_str(&l.status) == LeaseStatus::Active
                })
            })
    });

    let lease_id_for_occupants = Signal::derive(move || unit_lease.get().map(|l| l.id.to_string()));

    let occupants = Resource::new(
        move || (lease_id_for_occupants.get(), occ_refresh.get()),
        |(maybe, _)| async move {
            match maybe {
                Some(lid) => get_lease_occupants(lid)
                    .await
                    .map(|r| r.active)
                    .unwrap_or_default(),
                None => Vec::<OccupantRecord>::new(),
            }
        },
    );

    let vehicles = Resource::new(
        move || (lease_id_for_occupants.get(), veh_refresh.get()),
        |(maybe, _)| async move {
            match maybe {
                Some(lid) => get_lease_vehicles(lid).await.unwrap_or_default(),
                None => Vec::<VehicleRecord>::new(),
            }
        },
    );

    let unit_leases_all = Signal::derive(move || {
        leases
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .into_iter()
                    .filter(|l| l.asset_id == Some(unit_id))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    let unit_ledger = Signal::derive(move || {
        let lease_ids: std::collections::HashSet<_> =
            unit_leases_all.get().into_iter().map(|l| l.id).collect();
        ledger
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .into_iter()
                    .filter(|e| {
                        e.billable_entity_type == "atlas_contract"
                            && lease_ids.contains(&e.billable_entity_id)
                    })
                    .take(40)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    let unit_maint_all = Signal::derive(move || {
        tickets
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .into_iter()
                    .filter(|t| t.asset_id == Some(unit_id))
                    .take(40)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    let unit_tickets = Signal::derive(move || {
        tickets
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .into_iter()
                    .filter(|t| t.asset_id == Some(unit_id))
                    .filter(|t| {
                        matches!(
                            CaseStatus::from_str(&t.status),
                            CaseStatus::Open | CaseStatus::InProgress
                        )
                    })
                    .collect::<Vec<MaintenanceSummary>>()
            })
            .unwrap_or_default()
    });

    let wo_new = RwSignal::new(format!(
        "{}?asset_id={}",
        FolioRoute::LandlordMaintenanceNew.path(),
        unit_id
    ));
    let hist_lease_href = FolioRoute::LandlordHistoricalLease
        .path()
        .replace(":id", &unit_id.to_string());
    let hist_pay_href = FolioRoute::LandlordUnitPaymentHistory
        .path()
        .replace(":id", &unit_id.to_string());
    let hist_maint_href = FolioRoute::LandlordUnitMaintenanceHistory
        .path()
        .replace(":id", &unit_id.to_string());
    let log_paid_href = RwSignal::new(hist_maint_href.clone());
    let docs_href = parent_id.map(|pid| {
        FolioRoute::LandlordAssetDocuments
            .path()
            .replace(":id", &pid.to_string())
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
            match archive_unit_asset(unit_id.to_string()).await {
                Ok(outcome) => {
                    if outcome.archived {
                        archived_ok.set(true);
                    } else {
                        archive_blockers.set(outcome.blockers);
                        archive_err.set(Some(
                            "This unit cannot be archived until the items below are resolved."
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
        <div class="landlord-list-page unit-workspace">
            <nav class="unit-crumb" aria-label="Breadcrumb">
                <a href=FolioRoute::LandlordAssets.path()>"Assets"</a>
                <span class="material-symbols-outlined" style="font-size:14px;">"chevron_right"</span>
                <Suspense fallback=|| view! { <span>"Building"</span> }>
                    {move || {
                        let building_name = parent
                            .get()
                            .flatten()
                            .map(|p| p.name)
                            .unwrap_or_else(|| "Building".into());
                        match parent_id {
                            Some(pid) => {
                                let href = FolioRoute::LandlordAssetDetail
                                    .path()
                                    .replace(":id", &pid.to_string());
                                view! { <a href=href>{building_name}</a> }.into_any()
                            }
                            None => view! { <span>{building_name}</span> }.into_any(),
                        }
                    }}
                </Suspense>
                <span class="material-symbols-outlined" style="font-size:14px;">"chevron_right"</span>
                <span style="font-weight:700;color:#191c1e;">{name.clone()}</span>
            </nav>

            <PageHeader
                title=Signal::derive({
                    let n = name.clone();
                    move || n.clone()
                })
                subtitle=Signal::derive(move || -> String {
                    if str_mode {
                        "Unit · short-term rental".into()
                    } else {
                        "Unit · long-term rental".into()
                    }
                })
            >
                <a
                    class="folio-btn folio-btn--ghost press"
                    href=format!(
                        "{}?asset_id={}",
                        FolioRoute::LandlordLeaseCreate.path(),
                        unit_id
                    )
                >
                    "New lease"
                </a>
                <a class="folio-btn folio-btn--primary press" href=move || wo_new.get()>"Create WO"</a>
            </PageHeader>

            <div class="unit-tab-bar" role="tablist">
                {[
                    UnitTab::Overview,
                    UnitTab::LeaseHousehold,
                    UnitTab::Maintenance,
                    UnitTab::Spaces,
                    UnitTab::History,
                ]
                    .into_iter()
                    .map(|t| {
                        view! {
                            <button
                                type="button"
                                role="tab"
                                class=move || {
                                    if tab.get() == t {
                                        "unit-tab unit-tab--active"
                                    } else {
                                        "unit-tab"
                                    }
                                }
                                on:click=move |_| tab.set(t)
                            >
                                {t.label()}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            <Show when=move || tab.get() == UnitTab::Overview>
                <div class="unit-panel">
                    <div class="assets-card__pills">
                        <StatusPill
                            label=status.clone()
                            tone=if status == "vacant" {
                                StatusPillTone::Warn
                            } else {
                                StatusPillTone::Ok
                            }
                        />
                        <StatusPill
                            label=if str_mode {
                                "Short-term".to_string()
                            } else {
                                "Long-term".to_string()
                            }
                            tone=StatusPillTone::Info
                        />
                    </div>

                    <div class="unit-actions">
                        <a class="folio-btn folio-btn--primary press" href=move || wo_new.get()>"Create WO"</a>
                        <a
                            class="folio-btn folio-btn--ghost press"
                            href=move || log_paid_href.get()
                        >
                            "Log paid"
                        </a>
                        {move || unit_lease.get().map(|l| {
                            let href = FolioRoute::LandlordLeaseDetail
                                .path()
                                .replace(":id", &l.id.to_string());
                            view! {
                                <a class="folio-btn folio-btn--ghost press" href=href>"Open lease"</a>
                            }
                        })}
                        {docs_href.clone().map(|h| view! {
                            <a class="folio-btn folio-btn--ghost press" href=h>"Building documents"</a>
                        })}
                        <button
                            type="button"
                            class="folio-btn folio-btn--ghost press"
                            on:click=move |_| tab.set(UnitTab::History)
                        >
                            "History"
                        </button>
                    </div>

                    <section class="proj-section">
                        <div class="proj-section__head">
                            <h3 class="proj-section__title">"Lease snapshot"</h3>
                        </div>
                        {move || match unit_lease.get() {
                            Some(l) => {
                                let rent = l
                                    .monthly_rent_cents
                                    .map(|c| format!("${:.0}/mo", c as f64 / 100.0))
                                    .unwrap_or_else(|| "—".into());
                                let dates = match (l.start_date, l.end_date) {
                                    (Some(s), Some(e)) => format!("{s} → {e}"),
                                    (Some(s), None) => format!("From {s}"),
                                    _ => "Dates not set".into(),
                                };
                                view! {
                                    <div class="hub-activity-rail__row">
                                        <StatusPill label="Occupied".to_string() tone=StatusPillTone::Ok/>
                                        <div class="hub-activity-rail__body">
                                            <p class="hub-activity-rail__row-title">{rent}</p>
                                            <p class="hub-activity-rail__row-meta">{dates}</p>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            None => view! {
                                <div class="folio-empty--compact">
                                    {if str_mode {
                                        "No active long-term lease — manage stays from Reservations when connected."
                                    } else {
                                        "No active lease on this unit."
                                    }}
                                </div>
                            }.into_any(),
                        }}
                    </section>

                    <section class="proj-section">
                        <div class="proj-section__head">
                            <h3 class="proj-section__title">"Open work orders"</h3>
                            <button
                                type="button"
                                class="hub-activity-rail__all"
                                on:click=move |_| tab.set(UnitTab::Maintenance)
                            >
                                "View all"
                            </button>
                        </div>
                        {move || {
                            let list = unit_tickets.get();
                            if list.is_empty() {
                                view! {
                                    <div class="folio-empty--compact">"No open work orders on this unit."</div>
                                }.into_any()
                            } else {
                                view! {
                                    <For
                                        each=move || list.clone()
                                        key=|t| t.id
                                        children=move |t| {
                                            let href = FolioRoute::LandlordMaintenanceDetail
                                                .path()
                                                .replace(":id", &t.id.to_string());
                                            view! {
                                                <a class="hub-activity-rail__row press" href=href>
                                                    <StatusPill label=t.priority.clone() tone=StatusPillTone::Warn/>
                                                    <div class="hub-activity-rail__body">
                                                        <p class="hub-activity-rail__row-title">{t.subject.clone()}</p>
                                                        <p class="hub-activity-rail__row-meta">{t.status.clone()}</p>
                                                    </div>
                                                </a>
                                            }
                                        }
                                    />
                                }.into_any()
                            }
                        }}
                    </section>
                </div>
            </Show>

            <Show when=move || tab.get() == UnitTab::LeaseHousehold>
                <div class="unit-panel">
                    {move || match unit_lease.get() {
                        None => view! {
                            <div class="folio-empty">
                                <p class="folio-empty__heading">"No active lease"</p>
                                <p class="folio-empty__sub">
                                    "Household and vehicles appear when a lease is linked to this unit."
                                </p>
                                <a
                                    class="folio-btn folio-btn--primary press"
                                    href=format!(
                                        "{}?asset_id={}",
                                        FolioRoute::LandlordLeaseCreate.path(),
                                        unit_id
                                    )
                                >
                                    "New lease"
                                </a>
                            </div>
                        }.into_any(),
                        Some(l) => {
                            let href = FolioRoute::LandlordLeaseDetail
                                .path()
                                .replace(":id", &l.id.to_string());
                            let rent = l
                                .monthly_rent_cents
                                .map(|c| format!("${:.0}/mo", c as f64 / 100.0))
                                .unwrap_or_else(|| "—".into());
                            view! {
                                <section class="proj-section">
                                    <div class="proj-section__head">
                                        <h3 class="proj-section__title">"Lease"</h3>
                                        <a class="hub-activity-rail__all" href=href>"Open detail"</a>
                                    </div>
                                    <p class="hub-activity-rail__row-meta">{rent}</p>
                                </section>
                                <section class="proj-section">
                                    <div class="proj-section__head">
                                        <h3 class="proj-section__title">"Household"</h3>
                                        <button
                                            type="button"
                                            class="hub-activity-rail__all"
                                            on:click=move |_| show_add_occupant.set(true)
                                        >
                                            "Add occupant"
                                        </button>
                                    </div>
                                    <Suspense fallback=|| view! { <div class="folio-empty--compact">"Loading…"</div> }>
                                        {move || {
                                            let people = occupants.get().unwrap_or_default();
                                            if people.is_empty() {
                                                view! {
                                                    <div class="folio-empty--compact">
                                                        <p>"No occupants registered on this lease."</p>
                                                        <button
                                                            type="button"
                                                            class="folio-btn folio-btn--primary press"
                                                            style="margin-top:0.75rem;"
                                                            on:click=move |_| show_add_occupant.set(true)
                                                        >
                                                            "Add occupant"
                                                        </button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <For
                                                        each=move || people.clone()
                                                        key=|o| o.id
                                                        children=move |o: OccupantRecord| {
                                                            let oid = o.id;
                                                            let lid = unit_lease.get().map(|l| l.id);
                                                            view! {
                                                                <div class="hub-activity-rail__row">
                                                                    <StatusPill label="Occupant".to_string() tone=StatusPillTone::Info/>
                                                                    <div class="hub-activity-rail__body">
                                                                        <p class="hub-activity-rail__row-title">{o.full_name.clone()}</p>
                                                                        <p class="hub-activity-rail__row-meta">{o.relationship.clone()}</p>
                                                                    </div>
                                                                    <button
                                                                        type="button"
                                                                        class="folio-btn folio-btn--ghost press"
                                                                        on:click=move |_| {
                                                                            let Some(lease_id) = lid else { return; };
                                                                            spawn_local(async move {
                                                                                if depart_lease_occupant(lease_id, oid).await.is_ok() {
                                                                                    occ_refresh.update(|n| *n += 1);
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        "Depart"
                                                                    </button>
                                                                </div>
                                                            }
                                                        }
                                                    />
                                                }.into_any()
                                            }
                                        }}
                                    </Suspense>
                                </section>
                                <section class="proj-section">
                                    <div class="proj-section__head">
                                        <h3 class="proj-section__title">"Vehicles"</h3>
                                        <button
                                            type="button"
                                            class="hub-activity-rail__all"
                                            on:click=move |_| show_add_vehicle.set(true)
                                        >
                                            "Register vehicle"
                                        </button>
                                    </div>
                                    <Suspense fallback=|| view! { <div class="folio-empty--compact">"Loading…"</div> }>
                                        {move || {
                                            let list = vehicles.get().unwrap_or_default();
                                            if list.is_empty() {
                                                view! {
                                                    <div class="folio-empty--compact">"No vehicles registered."</div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <For
                                                        each=move || list.clone()
                                                        key=|v| v.id
                                                        children=move |v: VehicleRecord| {
                                                            view! {
                                                                <div class="hub-activity-rail__row">
                                                                    <StatusPill label="Vehicle".to_string() tone=StatusPillTone::Info/>
                                                                    <div class="hub-activity-rail__body">
                                                                        <p class="hub-activity-rail__row-title">
                                                                            {format!("{} {} {}", v.year, v.make, v.model)}
                                                                        </p>
                                                                        <p class="hub-activity-rail__row-meta">{v.license_plate.clone()}</p>
                                                                    </div>
                                                                </div>
                                                            }
                                                        }
                                                    />
                                                }.into_any()
                                            }
                                        }}
                                    </Suspense>
                                </section>
                            }.into_any()
                        }
                    }}
                </div>
            </Show>

            <Show when=move || tab.get() == UnitTab::Maintenance>
                <div class="unit-panel">
                    <div class="unit-actions">
                        <a class="folio-btn folio-btn--primary press" href=move || wo_new.get()>"Create WO"</a>
                    </div>
                    {move || {
                        let list = unit_tickets.get();
                        if list.is_empty() {
                            view! {
                                <div class="folio-empty--compact">"No open work orders for this unit."</div>
                            }.into_any()
                        } else {
                            view! {
                                <For
                                    each=move || list.clone()
                                    key=|t| t.id
                                    children=move |t| {
                                        let href = FolioRoute::LandlordMaintenanceDetail
                                            .path()
                                            .replace(":id", &t.id.to_string());
                                        view! {
                                            <a class="hub-activity-rail__row press" href=href>
                                                <StatusPill label=t.status.clone() tone=StatusPillTone::Warn/>
                                                <div class="hub-activity-rail__body">
                                                    <p class="hub-activity-rail__row-title">{t.subject.clone()}</p>
                                                    <p class="hub-activity-rail__row-meta">{t.priority.clone()}</p>
                                                </div>
                                            </a>
                                        }
                                    }
                                />
                            }.into_any()
                        }
                    }}
                </div>
            </Show>

            <Show when=move || tab.get() == UnitTab::Spaces>
                <div class="unit-panel">
                    <section class="proj-section">
                        <div class="proj-section__head">
                            <div>
                                <h3 class="proj-section__title">"Spaces"</h3>
                                <p class="proj-section__hint">"Kitchens, baths, and rooms for targeting work orders"</p>
                            </div>
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary press"
                                on:click=move |_| show_add_space.set(true)
                            >
                                "Add space"
                            </button>
                        </div>
                        <Suspense fallback=|| view! { <div class="folio-empty--compact">"Loading…"</div> }>
                            {move || {
                                let list = spaces.get().unwrap_or_default();
                                if list.is_empty() {
                                    view! {
                                        <div class="folio-empty--compact">
                                            <p>"No spaces yet."</p>
                                            <button
                                                type="button"
                                                class="folio-btn folio-btn--primary press"
                                                style="margin-top:0.75rem;"
                                                on:click=move |_| show_add_space.set(true)
                                            >
                                                "Add space"
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <For
                                            each=move || list.clone()
                                            key=|s| s.id
                                            children=move |s| {
                                                let href = FolioRoute::LandlordAssetDetail
                                                    .path()
                                                    .replace(":id", &s.id.to_string());
                                                view! {
                                                    <a class="hub-activity-rail__row press" href=href>
                                                        <StatusPill label="Space".to_string() tone=StatusPillTone::Info/>
                                                        <div class="hub-activity-rail__body">
                                                            <p class="hub-activity-rail__row-title">{s.name}</p>
                                                            <p class="hub-activity-rail__row-meta">{s.asset_type}</p>
                                                        </div>
                                                    </a>
                                                }
                                            }
                                        />
                                    }.into_any()
                                }
                            }}
                        </Suspense>
                    </section>
                </div>
            </Show>

            <Show when=move || tab.get() == UnitTab::History>
                <div class="unit-panel">
                    <div class="lg:grid lg:grid-cols-3 lg:gap-6">
                        <div class="lg:col-span-2" style="display:flex;flex-direction:column;gap:1.25rem;">
                            <section class="proj-section">
                                <div class="proj-section__head">
                                    <div>
                                        <h3 class="proj-section__title">"Leases"</h3>
                                        <p class="proj-section__hint">"All leases tied to this unit"</p>
                                    </div>
                                </div>
                                {move || {
                                    let list = unit_leases_all.get();
                                    if list.is_empty() {
                                        view! {
                                            <div class="folio-empty--compact">"No leases on this unit yet."</div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <For
                                                each=move || list.clone()
                                                key=|l| l.id
                                                children=move |l| {
                                                    let href = FolioRoute::LandlordLeaseDetail
                                                        .path()
                                                        .replace(":id", &l.id.to_string());
                                                    let rent = l.monthly_rent_cents
                                                        .map(|c| format!("${:.0}/mo", c as f64 / 100.0))
                                                        .unwrap_or_else(|| "—".into());
                                                    let dates = format!(
                                                        "{} → {}",
                                                        l.start_date.map(|d| d.to_string()).unwrap_or_else(|| "—".into()),
                                                        l.end_date.map(|d| d.to_string()).unwrap_or_else(|| "—".into())
                                                    );
                                                    view! {
                                                        <a class="hub-activity-rail__row press" href=href>
                                                            <StatusPill label=l.status.clone() tone=StatusPillTone::Info/>
                                                            <div class="hub-activity-rail__body">
                                                                <p class="hub-activity-rail__row-title">{rent}</p>
                                                                <p class="hub-activity-rail__row-meta">{dates}</p>
                                                            </div>
                                                        </a>
                                                    }
                                                }
                                            />
                                        }.into_any()
                                    }
                                }}
                            </section>

                            <section class="proj-section">
                                <div class="proj-section__head">
                                    <div>
                                        <h3 class="proj-section__title">"Payments"</h3>
                                        <p class="proj-section__hint">"Ledger rows for this unit’s leases"</p>
                                    </div>
                                </div>
                                {move || {
                                    let list = unit_ledger.get();
                                    if list.is_empty() {
                                        view! {
                                            <div class="folio-empty--compact">"No ledger entries yet."</div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <For
                                                each=move || list.clone()
                                                key=|e| e.id
                                                children=move |e| {
                                                    let amt = format!("${:.2}", e.gross_amount_cents as f64 / 100.0);
                                                    let desc = e.description.clone().unwrap_or_else(|| e.status.clone());
                                                    view! {
                                                        <div class="hub-activity-rail__row">
                                                            <StatusPill label=e.status.clone() tone=StatusPillTone::Warn/>
                                                            <div class="hub-activity-rail__body">
                                                                <p class="hub-activity-rail__row-title">{amt}</p>
                                                                <p class="hub-activity-rail__row-meta">{desc}</p>
                                                            </div>
                                                        </div>
                                                    }
                                                }
                                            />
                                        }.into_any()
                                    }
                                }}
                            </section>

                            <section class="proj-section">
                                <div class="proj-section__head">
                                    <div>
                                        <h3 class="proj-section__title">"Maintenance"</h3>
                                        <p class="proj-section__hint">"Work orders and expenses for this unit"</p>
                                    </div>
                                </div>
                                {move || {
                                    let list = unit_maint_all.get();
                                    if list.is_empty() {
                                        view! {
                                            <div class="folio-empty--compact">"No maintenance history yet."</div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <For
                                                each=move || list.clone()
                                                key=|t| t.id
                                                children=move |t| {
                                                    let href = FolioRoute::LandlordMaintenanceDetail
                                                        .path()
                                                        .replace(":id", &t.id.to_string());
                                                    view! {
                                                        <a class="hub-activity-rail__row press" href=href>
                                                            <StatusPill label=t.status.clone() tone=StatusPillTone::Warn/>
                                                            <div class="hub-activity-rail__body">
                                                                <p class="hub-activity-rail__row-title">{t.subject.clone()}</p>
                                                                <p class="hub-activity-rail__row-meta">{t.priority.clone()}</p>
                                                            </div>
                                                        </a>
                                                    }
                                                }
                                            />
                                        }.into_any()
                                    }
                                }}
                            </section>
                        </div>

                        <aside class="proj-section" style="position:sticky;top:1rem;align-self:start;">
                            <div class="proj-section__head">
                                <div>
                                    <h3 class="proj-section__title">"Add to history"</h3>
                                    <p class="proj-section__hint">"Backfill this unit’s timeline"</p>
                                </div>
                            </div>
                            <div class="unit-actions" style="flex-direction:column;align-items:stretch;">
                                <a class="folio-btn folio-btn--primary press" href=hist_lease_href.clone()>
                                    "Historical lease"
                                </a>
                                <a
                                    class="folio-btn folio-btn--ghost press"
                                    href=format!(
                                        "{}?asset_id={}",
                                        FolioRoute::LandlordLeaseCreate.path(),
                                        unit_id
                                    )
                                >
                                    "New live lease"
                                </a>
                                <a class="folio-btn folio-btn--ghost press" href=hist_pay_href.clone()>
                                    "Payment history"
                                </a>
                                <a class="folio-btn folio-btn--ghost press" href=move || log_paid_href.get()>
                                    "Maintenance history"
                                </a>
                                <a class="folio-btn folio-btn--ghost press" href=FolioRoute::LandlordVault.path()>
                                    "Digital vault"
                                </a>
                            </div>
                        </aside>
                    </div>

                    <section class="proj-section" style="margin-top:1.5rem;border-top:1px solid #e5e7eb;padding-top:1.25rem;">
                        <div class="proj-section__head">
                            <div>
                                <h3 class="proj-section__title" style="color:#b91c1c;">"Danger zone"</h3>
                                <p class="proj-section__hint">
                                    "Archive removes this unit from active portfolio views. Type DELETE to confirm."
                                </p>
                            </div>
                        </div>

                        {move || if archived_ok.get() {
                            view! {
                                <p style="color:#15803d;font-size:0.875rem;">"Unit archived."</p>
                            }.into_any()
                        } else {
                            view! {
                                <div style="display:flex;flex-direction:column;gap:0.75rem;max-width:28rem;">
                                    <input
                                        type="text"
                                        class="form-input"
                                        placeholder="Type DELETE"
                                        prop:value=move || archive_confirm.get()
                                        on:input=move |ev| archive_confirm.set(event_target_value(&ev))
                                    />
                                    <div class="unit-actions">
                                        <button
                                            type="button"
                                            class="folio-btn folio-btn--primary press"
                                            style="background:#b91c1c;border-color:#b91c1c;"
                                            disabled=move || {
                                                archive_pending.get()
                                                    || archive_confirm.get().trim() != "DELETE"
                                            }
                                            on:click=on_archive
                                        >
                                            {move || if archive_pending.get() { "Archiving…" } else { "Archive unit" }}
                                        </button>
                                    </div>
                                    {move || archive_err.get().map(|e| view! {
                                        <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                                    })}
                                    {move || {
                                        let blockers = archive_blockers.get();
                                        if blockers.is_empty() {
                                            ().into_any()
                                        } else {
                                            view! {
                                                <ul style="margin:0;padding-left:1.1rem;font-size:0.875rem;color:#7f1d1d;">
                                                    {blockers.into_iter().map(|b| {
                                                        view! {
                                                            <li>{format!("{} — {}", b.code.replace('_', " "), b.message)}</li>
                                                        }
                                                    }).collect_view()}
                                                </ul>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            }.into_any()
                        }}
                    </section>
                </div>
            </Show>

            <Show when=move || show_add_space.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:24rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Add space"</h3>
                            <button class="modal-close" on:click=move |_| show_add_space.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body">
                            <label class="form-label">
                                "Name"
                                <input
                                    class="form-input"
                                    type="text"
                                    placeholder="Kitchen"
                                    prop:value=move || new_space_name.get()
                                    on:input=move |ev| new_space_name.set(event_target_value(&ev))
                                />
                            </label>
                            {move || add_space_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add_space.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                disabled=move || add_space_pending.get()
                                on:click=move |_| {
                                    let name = new_space_name.get().trim().to_string();
                                    if name.is_empty() {
                                        add_space_err.set(Some("Name is required.".into()));
                                        return;
                                    }
                                    add_space_pending.set(true);
                                    spawn_local(async move {
                                        match create_child_asset(unit_id, name, "condo".into()).await {
                                            Ok(_) => {
                                                new_space_name.set(String::new());
                                                show_add_space.set(false);
                                                spaces_refresh.update(|n| *n += 1);
                                                add_space_pending.set(false);
                                            }
                                            Err(e) => {
                                                add_space_err.set(Some(e.to_string()));
                                                add_space_pending.set(false);
                                            }
                                        }
                                    });
                                }
                            >
                                "Create"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_add_occupant.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:24rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Add occupant"</h3>
                            <button class="modal-close" on:click=move |_| show_add_occupant.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-3">
                            <label class="form-label">
                                "Full name"
                                <input
                                    class="form-input"
                                    type="text"
                                    prop:value=move || occ_name.get()
                                    on:input=move |ev| occ_name.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="form-label">
                                "Relationship"
                                <input
                                    class="form-input"
                                    type="text"
                                    prop:value=move || occ_rel.get()
                                    on:input=move |ev| occ_rel.set(event_target_value(&ev))
                                />
                            </label>
                            {move || occ_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add_occupant.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                disabled=move || occ_pending.get()
                                on:click=move |_| {
                                    let Some(lid) = unit_lease.get().map(|l| l.id) else {
                                        occ_err.set(Some("No active lease.".into()));
                                        return;
                                    };
                                    let name = occ_name.get().trim().to_string();
                                    if name.is_empty() {
                                        occ_err.set(Some("Name is required.".into()));
                                        return;
                                    }
                                    let rel = occ_rel.get();
                                    occ_pending.set(true);
                                    spawn_local(async move {
                                        match hh_add_occupant(lid, name, rel, false, None).await {
                                            Ok(()) => {
                                                occ_name.set(String::new());
                                                show_add_occupant.set(false);
                                                occ_refresh.update(|n| *n += 1);
                                                occ_pending.set(false);
                                            }
                                            Err(e) => {
                                                occ_err.set(Some(e.to_string()));
                                                occ_pending.set(false);
                                            }
                                        }
                                    });
                                }
                            >
                                "Add"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_add_vehicle.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:24rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Register vehicle"</h3>
                            <button class="modal-close" on:click=move |_| show_add_vehicle.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-3">
                            <label class="form-label">"Make"
                                <input class="form-input" type="text" prop:value=move || veh_make.get()
                                    on:input=move |ev| veh_make.set(event_target_value(&ev))/>
                            </label>
                            <label class="form-label">"Model"
                                <input class="form-input" type="text" prop:value=move || veh_model.get()
                                    on:input=move |ev| veh_model.set(event_target_value(&ev))/>
                            </label>
                            <label class="form-label">"Year"
                                <input class="form-input" type="text" prop:value=move || veh_year.get()
                                    on:input=move |ev| veh_year.set(event_target_value(&ev))/>
                            </label>
                            <label class="form-label">"Color"
                                <input class="form-input" type="text" prop:value=move || veh_color.get()
                                    on:input=move |ev| veh_color.set(event_target_value(&ev))/>
                            </label>
                            <label class="form-label">"Plate"
                                <input class="form-input" type="text" prop:value=move || veh_plate.get()
                                    on:input=move |ev| veh_plate.set(event_target_value(&ev))/>
                            </label>
                            <label class="form-label">"State"
                                <input class="form-input" type="text" prop:value=move || veh_state.get()
                                    on:input=move |ev| veh_state.set(event_target_value(&ev))/>
                            </label>
                            {move || veh_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add_vehicle.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                disabled=move || veh_pending.get()
                                on:click=move |_| {
                                    let Some(lid) = unit_lease.get().map(|l| l.id) else {
                                        veh_err.set(Some("No active lease.".into()));
                                        return;
                                    };
                                    let year = match veh_year.get().trim().parse::<i32>() {
                                        Ok(y) => y,
                                        Err(_) => {
                                            veh_err.set(Some("Invalid year.".into()));
                                            return;
                                        }
                                    };
                                    veh_pending.set(true);
                                    let make = veh_make.get();
                                    let model = veh_model.get();
                                    let color = veh_color.get();
                                    let plate = veh_plate.get();
                                    let state = veh_state.get();
                                    spawn_local(async move {
                                        match hh_add_vehicle(
                                            lid, make, model, year, color, plate, state, None,
                                        )
                                        .await
                                        {
                                            Ok(()) => {
                                                show_add_vehicle.set(false);
                                                veh_refresh.update(|n| *n += 1);
                                                veh_pending.set(false);
                                            }
                                            Err(e) => {
                                                veh_err.set(Some(e.to_string()));
                                                veh_pending.set(false);
                                            }
                                        }
                                    });
                                }
                            >
                                "Register"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
