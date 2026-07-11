//! Compact scorecard embed for pilot entity detail pages (G-27).
//!
//! Resolves a published template for `entity_type`, get-or-creates a scorecard
//! for the subject, shows composite score + in-page Rate via ScorecardWidget.

use crate::api::admin::get_tenant_stats;
use crate::api::scorecards::{
    GetOrCreateInput, OpenSessionInput, ScorecardDetail, ScorecardDimension, ScorecardTemplate,
    SubmitEntryInput, get_or_create_scorecard, get_scorecard, list_dimensions, list_templates,
    open_session, recompute, submit_entry,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared_ui::components::scorecard::models::{RenderMode, ScaleType};
use shared_ui::components::scorecard::{ScoreSubmission, ScorecardWidget, SessionDimension};
use std::str::FromStr;
use uuid::Uuid;

#[component]
pub fn ScorecardPanel(
    /// Subject entity type string (e.g. `atlas_account`, `tenant`, `app_instance`).
    entity_type: String,
    /// Subject entity UUID as string.
    entity_id: String,
    /// Optional pilot/customer tenant override. When empty, uses first tenant from stats.
    #[prop(optional)]
    tenant_id: Option<String>,
    /// Display label for the subject (header).
    #[prop(optional)]
    subject_label: Option<String>,
) -> impl IntoView {
    let entity_type = StoredValue::new(entity_type);
    let entity_id = StoredValue::new(entity_id);
    let tenant_override = StoredValue::new(tenant_id);
    let subject_label = subject_label.unwrap_or_else(|| "Subject".to_string());
    let subject_label_sv = StoredValue::new(subject_label.clone());

    let refresh = RwSignal::new(0u32);
    let busy = RwSignal::new(false);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    // Active rating: (session_id, scorecard_id, dimensions).
    let rating: RwSignal<Option<(Uuid, Uuid, Vec<SessionDimension>)>> = RwSignal::new(None);

    let data_res = LocalResource::new(move || {
        let et = entity_type.get_value();
        let eid = entity_id.get_value();
        let override_tid = tenant_override.get_value();
        let _ = refresh.get();
        async move {
            let tid = match override_tid.filter(|s| !s.is_empty()) {
                Some(t) => t,
                None => {
                    let stats = get_tenant_stats().await.unwrap_or_default();
                    stats
                        .first()
                        .map(|t| t.tenant_id.clone())
                        .ok_or_else(|| "No tenant available".to_string())?
                }
            };

            let subject_uuid = Uuid::parse_str(&eid).map_err(|e| e.to_string())?;

            let templates = list_templates(&tid).await?;
            let template = pick_template(&templates, &et)
                .cloned()
                .ok_or_else(|| format!("No published template for entity_type={et}"))?;

            let created = get_or_create_scorecard(
                &tid,
                &GetOrCreateInput {
                    template_id: template.id,
                    subject_entity_type: et.clone(),
                    subject_entity_id: subject_uuid,
                },
            )
            .await?;

            let detail = get_scorecard(&tid, &created.id.to_string()).await?;
            Ok::<(String, ScorecardTemplate, ScorecardDetail), String>((tid, template, detail))
        }
    });

    view! {
        <div class="w-full card" style="margin-bottom:14px;">
            <div class="card-hdr" style="padding:9px 14px;border-bottom:1px solid var(--border-default);display:flex;align-items:center;justify-content:space-between;gap:8px;">
                <span class="card-title" style="font-size:11.5px;font-weight:600;">"Scorecard"</span>
                <span style="font-size:10px;color:var(--text-muted);">{subject_label}</span>
            </div>
            <Suspense fallback=move || view! {
                <div style="padding:14px;font-size:12px;color:var(--text-muted);">"Loading scorecard…"</div>
            }>
                {move || match data_res.get() {
                    None => view! {
                        <div style="padding:14px;font-size:12px;color:var(--text-muted);">"Loading…"</div>
                    }.into_any(),
                    Some(Err(e)) => view! {
                        <div style="padding:14px;font-size:12px;color:var(--text-muted);">
                            <div style="margin-bottom:6px;">{e}</div>
                            <a href="/billing/scorecards" style="color:var(--text-link);text-decoration:none;font-size:11px;">
                                "Manage templates →"
                            </a>
                        </div>
                    }.into_any(),
                    Some(Ok((tid, template, detail))) => {
                        let tid_sv = StoredValue::new(tid);
                        let sc_id = detail.scorecard.id;
                        let template_id = template.id;
                        let score = detail
                            .scorecard
                            .composite_score
                            .clone()
                            .unwrap_or_else(|| "—".into());
                        let confidence = detail.scorecard.confidence_level.clone();
                        let sessions = detail.scorecard.total_sessions;
                        let entries = detail.scorecard.total_entries;
                        let href = format!("/billing/scorecards/{sc_id}");
                        let tmpl_name = template.name.clone();

                        view! {
                            <PanelBody
                                tid_sv=tid_sv
                                sc_id=sc_id
                                template_id=template_id
                                score=score
                                confidence=confidence
                                sessions=sessions
                                entries=entries
                                href=href
                                tmpl_name=tmpl_name
                                subject_label_sv=subject_label_sv
                                rating=rating
                                busy=busy
                                error=error
                                refresh=refresh
                            />
                        }.into_any()
                    }
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn PanelBody(
    tid_sv: StoredValue<String>,
    sc_id: Uuid,
    template_id: Uuid,
    score: String,
    confidence: String,
    sessions: i32,
    entries: i32,
    href: String,
    tmpl_name: String,
    subject_label_sv: StoredValue<String>,
    rating: RwSignal<Option<(Uuid, Uuid, Vec<SessionDimension>)>>,
    busy: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    refresh: RwSignal<u32>,
) -> impl IntoView {
    view! {
        <div style="padding:14px;display:flex;flex-direction:column;gap:10px;">
            <Show when=move || error.get().is_some()>
                <div style="font-size:11px;color:#f87171;">{move || error.get().unwrap_or_default()}</div>
            </Show>

            <Show when=move || rating.get().is_none()>
                <div style="display:flex;flex-direction:column;gap:10px;">
                    <div style="font-size:11px;color:var(--text-muted);">{tmpl_name.clone()}</div>
                    <div style="display:flex;align-items:baseline;gap:10px;flex-wrap:wrap;">
                        <span style="font-size:28px;font-weight:700;letter-spacing:-0.5px;color:var(--text-primary);font-family:monospace;">
                            {score.clone()}
                        </span>
                        <span style="font-size:11px;font-weight:600;padding:2px 7px;border-radius:3px;border:1px solid var(--border-default);color:var(--text-secondary);">
                            {confidence.clone()}
                        </span>
                    </div>
                    <div style="display:flex;gap:14px;font-size:11px;color:var(--text-muted);">
                        <span>{sessions}" sessions"</span>
                        <span>{entries}" entries"</span>
                    </div>
                    <div style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;">
                        <button
                            type="button"
                            disabled=move || busy.get()
                            style="font-size:12px;font-weight:600;padding:5px 12px;border-radius:4px;border:1px solid var(--border-default);background:var(--bg-elevated, transparent);color:var(--text-primary);cursor:pointer;"
                            on:click=move |_| {
                                let tid = tid_sv.get_value();
                                error.set(None);
                                busy.set(true);
                                spawn_local(async move {
                                    let result = start_rating(&tid, sc_id, template_id).await;
                                    busy.set(false);
                                    match result {
                                        Ok(state) => rating.set(Some(state)),
                                        Err(e) => error.set(Some(e)),
                                    }
                                });
                            }
                        >
                            {move || if busy.get() { "Opening…" } else { "Rate" }}
                        </button>
                        <a
                            href=href.clone()
                            style="font-size:12px;color:var(--text-link);text-decoration:none;font-weight:600;"
                        >
                            "Open scorecard →"
                        </a>
                    </div>
                </div>
            </Show>

            <Show when=move || rating.get().is_some()>
                {move || {
                    let Some((session_id, scorecard_id, dims)) = rating.get() else {
                        return view! { <div></div> }.into_any();
                    };
                    let label = subject_label_sv.get_value();
                    view! {
                        <ScorecardWidget
                            scorecard_id=scorecard_id
                            session_id=session_id
                            subject_label=label
                            dimensions=dims
                            on_submit=Callback::new(move |subs: Vec<ScoreSubmission>| {
                                let tid = tid_sv.get_value();
                                busy.set(true);
                                error.set(None);
                                spawn_local(async move {
                                    let result = submit_ratings(
                                        &tid,
                                        session_id,
                                        scorecard_id,
                                        subs,
                                    )
                                    .await;
                                    busy.set(false);
                                    match result {
                                        Ok(()) => {
                                            rating.set(None);
                                            refresh.update(|n| *n = n.wrapping_add(1));
                                        }
                                        Err(e) => error.set(Some(e)),
                                    }
                                });
                            })
                            on_cancel=Callback::new(move |_| {
                                rating.set(None);
                                error.set(None);
                            })
                        />
                    }.into_any()
                }}
            </Show>
        </div>
    }
}

async fn start_rating(
    tenant_id: &str,
    scorecard_id: Uuid,
    template_id: Uuid,
) -> Result<(Uuid, Uuid, Vec<SessionDimension>), String> {
    let session = open_session(
        tenant_id,
        &scorecard_id.to_string(),
        &OpenSessionInput {
            session_type: "meeting".into(),
            occurred_at: None,
            context_entity_type: None,
            context_entity_id: None,
            session_label: Some("Platform admin rating".into()),
        },
    )
    .await?;

    let dims = list_dimensions(tenant_id, &template_id.to_string()).await?;
    let session_dims = dims
        .into_iter()
        .filter(|d| d.is_active)
        .map(to_session_dimension)
        .collect::<Vec<_>>();

    if session_dims.is_empty() {
        return Err("No active dimensions on this template".into());
    }

    Ok((session.id, scorecard_id, session_dims))
}

async fn submit_ratings(
    tenant_id: &str,
    session_id: Uuid,
    scorecard_id: Uuid,
    submissions: Vec<ScoreSubmission>,
) -> Result<(), String> {
    for sub in submissions {
        let source_type = match sub.source_type.as_str() {
            "manual" | "direct_entry" => Some("manual".into()),
            other if !other.is_empty() => Some(other.to_string()),
            _ => Some("manual".into()),
        };
        submit_entry(
            tenant_id,
            &session_id.to_string(),
            &SubmitEntryInput {
                scorecard_id,
                dimension_id: sub.dimension_id,
                score: sub.score,
                option_id: sub.option_id,
                source_type,
                context: None,
                note: None,
            },
        )
        .await?;
    }
    recompute(tenant_id, &scorecard_id.to_string()).await?;
    Ok(())
}

pub(crate) fn to_session_dimension(d: ScorecardDimension) -> SessionDimension {
    SessionDimension {
        dimension_id: d.id,
        slug: d.slug,
        name: d.name,
        description: d.description.unwrap_or_default(),
        scale_type: ScaleType::from_str(&d.scale_type).unwrap_or(ScaleType::Rating),
        scale_min: d.scale_min.parse().unwrap_or(1.0),
        scale_max: d.scale_max.parse().unwrap_or(10.0),
        unit_label: d.unit_label,
        is_inverted: d.is_inverted,
        is_required: false,
        render_mode: RenderMode::Normal,
        draft_score: None,
        inferred_score: None,
        inferred_confidence: None,
        draft_option_id: None,
    }
}

fn pick_template<'a>(
    templates: &'a [ScorecardTemplate],
    entity_type: &str,
) -> Option<&'a ScorecardTemplate> {
    templates
        .iter()
        .find(|t| t.entity_type == entity_type && t.is_published)
        .or_else(|| templates.iter().find(|t| t.entity_type == entity_type))
}
