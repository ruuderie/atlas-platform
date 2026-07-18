//! Project detail — `/l/projects/:id`
//! Budget, timeline, child WOs, G-27 rollup. Stitch: l_project_detail.

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::photo_lightbox::{PhotoItem, PhotoStrip};
use crate::components::status_pill::{StatusPill, StatusPillTone};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectDetailDto {
    id: Uuid,
    asset_id: Option<Uuid>,
    title: String,
    status: String,
    estimated_cost_cents: Option<i64>,
    committed_cents: i64,
    actual_spent_cents: i64,
    milestones: Option<serde_json::Value>,
    children: Vec<ProjectChildDto>,
    timeline: Vec<TimelineEventDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectChildDto {
    id: Uuid,
    subject: String,
    status: String,
    estimated_cost_cents: Option<i64>,
    actual_cost_cents: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineEventDto {
    at: chrono::DateTime<chrono::Utc>,
    kind: String,
    title: String,
    subtitle: Option<String>,
    ref_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct G27RollupDto {
    coverage: String,
    composite: Option<f64>,
    scored_count: u32,
    completed_wo_count: u32,
    pending_session_ids: Vec<Uuid>,
    dimension_means: Vec<DimMeanDto>,
    vendors: Vec<VendorRollupDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DimMeanDto {
    label: String,
    mean: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VendorRollupDto {
    service_provider_id: Uuid,
    job_count: u32,
    local_avg: Option<f64>,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(GetProjectDetail, "/api")]
async fn get_project_detail(id: Uuid) -> Result<ProjectDetailDto, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get(&format!("/api/folio/projects/{id}"), &token, None)
        .await
        .map_err(ServerFnError::new)
}

#[server(GetProjectG27Rollup, "/api")]
async fn get_project_g27_rollup(id: Uuid) -> Result<G27RollupDto, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get(
        &format!("/api/folio/projects/{id}/g27-rollup"),
        &token,
        None,
    )
    .await
    .map_err(ServerFnError::new)
}

fn money(cents: i64) -> String {
    format!("${:.0}", cents as f64 / 100.0)
}

fn timeline_dot_class(kind: &str) -> &'static str {
    match kind {
        "g27_scored" | "g27_pending" => "proj-timeline__dot proj-timeline__dot--g27",
        "expense" => "proj-timeline__dot proj-timeline__dot--exp",
        "milestone" => "proj-timeline__dot proj-timeline__dot--done",
        "work_order" => "proj-timeline__dot proj-timeline__dot--active",
        _ => "proj-timeline__dot",
    }
}

#[component]
pub fn ProjectDetail() -> impl IntoView {
    let params = use_params_map();
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
                Some(id) => get_project_detail(id).await,
                None => Err(ServerFnError::new("Missing project id")),
            }
        },
    );

    let rollup = Resource::new(
        move || (id.get(), refetch_tick.get()),
        |(maybe, _)| async move {
            match maybe {
                Some(id) => get_project_g27_rollup(id).await.ok(),
                None => None,
            }
        },
    );

    view! {
        <div class="proj-page landlord-list-page">
            <Suspense fallback=move || view! { <div class="folio-empty">"Loading project…"</div> }>
                {move || match detail.get() {
                    Some(Ok(p)) => view! { <ProjectDetailBody project=p rollup=rollup/> }.into_any(),
                    Some(Err(e)) => view! {
                        <div class="folio-empty">
                            <p>{format!("Could not load project: {e}")}</p>
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
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn ProjectDetailBody(
    project: ProjectDetailDto,
    rollup: Resource<Option<G27RollupDto>>,
) -> impl IntoView {
    let title = project.title.clone();
    let status = project.status.clone();
    let budget = project.estimated_cost_cents.unwrap_or(0);
    let committed = project.committed_cents;
    let actual = project.actual_spent_cents;
    let remaining = budget - actual;
    let pct = if budget > 0 {
        ((actual * 100) / budget).clamp(0, 100) as u32
    } else {
        0
    };
    let add_wo = FolioRoute::LandlordMaintenanceNew.path().to_string();
    let ratings = FolioRoute::LandlordRatings.path().to_string();
    let children = project.children.clone();
    let timeline = project.timeline.clone();

    view! {
        <PageHeader
            title=Signal::derive(move || title.clone())
            subtitle=Signal::derive(|| "Renovation project".to_string())
        >
            <StatusPill label=status.clone() tone=StatusPillTone::Warn/>
            <a class="folio-btn folio-btn--primary" href=add_wo>"Add work order"</a>
        </PageHeader>

        <div class="proj-kpi-grid">
            <div class="folio-stat-card">
                <p class="folio-stat-card__label">"Budget"</p>
                <p class="folio-stat-card__value">{money(budget)}</p>
            </div>
            <div class="folio-stat-card">
                <p class="folio-stat-card__label">"Committed"</p>
                <p class="folio-stat-card__value">{money(committed)}</p>
            </div>
            <div class="folio-stat-card">
                <p class="folio-stat-card__label">"Actual spent"</p>
                <p class="folio-stat-card__value">{money(actual)}</p>
            </div>
            <div class="folio-stat-card">
                <p class="folio-stat-card__label">"Remaining"</p>
                <p class="folio-stat-card__value">{money(remaining)}</p>
                <div class="proj-budget-bar"><i style=format!("width:{pct}%")></i></div>
            </div>
        </div>

        <div class="proj-layout">
            <div>
                <section class="proj-section" style="margin-bottom:1.25rem;">
                    <div class="proj-section__head">
                        <div>
                            <h3 class="proj-section__title">"Timeline"</h3>
                            <p class="proj-section__hint">"Composed read model · ProjectTimelineKind"</p>
                        </div>
                        <span class="proj-section__hint">{format!("{} events", timeline.len())}</span>
                    </div>
                    <div class="proj-timeline">
                        <For
                            each=move || timeline.clone()
                            key=|e| format!("{}-{}", e.at, e.title)
                            children=move |e| {
                                let href = e.ref_id.map(|rid| {
                                    FolioRoute::LandlordMaintenanceDetail.path().replace(":id", &rid.to_string())
                                }).unwrap_or_else(|| "#".into());
                                let date = e.at.format("%b %e").to_string();
                                view! {
                                    <a class="proj-timeline__item press" href=href>
                                        <span class=timeline_dot_class(&e.kind)></span>
                                        <div class="hub-activity-rail__body">
                                            <p class="hub-activity-rail__row-title">{e.title.clone()}</p>
                                            <p class="hub-activity-rail__row-meta">
                                                {e.subtitle.clone().unwrap_or_default()}
                                            </p>
                                        </div>
                                        <span class="proj-section__hint">{date}</span>
                                    </a>
                                }
                            }
                        />
                    </div>
                </section>

                <section class="proj-section">
                    <div class="proj-section__head">
                        <div>
                            <h3 class="proj-section__title">"Work orders"</h3>
                            <p class="proj-section__hint">"Each work order: cost, vendor, rating on complete"</p>
                        </div>
                        <span class="proj-section__hint">{format!("{} WOs", children.len())}</span>
                    </div>
                    <For
                        each=move || children.clone()
                        key=|c| c.id
                        children=move |c| {
                            let href = FolioRoute::LandlordMaintenanceDetail.path().replace(":id", &c.id.to_string());
                            let est = c.estimated_cost_cents.map(money).unwrap_or_else(|| "—".into());
                            let act = c.actual_cost_cents.map(money).unwrap_or_else(|| "—".into());
                            view! {
                                <a class="hub-activity-rail__row press" href=href>
                                    <StatusPill label="WO".to_string() tone=StatusPillTone::Info/>
                                    <div class="hub-activity-rail__body">
                                        <p class="hub-activity-rail__row-title">{c.subject.clone()}</p>
                                        <p class="hub-activity-rail__row-meta">{c.status.clone()}</p>
                                    </div>
                                    <div style="text-align:right;">
                                        <p class="proj-section__hint">{format!("Est {est}")}</p>
                                        <p class="hub-activity-rail__row-title">{act}</p>
                                    </div>
                                </a>
                            }
                        }
                    />
                </section>
            </div>

            <aside>
                <section class="proj-section" style="margin-bottom:1rem;">
                    <div class="proj-section__head">
                        <h3 class="proj-section__title">"Contractor ratings"</h3>
                    </div>
                    <div style="padding:1rem 1.25rem;">
                        <Suspense fallback=|| view! { <p class="proj-section__hint">"Loading rollup…"</p> }>
                            {move || match rollup.get() {
                                Some(Some(r)) => {
                                    let cov = r.coverage.clone();
                                    let dims = r.dimension_means.clone();
                                    let vendors = r.vendors.clone();
                                    let pending = r.pending_session_ids.len();
                                    view! {
                                        <div style="display:flex;justify-content:space-between;align-items:flex-end;margin-bottom:1rem;">
                                            <div>
                                                <p class="proj-section__hint">"Composite"</p>
                                                <p style="font-size:1.75rem;font-weight:700;margin:0;">
                                                    {r.composite.map(|v| format!("{v:.1}")).unwrap_or_else(|| "—".into())}
                                                </p>
                                            </div>
                                            <div style="text-align:right;">
                                                <StatusPill label=cov tone=StatusPillTone::Warn/>
                                                <p class="proj-section__hint" style="margin-top:0.35rem;">
                                                    {format!("{} / {} scored · {} pending", r.scored_count, r.completed_wo_count, pending)}
                                                </p>
                                            </div>
                                        </div>
                                        <div class="proj-g27-dims">
                                            <For
                                                each=move || dims.clone()
                                                key=|d| d.label.clone()
                                                children=move |d| {
                                                    let pct = d.mean.map(|m| ((m / 5.0) * 100.0) as u32).unwrap_or(0);
                                                    view! {
                                                        <div>
                                                            <div style="display:flex;justify-content:space-between;font-size:0.7rem;margin-bottom:0.25rem;">
                                                                <span class="proj-section__hint">{d.label.clone()}</span>
                                                                <strong>
                                                                    {d.mean.map(|m| format!("{m:.1}")).unwrap_or_else(|| "—".into())}
                                                                </strong>
                                                            </div>
                                                            <div class="proj-dim-bar"><i style=format!("width:{pct}%")></i></div>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>
                                        <p class="proj-section__hint" style="margin:1rem 0 0.5rem;">"Vendors on this project"</p>
                                        <For
                                            each=move || vendors.clone()
                                            key=|v| v.service_provider_id
                                            children=move |v| {
                                                view! {
                                                    <div style="display:flex;justify-content:space-between;font-size:0.875rem;margin-bottom:0.35rem;">
                                                        <span>{format!("{} jobs", v.job_count)}</span>
                                                        <strong>
                                                            {v.local_avg.map(|a| format!("{a:.1}")).unwrap_or_else(|| "—".into())}
                                                        </strong>
                                                    </div>
                                                }
                                            }
                                        />
                                        <PhotoStrip photos=Signal::derive(|| Vec::<PhotoItem>::new())/>
                                        <a class="hub-activity-rail__all" href=ratings.clone() style="display:inline-flex;margin-top:0.75rem;">
                                            "Open ratings"
                                        </a>
                                    }.into_any()
                                }
                                _ => view! {
                                    <p class="proj-section__hint">
                                        "No ratings yet. Complete child work orders to open rating sessions."
                                    </p>
                                }.into_any(),
                            }}
                        </Suspense>
                    </div>
                </section>
                <p class="proj-section__hint">
                    "Rule 7: read-side rollup only — durable reputation stays on the vendor scorecard."
                </p>
            </aside>
        </div>
    }
}
