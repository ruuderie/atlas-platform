//! Work order detail — `/l/maintenance/:id`

use crate::atlas_client::{authenticated_get, authenticated_post, session_token_from_request};
use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::photo_lightbox::{PhotoItem, PhotoStrip};
use crate::components::status_pill::{StatusPill, StatusPillTone};
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaintenanceDetailDto {
    id: Uuid,
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

#[server(GetMaintenanceDetail, "/api")]
async fn get_maintenance_detail(id: Uuid) -> Result<MaintenanceDetailDto, ServerFnError> {
    let token = session_token_from_request().await.map_err(ServerFnError::new)?;
    authenticated_get(&format!("/api/folio/maintenance/{id}"), &token, None)
        .await
        .map_err(ServerFnError::new)
}

#[server(CompleteMaintenance, "/api")]
async fn complete_maintenance(
    id: Uuid,
    actual_cost_cents: Option<i64>,
) -> Result<(), ServerFnError> {
    let token = session_token_from_request().await.map_err(ServerFnError::new)?;
    let body = CompleteBody {
        actual_cost_cents,
        note: None,
    };
    authenticated_post::<_, serde_json::Value>(
        &format!("/api/folio/maintenance/{id}/complete"),
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
                                    <h3 class="proj-section__title">"Job photos"</h3>
                                </div>
                                <div class="ratings-photos">
                                    <PhotoStrip photos=Signal::derive(|| Vec::<PhotoItem>::new())/>
                                </div>
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
                                    {move || if completing.get() { "Completing…" } else { "Mark complete → G-27" }}
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
