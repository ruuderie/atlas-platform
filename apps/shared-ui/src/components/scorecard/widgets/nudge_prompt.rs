//! G-27 Nudge Prompt — compact post-activity / pre-activity floating widget.
//!
//! Renders a small floating card that appears after an activity is logged
//! (post-activity mode) or before a scheduled activity (pre-activity / prep mode).
//!
//! The prompt captures 1–4 key dimension ratings without opening the full session form.
//! On submit it calls the `on_submit` callback with the entered scores.
//!
//! # Modes
//! - `"nudge"` — Post-activity: "You just had a call. Rate these now."
//! - `"prep"` — Pre-activity: "Call with Alex tomorrow. Review these beforehand."
//!
//! # AI inference integration
//! If a dimension has `inferred_score` set, the input renders pre-filled with
//! a confidence indicator and "Confirm / Reject" buttons. Rejection calls
//! `on_reject_inference(dimension_id)`. Confirmation calls `on_submit`.

use super::super::models::{ScaleType, SessionDimension};
use leptos::prelude::*;
use uuid::Uuid;

// ── NudgePrompt ───────────────────────────────────────────────────────────────

#[component]
pub fn NudgePrompt(
    /// The entity being rated. Used in the header.
    subject_label: String,
    /// "nudge" (post-activity) or "prep" (pre-activity).
    mode: String,
    /// Activity type that triggered this nudge. Used for context.
    activity_type: String,
    /// Dimensions to capture in this prompt (1–4 recommended).
    dimensions: Vec<SessionDimension>,
    /// Called on submit with (dimension_id, score) pairs.
    on_submit: Callback<Vec<(Uuid, Option<f64>)>>,
    /// Called when the user dismisses without submitting.
    on_dismiss: Callback<()>,
    /// Called when the user rejects an AI-inferred score.
    #[prop(optional)]
    on_reject_inference: Option<Callback<Uuid>>,
) -> impl IntoView {
    let dims = RwSignal::new(dimensions);
    let is_submitting = RwSignal::new(false);

    // Collect draft scores keyed by dimension_id
    let submit = move |_| {
        is_submitting.set(true);
        let scores: Vec<(Uuid, Option<f64>)> = dims
            .get()
            .into_iter()
            .map(|d| {
                // If confirmed inference, use inferred_score; else use draft_score
                let score = d.draft_score.or(d.inferred_score);
                (d.dimension_id, score)
            })
            .collect();
        on_submit.run(scores);
    };

    let header_copy = {
        let mode = mode.clone();
        let activity = activity_type.clone();
        move || match mode.as_str() {
            "prep" => format!(
                "Prep for your {activity} with {subject_label}",
                activity = activity,
                subject_label = subject_label
            ),
            _ => format!(
                "You just had a {activity} with {subject_label}",
                activity = activity,
                subject_label = subject_label
            ),
        }
    };

    let sub_copy = {
        let mode = mode.clone();
        move || match mode.as_str() {
            "prep" => "Review what you know before the call.",
            _ => "Capture these while it's fresh — takes ~20 seconds.",
        }
    };

    view! {
        <div class="nudge-prompt">
            // ── Header ─────────────────────────────────────────────────────
            <div class="nudge-header">
                <div class="nudge-header-left">
                    <span class="nudge-mode-badge">
                        {if mode == "prep" { "📋 Prep" } else { "⚡ Quick Rate" }}
                    </span>
                    <div class="nudge-title">{header_copy}</div>
                    <div class="nudge-subtitle">{sub_copy}</div>
                </div>
                <button
                    class="cfg-icon-btn nudge-dismiss"
                    title="Dismiss"
                    on:click=move |_| on_dismiss.run(())
                >
                    "✕"
                </button>
            </div>

            // ── Dimension inputs ────────────────────────────────────────────
            <div class="nudge-dimensions">
                <For
                    each=move || dims.get()
                    key=|d| d.dimension_id
                    children=move |dim| {
                        let dim_id = dim.dimension_id;
                        let has_inference = dim.has_pending_inference();
                        let confidence = dim.inferred_confidence.unwrap_or(0.0);
                        let reject_cb = on_reject_inference.clone();

                        view! {
                            <div class="nudge-dim-row">
                                <div class="nudge-dim-header">
                                    <span class="nudge-dim-name">{dim.name.clone()}</span>
                                    {if dim.is_required {
                                        Some(view! { <span class="nudge-dim-required">"Required"</span> })
                                    } else { None }}
                                    {if has_inference {
                                        Some(view! {
                                            <span class="nudge-ai-badge" title="AI-inferred from transcript">
                                                "✨ AI" {format!(" {:.0}%", confidence * 100.0)}
                                            </span>
                                        })
                                    } else { None }}
                                </div>

                                // Scale input
                                <NudgeDimensionInput
                                    scale_type=dim.scale_type.clone()
                                    scale_min=dim.scale_min
                                    scale_max=dim.scale_max
                                    unit_label=dim.unit_label.clone().unwrap_or_default()
                                    is_inverted=dim.is_inverted
                                    initial_value=dim.inferred_score.or(dim.draft_score)
                                    on_change=Callback::new(move |score: Option<f64>| {
                                        dims.update(|ds| {
                                            if let Some(d) = ds.iter_mut().find(|d| d.dimension_id == dim_id) {
                                                d.draft_score = score;
                                            }
                                        });
                                    })
                                />

                                // Inference confirm/reject
                                {if has_inference {
                                    Some(view! {
                                        <div class="nudge-inference-actions">
                                            <p class="nudge-inference-evidence">
                                                "\"" {dim.description.clone()} "\""
                                            </p>
                                            {reject_cb.map(|cb| view! {
                                                <button
                                                    class="cfg-btn cfg-btn--ghost cfg-btn--xs nudge-reject"
                                                    on:click=move |_| cb.run(dim_id)
                                                >
                                                    "✕ Reject AI suggestion"
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

            // ── Footer ─────────────────────────────────────────────────────
            <div class="nudge-footer">
                <button
                    class="cfg-btn cfg-btn--ghost cfg-btn--sm"
                    on:click=move |_| on_dismiss.run(())
                >
                    "Skip"
                </button>
                <button
                    class="cfg-btn cfg-btn--primary"
                    disabled=move || is_submitting.get()
                    on:click=submit
                >
                    {move || if is_submitting.get() { "Saving…" } else { "Save Ratings" }}
                </button>
            </div>
        </div>
    }
}

// ── NudgeDimensionInput ───────────────────────────────────────────────────────

/// Renders the appropriate input control for a dimension based on its scale_type.
/// Compact version for the nudge widget — no labels, minimal chrome.
#[component]
fn NudgeDimensionInput(
    scale_type: ScaleType,
    scale_min: f64,
    scale_max: f64,
    #[prop(optional)] unit_label: String,
    is_inverted: bool,
    initial_value: Option<f64>,
    on_change: Callback<Option<f64>>,
) -> impl IntoView {
    let value = RwSignal::new(initial_value);

    let scale_hint = {
        let unit = unit_label.clone();
        let inv_note = if is_inverted {
            " (lower is better)"
        } else {
            ""
        };
        if unit.is_empty() {
            format!("{:.0}–{:.0}{inv_note}", scale_min, scale_max)
        } else {
            format!("{} {}{inv_note}", unit, inv_note)
        }
    };

    match scale_type {
        ScaleType::Boolean => view! {
            <div class="nudge-input nudge-input--boolean">
                <button
                    class=move || if value.get() == Some(1.0) {
                        "nudge-bool-btn nudge-bool-btn--yes nudge-bool-btn--active"
                    } else {
                        "nudge-bool-btn nudge-bool-btn--yes"
                    }
                    on:click=move |_| {
                        value.set(Some(1.0));
                        on_change.run(Some(1.0));
                    }
                >
                    "✓ Yes"
                </button>
                <button
                    class=move || if value.get() == Some(0.0) {
                        "nudge-bool-btn nudge-bool-btn--no nudge-bool-btn--active"
                    } else {
                        "nudge-bool-btn nudge-bool-btn--no"
                    }
                    on:click=move |_| {
                        value.set(Some(0.0));
                        on_change.run(Some(0.0));
                    }
                >
                    "✗ No"
                </button>
            </div>
        }
        .into_any(),

        ScaleType::Rating => {
            // 5-star compact slider for rating dimensions
            let steps = (scale_max - scale_min).round() as usize + 1;
            let step_values: Vec<f64> = (0..steps).map(|i| scale_min + i as f64).collect();

            view! {
                <div class="nudge-input nudge-input--rating">
                    <div class="nudge-rating-dots">
                        <For
                            each=move || step_values.clone()
                            key=|v| (v * 10.0) as i64
                            children=move |v| {
                                view! {
                                    <button
                                        class=move || if value.get() == Some(v) {
                                            "nudge-dot nudge-dot--active"
                                        } else {
                                            "nudge-dot"
                                        }
                                        title=move || format!("{:.0}", v)
                                        on:click=move |_| {
                                            value.set(Some(v));
                                            on_change.run(Some(v));
                                        }
                                    >
                                    </button>
                                }
                            }
                        />
                    </div>
                    <div class="nudge-rating-labels">
                        <span>{if is_inverted { "Best" } else { "Low" }}</span>
                        <span class="nudge-scale-hint">{scale_hint.clone()}</span>
                        <span>{if is_inverted { "Worst" } else { "High" }}</span>
                    </div>
                    {move || value.get().map(|v| view! {
                        <span class="nudge-rating-selected">{format!("{:.0}", v)}</span>
                    })}
                </div>
            }
            .into_any()
        }

        _ => view! {
            // Absolute / generic numeric input
            <div class="nudge-input nudge-input--numeric">
                <input
                    type="number"
                    class="cfg-input"
                    step="0.1"
                    placeholder={scale_hint}
                    prop:value=move || value.get().map(|v| v.to_string()).unwrap_or_default()
                    on:input=move |ev| {
                        let v = event_target_value(&ev).parse::<f64>().ok();
                        value.set(v);
                        on_change.run(v);
                    }
                />
            </div>
        }
        .into_any(),
    }
}
