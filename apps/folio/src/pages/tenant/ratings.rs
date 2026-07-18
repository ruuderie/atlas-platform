//! G-27 pending ratings — `/t/ratings`
//!
//! After STR check-out (`post_checkout`), sessions appear here for the guest
//! to complete via `ScorecardWidget`.
//!
//! APIs:
//!   GET  /api/scorecard-sessions/pending
//!   GET  /api/scorecard-templates/{id}/dimensions
//!   POST /api/scorecard-sessions/{sid}/entries

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use shared_ui::components::scorecard::models::{RenderMode, ScaleType, SessionDimension};
use shared_ui::components::scorecard::{ScoreSubmission, ScorecardWidget};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingSession {
    pub session_id: Uuid,
    pub scorecard_id: Uuid,
    pub template_id: Uuid,
    pub subject_entity_type: String,
    pub subject_entity_id: Uuid,
    pub session_type: String,
    pub context_entity_type: Option<String>,
    pub context_entity_id: Option<Uuid>,
    pub session_label: Option<String>,
    pub status: String,
    pub occurred_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionDto {
    pub id: Uuid,
    pub template_id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub scale_type: String,
    pub scale_min: String,
    pub scale_max: String,
    pub weight: String,
    pub sort_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitEntryBody {
    pub scorecard_id: Uuid,
    pub dimension_id: Uuid,
    pub score: Option<f64>,
    pub option_id: Option<Uuid>,
    pub source_type: Option<String>,
    pub note: Option<String>,
}

#[cfg(feature = "ssr")]
fn session_token(
    headers: &axum::http::HeaderMap,
) -> Result<String, server_fn::error::ServerFnError> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            })
        })
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

#[server(FetchPendingRatings, "/api")]
pub async fn fetch_pending_ratings() -> Result<Vec<PendingSession>, server_fn::error::ServerFnError>
{
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<PendingSession>>(
        "/api/scorecard-sessions/pending",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(FetchRatingDimensions, "/api")]
pub async fn fetch_rating_dimensions(
    template_id: String,
) -> Result<Vec<DimensionDto>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/scorecard-templates/{template_id}/dimensions");
    crate::atlas_client::authenticated_get::<Vec<DimensionDto>>(&url, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(SubmitRatingEntry, "/api")]
pub async fn submit_rating_entry(
    session_id: String,
    body: SubmitEntryBody,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = format!("/api/scorecard-sessions/{session_id}/entries");
    crate::atlas_client::authenticated_post::<SubmitEntryBody, serde_json::Value>(
        &url, &token, None, &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(())
}

fn dims_to_session(dims: Vec<DimensionDto>) -> Vec<SessionDimension> {
    dims.into_iter()
        .map(|d| {
            let scale = ScaleType::from_str(&d.scale_type).unwrap_or(ScaleType::Rating);
            let min: f64 = d.scale_min.parse().unwrap_or(1.0);
            let max: f64 = d.scale_max.parse().unwrap_or(10.0);
            SessionDimension {
                dimension_id: d.id,
                slug: d.slug,
                name: d.name,
                description: d.description.unwrap_or_default(),
                scale_type: scale,
                scale_min: min,
                scale_max: max,
                unit_label: None,
                is_inverted: false,
                is_required: false,
                render_mode: RenderMode::Normal,
                draft_score: None,
                inferred_score: None,
                inferred_confidence: None,
                draft_option_id: None,
            }
        })
        .collect()
}

/// Shared pending-ratings UI for tenant (post_checkout) and landlord (work-order complete).
#[component]
pub fn PendingRatingsPage(
    title: &'static str,
    subtitle: &'static str,
    empty_message: &'static str,
    default_session_label: &'static str,
) -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let active: RwSignal<Option<(PendingSession, Vec<SessionDimension>)>> = RwSignal::new(None);
    let busy = RwSignal::new(false);

    let pending = LocalResource::new(move || {
        let _ = refresh.get();
        async move { fetch_pending_ratings().await }
    });

    view! {
        <div class="w-full">
            <div class="page-header">
                <h1 class="page-title">{title}</h1>
                <p class="page-subtitle">{subtitle}</p>
            </div>

            <Show when=move || error.get().is_some()>
                <p class="text-red-400 text-sm mb-4">{error.get().unwrap_or_default()}</p>
            </Show>

            <Show
                when=move || active.get().is_some()
                fallback=move || {
                    view! {
                        <Suspense fallback=move || view! { <p class="page-subtitle">"Loading…"</p> }>
                            {move || {
                                match pending.get() {
                                    Some(Ok(list)) if list.is_empty() => {
                                        view! {
                                            <div class="folio-empty">
                                                <p class="folio-empty__sub">{empty_message}</p>
                                            </div>
                                        }.into_any()
                                    }
                                    Some(Ok(list)) => {
                                        view! {
                                            <ul class="space-y-3">
                                                {list.into_iter().map(|s| {
                                                    let label = s.session_label.clone()
                                                        .unwrap_or_else(|| format!("{} rating", s.session_type));
                                                    let s2 = s.clone();
                                                    view! {
                                                        <li class="flex items-center justify-between gap-4 py-3 border-b border-[var(--folio-border)]">
                                                            <div>
                                                                <div class="font-medium">{label}</div>
                                                                <div class="text-sm text-[var(--folio-muted)]">
                                                                    {s.subject_entity_type.clone()}
                                                                </div>
                                                            </div>
                                                            <button
                                                                class="cfg-btn"
                                                                type="button"
                                                                on:click=move |_| {
                                                                    let session = s2.clone();
                                                                    busy.set(true);
                                                                    error.set(None);
                                                                    spawn_local(async move {
                                                                        let tid = session.template_id.to_string();
                                                                        match fetch_rating_dimensions(tid).await {
                                                                            Ok(dims) => {
                                                                                active.set(Some((session, dims_to_session(dims))));
                                                                            }
                                                                            Err(e) => error.set(Some(e.to_string())),
                                                                        }
                                                                        busy.set(false);
                                                                    });
                                                                }
                                                            >
                                                                "Rate"
                                                            </button>
                                                        </li>
                                                    }
                                                }).collect_view()}
                                            </ul>
                                        }.into_any()
                                    }
                                    Some(Err(e)) => {
                                        view! {
                                            <p class="text-red-400 text-sm">{e.to_string()}</p>
                                        }.into_any()
                                    }
                                    None => view! { <p class="page-subtitle">"Loading…"</p> }.into_any(),
                                }
                            }}
                        </Suspense>
                    }.into_any()
                }
            >
                {move || {
                    let Some((session, dims)) = active.get() else {
                        return view! { <div/> }.into_any();
                    };
                    let session_id = session.session_id;
                    let scorecard_id = session.scorecard_id;
                    let label = session.session_label.clone()
                        .unwrap_or_else(|| default_session_label.to_string());
                    view! {
                        <ScorecardWidget
                            scorecard_id=scorecard_id
                            session_id=session_id
                            subject_label=label
                            dimensions=dims
                            on_submit=Callback::new(move |subs: Vec<ScoreSubmission>| {
                                busy.set(true);
                                error.set(None);
                                spawn_local(async move {
                                    for sub in subs {
                                        let body = SubmitEntryBody {
                                            scorecard_id,
                                            dimension_id: sub.dimension_id,
                                            score: sub.score,
                                            option_id: sub.option_id,
                                            source_type: Some("manual".into()),
                                            note: None,
                                        };
                                        if let Err(e) = submit_rating_entry(
                                            session_id.to_string(),
                                            body,
                                        ).await {
                                            error.set(Some(e.to_string()));
                                            busy.set(false);
                                            return;
                                        }
                                    }
                                    active.set(None);
                                    refresh.update(|n| *n = n.wrapping_add(1));
                                    busy.set(false);
                                });
                            })
                            on_cancel=Callback::new(move |_| {
                                active.set(None);
                                error.set(None);
                            })
                        />
                    }.into_any()
                }}
            </Show>
        </div>
    }
}

#[component]
pub fn TenantRatings() -> impl IntoView {
    view! {
        <PendingRatingsPage
            title="Rate your stay"
            subtitle="Pending ratings opened after check-out."
            empty_message="No pending ratings. Complete a stay check-out to get a nudge here."
            default_session_label="Stay rating"
        />
    }
}
