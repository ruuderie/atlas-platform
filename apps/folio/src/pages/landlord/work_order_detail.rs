//! Work order detail — `/l/maintenance/:id`

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaintenanceDetailDto {
    id: Uuid,
    asset_id: Option<Uuid>,
    subject: String,
    description: Option<String>,
    status: String,
    priority: String,
    estimated_cost_cents: Option<i64>,
    actual_cost_cents: Option<i64>,
    project_id: Option<Uuid>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Serialize)]
struct CompleteBody {
    actual_cost_cents: Option<i64>,
    note: Option<String>,
}

#[derive(Clone, Serialize)]
struct LogExpenseBody {
    asset_id: Uuid,
    subject: String,
    description: Option<String>,
    actual_cost_cents: i64,
    related_case_id: Option<Uuid>,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(GetMaintenanceDetail, "/api")]
async fn get_maintenance_detail(id: Uuid) -> Result<MaintenanceDetailDto, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get(
        &format!("/api/folio/maintenance/{id}"),
        &token,
        None,
    )
    .await
    .map_err(ServerFnError::new)
}

#[server(CompleteMaintenance, "/api")]
async fn complete_maintenance(
    id: Uuid,
    actual_cost_cents: Option<i64>,
) -> Result<(), ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    let body = CompleteBody {
        actual_cost_cents,
        note: None,
    };
    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        &format!("/api/folio/maintenance/{id}/complete"),
        &token,
        None,
        &body,
    )
    .await
    .map(|_| ())
    .map_err(ServerFnError::new)
}

/// Record expense against this work order (`related_case_id` = this case).
#[server(LogExpenseOnWorkOrder, "/api")]
async fn log_expense_on_work_order(
    case_id: Uuid,
    asset_id: Uuid,
    amount_dollars: String,
    note: String,
) -> Result<(), ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    let dollars: f64 = amount_dollars
        .trim()
        .parse()
        .map_err(|_| ServerFnError::new("Enter a valid amount"))?;
    if dollars < 0.0 {
        return Err(ServerFnError::new("Amount cannot be negative"));
    }
    let cents = (dollars * 100.0).round() as i64;
    let body = LogExpenseBody {
        asset_id,
        subject: note.trim().to_string(),
        description: None,
        actual_cost_cents: cents,
        related_case_id: Some(case_id),
    };
    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/folio/maintenance/log-paid",
        &token,
        None,
        &body,
    )
    .await
    .map(|_| ())
    .map_err(ServerFnError::new)
}

#[component]
pub fn WorkOrderDetail() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();
    let id = Memo::new(move |_| {
        params
            .get()
            .get("id")
            .and_then(|s| Uuid::parse_str(&s).ok())
    });

    let refetch_tick = RwSignal::new(0u32);

    let detail = Resource::new(
        move || (id.get(), refetch_tick.get()),
        |(maybe, _)| async move {
            match maybe {
                Some(mid) => get_maintenance_detail(mid).await,
                None => Err(ServerFnError::new("Missing work order id")),
            }
        },
    );

    let completing = RwSignal::new(false);
    let expense_amount = RwSignal::new(String::new());
    let expense_note = RwSignal::new(String::new());
    let expense_pending = RwSignal::new(false);
    let expense_msg = RwSignal::new(None::<String>);

    view! {
        <div class="landlord-list-page">
            <Suspense fallback=move || view! { <div class="folio-empty">"Loading work order…"</div> }>
                {move || {
                    let navigate = navigate.clone();
                    match detail.get() {
                    Some(Ok(d)) => {
                        let subject = d.subject.clone();
                        let status = d.status.clone();
                        let desc = d.description.clone().unwrap_or_default();
                        let project_id = d.project_id;
                        let wo_id = d.id;
                        let asset_id = d.asset_id;
                        let closed = d.completed_at.is_some() || d.status == "closed";
                        let est = d
                            .estimated_cost_cents
                            .map(|c| format!("${:.0}", c as f64 / 100.0))
                            .unwrap_or_else(|| "—".into());
                        let act = d
                            .actual_cost_cents
                            .map(|c| format!("${:.0}", c as f64 / 100.0))
                            .unwrap_or_else(|| "—".into());
                        let title_sig = Signal::derive({
                            let subject = subject.clone();
                            move || subject.clone()
                        });
                        view! {
                            <PageHeader
                                title=title_sig
                                subtitle=Signal::derive(|| "Work order".to_string())
                            >
                                <StatusPill label=status tone=StatusPillTone::Warn/>
                            </PageHeader>

                            {project_id.map(|pid| {
                                let href = FolioRoute::LandlordProjectDetail
                                    .path()
                                    .replace(":id", &pid.to_string());
                                view! {
                                    <p class="proj-section__hint" style="margin-bottom:1rem;">
                                        "Project · "
                                        <a class="hub-activity-rail__all" href=href>"Open project"</a>
                                    </p>
                                }
                            })}

                            <section class="proj-section" style="margin-bottom:1.25rem;">
                                <div class="proj-section__head">
                                    <h3 class="proj-section__title">"Details"</h3>
                                </div>
                                <div style="padding:1rem 1.25rem;">
                                    <p style="margin:0 0 0.75rem;white-space:pre-wrap;">{desc}</p>
                                    <p class="proj-section__hint">
                                        {format!("Est {est} · Actual {act}")}
                                    </p>
                                </div>
                            </section>

                            <section class="proj-section" style="margin-bottom:1.25rem;">
                                <div class="proj-section__head">
                                    <h3 class="proj-section__title">"Expense on this job"</h3>
                                </div>
                                <div style="padding:1rem 1.25rem;display:flex;flex-direction:column;gap:0.75rem;">
                                    <p class="proj-section__hint" style="margin:0;">
                                        "Records actual cost on this work order — no UUID hunting."
                                    </p>
                                    <label class="folio-field__label">
                                        "Amount (USD)"
                                        <input
                                            class="folio-input"
                                            type="text"
                                            inputmode="decimal"
                                            placeholder="420.00"
                                            prop:value=move || expense_amount.get()
                                            on:input=move |ev| expense_amount.set(event_target_value(&ev))
                                        />
                                    </label>
                                    <label class="folio-field__label">
                                        "Note"
                                        <input
                                            class="folio-input"
                                            type="text"
                                            placeholder="Invoice # or parts note"
                                            prop:value=move || expense_note.get()
                                            on:input=move |ev| expense_note.set(event_target_value(&ev))
                                        />
                                    </label>
                                    <button
                                        type="button"
                                        class="folio-btn folio-btn--ghost"
                                        prop:disabled=move || {
                                            expense_pending.get() || asset_id.is_none()
                                        }
                                        on:click=move |_| {
                                            let Some(aid) = asset_id else {
                                                expense_msg.set(Some(
                                                    "This work order has no unit — cannot attach expense.".into(),
                                                ));
                                                return;
                                            };
                                            expense_pending.set(true);
                                            expense_msg.set(None);
                                            let amt = expense_amount.get();
                                            let note = expense_note.get();
                                            leptos::task::spawn_local(async move {
                                                match log_expense_on_work_order(
                                                    wo_id, aid, amt, note,
                                                )
                                                .await
                                                {
                                                    Ok(()) => {
                                                        expense_amount.set(String::new());
                                                        expense_note.set(String::new());
                                                        expense_msg.set(Some("Expense saved.".into()));
                                                        refetch_tick.update(|n| *n += 1);
                                                    }
                                                    Err(e) => {
                                                        expense_msg.set(Some(e.to_string()));
                                                    }
                                                }
                                                expense_pending.set(false);
                                            });
                                        }
                                    >
                                        {move || {
                                            if expense_pending.get() {
                                                "Saving…"
                                            } else {
                                                "Save expense"
                                            }
                                        }}
                                    </button>
                                    <Show when=move || expense_msg.get().is_some()>
                                        <p class="proj-section__hint">{move || expense_msg.get().unwrap_or_default()}</p>
                                    </Show>
                                </div>
                            </section>

                            <section class="proj-section" style="margin-bottom:1.25rem;">
                                <div class="proj-section__head">
                                    <h3 class="proj-section__title">"Job photos"</h3>
                                </div>
                                <p class="proj-section__hint" style="margin:0;">
                                    "Photo upload on work orders is not available yet. Attach receipts under Cost & receipts or use the Digital vault."
                                </p>
                            </section>

                            <Show when=move || !closed>
                                <button
                                    type="button"
                                    class="folio-btn folio-btn--primary"
                                    prop:disabled=move || completing.get()
                                    on:click={
                                        let navigate = navigate.clone();
                                        move |_| {
                                            completing.set(true);
                                            let nav = navigate.clone();
                                            leptos::task::spawn_local(async move {
                                                match complete_maintenance(wo_id, None).await {
                                                    Ok(()) => {
                                                        nav(
                                                            FolioRoute::LandlordRatings.path(),
                                                            Default::default(),
                                                        );
                                                    }
                                                    Err(e) => {
                                                        completing.set(false);
                                                        leptos::logging::error!("complete failed: {e}");
                                                    }
                                                }
                                            });
                                        }
                                    }
                                >
                                    {move || if completing.get() { "Completing…" } else { "Mark complete & rate" }}
                                </button>
                            </Show>
                        }.into_any()
                    }
                    Some(Err(e)) => view! {
                        <div class="folio-empty">
                            <p>{e.to_string()}</p>
                            <button
                                type="button"
                                class="folio-btn folio-btn--ghost"
                                on:click=move |_| refetch_tick.update(|n| *n += 1)
                            >
                                "Retry"
                            </button>
                        </div>
                    }.into_any(),
                    None => view! { <div class="folio-empty">"Loading…"</div> }.into_any(),
                }}}
            </Suspense>
        </div>
    }
}
