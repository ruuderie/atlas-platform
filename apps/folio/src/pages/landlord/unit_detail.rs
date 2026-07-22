//! First-class unit workspace — `/l/assets/:id` when the asset is a unit child.
//!
//! Not a building tab: own breadcrumb + UnitTabBar + lease/WO/spaces/history.

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::photo_media_card::{PhotoEntityKind, PhotoMediaCard};
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::digital_vault::{
    is_vault_photo_category, list_entity_vault_docs,
};
use crate::pages::landlord::property_documents::get_property_documents;
use crate::pages::landlord::asset_api::{
    archive_folio_asset as archive_unit_asset, create_child_asset, get_asset_children,
    get_asset_for_dispatch, ArchiveBlockerDto, AssetDetailDto,
};
use crate::pages::landlord::lease_detail::{
    get_lease_occupants, get_lease_vehicles, OccupantRecord, VehicleRecord,
};
use crate::pages::landlord::leases::{
    activate_lease, create_occupancy, list_leases, LeaseStatus, LeaseSummary,
};
use crate::pages::landlord::ledger::list_ledger_entries;
use crate::pages::landlord::maintenance_queue::{
    list_maintenance_tickets, CaseStatus, MaintenanceSummary,
};
use crate::pages::tenant::household::{
    hh_add_occupant, hh_add_vehicle, parse_relationship_select, vehicle_year_options,
    AdultRelationship, MinorRelationship, COMMON_VEHICLE_MAKES, US_STATES, VEHICLE_COLORS,
};
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
    let str_mode = asset.str_eligible;
    let unit_address_line = {
        let mut parts = Vec::new();
        if let Some(a) = asset.address_line_1.as_ref().filter(|s| !s.is_empty()) {
            parts.push(a.clone());
        }
        let city_st = [asset.city.clone(), asset.state_province.clone()]
            .into_iter()
            .flatten()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        if !city_st.is_empty() {
            parts.push(city_st);
        }
        parts.join(" · ")
    };
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

    let photos_refresh = RwSignal::new(0u32);
    let unit_photos = Resource::new(
        move || (unit_id, photos_refresh.get()),
        |(id, _)| async move {
            // Prefer property-documents compose (unit + spaces); fall back to vault entity list.
            let rows = get_property_documents(id, None).await.unwrap_or_default();
            let n = rows
                .iter()
                .filter(|r| {
                    r.kind == crate::pages::landlord::property_documents::PropertyDocumentKind::Vault
                        && is_vault_photo_category(&r.category)
                })
                .count();
            if n > 0 {
                return n;
            }
            list_entity_vault_docs("atlas_assets".into(), id)
                .await
                .unwrap_or_default()
                .into_iter()
                .filter(|d| is_vault_photo_category(&d.document_category))
                .count()
        },
    );

    let leases_refresh = RwSignal::new(0u32);
    let leases = Resource::new(
        move || leases_refresh.get(),
        |_| async move { list_leases().await },
    );
    let tickets = Resource::new(|| (), |_| async move { list_maintenance_tickets().await });
    let ledger = Resource::new(|| (), |_| async move { list_ledger_entries().await });

    let show_message = RwSignal::new(false);
    let show_archive = RwSignal::new(false);

    let show_add_space = RwSignal::new(false);
    let new_space_name = RwSignal::new(String::new());
    let add_space_err = RwSignal::new(None::<String>);
    let add_space_pending = RwSignal::new(false);

    let show_add_occupant = RwSignal::new(false);
    let occ_name = RwSignal::new(String::new());
    let occ_rel = RwSignal::new("adult:roommate".to_string());
    let occ_dob = RwSignal::new(String::new());
    let occ_err = RwSignal::new(None::<String>);
    let occ_pending = RwSignal::new(false);
    let occ_refresh = RwSignal::new(0u32);

    let show_add_vehicle = RwSignal::new(false);
    let veh_make = RwSignal::new("Toyota".to_string());
    let veh_make_other = RwSignal::new(String::new());
    let veh_model = RwSignal::new(String::new());
    let veh_year =
        RwSignal::new(vehicle_year_options().first().copied().unwrap_or(2020).to_string());
    let veh_color = RwSignal::new("Silver".to_string());
    let veh_plate = RwSignal::new(String::new());
    let veh_state = RwSignal::new("FL".to_string());
    let veh_err = RwSignal::new(None::<String>);
    let veh_pending = RwSignal::new(false);
    let veh_refresh = RwSignal::new(0u32);

    // Occupying lease: active > pending > draft (not only active).
    let unit_lease = Signal::derive(move || {
        leases.get().and_then(|r| r.ok()).and_then(|items| {
            let mut candidates: Vec<LeaseSummary> = items
                .into_iter()
                .filter(|l| l.asset_id == Some(unit_id) && l.is_occupying())
                .collect();
            candidates.sort_by_key(|l| match LeaseStatus::from_str(&l.status) {
                LeaseStatus::Active => 0,
                LeaseStatus::Pending => 1,
                LeaseStatus::Draft => 2,
                _ => 9,
            });
            candidates.into_iter().next()
        })
    });

    let occupied = Signal::derive(move || unit_lease.get().is_some());
    let seeking = Signal::derive(move || !occupied.get());
    let str_listing = asset.str_listing_active;

    let show_add_tenant = RwSignal::new(false);
    let add_tenant_name = RwSignal::new(String::new());
    let add_tenant_phone = RwSignal::new(String::new());
    let add_tenant_email = RwSignal::new(String::new());
    let add_tenant_move_in = RwSignal::new(String::new());
    let add_tenant_err = RwSignal::new(None::<String>);
    let add_tenant_pending = RwSignal::new(false);

    let show_attach = RwSignal::new(false);
    let attach_rent = RwSignal::new(String::new());
    let attach_currency = RwSignal::new("USD".to_string());
    let attach_guarantee = RwSignal::new("security_deposit".to_string());
    let attach_start = RwSignal::new(String::new());
    let attach_end = RwSignal::new(String::new());
    let attach_auto_renew = RwSignal::new(false);
    let attach_err = RwSignal::new(None::<String>);
    let attach_pending = RwSignal::new(false);

    let lease_id_for_occupants = Signal::derive(move || {
        unit_lease.get().and_then(|l| {
            if LeaseStatus::from_str(&l.status) == LeaseStatus::Active {
                Some(l.id.to_string())
            } else {
                None
            }
        })
    });

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
                        show_archive.set(false);
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
                subtitle=Signal::derive({
                    let addr = unit_address_line.clone();
                    move || {
                        if addr.is_empty() {
                            "Unit".to_string()
                        } else {
                            addr.clone()
                        }
                    }
                })
            >
                <button
                    type="button"
                    class="folio-btn folio-btn--ghost press"
                    on:click=move |_| show_message.set(true)
                >
                    {move || {
                        if occupied.get() {
                            if str_mode {
                                "Message guest"
                            } else {
                                "Message household"
                            }
                        } else {
                            "Message"
                        }
                    }}
                </button>
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

            <div class="folio-tab-bar" role="tablist">
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
                                        "folio-tab folio-tab--active"
                                    } else {
                                        "folio-tab"
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
                        {move || {
                            let (label, tone) = if occupied.get() {
                                ("Occupied".to_string(), StatusPillTone::Ok)
                            } else {
                                ("Vacant".to_string(), StatusPillTone::Warn)
                            };
                            view! { <StatusPill label=label tone=tone /> }
                        }}
                        <Show when=move || seeking.get()>
                            <StatusPill label="Seeking lease".to_string() tone=StatusPillTone::Info />
                        </Show>
                        {move || {
                            if str_listing {
                                view! {
                                    <StatusPill label="STR listed".to_string() tone=StatusPillTone::Ok />
                                }.into_any()
                            } else if str_mode {
                                view! {
                                    <StatusPill label="STR open".to_string() tone=StatusPillTone::Info />
                                }.into_any()
                            } else {
                                ().into_any()
                            }
                        }}
                    </div>

                    {
                        let gallery_href = Signal::derive(move || {
                            format!("/l/assets/{unit_id}/documents?kind=photo")
                        });
                        let photo_count = Signal::derive(move || unit_photos.get().unwrap_or(0));
                        view! {
                            <div class="hub-media-row" style="margin-bottom:1rem;">
                                <PhotoMediaCard
                                    entity_kind=PhotoEntityKind::Asset
                                    entity_id=unit_id
                                    gallery_href=gallery_href
                                    photo_count=photo_count
                                    has_cover=Signal::derive(|| false)
                                    parent_asset_id=parent_id.unwrap_or(Uuid::nil())
                                    empty_label="No photos yet".to_string()
                                    on_uploaded=Callback::new(move |_| {
                                        photos_refresh.update(|n| *n += 1);
                                    })
                                />
                            </div>
                        }
                    }

                    <section class="unit-availability">
                        <h3 class="unit-availability__title">"Availability"</h3>
                        <div class="unit-availability__grid">
                            <div>
                                <p class="unit-availability__label">"Occupancy"</p>
                                <p class="unit-availability__value">
                                    {move || if occupied.get() { "Occupied" } else { "Vacant" }}
                                </p>
                            </div>
                            <div>
                                <p class="unit-availability__label">"Lease applications"</p>
                                <p class="unit-availability__value">
                                    {move || if seeking.get() { "Seeking" } else { "Closed" }}
                                </p>
                            </div>
                            <div>
                                <p class="unit-availability__label">"Short-term"</p>
                                <p class="unit-availability__value">
                                    {if str_listing {
                                        "Listed"
                                    } else if str_mode {
                                        "Open"
                                    } else {
                                        "Off"
                                    }}
                                </p>
                            </div>
                        </div>
                        <div class="unit-availability__actions">
                            {move || if seeking.get() {
                                view! {
                                    <button
                                        type="button"
                                        class="folio-btn folio-btn--primary press"
                                        on:click=move |_| {
                                            add_tenant_err.set(None);
                                            show_add_tenant.set(true);
                                        }
                                    >
                                        "Add tenant"
                                    </button>
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
                                    <a
                                        class="folio-btn folio-btn--ghost press"
                                        href=FolioRoute::LandlordApplications.path()
                                    >
                                        "Applications"
                                    </a>
                                }.into_any()
                            } else {
                                let draft = unit_lease.get().filter(|l| {
                                    LeaseStatus::from_str(&l.status) == LeaseStatus::Draft
                                });
                                view! {
                                    {draft.map(|l| {
                                        let _ = l;
                                        view! {
                                            <button
                                                type="button"
                                                class="folio-btn folio-btn--primary press"
                                                on:click=move |_| {
                                                    attach_err.set(None);
                                                    show_attach.set(true);
                                                }
                                            >
                                                "Attach lease"
                                            </button>
                                        }
                                    })}
                                    {unit_lease.get().map(|l| {
                                        let href = FolioRoute::LandlordLeaseDetail
                                            .path()
                                            .replace(":id", &l.id.to_string());
                                        view! {
                                            <a class="folio-btn folio-btn--ghost press" href=href>"Open lease"</a>
                                        }
                                    })}
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
                                }.into_any()
                            }}
                        </div>
                    </section>

                    <div class="unit-actions">
                        <a class="folio-btn folio-btn--primary press" href=move || wo_new.get()>"Create WO"</a>
                        <a
                            class="folio-btn folio-btn--ghost press"
                            href=move || log_paid_href.get()
                        >
                            "Log paid"
                        </a>
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
                                let status = LeaseStatus::from_str(&l.status);
                                let rent = l
                                    .monthly_rent_cents
                                    .map(|c| format!("${:.0}/mo", c as f64 / 100.0))
                                    .unwrap_or_else(|| "—".into());
                                let tenant = l.tenant_display_label();
                                let dates = match (l.start_date, l.end_date) {
                                    (Some(s), Some(e)) => format!("{s} → {e}"),
                                    (Some(s), None) => format!("From {s}"),
                                    _ => "Dates not set".into(),
                                };
                                view! {
                                    <div class="hub-activity-rail__row">
                                        <StatusPill label=status.as_str().to_string() tone=StatusPillTone::Ok/>
                                        <div class="hub-activity-rail__body">
                                            <p class="hub-activity-rail__row-title">{tenant}</p>
                                            <p class="hub-activity-rail__row-meta">{format!("{rent} · {dates}")}</p>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            None => view! {
                                <div class="folio-empty--compact">
                                    "No occupying lease on this unit."
                                </div>
                            }.into_any(),
                        }}
                    </section>

                    <section class="proj-section">
                        <div class="proj-section__head">
                            <h3 class="proj-section__title">"Open work orders"</h3>
                            <button
                                type="button"
                                class="folio-btn folio-btn--ghost folio-btn--sm press"
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
                                <p class="folio-empty__heading">"No occupying lease"</p>
                                <p class="folio-empty__sub">
                                    "Add a tenant or create a lease to manage household here."
                                </p>
                                <div class="unit-actions" style="justify-content:center;">
                                    <button
                                        type="button"
                                        class="folio-btn folio-btn--primary press"
                                        on:click=move |_| show_add_tenant.set(true)
                                    >
                                        "Add tenant"
                                    </button>
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
                                </div>
                            </div>
                        }.into_any(),
                        Some(l) => {
                            let href = FolioRoute::LandlordLeaseDetail
                                .path()
                                .replace(":id", &l.id.to_string());
                            let status = LeaseStatus::from_str(&l.status);
                            let rent = l
                                .monthly_rent_cents
                                .map(|c| format!("${:.0}/mo", c as f64 / 100.0))
                                .unwrap_or_else(|| "—".into());
                            let tenant = l.tenant_display_label();
                            let is_draft = status == LeaseStatus::Draft;
                            view! {
                                <section class="proj-section">
                                    <div class="proj-section__head">
                                        <h3 class="proj-section__title">"Lease"</h3>
                                        <a class="folio-btn folio-btn--ghost folio-btn--sm press" href=href>"Open detail"</a>
                                    </div>
                                    <div class="proj-section__body proj-section__body--stack">
                                        <div>
                                            <p class="hub-activity-rail__row-title">{tenant}</p>
                                            <p class="hub-activity-rail__row-meta">
                                                {format!("{} · {rent}", status.as_str())}
                                            </p>
                                        </div>
                                        <Show when=move || is_draft>
                                            <button
                                                type="button"
                                                class="folio-btn folio-btn--primary press"
                                                on:click=move |_| show_attach.set(true)
                                            >
                                                "Attach lease"
                                            </button>
                                        </Show>
                                    </div>
                                </section>
                                <section class="proj-section">
                                    <div class="proj-section__head">
                                        <h3 class="proj-section__title">"Household"</h3>
                                        <button
                                            type="button"
                                            class="folio-btn folio-btn--primary folio-btn--sm press"
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
                                                            class="folio-btn folio-btn--primary folio-btn--sm press"
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
                                                            let name = o.full_name.clone();
                                                            let rel = o.relationship.replace('_', " ");
                                                            let profile_href = lid.map(|lease_id| {
                                                                FolioRoute::LandlordOccupantProfile
                                                                    .path()
                                                                    .replace(":lease_id", &lease_id.to_string())
                                                                    .replace(":entry_id", &oid.to_string())
                                                            });
                                                            view! {
                                                                <div class="hub-activity-rail__row">
                                                                    <StatusPill
                                                                        label=if o.is_minor {
                                                                            "Minor".to_string()
                                                                        } else {
                                                                            "Adult".to_string()
                                                                        }
                                                                        tone=StatusPillTone::Info
                                                                    />
                                                                    <div class="hub-activity-rail__body">
                                                                        {match profile_href.clone() {
                                                                            Some(href) => view! {
                                                                                <a class="hub-activity-rail__row-title press" href=href style="text-decoration:none;color:inherit;">
                                                                                    {name.clone()}
                                                                                </a>
                                                                            }.into_any(),
                                                                            None => view! {
                                                                                <p class="hub-activity-rail__row-title">{name.clone()}</p>
                                                                            }.into_any(),
                                                                        }}
                                                                        <p class="hub-activity-rail__row-meta">{rel}</p>
                                                                    </div>
                                                                    <button
                                                                        type="button"
                                                                        class="folio-btn folio-btn--ghost folio-btn--sm press"
                                                                        style="border-color:#fecaca;color:#991b1b;"
                                                                        on:click=move |_| {
                                                                            let Some(lease_id) = lid else { return; };
                                                                            let msg = format!("Depart {name} from this lease?");
                                                                            let confirmed = web_sys::window()
                                                                                .and_then(|w| w.confirm_with_message(&msg).ok())
                                                                                .unwrap_or(false);
                                                                            if !confirmed {
                                                                                return;
                                                                            }
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
                                            class="folio-btn folio-btn--ghost folio-btn--sm press"
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
                                                    let status = LeaseStatus::from_str(&l.status);
                                                    let tenant = l.tenant_display_label();
                                                    let rent = l.monthly_rent_cents
                                                        .map(|c| format!("${:.0}/mo", c as f64 / 100.0))
                                                        .unwrap_or_else(|| "—".into());
                                                    let dates = format!(
                                                        "{} → {}",
                                                        l.start_date.map(|d| d.to_string()).unwrap_or_else(|| "—".into()),
                                                        l.end_date.map(|d| d.to_string()).unwrap_or_else(|| "—".into())
                                                    );
                                                    let meta = format!("{rent} · {dates}");
                                                    view! {
                                                        <a class="hub-activity-rail__row press" href=href>
                                                            <StatusPill
                                                                label=status.as_str().to_string()
                                                                tone=if status == LeaseStatus::Draft {
                                                                    StatusPillTone::Warn
                                                                } else {
                                                                    StatusPillTone::Info
                                                                }
                                                            />
                                                            <div class="hub-activity-rail__body">
                                                                <p class="hub-activity-rail__row-title">{tenant}</p>
                                                                <p class="hub-activity-rail__row-meta">{meta}</p>
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

                        <aside class="proj-section unit-history-aside">
                            <div class="proj-section__head">
                                <div>
                                    <h3 class="proj-section__title">"Add to history"</h3>
                                    <p class="proj-section__hint">"Backfill this unit’s timeline"</p>
                                </div>
                            </div>
                            <div class="unit-actions unit-actions--stack">
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

                    <div class="hub-archive-foot">
                        {move || if archived_ok.get() {
                            view! {
                                <p class="hub-archive-foot__ok">"Unit archived."</p>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    type="button"
                                    class="hub-archive-foot__link"
                                    on:click=move |_| {
                                        archive_err.set(None);
                                        archive_blockers.set(vec![]);
                                        archive_confirm.set(String::new());
                                        show_archive.set(true);
                                    }
                                >
                                    "Archive unit…"
                                </button>
                            }.into_any()
                        }}
                    </div>
                </div>
            </Show>

            <Show when=move || show_message.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">
                                {if str_mode { "Message guest" } else { "Message household" }}
                            </h3>
                            <button
                                class="modal-close"
                                on:click=move |_| show_message.set(false)
                            >
                                <span class="material-symbols-outlined">"close"</span>
                            </button>
                        </div>
                        <div class="modal-body">
                            <p class="proj-section__hint">
                                "Starting a message thread from the unit is not available yet. Use Messages in the nav for existing conversations."
                            </p>
                        </div>
                        <div class="modal-footer">
                            <button
                                class="folio-btn folio-btn--ghost"
                                on:click=move |_| show_message.set(false)
                            >
                                "Close"
                            </button>
                            <a
                                class="folio-btn folio-btn--primary press"
                                href=FolioRoute::LandlordCommunications.path()
                                on:click=move |_| show_message.set(false)
                            >
                                "Go to Messages"
                            </a>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_archive.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Archive unit"</h3>
                            <button
                                class="modal-close"
                                on:click=move |_| show_archive.set(false)
                            >
                                <span class="material-symbols-outlined">"close"</span>
                            </button>
                        </div>
                        <div class="modal-body space-y-4">
                            <p class="proj-section__hint">
                                "Archive removes this unit from active portfolio views. Active leases and open work orders may block archive until resolved. Type DELETE to confirm."
                            </p>
                            <label class="folio-field__label">
                                "Type DELETE"
                                <input
                                    class="folio-input"
                                    type="text"
                                    autocomplete="off"
                                    prop:value=move || archive_confirm.get()
                                    on:input=move |ev| archive_confirm.set(event_target_value(&ev))
                                />
                            </label>
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
                                            <li>{format!("{} — {}", b.code.replace('_', " "), b.message)}</li>
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }}
                        </div>
                        <div class="modal-footer">
                            <button
                                class="folio-btn folio-btn--ghost"
                                on:click=move |_| show_archive.set(false)
                            >
                                "Cancel"
                            </button>
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary"
                                style="background:#b91c1c;border-color:#b91c1c;"
                                prop:disabled=move || {
                                    archive_pending.get()
                                        || archive_confirm.get().trim() != "DELETE"
                                }
                                on:click=on_archive
                            >
                                {move || if archive_pending.get() { "Archiving…" } else { "Archive unit" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_add_space.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:24rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Add space"</h3>
                            <button class="modal-close" on:click=move |_| show_add_space.set(false)><span class="material-symbols-outlined">"close"</span></button>
                        </div>
                        <div class="modal-body">
                            <label class="folio-field__label">
                                "Name"
                                <input
                                    class="folio-input"
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
                            <button class="folio-btn folio-btn--ghost" on:click=move |_| show_add_space.set(false)>"Cancel"</button>
                            <button
                                class="folio-btn folio-btn--primary"
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
                            <button class="modal-close" on:click=move |_| show_add_occupant.set(false)><span class="material-symbols-outlined">"close"</span></button>
                        </div>
                        <div class="modal-body space-y-3">
                            <div class="folio-field">
                                <label class="folio-field__label">"Full name"</label>
                                <input
                                    class="folio-input"
                                    type="text"
                                    prop:value=move || occ_name.get()
                                    on:input=move |ev| occ_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Relationship"</label>
                                <select
                                    class="folio-select"
                                    prop:value=move || occ_rel.get()
                                    on:change=move |ev| occ_rel.set(event_target_value(&ev))
                                >
                                    <optgroup label="Adult">
                                        {AdultRelationship::ALL.iter().map(|r| {
                                            let v = format!("adult:{}", r.as_str());
                                            let label = r.label();
                                            view! { <option value=v>{label}</option> }
                                        }).collect_view()}
                                    </optgroup>
                                    <optgroup label="Minor">
                                        {MinorRelationship::ALL.iter().map(|r| {
                                            let v = format!("minor:{}", r.as_str());
                                            let label = r.label();
                                            view! { <option value=v>{label}</option> }
                                        }).collect_view()}
                                    </optgroup>
                                </select>
                            </div>
                            <Show when=move || occ_rel.get().starts_with("minor:")>
                                <div class="folio-field">
                                    <label class="folio-field__label">"Date of birth"</label>
                                    <input
                                        class="folio-input"
                                        type="date"
                                        prop:value=move || occ_dob.get()
                                        on:input=move |ev| occ_dob.set(event_target_value(&ev))
                                    />
                                </div>
                            </Show>
                            {move || occ_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="folio-btn folio-btn--ghost" on:click=move |_| show_add_occupant.set(false)>"Cancel"</button>
                            <button
                                class="folio-btn folio-btn--primary"
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
                                    let Some((is_minor, rel)) = parse_relationship_select(&occ_rel.get()) else {
                                        occ_err.set(Some("Pick a valid relationship.".into()));
                                        return;
                                    };
                                    let dob = if is_minor {
                                        let d = occ_dob.get().trim().to_string();
                                        if d.is_empty() {
                                            occ_err.set(Some("Date of birth is required for minors.".into()));
                                            return;
                                        }
                                        Some(d)
                                    } else {
                                        None
                                    };
                                    occ_pending.set(true);
                                    spawn_local(async move {
                                        match hh_add_occupant(lid, name, rel, is_minor, dob).await {
                                            Ok(()) => {
                                                occ_name.set(String::new());
                                                occ_rel.set("adult:roommate".into());
                                                occ_dob.set(String::new());
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

            <Show when=move || show_add_tenant.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Add tenant"</h3>
                            <button class="modal-close" on:click=move |_| show_add_tenant.set(false)>
                                <span class="material-symbols-outlined">"close"</span>
                            </button>
                        </div>
                        <div class="modal-body space-y-3">
                            <p class="proj-section__hint">
                                "Record who’s living here now. You can attach commercial lease terms later."
                            </p>
                            <label class="folio-field__label">
                                "Name"
                                <input
                                    class="folio-input"
                                    type="text"
                                    prop:value=move || add_tenant_name.get()
                                    on:input=move |ev| add_tenant_name.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="folio-field__label">
                                "Phone (optional)"
                                <input
                                    class="folio-input"
                                    type="tel"
                                    prop:value=move || add_tenant_phone.get()
                                    on:input=move |ev| add_tenant_phone.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="folio-field__label">
                                "Email (optional)"
                                <input
                                    class="folio-input"
                                    type="email"
                                    prop:value=move || add_tenant_email.get()
                                    on:input=move |ev| add_tenant_email.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="folio-field__label">
                                "Move-in date (optional)"
                                <input
                                    class="folio-input"
                                    type="date"
                                    prop:value=move || add_tenant_move_in.get()
                                    on:input=move |ev| add_tenant_move_in.set(event_target_value(&ev))
                                />
                            </label>
                            {move || add_tenant_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="folio-btn folio-btn--ghost" on:click=move |_| show_add_tenant.set(false)>
                                "Cancel"
                            </button>
                            <button
                                class="folio-btn folio-btn--primary"
                                disabled=move || add_tenant_pending.get()
                                on:click=move |_| {
                                    let name = add_tenant_name.get().trim().to_string();
                                    if name.is_empty() {
                                        add_tenant_err.set(Some("Name is required.".into()));
                                        return;
                                    }
                                    add_tenant_pending.set(true);
                                    let phone = add_tenant_phone.get();
                                    let email = add_tenant_email.get();
                                    let move_in = add_tenant_move_in.get();
                                    spawn_local(async move {
                                        match create_occupancy(
                                            unit_id,
                                            name,
                                            Some(phone).filter(|s| !s.trim().is_empty()),
                                            Some(email).filter(|s| !s.trim().is_empty()),
                                            None,
                                            Some(move_in).filter(|s| !s.trim().is_empty()),
                                        )
                                        .await
                                        {
                                            Ok(_) => {
                                                add_tenant_name.set(String::new());
                                                add_tenant_phone.set(String::new());
                                                add_tenant_email.set(String::new());
                                                add_tenant_move_in.set(String::new());
                                                show_add_tenant.set(false);
                                                leases_refresh.update(|n| *n += 1);
                                            }
                                            Err(e) => add_tenant_err.set(Some(e.to_string())),
                                        }
                                        add_tenant_pending.set(false);
                                    });
                                }
                            >
                                {move || if add_tenant_pending.get() { "Saving…" } else { "Save occupancy" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_attach.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Attach lease"</h3>
                            <button class="modal-close" on:click=move |_| show_attach.set(false)>
                                <span class="material-symbols-outlined">"close"</span>
                            </button>
                        </div>
                        <div class="modal-body space-y-3">
                            <p class="proj-section__hint">
                                "Add rent and terms to activate the draft occupancy."
                            </p>
                            <label class="folio-field__label">
                                "Monthly rent"
                                <input
                                    class="folio-input"
                                    type="text"
                                    inputmode="decimal"
                                    placeholder="1850"
                                    prop:value=move || attach_rent.get()
                                    on:input=move |ev| attach_rent.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="folio-field__label">
                                "Currency"
                                <select
                                    class="folio-input"
                                    prop:value=move || attach_currency.get()
                                    on:change=move |ev| attach_currency.set(event_target_value(&ev))
                                >
                                    <option value="USD">"USD"</option>
                                    <option value="BRL">"BRL"</option>
                                </select>
                            </label>
                            <label class="folio-field__label">
                                "Guarantee"
                                <select
                                    class="folio-input"
                                    prop:value=move || attach_guarantee.get()
                                    on:change=move |ev| attach_guarantee.set(event_target_value(&ev))
                                >
                                    <option value="security_deposit">"Security deposit"</option>
                                    <option value="guarantor">"Guarantor"</option>
                                    <option value="none">"None"</option>
                                </select>
                            </label>
                            <label class="folio-field__label">
                                "Start date"
                                <input
                                    class="folio-input"
                                    type="date"
                                    prop:value=move || attach_start.get()
                                    on:input=move |ev| attach_start.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="folio-field__label">
                                "End date (optional)"
                                <input
                                    class="folio-input"
                                    type="date"
                                    prop:value=move || attach_end.get()
                                    on:input=move |ev| attach_end.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="folio-field folio-field--check">
                                <input
                                    type="checkbox"
                                    prop:checked=move || attach_auto_renew.get()
                                    on:change=move |ev| {
                                        let el = event_target::<web_sys::HtmlInputElement>(&ev);
                                        attach_auto_renew.set(el.checked());
                                    }
                                />
                                <span>"Auto-renew"</span>
                            </label>
                            {move || attach_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="folio-btn folio-btn--ghost" on:click=move |_| show_attach.set(false)>
                                "Cancel"
                            </button>
                            <button
                                class="folio-btn folio-btn--primary"
                                disabled=move || attach_pending.get()
                                on:click=move |_| {
                                    let Some(lease) = unit_lease.get() else {
                                        attach_err.set(Some("No draft occupancy.".into()));
                                        return;
                                    };
                                    let rent_cents = match attach_rent.get().trim().parse::<f64>() {
                                        Ok(v) if v >= 0.0 => (v * 100.0).round() as i64,
                                        _ => {
                                            attach_err.set(Some("Enter monthly rent.".into()));
                                            return;
                                        }
                                    };
                                    let start = attach_start.get();
                                    if start.is_empty() {
                                        attach_err.set(Some("Start date is required.".into()));
                                        return;
                                    }
                                    let end = {
                                        let e = attach_end.get();
                                        if e.is_empty() { None } else { Some(e) }
                                    };
                                    attach_pending.set(true);
                                    let currency = attach_currency.get();
                                    let guarantee = attach_guarantee.get();
                                    let auto = attach_auto_renew.get();
                                    spawn_local(async move {
                                        match activate_lease(
                                            lease.id,
                                            rent_cents,
                                            currency,
                                            guarantee,
                                            start,
                                            end,
                                            auto,
                                            None,
                                        )
                                        .await
                                        {
                                            Ok(()) => {
                                                show_attach.set(false);
                                                leases_refresh.update(|n| *n += 1);
                                            }
                                            Err(e) => attach_err.set(Some(e.to_string())),
                                        }
                                        attach_pending.set(false);
                                    });
                                }
                            >
                                {move || if attach_pending.get() { "Activating…" } else { "Activate lease" }}
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
                            <button class="modal-close" on:click=move |_| show_add_vehicle.set(false)><span class="material-symbols-outlined">"close"</span></button>
                        </div>
                        <div class="modal-body space-y-3">
                            <div class="folio-field">
                                <label class="folio-field__label">"Make"</label>
                                <select
                                    class="folio-select"
                                    prop:value=move || veh_make.get()
                                    on:change=move |ev| veh_make.set(event_target_value(&ev))
                                >
                                    {COMMON_VEHICLE_MAKES.iter().map(|m| {
                                        let m = *m;
                                        view! { <option value=m>{m}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <Show when=move || veh_make.get() == "Other">
                                <div class="folio-field">
                                    <label class="folio-field__label">"Make (other)"</label>
                                    <input
                                        class="folio-input"
                                        type="text"
                                        prop:value=move || veh_make_other.get()
                                        on:input=move |ev| veh_make_other.set(event_target_value(&ev))
                                    />
                                </div>
                            </Show>
                            <div class="folio-field">
                                <label class="folio-field__label">"Model"</label>
                                <input
                                    class="folio-input"
                                    type="text"
                                    prop:value=move || veh_model.get()
                                    on:input=move |ev| veh_model.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Year"</label>
                                <select
                                    class="folio-select"
                                    prop:value=move || veh_year.get()
                                    on:change=move |ev| veh_year.set(event_target_value(&ev))
                                >
                                    {vehicle_year_options().into_iter().map(|y| {
                                        let ys = y.to_string();
                                        view! { <option value=ys.clone()>{ys.clone()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Color"</label>
                                <select
                                    class="folio-select"
                                    prop:value=move || veh_color.get()
                                    on:change=move |ev| veh_color.set(event_target_value(&ev))
                                >
                                    {VEHICLE_COLORS.iter().map(|c| {
                                        let c = *c;
                                        view! { <option value=c>{c}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Plate"</label>
                                <input
                                    class="folio-input"
                                    type="text"
                                    prop:value=move || veh_plate.get()
                                    on:input=move |ev| veh_plate.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"State"</label>
                                <select
                                    class="folio-select"
                                    prop:value=move || veh_state.get()
                                    on:change=move |ev| veh_state.set(event_target_value(&ev))
                                >
                                    {US_STATES.iter().map(|s| {
                                        let s = *s;
                                        view! { <option value=s>{s}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            {move || veh_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="folio-btn folio-btn--ghost" on:click=move |_| show_add_vehicle.set(false)>"Cancel"</button>
                            <button
                                class="folio-btn folio-btn--primary"
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
                                    let make_sel = veh_make.get();
                                    let make = if make_sel == "Other" {
                                        veh_make_other.get()
                                    } else {
                                        make_sel
                                    };
                                    if make.trim().is_empty() {
                                        veh_err.set(Some("Make is required.".into()));
                                        return;
                                    }
                                    let model = veh_model.get();
                                    let plate = veh_plate.get();
                                    if model.trim().is_empty() || plate.trim().is_empty() {
                                        veh_err.set(Some("Model and plate are required.".into()));
                                        return;
                                    }
                                    veh_pending.set(true);
                                    let color = veh_color.get();
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
