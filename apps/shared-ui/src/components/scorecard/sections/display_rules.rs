//! G-27 Display Rules Builder — Section 4 of the Scorecard Configurator.
//!
//! Allows business admins to configure context-aware rules that control
//! which dimensions appear on the session form, when they are required,
//! and when they surface as post-activity nudge prompts.
//!
//! # Rule lifecycle in this UI
//! 1. Admin opens the "Display Rules" section in the Configurator.
//! 2. They click "+ Add Rule" to create a new `DisplayRuleForm`.
//! 3. They select a target dimension (or category), trigger, operator, and action.
//! 4. The rule row shows a human-readable summary.
//! 5. On save, the parent Configurator serializes rules and POSTs to the backend.
//!
//! # Tier gate
//! The section renders a paywall notice for tenants without
//! `scorecard_display_rules_enabled`. The parent passes `rules_enabled: bool`.

use leptos::prelude::*;
use uuid::Uuid;
use super::super::models::{DimensionForm, DisplayRuleForm};

// ── Trigger category options ─────────────────────────────────────────────────

const TRIGGER_OPTIONS: &[(&str, &str)] = &[
    ("record_state",     "Record State Change"),
    ("time_proximity",   "Time Proximity (date field)"),
    ("activity_trigger", "Activity Logged"),
    ("score_gap",        "Score Gap on Dimension"),
];

const OPERATOR_OPTIONS_RECORD: &[(&str, &str)] = &[
    ("equals",     "Equals"),
    ("not_equals", "Does not equal"),
    ("in",         "Is one of"),
    ("not_in",     "Is not one of"),
];

const OPERATOR_OPTIONS_TIME: &[(&str, &str)] = &[
    ("within_days",  "Within N days"),
    ("overdue_days", "Overdue by N days"),
];

const OPERATOR_OPTIONS_SCORE: &[(&str, &str)] = &[
    ("dimension_score_below",  "Score below threshold"),
    ("dimension_score_above",  "Score above threshold"),
    ("dimension_unrated",      "Not yet rated"),
];

const OPERATOR_OPTIONS_ACTIVITY: &[(&str, &str)] = &[
    ("activity_type_is", "Activity type is one of"),
];

const ACTION_OPTIONS: &[(&str, &str, &str)] = &[
    ("show",              "Show",               "Make this dimension visible"),
    ("hide",              "Hide",               "Suppress from session form"),
    ("require",           "Require",            "Show and mark as required"),
    ("surface_as_nudge",  "Surface as Nudge",   "Show in post-activity compact prompt"),
    ("show_in_prep_mode", "Prep Mode Only",     "Show only before a scheduled activity"),
    ("show_alert_banner", "Show Alert Banner",  "Show a warning banner in the widget"),
];

const SCOPE_OPTIONS: &[(&str, &str)] = &[
    ("always",        "Always"),
    ("post_activity", "Post-activity only"),
    ("pre_activity",  "Pre-activity only"),
    ("on_score_gap",  "When score gap is active"),
];

// ── DisplayRulesSection ───────────────────────────────────────────────────────

#[component]
pub fn DisplayRulesSection(
    /// All dimensions on this template — used to populate the dimension picker.
    dimensions: ReadSignal<Vec<DimensionForm>>,
    /// All display rules for this template.
    rules: RwSignal<Vec<DisplayRuleForm>>,
    /// Counter for allocating stable local_ids.
    next_local_id: RwSignal<usize>,
    /// Whether this tenant has the Display Rules feature enabled.
    rules_enabled: bool,
) -> impl IntoView {
    let editing_rule: RwSignal<Option<usize>> = RwSignal::new(None);

    let alloc_id = move || {
        let id = next_local_id.get_untracked();
        next_local_id.set(id + 1);
        id
    };

    let add_rule = move |_| {
        let priority = (rules.get_untracked().len() as i32 + 1) * 10;
        let id = alloc_id();
        rules.update(|rs| rs.push(DisplayRuleForm::new(id, priority)));
        editing_rule.set(Some(id));
    };

    let remove_rule = move |local_id: usize| {
        rules.update(|rs| rs.retain(|r| r.local_id != local_id));
        if editing_rule.get_untracked() == Some(local_id) {
            editing_rule.set(None);
        }
    };

    view! {
        <div class="cfg-section cfg-section--rules">
            <div class="cfg-section-header">
                <div class="cfg-section-header-main">
                    <h2 class="cfg-section-title">"Display Rules"</h2>
                    <span class="cfg-badge cfg-badge--pro">"Professional+"</span>
                </div>
                <p class="cfg-section-desc">
                    "Control which dimensions appear, when they are required, and when to surface "
                    "post-activity nudge prompts. Rules are evaluated client-side against the current "
                    "entity field values each time the session form renders."
                </p>
            </div>

            // ── Tier gate ─────────────────────────────────────────────────────
            {if !rules_enabled {
                view! {
                    <div class="cfg-paywall-notice">
                        <div class="cfg-paywall-icon">"🔒"</div>
                        <div>
                            <p class="cfg-paywall-title">"Display Rules require Professional tier"</p>
                            <p class="cfg-paywall-desc">
                                "Upgrade to configure context-aware visibility, nudge prompts, and pre-activity prep mode. "
                                "Contact your admin to enable "
                                <code>"scorecard_display_rules_enabled"</code>
                                " for this tenant."
                            </p>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="cfg-rules-container">
                        // ── Rules list panel ──────────────────────────────────
                        <div class="cfg-rules-list-panel">
                            <div class="cfg-rules-list-header">
                                <span class="cfg-rules-count">
                                    {move || rules.get().len()} " rules"
                                </span>
                                <button
                                    class="cfg-btn cfg-btn--ghost cfg-btn--sm"
                                    on:click=add_rule
                                >
                                    <span class="cfg-icon">"＋"</span>
                                    "Add Rule"
                                </button>
                            </div>

                            <Show
                                when=move || rules.get().is_empty()
                                fallback=|| view! { <span></span> }
                            >
                                <div class="cfg-empty-state cfg-empty-state--rules">
                                    <div class="cfg-empty-icon">"⚡"</div>
                                    <p class="cfg-empty-title">"No rules configured"</p>
                                    <p class="cfg-empty-desc">
                                        "All dimensions render unconditionally. Add a rule to make "
                                        "dimensions context-aware."
                                    </p>
                                    <button
                                        class="cfg-btn cfg-btn--primary cfg-btn--sm"
                                        on:click=add_rule
                                    >
                                        "Add First Rule"
                                    </button>
                                </div>
                            </Show>

                            <div class="cfg-rules-list">
                                <For
                                    each=move || {
                                        let mut rs = rules.get();
                                        rs.sort_by_key(|r| r.priority);
                                        rs
                                    }
                                    key=|r| r.local_id
                                    children=move |rule| {
                                        let local_id = rule.local_id;
                                        let is_editing = move || editing_rule.get() == Some(local_id);
                                        let summary = rule.summary();
                                        let trigger_label = rule.trigger_label();
                                        let action = rule.action.clone();

                                        let action_class = match action.as_str() {
                                            "hide"    => "cfg-rule-action-badge cfg-rule-action-badge--hide",
                                            "require" => "cfg-rule-action-badge cfg-rule-action-badge--require",
                                            "surface_as_nudge" => "cfg-rule-action-badge cfg-rule-action-badge--nudge",
                                            _         => "cfg-rule-action-badge",
                                        };

                                        view! {
                                            <div
                                                class=move || if is_editing() {
                                                    "cfg-rule-row cfg-rule-row--active"
                                                } else {
                                                    "cfg-rule-row"
                                                }
                                            >
                                                <button
                                                    class="cfg-rule-summary"
                                                    on:click=move |_| {
                                                        if editing_rule.get() == Some(local_id) {
                                                            editing_rule.set(None);
                                                        } else {
                                                            editing_rule.set(Some(local_id));
                                                        }
                                                    }
                                                >
                                                    <div class="cfg-rule-summary-left">
                                                        <span class={action_class}>
                                                            {rule.action_label()}
                                                        </span>
                                                        <span class="cfg-rule-summary-text">{summary}</span>
                                                    </div>
                                                    <div class="cfg-rule-summary-right">
                                                        <span class="cfg-rule-trigger-tag">{trigger_label}</span>
                                                        <span class="cfg-rule-priority">
                                                            "p" {rule.priority}
                                                        </span>
                                                    </div>
                                                </button>
                                                <button
                                                    class="cfg-icon-btn cfg-icon-btn--danger"
                                                    title="Remove rule"
                                                    on:click=move |_| remove_rule(local_id)
                                                >
                                                    "✕"
                                                </button>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        </div>

                        // ── Rule editor panel ─────────────────────────────────
                        <div class="cfg-rule-editor-panel">
                            <Show
                                when=move || editing_rule.get().is_some()
                                fallback=|| view! {
                                    <div class="cfg-dim-editor-empty">
                                        <div class="cfg-empty-icon">"←"</div>
                                        <p>"Select a rule to edit it"</p>
                                        <p class="cfg-hint">
                                            "Rules are evaluated in priority order (lowest number first). "
                                            "Require beats Hide beats Show."
                                        </p>
                                    </div>
                                }
                            >
                                {move || {
                                    let local_id = editing_rule.get().unwrap();
                                    view! {
                                        <RuleEditor
                                            local_id=local_id
                                            dimensions=dimensions
                                            rules=rules
                                            on_close=Callback::new(move |_| editing_rule.set(None))
                                        />
                                    }
                                }}
                            </Show>
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ── RuleEditor ────────────────────────────────────────────────────────────────

#[component]
fn RuleEditor(
    local_id: usize,
    dimensions: ReadSignal<Vec<DimensionForm>>,
    rules: RwSignal<Vec<DisplayRuleForm>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let rule = move || rules.get().into_iter().find(|r| r.local_id == local_id);

    let upd = move |f: &dyn Fn(&mut DisplayRuleForm)| {
        rules.update(|rs| {
            if let Some(r) = rs.iter_mut().find(|r| r.local_id == local_id) {
                f(r);
            }
        });
    };

    // Operators change based on trigger category
    let available_operators = move || {
        match rule().map(|r| r.trigger_category).as_deref() {
            Some("record_state")     => OPERATOR_OPTIONS_RECORD,
            Some("time_proximity")   => OPERATOR_OPTIONS_TIME,
            Some("score_gap")        => OPERATOR_OPTIONS_SCORE,
            Some("activity_trigger") => OPERATOR_OPTIONS_ACTIVITY,
            _                        => OPERATOR_OPTIONS_RECORD,
        }
    };

    let trigger_cat = move || rule().map(|r| r.trigger_category).unwrap_or_default();
    let operator = move || rule().map(|r| r.operator).unwrap_or_default();
    let action = move || rule().map(|r| r.action).unwrap_or_default();

    // Show value field for operators that use a scalar
    let needs_scalar_value = move || {
        matches!(
            operator().as_str(),
            "equals" | "not_equals" | "within_days" | "overdue_days"
            | "dimension_score_below" | "dimension_score_above"
        )
    };

    // Show value_list field for operators that use a list
    let needs_list_value = move || {
        matches!(operator().as_str(), "in" | "not_in" | "activity_type_is")
    };

    // Show field_reference for record_state and time_proximity
    let needs_field_ref = move || {
        matches!(trigger_cat().as_str(), "record_state" | "time_proximity")
    };

    // Show alert_message when action is show_alert_banner
    let needs_alert_msg = move || action() == "show_alert_banner";

    view! {
        <div class="cfg-rule-editor">
            <div class="cfg-dim-editor-header">
                <h3 class="cfg-dim-editor-title">
                    {move || rule().map(|r| if r.description.trim().is_empty() {
                        "New Rule".to_string()
                    } else {
                        r.description.clone()
                    }).unwrap_or_default()}
                </h3>
                <button class="cfg-icon-btn" on:click=move |_| on_close.run(())>"✕"</button>
            </div>

            <div class="cfg-dim-editor-body">
                // ── Description ─────────────────────────────────────────────
                <div class="cfg-editor-section">
                    <h4 class="cfg-editor-section-title">"Rule Description"</h4>
                    <div class="cfg-field">
                        <label class="cfg-label">"Description (optional)"</label>
                        <input
                            type="text"
                            class="cfg-input"
                            placeholder="e.g. Show champion_strength when stage is Negotiation"
                            prop:value=move || rule().map(|r| r.description).unwrap_or_default()
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                upd(&|r| r.description = v.clone());
                            }
                        />
                    </div>
                </div>

                // ── Target ──────────────────────────────────────────────────
                <div class="cfg-editor-section">
                    <h4 class="cfg-editor-section-title">"Target"</h4>
                    <p class="cfg-hint">
                        "Which dimension (or category) does this rule act on?"
                    </p>
                    <div class="cfg-field">
                        <label class="cfg-label">"Dimension"</label>
                        <div class="cfg-select-wrap">
                            <select
                                class="cfg-select"
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    let dims = dimensions.get_untracked();
                                    if v == "__category__" {
                                        upd(&|r| { r.dimension_id = None; r.dimension_name = String::new(); });
                                    } else if let Ok(uid) = v.parse::<Uuid>() {
                                        let name = dims.iter().find(|d| d.id == Some(uid))
                                            .map(|d| d.name.clone())
                                            .unwrap_or_default();
                                        upd(&move |r| {
                                            r.dimension_id = Some(uid);
                                            r.dimension_name = name.clone();
                                        });
                                    }
                                }
                            >
                                <option value="__category__">"— Category-level rule —"</option>
                                <For
                                    each=move || {
                                        let mut dims = dimensions.get();
                                        dims.sort_by_key(|d| d.sort_order);
                                        dims
                                    }
                                    key=|d| d.local_id
                                    children=move |dim| {
                                        let dim_id = dim.id.map(|u| u.to_string()).unwrap_or_default();
                                        let label = if dim.name.trim().is_empty() {
                                            format!("[Untitled — {}]", dim.slug)
                                        } else {
                                            dim.name.clone()
                                        };
                                        view! {
                                            <option value={dim_id}>{label}</option>
                                        }
                                    }
                                />
                            </select>
                        </div>
                    </div>

                    // Category target (shown when no dimension selected)
                    <Show when=move || rule().map(|r| r.dimension_id.is_none()).unwrap_or(true)>
                        <div class="cfg-field">
                            <label class="cfg-label">"Category Target"</label>
                            <input
                                type="text"
                                class="cfg-input"
                                placeholder="e.g. deal_health, stakeholder, competitive"
                                prop:value=move || rule().map(|r| r.category_target).unwrap_or_default()
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    upd(&|r| r.category_target = v.clone());
                                }
                            />
                            <p class="cfg-hint">
                                "This rule applies to all active dimensions in this category."
                            </p>
                        </div>
                    </Show>
                </div>

                // ── Trigger ─────────────────────────────────────────────────
                <div class="cfg-editor-section">
                    <h4 class="cfg-editor-section-title">"Trigger Condition"</h4>

                    <div class="cfg-field">
                        <label class="cfg-label">"Trigger Type"</label>
                        <div class="cfg-select-wrap">
                            <select
                                class="cfg-select"
                                prop:value=move || rule().map(|r| r.trigger_category).unwrap_or_default()
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    // Reset operator when trigger type changes
                                    let default_op = match v.as_str() {
                                        "time_proximity"   => "within_days",
                                        "score_gap"        => "dimension_score_below",
                                        "activity_trigger" => "activity_type_is",
                                        _                  => "equals",
                                    }.to_string();
                                    upd(&move |r| {
                                        r.trigger_category = v.clone();
                                        r.operator = default_op.clone();
                                        r.value = String::new();
                                        r.value_list_raw = String::new();
                                    });
                                }
                            >
                                <For
                                    each=|| TRIGGER_OPTIONS.to_vec()
                                    key=|(v, _)| v.to_string()
                                    children=|(val, label)| view! {
                                        <option value={val}>{label}</option>
                                    }
                                />
                            </select>
                        </div>
                    </div>

                    // Field reference (record_state, time_proximity)
                    <Show when=needs_field_ref>
                        <div class="cfg-field">
                            <label class="cfg-label">"Field Reference"</label>
                            <input
                                type="text"
                                class="cfg-input cfg-input--mono"
                                placeholder="e.g. stage, close_date, lead_status"
                                prop:value=move || rule().map(|r| r.field_reference).unwrap_or_default()
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    upd(&|r| r.field_reference = v.clone());
                                }
                            />
                            <p class="cfg-hint">
                                "The field path on the subject entity to evaluate."
                            </p>
                        </div>
                    </Show>

                    // Operator
                    <div class="cfg-field">
                        <label class="cfg-label">"Operator"</label>
                        <div class="cfg-select-wrap">
                            <select
                                class="cfg-select"
                                prop:value=move || rule().map(|r| r.operator).unwrap_or_default()
                                on:change=move |ev| {
                                    let v = event_target_value(&ev);
                                    upd(&|r| r.operator = v.clone());
                                }
                            >
                                <For
                                    each=move || available_operators().to_vec()
                                    key=|(v, _)| v.to_string()
                                    children=|(val, label)| view! {
                                        <option value={val}>{label}</option>
                                    }
                                />
                            </select>
                        </div>
                    </div>

                    // Scalar value
                    <Show when=needs_scalar_value>
                        <div class="cfg-field">
                            <label class="cfg-label">
                                {move || match operator().as_str() {
                                    "within_days" | "overdue_days"     => "Days",
                                    "dimension_score_below"
                                    | "dimension_score_above"          => "Score Threshold",
                                    _                                   => "Value",
                                }}
                            </label>
                            <input
                                type="text"
                                class="cfg-input"
                                placeholder="e.g. Negotiation, 7, 5.0"
                                prop:value=move || rule().map(|r| r.value).unwrap_or_default()
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    upd(&|r| r.value = v.clone());
                                }
                            />
                        </div>
                    </Show>

                    // List value
                    <Show when=needs_list_value>
                        <div class="cfg-field">
                            <label class="cfg-label">
                                {move || match operator().as_str() {
                                    "activity_type_is" => "Activity Types",
                                    _                  => "Values",
                                }}
                            </label>
                            <input
                                type="text"
                                class="cfg-input"
                                placeholder="e.g. call, demo, meeting (comma-separated)"
                                prop:value=move || rule().map(|r| r.value_list_raw).unwrap_or_default()
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    upd(&|r| r.value_list_raw = v.clone());
                                }
                            />
                            <p class="cfg-hint">"Comma-separated list. Whitespace is trimmed."</p>
                        </div>
                    </Show>
                </div>

                // ── Action ──────────────────────────────────────────────────
                <div class="cfg-editor-section">
                    <h4 class="cfg-editor-section-title">"Action"</h4>
                    <p class="cfg-hint">
                        "Conflict resolution: " <strong>"Require"</strong> " > "
                        <strong>"Hide"</strong> " > " <strong>"Show"</strong>
                        ". Highest-priority matching rule wins."
                    </p>

                    // Action radio group
                    <div class="cfg-action-group">
                        <For
                            each=|| ACTION_OPTIONS.to_vec()
                            key=|(v, _, _)| v.to_string()
                            children=move |(val, label, hint)| {
                                let val_str = val.to_string();
                                let val_str2 = val_str.clone();
                                view! {
                                    <label class=move || {
                                        if action() == val_str {
                                            "cfg-action-option cfg-action-option--selected"
                                        } else {
                                            "cfg-action-option"
                                        }
                                    }>
                                        <input
                                            type="radio"
                                            name=format!("rule-action-{local_id}")
                                            value={val}
                                            checked=move || action() == val_str2
                                            on:change=move |ev| {
                                                let v = event_target_value(&ev);
                                                upd(&|r| r.action = v.clone());
                                            }
                                        />
                                        <div class="cfg-action-option-content">
                                            <span class="cfg-action-option-label">{label}</span>
                                            <span class="cfg-action-option-hint">{hint}</span>
                                        </div>
                                    </label>
                                }
                            }
                        />
                    </div>

                    // Alert message (show_alert_banner only)
                    <Show when=needs_alert_msg>
                        <div class="cfg-field">
                            <label class="cfg-label">"Alert Message"</label>
                            <textarea
                                class="cfg-textarea"
                                rows="2"
                                placeholder="e.g. Close date is in 7 days — capture champion strength now."
                                prop:value=move || rule().map(|r| r.alert_message).unwrap_or_default()
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    upd(&|r| r.alert_message = v.clone());
                                }
                            />
                        </div>
                    </Show>
                </div>

                // ── Scope & Priority ─────────────────────────────────────────
                <div class="cfg-editor-section">
                    <h4 class="cfg-editor-section-title">"Scope & Priority"</h4>

                    <div class="cfg-form-row">
                        <div class="cfg-field">
                            <label class="cfg-label">"Mode Scope"</label>
                            <div class="cfg-select-wrap">
                                <select
                                    class="cfg-select"
                                    prop:value=move || rule().map(|r| r.mode_scope).unwrap_or_default()
                                    on:change=move |ev| {
                                        let v = event_target_value(&ev);
                                        upd(&|r| r.mode_scope = v.clone());
                                    }
                                >
                                    <For
                                        each=|| SCOPE_OPTIONS.to_vec()
                                        key=|(v, _)| v.to_string()
                                        children=|(val, label)| view! {
                                            <option value={val}>{label}</option>
                                        }
                                    />
                                </select>
                            </div>
                            <p class="cfg-hint">
                                "When must this rendering context be active for the rule to apply?"
                            </p>
                        </div>

                        <div class="cfg-field">
                            <label class="cfg-label">"Priority"</label>
                            <input
                                type="number"
                                class="cfg-input"
                                min="1"
                                step="1"
                                prop:value=move || rule().map(|r| r.priority).unwrap_or(10)
                                on:input=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                        upd(&|r| r.priority = v);
                                    }
                                }
                            />
                            <p class="cfg-hint">
                                "Lower = higher priority. Tie-break when multiple rules conflict."
                            </p>
                        </div>
                    </div>

                    <div class="cfg-toggle-row">
                        <div>
                            <p class="cfg-label">"Active"</p>
                            <p class="cfg-hint">"Inactive rules are stored but never evaluated."</p>
                        </div>
                        <button
                            class=move || if rule().map(|r| r.is_active).unwrap_or(true) {
                                "cfg-toggle cfg-toggle--on"
                            } else {
                                "cfg-toggle"
                            }
                            role="switch"
                            on:click=move |_| upd(&|r| r.is_active = !r.is_active)
                        >
                            <span class="cfg-toggle-thumb"></span>
                        </button>
                    </div>
                </div>

                // ── Preview ─────────────────────────────────────────────────
                <div class="cfg-editor-section cfg-editor-section--preview">
                    <h4 class="cfg-editor-section-title">"Rule Preview"</h4>
                    <div class="cfg-rule-preview">
                        <div class="cfg-rule-preview-line">
                            <span class="cfg-rule-preview-label">"If"</span>
                            <code class="cfg-rule-preview-val">
                                {move || rule().map(|r| {
                                    match r.trigger_category.as_str() {
                                        "record_state" => format!(
                                            "{} {} {}",
                                            r.field_reference,
                                            r.operator.replace('_', " "),
                                            if r.value_list_raw.is_empty() { r.value.clone() } else { format!("[{}]", r.value_list_raw) }
                                        ),
                                        "time_proximity" => format!(
                                            "{} {} {} days",
                                            r.field_reference,
                                            r.operator.replace('_', " "),
                                            r.value
                                        ),
                                        "activity_trigger" => format!(
                                            "activity type is one of [{}]",
                                            r.value_list_raw
                                        ),
                                        "score_gap" => format!(
                                            "dimension score {} {}",
                                            r.operator.replace('_', " "),
                                            r.value
                                        ),
                                        _ => "—".to_string(),
                                    }
                                }).unwrap_or_default()}
                            </code>
                        </div>
                        <div class="cfg-rule-preview-line">
                            <span class="cfg-rule-preview-label">"Then"</span>
                            <code class="cfg-rule-preview-val">
                                {move || rule().map(|r| {
                                    let target = if !r.dimension_name.is_empty() {
                                        r.dimension_name.clone()
                                    } else if !r.category_target.is_empty() {
                                        format!("[{}] category", r.category_target)
                                    } else {
                                        "target".to_string()
                                    };
                                    format!("{} → {}", target, r.action.replace('_', " "))
                                }).unwrap_or_default()}
                            </code>
                        </div>
                        <div class="cfg-rule-preview-line">
                            <span class="cfg-rule-preview-label">"Scope"</span>
                            <code class="cfg-rule-preview-val">
                                {move || rule().map(|r| r.mode_scope.replace('_', " ")).unwrap_or_default()}
                            </code>
                            <span class="cfg-rule-preview-sep">"·"</span>
                            <span class="cfg-rule-preview-label">"Priority"</span>
                            <code class="cfg-rule-preview-val">
                                {move || rule().map(|r| r.priority.to_string()).unwrap_or_default()}
                            </code>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
