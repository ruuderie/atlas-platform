// apps/folio/src/pages/landlord/building_systems.rs
//
// Building Systems page — /l/systems
//
// Portfolio-wide view of all registered building systems (elevators, boilers,
// HVAC, fire suppression, roofs, etc.) across all properties.
//
// Data source: GET /api/folio/systems → [BuildingSystemDetail]
//
// ─── Layout ───────────────────────────────────────────────────────────────────
//  KPI strip   : Total | Service Due ≤30d | Cert Expiring ≤30d | Overdue
//  Filter bar  : search + category chips
//  Card grid   : system name · type · condition · next service · cert expiry
// ─────────────────────────────────────────────────────────────────────────────

use chrono::{NaiveDate, Utc};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::pages::landlord::vendors::{list_assets_for_picker, AssetPickerItem};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingSystemDetail {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub property_id: Option<Uuid>,
    pub name: String,
    pub serial_number: Option<String>,
    pub status: String,
    pub condition: Option<String>,
    pub scheduled_service_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<Utc>,
}

// ── Local enums ───────────────────────────────────────────────────────────────

/// High-level category derived from metadata.system_type for filter chips.
#[derive(Debug, Clone, Copy, PartialEq)]
enum SystemCategory {
    All,
    LifeSafety,
    Mechanical,
    Electrical,
    Water,
    Structure,
    Amenity,
    Access,
    Other,
}

impl SystemCategory {
    fn label(self) -> &'static str {
        match self {
            SystemCategory::All => "All",
            SystemCategory::LifeSafety => "Life Safety",
            SystemCategory::Mechanical => "Mechanical",
            SystemCategory::Electrical => "Electrical",
            SystemCategory::Water => "Water",
            SystemCategory::Structure => "Structure",
            SystemCategory::Amenity => "Amenity",
            SystemCategory::Access => "Access",
            SystemCategory::Other => "Other",
        }
    }

    fn from_system_type(t: &str) -> Self {
        match t {
            "fire_suppression" | "fire_alarm" | "emergency_lighting" | "elevator" | "escalator" => {
                SystemCategory::LifeSafety
            }
            "common_area_hvac" | "boiler" | "cooling_tower" | "chiller" => {
                SystemCategory::Mechanical
            }
            "generator" | "electrical_panel" | "transformer_vault" => SystemCategory::Electrical,
            "roof_drain_system" | "sewer_lift" | "backflow_preventer" => SystemCategory::Water,
            "roof" | "facade" | "parking_structure" => SystemCategory::Structure,
            "pool" | "spa" | "gym_equipment" => SystemCategory::Amenity,
            "security_system" | "access_control" | "intercom" => SystemCategory::Access,
            _ => SystemCategory::Other,
        }
    }

    fn matches(self, system_type: &str) -> bool {
        self == SystemCategory::All || self == SystemCategory::from_system_type(system_type)
    }
}

/// Visual urgency derived from dates.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Urgency {
    Overdue,
    DueSoon, // ≤30 days
    Ok,
    Unknown,
}

impl Urgency {
    fn from_system(sys: &BuildingSystemDetail) -> Self {
        let today = Utc::now().date_naive();
        let dates: Vec<NaiveDate> = [sys.scheduled_service_date, sys.expiry_date]
            .into_iter()
            .flatten()
            .collect();
        if dates.is_empty() {
            return Urgency::Unknown;
        }
        let earliest = dates.iter().cloned().min().unwrap();
        let days = (earliest - today).num_days();
        if days < 0 {
            Urgency::Overdue
        } else if days <= 30 {
            Urgency::DueSoon
        } else {
            Urgency::Ok
        }
    }

    fn card_class(self) -> &'static str {
        match self {
            Urgency::Overdue => "bsys-card bsys-card--overdue",
            Urgency::DueSoon => "bsys-card bsys-card--due-soon",
            Urgency::Ok => "bsys-card",
            Urgency::Unknown => "bsys-card bsys-card--unknown",
        }
    }

    fn badge_class(self) -> &'static str {
        match self {
            Urgency::Overdue => "bsys-badge bsys-badge--overdue",
            Urgency::DueSoon => "bsys-badge bsys-badge--due-soon",
            Urgency::Ok => "bsys-badge bsys-badge--ok",
            Urgency::Unknown => "bsys-badge bsys-badge--unknown",
        }
    }

    fn badge_label(self) -> &'static str {
        match self {
            Urgency::Overdue => "Overdue",
            Urgency::DueSoon => "Due Soon",
            Urgency::Ok => "OK",
            Urgency::Unknown => "No Date",
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmt_date(d: Option<&NaiveDate>) -> String {
    d.map(|d| d.format("%b %d, %Y").to_string())
        .unwrap_or_else(|| "—".to_string())
}

fn days_label(d: Option<&NaiveDate>) -> String {
    let today = Utc::now().date_naive();
    match d {
        None => "—".to_string(),
        Some(d) => {
            let diff = (*d - today).num_days();
            if diff < 0 {
                format!("{} days overdue", diff.abs())
            } else if diff == 0 {
                "Today".to_string()
            } else {
                format!("in {} days", diff)
            }
        }
    }
}

/// Extract system_type string from JSONB metadata.
fn system_type(sys: &BuildingSystemDetail) -> String {
    sys.metadata
        .as_ref()
        .and_then(|m| m.get("system_type"))
        .and_then(|v| v.as_str())
        .unwrap_or("other")
        .to_string()
}

/// Human-readable system type label.
fn system_type_label(t: &str) -> &'static str {
    match t {
        "elevator" => "Elevator",
        "escalator" => "Escalator",
        "fire_suppression" => "Fire Suppression",
        "fire_alarm" => "Fire Alarm",
        "emergency_lighting" => "Emergency Lighting",
        "common_area_hvac" => "HVAC",
        "boiler" => "Boiler",
        "cooling_tower" => "Cooling Tower",
        "chiller" => "Chiller",
        "generator" => "Generator",
        "electrical_panel" => "Electrical Panel",
        "transformer_vault" => "Transformer Vault",
        "roof_drain_system" => "Roof Drain",
        "sewer_lift" => "Sewer Lift",
        "backflow_preventer" => "Backflow Preventer",
        "roof" => "Roof",
        "facade" => "Facade",
        "parking_structure" => "Parking Structure",
        "pool" => "Pool",
        "spa" => "Spa",
        "gym_equipment" => "Gym Equipment",
        "security_system" => "Security System",
        "access_control" => "Access Control",
        "intercom" => "Intercom",
        _ => "Other",
    }
}

/// Material icon for each system type.
fn system_icon(t: &str) -> &str {
    match t {
        "elevator" | "escalator" => "elevator",
        "fire_suppression" | "fire_alarm" => "local_fire_department",
        "emergency_lighting" => "emergency_share",
        "common_area_hvac" | "cooling_tower" | "chiller" => "air",
        "boiler" => "heat",
        "generator" => "bolt",
        "electrical_panel" | "transformer_vault" => "electrical_services",
        "roof_drain_system" | "sewer_lift" | "backflow_preventer" => "water_drop",
        "roof" => "roofing",
        "facade" => "apartment",
        "parking_structure" => "local_parking",
        "pool" | "spa" => "pool",
        "gym_equipment" => "fitness_center",
        "security_system" | "access_control" => "security",
        "intercom" => "intercom",
        _ => "settings",
    }
}

fn condition_class(c: Option<&str>) -> &'static str {
    match c {
        Some("Good") | Some("good") => "bsys-cond bsys-cond--good",
        Some("Fair") | Some("fair") => "bsys-cond bsys-cond--fair",
        Some("Poor") | Some("poor") => "bsys-cond bsys-cond--poor",
        Some("Critical") | Some("critical") => "bsys-cond bsys-cond--critical",
        _ => "bsys-cond bsys-cond--unknown",
    }
}

/// Building system type — mirrors backend `BuildingSystemType` snake_case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildingSystemTypeOpt {
    Elevator,
    Escalator,
    FireSuppression,
    FireAlarm,
    EmergencyLighting,
    CommonAreaHvac,
    Boiler,
    CoolingTower,
    Chiller,
    Generator,
    ElectricalPanel,
    TransformerVault,
    RoofDrainSystem,
    SewerLift,
    BackflowPreventer,
    Roof,
    Facade,
    ParkingStructure,
    Pool,
    Spa,
    GymEquipment,
    SecuritySystem,
    AccessControl,
    Intercom,
    Other,
}

impl BuildingSystemTypeOpt {
    pub const ALL: &'static [Self] = &[
        Self::Elevator,
        Self::Escalator,
        Self::FireSuppression,
        Self::FireAlarm,
        Self::EmergencyLighting,
        Self::CommonAreaHvac,
        Self::Boiler,
        Self::CoolingTower,
        Self::Chiller,
        Self::Generator,
        Self::ElectricalPanel,
        Self::TransformerVault,
        Self::RoofDrainSystem,
        Self::SewerLift,
        Self::BackflowPreventer,
        Self::Roof,
        Self::Facade,
        Self::ParkingStructure,
        Self::Pool,
        Self::Spa,
        Self::GymEquipment,
        Self::SecuritySystem,
        Self::AccessControl,
        Self::Intercom,
        Self::Other,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Elevator => "elevator",
            Self::Escalator => "escalator",
            Self::FireSuppression => "fire_suppression",
            Self::FireAlarm => "fire_alarm",
            Self::EmergencyLighting => "emergency_lighting",
            Self::CommonAreaHvac => "common_area_hvac",
            Self::Boiler => "boiler",
            Self::CoolingTower => "cooling_tower",
            Self::Chiller => "chiller",
            Self::Generator => "generator",
            Self::ElectricalPanel => "electrical_panel",
            Self::TransformerVault => "transformer_vault",
            Self::RoofDrainSystem => "roof_drain_system",
            Self::SewerLift => "sewer_lift",
            Self::BackflowPreventer => "backflow_preventer",
            Self::Roof => "roof",
            Self::Facade => "facade",
            Self::ParkingStructure => "parking_structure",
            Self::Pool => "pool",
            Self::Spa => "spa",
            Self::GymEquipment => "gym_equipment",
            Self::SecuritySystem => "security_system",
            Self::AccessControl => "access_control",
            Self::Intercom => "intercom",
            Self::Other => "other",
        }
    }

    pub fn label(self) -> &'static str {
        system_type_label(self.as_str())
    }
}

// ── Server function ───────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(FetchBuildingSystems, "/api")]
pub async fn fetch_building_systems(
) -> Result<Vec<BuildingSystemDetail>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<BuildingSystemDetail>>(
        "/api/folio/systems",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Fetch systems failed: {e}")))
}

/// POST /api/folio/assets/{property_id}/systems
#[server(CreateBuildingSystem, "/api")]
pub async fn create_building_system(
    property_id: String,
    name: String,
    system_type: String,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let property_id = Uuid::parse_str(property_id.trim())
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid property ID"))?;
    if name.trim().is_empty() {
        return Err(server_fn::error::ServerFnError::new("Name is required"));
    }
    if BuildingSystemTypeOpt::ALL
        .iter()
        .all(|t| t.as_str() != system_type.as_str())
    {
        return Err(server_fn::error::ServerFnError::new("Invalid system type"));
    }

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let body = serde_json::json!({
        "property_id": property_id,
        "name": name.trim(),
        "serial_number": null,
        "next_inspection_date": null,
        "cert_expiry_date": null,
        "condition": null,
        "metadata": {
            "metadata_version": 1,
            "system_type": system_type,
            "make": null,
            "model": null,
            "serial_number": null,
            "year_installed": null,
            "certificate_type": "none",
            "certificate_issuer": null,
            "certificate_number": null,
            "contractor_sp_id": null,
            "service_contract_expiry": null,
            "replacement_cost_cents": null,
            "useful_life_years": null,
            "is_building_wide": null,
            "location_notes": null
        }
    });

    #[derive(Deserialize)]
    struct Resp {
        id: Uuid,
    }

    let resp = crate::atlas_client::authenticated_post::<serde_json::Value, Resp>(
        &format!("/api/folio/assets/{property_id}/systems"),
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Create system failed: {e}")))?;
    Ok(resp.id)
}

// ── KPI strip ─────────────────────────────────────────────────────────────────

#[component]
fn BsysKpiStrip(systems: Vec<BuildingSystemDetail>) -> impl IntoView {
    let today = Utc::now().date_naive();
    let total = systems.len();
    let overdue = systems
        .iter()
        .filter(|s| Urgency::from_system(s) == Urgency::Overdue)
        .count();
    let due_soon = systems
        .iter()
        .filter(|s| Urgency::from_system(s) == Urgency::DueSoon)
        .count();
    // Cert expiring ≤30 days
    let cert_exp = systems
        .iter()
        .filter(|s| {
            s.expiry_date
                .map(|d| (d - today).num_days())
                .map(|d| d >= 0 && d <= 30)
                .unwrap_or(false)
        })
        .count();

    view! {
        <div class="bsys-kpi-strip">
            <div class="bsys-kpi-card">
                <span class="bsys-kpi-icon material-symbols-outlined">"settings"</span>
                <div class="bsys-kpi-body">
                    <span class="bsys-kpi-value">{total}</span>
                    <span class="bsys-kpi-label">"Total Systems"</span>
                </div>
            </div>
            <div class="bsys-kpi-card bsys-kpi-card--overdue">
                <span class="bsys-kpi-icon material-symbols-outlined">"warning"</span>
                <div class="bsys-kpi-body">
                    <span class="bsys-kpi-value">{overdue}</span>
                    <span class="bsys-kpi-label">"Overdue"</span>
                </div>
            </div>
            <div class="bsys-kpi-card bsys-kpi-card--due-soon">
                <span class="bsys-kpi-icon material-symbols-outlined">"schedule"</span>
                <div class="bsys-kpi-body">
                    <span class="bsys-kpi-value">{due_soon}</span>
                    <span class="bsys-kpi-label">"Due ≤30 Days"</span>
                </div>
            </div>
            <div class="bsys-kpi-card bsys-kpi-card--cert">
                <span class="bsys-kpi-icon material-symbols-outlined">"verified"</span>
                <div class="bsys-kpi-body">
                    <span class="bsys-kpi-value">{cert_exp}</span>
                    <span class="bsys-kpi-label">"Certs Expiring"</span>
                </div>
            </div>
        </div>
    }
}

// ── System card ───────────────────────────────────────────────────────────────

#[component]
fn BsysCard(sys: BuildingSystemDetail) -> impl IntoView {
    let urgency = Urgency::from_system(&sys);
    let st = system_type(&sys);
    let st_label = system_type_label(&st).to_string();
    let icon = system_icon(&st).to_string();
    let cond_cls = condition_class(sys.condition.as_deref());
    let cond_str = sys
        .condition
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());
    let svc_date = fmt_date(sys.scheduled_service_date.as_ref());
    let svc_label = days_label(sys.scheduled_service_date.as_ref());
    let exp_date = fmt_date(sys.expiry_date.as_ref());
    let make_str = sys
        .metadata
        .as_ref()
        .and_then(|m| m.get("make"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let serial = sys.serial_number.clone();

    view! {
        <div class=urgency.card_class()>
            <div class="bsys-card-header">
                <span class="bsys-card-icon material-symbols-outlined">{icon}</span>
                <div class="bsys-card-title-wrap">
                    <span class="bsys-card-name">{sys.name.clone()}</span>
                    <span class="bsys-card-type">{st_label}</span>
                </div>
                <span class=urgency.badge_class()>{urgency.badge_label()}</span>
            </div>

            <div class="bsys-card-meta">
                <div class="bsys-meta-row">
                    <span class="bsys-meta-icon material-symbols-outlined">"health_and_safety"</span>
                    <span class={cond_cls}>{cond_str}</span>
                </div>

                {make_str.map(|m| view! {
                    <div class="bsys-meta-row">
                        <span class="bsys-meta-icon material-symbols-outlined">"precision_manufacturing"</span>
                        <span class="bsys-meta-val">{m}</span>
                    </div>
                }.into_any())}

                {serial.map(|s| view! {
                    <div class="bsys-meta-row">
                        <span class="bsys-meta-icon material-symbols-outlined">"tag"</span>
                        <span class="bsys-meta-val">{s}</span>
                    </div>
                }.into_any())}
            </div>

            <div class="bsys-card-dates">
                <div class="bsys-date-item">
                    <span class="bsys-date-label">"Next Service"</span>
                    <span class="bsys-date-val">{svc_date}</span>
                    <span class="bsys-date-sub">{svc_label}</span>
                </div>
                <div class="bsys-date-divider"></div>
                <div class="bsys-date-item">
                    <span class="bsys-date-label">"Cert / Expiry"</span>
                    <span class="bsys-date-val">{exp_date}</span>
                </div>
            </div>
        </div>
    }
}

// ── Skeleton ──────────────────────────────────────────────────────────────────

#[component]
fn BsysGridSkeleton() -> impl IntoView {
    view! {
        <div class="bsys-grid">
            {(0..8).map(|_| view! {
                <div class="bsys-card-skel">
                    <div class="bsys-skel bsys-skel--icon"></div>
                    <div class="bsys-skel bsys-skel--title"></div>
                    <div class="bsys-skel bsys-skel--sub"></div>
                    <div class="bsys-skel bsys-skel--date"></div>
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}

// ── Main page ─────────────────────────────────────────────────────────────────

#[component]
pub fn BuildingSystems() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let systems = Resource::new(move || refresh.get(), |_| fetch_building_systems());
    let search = RwSignal::new(String::new());
    let cat_filter = RwSignal::new(SystemCategory::All);
    let show_add = RwSignal::new(false);
    let new_property = RwSignal::new(String::new());
    let new_name = RwSignal::new(String::new());
    let new_type = RwSignal::new(BuildingSystemTypeOpt::Elevator.as_str().to_string());
    let creating = RwSignal::new(false);
    let create_err = RwSignal::new(None::<String>);

    let assets = Resource::new(
        move || show_add.get(),
        |open| async move {
            if !open {
                return Ok::<Vec<AssetPickerItem>, server_fn::error::ServerFnError>(vec![]);
            }
            list_assets_for_picker().await
        },
    );

    let on_create = move |_| {
        let property_id = new_property.get().trim().to_string();
        let name = new_name.get().trim().to_string();
        let system_type = new_type.get();
        if property_id.is_empty() || name.is_empty() {
            create_err.set(Some("Property and name are required.".into()));
            return;
        }
        creating.set(true);
        create_err.set(None);
        spawn_local(async move {
            match create_building_system(property_id, name, system_type).await {
                Ok(_) => {
                    show_add.set(false);
                    new_name.set(String::new());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => create_err.set(Some(e.to_string())),
            }
            creating.set(false);
        });
    };

    view! {
        <div class="bsys-page">
            // Header
            <div class="bsys-header">
                <div class="bsys-header-left">
                    <h1 class="bsys-title">"Building Systems"</h1>
                    <p class="bsys-subtitle">
                        "Elevators, HVAC, and building systems"
                    </p>
                </div>
                <button
                    type="button"
                    class="folio-btn folio-btn--primary press"
                    on:click=move |_| {
                        create_err.set(None);
                        show_add.set(true);
                    }
                >
                    "+ Add System"
                </button>
            </div>

            // KPI strip
            <Suspense fallback=|| view! {
                <div class="bsys-kpi-strip bsys-kpi-strip--loading">
                    {(0..4).map(|_| view! { <div class="bsys-kpi-skel"></div> }).collect::<Vec<_>>()}
                </div>
            }>
                {move || systems.get().map(|res| match res {
                    Ok(data) => view! { <BsysKpiStrip systems=data /> }.into_any(),
                    Err(_)   => view! { <div></div> }.into_any(),
                })}
            </Suspense>

            // Filter bar
            <div class="bsys-filter-bar">
                <div class="bsys-search-wrap">
                    <span class="material-symbols-outlined bsys-search-icon">"search"</span>
                    <input
                        class="bsys-search-input"
                        placeholder="Search by name…"
                        prop:value=move || search.get()
                        on:input=move |e| search.set(event_target_value(&e))
                    />
                </div>
                <div class="bsys-chips">
                    {[
                        SystemCategory::All,
                        SystemCategory::LifeSafety,
                        SystemCategory::Mechanical,
                        SystemCategory::Electrical,
                        SystemCategory::Water,
                        SystemCategory::Structure,
                        SystemCategory::Amenity,
                        SystemCategory::Access,
                        SystemCategory::Other,
                    ].iter().map(|&c| view! {
                        <button
                            class=move || if cat_filter.get() == c {
                                "bsys-chip bsys-chip--active"
                            } else { "bsys-chip" }
                            on:click=move |_| cat_filter.set(c)>
                            {c.label()}
                        </button>
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            // Card grid
            <Suspense fallback=|| view! { <BsysGridSkeleton /> }>
                {move || systems.get().map(|res| match res {
                    Err(e) => view! {
                        <div class="bsys-error">
                            <span class="material-symbols-outlined">"error"</span>
                            <p>"Failed to load systems: " {e.to_string()}</p>
                            <button class="bsys-btn bsys-btn--ghost"
                                on:click=move |_| systems.refetch()>"Retry"</button>
                        </div>
                    }.into_any(),
                    Ok(data) => {
                        let q  = search.get().to_lowercase();
                        let cf = cat_filter.get();
                        let filtered: Vec<BuildingSystemDetail> = data.into_iter()
                            .filter(|s| {
                                let st = system_type(s);
                                cf.matches(&st) && (q.is_empty() || s.name.to_lowercase().contains(&q))
                            })
                            .collect();

                        // Sort: overdue first, due-soon next, ok last
                        let mut filtered = filtered;
                        filtered.sort_by_key(|s| match Urgency::from_system(s) {
                            Urgency::Overdue  => 0i32,
                            Urgency::DueSoon  => 1,
                            Urgency::Unknown  => 2,
                            Urgency::Ok       => 3,
                        });

                        if filtered.is_empty() {
                            view! {
                                <div class="bsys-empty">
                                    <span class="material-symbols-outlined bsys-empty-icon">"settings"</span>
                                    <p class="bsys-empty-title">"No building systems found"</p>
                                    <p class="bsys-empty-sub">
                                        "Register systems from the property asset detail page."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="bsys-grid">
                                    {filtered.into_iter().map(|sys| view! {
                                        <BsysCard sys=sys />
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>

            <Show when=move || show_add.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Add Building System"</h3>
                            <button type="button" class="modal-close" on:click=move |_| show_add.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="form-field">
                                <label class="form-label">"Property *"</label>
                                <select class="form-select" on:change=move |ev| new_property.set(event_target_value(&ev))>
                                    <option value="">"Select property…"</option>
                                    {move || assets.get().and_then(|r| r.ok()).unwrap_or_default().into_iter().map(|a| {
                                        let id = a.id.to_string();
                                        let label = format!("{} ({})", a.name, a.asset_type.replace('_', " "));
                                        view! { <option value=id>{label}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Name *"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    placeholder="Main Elevator"
                                    prop:value=new_name
                                    on:input=move |ev| new_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"System Type *"</label>
                                <select class="form-select" on:change=move |ev| new_type.set(event_target_value(&ev))>
                                    {BuildingSystemTypeOpt::ALL.iter().copied().map(|t| {
                                        view! { <option value=t.as_str()>{t.label()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            {move || create_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button type="button" class="btn btn-ghost" on:click=move |_| show_add.set(false)>"Cancel"</button>
                            <button
                                type="button"
                                class="btn btn-primary"
                                disabled=move || creating.get() || new_name.get().trim().is_empty()
                                on:click=on_create
                            >
                                {move || if creating.get() { "Saving…" } else { "Add System" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
