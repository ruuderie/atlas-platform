//! G-27 Scorecard Widget — full session form embedded in entity records.
//!
//! This is the primary rating surface rendered on entity detail pages
//! (Lead, Opportunity, Account, etc.). It:
//!
//! 1. Receives the list of `SessionDimension`s (already filtered/sorted by
//!    the service via display rules) and renders the appropriate input.
//! 2. Groups dimensions by category with collapsible sections.
//! 3. Shows an alert banner for dimensions with `render_mode = "alert"`.
//! 4. Handles AI inference confirmation inline.
//! 5. Provides a transcript upload zone that triggers the AI inference flow.
//!
//! # Usage
//! ```
//! <ScorecardWidget
//!     scorecard_id=scorecard_id
//!     session_id=session_id
//!     subject_label="ACME Corp"
//!     dimensions=session_dimensions
//!     on_submit=Callback::new(move |scores| { /* POST to /api/scorecard/entries */ })
//!     transcript_upload_enabled=true
//! />
//! ```

use leptos::prelude::*;
use uuid::Uuid;
use std::collections::BTreeMap;
use super::super::models::SessionDimension;

// ── ScorecardWidget ───────────────────────────────────────────────────────────

#[component]
pub fn ScorecardWidget(
    /// ID of the scorecard being rated.
    scorecard_id: Uuid,
    /// ID of the active session.
    session_id: Uuid,
    /// Display name of the entity being rated (for the header).
    subject_label: String,
    /// Dimensions to render, already filtered/sorted by display rules.
    dimensions: Vec<SessionDimension>,
    /// Called on submit with (dimension_id, score, option_id) triples.
    on_submit: Callback<Vec<ScoreSubmission>>,
    /// Enable transcript/recording upload zone.
    #[prop(default = false)] transcript_upload_enabled: bool,
    /// Called when the user drops/uploads a transcript file.
    #[prop(optional)] on_transcript_upload: Option<Callback<web_sys::File>>,
    /// Called when the user rejects an AI-inferred score.
    #[prop(optional)] on_reject_inference: Option<Callback<Uuid>>,
    /// Called on cancel.
    #[prop(optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let dims = RwSignal::new(dimensions);
    let is_submitting = RwSignal::new(false);
    let show_upload_zone = RwSignal::new(false);
    let upload_hover = RwSignal::new(false);
    let submission_error: RwSignal<Option<String>> = RwSignal::new(None);

    // Group by category for collapsible sections
    let categories = move || {
        let mut map: BTreeMap<String, Vec<SessionDimension>> = BTreeMap::new();
        for dim in dims.get() {
            let cat = if dim.render_mode == "nudge" || dim.render_mode == "prep" {
                // Surface nudge/prep dims in their own group at top
                format!("__priority__{}", dim.render_mode)
            } else {
                dim.slug.chars().take(0).collect::<String>(); // placeholder — real category from model
                // In a real implementation this would be dim.category
                "General".to_string()
            };
            map.entry(cat).or_default().push(dim);
        }
        map.into_iter().collect::<Vec<_>>()
    };

    // Alert banner dimensions
    let alert_dims = move || {
        dims.get().into_iter().filter(|d| d.render_mode == "alert").collect::<Vec<_>>()
    };

    let required_unrated = move || {
        dims.get().into_iter().filter(|d| d.is_required && d.draft_score.is_none() && d.draft_option_id.is_none()).count()
    };

    let handle_submit = move |_| {
        if required_unrated() > 0 {
            submission_error.set(Some(format!(
                "{} required dimension(s) not yet rated.",
                required_unrated()
            )));
            return;
        }
        submission_error.set(None);
        is_submitting.set(true);

        let submissions: Vec<ScoreSubmission> = dims.get().into_iter()
            .filter(|d| d.draft_score.is_some() || d.draft_option_id.is_some())
            .map(|d| ScoreSubmission {
                dimension_id: d.dimension_id,
                score: d.draft_score,
                option_id: d.draft_option_id,
                // transcript_inferred entries come from a separate flow —
                // here we only submit user-entered scores
                source_type: "direct_entry".to_string(),
            })
            .collect();

        on_submit.run(submissions);
    };

    view! {
        <div class="scorecard-widget">
            // ── Header ─────────────────────────────────────────────────────
            <div class="sw-header">
                <div class="sw-header-main">
                    <h3 class="sw-title">"Scorecard"</h3>
                    <span class="sw-subject">{subject_label.clone()}</span>
                </div>
                <div class="sw-header-actions">
                    {if transcript_upload_enabled {
                        Some(view! {
                            <button
                                class=move || if show_upload_zone.get() {
                                    "cfg-btn cfg-btn--ghost cfg-btn--sm sw-upload-btn sw-upload-btn--active"
                                } else {
                                    "cfg-btn cfg-btn--ghost cfg-btn--sm sw-upload-btn"
                                }
                                title="Upload transcript or recording to auto-fill scores"
                                on:click=move |_| show_upload_zone.update(|v| *v = !*v)
                            >
                                "✨ Upload Transcript"
                            </button>
                        })
                    } else { None }}
                </div>
            </div>

            // ── Transcript upload zone ──────────────────────────────────────
            <Show when=move || show_upload_zone.get()>
                <div
                    class=move || if upload_hover.get() {
                        "sw-upload-zone sw-upload-zone--hover"
                    } else {
                        "sw-upload-zone"
                    }
                    on:dragover=move |ev| {
                        ev.prevent_default();
                        upload_hover.set(true);
                    }
                    on:dragleave=move |_| upload_hover.set(false)
                    on:drop=move |ev| {
                        ev.prevent_default();
                        upload_hover.set(false);
                        if let Some(files) = ev.data_transfer().and_then(|dt| dt.files()) {
                            if let Some(file) = files.get(0) {
                                if let Some(cb) = &on_transcript_upload {
                                    cb.run(file);
                                }
                            }
                        }
                    }
                >
                    <div class="sw-upload-icon">"📄"</div>
                    <p class="sw-upload-title">"Drop transcript or recording here"</p>
                    <p class="sw-upload-hint">
                        "Supported: .txt, .pdf, .docx, .mp3, .mp4, .m4a — AI will infer "
                        "dimension scores and highlight them for your review."
                    </p>
                    <label class="cfg-btn cfg-btn--ghost cfg-btn--sm">
                        "Browse files"
                        <input
                            type="file"
                            accept=".txt,.pdf,.docx,.mp3,.mp4,.m4a"
                            style="display:none"
                            on:change=move |ev| {
                                use wasm_bindgen::JsCast;
                                if let Some(input) = ev.target()
                                    .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                {
                                    if let Some(files) = input.files() {
                                        if let Some(file) = files.get(0) {
                                            if let Some(cb) = &on_transcript_upload {
                                                cb.run(file);
                                            }
                                        }
                                    }
                                }
                            }
                        />
                    </label>
                </div>
            </Show>

            // ── Alert banners ───────────────────────────────────────────────
            <Show when=move || !alert_dims().is_empty()>
                <div class="sw-alert-banners">
                    <For
                        each=alert_dims
                        key=|d| d.dimension_id
                        children=move |dim| {
                            view! {
                                <div class="sw-alert-banner">
                                    <span class="sw-alert-icon">"⚠"</span>
                                    <span>{dim.description.clone()}</span>
                                </div>
                            }
                        }
                    />
                </div>
            </Show>

            // ── Dimensions ──────────────────────────────────────────────────
            <div class="sw-dimensions">
                <For
                    each=move || dims.get()
                    key=|d| d.dimension_id
                    children=move |dim| {
                        let dim_id = dim.dimension_id;
                        let has_inference = dim.has_pending_inference();
                        let inferred = dim.inferred_score;
                        let confidence = dim.inferred_confidence.unwrap_or(0.0);
                        let is_required = dim.is_required;
                        let is_inverted = dim.is_inverted;
                        let scale_type = dim.scale_type.clone();
                        let reject_cb = on_reject_inference.clone();

                        let is_unrated = move || {
                            dims.get().iter()
                                .find(|d| d.dimension_id == dim_id)
                                .map(|d| d.draft_score.is_none() && d.draft_option_id.is_none())
                                .unwrap_or(true)
                        };

                        view! {
                            <div class=move || {
                                let mut cls = "sw-dim-row".to_string();
                                if is_required && is_unrated() { cls.push_str(" sw-dim-row--required-unrated"); }
                                if has_inference { cls.push_str(" sw-dim-row--inferred"); }
                                cls
                            }>
                                // Dimension header
                                <div class="sw-dim-header">
                                    <div class="sw-dim-name-row">
                                        <span class="sw-dim-name">{dim.name.clone()}</span>
                                        {if is_required {
                                            Some(view! { <span class="sw-dim-required">"*"</span> })
                                        } else { None }}
                                        {if is_inverted {
                                            Some(view! {
                                                <span class="sw-dim-inverted-badge" title="Lower score = better outcome">
                                                    "↓ lower is better"
                                                </span>
                                            })
                                        } else { None }}
                                        {if has_inference {
                                            Some(view! {
                                                <span class="sw-ai-badge">
                                                    "✨ AI " {format!("{:.0}%", confidence * 100.0)} " confident"
                                                </span>
                                            })
                                        } else { None }}
                                    </div>
                                    {if !dim.description.is_empty() {
                                        Some(view! {
                                            <p class="sw-dim-desc">{dim.description.clone()}</p>
                                        })
                                    } else { None }}
                                </div>

                                // Scale input
                                <WidgetDimensionInput
                                    dim_id=dim_id
                                    scale_type=scale_type
                                    scale_min=dim.scale_min
                                    scale_max=dim.scale_max
                                    unit_label=dim.unit_label.clone().unwrap_or_default()
                                    is_inverted=is_inverted
                                    initial_value=inferred.or(dim.draft_score)
                                    on_change=Callback::new(move |score: Option<f64>| {
                                        dims.update(|ds| {
                                            if let Some(d) = ds.iter_mut().find(|d| d.dimension_id == dim_id) {
                                                d.draft_score = score;
                                            }
                                        });
                                    })
                                />

                                // Inference confirm/reject (transcript_inferred)
                                {if has_inference {
                                    Some(view! {
                                        <div class="sw-inference-actions">
                                            <span class="sw-inference-label">
                                                "AI suggestion — confirm or reject"
                                            </span>
                                            {reject_cb.map(|cb| view! {
                                                <button
                                                    class="cfg-btn cfg-btn--ghost cfg-btn--xs"
                                                    on:click=move |_| cb.run(dim_id)
                                                >
                                                    "✕ Reject"
                                                </button>
                                            })}
                                        </div>
                                    })
                                } else { None }}
                            </div>
                        }
                    }
                />
            </div>

            // ── Validation error ────────────────────────────────────────────
            <Show when=move || submission_error.get().is_some()>
                <div class="sw-error-banner">
                    <span class="sw-error-icon">"⚠"</span>
                    {move || submission_error.get().unwrap_or_default()}
                </div>
            </Show>

            // ── Footer ─────────────────────────────────────────────────────
            <div class="sw-footer">
                <div class="sw-footer-left">
                    {move || {
                        let unrated = required_unrated();
                        if unrated > 0 {
                            view! {
                                <span class="sw-required-count">
                                    {unrated} " required dimension(s) unrated"
                                </span>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}
                </div>
                <div class="sw-footer-actions">
                    {on_cancel.map(|cb| view! {
                        <button
                            class="cfg-btn cfg-btn--ghost"
                            on:click=move |_| cb.run(())
                        >
                            "Cancel"
                        </button>
                    })}
                    <button
                        class="cfg-btn cfg-btn--primary"
                        disabled=move || is_submitting.get() || dims.get().is_empty()
                        on:click=handle_submit
                    >
                        {move || if is_submitting.get() { "Saving…" } else { "Submit Ratings" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

// ── ScoreSubmission ───────────────────────────────────────────────────────────

/// Output type from `ScorecardWidget::on_submit`.
#[derive(Clone, Debug)]
pub struct ScoreSubmission {
    pub dimension_id: Uuid,
    pub score: Option<f64>,
    pub option_id: Option<Uuid>,
    /// 'direct_entry' | 'transcript_inferred' | 'official_data'
    pub source_type: String,
}

// ── WidgetDimensionInput ──────────────────────────────────────────────────────

/// Full-size dimension input for the session form.
/// Renders a slider for rating, toggle for boolean, number for absolute.
#[component]
fn WidgetDimensionInput(
    dim_id: Uuid,
    scale_type: String,
    scale_min: f64,
    scale_max: f64,
    #[prop(optional)] unit_label: String,
    is_inverted: bool,
    initial_value: Option<f64>,
    on_change: Callback<Option<f64>>,
) -> impl IntoView {
    let value = RwSignal::new(initial_value);

    let low_label = if is_inverted { "Best" } else { "Low" };
    let high_label = if is_inverted { "Worst" } else { "High" };

    let unit = unit_label;
    let range = scale_max - scale_min;

    match scale_type.as_str() {
        "boolean" => view! {
            <div class="sw-input sw-input--boolean">
                <button
                    class=move || if value.get() == Some(1.0) {
                        "sw-bool-btn sw-bool-btn--yes sw-bool-btn--active"
                    } else {
                        "sw-bool-btn sw-bool-btn--yes"
                    }
                    on:click=move |_| { value.set(Some(1.0)); on_change.run(Some(1.0)); }
                >
                    "✓  Yes"
                </button>
                <button
                    class=move || if value.get() == Some(0.0) {
                        "sw-bool-btn sw-bool-btn--no sw-bool-btn--active"
                    } else {
                        "sw-bool-btn sw-bool-btn--no"
                    }
                    on:click=move |_| { value.set(Some(0.0)); on_change.run(Some(0.0)); }
                >
                    "✗  No"
                </button>
            </div>
        }.into_any(),

        "rating" | "absolute" => view! {
            <div class="sw-input sw-input--slider">
                <input
                    type="range"
                    class="sw-slider"
                    min=scale_min.to_string()
                    max=scale_max.to_string()
                    step=if range <= 10.0 { "1" } else { "0.1" }
                    prop:value=move || value.get().unwrap_or(scale_min).to_string()
                    on:input=move |ev| {
                        let v = event_target_value(&ev).parse::<f64>().ok();
                        value.set(v);
                        on_change.run(v);
                    }
                />
                <div class="sw-slider-labels">
                    <span class="sw-slider-label-low">
                        {low_label}
                        {if !unit.is_empty() { format!(" ({scale_min:.0} {unit})") } else { format!(" ({scale_min:.0})") }}
                    </span>
                    {
                        let unit_current = unit.clone();
                        move || value.get().map(|v| {
                            let unit_current = unit_current.clone();
                            view! {
                                <span class="sw-slider-current">
                                    {format!("{v:.1}")}
                                    {if !unit_current.is_empty() { format!(" {unit_current}") } else { String::new() }}
                                </span>
                            }
                        })
                    }
                    <span class="sw-slider-label-high">
                        {high_label}
                        {if !unit.is_empty() { format!(" ({scale_max:.0} {unit})") } else { format!(" ({scale_max:.0})") }}
                    </span>
                </div>
            </div>
        }.into_any(),

        _ => view! {
            <div class="sw-input sw-input--numeric">
                <input
                    type="number"
                    class="cfg-input"
                    step="0.1"
                    placeholder={if unit.is_empty() {
                        format!("{scale_min:.0}–{scale_max:.0}")
                    } else {
                        format!("{unit} ({scale_min:.0}–{scale_max:.0})")
                    }}
                    prop:value=move || value.get().map(|v| v.to_string()).unwrap_or_default()
                    on:input=move |ev| {
                        let v = event_target_value(&ev).parse::<f64>().ok();
                        value.set(v);
                        on_change.run(v);
                    }
                />
            </div>
        }.into_any(),
    }
}
