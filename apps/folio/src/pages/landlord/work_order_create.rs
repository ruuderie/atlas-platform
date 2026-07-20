//! Work order create sheet page — `/l/maintenance/new`
//! Modes: work order | schedule | log paid. Optional project_id query.

use crate::components::interruptible_sheet::InterruptibleSheet;
use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::pages::landlord::assets::{list_assets, AssetSummary};
use crate::utils::format_asset_place_label;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};
use serde::Serialize;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CreateMode {
    WorkOrder,
    Schedule,
    LogPaid,
}

#[derive(Serialize)]
struct CreateTicketBody {
    asset_id: Uuid,
    reported_by_user_id: Uuid,
    category: String,
    description: String,
    is_emergency: bool,
    voice_note_r2_key: Option<String>,
}

#[derive(Serialize)]
struct LogPaidBody {
    asset_id: Uuid,
    subject: String,
    description: Option<String>,
    actual_cost_cents: i64,
    service_provider_id: Option<Uuid>,
    project_id: Option<Uuid>,
}

#[derive(Serialize)]
struct ScheduleBody {
    asset_id: Uuid,
    subject: String,
    notes: Option<String>,
    scheduled_at: String,
    estimated_cost_cents: Option<i64>,
}

#[derive(serde::Deserialize)]
struct IdResp {
    id: Uuid,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(CreateMaintenanceTicket, "/api")]
async fn create_maintenance_ticket(
    asset_id: Uuid,
    reported_by_user_id: Uuid,
    category: String,
    description: String,
    is_emergency: bool,
) -> Result<Uuid, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    let body = CreateTicketBody {
        asset_id,
        reported_by_user_id,
        category,
        description,
        is_emergency,
        voice_note_r2_key: None,
    };
    let resp: IdResp = crate::atlas_client::authenticated_post(
        "/api/folio/maintenance",
        &token,
        None,
        &body,
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(resp.id)
}

#[server(ScheduleMaintenanceVisit, "/api")]
async fn schedule_maintenance_visit(
    asset_id: Uuid,
    subject: String,
    notes: Option<String>,
    scheduled_at: String,
) -> Result<Uuid, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    let body = ScheduleBody {
        asset_id,
        subject,
        notes,
        scheduled_at,
        estimated_cost_cents: None,
    };
    let resp: IdResp = crate::atlas_client::authenticated_post(
        "/api/folio/inspections",
        &token,
        None,
        &body,
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(resp.id)
}

#[server(LogPaidMaintenance, "/api")]
async fn log_paid_maintenance(
    asset_id: Uuid,
    subject: String,
    description: Option<String>,
    actual_cost_cents: i64,
    project_id: Option<Uuid>,
) -> Result<Uuid, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    let body = LogPaidBody {
        asset_id,
        subject,
        description,
        actual_cost_cents,
        service_provider_id: None,
        project_id,
    };
    let resp: IdResp = crate::atlas_client::authenticated_post(
        "/api/folio/maintenance/log-paid",
        &token,
        None,
        &body,
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(resp.id)
}

#[component]
pub fn WorkOrderCreate() -> impl IntoView {
    let q = use_query_map();
    let navigate = use_navigate();
    let mode = RwSignal::new(CreateMode::WorkOrder);
    // Keep open when landing (with or without query); dismiss navigates away.
    let open = RwSignal::new(true);
    let subject = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let asset_id = RwSignal::new(String::new());
    let cost = RwSignal::new(String::new());
    let scheduled_date = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let saving = RwSignal::new(false);
    let assets = Resource::new(|| (), |_| async move { list_assets().await });

    Effect::new(move |_| {
        let map = q.get();
        match map.get("mode").as_deref() {
            Some("paid") => mode.set(CreateMode::LogPaid),
            Some("schedule") => mode.set(CreateMode::Schedule),
            _ => {}
        }
        if let Some(aid) = map.get("asset_id") {
            if !aid.is_empty() {
                asset_id.set(aid);
            }
        }
    });

    // Never leave a blank shell after dismiss / Escape / overlay close.
    Effect::new({
        let navigate = navigate.clone();
        move |_| {
            if !open.get() {
                navigate(
                    FolioRoute::LandlordMaintenance.path(),
                    Default::default(),
                );
            }
        }
    });

    let project_id = Memo::new(move |_| {
        q.get()
            .get("project")
            .and_then(|s| Uuid::parse_str(&s).ok())
    });

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "New work order".to_string())
                subtitle=Signal::derive(|| "Create · Schedule · Log paid".to_string())
            />
            <InterruptibleSheet
                open=open
                title=Signal::derive(move || match mode.get() {
                    CreateMode::WorkOrder => "Create work order".to_string(),
                    CreateMode::Schedule => "Schedule visit".to_string(),
                    CreateMode::LogPaid => "Log paid (off-platform)".to_string(),
                })
                subtitle=Signal::derive(move || match mode.get() {
                    CreateMode::Schedule => {
                        "Scheduled inspection — date required.".to_string()
                    }
                    CreateMode::LogPaid => "Record a completed off-platform job.".to_string(),
                    CreateMode::WorkOrder => "Scope + optional project".to_string(),
                })
            >
                <div style="display:flex;gap:0.5rem;margin-bottom:1rem;">
                    <button type="button" class="folio-btn folio-btn--ghost" on:click=move |_| mode.set(CreateMode::WorkOrder)>"Work order"</button>
                    <button type="button" class="folio-btn folio-btn--ghost" on:click=move |_| mode.set(CreateMode::Schedule)>"Schedule"</button>
                    <button type="button" class="folio-btn folio-btn--ghost" on:click=move |_| mode.set(CreateMode::LogPaid)>"Log paid"</button>
                </div>
                <label class="proj-section__hint">"Unit / property"</label>
                <Suspense fallback=|| view! { <p class="proj-section__hint">"Loading assets…"</p> }>
                    {move || assets.get().map(|res| match res {
                        Ok(list) => view! {
                            <select
                                class="landlord-search-input"
                                style="width:100%;margin-bottom:0.75rem;"
                                prop:value=move || asset_id.get()
                                on:change=move |ev| asset_id.set(event_target_value(&ev))
                            >
                                <option value="">"Select…"</option>
                                {list.into_iter().map(|a: AssetSummary| {
                                    let id = a.id.to_string();
                                    let label = format_asset_place_label(
                                        &a.name,
                                        a.address_line_1.as_deref(),
                                        a.city.as_deref(),
                                        a.state_province.as_deref(),
                                    );
                                    view! { <option value=id>{label}</option> }
                                }).collect_view()}
                            </select>
                        }.into_any(),
                        Err(e) => view! { <p style="color:#93000a;">{e.to_string()}</p> }.into_any(),
                    })}
                </Suspense>
                <label class="proj-section__hint">"Subject"</label>
                <input class="landlord-search-input" style="width:100%;margin-bottom:0.75rem;" prop:value=move || subject.get()
                    on:input=move |ev| subject.set(event_target_value(&ev))/>
                <label class="proj-section__hint">"Description"</label>
                <textarea class="landlord-search-input" style="width:100%;min-height:6rem;margin-bottom:0.75rem;"
                    prop:value=move || description.get()
                    on:input=move |ev| description.set(event_target_value(&ev))/>
                <Show when=move || mode.get() == CreateMode::Schedule>
                    <label class="proj-section__hint">"Scheduled date"</label>
                    <input
                        class="landlord-search-input"
                        style="width:100%;margin-bottom:0.75rem;"
                        type="date"
                        prop:value=move || scheduled_date.get()
                        on:input=move |ev| scheduled_date.set(event_target_value(&ev))
                    />
                </Show>
                <Show when=move || mode.get() == CreateMode::LogPaid>
                    <label class="proj-section__hint">"Amount paid (dollars)"</label>
                    <input class="landlord-search-input" style="width:100%;margin-bottom:0.75rem;" prop:value=move || cost.get()
                        on:input=move |ev| cost.set(event_target_value(&ev))/>
                </Show>
                {move || project_id.get().map(|_| view! {
                    <p class="proj-section__hint">"Linked to the open renovation project."</p>
                })}
                {move || error.get().map(|e| view! { <p style="color:#93000a;font-size:0.875rem;">{e}</p> })}
                <button
                    type="button"
                    class="folio-btn folio-btn--primary"
                    prop:disabled=move || saving.get()
                    on:click=move |_| {
                        error.set(None);
                        let Ok(aid) = Uuid::parse_str(&asset_id.get()) else {
                            error.set(Some("Select a unit or property.".into()));
                            return;
                        };
                        let subj = subject.get();
                        if subj.trim().is_empty() {
                            error.set(Some("Subject required".into()));
                            return;
                        }
                        let m = mode.get();
                        if m == CreateMode::Schedule && scheduled_date.get().trim().is_empty() {
                            error.set(Some("Choose a scheduled date.".into()));
                            return;
                        }
                        saving.set(true);
                        let desc = description.get();
                        let pid = project_id.get();
                        let sat = scheduled_date.get();
                        let nav = navigate.clone();
                        leptos::task::spawn_local(async move {
                            let result = match m {
                                CreateMode::LogPaid => {
                                    let cents = (cost.get().parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
                                    log_paid_maintenance(aid, subj, Some(desc), cents, pid).await
                                }
                                CreateMode::Schedule => {
                                    let when = format!("{}T09:00:00Z", sat.trim());
                                    schedule_maintenance_visit(aid, subj, Some(desc), when).await
                                }
                                CreateMode::WorkOrder => {
                                    create_maintenance_ticket(
                                        aid,
                                        Uuid::nil(),
                                        "general".into(),
                                        desc,
                                        false,
                                    )
                                    .await
                                }
                            };
                            match result {
                                Ok(_id) if m == CreateMode::LogPaid => {
                                    nav(FolioRoute::LandlordRatings.path(), Default::default());
                                }
                                Ok(_id) if m == CreateMode::Schedule => {
                                    nav(FolioRoute::LandlordMaintenance.path(), Default::default());
                                }
                                Ok(id) => {
                                    let path = FolioRoute::LandlordMaintenanceDetail
                                        .path()
                                        .replace(":id", &id.to_string());
                                    nav(&path, Default::default());
                                }
                                Err(e) => {
                                    saving.set(false);
                                    error.set(Some(e.to_string()));
                                }
                            }
                        });
                    }
                >
                    {move || if saving.get() { "Saving…" } else { "Submit" }}
                </button>
            </InterruptibleSheet>
        </div>
    }
}
