// apps/folio/src/pages/vendor/schedule.rs
//
// Vendor Schedule page — /v/schedule
//
// Displays a calendar/chronological timeline view of all active maintenance jobs
// assigned to this vendor that have a `scheduled_at` date set.
//
// Data source:
//   - GET /api/folio/vendor/work-orders?status=active

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkOrderSummary {
    pub id: uuid::Uuid,
    pub subject: String,
    pub priority: String,
    pub status: String,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_cost: Option<i64>,
    pub asset_id: Option<uuid::Uuid>,
}

#[component]
pub fn VendorSchedule() -> impl IntoView {
    let work_orders = Resource::new(
        || (),
        |_| async move { get_vendor_schedule_work_orders().await },
    );

    let (selected_work_order, set_selected_work_order) = signal(Option::<WorkOrderSummary>::None);

    view! {
        <div class="vsch-page">
            <div class="vsch-header">
                <h1 class="vsch-title">"Work Schedule"</h1>
                <p class="vsch-subtitle">"Keep track of your scheduled maintenance visits and work orders."</p>
            </div>

            <Suspense fallback=|| view! { <div class="vsch-loading-skeleton" /> }>
                {move || work_orders.get().map(|res| match res {
                    Err(e) => view! {
                        <div class="vsch-error-state">
                            <span class="material-symbols-outlined vsch-error-icon">"error"</span>
                            <p class="vsch-error-text">{format!("Could not load schedule: {e}")}</p>
                        </div>
                    }.into_any(),
                    Ok(orders) => {
                        // Filter out orders that have no scheduled date
                        let mut scheduled_orders: Vec<WorkOrderSummary> = orders
                            .into_iter()
                            .filter(|o| o.scheduled_at.is_some())
                            .collect();

                        // Sort chronological
                        scheduled_orders.sort_by(|a, b| a.scheduled_at.cmp(&b.scheduled_at));

                        if scheduled_orders.is_empty() {
                            view! {
                                <div class="vsch-empty-state">
                                    <span class="material-symbols-outlined vsch-empty-icon">"calendar_today"</span>
                                    <h3 class="vsch-empty-title">"No Scheduled Work"</h3>
                                    <p class="vsch-empty-sub">"You don't have any upcoming jobs scheduled right now."</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="vsch-content">
                                    // Chronological Schedule Feed
                                    <div class="vsch-timeline">
                                        {scheduled_orders.into_iter().map(|order| {
                                            let local_time = order.scheduled_at.map(|dt| {
                                                dt.with_timezone(&chrono::Local)
                                            });
                                            let date_str = local_time.map(|lt| lt.format("%A, %B %d, %Y").to_string()).unwrap_or_default();
                                            let time_str = local_time.map(|lt| lt.format("%I:%M %p").to_string()).unwrap_or_default();
                                            let priority_cls = if order.priority == "emergency" {
                                                "vsch-priority vsch-priority--emergency"
                                            } else {
                                                "vsch-priority vsch-priority--routine"
                                            };
                                            let status_cls = match order.status.as_str() {
                                                "in_progress" => "vsch-status vsch-status--progress",
                                                _ => "vsch-status vsch-status--open",
                                            };
                                            let asset_name = order.asset_id.map(|id| {
                                                id.to_string().split('-').next().unwrap_or("").to_uppercase()
                                            }).unwrap_or_else(|| "Unspecified Unit".to_string());

                                            let order_clone = order.clone();
                                            view! {
                                                <div class="vsch-card" on:click=move |_| set_selected_work_order.set(Some(order_clone.clone()))>
                                                    <div class="vsch-card-header">
                                                        <span class="vsch-date">{date_str}</span>
                                                        <span class="vsch-time">{time_str}</span>
                                                    </div>
                                                    <div class="vsch-card-body">
                                                        <h3 class="vsch-subject">{order.subject}</h3>
                                                        <div class="vsch-meta-row">
                                                            <span class="vsch-meta-item">
                                                                <span class="material-symbols-outlined" style="font-size: 14px;">"home"</span>
                                                                {asset_name}
                                                            </span>
                                                            <span class=priority_cls>{order.priority}</span>
                                                            <span class=status_cls>{order.status}</span>
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>

                                    // Interactive details side-panel/drawer
                                    {move || selected_work_order.get().map(|order| {
                                        let local_time = order.scheduled_at.map(|dt| {
                                            dt.with_timezone(&chrono::Local)
                                        });
                                        let date_str = local_time.map(|lt| lt.format("%B %d, %Y").to_string()).unwrap_or_default();
                                        let time_str = local_time.map(|lt| lt.format("%I:%M %p").to_string()).unwrap_or_default();
                                        let cost_str = order.estimated_cost.map(|c| format!("${:.2}", c as f64 / 100.0)).unwrap_or_else(|| "TBD".to_string());
                                        let asset_name = order.asset_id.map(|id| id.to_string()).unwrap_or_default();

                                        view! {
                                            <div class="vsch-drawer">
                                                <div class="vsch-drawer-header">
                                                    <h3 class="vsch-drawer-title">"Job Details"</h3>
                                                    <button class="vsch-drawer-close" on:click=move |_| set_selected_work_order.set(None)>
                                                        <span class="material-symbols-outlined">"close"</span>
                                                    </button>
                                                </div>
                                                <div class="vsch-drawer-body">
                                                    <div class="vsch-drawer-section">
                                                        <label class="vsch-drawer-label">"Subject"</label>
                                                        <p class="vsch-drawer-value vsch-drawer-value--bold">{order.subject}</p>
                                                    </div>
                                                    <div class="vsch-drawer-section">
                                                        <label class="vsch-drawer-label">"Scheduled Visit"</label>
                                                        <p class="vsch-drawer-value">{format!("{} at {}", date_str, time_str)}</p>
                                                    </div>
                                                    <div class="vsch-drawer-section">
                                                        <label class="vsch-drawer-label">"Asset / Unit ID"</label>
                                                        <p class="vsch-drawer-value vsch-drawer-value--mono">{asset_name}</p>
                                                    </div>
                                                    <div class="vsch-drawer-section">
                                                        <label class="vsch-drawer-label">"Estimated Cost"</label>
                                                        <p class="vsch-drawer-value">{cost_str}</p>
                                                    </div>
                                                    <div class="vsch-drawer-section">
                                                        <label class="vsch-drawer-label">"Priority"</label>
                                                        <span class=move || {
                                                            if order.priority == "emergency" {
                                                                "vsch-priority vsch-priority--emergency"
                                                            } else {
                                                                "vsch-priority vsch-priority--routine"
                                                            }
                                                        }>{order.priority.clone()}</span>
                                                    </div>
                                                    <div class="vsch-drawer-section">
                                                        <label class="vsch-drawer-label">"Status"</label>
                                                        <span class=move || {
                                                            match order.status.as_str() {
                                                                "in_progress" => "vsch-status vsch-status--progress",
                                                                _ => "vsch-status vsch-status--open",
                                                            }
                                                        }>{order.status.clone()}</span>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    })}
                                </div>
                            }.into_any()
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}

// ── Server Functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(GetVendorScheduleWorkOrders, "/api")]
pub async fn get_vendor_schedule_work_orders(
) -> Result<Vec<WorkOrderSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<WorkOrderSummary>>(
        "/api/folio/vendor/work-orders?status=active",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Schedule query failed: {e}")))
}
