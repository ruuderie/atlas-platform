//! Property hub / unit / leaf dispatch — `/l/assets/:id`

use crate::components::activity_rail::{ActivityRail, ActivityRailItem};
use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::property_tab_bar::{PropertyTab, PropertyTabBar};
use crate::components::stat_card::StatCard;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::asset_api::{
    archive_folio_asset, create_child_asset, create_project_for_asset, geocode_asset,
    get_asset_children, get_asset_for_dispatch, get_projects_for_asset, purge_folio_asset,
    put_asset_capital, put_asset_details, ArchiveBlockerDto, AssetChildDto, AssetDetailDto,
    CapitalDto, PropertyDetailsDto,
};
use crate::pages::landlord::asset_detail::{
    get_asset_contractor, get_vendor_list, set_default_contractor, AssetContractorSummary,
    VendorListItem,
};
use crate::pages::landlord::asset_detail::AssetDetail as LeafAssetDetail;
use crate::pages::landlord::leases::{list_leases, LeaseStatus, LeaseSummary};
use crate::pages::landlord::maintenance_queue::{
    list_maintenance_tickets, CaseStatus, MaintenanceSummary,
};
use crate::pages::landlord::property_documents::{
    get_property_documents, PropertyDocumentKind, PropertyDocumentRow,
};
use crate::pages::landlord::property_systems::{get_nested_building_systems, NestedSystemDto};
use crate::pages::landlord::unit_detail::UnitDetailPage;
use crate::pages::property_owner::property_value::{
    fetch_value_history, log_property_value, ValueHistoryEntry,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_params_map};
use uuid::Uuid;

fn is_multi_unit_parent(a: &AssetDetailDto, children: &[AssetChildDto]) -> bool {
    // Hierarchy is authoritative. Onboarding stores PropertyType strings
    // (`multi_family`, …) on both parent and children — not `*property*` / `*unit*`.
    a.parent_asset_id.is_none() && !children.is_empty()
}

fn is_unit(a: &AssetDetailDto) -> bool {
    a.parent_asset_id.is_some()
}

fn asset_type_label(asset_type: &str) -> String {
    match asset_type {
        "multi_family" => "Multi-family".into(),
        "single_family" => "Single family".into(),
        "condo" => "Condo".into(),
        "townhouse" => "Townhouse".into(),
        "str" => "STR".into(),
        "commercial" => "Commercial".into(),
        other => other
            .split('_')
            .map(|w| {
                let mut c = w.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UnitOccupancy {
    Occupied,
    Vacant,
}

impl UnitOccupancy {
    fn from_unit(status: &str, has_active_lease: bool) -> Self {
        if has_active_lease || status.eq_ignore_ascii_case("occupied") {
            Self::Occupied
        } else {
            Self::Vacant
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Occupied => "Occupied",
            Self::Vacant => "Vacant",
        }
    }

    fn tone(self) -> StatusPillTone {
        match self {
            Self::Occupied => StatusPillTone::Ok,
            Self::Vacant => StatusPillTone::Warn,
        }
    }
}

fn unit_number_label(name: &str, index: usize) -> String {
    let digits: String = name.chars().filter(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() && digits.len() <= 3 {
        digits
    } else if !digits.is_empty() {
        digits[digits.len().saturating_sub(2)..].to_string()
    } else {
        (index + 1).to_string()
    }
}

fn format_cents(cents: i64) -> String {
    format!("${:.0}", cents as f64 / 100.0)
}

fn format_optional_cents(cents: Option<i64>) -> String {
    cents.map(format_cents).unwrap_or_else(|| "—".into())
}

fn format_count_label(n: Option<f64>) -> String {
    match n {
        Some(v) if (v.fract()).abs() < f64::EPSILON => format!("{:.0}", v),
        Some(v) => format!("{v}"),
        None => "—".into(),
    }
}

/// Vault docs whose category suggests a photo — API has no mime filter.
fn is_photo_like_doc(row: &PropertyDocumentRow) -> bool {
    if row.kind != PropertyDocumentKind::Vault {
        return false;
    }
    let c = row.category.to_lowercase();
    c.contains("photo")
        || c.contains("image")
        || c.contains("picture")
        || c.contains("gallery")
        || c.contains("cover")
}

fn parse_dollars_to_cents(raw: &str) -> Result<Option<i64>, String> {
    let s = raw.trim().replace(',', "").replace('$', "");
    if s.is_empty() {
        return Ok(None);
    }
    let v: f64 = s
        .parse()
        .map_err(|_| "Enter a valid dollar amount.".to_string())?;
    if v < 0.0 {
        return Err("Amount cannot be negative.".into());
    }
    Ok(Some((v * 100.0).round() as i64))
}

fn cents_to_dollars_input(cents: Option<i64>) -> String {
    cents
        .map(|c| format!("{:.0}", c as f64 / 100.0))
        .unwrap_or_default()
}

fn active_lease_for_unit<'a>(
    leases: &'a [LeaseSummary],
    unit_id: Uuid,
) -> Option<&'a LeaseSummary> {
    leases.iter().find(|l| {
        l.asset_id == Some(unit_id) && LeaseStatus::from_str(&l.status) == LeaseStatus::Active
    })
}

fn occupying_lease_for_unit<'a>(
    leases: &'a [LeaseSummary],
    unit_id: Uuid,
) -> Option<&'a LeaseSummary> {
    let mut candidates: Vec<&LeaseSummary> = leases
        .iter()
        .filter(|l| l.asset_id == Some(unit_id) && l.is_occupying())
        .collect();
    candidates.sort_by_key(|l| match LeaseStatus::from_str(&l.status) {
        LeaseStatus::Active => 0,
        LeaseStatus::Pending => 1,
        LeaseStatus::Draft => 2,
        _ => 9,
    });
    candidates.into_iter().next()
}

fn open_wo_count(tickets: &[MaintenanceSummary], scope: &[Uuid]) -> usize {
    tickets
        .iter()
        .filter(|t| t.asset_id.map(|aid| scope.contains(&aid)).unwrap_or(false))
        .filter(|t| {
            matches!(
                CaseStatus::from_str(&t.status),
                CaseStatus::Open | CaseStatus::InProgress
            )
        })
        .count()
}

fn open_wo_count_for_unit(tickets: &[MaintenanceSummary], unit_id: Uuid) -> usize {
    tickets
        .iter()
        .filter(|t| t.asset_id == Some(unit_id))
        .filter(|t| {
            matches!(
                CaseStatus::from_str(&t.status),
                CaseStatus::Open | CaseStatus::InProgress
            )
        })
        .count()
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
    let display_title = asset
        .address_line_1
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| asset.name.clone());
    let type_label = asset_type_label(&asset.asset_type);
    let street = asset.address_line_1.clone().unwrap_or_default();
    let loc = [
        asset.city.clone().unwrap_or_default(),
        asset.state_province.clone().unwrap_or_default(),
        asset.postal_code.clone().unwrap_or_default(),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join(", ");
    let map_href = format!(
        "{}?asset_id={}",
        FolioRoute::LandlordMap.path(),
        asset_id
    );
    let pin_lat = RwSignal::new(asset.latitude);
    let pin_lng = RwSignal::new(asset.longitude);
    let geocode_pending = RwSignal::new(false);
    let geocode_err = RwSignal::new(None::<String>);
    let docs_photos_href = StoredValue::new(format!(
        "{}?kind=photo",
        FolioRoute::LandlordAssetDocuments
            .path()
            .replace(":id", &asset_id.to_string())
    ));

    // Stable Leaflet canvas: init once, update marker/setView on pin change,
    // map.remove() only on dispose — never wipe innerHTML / _leaflet_id.
    Effect::new(move |_| {
        let lat = pin_lat.get();
        let lng = pin_lng.get();
        let (Some(lat), Some(lng)) = (lat, lng) else {
            return;
        };
        if lat == 0.0 && lng == 0.0 {
            return;
        }
        #[cfg(feature = "hydrate")]
        {
            let script = format!(
                r#"(function(){{
  function ensure(){{
    if(typeof L==='undefined'){{setTimeout(ensure,80);return;}}
    var el=document.getElementById('hub-mini-map');
    if(!el){{setTimeout(ensure,80);return;}}
    var st=window.__hubMiniMap;
    var live=st&&st.map&&st.map.getContainer&&st.map.getContainer()===el;
    if(live){{
      st.marker.setLatLng([{lat},{lng}]);
      st.map.setView([{lat},{lng}],15);
      setTimeout(function(){{st.map.invalidateSize();}},80);
      return;
    }}
    if(st&&st.map){{try{{st.map.remove();}}catch(e){{}}}}
    var map=L.map(el,{{center:[{lat},{lng}],zoom:15,zoomControl:false,attributionControl:false}});
    L.tileLayer('https://{{s}}.basemaps.cartocdn.com/light_all/{{z}}/{{x}}/{{y}}{{r}}.png',{{maxZoom:19,subdomains:'abcd'}}).addTo(map);
    var marker=L.circleMarker([{lat},{lng}],{{radius:8,color:'#2563eb',fillColor:'#2563eb',fillOpacity:0.9,weight:2}}).addTo(map);
    window.__hubMiniMap={{map:map,marker:marker}};
    setTimeout(function(){{map.invalidateSize();}},80);
  }}
  ensure();
}})();"#
            );
            let _ = js_sys::eval(&script);
        }
    });
    on_cleanup(|| {
        #[cfg(feature = "hydrate")]
        {
            let _ = js_sys::eval(
                r#"(function(){
  var st=window.__hubMiniMap;
  if(st&&st.map){try{st.map.remove();}catch(e){}}
  window.__hubMiniMap=null;
})();"#,
            );
        }
    });

    let units: Vec<AssetChildDto> = children;
    let unit_count = units.len();
    let mut scope = units.iter().map(|u| u.id).collect::<Vec<_>>();
    scope.push(asset_id);
    let scope_ids_sig = RwSignal::new(scope);
    let units_sig = RwSignal::new(units);
    let tab = RwSignal::new(PropertyTab::Overview);
    let units_search = RwSignal::new(String::new());

    let projects_refresh = RwSignal::new(0u32);
    let projects = Resource::new(
        move || (asset_id, projects_refresh.get()),
        |(aid, _)| async move { get_projects_for_asset(aid).await.unwrap_or_default() },
    );

    let tickets = Resource::new(|| (), |_| async move { list_maintenance_tickets().await });
    let leases = Resource::new(|| (), |_| async move { list_leases().await });
    let contractor_refresh = RwSignal::new(0u32);
    let contractor = Resource::new(
        move || (asset_id, contractor_refresh.get()),
        |(aid, _)| async move { get_asset_contractor(aid.to_string()).await.ok().flatten() },
    );
    let show_assign_contractor = RwSignal::new(false);
    let assign_contractor_pending = RwSignal::new(false);
    let assign_contractor_err = RwSignal::new(None::<String>);
    let vendors_for_assign = Resource::new(
        move || show_assign_contractor.get(),
        |open| async move {
            if !open {
                return Vec::<VendorListItem>::new();
            }
            get_vendor_list().await.unwrap_or_default()
        },
    );
    let systems = Resource::new(
        move || asset_id,
        |aid| async move { get_nested_building_systems(aid).await.unwrap_or_default() },
    );
    let documents = Resource::new(
        move || asset_id,
        |aid| async move { get_property_documents(aid, None).await.unwrap_or_default() },
    );
    let value_refresh = RwSignal::new(0u32);
    let value_history = Resource::new(
        move || (asset_id, value_refresh.get()),
        |(aid, _)| async move { fetch_value_history(aid).await.unwrap_or_default() },
    );
    let latest_value_cents = Signal::derive(move || {
        value_history
            .get()
            .unwrap_or_default()
            .first()
            .map(|e: &ValueHistoryEntry| e.value_cents)
    });

    let property_details = RwSignal::new(
        asset
            .property_details
            .clone()
            .unwrap_or_default(),
    );
    let capital = RwSignal::new(asset.capital.clone().unwrap_or_default());

    let show_edit_value = RwSignal::new(false);
    let edit_value_dollars = RwSignal::new(String::new());
    let edit_value_date = RwSignal::new(String::new());
    let edit_value_pending = RwSignal::new(false);
    let edit_value_err = RwSignal::new(None::<String>);

    let show_edit_details = RwSignal::new(false);
    let edit_beds = RwSignal::new(String::new());
    let edit_baths = RwSignal::new(String::new());
    let edit_sqft = RwSignal::new(String::new());
    let edit_year = RwSignal::new(String::new());
    let edit_notes = RwSignal::new(String::new());
    let edit_details_pending = RwSignal::new(false);
    let edit_details_err = RwSignal::new(None::<String>);

    let show_edit_capital = RwSignal::new(false);
    let edit_purchase = RwSignal::new(String::new());
    let edit_mortgage = RwSignal::new(String::new());
    let edit_other_debt = RwSignal::new(String::new());
    let edit_capital_pending = RwSignal::new(false);
    let edit_capital_err = RwSignal::new(None::<String>);

    let show_add_unit = RwSignal::new(false);
    let new_unit_name = RwSignal::new(String::new());
    let add_unit_err = RwSignal::new(None::<String>);
    let add_unit_pending = RwSignal::new(false);

    let show_add_project = RwSignal::new(false);
    let new_project_title = RwSignal::new(String::new());
    let new_project_budget = RwSignal::new(String::new());
    let add_project_err = RwSignal::new(None::<String>);
    let add_project_pending = RwSignal::new(false);

    let show_more = RwSignal::new(false);
    let show_archive = RwSignal::new(false);
    let archive_confirm = RwSignal::new(String::new());
    let archive_pending = RwSignal::new(false);
    let archive_err = RwSignal::new(None::<String>);
    let archive_blockers = RwSignal::new(Vec::<ArchiveBlockerDto>::new());
    let archived_ok = RwSignal::new(false);
    let show_purge = RwSignal::new(false);
    let purge_confirm = RwSignal::new(String::new());
    let purge_pending = RwSignal::new(false);
    let purge_err = RwSignal::new(None::<String>);

    let new_wo_href = format!(
        "{}?asset_id={}",
        FolioRoute::LandlordMaintenanceNew.path(),
        asset_id
    );
    let schedule_href = format!(
        "{}?mode=schedule&asset_id={}",
        FolioRoute::LandlordMaintenanceNew.path(),
        asset_id
    );
    let systems_href = FolioRoute::LandlordAssetSystems
        .path()
        .replace(":id", &asset_id.to_string());
    let docs_href = FolioRoute::LandlordAssetDocuments
        .path()
        .replace(":id", &asset_id.to_string());
    let maint_href = FolioRoute::LandlordMaintenance.path().to_string();

    let scoped_tickets = Signal::derive(move || {
        let scope = scope_ids_sig.get();
        tickets
            .get()
            .and_then(|r| r.ok())
            .map(|items: Vec<MaintenanceSummary>| {
                items
                    .into_iter()
                    .filter(|t| t.asset_id.map(|aid| scope.contains(&aid)).unwrap_or(false))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });

    let activity_items = Signal::derive(move || {
        scoped_tickets
            .get()
            .into_iter()
            .filter(|t| {
                matches!(
                    CaseStatus::from_str(&t.status),
                    CaseStatus::Open | CaseStatus::InProgress | CaseStatus::Scheduled
                )
            })
            .take(8)
            .map(|t| {
                let status = CaseStatus::from_str(&t.status);
                let (kind_label, tone) = match status {
                    CaseStatus::Scheduled => ("Sched", StatusPillTone::Info),
                    _ => ("WO", StatusPillTone::Warn),
                };
                ActivityRailItem {
                    id: t.id.to_string(),
                    kind_label: kind_label.into(),
                    title: t.subject,
                    meta: format!("{} · {}", status.as_str(), t.priority),
                    href: FolioRoute::LandlordMaintenanceDetail
                        .path()
                        .replace(":id", &t.id.to_string()),
                    tone,
                    show_photo_slot: true,
                }
            })
            .collect::<Vec<_>>()
    });

    let occupancy_value = Signal::derive(move || {
        let units = units_sig.get();
        let total = units.len();
        if total == 0 {
            return "—".into();
        }
        let lease_list = leases.get().and_then(|r| r.ok()).unwrap_or_default();
        let occupied = units
            .iter()
            .filter(|u| {
                let has_lease = occupying_lease_for_unit(&lease_list, u.id).is_some();
                UnitOccupancy::from_unit(&u.status, has_lease) == UnitOccupancy::Occupied
            })
            .count();
        format!("{}%", (occupied * 100) / total)
    });

    let rent_roll_value = Signal::derive(move || {
        let units = units_sig.get();
        let lease_list = leases.get().and_then(|r| r.ok()).unwrap_or_default();
        let mut sum = 0i64;
        let mut any = false;
        for u in &units {
            if let Some(l) = active_lease_for_unit(&lease_list, u.id) {
                if let Some(cents) = l.monthly_rent_cents {
                    sum += cents;
                    any = true;
                }
            }
        }
        if any {
            format_cents(sum)
        } else {
            "—".into()
        }
    });

    let open_items_value = Signal::derive(move || {
        let scope = scope_ids_sig.get();
        let n = tickets
            .get()
            .and_then(|r| r.ok())
            .map(|items| open_wo_count(&items, &scope))
            .unwrap_or(0);
        n.to_string()
    });

    let est_value = Signal::derive(move || {
        latest_value_cents
            .get()
            .map(format_cents)
            .unwrap_or_else(|| "—".into())
    });

    let monthly_rent_cents = Signal::derive(move || {
        let units = units_sig.get();
        let lease_list = leases.get().and_then(|r| r.ok()).unwrap_or_default();
        let mut sum = 0i64;
        let mut any = false;
        for u in &units {
            if let Some(l) = active_lease_for_unit(&lease_list, u.id) {
                if let Some(cents) = l.monthly_rent_cents {
                    sum += cents;
                    any = true;
                }
            }
        }
        any.then_some(sum)
    });

    let subtitle_sig = Signal::derive({
        let loc = loc.clone();
        let nick = asset_name.clone();
        let title = display_title.clone();
        move || {
            let n = units_sig.get().len();
            let units_label = if n == 1 {
                "1 unit".to_string()
            } else {
                format!("{n} units")
            };
            let place = if loc.is_empty() {
                units_label
            } else {
                format!("{loc} · {units_label}")
            };
            if nick != title && !nick.is_empty() {
                format!("{place} · Nickname: {nick}")
            } else {
                place
            }
        }
    });

    let navigate = StoredValue::new(use_navigate());
    let on_purge = move |_| {
        if purge_confirm.get().trim() != "PURGE" {
            purge_err.set(Some("Type PURGE to permanently delete.".into()));
            return;
        }
        purge_pending.set(true);
        purge_err.set(None);
        spawn_local(async move {
            match purge_folio_asset(asset_id.to_string()).await {
                Ok(()) => {
                    show_purge.set(false);
                    navigate.with_value(|nav| {
                        nav(FolioRoute::LandlordAssets.path(), Default::default());
                    });
                }
                Err(e) => purge_err.set(Some(e.to_string())),
            }
            purge_pending.set(false);
        });
    };

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
                        show_archive.set(false);
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

    let filtered_units = Signal::derive(move || {
        let q = units_search.get().trim().to_lowercase();
        let list = units_sig.get();
        if q.is_empty() {
            list
        } else {
            list.into_iter()
                .filter(|u| u.name.to_lowercase().contains(&q))
                .collect()
        }
    });

    view! {
        <div class="landlord-list-page">
            <nav class="unit-crumb" aria-label="Breadcrumb">
                <a href=FolioRoute::LandlordAssets.path()>"Assets"</a>
                <span class="material-symbols-outlined" style="font-size:14px;">"chevron_right"</span>
                <span style="font-weight:700;color:#191c1e;">{display_title.clone()}</span>
            </nav>

            <div class="hub-title-row" style="margin-bottom:0.35rem;">
                <StatusPill label=type_label.clone() tone=StatusPillTone::Neutral/>
            </div>
            {
                let new_wo_header = new_wo_href.clone();
                let schedule_header = schedule_href.clone();
                let title = display_title.clone();
                view! {
                    <PageHeader
                        title=Signal::derive({
                            let n = title.clone();
                            move || n.clone()
                        })
                        subtitle=subtitle_sig
                    >
                        <a class="folio-btn folio-btn--primary press" href=new_wo_header>
                            "New WO"
                        </a>
                        <a class="folio-btn folio-btn--ghost press" href=schedule_header>
                            "Schedule"
                        </a>
                        <button
                            type="button"
                            class="folio-btn folio-btn--ghost"
                            disabled=true
                            title="Not available"
                        >
                            "Delegate PM"
                        </button>
                        <div class="hub-more">
                            <button
                                type="button"
                                class="folio-btn folio-btn--ghost press"
                                aria-label="More actions"
                                on:click=move |_| show_more.update(|v| *v = !*v)
                            >
                                <span class="material-symbols-outlined">"more_horiz"</span>
                            </button>
                            <Show when=move || show_more.get()>
                                <div class="hub-more__menu" role="menu">
                                    <button
                                        type="button"
                                        class="hub-more__item"
                                        role="menuitem"
                                        on:click=move |_| {
                                            show_more.set(false);
                                            archive_err.set(None);
                                            archive_blockers.set(vec![]);
                                            archive_confirm.set(String::new());
                                            show_archive.set(true);
                                        }
                                    >
                                        "Archive…"
                                    </button>
                                    <button
                                        type="button"
                                        class="hub-more__item hub-more__item--danger"
                                        role="menuitem"
                                        on:click=move |_| {
                                            show_more.set(false);
                                            purge_err.set(None);
                                            purge_confirm.set(String::new());
                                            show_purge.set(true);
                                        }
                                    >
                                        "Delete permanently…"
                                    </button>
                                </div>
                            </Show>
                        </div>
                    </PageHeader>
                }
            }

            <PropertyTabBar
                asset_id=asset_id
                active=tab
                on_overview=Callback::new(move |_| {
                    tab.set(PropertyTab::Overview);
                })
                on_units=Callback::new(move |_| {
                    show_archive.set(false);
                    archive_confirm.set(String::new());
                    tab.set(PropertyTab::Units);
                })
            />

            <Show when=move || matches!(tab.get(), PropertyTab::Overview)>
                <div class="hub-overview">
                    <div class="hub-overview__main">
                        <div class="folio-stat-grid" style="margin-bottom:1.5rem;">
                            <StatCard label="Occupancy" value=occupancy_value icon="apartment"/>
                            <StatCard label="Rent roll" value=rent_roll_value icon="payments"/>
                            <StatCard label="Open items" value=open_items_value icon="build"/>
                            <button
                                type="button"
                                class="folio-stat-card folio-stat-card--link press"
                                style="text-align:left;cursor:pointer;border:none;width:100%;font:inherit;"
                                on:click=move |_| {
                                    edit_value_err.set(None);
                                    edit_value_dollars.set(
                                        latest_value_cents
                                            .get()
                                            .map(|c| format!("{:.0}", c as f64 / 100.0))
                                            .unwrap_or_default(),
                                    );
                                    edit_value_date.set(
                                        chrono::Utc::now()
                                            .date_naive()
                                            .format("%Y-%m-%d")
                                            .to_string(),
                                    );
                                    show_edit_value.set(true);
                                }
                            >
                                <span class="material-symbols-outlined folio-stat-card__icon">"real_estate_agent"</span>
                                <div class="folio-stat-card__body">
                                    <p class="folio-stat-card__label">"Est. value"</p>
                                    <p class="folio-stat-card__value">{move || est_value.get()}</p>
                                </div>
                            </button>
                        </div>

                        // Property details
                        <section class="proj-section" style="margin-bottom:1.5rem;">
                            <div class="proj-section__head">
                                <div>
                                    <h3 class="proj-section__title">"Property details"</h3>
                                    <p class="proj-section__hint">"Beds, baths, size, year, notes"</p>
                                </div>
                                <button
                                    type="button"
                                    class="folio-btn folio-btn--ghost folio-btn--sm press"
                                    on:click=move |_| {
                                        let d = property_details.get();
                                        edit_beds.set(
                                            d.beds.map(|b| format_count_label(Some(b))).unwrap_or_default(),
                                        );
                                        edit_baths.set(
                                            d.baths.map(|b| format_count_label(Some(b))).unwrap_or_default(),
                                        );
                                        edit_sqft.set(d.sqft.map(|s| s.to_string()).unwrap_or_default());
                                        edit_year.set(d.year_built.map(|y| y.to_string()).unwrap_or_default());
                                        edit_notes.set(d.notes.clone().unwrap_or_default());
                                        edit_details_err.set(None);
                                        show_edit_details.set(true);
                                    }
                                >
                                    "Edit"
                                </button>
                            </div>
                            <div class="hub-details-grid">
                                <div>
                                    <p class="hub-mgmt-label">"Beds"</p>
                                    <p class="hub-mgmt-name">
                                        {move || format_count_label(property_details.get().beds)}
                                    </p>
                                </div>
                                <div>
                                    <p class="hub-mgmt-label">"Baths"</p>
                                    <p class="hub-mgmt-name">
                                        {move || format_count_label(property_details.get().baths)}
                                    </p>
                                </div>
                                <div>
                                    <p class="hub-mgmt-label">"Sq ft"</p>
                                    <p class="hub-mgmt-name">
                                        {move || {
                                            property_details
                                                .get()
                                                .sqft
                                                .map(|s| format!("{s}"))
                                                .unwrap_or_else(|| "—".into())
                                        }}
                                    </p>
                                </div>
                                <div>
                                    <p class="hub-mgmt-label">"Year"</p>
                                    <p class="hub-mgmt-name">
                                        {move || {
                                            property_details
                                                .get()
                                                .year_built
                                                .map(|y| y.to_string())
                                                .unwrap_or_else(|| "—".into())
                                        }}
                                    </p>
                                </div>
                            </div>
                            {move || {
                                property_details.get().notes.filter(|n| !n.is_empty()).map(|n| {
                                    view! { <p class="hub-mgmt-meta" style="margin-top:0.85rem;">{n}</p> }
                                })
                            }}
                        </section>

                        // Capital
                        <section class="proj-section" style="margin-bottom:1.5rem;">
                            <div class="proj-section__head">
                                <div>
                                    <h3 class="proj-section__title">"Capital"</h3>
                                    <p class="proj-section__hint">"Mortgage, equity, NOI / cap when known"</p>
                                </div>
                                <button
                                    type="button"
                                    class="folio-btn folio-btn--ghost folio-btn--sm press"
                                    on:click=move |_| {
                                        let c = capital.get();
                                        edit_purchase.set(cents_to_dollars_input(c.purchase_price_cents));
                                        edit_mortgage.set(cents_to_dollars_input(c.mortgage_balance_cents));
                                        edit_other_debt.set(cents_to_dollars_input(c.other_debt_cents));
                                        edit_capital_err.set(None);
                                        show_edit_capital.set(true);
                                    }
                                >
                                    "Edit"
                                </button>
                            </div>
                            <div class="hub-details-grid hub-details-grid--capital">
                                <div>
                                    <p class="hub-mgmt-label">"Mortgage"</p>
                                    <p class="hub-mgmt-name">
                                        {move || format_optional_cents(capital.get().mortgage_balance_cents)}
                                    </p>
                                    <p class="hub-mgmt-meta">"Balance"</p>
                                </div>
                                <div>
                                    <p class="hub-mgmt-label">"Equity"</p>
                                    <p class="hub-mgmt-name">
                                        {move || {
                                            match latest_value_cents.get() {
                                                Some(value) => {
                                                    let c = capital.get();
                                                    let debt = c.mortgage_balance_cents.unwrap_or(0)
                                                        + c.other_debt_cents.unwrap_or(0);
                                                    format_cents(value - debt)
                                                }
                                                None => "—".into(),
                                            }
                                        }}
                                    </p>
                                    <p class="hub-mgmt-meta">"Value − mortgage − other debt"</p>
                                </div>
                                <div>
                                    <p class="hub-mgmt-label">"NOI"</p>
                                    <p class="hub-mgmt-name">
                                        {move || {
                                            monthly_rent_cents
                                                .get()
                                                .map(|m| format_cents(m * 12))
                                                .unwrap_or_else(|| "—".into())
                                        }}
                                    </p>
                                    <p class="hub-mgmt-meta">
                                        {move || {
                                            if monthly_rent_cents.get().is_some() {
                                                "Rough · rent × 12".to_string()
                                            } else {
                                                "Not enough data".into()
                                            }
                                        }}
                                    </p>
                                </div>
                                <div>
                                    <p class="hub-mgmt-label">"Cap rate"</p>
                                    <p class="hub-mgmt-name">
                                        {move || {
                                            match (monthly_rent_cents.get(), latest_value_cents.get()) {
                                                (Some(rent), Some(value)) if value > 0 => {
                                                    let noi = (rent * 12) as f64;
                                                    let rate = (noi / value as f64) * 100.0;
                                                    format!("{rate:.1}%")
                                                }
                                                _ => "—".into(),
                                            }
                                        }}
                                    </p>
                                    <p class="hub-mgmt-meta">
                                        {move || {
                                            match (monthly_rent_cents.get(), latest_value_cents.get()) {
                                                (Some(_), Some(v)) if v > 0 => "NOI ÷ value".to_string(),
                                                _ => "Need rent roll + est. value".into(),
                                            }
                                        }}
                                    </p>
                                </div>
                            </div>
                        </section>

                        <div class="hub-media-row">
                            {move || {
                                let href = docs_photos_href.get_value();
                                let photos: Vec<_> = documents
                                    .get()
                                    .unwrap_or_default()
                                    .into_iter()
                                    .filter(is_photo_like_doc)
                                    .collect();
                                if photos.is_empty() {
                                    view! {
                                        <a class="hub-media-card hub-media-card--photos press" href=href.clone()>
                                            <div class="hub-media-card__photo-empty">
                                                <span class="material-symbols-outlined">"add_a_photo"</span>
                                                <span>"No cover yet"</span>
                                            </div>
                                            <div class="hub-media-card__photo-foot">
                                                <div>
                                                    <p class="hub-media-card__title">"Photos"</p>
                                                    <p class="hub-media-card__sub">
                                                        "Photo upload — use Documents / vault"
                                                    </p>
                                                </div>
                                                <span class="folio-btn folio-btn--primary folio-btn--sm">"Open"</span>
                                            </div>
                                        </a>
                                    }.into_any()
                                } else {
                                    let n = photos.len();
                                    let sub = if n == 1 {
                                        "1 vault photo · open Documents".to_string()
                                    } else {
                                        format!("{n} vault photos · open Documents")
                                    };
                                    view! {
                                        <a class="hub-media-card hub-media-card--photos press" href=href>
                                            <div class="hub-media-card__photo-empty hub-media-card__photo-empty--has">
                                                <span class="material-symbols-outlined">"photo_library"</span>
                                                <span>{format!("{n} photos")}</span>
                                            </div>
                                            <div class="hub-media-card__photo-foot">
                                                <div>
                                                    <p class="hub-media-card__title">"Photos"</p>
                                                    <p class="hub-media-card__sub">{sub}</p>
                                                </div>
                                                <span class="folio-btn folio-btn--primary folio-btn--sm">"Open"</span>
                                            </div>
                                        </a>
                                    }.into_any()
                                }
                            }}
                            {
                                let empty_pin_label = if street.is_empty() {
                                    display_title.clone()
                                } else {
                                    street.clone()
                                };
                                let loc_for_map = loc.clone();
                                let map_open_href = map_href.clone();
                                view! {
                                    <div class="hub-media-card hub-media-card--map hub-mini-map">
                                        <leptos_meta::Link
                                            rel="stylesheet"
                                            href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
                                        />
                                        <leptos_meta::Script
                                            src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
                                        />
                                        {move || {
                                            let has_pin = matches!(
                                                (pin_lat.get(), pin_lng.get()),
                                                (Some(lat), Some(lng)) if lat != 0.0 || lng != 0.0
                                            );
                                            if has_pin {
                                                view! {
                                                    <div id="hub-mini-map" class="hub-mini-map__canvas"></div>
                                                }.into_any()
                                            } else {
                                                let pin_label = empty_pin_label.clone();
                                                view! {
                                                    <div class="hub-media-card__map-visual">
                                                        <span class="material-symbols-outlined">"location_off"</span>
                                                        <span>{pin_label}</span>
                                                    </div>
                                                }.into_any()
                                            }
                                        }}
                                        <div class="hub-media-card__map-foot">
                                            <div>
                                                <p class="hub-media-card__title">"On the map"</p>
                                                <p class="hub-media-card__sub">
                                                    {move || {
                                                        let loc = loc_for_map.clone();
                                                        if pin_lat.get().is_some() && pin_lng.get().is_some() {
                                                            if loc.is_empty() {
                                                                "Pin set · open portfolio map for full view".to_string()
                                                            } else {
                                                                format!("{loc} · open portfolio map")
                                                            }
                                                        } else if loc.is_empty() {
                                                            "No pin yet — geocode from the address".to_string()
                                                        } else {
                                                            format!("{loc} · add location")
                                                        }
                                                    }}
                                                </p>
                                                {move || geocode_err.get().map(|e| view! {
                                                    <p class="hub-mini-map__err">{e}</p>
                                                })}
                                            </div>
                                            <div class="hub-mini-map__actions">
                                                {move || {
                                                    let has = pin_lat.get().is_some() && pin_lng.get().is_some();
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class=if has {
                                                                "folio-btn folio-btn--ghost folio-btn--sm press"
                                                            } else {
                                                                "folio-btn folio-btn--primary folio-btn--sm press"
                                                            }
                                                            disabled=move || geocode_pending.get()
                                                            on:click=move |_| {
                                                                geocode_pending.set(true);
                                                                geocode_err.set(None);
                                                                spawn_local(async move {
                                                                    match geocode_asset(asset_id).await {
                                                                        Ok(c) => {
                                                                            pin_lat.set(Some(c.lat));
                                                                            pin_lng.set(Some(c.lng));
                                                                        }
                                                                        Err(e) => geocode_err.set(Some(e.to_string())),
                                                                    }
                                                                    geocode_pending.set(false);
                                                                });
                                                            }
                                                        >
                                                            {move || {
                                                                if geocode_pending.get() {
                                                                    "Geocoding…"
                                                                } else if pin_lat.get().is_some() {
                                                                    "Re-geocode"
                                                                } else {
                                                                    "Add location"
                                                                }
                                                            }}
                                                        </button>
                                                    }
                                                }}
                                                <a class="folio-btn folio-btn--ghost folio-btn--sm press" href=map_open_href>
                                                    "Open map"
                                                </a>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }
                        </div>

                        // Units peek
                        <section class="proj-section" style="margin-bottom:1.5rem;">
                            <div class="proj-section__head">
                                <h3 class="proj-section__title">"Units"</h3>
                                <div class="unit-actions">
                                    <button
                                        type="button"
                                        class="folio-btn folio-btn--ghost folio-btn--sm press"
                                        on:click=move |_| show_add_unit.set(true)
                                    >
                                        "Add unit"
                                    </button>
                                    <button
                                        type="button"
                                        class="folio-btn folio-btn--ghost folio-btn--sm press"
                                        on:click=move |_| tab.set(PropertyTab::Units)
                                    >
                                        "View all"
                                    </button>
                                </div>
                            </div>
                            <For
                                each=move || {
                                    units_sig
                                        .get()
                                        .into_iter()
                                        .enumerate()
                                        .collect::<Vec<_>>()
                                }
                                key=|(_, u)| u.id
                                children=move |(idx, u)| {
                                    let unit_id = u.id;
                                    let href = FolioRoute::LandlordAssetDetail
                                        .path()
                                        .replace(":id", &unit_id.to_string());
                                    let tile = unit_number_label(&u.name, idx);
                                    let name = u.name.clone();
                                    let status = u.status.clone();
                                    view! {
                                        <a class="hub-unit-row press" href=href>
                                            <div class="hub-unit-tile">{tile}</div>
                                            <div class="hub-unit-row__body">
                                                <div class="hub-unit-row__title-line">
                                                    <p class="hub-unit-row__name">{name}</p>
                                                    {move || {
                                                        let lease_list = leases
                                                            .get()
                                                            .and_then(|r| r.ok())
                                                            .unwrap_or_default();
                                                        let has_lease = occupying_lease_for_unit(
                                                            &lease_list,
                                                            unit_id,
                                                        )
                                                        .is_some();
                                                        let occ = UnitOccupancy::from_unit(
                                                            &status,
                                                            has_lease,
                                                        );
                                                        view! {
                                                            <StatusPill
                                                                label=occ.label().to_string()
                                                                tone=occ.tone()
                                                            />
                                                        }
                                                    }}
                                                </div>
                                                <p class="hub-unit-row__meta">
                                                    {move || {
                                                        let lease_list = leases
                                                            .get()
                                                            .and_then(|r| r.ok())
                                                            .unwrap_or_default();
                                                        match occupying_lease_for_unit(
                                                            &lease_list,
                                                            unit_id,
                                                        ) {
                                                            Some(l) => {
                                                                match LeaseStatus::from_str(&l.status) {
                                                                    LeaseStatus::Active => l
                                                                        .end_date
                                                                        .map(|d| {
                                                                            format!(
                                                                                "Lease ends {}",
                                                                                d.format("%b %Y")
                                                                            )
                                                                        })
                                                                        .unwrap_or_else(|| {
                                                                            "Active lease".into()
                                                                        }),
                                                                    LeaseStatus::Draft => {
                                                                        "Tenant · pending lease".into()
                                                                    }
                                                                    other => other.as_str().into(),
                                                                }
                                                            }
                                                            None => "Vacant".into(),
                                                        }
                                                    }}
                                                </p>
                                            </div>
                                            <div class="hub-unit-row__side">
                                                <p class="hub-unit-row__rent">
                                                    {move || {
                                                        let lease_list = leases
                                                            .get()
                                                            .and_then(|r| r.ok())
                                                            .unwrap_or_default();
                                                        active_lease_for_unit(
                                                            &lease_list,
                                                            unit_id,
                                                        )
                                                        .and_then(|l| l.monthly_rent_cents)
                                                        .map(format_cents)
                                                        .unwrap_or_else(|| "—".into())
                                                    }}
                                                </p>
                                                <p class="hub-unit-row__wo">
                                                    {move || {
                                                        let n = tickets
                                                            .get()
                                                            .and_then(|r| r.ok())
                                                            .map(|items| {
                                                                open_wo_count_for_unit(
                                                                    &items, unit_id,
                                                                )
                                                            })
                                                            .unwrap_or(0);
                                                        if n == 0 {
                                                            "No open items".into()
                                                        } else if n == 1 {
                                                            "1 open WO".into()
                                                        } else {
                                                            format!("{n} open WOs")
                                                        }
                                                    }}
                                                </p>
                                            </div>
                                            <span class="material-symbols-outlined hub-unit-row__chevron">
                                                "chevron_right"
                                            </span>
                                        </a>
                                    }
                                }
                            />
                        </section>

                        // Projects peek
                        <section class="proj-section" style="margin-bottom:1.5rem;">
                            <div class="proj-section__head">
                                <div>
                                    <h3 class="proj-section__title">"Projects"</h3>
                                    <p class="proj-section__hint">"Renovation projects"</p>
                                </div>
                                <button
                                    type="button"
                                    class="folio-btn folio-btn--ghost folio-btn--sm press"
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
                                                let spent = format_cents(p.actual_spent_cents);
                                                let budget = p
                                                    .estimated_cost_cents
                                                    .map(format_cents)
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

                        // Management + Upcoming
                        <div class="hub-peek-split">
                            <section class="proj-section hub-mgmt-card">
                                <div class="proj-section__head">
                                    <h3 class="proj-section__title">"Management"</h3>
                                </div>
                                <div class="hub-mgmt-body">
                                    <Suspense fallback=|| view! {
                                        <p class="proj-section__hint">"Loading…"</p>
                                    }>
                                        {move || {
                                            let c: Option<AssetContractorSummary> =
                                                contractor.get().flatten();
                                            match c {
                                                Some(c) => {
                                                    let trade = c
                                                        .primary_trade
                                                        .clone()
                                                        .unwrap_or_else(|| "Contractor".into());
                                                    view! {
                                                        <div class="hub-mgmt-row">
                                                            <span class="material-symbols-outlined hub-mgmt-icon">
                                                                "engineering"
                                                            </span>
                                                            <div style="flex:1;min-width:0;">
                                                                <p class="hub-mgmt-label">"Preferred contractor"</p>
                                                                <p class="hub-mgmt-name">{c.business_name.clone()}</p>
                                                                <p class="hub-mgmt-meta">{trade}</p>
                                                            </div>
                                                            <button
                                                                type="button"
                                                                class="folio-btn folio-btn--ghost folio-btn--sm press"
                                                                on:click=move |_| {
                                                                    assign_contractor_err.set(None);
                                                                    show_assign_contractor.set(true);
                                                                }
                                                            >
                                                                "Change"
                                                            </button>
                                                        </div>
                                                    }.into_any()
                                                }
                                                None => view! {
                                                    <div class="hub-mgmt-row">
                                                        <span class="material-symbols-outlined hub-mgmt-icon">
                                                            "engineering"
                                                        </span>
                                                        <div style="flex:1;min-width:0;">
                                                            <p class="hub-mgmt-label">"Preferred contractor"</p>
                                                            <p class="hub-mgmt-name hub-mgmt-name--muted">
                                                                "None set"
                                                            </p>
                                                            <p class="hub-mgmt-meta">
                                                                "Pick a vendor as the default for this property."
                                                            </p>
                                                        </div>
                                                        <button
                                                            type="button"
                                                            class="folio-btn folio-btn--primary folio-btn--sm press"
                                                            on:click=move |_| {
                                                                assign_contractor_err.set(None);
                                                                show_assign_contractor.set(true);
                                                            }
                                                        >
                                                            "Assign"
                                                        </button>
                                                    </div>
                                                }.into_any(),
                                            }
                                        }}
                                    </Suspense>
                                    <Show when=move || show_assign_contractor.get()>
                                        <div class="hub-assign-vendor">
                                            <p class="hub-mgmt-label" style="margin-bottom:0.5rem;">
                                                "Select a vendor"
                                            </p>
                                            {move || assign_contractor_err.get().map(|e| view! {
                                                <p style="color:#b91c1c;font-size:0.8rem;margin:0 0 0.5rem;">{e}</p>
                                            })}
                                            <Suspense fallback=|| view! {
                                                <p class="hub-mgmt-meta">"Loading vendors…"</p>
                                            }>
                                                {move || {
                                                    let list = vendors_for_assign.get().unwrap_or_default();
                                                    if list.is_empty() {
                                                        let vendors_href =
                                                            FolioRoute::LandlordVendors.path().to_string();
                                                        return view! {
                                                            <div class="folio-empty--compact">
                                                                <p>"No vendors yet."</p>
                                                                <a
                                                                    class="folio-btn folio-btn--ghost folio-btn--sm press"
                                                                    style="margin-top:0.5rem;"
                                                                    href=vendors_href
                                                                >
                                                                    "Open Vendors"
                                                                </a>
                                                            </div>
                                                        }.into_any();
                                                    }
                                                    view! {
                                                        <div class="hub-assign-vendor__list">
                                                            <For
                                                                each=move || list.clone()
                                                                key=|v| v.id
                                                                children=move |v| {
                                                                    let vid = v.id.to_string();
                                                                    let name = v.business_name.clone();
                                                                    let trade = v
                                                                        .primary_trade
                                                                        .clone()
                                                                        .unwrap_or_else(|| "Vendor".into());
                                                                    view! {
                                                                        <button
                                                                            type="button"
                                                                            class="hub-assign-vendor__option press"
                                                                            disabled=move || assign_contractor_pending.get()
                                                                            on:click=move |_| {
                                                                                let v2 = vid.clone();
                                                                                assign_contractor_pending.set(true);
                                                                                assign_contractor_err.set(None);
                                                                                spawn_local(async move {
                                                                                    match set_default_contractor(
                                                                                        asset_id.to_string(),
                                                                                        v2,
                                                                                    )
                                                                                    .await
                                                                                    {
                                                                                        Ok(()) => {
                                                                                            show_assign_contractor
                                                                                                .set(false);
                                                                                            contractor_refresh
                                                                                                .update(|n| *n += 1);
                                                                                        }
                                                                                        Err(e) => {
                                                                                            assign_contractor_err
                                                                                                .set(Some(e.to_string()));
                                                                                        }
                                                                                    }
                                                                                    assign_contractor_pending
                                                                                        .set(false);
                                                                                });
                                                                            }
                                                                        >
                                                                            <span class="hub-mgmt-name">{name}</span>
                                                                            <span class="hub-mgmt-meta">{trade}</span>
                                                                        </button>
                                                                    }
                                                                }
                                                            />
                                                        </div>
                                                    }.into_any()
                                                }}
                                            </Suspense>
                                            <button
                                                type="button"
                                                class="folio-btn folio-btn--ghost folio-btn--sm"
                                                style="margin-top:0.5rem;"
                                                on:click=move |_| show_assign_contractor.set(false)
                                            >
                                                "Cancel"
                                            </button>
                                        </div>
                                    </Show>
                                    <div class="hub-mgmt-row">
                                        <span class="material-symbols-outlined hub-mgmt-icon">"badge"</span>
                                        <div>
                                            <p class="hub-mgmt-label">"Property manager"</p>
                                            <p class="hub-mgmt-name hub-mgmt-name--muted">"Not delegated"</p>
                                            <p class="hub-mgmt-meta">"You manage day-to-day"</p>
                                        </div>
                                    </div>
                                    <button
                                        type="button"
                                        class="folio-btn folio-btn--ghost"
                                        disabled=true
                                        aria-disabled="true"
                                        title="Delegating to a property manager is not available yet"
                                    >
                                        "Delegate to PM — Not available"
                                    </button>
                                </div>
                            </section>

                            <section class="proj-section">
                                <div class="proj-section__head">
                                    <h3 class="proj-section__title">"Upcoming"</h3>
                                    <a
                                        class="folio-btn folio-btn--ghost folio-btn--sm press"
                                        href=schedule_href.clone()
                                    >
                                        "Schedule"
                                    </a>
                                </div>
                                {move || {
                                    let scheduled: Vec<MaintenanceSummary> = scoped_tickets
                                        .get()
                                        .into_iter()
                                        .filter(|t| {
                                            CaseStatus::from_str(&t.status) == CaseStatus::Scheduled
                                        })
                                        .take(5)
                                        .collect();
                                    if scheduled.is_empty() {
                                        let href = format!(
                                            "{}?mode=schedule&asset_id={}",
                                            FolioRoute::LandlordMaintenanceNew.path(),
                                            asset_id
                                        );
                                        return view! {
                                            <div class="folio-empty--compact">
                                                <p>"Nothing scheduled."</p>
                                                <a
                                                    class="folio-btn folio-btn--ghost folio-btn--sm press"
                                                    style="margin-top:0.75rem;"
                                                    href=href
                                                >
                                                    "Schedule"
                                                </a>
                                            </div>
                                        }.into_any();
                                    }
                                    view! {
                                        <For
                                            each=move || scheduled.clone()
                                            key=|t| t.id
                                            children=move |t| {
                                                let href = FolioRoute::LandlordMaintenanceDetail
                                                    .path()
                                                    .replace(":id", &t.id.to_string());
                                                view! {
                                                    <a class="hub-activity-rail__row press" href=href>
                                                        <StatusPill
                                                            label="Sched".to_string()
                                                            tone=StatusPillTone::Info
                                                        />
                                                        <div class="hub-activity-rail__body">
                                                            <p class="hub-activity-rail__row-title">
                                                                {t.subject.clone()}
                                                            </p>
                                                            <p class="hub-activity-rail__row-meta">
                                                                {format!("{} · {}", t.priority, t.case_type)}
                                                            </p>
                                                        </div>
                                                    </a>
                                                }
                                            }
                                        />
                                    }.into_any()
                                }}
                            </section>
                        </div>

                        // Systems peek
                        <section class="proj-section" style="margin:1.5rem 0;">
                            <div class="proj-section__head">
                                <h3 class="proj-section__title">"Building systems"</h3>
                                <a
                                    class="folio-btn folio-btn--ghost folio-btn--sm press"
                                    href=systems_href.clone()
                                >
                                    "Manage"
                                </a>
                            </div>
                            <Suspense fallback=|| view! {
                                <div class="folio-empty--compact">"Loading…"</div>
                            }>
                                {move || {
                                    let list: Vec<NestedSystemDto> =
                                        systems.get().unwrap_or_default();
                                    if list.is_empty() {
                                        let href = FolioRoute::LandlordAssetSystems
                                            .path()
                                            .replace(":id", &asset_id.to_string());
                                        return view! {
                                            <div class="folio-empty--compact">
                                                <p>"No systems registered yet."</p>
                                                <a
                                                    class="folio-btn folio-btn--ghost folio-btn--sm press"
                                                    style="margin-top:0.75rem;"
                                                    href=href
                                                >
                                                    "Add systems"
                                                </a>
                                            </div>
                                        }.into_any();
                                    }
                                    let top: Vec<_> = list.into_iter().take(3).collect();
                                    view! {
                                        <For
                                            each=move || top.clone()
                                            key=|s| s.id
                                            children=move |s| {
                                                let cond = s
                                                    .condition
                                                    .clone()
                                                    .unwrap_or_else(|| s.status.clone());
                                                view! {
                                                    <div class="hub-activity-rail__row">
                                                        <StatusPill
                                                            label="System".to_string()
                                                            tone=StatusPillTone::Info
                                                        />
                                                        <div class="hub-activity-rail__body">
                                                            <p class="hub-activity-rail__row-title">
                                                                {s.name.clone()}
                                                            </p>
                                                            <p class="hub-activity-rail__row-meta">
                                                                {cond}
                                                            </p>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                    }.into_any()
                                }}
                            </Suspense>
                        </section>

                        // Expenses peek
                        <section class="proj-section">
                            <div class="proj-section__head">
                                <div>
                                    <h3 class="proj-section__title">"Expenses & receipts"</h3>
                                    <p class="proj-section__hint">"Documents and logged costs"</p>
                                </div>
                                <a
                                    class="folio-btn folio-btn--ghost folio-btn--sm press"
                                    href=docs_href.clone()
                                >
                                    "All"
                                </a>
                            </div>
                            <Suspense fallback=|| view! {
                                <div class="folio-empty--compact">"Loading…"</div>
                            }>
                                {move || {
                                    let docs_path = FolioRoute::LandlordAssetDocuments
                                        .path()
                                        .replace(":id", &asset_id.to_string());
                                    let rows: Vec<PropertyDocumentRow> =
                                        documents.get().unwrap_or_default();
                                    let recent: Vec<_> = rows.into_iter().take(5).collect();
                                    if recent.is_empty() {
                                        let href = docs_path.clone();
                                        return view! {
                                            <div class="folio-empty--compact">
                                                <p>"No expenses or documents yet."</p>
                                                <a
                                                    class="folio-btn folio-btn--ghost folio-btn--sm press"
                                                    style="margin-top:0.75rem;"
                                                    href=href
                                                >
                                                    "Open documents"
                                                </a>
                                            </div>
                                        }.into_any();
                                    }
                                    view! {
                                        <For
                                            each=move || recent.clone()
                                            key=|r| r.id
                                            children={
                                                let docs_path = docs_path.clone();
                                                move |r| {
                                                let kind_label = match r.kind {
                                                    PropertyDocumentKind::Vault => "Vault",
                                                    PropertyDocumentKind::Expense => "Exp",
                                                };
                                                let tone = match r.kind {
                                                    PropertyDocumentKind::Expense => {
                                                        StatusPillTone::Danger
                                                    }
                                                    PropertyDocumentKind::Vault => {
                                                        StatusPillTone::Neutral
                                                    }
                                                };
                                                let amount = r
                                                    .amount_cents
                                                    .map(format_cents)
                                                    .unwrap_or_else(|| "—".into());
                                                let href = docs_path.clone();
                                                view! {
                                                    <a
                                                        class="hub-activity-rail__row press"
                                                        href=href
                                                    >
                                                        <StatusPill
                                                            label=kind_label.to_string()
                                                            tone=tone
                                                        />
                                                        <div class="hub-activity-rail__body">
                                                            <p class="hub-activity-rail__row-title">
                                                                {r.title.clone()}
                                                            </p>
                                                            <p class="hub-activity-rail__row-meta">
                                                                {r.category.clone()}
                                                            </p>
                                                        </div>
                                                        <strong class="hub-expense-amount">{amount}</strong>
                                                    </a>
                                                }
                                            }}
                                        />
                                    }.into_any()
                                }}
                            </Suspense>
                        </section>

                        {move || archived_ok.get().then(|| view! {
                            <p class="hub-archive-foot__ok">"Property archived."</p>
                        })}
                    </div>

                    <ActivityRail
                        items=activity_items
                        see_all_href=maint_href.clone()
                        subtitle="This property"
                        new_wo_href=new_wo_href.clone()
                        schedule_href=schedule_href.clone()
                    />
                </div>
            </Show>

            <Show when=move || matches!(tab.get(), PropertyTab::Units)>
                <section class="proj-section">
                    <div class="proj-section__head">
                        <div>
                            <h3 class="proj-section__title">"Units"</h3>
                            <p class="proj-section__hint">
                                {format!(
                                    "{unit_count} units — open a unit for occupant, lease, and activity."
                                )}
                            </p>
                        </div>
                        <button
                            type="button"
                            class="folio-btn folio-btn--primary press"
                            on:click=move |_| show_add_unit.set(true)
                        >
                            "Add unit"
                        </button>
                    </div>
                    <div class="hub-units-search">
                        <span class="material-symbols-outlined hub-units-search__icon">"search"</span>
                        <input
                            class="folio-input"
                            type="search"
                            placeholder="Search unit…"
                            prop:value=move || units_search.get()
                            on:input=move |ev| units_search.set(event_target_value(&ev))
                        />
                    </div>
                    <For
                        each=move || {
                            filtered_units
                                .get()
                                .into_iter()
                                .enumerate()
                                .collect::<Vec<_>>()
                        }
                        key=|(_, u)| u.id
                        children=move |(idx, u)| {
                            let unit_id = u.id;
                            let href = FolioRoute::LandlordAssetDetail
                                .path()
                                .replace(":id", &unit_id.to_string());
                            let tile = unit_number_label(&u.name, idx);
                            let name = u.name.clone();
                            let status = u.status.clone();
                            view! {
                                <a class="hub-unit-row press" href=href>
                                    <div class="hub-unit-tile">{tile}</div>
                                    <div class="hub-unit-row__body">
                                        <div class="hub-unit-row__title-line">
                                            <p class="hub-unit-row__name">{name}</p>
                                            {move || {
                                                let lease_list = leases
                                                    .get()
                                                    .and_then(|r| r.ok())
                                                    .unwrap_or_default();
                                                let has_lease =
                                                    occupying_lease_for_unit(&lease_list, unit_id)
                                                        .is_some();
                                                let occ =
                                                    UnitOccupancy::from_unit(&status, has_lease);
                                                view! {
                                                    <StatusPill
                                                        label=occ.label().to_string()
                                                        tone=occ.tone()
                                                    />
                                                }
                                            }}
                                        </div>
                                        <p class="hub-unit-row__meta">
                                            {move || {
                                                let lease_list = leases
                                                    .get()
                                                    .and_then(|r| r.ok())
                                                    .unwrap_or_default();
                                                match occupying_lease_for_unit(&lease_list, unit_id)
                                                {
                                                    Some(l) => {
                                                        match LeaseStatus::from_str(&l.status) {
                                                            LeaseStatus::Active => l
                                                                .end_date
                                                                .map(|d| {
                                                                    format!(
                                                                        "Lease ends {}",
                                                                        d.format("%b %Y")
                                                                    )
                                                                })
                                                                .unwrap_or_else(|| {
                                                                    "Active lease".into()
                                                                }),
                                                            LeaseStatus::Draft => {
                                                                "Tenant · pending lease".into()
                                                            }
                                                            other => other.as_str().into(),
                                                        }
                                                    }
                                                    None => "Vacant".into(),
                                                }
                                            }}
                                        </p>
                                    </div>
                                    <div class="hub-unit-row__side">
                                        <p class="hub-unit-row__rent">
                                            {move || {
                                                let lease_list = leases
                                                    .get()
                                                    .and_then(|r| r.ok())
                                                    .unwrap_or_default();
                                                active_lease_for_unit(&lease_list, unit_id)
                                                    .and_then(|l| l.monthly_rent_cents)
                                                    .map(format_cents)
                                                    .unwrap_or_else(|| "—".into())
                                            }}
                                        </p>
                                        <p class="hub-unit-row__wo">
                                            {move || {
                                                let n = tickets
                                                    .get()
                                                    .and_then(|r| r.ok())
                                                    .map(|items| {
                                                        open_wo_count_for_unit(&items, unit_id)
                                                    })
                                                    .unwrap_or(0);
                                                if n == 0 {
                                                    "No open items".into()
                                                } else if n == 1 {
                                                    "1 open WO".into()
                                                } else {
                                                    format!("{n} open WOs")
                                                }
                                            }}
                                        </p>
                                    </div>
                                    <span class="material-symbols-outlined hub-unit-row__chevron">
                                        "chevron_right"
                                    </span>
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
                            <button class="modal-close" on:click=move |_| show_add_unit.set(false)><span class="material-symbols-outlined">"close"</span></button>
                        </div>
                        <div class="modal-body space-y-4">
                            <label class="folio-field__label">
                                "Unit name"
                                <input
                                    class="folio-input"
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
                            <button class="folio-btn folio-btn--ghost" on:click=move |_| show_add_unit.set(false)>"Cancel"</button>
                            <button
                                class="folio-btn folio-btn--primary"
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
                            <button class="modal-close" on:click=move |_| show_add_project.set(false)><span class="material-symbols-outlined">"close"</span></button>
                        </div>
                        <div class="modal-body space-y-4">
                            <label class="folio-field__label">
                                "Title"
                                <input
                                    class="folio-input"
                                    type="text"
                                    placeholder="Kitchen renovation"
                                    prop:value=move || new_project_title.get()
                                    on:input=move |ev| new_project_title.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="folio-field__label">
                                "Budget (optional)"
                                <input
                                    class="folio-input"
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
                            <button class="folio-btn folio-btn--ghost" on:click=move |_| show_add_project.set(false)>"Cancel"</button>
                            <button
                                class="folio-btn folio-btn--primary"
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

            <Show when=move || show_edit_value.get()>
                <div class="modal-backdrop" on:click=move |_| show_edit_value.set(false)>
                    <div
                        class="modal-card"
                        style="max-width:24rem;"
                        on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                    >
                        <div class="modal-header">
                            <h3 class="modal-title">"Update est. value"</h3>
                            <button
                                type="button"
                                class="modal-close"
                                aria-label="Close"
                                on:click=move |_| show_edit_value.set(false)
                            >
                                <span class="material-symbols-outlined">"close"</span>
                            </button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="folio-field">
                                <label class="folio-field__label">"Estimated value (USD)"</label>
                                <input
                                    class="folio-input"
                                    type="text"
                                    inputmode="decimal"
                                    placeholder="285000"
                                    prop:value=move || edit_value_dollars.get()
                                    on:input=move |ev| edit_value_dollars.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Valuation date"</label>
                                <input
                                    class="folio-input"
                                    type="date"
                                    prop:value=move || edit_value_date.get()
                                    on:input=move |ev| edit_value_date.set(event_target_value(&ev))
                                />
                            </div>
                            {move || edit_value_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button
                                class="folio-btn folio-btn--ghost"
                                on:click=move |_| show_edit_value.set(false)
                            >
                                "Cancel"
                            </button>
                            <button
                                class="folio-btn folio-btn--primary"
                                disabled=move || edit_value_pending.get()
                                on:click=move |_| {
                                    let dollars = match parse_dollars_to_cents(&edit_value_dollars.get()) {
                                        Ok(Some(c)) if c > 0 => c,
                                        Ok(_) => {
                                            edit_value_err.set(Some("Enter a positive value.".into()));
                                            return;
                                        }
                                        Err(e) => {
                                            edit_value_err.set(Some(e));
                                            return;
                                        }
                                    };
                                    let date = edit_value_date.get().trim().to_string();
                                    if date.is_empty() {
                                        edit_value_err.set(Some("Pick a valuation date.".into()));
                                        return;
                                    }
                                    edit_value_pending.set(true);
                                    edit_value_err.set(None);
                                    spawn_local(async move {
                                        match log_property_value(
                                            asset_id,
                                            "manual".into(),
                                            dollars,
                                            date,
                                            None,
                                        )
                                        .await
                                        {
                                            Ok(_) => {
                                                show_edit_value.set(false);
                                                value_refresh.update(|n| *n += 1);
                                            }
                                            Err(e) => edit_value_err.set(Some(e.to_string())),
                                        }
                                        edit_value_pending.set(false);
                                    });
                                }
                            >
                                {move || if edit_value_pending.get() { "Saving…" } else { "Save value" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_edit_details.get()>
                <div class="modal-backdrop" on:click=move |_| show_edit_details.set(false)>
                    <div
                        class="modal-card"
                        style="max-width:28rem;"
                        on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                    >
                        <div class="modal-header">
                            <h3 class="modal-title">"Property details"</h3>
                            <button
                                type="button"
                                class="modal-close"
                                aria-label="Close"
                                on:click=move |_| show_edit_details.set(false)
                            >
                                <span class="material-symbols-outlined">"close"</span>
                            </button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="hub-details-grid">
                                <div class="folio-field">
                                    <label class="folio-field__label">"Beds"</label>
                                    <input
                                        class="folio-input"
                                        type="text"
                                        inputmode="decimal"
                                        prop:value=move || edit_beds.get()
                                        on:input=move |ev| edit_beds.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="folio-field">
                                    <label class="folio-field__label">"Baths"</label>
                                    <input
                                        class="folio-input"
                                        type="text"
                                        inputmode="decimal"
                                        prop:value=move || edit_baths.get()
                                        on:input=move |ev| edit_baths.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="folio-field">
                                    <label class="folio-field__label">"Sq ft"</label>
                                    <input
                                        class="folio-input"
                                        type="text"
                                        inputmode="numeric"
                                        prop:value=move || edit_sqft.get()
                                        on:input=move |ev| edit_sqft.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="folio-field">
                                    <label class="folio-field__label">"Year built"</label>
                                    <input
                                        class="folio-input"
                                        type="text"
                                        inputmode="numeric"
                                        prop:value=move || edit_year.get()
                                        on:input=move |ev| edit_year.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Notes"</label>
                                <textarea
                                    class="folio-input"
                                    rows="3"
                                    prop:value=move || edit_notes.get()
                                    on:input=move |ev| edit_notes.set(event_target_value(&ev))
                                ></textarea>
                            </div>
                            {move || edit_details_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button
                                class="folio-btn folio-btn--ghost"
                                on:click=move |_| show_edit_details.set(false)
                            >
                                "Cancel"
                            </button>
                            <button
                                class="folio-btn folio-btn--primary"
                                disabled=move || edit_details_pending.get()
                                on:click=move |_| {
                                    let parse_f = |s: String| -> Result<Option<f64>, String> {
                                        let t = s.trim();
                                        if t.is_empty() {
                                            return Ok(None);
                                        }
                                        t.parse::<f64>()
                                            .map(Some)
                                            .map_err(|_| "Enter a valid number.".into())
                                    };
                                    let parse_i = |s: String| -> Result<Option<i32>, String> {
                                        let t = s.trim();
                                        if t.is_empty() {
                                            return Ok(None);
                                        }
                                        t.parse::<i32>()
                                            .map(Some)
                                            .map_err(|_| "Enter a whole number.".into())
                                    };
                                    let beds = match parse_f(edit_beds.get()) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            edit_details_err.set(Some(e));
                                            return;
                                        }
                                    };
                                    let baths = match parse_f(edit_baths.get()) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            edit_details_err.set(Some(e));
                                            return;
                                        }
                                    };
                                    let sqft = match parse_i(edit_sqft.get()) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            edit_details_err.set(Some(e));
                                            return;
                                        }
                                    };
                                    let year_built = match parse_i(edit_year.get()) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            edit_details_err.set(Some(e));
                                            return;
                                        }
                                    };
                                    let notes = {
                                        let n = edit_notes.get().trim().to_string();
                                        if n.is_empty() { None } else { Some(n) }
                                    };
                                    let body = PropertyDetailsDto {
                                        beds,
                                        baths,
                                        sqft,
                                        year_built,
                                        notes,
                                    };
                                    edit_details_pending.set(true);
                                    edit_details_err.set(None);
                                    spawn_local(async move {
                                        match put_asset_details(asset_id, body.clone()).await {
                                            Ok(saved) => {
                                                property_details.set(saved);
                                                show_edit_details.set(false);
                                            }
                                            Err(e) => edit_details_err.set(Some(e.to_string())),
                                        }
                                        edit_details_pending.set(false);
                                    });
                                }
                            >
                                {move || if edit_details_pending.get() { "Saving…" } else { "Save" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_edit_capital.get()>
                <div class="modal-backdrop" on:click=move |_| show_edit_capital.set(false)>
                    <div
                        class="modal-card"
                        style="max-width:24rem;"
                        on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                    >
                        <div class="modal-header">
                            <h3 class="modal-title">"Capital"</h3>
                            <button
                                type="button"
                                class="modal-close"
                                aria-label="Close"
                                on:click=move |_| show_edit_capital.set(false)
                            >
                                <span class="material-symbols-outlined">"close"</span>
                            </button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="folio-field">
                                <label class="folio-field__label">"Purchase price (USD)"</label>
                                <input
                                    class="folio-input"
                                    type="text"
                                    inputmode="decimal"
                                    prop:value=move || edit_purchase.get()
                                    on:input=move |ev| edit_purchase.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Mortgage balance (USD)"</label>
                                <input
                                    class="folio-input"
                                    type="text"
                                    inputmode="decimal"
                                    prop:value=move || edit_mortgage.get()
                                    on:input=move |ev| edit_mortgage.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Other debt (USD)"</label>
                                <input
                                    class="folio-input"
                                    type="text"
                                    inputmode="decimal"
                                    prop:value=move || edit_other_debt.get()
                                    on:input=move |ev| edit_other_debt.set(event_target_value(&ev))
                                />
                            </div>
                            {move || edit_capital_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button
                                class="folio-btn folio-btn--ghost"
                                on:click=move |_| show_edit_capital.set(false)
                            >
                                "Cancel"
                            </button>
                            <button
                                class="folio-btn folio-btn--primary"
                                disabled=move || edit_capital_pending.get()
                                on:click=move |_| {
                                    let purchase_price_cents = match parse_dollars_to_cents(&edit_purchase.get()) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            edit_capital_err.set(Some(e));
                                            return;
                                        }
                                    };
                                    let mortgage_balance_cents =
                                        match parse_dollars_to_cents(&edit_mortgage.get()) {
                                            Ok(v) => v,
                                            Err(e) => {
                                                edit_capital_err.set(Some(e));
                                                return;
                                            }
                                        };
                                    let other_debt_cents =
                                        match parse_dollars_to_cents(&edit_other_debt.get()) {
                                            Ok(v) => v,
                                            Err(e) => {
                                                edit_capital_err.set(Some(e));
                                                return;
                                            }
                                        };
                                    let body = CapitalDto {
                                        purchase_price_cents,
                                        mortgage_balance_cents,
                                        other_debt_cents,
                                    };
                                    edit_capital_pending.set(true);
                                    edit_capital_err.set(None);
                                    spawn_local(async move {
                                        match put_asset_capital(asset_id, body.clone()).await {
                                            Ok(saved) => {
                                                capital.set(saved);
                                                show_edit_capital.set(false);
                                            }
                                            Err(e) => edit_capital_err.set(Some(e.to_string())),
                                        }
                                        edit_capital_pending.set(false);
                                    });
                                }
                            >
                                {move || if edit_capital_pending.get() { "Saving…" } else { "Save" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_archive.get()>
                <div
                    class="modal-backdrop"
                    on:click=move |_| {
                        show_archive.set(false);
                        archive_confirm.set(String::new());
                    }
                >
                    <div
                        class="modal-card"
                        style="max-width:28rem;"
                        on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                    >
                        <div class="modal-header">
                            <h3 class="modal-title">"Archive property"</h3>
                            <button
                                type="button"
                                class="modal-close"
                                aria-label="Close"
                                on:click=move |_| {
                                    show_archive.set(false);
                                    archive_confirm.set(String::new());
                                }
                            >
                                <span class="material-symbols-outlined">"close"</span>
                            </button>
                        </div>
                        <div class="modal-body space-y-4">
                            <p class="proj-section__hint">
                                "Archive hides this property from the Assets list. Active units block archive until they are archived first. Type DELETE to confirm."
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
                                            <li>{b.message}</li>
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
                                {move || if archive_pending.get() { "Archiving…" } else { "Archive property" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_purge.get()>
                <div
                    class="modal-backdrop"
                    on:click=move |_| {
                        show_purge.set(false);
                        purge_confirm.set(String::new());
                    }
                >
                    <div
                        class="modal-card"
                        style="max-width:28rem;"
                        on:click=|ev: web_sys::MouseEvent| ev.stop_propagation()
                    >
                        <div class="modal-header">
                            <h3 class="modal-title">"Delete permanently"</h3>
                            <button
                                type="button"
                                class="modal-close"
                                aria-label="Close"
                                on:click=move |_| {
                                    show_purge.set(false);
                                    purge_confirm.set(String::new());
                                }
                            >
                                <span class="material-symbols-outlined">"close"</span>
                            </button>
                        </div>
                        <div class="modal-body space-y-4">
                            <p class="proj-section__hint">
                                "Irreversible. Removes this property, all units and systems under it, and associated leases, work orders, reservations, and related Folio records. Type PURGE to confirm."
                            </p>
                            <p class="proj-section__hint">
                                {format!(
                                    "{} unit(s) in this building will be deleted.",
                                    unit_count
                                )}
                            </p>
                            <label class="folio-field__label">
                                "Type PURGE"
                                <input
                                    class="folio-input"
                                    type="text"
                                    autocomplete="off"
                                    prop:value=move || purge_confirm.get()
                                    on:input=move |ev| purge_confirm.set(event_target_value(&ev))
                                />
                            </label>
                            {move || purge_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button
                                class="folio-btn folio-btn--ghost"
                                on:click=move |_| show_purge.set(false)
                            >
                                "Cancel"
                            </button>
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary"
                                style="background:#93000a;border-color:#93000a;"
                                prop:disabled=move || {
                                    purge_pending.get()
                                        || purge_confirm.get().trim() != "PURGE"
                                }
                                on:click=on_purge
                            >
                                {move || if purge_pending.get() { "Deleting…" } else { "Delete forever" }}
                            </button>
                        </div>
                    </div>
                </div>
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
            attributes: None,
            latitude: None,
            longitude: None,
            property_details: None,
            capital: None,
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
