//! Floating G-27 rating nudge — polls pending sessions and renders `NudgePrompt`.
//!
//! Fallback UX when WS delivery is unavailable: on landlord/tenant shell mount,
//! fetch `GET /api/scorecard-sessions/pending` and surface the first incomplete
//! session. Full rating remains on `/l/ratings` and `/t/ratings`.

use leptos::prelude::*;
use leptos::task::spawn_local;
use shared_ui::components::scorecard::models::{RenderMode, ScaleType, SessionDimension};
use shared_ui::components::scorecard::NudgePrompt;
use std::str::FromStr;
use uuid::Uuid;

use crate::pages::tenant::ratings::{
    fetch_pending_ratings, fetch_rating_dimensions, submit_rating_entry, DimensionDto,
    PendingSession, SubmitEntryBody,
};

fn dims_to_session(dims: Vec<DimensionDto>) -> Vec<SessionDimension> {
    dims.into_iter()
        .take(4)
        .map(|d| {
            let scale = ScaleType::from_str(&d.scale_type).unwrap_or(ScaleType::Rating);
            SessionDimension {
                dimension_id: d.id,
                slug: d.slug,
                name: d.name,
                description: d.description.unwrap_or_default(),
                scale_type: scale,
                scale_min: d.scale_min.parse().unwrap_or(1.0),
                scale_max: d.scale_max.parse().unwrap_or(10.0),
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

#[component]
pub fn ScorecardNudgeHost() -> impl IntoView {
    let visible: RwSignal<Option<(PendingSession, Vec<SessionDimension>)>> = RwSignal::new(None);
    let dismissed = RwSignal::new(false);

    Effect::new(move |_| {
        if dismissed.get() {
            return;
        }
        spawn_local(async move {
            let Ok(list) = fetch_pending_ratings().await else {
                return;
            };
            let Some(session) = list.into_iter().next() else {
                return;
            };
            let Ok(dims) = fetch_rating_dimensions(session.template_id.to_string()).await else {
                return;
            };
            let session_dims = dims_to_session(dims);
            if session_dims.is_empty() {
                return;
            }
            visible.set(Some((session, session_dims)));
        });
    });

    view! {
        <Show when=move || visible.get().is_some() && !dismissed.get()>
            {move || {
                let Some((session, dims)) = visible.get() else {
                    return view! { <div/> }.into_any();
                };
                let session_id = session.session_id;
                let scorecard_id = session.scorecard_id;
                let label = session
                    .session_label
                    .clone()
                    .unwrap_or_else(|| session.session_type.clone());
                let activity = session.session_type.clone();
                view! {
                    <NudgePrompt
                        subject_label=label
                        mode="nudge".into()
                        activity_type=activity
                        dimensions=dims
                        on_submit=Callback::new(move |scores: Vec<(Uuid, Option<f64>)>| {
                            spawn_local(async move {
                                for (dimension_id, score) in scores {
                                    let body = SubmitEntryBody {
                                        scorecard_id,
                                        dimension_id,
                                        score,
                                        option_id: None,
                                        source_type: Some("manual".into()),
                                        note: None,
                                    };
                                    let _ = submit_rating_entry(session_id.to_string(), body).await;
                                }
                                dismissed.set(true);
                                visible.set(None);
                            });
                        })
                        on_dismiss=Callback::new(move |_| {
                            dismissed.set(true);
                            visible.set(None);
                        })
                    />
                }.into_any()
            }}
        </Show>
    }
}
