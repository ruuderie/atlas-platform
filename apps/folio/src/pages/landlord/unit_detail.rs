//! First-class unit workspace — `/l/assets/:id` when the asset is a unit child.
//!
//! Not a building tab: own breadcrumb + UnitTabBar + lease/WO/spaces sections.

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::asset_api::{
    get_asset_children, get_asset_for_dispatch, AssetChildDto, AssetDetailDto,
};
use crate::pages::landlord::lease_detail::{get_lease_occupants, OccupantRecord};
use crate::pages::landlord::leases::{list_leases, LeaseStatus};
use crate::pages::landlord::maintenance_queue::{
    list_maintenance_tickets, CaseStatus, MaintenanceSummary,
};
use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitTab {
    Overview,
    LeaseHousehold,
    Maintenance,
    Spaces,
}

impl UnitTab {
    const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::LeaseHousehold => "Lease & household",
            Self::Maintenance => "Maintenance",
            Self::Spaces => "Spaces",
        }
    }
}

#[component]
pub fn UnitDetailPage(asset: AssetDetailDto) -> impl IntoView {
    let unit_id = asset.id;
    let parent_id = asset.parent_asset_id;
    let name = asset.name.clone();
    let status = asset.status.clone();
    let str_mode = asset.str_eligible;
    let tab = RwSignal::new(UnitTab::Overview);

    let parent = Resource::new(
        move || parent_id,
        |maybe| async move {
            match maybe {
                Some(pid) => get_asset_for_dispatch(pid).await.ok(),
                None => None,
            }
        },
    );

    let spaces = Resource::new(
        move || unit_id,
        |id| async move {
            get_asset_children(id)
                .await
                .unwrap_or_default()
                .into_iter()
                .filter(|c| {
                    let t = c.asset_type.to_ascii_lowercase();
                    t.contains("space") || t.contains("kitchen") || t.contains("bath")
                })
                .collect::<Vec<AssetChildDto>>()
        },
    );

    let leases = Resource::new(|| (), |_| async move { list_leases().await });
    let tickets = Resource::new(|| (), |_| async move { list_maintenance_tickets().await });

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
        move || lease_id_for_occupants.get(),
        |maybe| async move {
            match maybe {
                Some(lid) => get_lease_occupants(lid)
                    .await
                    .map(|r| r.active)
                    .unwrap_or_default(),
                None => Vec::<OccupantRecord>::new(),
            }
        },
    );

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
    let log_paid_href = format!(
        "{}?mode=log_paid&asset_id={}",
        FolioRoute::LandlordMaintenanceNew.path(),
        unit_id
    );
    let docs_href = parent_id.map(|pid| {
        FolioRoute::LandlordAssetDocuments
            .path()
            .replace(":id", &pid.to_string())
    });

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
                            href=log_paid_href.clone()
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
                                    </div>
                                    <Suspense fallback=|| view! { <div class="folio-empty--compact">"Loading…"</div> }>
                                        {move || {
                                            let people = occupants.get().unwrap_or_default();
                                            if people.is_empty() {
                                                view! {
                                                    <div class="folio-empty--compact">"No occupants registered on this lease."</div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <For
                                                        each=move || people.clone()
                                                        key=|o| o.id
                                                        children=move |o: OccupantRecord| {
                                                            view! {
                                                                <div class="hub-activity-rail__row">
                                                                    <StatusPill label="Occupant".to_string() tone=StatusPillTone::Info/>
                                                                    <div class="hub-activity-rail__body">
                                                                        <p class="hub-activity-rail__row-title">{o.full_name.clone()}</p>
                                                                        <p class="hub-activity-rail__row-meta">{o.relationship.clone()}</p>
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
                        </div>
                        <Suspense fallback=|| view! { <div class="folio-empty--compact">"Loading…"</div> }>
                            {move || {
                                let list = spaces.get().unwrap_or_default();
                                if list.is_empty() {
                                    view! {
                                        <div class="folio-empty--compact">
                                            "No spaces yet. Add kitchens, bathrooms, and other spaces to target work orders."
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
        </div>
    }
}
