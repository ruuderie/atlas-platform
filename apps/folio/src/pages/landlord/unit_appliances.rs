// apps/folio/src/pages/landlord/unit_appliances.rs
//
// Unit Appliances page — /l/appliances
//
// Portfolio-wide view of all registered appliances across all units.
// Shows lifecycle urgency (warranty expiry, next service date), condition,
// make/model, and fuel type.
//
// ─── Layout ───────────────────────────────────────────────────────────────────
//  KPI strip   : Total | Service Due ≤30d | Warranty Expiring ≤30d | Overdue
//  Filter bar  : search + appliance-type chips
//  Card grid   : appliance name · type icon · condition · make/model ·
//                next service · warranty expiry
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use uuid::Uuid;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplianceDetail {
    pub id:                     Uuid,
    pub tenant_id:              Uuid,
    pub unit_id:                Option<Uuid>,
    pub name:                   String,
    pub serial_number:          Option<String>,
    pub status:                 String,
    pub condition:              Option<String>,
    pub scheduled_service_date: Option<NaiveDate>,
    pub expiry_date:            Option<NaiveDate>,
    pub metadata:               Option<serde_json::Value>,
    pub created_at:             chrono::DateTime<Utc>,
}

// ── Local enums ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum ApplianceCategory {
    All,
    Kitchen,
    Laundry,
    Hvac,
    Water,
    Garage,
    Other,
}

impl ApplianceCategory {
    fn label(self) -> &'static str {
        match self {
            ApplianceCategory::All     => "All",
            ApplianceCategory::Kitchen => "Kitchen",
            ApplianceCategory::Laundry => "Laundry",
            ApplianceCategory::Hvac    => "HVAC",
            ApplianceCategory::Water   => "Water",
            ApplianceCategory::Garage  => "Garage",
            ApplianceCategory::Other   => "Other",
        }
    }

    fn from_type(t: &str) -> Self {
        match t {
            "refrigerator" | "dishwasher" | "oven_range" | "garbage_disposal"
                => ApplianceCategory::Kitchen,
            "washer" | "dryer" | "washer_dryer_combo"
                => ApplianceCategory::Laundry,
            "hvac_unit" | "air_handler" | "boiler"
                => ApplianceCategory::Hvac,
            "water_heater" | "water_softener" | "pool_pump"
                => ApplianceCategory::Water,
            "garage_door_opener"
                => ApplianceCategory::Garage,
            _ => ApplianceCategory::Other,
        }
    }

    fn matches(self, appliance_type: &str) -> bool {
        self == ApplianceCategory::All
            || self == ApplianceCategory::from_type(appliance_type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Urgency {
    Overdue,
    DueSoon,
    Ok,
    Unknown,
}

impl Urgency {
    fn from_appliance(a: &ApplianceDetail) -> Self {
        let today = Utc::now().date_naive();
        let dates: Vec<NaiveDate> = [a.scheduled_service_date, a.expiry_date]
            .into_iter()
            .flatten()
            .collect();
        if dates.is_empty() { return Urgency::Unknown; }
        let earliest = dates.iter().cloned().min().unwrap();
        let days = (earliest - today).num_days();
        if days < 0        { Urgency::Overdue }
        else if days <= 30 { Urgency::DueSoon }
        else               { Urgency::Ok }
    }

    fn card_class(self) -> &'static str {
        match self {
            Urgency::Overdue  => "appl-card appl-card--overdue",
            Urgency::DueSoon  => "appl-card appl-card--due-soon",
            Urgency::Ok       => "appl-card",
            Urgency::Unknown  => "appl-card appl-card--unknown",
        }
    }

    fn badge_class(self) -> &'static str {
        match self {
            Urgency::Overdue  => "appl-badge appl-badge--overdue",
            Urgency::DueSoon  => "appl-badge appl-badge--due-soon",
            Urgency::Ok       => "appl-badge appl-badge--ok",
            Urgency::Unknown  => "appl-badge appl-badge--unknown",
        }
    }

    fn badge_label(self) -> &'static str {
        match self {
            Urgency::Overdue  => "Overdue",
            Urgency::DueSoon  => "Due Soon",
            Urgency::Ok       => "OK",
            Urgency::Unknown  => "No Date",
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
        None    => "—".to_string(),
        Some(d) => {
            let diff = (*d - today).num_days();
            if diff < 0       { format!("{} days overdue", diff.abs()) }
            else if diff == 0 { "Today".to_string() }
            else              { format!("in {} days", diff) }
        }
    }
}

fn appliance_type(a: &ApplianceDetail) -> String {
    a.metadata
        .as_ref()
        .and_then(|m| m.get("appliance_type"))
        .and_then(|v| v.as_str())
        .unwrap_or("other")
        .to_string()
}

fn appliance_type_label(t: &str) -> &str {
    match t {
        "refrigerator"       => "Refrigerator",
        "washer"             => "Washer",
        "dryer"              => "Dryer",
        "washer_dryer_combo" => "Washer/Dryer",
        "water_heater"       => "Water Heater",
        "boiler"             => "Boiler",
        "hvac_unit"          => "HVAC Unit",
        "dishwasher"         => "Dishwasher",
        "oven_range"         => "Oven / Range",
        "garbage_disposal"   => "Garbage Disposal",
        "pool_pump"          => "Pool Pump",
        "garage_door_opener" => "Garage Door Opener",
        "air_handler"        => "Air Handler",
        "water_softener"     => "Water Softener",
        _                    => "Appliance",
    }
}

fn appliance_icon(t: &str) -> &str {
    match t {
        "refrigerator"                => "kitchen",
        "washer" | "washer_dryer_combo" => "local_laundry_service",
        "dryer"                       => "dry",
        "water_heater"                => "water_heater",
        "boiler"                      => "heat",
        "hvac_unit" | "air_handler"   => "air",
        "dishwasher"                  => "dishwasher_gen",
        "oven_range"                  => "oven_gen",
        "garbage_disposal"            => "delete_sweep",
        "pool_pump"                   => "pool",
        "garage_door_opener"          => "garage_door",
        "water_softener"              => "water_drop",
        _                             => "construction",
    }
}

fn condition_class(c: Option<&str>) -> &'static str {
    match c {
        Some("Good") | Some("good")         => "appl-cond appl-cond--good",
        Some("Fair") | Some("fair")         => "appl-cond appl-cond--fair",
        Some("Poor") | Some("poor")         => "appl-cond appl-cond--poor",
        Some("Critical") | Some("critical") => "appl-cond appl-cond--critical",
        _                                   => "appl-cond appl-cond--unknown",
    }
}

fn meta_str<'a>(m: Option<&'a serde_json::Value>, key: &str) -> Option<String> {
    m?.get(key)?.as_str().map(|s| s.to_string())
}

// ── Server function ───────────────────────────────────────────────────────────

#[server(FetchAppliances, "/api")]
pub async fn fetch_appliances() -> Result<Vec<ApplianceDetail>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = headers.get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find_map(|p| {
            let p = p.trim();
            p.strip_prefix("session=").map(|t| t.to_string())
        }))
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<ApplianceDetail>>(
        "/api/folio/appliances",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Fetch appliances failed: {e}")))
}

// ── KPI strip ─────────────────────────────────────────────────────────────────

#[component]
fn ApplKpiStrip(appliances: Vec<ApplianceDetail>) -> impl IntoView {
    let today    = Utc::now().date_naive();
    let total    = appliances.len();
    let overdue  = appliances.iter().filter(|a| Urgency::from_appliance(a) == Urgency::Overdue).count();
    let due_soon = appliances.iter().filter(|a| Urgency::from_appliance(a) == Urgency::DueSoon).count();
    let warranty = appliances.iter().filter(|a| {
        a.expiry_date
            .map(|d| { let n = (d - today).num_days(); n >= 0 && n <= 30 })
            .unwrap_or(false)
    }).count();

    view! {
        <div class="appl-kpi-strip">
            <div class="appl-kpi-card">
                <span class="appl-kpi-icon material-symbols-outlined">"construction"</span>
                <div class="appl-kpi-body">
                    <span class="appl-kpi-value">{total}</span>
                    <span class="appl-kpi-label">"Total Appliances"</span>
                </div>
            </div>
            <div class="appl-kpi-card appl-kpi-card--overdue">
                <span class="appl-kpi-icon material-symbols-outlined">"warning"</span>
                <div class="appl-kpi-body">
                    <span class="appl-kpi-value">{overdue}</span>
                    <span class="appl-kpi-label">"Overdue"</span>
                </div>
            </div>
            <div class="appl-kpi-card appl-kpi-card--due-soon">
                <span class="appl-kpi-icon material-symbols-outlined">"schedule"</span>
                <div class="appl-kpi-body">
                    <span class="appl-kpi-value">{due_soon}</span>
                    <span class="appl-kpi-label">"Due ≤30 Days"</span>
                </div>
            </div>
            <div class="appl-kpi-card appl-kpi-card--warranty">
                <span class="appl-kpi-icon material-symbols-outlined">"verified_user"</span>
                <div class="appl-kpi-body">
                    <span class="appl-kpi-value">{warranty}</span>
                    <span class="appl-kpi-label">"Warranty Expiring"</span>
                </div>
            </div>
        </div>
    }
}

// ── Appliance card ────────────────────────────────────────────────────────────

#[component]
fn ApplCard(appl: ApplianceDetail) -> impl IntoView {
    let urgency    = Urgency::from_appliance(&appl);
    let at         = appliance_type(&appl);
    let at_label   = appliance_type_label(&at).to_string();
    let icon       = appliance_icon(&at).to_string();
    let cond_cls   = condition_class(appl.condition.as_deref());
    let cond_str   = appl.condition.clone().unwrap_or_else(|| "Unknown".to_string());
    let svc_date   = fmt_date(appl.scheduled_service_date.as_ref());
    let svc_lbl    = days_label(appl.scheduled_service_date.as_ref());
    let war_date   = fmt_date(appl.expiry_date.as_ref());
    let m          = appl.metadata.as_ref();
    let make       = meta_str(m, "make");
    let model      = meta_str(m, "model");
    let year       = m.and_then(|m| m.get("year_manufactured"))
                      .and_then(|v| v.as_u64())
                      .map(|y| y.to_string());
    let fuel       = meta_str(m, "fuel_type");

    let make_model: Option<String> = match (make, model) {
        (Some(mk), Some(md)) => Some(format!("{mk} {md}")),
        (Some(mk), None)     => Some(mk),
        (None, Some(md))     => Some(md),
        (None, None)         => None,
    };

    view! {
        <div class=urgency.card_class()>
            <div class="appl-card-header">
                <span class="appl-card-icon material-symbols-outlined">{icon}</span>
                <div class="appl-card-title-wrap">
                    <span class="appl-card-name">{appl.name.clone()}</span>
                    <span class="appl-card-type">{at_label}</span>
                </div>
                <span class=urgency.badge_class()>{urgency.badge_label()}</span>
            </div>

            <div class="appl-card-meta">
                <div class="appl-meta-row">
                    <span class="appl-meta-icon material-symbols-outlined">"health_and_safety"</span>
                    <span class={cond_cls}>{cond_str}</span>
                </div>

                {make_model.map(|mm| view! {
                    <div class="appl-meta-row">
                        <span class="appl-meta-icon material-symbols-outlined">"precision_manufacturing"</span>
                        <span class="appl-meta-val">{mm}</span>
                    </div>
                }.into_any())}

                {year.map(|y| view! {
                    <div class="appl-meta-row">
                        <span class="appl-meta-icon material-symbols-outlined">"calendar_today"</span>
                        <span class="appl-meta-val">"Mfr. " {y}</span>
                    </div>
                }.into_any())}

                {fuel.map(|f| view! {
                    <div class="appl-meta-row">
                        <span class="appl-meta-icon material-symbols-outlined">"local_gas_station"</span>
                        <span class="appl-meta-val">{f.replace('_', " ")}</span>
                    </div>
                }.into_any())}
            </div>

            <div class="appl-card-dates">
                <div class="appl-date-item">
                    <span class="appl-date-label">"Next Service"</span>
                    <span class="appl-date-val">{svc_date}</span>
                    <span class="appl-date-sub">{svc_lbl}</span>
                </div>
                <div class="appl-date-divider"></div>
                <div class="appl-date-item">
                    <span class="appl-date-label">"Warranty / Expiry"</span>
                    <span class="appl-date-val">{war_date}</span>
                </div>
            </div>
        </div>
    }
}

// ── Skeleton ──────────────────────────────────────────────────────────────────

#[component]
fn ApplGridSkeleton() -> impl IntoView {
    view! {
        <div class="appl-grid">
            {(0..8).map(|_| view! {
                <div class="appl-card-skel">
                    <div class="appl-skel appl-skel--icon"></div>
                    <div class="appl-skel appl-skel--title"></div>
                    <div class="appl-skel appl-skel--sub"></div>
                    <div class="appl-skel appl-skel--meta"></div>
                    <div class="appl-skel appl-skel--date"></div>
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
}

// ── Main page ─────────────────────────────────────────────────────────────────

#[component]
pub fn UnitAppliances() -> impl IntoView {
    let refetch_count = RwSignal::new(0u32);
    let appliances    = Resource::new(
        move || refetch_count.get(),
        |_| fetch_appliances(),
    );
    let search     = RwSignal::new(String::new());
    let cat_filter = RwSignal::new(ApplianceCategory::All);

    view! {
        <div class="appl-page">
            // Header
            <div class="appl-header">
                <div class="appl-header-left">
                    <h1 class="appl-title">"Unit Appliances"</h1>
                    <p class="appl-subtitle">
                        "Lifecycle tracking for all appliances across your entire portfolio"
                    </p>
                </div>
            </div>

            // KPI strip
            <Suspense fallback=|| view! {
                <div class="appl-kpi-strip appl-kpi-strip--loading">
                    {(0..4).map(|_| view! { <div class="appl-kpi-skel"></div> }).collect::<Vec<_>>()}
                </div>
            }>
                {move || appliances.get().map(|res| match res {
                    Ok(data) => view! { <ApplKpiStrip appliances=data /> }.into_any(),
                    Err(_)   => view! { <div></div> }.into_any(),
                })}
            </Suspense>

            // Filter bar
            <div class="appl-filter-bar">
                <div class="appl-search-wrap">
                    <span class="material-symbols-outlined appl-search-icon">"search"</span>
                    <input
                        class="appl-search-input"
                        placeholder="Search by name…"
                        prop:value=move || search.get()
                        on:input=move |e| search.set(event_target_value(&e))
                    />
                </div>
                <div class="appl-chips">
                    {[
                        ApplianceCategory::All,
                        ApplianceCategory::Kitchen,
                        ApplianceCategory::Laundry,
                        ApplianceCategory::Hvac,
                        ApplianceCategory::Water,
                        ApplianceCategory::Garage,
                        ApplianceCategory::Other,
                    ].iter().map(|&c| view! {
                        <button
                            class=move || if cat_filter.get() == c {
                                "appl-chip appl-chip--active"
                            } else { "appl-chip" }
                            on:click=move |_| cat_filter.set(c)>
                            {c.label()}
                        </button>
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            // Card grid
            <Suspense fallback=|| view! { <ApplGridSkeleton /> }>
                {move || appliances.get().map(|res| match res {
                    Err(e) => view! {
                        <div class="appl-error">
                            <span class="material-symbols-outlined">"error"</span>
                            <p>"Failed to load appliances: " {e.to_string()}</p>
                            <button class="appl-btn appl-btn--ghost"
                                on:click=move |_| appliances.refetch()>"Retry"</button>
                        </div>
                    }.into_any(),
                    Ok(data) => {
                        let q  = search.get().to_lowercase();
                        let cf = cat_filter.get();
                        let mut filtered: Vec<ApplianceDetail> = data.into_iter()
                            .filter(|a| {
                                let at = appliance_type(a);
                                cf.matches(&at) && (q.is_empty() || a.name.to_lowercase().contains(&q))
                            })
                            .collect();

                        // Sort: overdue first, then due-soon, unknown, ok
                        filtered.sort_by_key(|a| match Urgency::from_appliance(a) {
                            Urgency::Overdue  => 0i32,
                            Urgency::DueSoon  => 1,
                            Urgency::Unknown  => 2,
                            Urgency::Ok       => 3,
                        });

                        if filtered.is_empty() {
                            view! {
                                <div class="appl-empty">
                                    <span class="material-symbols-outlined appl-empty-icon">"construction"</span>
                                    <p class="appl-empty-title">"No appliances found"</p>
                                    <p class="appl-empty-sub">
                                        "Register appliances from the unit asset detail page."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="appl-grid">
                                    {filtered.into_iter().map(|appl| view! {
                                        <ApplCard appl=appl />
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
