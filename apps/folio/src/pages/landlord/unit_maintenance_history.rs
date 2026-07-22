//! Unit maintenance history — `/l/assets/:id/history/maintenance`
//! Stitch: timeline list + log expense (WO picker) + receipt honesty.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::asset_api::{get_asset_for_dispatch, AssetDetailDto};
use crate::pages::landlord::maintenance_queue::{list_maintenance_tickets, MaintenanceSummary};
use crate::utils::asset_label::format_asset_place_label;

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

/// Street · unit label, falling back to parent building street when the unit has none.
async fn place_label_for_unit(unit: &AssetDetailDto) -> String {
    let mut street = unit.address_line_1.clone();
    let mut city = unit.city.clone();
    let mut state = unit.state_province.clone();
    if street.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
        if let Some(pid) = unit.parent_asset_id {
            if let Ok(parent) = get_asset_for_dispatch(pid).await {
                street = parent.address_line_1.or(street);
                city = parent.city.or(city);
                state = parent.state_province.or(state);
            }
        }
    }
    format_asset_place_label(
        &unit.name,
        street.as_deref(),
        city.as_deref(),
        state.as_deref(),
    )
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

    let place = Resource::new(
        move || asset_id.get(),
        |aid| async move {
            if aid.is_nil() {
                return None::<String>;
            }
            let unit = get_asset_for_dispatch(aid).await.ok()?;
            Some(place_label_for_unit(&unit).await)
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

    let on_submit = move |ev: web_sys::SubmitEvent| {
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
            if d.is_empty() {
                None
            } else {
                Some(d)
            }
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
    };

    view! {
        <div class="folio-form-page maint-hist">
            <PageHeader
                title=Signal::derive(|| "Maintenance history".to_string())
                subtitle=Signal::derive(move || {
                    place
                        .get()
                        .flatten()
                        .unwrap_or_else(|| {
                            "Closed work orders and logged spend for this unit.".into()
                        })
                })
            >
                <a class="folio-btn folio-btn--ghost press" href=move || history_href.get()>
                    "Back to History"
                </a>
            </PageHeader>

            <p class="maint-hist__place">
                "Logging expenses for "
                <strong>
                    {move || {
                        place
                            .get()
                            .flatten()
                            .unwrap_or_else(|| "this unit".into())
                    }}
                </strong>
                "."
            </p>

            <div class="maint-hist__grid">
                <div class="maint-hist__col">
                    <section class="proj-section">
                        <div class="proj-section__head">
                            <div>
                                <h3 class="proj-section__title">"Timeline"</h3>
                                <p class="proj-section__hint">"Work orders and expenses on this unit"</p>
                            </div>
                        </div>
                        <Suspense fallback=|| view! { <div class="folio-empty--compact">"Loading…"</div> }>
                            {move || {
                                let list = tickets.get().and_then(|r| r.ok()).unwrap_or_default();
                                if list.is_empty() {
                                    return view! {
                                        <div class="folio-empty--compact" style="padding:1rem 1.25rem;">
                                            "No work orders yet. Log a standalone expense on the right."
                                        </div>
                                    }.into_any();
                                }
                                view! {
                                    <div class="maint-hist__timeline">
                                        <For
                                            each=move || list.clone()
                                            key=|t| t.id
                                            children=move |t: MaintenanceSummary| {
                                                let href = FolioRoute::LandlordMaintenanceDetail
                                                    .path()
                                                    .replace(":id", &t.id.to_string());
                                                let pick = t.id.to_string();
                                                let created = t.created_at.format("%b %e, %Y").to_string();
                                                view! {
                                                    <div class="maint-hist__row">
                                                        <div class="maint-hist__thumb" title="Photo on work order when attached">
                                                            <span class="material-symbols-outlined">"photo"</span>
                                                        </div>
                                                        <div class="maint-hist__row-body">
                                                            <div class="maint-hist__row-top">
                                                                <a class="maint-hist__row-title" href=href.clone()>{t.subject.clone()}</a>
                                                                <StatusPill label=t.status.clone() tone=StatusPillTone::Info/>
                                                            </div>
                                                            <p class="maint-hist__row-meta">
                                                                {format!("{created} · {} · Linked WO", t.priority)}
                                                            </p>
                                                        </div>
                                                        <div class="maint-hist__row-aside">
                                                            <span class="maint-hist__amount">"—"</span>
                                                            <button
                                                                type="button"
                                                                class="folio-btn folio-btn--ghost folio-btn--sm press"
                                                                on:click=move |_| related_case.set(pick.clone())
                                                            >
                                                                "Link"
                                                            </button>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                }.into_any()
                            }}
                        </Suspense>
                    </section>
                </div>

                <div class="maint-hist__col">
                    <section class="proj-section">
                        <div class="proj-section__head">
                            <div>
                                <h3 class="proj-section__title">"Log expense"</h3>
                                <p class="proj-section__hint">
                                    "Link cost to a work order when it settles that job."
                                </p>
                            </div>
                        </div>
                        <form class="folio-form" on:submit=on_submit>
                            <fieldset class="maint-hist__wo-picker">
                                <legend class="folio-field__label">"Work order"</legend>
                                <label class="maint-hist__wo-opt">
                                    <input
                                        type="radio"
                                        name="wo"
                                        value=""
                                        prop:checked=move || related_case.get().is_empty()
                                        on:change=move |_| related_case.set(String::new())
                                    />
                                    <span>
                                        <span class="maint-hist__wo-opt-title">"No work order"</span>
                                        <span class="maint-hist__wo-opt-sub">"Standalone expense on this unit"</span>
                                    </span>
                                </label>
                                {move || {
                                    let list = tickets.get().and_then(|r| r.ok()).unwrap_or_default();
                                    list.into_iter().map(|t| {
                                        let id = t.id.to_string();
                                        let id_chk = id.clone();
                                        let subject = t.subject.clone();
                                        let meta = format!("{} · {}", t.status, t.priority);
                                        view! {
                                            <label class="maint-hist__wo-opt">
                                                <input
                                                    type="radio"
                                                    name="wo"
                                                    value=id.clone()
                                                    prop:checked=move || related_case.get() == id_chk
                                                    on:change=move |_| related_case.set(id.clone())
                                                />
                                                <span>
                                                    <span class="maint-hist__wo-opt-title">{subject}</span>
                                                    <span class="maint-hist__wo-opt-sub">{meta}</span>
                                                </span>
                                            </label>
                                        }
                                    }).collect_view()
                                }}
                            </fieldset>
                            <label class="folio-field__label">
                                "What"
                                <input
                                    class="folio-input"
                                    type="text"
                                    placeholder="e.g. Invoice · parts + labor"
                                    prop:value=move || subject.get()
                                    on:input=move |ev| subject.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="folio-field__label">
                                "Amount"
                                <input
                                    class="folio-input"
                                    type="text"
                                    inputmode="decimal"
                                    placeholder="175.00"
                                    prop:value=move || cost.get()
                                    on:input=move |ev| cost.set(event_target_value(&ev))
                                />
                            </label>
                            <label class="folio-field__label">
                                "Notes"
                                <textarea
                                    class="folio-input"
                                    prop:value=move || description.get()
                                    on:input=move |ev| description.set(event_target_value(&ev))
                                />
                            </label>
                            {move || error.get().map(|e| view! { <p style="color:#b91c1c;">{e}</p> })}
                            {move || success.get().map(|s| view! { <p style="color:#15803d;">{s}</p> })}
                            <button
                                type="submit"
                                class="folio-btn folio-btn--primary press"
                                disabled=move || pending.get()
                            >
                                {move || if pending.get() { "Saving…" } else { "Save expense" }}
                            </button>
                        </form>
                    </section>
                    <section class="proj-section">
                        <div class="proj-section__head">
                            <h3 class="proj-section__title">"Receipt"</h3>
                        </div>
                        <div class="maint-hist__receipt">
                            <span class="material-symbols-outlined">"receipt_long"</span>
                            <p class="maint-hist__receipt-title">"Attach receipt"</p>
                            <p class="maint-hist__receipt-sub">"PDF or image → vault — Not available yet"</p>
                        </div>
                    </section>
                </div>
            </div>
        </div>
    }
}
