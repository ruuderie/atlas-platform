//! Vendor work orders — `/v/work-orders`
//! Wired to `GET /api/folio/vendor/work-orders`.

use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkOrderSummary {
    pub id: Uuid,
    pub subject: String,
    pub priority: String,
    pub status: String,
    pub scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub estimated_cost: Option<i64>,
    pub asset_id: Option<Uuid>,
}

fn tone(status: &str) -> StatusPillTone {
    match status.to_ascii_lowercase().as_str() {
        "open" | "in_progress" | "assigned" => StatusPillTone::Warn,
        "completed" => StatusPillTone::Ok,
        _ => StatusPillTone::Neutral,
    }
}

#[component]
pub fn WorkOrders() -> impl IntoView {
    let filter = RwSignal::new("active".to_string());
    let orders = Resource::new(
        move || filter.get(),
        |status| async move { list_vendor_work_orders(status).await },
    );

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Work Orders".to_string())
                subtitle=Signal::derive(|| "Jobs assigned to your vendor account.".to_string())
            >
                <A href=FolioRoute::VendorSchedule.path() attr:class="folio-btn folio-btn--ghost press">
                    "Schedule"
                </A>
            </PageHeader>

            <div class="landlord-filter-chips" style="margin-bottom:1rem;">
                {[("active", "Active"), ("open", "Open"), ("in_progress", "In progress"), ("completed", "Completed"), ("all", "All")]
                    .into_iter()
                    .map(|(v, label)| {
                        let v_for_class = v.to_string();
                        let v_for_click = v.to_string();
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    if filter.get() == v_for_class {
                                        "landlord-chip landlord-chip--active"
                                    } else {
                                        "landlord-chip"
                                    }
                                }
                                on:click=move |_| filter.set(v_for_click.clone())
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading work orders…"</p></div>
            }>
                {move || orders.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load work orders"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(items) if items.is_empty() => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"assignment"</span>
                            <p class="folio-empty__heading">"No work orders"</p>
                            <p class="folio-empty__sub">"Assigned jobs appear here when landlords dispatch you."</p>
                        </div>
                    }.into_any(),
                    Ok(items) => view! {
                        <For
                            each=move || items.clone()
                            key=|o| o.id
                            children=move |o| {
                                let when = o.scheduled_at
                                    .map(|d| d.format("%b %d, %Y %H:%M").to_string())
                                    .unwrap_or_else(|| "Unscheduled".into());
                                view! {
                                    <div class="hub-activity-rail__row">
                                        <StatusPill label=o.status.clone() tone=tone(&o.status)/>
                                        <div class="hub-activity-rail__body">
                                            <p class="hub-activity-rail__row-title">{o.subject.clone()}</p>
                                            <p class="hub-activity-rail__row-meta">
                                                {format!("{} · {}", o.priority, when)}
                                            </p>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(ListVendorWorkOrders, "/api")]
pub async fn list_vendor_work_orders(
    status: String,
) -> Result<Vec<WorkOrderSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let path = format!("/api/folio/vendor/work-orders?status={status}");
    crate::atlas_client::authenticated_get::<Vec<WorkOrderSummary>>(&path, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Vendor WOs failed: {e}")))
}
