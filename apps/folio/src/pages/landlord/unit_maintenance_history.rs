//! Unit maintenance history — `/l/assets/:id/history/maintenance`
//! Log paid expense with optional related work-order link.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::maintenance_queue::{list_maintenance_tickets, MaintenanceSummary};

#[derive(Serialize)]
struct LogPaidBody {
    asset_id: Uuid,
    subject: String,
    description: Option<String>,
    actual_cost_cents: i64,
    service_provider_id: Option<Uuid>,
    project_id: Option<Uuid>,
    related_case_id: Option<Uuid>,
}

#[derive(Deserialize)]
struct IdResp {
    id: Uuid,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(ListMaintenanceForAsset, "/api")]
async fn list_maintenance_for_asset(
    asset_id: Uuid,
) -> Result<Vec<MaintenanceSummary>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get(
        &format!("/api/folio/maintenance?asset_id={asset_id}"),
        &token,
        None,
    )
    .await
    .map_err(ServerFnError::new)
}

#[server(LogPaidWithRelated, "/api")]
async fn log_paid_with_related(
    asset_id: Uuid,
    subject: String,
    description: Option<String>,
    actual_cost_cents: i64,
    related_case_id: Option<Uuid>,
) -> Result<Uuid, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let body = LogPaidBody {
        asset_id,
        subject,
        description,
        actual_cost_cents,
        service_provider_id: None,
        project_id: None,
        related_case_id,
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
pub fn UnitMaintenanceHistory() -> impl IntoView {
    let params = use_params_map();
    let asset_id = Memo::new(move |_| {
        params
            .get()
            .get("id")
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or(Uuid::nil())
    });

    let tickets = Resource::new(
        move || asset_id.get(),
        |aid| async move {
            if aid.is_nil() {
                return Ok(Vec::new());
            }
            match list_maintenance_for_asset(aid).await {
                Ok(v) => Ok(v),
                Err(_) => list_maintenance_tickets().await.map(|all| {
                    all.into_iter()
                        .filter(|t| t.asset_id == Some(aid))
                        .collect()
                }),
            }
        },
    );

    let subject = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let cost = RwSignal::new(String::new());
    let related_case = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let success = RwSignal::new(None::<String>);
    let pending = RwSignal::new(false);
    let refresh = RwSignal::new(0u32);

    let history_href = Memo::new(move |_| {
        FolioRoute::LandlordUnitHistory
            .path()
            .replace(":id", &asset_id.get().to_string())
    });

    Effect::new(move |_| {
        let _ = refresh.get();
        tickets.refetch();
    });

    view! {
        <div class="folio-form-page">
            <PageHeader
                title=Signal::derive(|| "Maintenance history".to_string())
                subtitle=Signal::derive(|| {
                    "Log paid work for this unit. Optionally attach cost to an existing work order."
                        .to_string()
                })
            >
                <a class="folio-btn folio-btn--ghost press" href=move || history_href.get()>
                    "Back to History"
                </a>
            </PageHeader>

            <section class="proj-section" style="margin-bottom:1.5rem;">
                <h3 class="proj-section__title">"Work orders on this unit"</h3>
                <Suspense fallback=|| view! { <div class="folio-empty--compact">"Loading…"</div> }>
                    {move || {
                        let list = tickets.get().and_then(|r| r.ok()).unwrap_or_default();
                        if list.is_empty() {
                            return view! {
                                <div class="folio-empty--compact">"No work orders for this unit yet."</div>
                            }.into_any();
                        }
                        view! {
                            <For
                                each=move || list.clone()
                                key=|t| t.id
                                children=move |t: MaintenanceSummary| {
                                    let href = FolioRoute::LandlordMaintenanceDetail
                                        .path()
                                        .replace(":id", &t.id.to_string());
                                    let pick = t.id.to_string();
                                    view! {
                                        <div class="hub-activity-rail__row">
                                            <StatusPill label=t.status.clone() tone=StatusPillTone::Warn/>
                                            <div class="hub-activity-rail__body">
                                                <a class="hub-activity-rail__row-title" href=href>{t.subject.clone()}</a>
                                                <p class="hub-activity-rail__row-meta">{t.priority.clone()}</p>
                                            </div>
                                            <button
                                                type="button"
                                                class="folio-btn folio-btn--ghost press"
                                                on:click=move |_| related_case.set(pick.clone())
                                            >
                                                "Use as related"
                                            </button>
                                        </div>
                                    }
                                }
                            />
                        }.into_any()
                    }}
                </Suspense>
            </section>

            <form
                class="folio-form"
                on:submit=move |ev| {
                    ev.prevent_default();
                    error.set(None);
                    success.set(None);
                    let aid = asset_id.get();
                    if aid.is_nil() {
                        error.set(Some("Missing unit id.".into()));
                        return;
                    }
                    let subj = subject.get().trim().to_string();
                    if subj.is_empty() {
                        error.set(Some("Subject is required.".into()));
                        return;
                    }
                    let cents = match cost.get().trim().parse::<f64>() {
                        Ok(v) if v >= 0.0 => (v * 100.0).round() as i64,
                        _ => {
                            error.set(Some("Enter cost (e.g. 250).".into()));
                            return;
                        }
                    };
                    let desc = {
                        let d = description.get().trim().to_string();
                        if d.is_empty() { None } else { Some(d) }
                    };
                    let related = {
                        let s = related_case.get().trim().to_string();
                        if s.is_empty() {
                            None
                        } else {
                            Uuid::parse_str(&s).ok()
                        }
                    };
                    pending.set(true);
                    spawn_local(async move {
                        match log_paid_with_related(aid, subj, desc, cents, related).await {
                            Ok(_) => {
                                success.set(Some("Logged paid maintenance.".into()));
                                subject.set(String::new());
                                cost.set(String::new());
                                related_case.set(String::new());
                                refresh.update(|n| *n += 1);
                                pending.set(false);
                            }
                            Err(e) => {
                                error.set(Some(e.to_string()));
                                pending.set(false);
                            }
                        }
                    });
                }
            >
                <label class="folio-form__label">
                    "Subject"
                    <input
                        class="form-input"
                        type="text"
                        prop:value=move || subject.get()
                        on:input=move |ev| subject.set(event_target_value(&ev))
                    />
                </label>
                <label class="folio-form__label">
                    "Cost"
                    <input
                        class="form-input"
                        type="text"
                        inputmode="decimal"
                        placeholder="250"
                        prop:value=move || cost.get()
                        on:input=move |ev| cost.set(event_target_value(&ev))
                    />
                </label>
                <label class="folio-form__label">
                    "Description"
                    <textarea
                        class="form-input"
                        prop:value=move || description.get()
                        on:input=move |ev| description.set(event_target_value(&ev))
                    />
                </label>
                <label class="folio-form__label">
                    "Related work order (optional)"
                    <select
                        class="form-input"
                        prop:value=move || related_case.get()
                        on:change=move |ev| related_case.set(event_target_value(&ev))
                    >
                        <option value="">"None — standalone expense"</option>
                        {move || {
                            let list = tickets.get().and_then(|r| r.ok()).unwrap_or_default();
                            list.into_iter().map(|t| {
                                let id = t.id.to_string();
                                let label = format!("{} · {}", t.subject, t.status);
                                view! { <option value=id>{label}</option> }
                            }).collect::<Vec<_>>()
                        }}
                    </select>
                </label>

                {move || error.get().map(|e| view! { <p style="color:#b91c1c;">{e}</p> })}
                {move || success.get().map(|s| view! { <p style="color:#15803d;">{s}</p> })}

                <button
                    type="submit"
                    class="folio-btn folio-btn--primary press"
                    disabled=move || pending.get()
                >
                    {move || if pending.get() { "Saving…" } else { "Log paid maintenance" }}
                </button>
            </form>
        </div>
    }
}
