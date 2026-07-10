//! Compact scorecard embed for pilot entity detail pages (G-27).
//!
//! Resolves a published template for `entity_type`, get-or-creates a scorecard
//! for the subject, and shows composite score + link to detail.
//! Full `ScorecardWidget` rating requires session dimensions from a session API
//! that is not yet wired here — link out for rating for now.

use crate::api::admin::get_tenant_stats;
use crate::api::scorecards::{
    get_or_create_scorecard, get_scorecard, list_templates, GetOrCreateInput, ScorecardDetail,
    ScorecardTemplate,
};
use leptos::prelude::*;
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

    let data_res = LocalResource::new(move || {
        let et = entity_type.get_value();
        let eid = entity_id.get_value();
        let override_tid = tenant_override.get_value();
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
                        let sc_id = detail.scorecard.id;
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
                        let _ = tid; // reserved for future Rate / ScorecardWidget mount
                        view! {
                            <div style="padding:14px;display:flex;flex-direction:column;gap:10px;">
                                <div style="font-size:11px;color:var(--text-muted);">{tmpl_name}</div>
                                <div style="display:flex;align-items:baseline;gap:10px;flex-wrap:wrap;">
                                    <span style="font-size:28px;font-weight:700;letter-spacing:-0.5px;color:var(--text-primary);font-family:monospace;">
                                        {score}
                                    </span>
                                    <span style="font-size:11px;font-weight:600;padding:2px 7px;border-radius:3px;border:1px solid var(--border-default);color:var(--text-secondary);">
                                        {confidence}
                                    </span>
                                </div>
                                <div style="display:flex;gap:14px;font-size:11px;color:var(--text-muted);">
                                    <span>{sessions}" sessions"</span>
                                    <span>{entries}" entries"</span>
                                </div>
                                <a
                                    href=href
                                    style="font-size:12px;color:var(--text-link);text-decoration:none;font-weight:600;"
                                >
                                    "Open scorecard →"
                                </a>
                            </div>
                        }.into_any()
                    }
                }}
            </Suspense>
        </div>
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
