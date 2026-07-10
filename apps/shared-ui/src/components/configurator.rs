use leptos::prelude::*;
use crate::components::scorecard::{
    DisplayRulesSection,
    models::{DisplayRuleForm, DimensionForm, OptionForm, ScaleType, TemplateForm},
};

/// Re-export for consumers that import from `configurator`.
pub use crate::components::scorecard::models::{ConfiguratorMode, TemplateSavePayload};

// ═══════════════════════════════════════════════════════════════════════════
// Configurator — Top-level entry point
//
// Three-panel layout:
//   Left sidebar  — Template meta + section nav
//   Center        — Active section content
//   Right drawer  — Dimension detail editor (when a dim is selected)
// ═══════════════════════════════════════════════════════════════════════════

#[component]
pub fn Configurator(
    /// Existing template data for edit mode; None for create mode.
    #[prop(optional)] initial_template: Option<TemplateForm>,
    /// Existing dimensions for edit mode.
    #[prop(optional)] initial_dimensions: Option<Vec<DimensionForm>>,
    /// Existing display rules for edit mode.
    #[prop(optional)] initial_display_rules: Option<Vec<DisplayRuleForm>>,
    /// Operator vs tenant-admin field locks (default: PlatformOperator).
    #[prop(default = ConfiguratorMode::PlatformOperator)] mode: ConfiguratorMode,
    /// Called on save with full template payload (incl. rules + display_config).
    on_save: Callback<TemplateSavePayload>,
    /// Called when the user cancels.
    #[prop(optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let template = RwSignal::new(initial_template.unwrap_or_default());
    let dimensions = RwSignal::new(initial_dimensions.unwrap_or_default());
    let next_local_id = RwSignal::new({
        let max_dim = dimensions
            .get_untracked()
            .iter()
            .map(|d| d.local_id)
            .max()
            .unwrap_or(0);
        let max_rule = initial_display_rules
            .as_ref()
            .map(|r| r.iter().map(|x| x.local_id).max().unwrap_or(0))
            .unwrap_or(0);
        max_dim.max(max_rule).saturating_add(1).max(100)
    });
    // "overview" | "dimensions" | "combinator" | "display_config" | "display_rules"
    let active_section = RwSignal::new("overview".to_string());
    // Which dimension is open in the right-side detail editor (keyed by local_id)
    let editing_dim: RwSignal<Option<usize>> = RwSignal::new(None);
    let save_attempted = RwSignal::new(false);
    let display_rules: RwSignal<Vec<DisplayRuleForm>> =
        RwSignal::new(initial_display_rules.unwrap_or_default());

    let alloc_id = move || {
        let id = next_local_id.get_untracked();
        next_local_id.set(id + 1);
        id
    };

    let add_dimension = move |_| {
        let new_id = alloc_id();
        dimensions.update(|dims| {
            let sort = dims.len() as i32;
            dims.push(DimensionForm::new(new_id, sort));
        });
        editing_dim.set(Some(new_id));
        active_section.set("dimensions".to_string());
    };

    let remove_dimension = move |local_id: usize| {
        dimensions.update(|dims| dims.retain(|d| d.local_id != local_id));
        editing_dim.set(None);
    };

    let move_up = move |local_id: usize| {
        dimensions.update(|dims| {
            if let Some(pos) = dims.iter().position(|d| d.local_id == local_id) {
                if pos > 0 {
                    dims.swap(pos, pos - 1);
                    dims.iter_mut().enumerate().for_each(|(i, d)| d.sort_order = i as i32);
                }
            }
        });
    };

    let move_down = move |local_id: usize| {
        dimensions.update(|dims| {
            if let Some(pos) = dims.iter().position(|d| d.local_id == local_id) {
                if pos + 1 < dims.len() {
                    dims.swap(pos, pos + 1);
                    dims.iter_mut().enumerate().for_each(|(i, d)| d.sort_order = i as i32);
                }
            }
        });
    };

    let handle_save = move |_| {
        save_attempted.set(true);
        let t = template.get();
        if t.name.trim().is_empty() || t.entity_type.is_empty() {
            active_section.set("overview".to_string());
            return;
        }
        let display_config = t.display_config.clone();
        on_save.run(TemplateSavePayload {
            template: t,
            dimensions: dimensions.get(),
            display_rules: display_rules.get(),
            display_config,
        });
    };

    view! {
        <div class="configurator-root w-full">
            <ConfiguratorTopBar
                template=template.read_only()
                active_section=active_section
                on_save=Callback::new(handle_save)
                on_cancel=on_cancel
                on_add_dimension=Callback::new(add_dimension)
            />

            <div class="configurator-body">
                <ConfiguratorSidebar
                    template=template.read_only()
                    dimensions=dimensions.read_only()
                    active_section=active_section
                    editing_dim=editing_dim
                    on_add_dimension=Callback::new(add_dimension)
                />

                <div class="configurator-main">
                    <Show when=move || active_section.get() == "overview">
                        <TemplateOverviewSection
                            template=template
                            save_attempted=save_attempted.read_only()
                            mode=mode
                        />
                    </Show>
                    <Show when=move || active_section.get() == "dimensions">
                        <DimensionsSection
                            dimensions=dimensions
                            editing_dim=editing_dim
                            next_local_id=next_local_id
                            on_add=Callback::new(add_dimension)
                            on_remove=Callback::new(move |id| remove_dimension(id))
                            on_move_up=Callback::new(move |id| move_up(id))
                            on_move_down=Callback::new(move |id| move_down(id))
                        />
                    </Show>
                    <Show when=move || active_section.get() == "combinator">
                        <CombinatorSection dimensions=dimensions />
                    </Show>
                    <Show when=move || active_section.get() == "display_config">
                        <DisplayConfigSection template=template />
                    </Show>
                    <Show when=move || active_section.get() == "display_rules">
                        <DisplayRulesSection
                            dimensions=dimensions.read_only()
                            rules=display_rules
                            next_local_id=next_local_id
                            rules_enabled=true
                        />
                    </Show>
                </div>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Top Bar
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn ConfiguratorTopBar(
    template: ReadSignal<TemplateForm>,
    active_section: RwSignal<String>,
    on_save: Callback<()>,
    on_cancel: Option<Callback<()>>,
    on_add_dimension: Callback<()>,
) -> impl IntoView {
    let title = move || {
        let t = template.get();
        if t.name.trim().is_empty() {
            "New Scorecard Template".to_string()
        } else {
            t.name.clone()
        }
    };

    let entity_label = move || {
        match template.get().entity_type.as_str() {
            "atlas_lead"     => "Lead",
            "atlas_account"  => "Account",
            "atlas_contact"  => "Contact",
            "atlas_asset"    => "Asset",
            "atlas_case"     => "Case",
            other            => other,
        }
        .to_string()
    };

    view! {
        <div class="cfg-topbar">
            <div class="cfg-topbar-left">
                <div class="cfg-breadcrumb">
                    <span class="cfg-breadcrumb-root">"Scorecard Templates"</span>
                    <span class="cfg-breadcrumb-sep">"/"</span>
                    <span class="cfg-breadcrumb-current">{title}</span>
                </div>
                <div class="cfg-entity-badge">
                    <span class="cfg-entity-dot"></span>
                    {entity_label}
                </div>
            </div>
            <div class="cfg-topbar-nav">
                {["overview", "dimensions", "combinator", "display_config", "display_rules"].iter().map(|&sec| {
                    let sec_str = sec.to_string();
                    let sec_str2 = sec_str.clone();
                    let label = match sec {
                        "overview"        => "Template",
                        "dimensions"      => "Dimensions",
                        "combinator"      => "Combinator",
                        "display_config"  => "Display Config",
                        "display_rules"   => "Display Rules",
                        _                 => sec,
                    };
                    view! {
                        <button
                            class=move || if active_section.get() == sec_str {
                                "cfg-nav-btn cfg-nav-btn--active"
                            } else {
                                "cfg-nav-btn"
                            }
                            on:click=move |_| active_section.set(sec_str2.clone())
                        >
                            {label}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
            <div class="cfg-topbar-actions">
                <Show when=move || active_section.get() == "dimensions">
                    <button
                        class="cfg-btn cfg-btn--ghost"
                        on:click=move |_| on_add_dimension.run(())
                    >
                        <span class="cfg-icon">"＋"</span>
                        "Add Dimension"
                    </button>
                </Show>
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
                    on:click=move |_| on_save.run(())
                >
                    <span class="cfg-icon">"✓"</span>
                    "Save Template"
                </button>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Left Sidebar — section navigator + dimensions mini-list
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn ConfiguratorSidebar(
    template: ReadSignal<TemplateForm>,
    dimensions: ReadSignal<Vec<DimensionForm>>,
    active_section: RwSignal<String>,
    editing_dim: RwSignal<Option<usize>>,
    on_add_dimension: Callback<()>,
) -> impl IntoView {
    let is_published = move || template.get().is_published;

    view! {
        <aside class="cfg-sidebar">
            // Status pill
            <div class="cfg-sidebar-status">
                <span class=move || if is_published() { "cfg-status cfg-status--live" } else { "cfg-status cfg-status--draft" }>
                    {move || if is_published() { "Live" } else { "Draft" }}
                </span>
            </div>

            // Section nav
            <nav class="cfg-sidebar-nav">
                <SidebarNavItem
                    label="Template Settings"
                    icon="⚙"
                    section="overview"
                    active_section=active_section
                />
                <SidebarNavItem
                    label="Dimensions"
                    icon="◫"
                    section="dimensions"
                    active_section=active_section
                />
                <SidebarNavItem
                    label="Combinator Config"
                    icon="⊕"
                    section="combinator"
                    active_section=active_section
                />
                <SidebarNavItem
                    label="Display Config"
                    icon="▣"
                    section="display_config"
                    active_section=active_section
                />
                <SidebarNavItem
                    label="Display Rules"
                    icon="⚡"
                    section="display_rules"
                    active_section=active_section
                />
            </nav>

            // Dimensions quick-list (shown when on dimensions section)
            <Show when=move || active_section.get() == "dimensions">
                <div class="cfg-sidebar-dims">
                    <div class="cfg-sidebar-dims-header">
                        <span>"Dimensions"</span>
                        <span class="cfg-dim-count">{move || dimensions.get().len()}</span>
                    </div>
                    <div class="cfg-sidebar-dims-list">
                        <For
                            each=move || dimensions.get()
                            key=|d| d.local_id
                            children=move |dim| {
                                let local_id = dim.local_id;
                                view! {
                                    <button
                                        class=move || {
                                            let selected = editing_dim.get() == Some(local_id);
                                            if selected { "cfg-sidebar-dim cfg-sidebar-dim--active" }
                                            else { "cfg-sidebar-dim" }
                                        }
                                        on:click=move |_| {
                                            editing_dim.set(Some(local_id));
                                            active_section.set("dimensions".to_string());
                                        }
                                    >
                                        <span class=move || {
                                            if dim.is_active { "cfg-dim-dot cfg-dim-dot--active" }
                                            else { "cfg-dim-dot cfg-dim-dot--inactive" }
                                        }></span>
                                        <span class="cfg-dim-name">
                                            {if dim.name.trim().is_empty() { "Untitled Dimension".to_string() } else { dim.name.clone() }}
                                        </span>
                                    </button>
                                }
                            }
                        />
                        <button
                            class="cfg-sidebar-add-dim"
                            on:click=move |_| on_add_dimension.run(())
                        >
                            <span>"＋"</span>
                            " Add Dimension"
                        </button>
                    </div>
                </div>
            </Show>
        </aside>
    }
}

#[component]
fn SidebarNavItem(
    label: &'static str,
    icon: &'static str,
    section: &'static str,
    active_section: RwSignal<String>,
) -> impl IntoView {
    let sec = section.to_string();
    let sec2 = sec.clone();
    view! {
        <button
            class=move || if active_section.get() == sec { "cfg-sidebar-item cfg-sidebar-item--active" } else { "cfg-sidebar-item" }
            on:click=move |_| active_section.set(sec2.clone())
        >
            <span class="cfg-sidebar-icon">{icon}</span>
            <span>{label}</span>
        </button>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 1: Template Overview
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn TemplateOverviewSection(
    template: RwSignal<TemplateForm>,
    save_attempted: ReadSignal<bool>,
    mode: ConfiguratorMode,
) -> impl IntoView {
    let name_error = move || {
        save_attempted.get() && template.get().name.trim().is_empty()
    };
    let is_operator = mode == ConfiguratorMode::PlatformOperator;

    view! {
        <div class="cfg-section">
            <div class="cfg-section-header">
                <h2 class="cfg-section-title">"Template Settings"</h2>
                <p class="cfg-section-desc">"Define the scoring blueprint. Every scorecard created from this template inherits these defaults."</p>
            </div>

            <div class="cfg-form-grid">
                // ── Name ─────────────────────────────────────────────────
                <div class="cfg-field cfg-field--full">
                    <label class="cfg-label">"Template Name" <span class="cfg-required">"*"</span></label>
                    <input
                        type="text"
                        class=move || if name_error() { "cfg-input cfg-input--error" } else { "cfg-input" }
                        placeholder="e.g. Lead Quality Scorecard"
                        prop:value=move || template.get().name.clone()
                        on:input=move |ev| template.update(|t| t.name = event_target_value(&ev))
                    />
                    <Show when=name_error>
                        <p class="cfg-error-msg">"Template name is required."</p>
                    </Show>
                </div>

                // ── Description ───────────────────────────────────────────
                <div class="cfg-field cfg-field--full">
                    <label class="cfg-label">"Description"</label>
                    <textarea
                        class="cfg-textarea"
                        rows="3"
                        placeholder="Explain what this template measures and when to use it..."
                        prop:value=move || template.get().description.clone()
                        on:input=move |ev| template.update(|t| t.description = event_target_value(&ev))
                    />
                </div>

                // ── Entity Type ───────────────────────────────────────────
                <div class="cfg-field">
                    <label class="cfg-label">"Entity Type" <span class="cfg-required">"*"</span></label>
                    <div class="cfg-select-wrap">
                        <select
                            class="cfg-select"
                            prop:value=move || template.get().entity_type.clone()
                            on:change=move |ev| template.update(|t| t.entity_type = event_target_value(&ev))
                        >
                            <option value="atlas_lead">"Lead"</option>
                            <option value="atlas_account">"Account"</option>
                            <option value="atlas_contact">"Contact"</option>
                            <option value="atlas_opportunity">"Opportunity"</option>
                            <option value="atlas_asset">"Asset"</option>
                            <option value="atlas_case">"Case"</option>
                            <option value="tenant">"Tenant"</option>
                            <option value="app_instance">"App Instance"</option>
                        </select>
                    </div>
                    <p class="cfg-hint">"Scorecards from this template will be attached to records of this type."</p>
                </div>

                // ── Scoring Method ────────────────────────────────────────
                <div class="cfg-field">
                    <label class="cfg-label">"Scoring Method"</label>
                    <div class="cfg-select-wrap">
                        <select
                            class="cfg-select"
                            prop:value=move || template.get().scoring_method.clone()
                            on:change=move |ev| template.update(|t| t.scoring_method = event_target_value(&ev))
                        >
                            <option value="weighted_mean">"Weighted Mean"</option>
                            <option value="simple_mean">"Simple Mean"</option>
                            <option value="percentile_rank">"Percentile Rank"</option>
                        </select>
                    </div>
                    <p class="cfg-hint">"How dimension scores are combined into a composite score."</p>
                </div>

                // ── Scale range ───────────────────────────────────────────
                <div class="cfg-field">
                    <label class="cfg-label">"Default Scale Min"</label>
                    <input
                        type="number"
                        class="cfg-input"
                        step="0.1"
                        prop:value=move || template.get().default_scale_min
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                template.update(|t| t.default_scale_min = v);
                            }
                        }
                    />
                </div>

                <div class="cfg-field">
                    <label class="cfg-label">"Default Scale Max"</label>
                    <input
                        type="number"
                        class="cfg-input"
                        step="0.1"
                        prop:value=move || template.get().default_scale_max
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                template.update(|t| t.default_scale_max = v);
                            }
                        }
                    />
                </div>

                // ── Min entries to publish ────────────────────────────────
                <div class="cfg-field">
                    <label class="cfg-label">"Min Entries to Publish"</label>
                    <input
                        type="number"
                        class="cfg-input"
                        min="1"
                        prop:value=move || template.get().min_entries_to_publish
                        on:input=move |ev| {
                            if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                template.update(|t| t.min_entries_to_publish = v);
                            }
                        }
                    />
                    <p class="cfg-hint">"Composite score is hidden until this many entries exist."</p>
                </div>

                // ── Template scope (PlatformOperator only) ────────────────
                <Show when=move || is_operator>
                    <div class="cfg-field">
                        <label class="cfg-label">"Template Scope"</label>
                        <div class="cfg-select-wrap">
                            <select
                                class="cfg-select"
                                prop:value=move || template.get().template_scope.clone()
                                on:change=move |ev| template.update(|t| t.template_scope = event_target_value(&ev))
                            >
                                <option value="platform">"Platform (benchmark-eligible)"</option>
                                <option value="tenant">"Tenant (private)"</option>
                            </select>
                        </div>
                        <p class="cfg-hint">"Platform-scoped templates can join cross-tenant benchmark pools."</p>
                    </div>
                </Show>

                // ── Cold start / calibration (operator-facing) ────────────
                <Show when=move || is_operator>
                    <div class="cfg-field">
                        <label class="cfg-label">"Cold Start Strategy"</label>
                        <div class="cfg-select-wrap">
                            <select
                                class="cfg-select"
                                prop:value=move || template.get().cold_start_strategy.clone()
                                on:change=move |ev| template.update(|t| t.cold_start_strategy = event_target_value(&ev))
                            >
                                <option value="suppress">"Suppress"</option>
                                <option value="bayesian_prior">"Bayesian Prior"</option>
                                <option value="show_raw">"Show Raw"</option>
                            </select>
                        </div>
                    </div>
                    <div class="cfg-field">
                        <label class="cfg-label">"Cold Start Saturation"</label>
                        <input
                            type="number"
                            class="cfg-input"
                            min="1"
                            prop:value=move || template.get().cold_start_saturation_threshold
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    template.update(|t| t.cold_start_saturation_threshold = v);
                                }
                            }
                        />
                    </div>
                    <div class="cfg-field">
                        <label class="cfg-label">"Calibration Min Entries"</label>
                        <input
                            type="number"
                            class="cfg-input"
                            min="1"
                            prop:value=move || template.get().calibration_minimum_entries
                            on:input=move |ev| {
                                if let Ok(v) = event_target_value(&ev).parse::<i32>() {
                                    template.update(|t| t.calibration_minimum_entries = v);
                                }
                            }
                        />
                    </div>
                </Show>

                // ── Published toggle ──────────────────────────────────────
                <div class="cfg-field cfg-field--full">
                    <div class="cfg-toggle-row">
                        <div>
                            <p class="cfg-label">"Published"</p>
                            <p class="cfg-hint">"When live, scorecards can be rated by contributors and The Combinator uses it for matching."</p>
                        </div>
                        <button
                            class=move || if template.get().is_published { "cfg-toggle cfg-toggle--on" } else { "cfg-toggle" }
                            role="switch"
                            aria-checked=move || template.get().is_published.to_string()
                            on:click=move |_| template.update(|t| t.is_published = !t.is_published)
                        >
                            <span class="cfg-toggle-thumb"></span>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Display Config — surface toggles (separate from Display Rules)
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn DisplayConfigSection(template: RwSignal<TemplateForm>) -> impl IntoView {
    let toggle = move |key: &'static str| {
        template.update(|t| {
            let c = &mut t.display_config;
            match key {
                "show_on_portfolio_table" => c.show_on_portfolio_table = !c.show_on_portfolio_table,
                "show_on_anomaly_panel" => c.show_on_anomaly_panel = !c.show_on_anomaly_panel,
                "show_on_leaderboard" => c.show_on_leaderboard = !c.show_on_leaderboard,
                "show_on_maintenance_queue" => c.show_on_maintenance_queue = !c.show_on_maintenance_queue,
                "show_on_property_detail" => c.show_on_property_detail = !c.show_on_property_detail,
                "show_on_lead_card" => c.show_on_lead_card = !c.show_on_lead_card,
                "show_on_public_listing" => c.show_on_public_listing = !c.show_on_public_listing,
                "tenant_visible" => c.tenant_visible = !c.tenant_visible,
                "nudge_on_maintenance_case_close" => {
                    c.nudge_on_maintenance_case_close = !c.nudge_on_maintenance_case_close
                }
                "nudge_on_str_checkout" => c.nudge_on_str_checkout = !c.nudge_on_str_checkout,
                "collapsed_by_default" => c.collapsed_by_default = !c.collapsed_by_default,
                _ => {}
            }
        });
    };

    let row = move |key: &'static str, label: &'static str, hint: &'static str| {
        view! {
            <div class="cfg-field cfg-field--full">
                <div class="cfg-toggle-row">
                    <div>
                        <p class="cfg-label">{label}</p>
                        <p class="cfg-hint">{hint}</p>
                    </div>
                    <button
                        class=move || {
                            let on = match key {
                                "show_on_portfolio_table" => template.get().display_config.show_on_portfolio_table,
                                "show_on_anomaly_panel" => template.get().display_config.show_on_anomaly_panel,
                                "show_on_leaderboard" => template.get().display_config.show_on_leaderboard,
                                "show_on_maintenance_queue" => template.get().display_config.show_on_maintenance_queue,
                                "show_on_property_detail" => template.get().display_config.show_on_property_detail,
                                "show_on_lead_card" => template.get().display_config.show_on_lead_card,
                                "show_on_public_listing" => template.get().display_config.show_on_public_listing,
                                "tenant_visible" => template.get().display_config.tenant_visible,
                                "nudge_on_maintenance_case_close" => {
                                    template.get().display_config.nudge_on_maintenance_case_close
                                }
                                "nudge_on_str_checkout" => template.get().display_config.nudge_on_str_checkout,
                                "collapsed_by_default" => template.get().display_config.collapsed_by_default,
                                _ => false,
                            };
                            if on { "cfg-toggle cfg-toggle--on" } else { "cfg-toggle" }
                        }
                        role="switch"
                        on:click=move |_| toggle(key)
                    >
                        <span class="cfg-toggle-thumb"></span>
                    </button>
                </div>
            </div>
        }
    };

    view! {
        <div class="cfg-section">
            <div class="cfg-section-header">
                <h2 class="cfg-section-title">"Display Config"</h2>
                <p class="cfg-section-desc">
                    "Where this scorecard surfaces across product UIs. Separate from conditional Display Rules."
                </p>
            </div>
            <div class="cfg-form-grid">
                {row("show_on_portfolio_table", "Portfolio table", "Show aggregate column on portfolio views.")}
                {row("show_on_anomaly_panel", "Anomaly panel", "Include in anomaly / alert surfaces.")}
                {row("show_on_leaderboard", "Leaderboard", "Eligible for leaderboard rankings.")}
                {row("show_on_maintenance_queue", "Maintenance queue", "Surface on maintenance work queues.")}
                {row("show_on_property_detail", "Property detail", "Show on property / asset detail.")}
                {row("show_on_lead_card", "Lead card", "Compact badge on lead cards.")}
                {row("show_on_public_listing", "Public listing", "Visible on public listing pages.")}
                {row("tenant_visible", "Tenant visible", "Visible to tenant-facing users.")}
                {row("nudge_on_maintenance_case_close", "Nudge on case close", "Prompt rating when a maintenance case closes.")}
                {row("nudge_on_str_checkout", "Nudge on STR checkout", "Prompt rating after STR checkout.")}
                {row("collapsed_by_default", "Collapsed by default", "Widget starts collapsed on detail pages.")}

                <div class="cfg-field">
                    <label class="cfg-label">"Min Entries Before Display"</label>
                    <input
                        type="number"
                        class="cfg-input"
                        min="0"
                        prop:value=move || {
                            template
                                .get()
                                .display_config
                                .min_entries_before_display
                                .map(|n| n.to_string())
                                .unwrap_or_default()
                        }
                        on:input=move |ev| {
                            let raw = event_target_value(&ev);
                            template.update(|t| {
                                t.display_config.min_entries_before_display = if raw.trim().is_empty() {
                                    None
                                } else {
                                    raw.parse::<i32>().ok()
                                };
                            });
                        }
                    />
                    <p class="cfg-hint">"Leave empty for no minimum."</p>
                </div>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 2: Dimensions
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn DimensionsSection(
    dimensions: RwSignal<Vec<DimensionForm>>,
    editing_dim: RwSignal<Option<usize>>,
    next_local_id: RwSignal<usize>,
    on_add: Callback<()>,
    on_remove: Callback<usize>,
    on_move_up: Callback<usize>,
    on_move_down: Callback<usize>,
) -> impl IntoView {
    view! {
        <div class="cfg-section cfg-section--split">
            // ── Dimensions list ───────────────────────────────────────────
            <div class="cfg-dim-list-panel">
                <div class="cfg-section-header">
                    <h2 class="cfg-section-title">"Dimensions"</h2>
                    <p class="cfg-section-desc">"Each dimension is one rated axis. Drag to reorder — order affects the composite vector used by The Combinator."</p>
                </div>

                <Show when=move || dimensions.get().is_empty()>
                    <div class="cfg-empty-state">
                        <div class="cfg-empty-icon">"◫"</div>
                        <p class="cfg-empty-title">"No dimensions yet"</p>
                        <p class="cfg-empty-desc">"Add your first dimension to start building the scoring model."</p>
                        <button class="cfg-btn cfg-btn--primary" on:click=move |_| on_add.run(())>
                            "Add First Dimension"
                        </button>
                    </div>
                </Show>

                <div class="cfg-dim-list">
                    <For
                        each=move || {
                            let mut dims = dimensions.get();
                            dims.sort_by_key(|d| d.sort_order);
                            dims.into_iter().enumerate().collect::<Vec<_>>()
                        }
                        key=|(_, d)| d.local_id
                        children=move |(pos, dim)| {
                            let local_id = dim.local_id;
                            let is_first = pos == 0;
                            let is_last = {
                                let len = dimensions.get().len();
                                pos + 1 >= len
                            };
                            let is_editing = move || editing_dim.get() == Some(local_id);
                            // Extract before view! to avoid partial move of dim
                            let is_active = dim.is_active;
                            let dim_name = if dim.name.trim().is_empty() { "Untitled Dimension".to_string() } else { dim.name.clone() };
                            let scale_type = dim.scale_type.clone();
                            let weight_fmt = format!("{:.1}", dim.weight);
                            let scale_fmt = format!("{:.0}–{:.0}", dim.scale_min, dim.scale_max);
                            let options_count = dim.options.len();
                            let has_options = options_count > 0;

                            view! {
                                <div class=move || if is_editing() { "cfg-dim-row cfg-dim-row--active" } else { "cfg-dim-row" }>
                                    // Drag handle / reorder
                                    <div class="cfg-dim-reorder">
                                        <button
                                            class="cfg-icon-btn"
                                            disabled=is_first
                                            title="Move up"
                                            on:click=move |_| on_move_up.run(local_id)
                                        >"▲"</button>
                                        <button
                                            class="cfg-icon-btn"
                                            disabled=is_last
                                            title="Move down"
                                            on:click=move |_| on_move_down.run(local_id)
                                        >"▼"</button>
                                    </div>

                                    // Dimension summary — click to edit
                                    <button
                                        class="cfg-dim-summary"
                                        on:click=move |_| editing_dim.set(Some(local_id))
                                    >
                                        <div class="cfg-dim-summary-main">
                                            <span class=if is_active { "cfg-dim-dot cfg-dim-dot--active" } else { "cfg-dim-dot cfg-dim-dot--inactive" }></span>
                                            <span class="cfg-dim-summary-name">{dim_name}</span>
                                            <span class="cfg-dim-summary-type">{scale_type.to_string()}</span>
                                        </div>
                                        <div class="cfg-dim-summary-meta">
                                            <span class="cfg-dim-meta-pill">"weight: " {weight_fmt}</span>
                                            <span class="cfg-dim-meta-pill">{scale_fmt}</span>
                                            <Show when=move || has_options>
                                                <span class="cfg-dim-meta-pill">{options_count} " options"</span>
                                            </Show>
                                        </div>
                                    </button>

                                    // Row actions
                                    <div class="cfg-dim-row-actions">
                                        <button
                                            class="cfg-icon-btn cfg-icon-btn--danger"
                                            title="Remove dimension"
                                            on:click=move |_| on_remove.run(local_id)
                                        >"✕"</button>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>

                <Show when=move || !dimensions.get().is_empty()>
                    <button class="cfg-add-dim-btn" on:click=move |_| on_add.run(())>
                        <span>"＋"</span>" Add Dimension"
                    </button>
                </Show>
            </div>

            // ── Right: Dimension editor ───────────────────────────────────
            <div class="cfg-dim-editor-panel">
                <Show
                    when=move || editing_dim.get().is_some()
                    fallback=|| view! {
                        <div class="cfg-dim-editor-empty">
                            <div class="cfg-empty-icon">"←"</div>
                            <p>"Select a dimension to edit it"</p>
                        </div>
                    }
                >
                    {move || {
                        let local_id = editing_dim.get().unwrap();
                        view! {
                            <DimensionEditor
                                local_id=local_id
                                dimensions=dimensions
                                next_local_id=next_local_id
                                on_close=Callback::new(move |_| editing_dim.set(None))
                            />
                        }
                    }}
                </Show>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Dimension Editor — right panel when a dimension is selected
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn DimensionEditor(
    local_id: usize,
    dimensions: RwSignal<Vec<DimensionForm>>,
    next_local_id: RwSignal<usize>,
    on_close: Callback<()>,
) -> impl IntoView {
    let update_dim = move |f: &dyn Fn(&mut DimensionForm)| {
        dimensions.update(|dims| {
            if let Some(dim) = dims.iter_mut().find(|d| d.local_id == local_id) {
                f(dim);
            }
        });
    };

    let dim = move || {
        dimensions.get().into_iter().find(|d| d.local_id == local_id)
    };

    let needs_options = move || {
        dim().map(|d| d.scale_type == ScaleType::PollSingle || d.scale_type == ScaleType::PollMulti)
            .unwrap_or(false)
    };

    let alloc_opt_id = move || {
        let id = next_local_id.get_untracked();
        next_local_id.set(id + 1);
        id
    };

    let add_option = move |_| {
        dimensions.update(|dims| {
            if let Some(dim) = dims.iter_mut().find(|d| d.local_id == local_id) {
                let sort = dim.options.len() as i32;
                dim.options.push(OptionForm::new(alloc_opt_id(), sort));
            }
        });
    };

    view! {
        <div class="cfg-dim-editor">
            <div class="cfg-dim-editor-header">
                <h3 class="cfg-dim-editor-title">
                    {move || dim().map(|d| if d.name.trim().is_empty() { "New Dimension".to_string() } else { d.name.clone() }).unwrap_or_default()}
                </h3>
                <button class="cfg-icon-btn" on:click=move |_| on_close.run(())>"✕"</button>
            </div>

            <div class="cfg-dim-editor-body">
                // ── Basic info ────────────────────────────────────────────
                <div class="cfg-editor-section">
                    <h4 class="cfg-editor-section-title">"Identity"</h4>

                    <div class="cfg-field">
                        <label class="cfg-label">"Name" <span class="cfg-required">"*"</span></label>
                        <input
                            type="text"
                            class="cfg-input"
                            placeholder="e.g. Company Revenue, Team Size, Geographic Fit"
                            prop:value=move || dim().map(|d| d.name).unwrap_or_default()
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                update_dim(&|d| d.name = v.clone());
                            }
                        />
                    </div>

                    <div class="cfg-field">
                        <label class="cfg-label">"Slug"</label>
                        <input
                            type="text"
                            class="cfg-input cfg-input--mono"
                            placeholder="auto-generated from name if blank"
                            prop:value=move || dim().map(|d| d.slug).unwrap_or_default()
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                update_dim(&|d| d.slug = v.clone());
                            }
                        />
                    </div>

                    <div class="cfg-field">
                        <label class="cfg-label">"Category"</label>
                        <input
                            type="text"
                            class="cfg-input"
                            placeholder="e.g. Financials, Operations, Fit"
                            prop:value=move || dim().map(|d| d.category).unwrap_or_default()
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                update_dim(&|d| d.category = v.clone());
                            }
                        />
                        <p class="cfg-hint">"Groups dimensions in the rater UI."</p>
                    </div>

                    <div class="cfg-field">
                        <label class="cfg-label">"Description"</label>
                        <textarea
                            class="cfg-textarea"
                            rows="2"
                            placeholder="What does this dimension measure? What does a high vs low score mean?"
                            prop:value=move || dim().map(|d| d.description).unwrap_or_default()
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                update_dim(&|d| d.description = v.clone());
                            }
                        />
                    </div>
                </div>

                // ── Scoring config ────────────────────────────────────────
                <div class="cfg-editor-section">
                    <h4 class="cfg-editor-section-title">"Scoring"</h4>

                    <div class="cfg-form-row">
                        <div class="cfg-field">
                            <label class="cfg-label">"Scale Type"</label>
                            <div class="cfg-select-wrap">
                                <select
                                    class="cfg-select"
                                    prop:value=move || dim().map(|d| d.scale_type.to_string()).unwrap_or_default()
                                    on:change=move |ev| {
                                        let v = event_target_value(&ev);
                                        if let Ok(st) = v.parse::<ScaleType>() {
                                            update_dim(&move |d| d.scale_type = st);
                                        }
                                    }
                                >
                                    <option value="rating">"Rating (numeric)"</option>
                                    <option value="absolute">"Absolute value"</option>
                                    // Backend enum: poll_single / poll_multi
                                    <option value="poll_single">"Poll (single choice)"</option>
                                    <option value="poll_multi">"Multi-select poll"</option>
                                    <option value="boolean">"Boolean (yes/no)"</option>
                                </select>
                            </div>
                        </div>

                        <div class="cfg-field">
                            <label class="cfg-label">"Weight"</label>
                            <input
                                type="number"
                                class="cfg-input"
                                step="0.1"
                                min="0"
                                prop:value=move || dim().map(|d| d.weight).unwrap_or(1.0)
                                on:input=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                        update_dim(&|d| d.weight = v);
                                    }
                                }
                            />
                            <p class="cfg-hint">"Relative weight in weighted_mean scoring."</p>
                        </div>
                    </div>

                    <div class="cfg-form-row">
                        <div class="cfg-field">
                            <label class="cfg-label">"Scale Min"</label>
                            <input
                                type="number"
                                class="cfg-input"
                                step="0.1"
                                prop:value=move || dim().map(|d| d.scale_min).unwrap_or(1.0)
                                on:input=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                        update_dim(&|d| d.scale_min = v);
                                    }
                                }
                            />
                        </div>
                        <div class="cfg-field">
                            <label class="cfg-label">"Scale Max"</label>
                            <input
                                type="number"
                                class="cfg-input"
                                step="0.1"
                                prop:value=move || dim().map(|d| d.scale_max).unwrap_or(10.0)
                                on:input=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse::<f64>() {
                                        update_dim(&|d| d.scale_max = v);
                                    }
                                }
                            />
                        </div>
                        <div class="cfg-field">
                            <label class="cfg-label">"Unit Label"</label>
                            <input
                                type="text"
                                class="cfg-input"
                                placeholder="Mbps, USD/mo, %..."
                                prop:value=move || dim().map(|d| d.unit_label).unwrap_or_default()
                                on:input=move |ev| {
                                    let v = event_target_value(&ev);
                                    update_dim(&|d| d.unit_label = v.clone());
                                }
                            />
                        </div>
                    </div>

                    // is_inverted toggle
                    <div class="cfg-toggle-row">
                        <div>
                            <p class="cfg-label">"Inverted Scale (lower = better)"</p>
                            <p class="cfg-hint">
                                "Use for dimensions where low score is a good outcome: "
                                "timeline slippage, competition risk, churn probability, etc. "
                                "The service normalizes and labels this correctly."
                            </p>
                        </div>
                        <button
                            class=move || if dim().map(|d| d.is_inverted).unwrap_or(false) { "cfg-toggle cfg-toggle--on" } else { "cfg-toggle" }
                            role="switch"
                            aria-checked=move || dim().map(|d| d.is_inverted).unwrap_or(false).to_string()
                            on:click=move |_| update_dim(&|d| d.is_inverted = !d.is_inverted)
                        >
                            <span class="cfg-toggle-thumb"></span>
                        </button>
                    </div>

                    <div class="cfg-toggle-row">
                        <div>
                            <p class="cfg-label">"Community Ratable"</p>
                            <p class="cfg-hint">"Allow any verified contributor to rate, not just the record owner."</p>
                        </div>
                        <button
                            class=move || if dim().map(|d| d.is_community_ratable).unwrap_or(true) { "cfg-toggle cfg-toggle--on" } else { "cfg-toggle" }
                            role="switch"
                            on:click=move |_| update_dim(&|d| d.is_community_ratable = !d.is_community_ratable)
                        >
                            <span class="cfg-toggle-thumb"></span>
                        </button>
                    </div>

                    <div class="cfg-toggle-row">
                        <div>
                            <p class="cfg-label">"Active"</p>
                            <p class="cfg-hint">"Inactive dimensions are hidden from raters and excluded from scoring."</p>
                        </div>
                        <button
                            class=move || if dim().map(|d| d.is_active).unwrap_or(true) { "cfg-toggle cfg-toggle--on" } else { "cfg-toggle" }
                            role="switch"
                            on:click=move |_| update_dim(&|d| d.is_active = !d.is_active)
                        >
                            <span class="cfg-toggle-thumb"></span>
                        </button>
                    </div>
                </div>

                // ── Options (poll / multiselect) ───────────────────────────
                <Show when=needs_options>
                    <div class="cfg-editor-section">
                        <div class="cfg-editor-section-header">
                            <h4 class="cfg-editor-section-title">"Options"</h4>
                            <button class="cfg-btn cfg-btn--ghost cfg-btn--sm" on:click=add_option>
                                "＋ Add Option"
                            </button>
                        </div>

                        <div class="cfg-options-list">
                            <For
                                each=move || dim().map(|d| d.options.clone()).unwrap_or_default()
                                key=|o| o.local_id
                                children=move |opt| {
                                    let opt_local = opt.local_id;
                                    view! {
                                        <div class="cfg-option-row">
                                            <div class="cfg-option-drag">"⋮⋮"</div>
                                            <input
                                                type="text"
                                                class="cfg-input cfg-input--sm"
                                                placeholder="Label"
                                                prop:value=opt.label.clone()
                                                on:input=move |ev| {
                                                    let v = event_target_value(&ev);
                                                    dimensions.update(|dims| {
                                                        if let Some(dim) = dims.iter_mut().find(|d| d.local_id == local_id) {
                                                            if let Some(o) = dim.options.iter_mut().find(|o| o.local_id == opt_local) {
                                                                o.label = v.clone();
                                                            }
                                                        }
                                                    });
                                                }
                                            />
                                            <input
                                                type="text"
                                                class="cfg-input cfg-input--sm cfg-input--mono"
                                                placeholder="value_key"
                                                prop:value=opt.value_key.clone()
                                                on:input=move |ev| {
                                                    let v = event_target_value(&ev);
                                                    dimensions.update(|dims| {
                                                        if let Some(dim) = dims.iter_mut().find(|d| d.local_id == local_id) {
                                                            if let Some(o) = dim.options.iter_mut().find(|o| o.local_id == opt_local) {
                                                                o.value_key = v.clone();
                                                            }
                                                        }
                                                    });
                                                }
                                            />
                                            <button
                                                class="cfg-icon-btn cfg-icon-btn--danger"
                                                on:click=move |_| {
                                                    dimensions.update(|dims| {
                                                        if let Some(dim) = dims.iter_mut().find(|d| d.local_id == local_id) {
                                                            dim.options.retain(|o| o.local_id != opt_local);
                                                        }
                                                    });
                                                }
                                            >"✕"</button>
                                        </div>
                                    }
                                }
                            />
                        </div>

                        <Show when=move || dim().map(|d| d.options.is_empty()).unwrap_or(true)>
                            <p class="cfg-hint">"No options yet. Click '＋ Add Option' to add poll choices."</p>
                        </Show>
                    </div>
                </Show>
            </div>
        </div>
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Section 3: Combinator Config — ideal ranges for The Combinator
// ═══════════════════════════════════════════════════════════════════════════

#[component]
fn CombinatorSection(
    dimensions: RwSignal<Vec<DimensionForm>>,
) -> impl IntoView {
    view! {
        <div class="cfg-section">
            <div class="cfg-section-header">
                <h2 class="cfg-section-title">"Combinator Configuration"</h2>
                <p class="cfg-section-desc">
                    "Define the ideal score, acceptable range, and search weight for each dimension. "
                    "The Combinator uses these to rank records by proximity to the target profile."
                </p>
            </div>

            <Show
                when=move || !dimensions.get().is_empty()
                fallback=|| view! {
                    <div class="cfg-empty-state">
                        <p class="cfg-empty-title">"No dimensions defined yet"</p>
                        <p class="cfg-empty-desc">"Add dimensions first, then configure ideal ranges here."</p>
                    </div>
                }
            >
                <div class="cfg-combinator-grid">
                    // Column headers
                    <div class="cfg-combinator-header">
                        <span>"Dimension"</span>
                        <span>"Ideal Score"</span>
                        <span>"Min Acceptable"</span>
                        <span>"Max Acceptable"</span>
                        <span>"Search Weight"</span>
                    </div>

                    <For
                        each=move || {
                            let mut dims = dimensions.get();
                            dims.sort_by_key(|d| d.sort_order);
                            dims
                        }
                        key=|d| d.local_id
                        children=move |dim| {
                            let local_id = dim.local_id;

                            let upd = move |f: &dyn Fn(&mut DimensionForm)| {
                                dimensions.update(|dims| {
                                    if let Some(d) = dims.iter_mut().find(|d| d.local_id == local_id) {
                                        f(d);
                                    }
                                });
                            };

                            view! {
                                <div class="cfg-combinator-row">
                                    <div class="cfg-combinator-dim-name">
                                        <span class=if dim.is_active { "cfg-dim-dot cfg-dim-dot--active" } else { "cfg-dim-dot cfg-dim-dot--inactive" }></span>
                                        <span>{if dim.name.trim().is_empty() { "Untitled".to_string() } else { dim.name.clone() }}</span>
                                        <span class="cfg-combinator-range-hint">
                                            {format!("({:.0}–{:.0})", dim.scale_min, dim.scale_max)}
                                        </span>
                                    </div>

                                    // Ideal score
                                    <input
                                        type="number"
                                        class="cfg-input cfg-input--sm"
                                        step="0.1"
                                        placeholder="—"
                                        prop:value=dim.ideal_score.map(|v| v.to_string()).unwrap_or_default()
                                        on:input=move |ev| {
                                            let v = event_target_value(&ev).parse::<f64>().ok();
                                            upd(&|d| d.ideal_score = v);
                                        }
                                    />

                                    // Range min
                                    <input
                                        type="number"
                                        class="cfg-input cfg-input--sm"
                                        step="0.1"
                                        placeholder="—"
                                        prop:value=dim.range_min.map(|v| v.to_string()).unwrap_or_default()
                                        on:input=move |ev| {
                                            let v = event_target_value(&ev).parse::<f64>().ok();
                                            upd(&|d| d.range_min = v);
                                        }
                                    />

                                    // Range max
                                    <input
                                        type="number"
                                        class="cfg-input cfg-input--sm"
                                        step="0.1"
                                        placeholder="—"
                                        prop:value=dim.range_max.map(|v| v.to_string()).unwrap_or_default()
                                        on:input=move |ev| {
                                            let v = event_target_value(&ev).parse::<f64>().ok();
                                            upd(&|d| d.range_max = v);
                                        }
                                    />

                                    // Search weight override
                                    <input
                                        type="number"
                                        class="cfg-input cfg-input--sm"
                                        step="0.1"
                                        min="0"
                                        placeholder=move || format!("{:.1} (dim default)", dim.weight)
                                        prop:value=dim.search_weight.map(|v| v.to_string()).unwrap_or_default()
                                        on:input=move |ev| {
                                            let raw = event_target_value(&ev);
                                            let v = if raw.trim().is_empty() { None } else { raw.parse::<f64>().ok() };
                                            upd(&|d| d.search_weight = v);
                                        }
                                    />
                                </div>
                            }
                        }
                    />

                    // Summary callout
                    <div class="cfg-combinator-note">
                        <span class="cfg-combinator-note-icon">"ⓘ"</span>
                        <p>
                            "The Combinator computes weighted Euclidean distance between a target vector and each scorecard's "
                            <code>"dimension_vector"</code>
                            ". Lower distance = higher match score. Leave Search Weight blank to use the dimension's default weight."
                        </p>
                    </div>
                </div>
            </Show>
        </div>
    }
}
