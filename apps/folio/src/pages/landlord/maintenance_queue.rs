// apps/folio/src/pages/landlord/maintenance_queue.rs
//
// Maintenance Queue page — /l/maintenance
//
// Two-tab view for the landlord's active work:
//   Tab 1 "Work Orders"  — reactive maintenance tickets (GET /api/folio/maintenance)
//   Tab 2 "Inspections"  — scheduled proactive inspections (GET /api/folio/inspections)
//
// Category chips let the operator filter by trade (plumbing, electrical, …).
// Priority badge (emergency / routine) drives row colour.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::nav::{FolioRoute, NavIcon};

// ── Response types (mirror backend MaintenanceSummary + InspectionDetail) ─────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaintenanceSummary {
    pub id: uuid::Uuid,
    pub asset_id: Option<uuid::Uuid>,
    pub case_type: String,
    pub subject: String,
    pub status: String,
    pub priority: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InspectionDetail {
    pub id: uuid::Uuid,
    pub asset_id: Option<uuid::Uuid>,
    pub subject: String,
    pub status: String,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub service_provider_id: Option<uuid::Uuid>,
    pub estimated_cost_cents: Option<i64>,
    pub actual_cost_cents: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Active tab on the Maintenance Queue page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaintenanceTab {
    WorkOrders,
    Inspections,
}

impl MaintenanceTab {
    pub const fn label(self) -> &'static str {
        match self {
            Self::WorkOrders => "Work Orders",
            Self::Inspections => "Inspections",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::WorkOrders => "handyman",
            Self::Inspections => "fact_check",
        }
    }
}

/// Maintenance case status — mirrors atlas_case.status values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
    Scheduled,
    Completed,
    Unknown,
}

impl CaseStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "open" => Self::Open,
            "in_progress" => Self::InProgress,
            "resolved" => Self::Resolved,
            "closed" => Self::Closed,
            "scheduled" => Self::Scheduled,
            "completed" => Self::Completed,
            _ => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::InProgress => "In Progress",
            Self::Resolved => "Resolved",
            Self::Closed => "Closed",
            Self::Scheduled => "Scheduled",
            Self::Completed => "Completed",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Open => "mq-status--open",
            Self::InProgress => "mq-status--in-progress",
            Self::Resolved => "mq-status--resolved",
            Self::Closed => "mq-status--closed",
            Self::Scheduled => "mq-status--scheduled",
            Self::Completed => "mq-status--completed",
            Self::Unknown => "mq-status--unknown",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Open => "radio_button_unchecked",
            Self::InProgress => "sync",
            Self::Resolved => "check_circle",
            Self::Closed => "lock",
            Self::Scheduled => "event",
            Self::Completed => "task_alt",
            Self::Unknown => "help",
        }
    }
}

impl std::fmt::Display for CaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Case priority — "emergency" | "routine".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CasePriority {
    Emergency,
    Routine,
    Unknown,
}

impl CasePriority {
    pub fn from_str(s: &str) -> Self {
        match s {
            "emergency" => Self::Emergency,
            "routine" => Self::Routine,
            _ => Self::Unknown,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Emergency => "Emergency",
            Self::Routine => "Routine",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Emergency => "mq-priority--emergency",
            Self::Routine => "mq-priority--routine",
            Self::Unknown => "mq-priority--unknown",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Emergency => "priority_high",
            Self::Routine => "low_priority",
            Self::Unknown => "help",
        }
    }
}

impl std::fmt::Display for CasePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Maintenance trade category — mirrors MaintenanceCategory in backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TradeCategory {
    All,
    Plumbing,
    Electrical,
    Hvac,
    Structural,
    Pest,
    Appliance,
    Roofing,
    General,
}

impl TradeCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Plumbing => "plumbing",
            Self::Electrical => "electrical",
            Self::Hvac => "hvac",
            Self::Structural => "structural",
            Self::Pest => "pest",
            Self::Appliance => "appliance",
            Self::Roofing => "roofing",
            Self::General => "general",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Plumbing => "Plumbing",
            Self::Electrical => "Electrical",
            Self::Hvac => "HVAC",
            Self::Structural => "Structural",
            Self::Pest => "Pest",
            Self::Appliance => "Appliance",
            Self::Roofing => "Roofing",
            Self::General => "General",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::All => "apps",
            Self::Plumbing => "water_drop",
            Self::Electrical => "bolt",
            Self::Hvac => "ac_unit",
            Self::Structural => "foundation",
            Self::Pest => "pest_control",
            Self::Appliance => "kitchen",
            Self::Roofing => "roofing",
            Self::General => "build",
        }
    }
}

// ── Page ──────────────────────────────────────────────────────────────────────

/// Maintenance Queue — work orders and inspections across all assets.
#[component]
pub fn MaintenanceQueue() -> impl IntoView {
    let (active_tab, set_tab) = signal(MaintenanceTab::WorkOrders);
    let (category_filter, set_category) = signal(TradeCategory::All);
    let (priority_filter, set_priority) = signal(Option::<CasePriority>::None);
    let (search_query, set_search) = signal(String::new());

    let tickets = Resource::new(|| (), |_| async move { list_maintenance_tickets().await });

    let inspections = Resource::new(|| (), |_| async move { list_inspections().await });

    let categories = [
        TradeCategory::All,
        TradeCategory::Plumbing,
        TradeCategory::Electrical,
        TradeCategory::Hvac,
        TradeCategory::Structural,
        TradeCategory::Pest,
        TradeCategory::Appliance,
        TradeCategory::Roofing,
        TradeCategory::General,
    ];

    view! {
        <div class="mq-page">
            // ── Header ────────────────────────────────────────────────
            <div class="mq-header">
                <div>
                    <h1 class="mq-title">"Maintenance"</h1>
                    <p class="mq-subtitle">"Work orders and scheduled inspections across your portfolio."</p>
                    <div style="display:flex;gap:0.5rem;flex-wrap:wrap;margin-top:0.75rem;">
                        <a class="folio-btn folio-btn--primary" href=FolioRoute::LandlordMaintenanceNew.path()>
                            "New work order"
                        </a>
                        <a
                            class="folio-btn folio-btn--ghost"
                            href=format!("{}?mode=paid", FolioRoute::LandlordMaintenanceNew.path())
                        >
                            "Log paid"
                        </a>
                        <a
                            class="folio-btn folio-btn--ghost"
                            href=format!("{}?mode=schedule", FolioRoute::LandlordMaintenanceNew.path())
                        >
                            "Schedule"
                        </a>
                        <a class="folio-btn folio-btn--ghost" href=FolioRoute::LandlordRatings.path()>
                            "Ratings"
                        </a>
                    </div>
                </div>
                // KPI badges
                <div class="mq-kpi-strip">
                    <Suspense fallback=|| view! { <div class="mq-kpi-skel"/> }>
                        {move || tickets.get().map(|res| {
                            let (open, emergency) = res.as_ref().map(|v| {
                                let open = v.iter().filter(|t| CaseStatus::from_str(&t.status) == CaseStatus::Open).count();
                                let emg = v.iter().filter(|t| CasePriority::from_str(&t.priority) == CasePriority::Emergency).count();
                                (open, emg)
                            }).unwrap_or((0, 0));
                            view! {
                                <div class="mq-kpi">
                                    <span class="mq-kpi-val">{open}</span>
                                    <span class="mq-kpi-label">"Open"</span>
                                </div>
                                {(emergency > 0).then(|| view! {
                                    <div class="mq-kpi mq-kpi--emergency">
                                        <span class="material-symbols-outlined" style="font-size:14px;">"priority_high"</span>
                                        <span class="mq-kpi-val">{emergency}</span>
                                        <span class="mq-kpi-label">"Emergency"</span>
                                    </div>
                                })}
                            }
                        })}
                    </Suspense>
                    <Suspense fallback=|| view! { <div class="mq-kpi-skel"/> }>
                        {move || inspections.get().map(|res| {
                            let upcoming = res.as_ref().map(|v| {
                                v.iter().filter(|i| {
                                    CaseStatus::from_str(&i.status) == CaseStatus::Scheduled
                                }).count()
                            }).unwrap_or(0);
                            view! {
                                <div class="mq-kpi">
                                    <span class="mq-kpi-val">{upcoming}</span>
                                    <span class="mq-kpi-label">"Upcoming Inspections"</span>
                                </div>
                            }
                        })}
                    </Suspense>
                </div>
            </div>

            // ── Tabs ──────────────────────────────────────────────────
            <div class="mq-tabs">
                {[MaintenanceTab::WorkOrders, MaintenanceTab::Inspections].iter().copied().map(|tab| view! {
                    <button
                        class=move || {
                            if active_tab.get() == tab {
                                "mq-tab mq-tab--active"
                            } else {
                                "mq-tab"
                            }
                        }
                        on:click=move |_| {
                            set_tab.set(tab);
                            set_category.set(TradeCategory::All);
                        }
                    >
                        <span class="material-symbols-outlined" style="font-size:16px;">
                            {tab.material_icon()}
                        </span>
                        {tab.label()}
                    </button>
                }).collect_view()}
            </div>

            // ── Filter bar ────────────────────────────────────────────
            <div class="mq-filter-bar">
                <div class="mq-search-wrap">
                    <span class="material-symbols-outlined mq-search-icon">"search"</span>
                    <input
                        id="mq-search"
                        class="mq-search-input"
                        type="search"
                        placeholder="Search by subject or asset\u{2026}"
                        on:input=move |e| set_search.set(event_target_value(&e))
                    />
                </div>
                // Category chips (only shown on Work Orders tab)
                <Show when=move || active_tab.get() == MaintenanceTab::WorkOrders fallback=|| ()>
                    <div class="mq-category-chips">
                        {categories.iter().copied().map(|cat| view! {
                            <button
                                class=move || {
                                    if category_filter.get() == cat {
                                        "mq-cat-chip mq-cat-chip--active"
                                    } else {
                                        "mq-cat-chip"
                                    }
                                }
                                on:click=move |_| set_category.set(cat)
                            >
                                <span class="material-symbols-outlined" style="font-size:13px;">
                                    {cat.material_icon()}
                                </span>
                                {cat.label()}
                            </button>
                        }).collect_view()}
                    </div>
                </Show>
                // Priority filter (only shown on Work Orders tab)
                <Show when=move || active_tab.get() == MaintenanceTab::WorkOrders fallback=|| ()>
                    <div class="mq-priority-chips">
                        <button
                            class=move || if priority_filter.get().is_none() { "mq-priority-chip mq-priority-chip--active" } else { "mq-priority-chip" }
                            on:click=move |_| set_priority.set(None)
                        >"All priorities"</button>
                        {[CasePriority::Emergency, CasePriority::Routine].iter().copied().map(|p| view! {
                            <button
                                class=move || {
                                    if priority_filter.get() == Some(p) {
                                        "mq-priority-chip mq-priority-chip--active"
                                    } else {
                                        "mq-priority-chip"
                                    }
                                }
                                on:click=move |_| set_priority.set(Some(p))
                            >
                                <span class="material-symbols-outlined" style="font-size:12px;">
                                    {p.material_icon()}
                                </span>
                                {p.as_str()}
                            </button>
                        }).collect_view()}
                    </div>
                </Show>
            </div>

            // ── Content ───────────────────────────────────────────────
            {move || match active_tab.get() {
                MaintenanceTab::WorkOrders => view! {
                    <Suspense fallback=|| view! { <MqSkeleton rows=8/> }>
                        {move || tickets.get().map(|res| match res {
                            Err(e) => view! {
                                <div class="mq-error">
                                    <p class="mq-error-text">{format!("Could not load tickets: {e}")}</p>
                                </div>
                            }.into_any(),
                            Ok(all) => {
                                let q   = search_query.get().to_lowercase();
                                let cat = category_filter.get();
                                let pri = priority_filter.get();

                                let filtered: Vec<MaintenanceSummary> = all.into_iter().filter(|t| {
                                    let search_ok = q.is_empty()
                                        || t.subject.to_lowercase().contains(&q)
                                        || t.id.to_string().contains(&q);
                                    let cat_ok = cat == TradeCategory::All
                                        || t.case_type.contains(cat.as_str());
                                    let pri_ok = pri.map(|p| CasePriority::from_str(&t.priority) == p)
                                        .unwrap_or(true);
                                    search_ok && cat_ok && pri_ok
                                }).collect();

                                if filtered.is_empty() {
                                    view! {
                                        <div class="mq-empty">
                                            <span class="material-symbols-outlined mq-empty-icon">
                                                {NavIcon::Build.as_str()}
                                            </span>
                                            <p class="mq-empty-title">"No work orders"</p>
                                            <p class="mq-empty-sub">"Adjust filters or create a new work order."</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <WorkOrderTable tickets=filtered/> }.into_any()
                                }
                            }
                        })}
                    </Suspense>
                }.into_any(),
                MaintenanceTab::Inspections => view! {
                    <Suspense fallback=|| view! { <MqSkeleton rows=5/> }>
                        {move || inspections.get().map(|res| match res {
                            Err(e) => view! {
                                <div class="mq-error">
                                    <p class="mq-error-text">{format!("Could not load inspections: {e}")}</p>
                                </div>
                            }.into_any(),
                            Ok(all) => {
                                let q = search_query.get().to_lowercase();
                                let filtered: Vec<InspectionDetail> = all.into_iter().filter(|i| {
                                    q.is_empty() || i.subject.to_lowercase().contains(&q)
                                }).collect();

                                if filtered.is_empty() {
                                    view! {
                                        <div class="mq-empty">
                                            <span class="material-symbols-outlined mq-empty-icon">
                                                "fact_check"
                                            </span>
                                            <p class="mq-empty-title">"No scheduled inspections"</p>
                                            <p class="mq-empty-sub">"Inspections you schedule will appear here."</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <InspectionTable inspections=filtered/> }.into_any()
                                }
                            }
                        })}
                    </Suspense>
                }.into_any(),
            }}
        </div>
    }
}

// ── Work Order Table ──────────────────────────────────────────────────────────

#[component]
fn WorkOrderTable(tickets: Vec<MaintenanceSummary>) -> impl IntoView {
    view! {
        <div class="mq-table-wrap">
            <table class="mq-table">
                <thead>
                    <tr>
                        <th class="mq-th">"Priority"</th>
                        <th class="mq-th">"Status"</th>
                        <th class="mq-th">"Subject"</th>
                        <th class="mq-th">"Asset"</th>
                        <th class="mq-th">"Opened"</th>
                    </tr>
                </thead>
                <tbody>
                    {tickets.into_iter().map(|t| {
                        let priority = CasePriority::from_str(&t.priority);
                        let status   = CaseStatus::from_str(&t.status);
                        let asset    = t.asset_id.map(|id| id.to_string().split('-').next().unwrap_or("").to_uppercase()).unwrap_or_else(|| "\u{2014}".to_string());
                        let opened   = t.created_at.format("%Y-%m-%d").to_string();
                        let row_class = if priority == CasePriority::Emergency {
                            "mq-row mq-row--emergency"
                        } else {
                            "mq-row"
                        };
                        let href = FolioRoute::LandlordMaintenanceDetail
                            .path()
                            .replace(":id", &t.id.to_string());
                        view! {
                            <tr class=row_class>
                                <td class="mq-td">
                                    <span class={format!("mq-priority-badge {}", priority.pill_class())}>
                                        <span class="material-symbols-outlined" style="font-size:11px;">
                                            {priority.material_icon()}
                                        </span>
                                        {priority.as_str()}
                                    </span>
                                </td>
                                <td class="mq-td">
                                    <span class={format!("mq-status-badge {}", status.pill_class())}>
                                        <span class="material-symbols-outlined" style="font-size:11px;">
                                            {status.material_icon()}
                                        </span>
                                        {status.as_str()}
                                    </span>
                                </td>
                                <td class="mq-td mq-td--subject">
                                    <a class="hub-activity-rail__all" href=href>{t.subject}</a>
                                </td>
                                <td class="mq-td mq-td--mono">{asset}</td>
                                <td class="mq-td">{opened}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

// ── Inspection Table ──────────────────────────────────────────────────────────

#[component]
fn InspectionTable(inspections: Vec<InspectionDetail>) -> impl IntoView {
    view! {
        <div class="mq-table-wrap">
            <table class="mq-table">
                <thead>
                    <tr>
                        <th class="mq-th">"Status"</th>
                        <th class="mq-th">"Subject"</th>
                        <th class="mq-th">"Asset"</th>
                        <th class="mq-th">"Scheduled"</th>
                        <th class="mq-th">"Completed"</th>
                        <th class="mq-th">"Est. Cost"</th>
                    </tr>
                </thead>
                <tbody>
                    {inspections.into_iter().map(|i| {
                        let status    = CaseStatus::from_str(&i.status);
                        let asset     = i.asset_id.map(|id| id.to_string().split('-').next().unwrap_or("").to_uppercase()).unwrap_or_else(|| "\u{2014}".to_string());
                        let scheduled = i.scheduled_at.map(|d| d.format("%Y-%m-%d %H:%M").to_string()).unwrap_or_else(|| "\u{2014}".to_string());
                        let completed = i.completed_at.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_else(|| "\u{2014}".to_string());
                        let cost      = i.estimated_cost_cents.map(|c| format!("${:.0}", c as f64 / 100.0)).unwrap_or_else(|| "\u{2014}".to_string());
                        view! {
                            <tr class="mq-row">
                                <td class="mq-td">
                                    <span class={format!("mq-status-badge {}", status.pill_class())}>
                                        <span class="material-symbols-outlined" style="font-size:11px;">
                                            {status.material_icon()}
                                        </span>
                                        {status.as_str()}
                                    </span>
                                </td>
                                <td class="mq-td mq-td--subject">{i.subject}</td>
                                <td class="mq-td mq-td--mono">{asset}</td>
                                <td class="mq-td">{scheduled}</td>
                                <td class="mq-td">{completed}</td>
                                <td class="mq-td mq-td--cost">{cost}</td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

// ── Skeleton ──────────────────────────────────────────────────────────────────

#[component]
fn MqSkeleton(rows: usize) -> impl IntoView {
    view! {
        <div class="mq-table-wrap">
            <table class="mq-table">
                <thead>
                    <tr>
                        <th class="mq-th">"Priority"</th>
                        <th class="mq-th">"Status"</th>
                        <th class="mq-th">"Subject"</th>
                        <th class="mq-th">"Asset"</th>
                        <th class="mq-th">"Opened"</th>
                    </tr>
                </thead>
                <tbody>
                    {(0..rows).map(|_| view! {
                        <tr class="mq-row">
                            <td class="mq-td"><div class="mq-skel mq-skel--badge"/></td>
                            <td class="mq-td"><div class="mq-skel mq-skel--badge"/></td>
                            <td class="mq-td"><div class="mq-skel mq-skel--text"/></td>
                            <td class="mq-td"><div class="mq-skel mq-skel--sm"/></td>
                            <td class="mq-td"><div class="mq-skel mq-skel--sm"/></td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

// ── Server functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

/// GET /api/folio/maintenance
#[server(ListMaintenanceTickets, "/api")]
pub async fn list_maintenance_tickets(
) -> Result<Vec<MaintenanceSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<MaintenanceSummary>>(
        "/api/folio/maintenance",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Maintenance list failed: {e}")))
}

/// GET /api/folio/inspections
#[server(ListInspections, "/api")]
pub async fn list_inspections() -> Result<Vec<InspectionDetail>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<InspectionDetail>>(
        "/api/folio/inspections",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Inspections list failed: {e}")))
}
