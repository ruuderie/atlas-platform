use leptos::prelude::*;
use shared_ui::components::ui::switch::Switch;
use crate::api::models::{FeatureFlagModel, FlagOverrideModel, FlagAuditLogModel};

// ── UI-only wrappers ─────────────────────────────────────────────────────────
// FlagState adds the reactive UI signals needed by the view (expand toggle,
// active tab, local copies of server state) to a real FeatureFlagModel.
// These are reset/re-derived whenever the server resource refetches.

#[derive(Clone)]
pub struct FlagState {
    pub key: String,
    pub description: String,
    pub is_enabled: RwSignal<bool>,
    pub has_global: bool,
    pub global_rollout: RwSignal<i32>,
    pub is_plan_gated: bool,
    pub plan_gate_tier: String,
    pub overrides: RwSignal<Vec<FlagOverrideModel>>,
    pub audit_logs: RwSignal<Vec<FlagAuditLogModel>>,
    pub jira: String,
    pub owner: String,
    pub date_created: String,
    pub expanded: RwSignal<bool>,
    pub active_tab: RwSignal<String>,
}

impl FlagState {
    fn from_model(m: &FeatureFlagModel) -> Self {
        FlagState {
            key: m.key.clone(),
            description: m.description.clone(),
            is_enabled: RwSignal::new(m.is_enabled),
            has_global: m.has_global,
            global_rollout: RwSignal::new(m.global_rollout_pct),
            is_plan_gated: m.is_plan_gated,
            plan_gate_tier: m.plan_gate_tier.clone().unwrap_or_default(),
            overrides: RwSignal::new(m.overrides.clone()),
            audit_logs: RwSignal::new(m.audit_logs.clone()),
            jira: m.jira.clone().unwrap_or_default(),
            owner: m.owner.clone(),
            date_created: m.created_at.as_deref().map(|s| s.chars().take(10).collect()).unwrap_or_default(),
            expanded: RwSignal::new(false),
            active_tab: RwSignal::new("variants".to_string()),
        }
    }
}

// ── Tenant-search ────────────────────────────────────────────────────────────
// Populated from GET /api/admin/tenant-stats. Colors are derived deterministically
// from the tenant name so they remain stable without a backend color field.
#[derive(Clone, Debug, PartialEq)]
pub struct TenantOption {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub plan: String,
    pub icon_char: char,
    pub bg_class: String,
    pub text_class: String,
}

impl TenantOption {
    /// Derive a stable color pair from the tenant name's first character.
    fn color_from_name(name: &str) -> (String, String) {
        let palettes = [
            ("bg-blue-500/10 border-blue-500/30",    "text-blue-400"),
            ("bg-amber-500/10 border-amber-500/30",  "text-amber-400"),
            ("bg-emerald-500/10 border-emerald-500/30", "text-emerald-400"),
            ("bg-violet-500/10 border-violet-500/30", "text-violet-400"),
            ("bg-rose-500/10 border-rose-500/30",    "text-rose-400"),
            ("bg-cyan-500/10 border-cyan-500/30",    "text-cyan-400"),
            ("bg-slate-500/10 border-slate-500/30",  "text-slate-400"),
        ];
        let idx = name.chars().next().map(|c| c as usize).unwrap_or(0) % palettes.len();
        (palettes[idx].0.to_string(), palettes[idx].1.to_string())
    }

    fn from_stat(stat: &crate::api::models::TenantStatModel) -> Self {
        let (bg_class, text_class) = Self::color_from_name(&stat.name);
        let icon_char = stat.name.chars().next().unwrap_or('?');
        Self {
            id: stat.tenant_id.clone(),
            slug: stat.slug.clone(),
            name: stat.name.clone(),
            plan: stat.plan.clone().unwrap_or_else(|| "—".to_string()),
            icon_char,
            bg_class,
            text_class,
        }
    }
}

#[component]
pub fn FeatureFlags() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // ── Server resource ───────────────────────────────────────────────────────
    let refresh = RwSignal::new(0u32);
    let flags_error: RwSignal<Option<String>> = RwSignal::new(None);
    let flags_resource = LocalResource::new(move || async move {
        let _ = refresh.get();
        let res = crate::api::admin::get_admin_flags().await;
        if let Err(ref e) = res { flags_error.set(Some(e.clone())); } else { flags_error.set(None); }
        res
    });

    // Derive stable FlagState list from server data.
    // We re-derive on every refetch so UI signals reflect current server values.
    let flags = Signal::derive(move || {
        flags_resource
            .get()
            .and_then(|r| r.ok())
            .map(|models| models.iter().map(FlagState::from_model).collect::<Vec<_>>())
            .unwrap_or_default()
    });

    // Live tenant list from the API — used by the NI override autocomplete.
    let tenants_resource = LocalResource::new(move || async move {
        let _ = refresh.get();
        crate::api::admin::get_tenant_stats().await
    });

    // Derive TenantOption list from server data so the autocomplete always
    // reflects the real tenant registry instead of a hardcoded stub.
    let tenant_options = Signal::derive(move || {
        tenants_resource
            .get()
            .and_then(|r| r.ok())
            .map(|stats| stats.iter().map(TenantOption::from_stat).collect::<Vec<_>>())
            .unwrap_or_default()
    });


    // Filtering & Searching UI State
    let search_query = RwSignal::new(String::new());
    let filter_scope = RwSignal::new("all".to_string()); // "all", "global", "plan", "ni", "off"

    // Side Drawer / Panel state for adding override
    let active_override_flag_key = RwSignal::new(None::<String>);
    let show_override_panel = RwSignal::new(false);

    // Forms state for NI override
    let override_tenant_input = RwSignal::new(String::new());
    let selected_tenant = RwSignal::new(None::<TenantOption>);
    let override_type = RwSignal::new("grant".to_string());
    let override_rollout_pct = RwSignal::new(100);
    let override_reason = RwSignal::new(String::new());
    let override_jira = RwSignal::new(String::new());

    // Autocomplete list state
    let autocomplete_open = RwSignal::new(false);

    // Modal state for New Flag Creation
    let show_new_flag_modal = RwSignal::new(false);
    let new_flag_key = RwSignal::new(String::new());
    let new_flag_desc = RwSignal::new(String::new());
    let new_flag_jira = RwSignal::new(String::new());
    let new_flag_has_global = RwSignal::new(true);

    // Edit Rollout Percentage inline dialog
    let show_rollout_modal = RwSignal::new(None::<(String, String, i32)>);
    let temp_rollout_val = RwSignal::new(100);

    // Filtered flag list Memo
    let filtered_flags = Signal::derive(move || {
        let query = search_query.get().to_lowercase();
        let scope = filter_scope.get();

        flags.get().into_iter().filter(|f| {
            // Text Search Match
            let matches_query = query.is_empty()
                || f.key.to_lowercase().contains(&query)
                || f.description.to_lowercase().contains(&query)
                || f.jira.to_lowercase().contains(&query)
                || f.overrides.get().iter().any(|o| o.tenant_id.to_lowercase().contains(&query));

            // Scope Filter Match
            let matches_scope = match scope.as_str() {
                "global" => f.has_global,
                "plan"   => f.is_plan_gated,
                "ni"     => !f.overrides.get().is_empty(),
                "off"    => !f.is_enabled.get() || (f.has_global && f.global_rollout.get() == 0),
                _        => true,
            };

            matches_query && matches_scope
        }).collect::<Vec<FlagState>>()
    });

    // Handle Open Assign Override Drawer
    let open_override_drawer = move |flag_key: String| {
        active_override_flag_key.set(Some(flag_key));
        override_tenant_input.set(String::new());
        selected_tenant.set(None);
        override_type.set("grant".to_string());
        override_rollout_pct.set(100);
        override_reason.set(String::new());
        override_jira.set(String::new());
        autocomplete_open.set(false);
        show_override_panel.set(true);
    };

    // Close Assign Override Drawer
    let close_override_drawer = move || {
        show_override_panel.set(false);
        active_override_flag_key.set(None);
    };

    // Save Override — calls real API
    let handle_save_override = move |_| {
        let fkey_opt = active_override_flag_key.get();
        let tenant_opt = selected_tenant.get();
        let reason = override_reason.get();

        if fkey_opt.is_none() { return; }
        let fkey = fkey_opt.unwrap();

        if tenant_opt.is_none() {
            toast.show_toast("Error", "A valid tenant selection is required.", "error");
            return;
        }
        let tenant = tenant_opt.unwrap();

        if reason.trim().is_empty() {
            toast.show_toast("Error", "Update reason is required for audit logs.", "error");
            return;
        }

        let ovr_type = override_type.get();
        let rollout = if ovr_type == "deny" { 0 } else { override_rollout_pct.get() };
        let jira_val = if override_jira.get().trim().is_empty() { None } else { Some(override_jira.get()) };

        let input = crate::api::models::CreateFlagOverrideInput {
            tenant_id: tenant.id.clone(),
            override_type: ovr_type.clone(),
            rollout_pct: Some(rollout),
            reason: reason.clone(),
            jira: jira_val,
        };

        let fk2 = fkey.clone();
        let tenant_slug = tenant.slug.clone();
        let resource = flags_resource.clone();
        leptos::task::spawn_local(async move {
            match crate::api::admin::add_flag_override(fk2, input).await {
                Ok(_) => {
                    resource.refetch();
                    close_override_drawer();
                    toast.show_toast("Success", &format!("Tenant override saved for {}.", tenant_slug), "success");
                }
                Err(e) => toast.show_toast("Error", &e, "error"),
            }
        });
    };

    // Remove Override — calls real API
    let handle_remove_override = move |flag_key: String, tenant_id: String| {
        let resource = flags_resource.clone();
        leptos::task::spawn_local(async move {
            match crate::api::admin::remove_flag_override(flag_key, tenant_id.clone()).await {
                Ok(_) => {
                    resource.refetch();
                    toast.show_toast("Warning", &format!("Override removed for {}.", tenant_id), "warn");
                }
                Err(e) => toast.show_toast("Error", &e, "error"),
            }
        });
    };

    // Handle Global Enable/Disable Toggle — calls real API
    let handle_global_toggle = move |flag_key: String, checked: bool| {
        let resource = flags_resource.clone();
        leptos::task::spawn_local(async move {
            let input = crate::api::models::UpdateFlagInput {
                is_enabled: Some(checked),
                ..Default::default()
            };
            match crate::api::admin::update_flag(flag_key.clone(), input).await {
                Ok(_) => {
                    resource.refetch();
                    toast.show_toast("Info", &format!("Global switch updated for {}.", flag_key), "info");
                }
                Err(e) => toast.show_toast("Error", &e, "error"),
            }
        });
    };

    // New flag submit handler — calls real API
    let handle_create_flag = move |_| {
        let key = new_flag_key.get();
        let desc = new_flag_desc.get();
        let jira = new_flag_jira.get();

        if key.trim().is_empty() || desc.trim().is_empty() {
            toast.show_toast("Error", "Flag key and description are required.", "error");
            return;
        }

        let jira_opt = if jira.trim().is_empty() { None } else { Some(jira.trim().to_uppercase()) };
        let input = crate::api::models::CreateFlagInput {
            key: key.trim().to_lowercase().replace(' ', "_"),
            description: desc,
            has_global: Some(new_flag_has_global.get()),
            global_rollout_pct: Some(if new_flag_has_global.get() { 100 } else { 0 }),
            jira: jira_opt,
        };

        let resource = flags_resource.clone();
        leptos::task::spawn_local(async move {
            match crate::api::admin::create_flag(input.clone()).await {
                Ok(_) => {
                    resource.refetch();
                    show_new_flag_modal.set(false);
                    new_flag_key.set(String::new());
                    new_flag_desc.set(String::new());
                    new_flag_jira.set(String::new());
                    toast.show_toast("Success", &format!("Feature flag {} created.", input.key), "success");
                }
                Err(e) => toast.show_toast("Error", &e, "error"),
            }
        });
    };

    // Rollout save handler — calls real API
    let handle_save_rollout = move |_| {
        if let Some((flag_key, _scope, _)) = show_rollout_modal.get() {
            let new_pct = temp_rollout_val.get();
            let input = crate::api::models::UpdateFlagInput {
                global_rollout_pct: Some(new_pct),
                ..Default::default()
            };
            let resource = flags_resource.clone();
            leptos::task::spawn_local(async move {
                match crate::api::admin::update_flag(flag_key.clone(), input).await {
                    Ok(_) => {
                        resource.refetch();
                        show_rollout_modal.set(None);
                        toast.show_toast("Success", &format!("Rollout updated to {}% for {}.", new_pct, flag_key), "success");
                    }
                    Err(e) => toast.show_toast("Error", &e, "error"),
                }
            });
        }
    };



    view! {
        <div class="main-area">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <div class="page-title">"Feature Flags"</div>
                    <div class="page-subtitle">"Flag registry · Each flag may have a Global variant, Plan gate, and per-NI overrides — all managed here"</div>
                </div>
                <div class="page-actions">
                    <button
                        on:click=move |_| refresh.update(|n| *n += 1)
                        class="btn btn-ghost btn-sm"
                        title="Reload flags from backend"
                    >
                        <svg class="w-3 h-3 inline-block mr-1" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8">
                            <path d="M13.5 8A5.5 5.5 0 1 1 8 2.5M13.5 2.5v3h-3"/>
                        </svg>
                        "Refresh"
                    </button>
                    <button 
                        on:click=move |_| toast.show_toast("Info", "Exporting flag audit logs to CSV...", "info")
                        class="btn btn-ghost btn-sm"
                    >
                        "↓ Export"
                    </button>
                    <button 
                        on:click=move |_| {
                            new_flag_key.set(String::new());
                            new_flag_desc.set(String::new());
                            new_flag_jira.set(String::new());
                            new_flag_has_global.set(true);
                            show_new_flag_modal.set(true);
                        }
                        class="btn btn-primary"
                    >
                        <svg width="11" height="11" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2.5" style="margin-right:4px; display:inline-block; vertical-align:middle;">
                            <line x1="8" y1="2" x2="8" y2="14"/>
                            <line x1="2" y1="8" x2="14" y2="8"/>
                        </svg>
                        "New Flag"
                    </button>
                </div>
            </div>

            // ── Error banner ──
            {move || flags_error.get().map(|e| crate::utils::inline_error(&e))}

            // ── Stat Strip ──
            <div class="stat-strip">
                <div class="stat">
                    <span class="stat-val">{move || flags.get().len().to_string()}</span>
                    <span class="stat-lbl">"Total Flags"</span>
                </div>
                <div class="stat">
                    <span class="stat-val" style="color:var(--green)">
                        {move || flags.get().iter().filter(|f| f.is_enabled.get() && f.has_global && f.global_rollout.get() == 100).count().to_string()}
                    </span>
                    <span class="stat-lbl">"Globally On"</span>
                </div>
                <div class="stat">
                    <span class="stat-val" style="color:var(--text-muted)">
                        {move || flags.get().iter().filter(|f| !f.is_enabled.get() || (f.has_global && f.global_rollout.get() == 0)).count().to_string()}
                    </span>
                    <span class="stat-lbl">"Dark / Off"</span>
                </div>
                <div class="stat">
                    <span class="stat-val" style="color:var(--violet)">
                        {move || flags.get().iter().filter(|f| f.is_plan_gated).count().to_string()}
                    </span>
                    <span class="stat-lbl">"Plan-Gated"</span>
                </div>
                <div class="stat">
                    <span class="stat-val" style="color:var(--green)">
                        {move || flags.get().iter().map(|f| f.overrides.get().len()).sum::<usize>().to_string()}
                    </span>
                    <span class="stat-lbl">"NI Overrides"</span>
                </div>
                <div class="stat">
                    <span class="stat-val" style="color:var(--amber)">
                        {move || flags.get().iter().filter(|f| f.is_enabled.get() && f.has_global && f.global_rollout.get() > 0 && f.global_rollout.get() < 100).count().to_string()}
                    </span>
                    <span class="stat-lbl">"Canary <100%"</span>
                </div>
            </div>

            // ── Filter Bar ──
            <div class="filter-bar">
                <div class="flag-search-wrap">
                    <span class="si">
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                            <circle cx="6.5" cy="6.5" r="4"/>
                            <line x1="10" y1="10" x2="14" y2="14"/>
                        </svg>
                    </span>
                    <input 
                        type="text" 
                        placeholder="Search flag key, Jira, NI slug…" 
                        prop:value=search_query
                        on:input=move |ev| search_query.set(event_target_value(&ev))
                    />
                </div>
                {
                    let filter_pill = move |scope: &'static str, label: &'static str, active_class_name: &'static str| {
                        view! {
                            <button 
                                on:click=move |_| filter_scope.set(scope.to_string())
                                class=move || format!(
                                    "fpill {}",
                                    if filter_scope.get() == scope {
                                        active_class_name
                                    } else {
                                        ""
                                    }
                                )
                            >
                                <Show when=move || scope != "all">
                                    <span style=move || format!(
                                        "width:6px;height:6px;border-radius:50%;background:{};display:inline-block;margin-right:4px;",
                                        match scope {
                                            "global" => "var(--cobalt)",
                                            "plan" => "var(--violet)",
                                            "ni" => "var(--green)",
                                            _ => "var(--amber)"
                                        }
                                    )></span>
                                </Show>
                                {label}
                            </button>
                        }
                    };
                    view! {
                        {filter_pill("all", "All Flags", "a-all")}
                        {filter_pill("global", "Has Global", "a-global")}
                        {filter_pill("plan", "Plan-Gated", "a-plan")}
                        {filter_pill("ni", "Has NI Overrides", "a-ni")}
                        {filter_pill("off", "Off / Dark", "a-off")}
                    }
                }
            </div>

            // ── Flags List Container ──
            <div class="flags-body">
                <For 
                    each=move || filtered_flags.get()
                    key=|f| f.key.clone()
                    children=move |f| {
                        let f_val = StoredValue::new(f);
                        let fkey = StoredValue::new(f_val.with_value(|v| v.key.clone()));
                        
                        let has_global = f_val.with_value(|v| v.has_global);
                        let is_plan_gated = f_val.with_value(|v| v.is_plan_gated);
                        let global_rollout = f_val.with_value(|v| v.global_rollout);
                        let plan_gate_tier = StoredValue::new(f_val.with_value(|v| v.plan_gate_tier.clone()));
                        let expanded = f_val.with_value(|v| v.expanded);
                        let active_tab = f_val.with_value(|v| v.active_tab);
                        let is_enabled = f_val.with_value(|v| v.is_enabled);
                        let overrides = f_val.with_value(|v| v.overrides);
                        let audit_logs = f_val.with_value(|v| v.audit_logs);
                        let jira = StoredValue::new(f_val.with_value(|v| v.jira.clone()));
                        let description = StoredValue::new(f_val.with_value(|v| v.description.clone()));
                        let owner = StoredValue::new(f_val.with_value(|v| v.owner.clone()));

                        let has_overrides = Signal::derive(move || !overrides.get().is_empty());
                        let count_overrides = Signal::derive(move || overrides.get().len());
                        let is_expanded = expanded;

                        view! {
                            <div class=move || format!(
                                "flag-card {}",
                                if is_expanded.get() { "expanded" } else { "" }
                            )>
                                // Card Header
                                <div 
                                    on:click=move |_| is_expanded.update(|v| *v = !*v)
                                    class="flag-card-hdr"
                                >
                                    <div class="flag-toggle" on:click=move |e| e.stop_propagation()>
                                        <Switch 
                                            variant="compact"
                                            checked=is_enabled
                                            on_checked_change=Callback::new({
                                                let fk = fkey.clone();
                                                move |next_val| handle_global_toggle(fk.get_value(), next_val)
                                            })
                                        />
                                    </div>

                                    <div class="flag-identity">
                                        <div class="flag-key">{fkey.get_value()}</div>
                                        <div class="flag-desc-text">{description.get_value()}</div>
                                        <div class="flag-meta-row">
                                            <Show when=move || has_global>
                                                <span class="scope-chip sc-global">"Global · " {move || global_rollout.get().to_string()} "%"</span>
                                            </Show>
                                            <Show when=move || is_plan_gated>
                                                <span class="scope-chip sc-plan">"Plan · " {plan_gate_tier.get_value()}</span>
                                            </Show>
                                            <Show when=move || has_overrides.get()>
                                                <span class="scope-chip sc-ni">{move || count_overrides.get().to_string()} " NI overrides"</span>
                                            </Show>
                                            <Show when=move || !has_global && !is_plan_gated && !has_overrides.get()>
                                                <span class="scope-chip sc-off">"No Global"</span>
                                            </Show>
                                            <Show when={
                                                let jr = jira.clone();
                                                move || {
                                                    let j = jr.get_value();
                                                    !j.is_empty() && j != "None"
                                                }
                                            }>
                                                <a class="jira-link" href="#" on:click=move |e| { e.stop_propagation(); e.prevent_default(); }>{jira.get_value()}</a>
                                            </Show>
                                            <span style="font-size:10.5px;color:var(--text-muted);">{owner.get_value()} " · " {f_val.with_value(|v| v.date_created.clone())}</span>
                                        </div>
                                    </div>

                                    <div class="flag-card-right">
                                        <div class="effective-badge">
                                            {move || {
                                                let total_nis = 24;
                                                if !is_enabled.get() {
                                                    view! { <span style="color:var(--amber)">"⚠ Dark launch"</span> }.into_any()
                                                } else if !has_global && has_overrides.get() {
                                                    view! { <strong>{count_overrides.get().to_string()}</strong> "/" {total_nis.to_string()} " NIs active" }.into_any()
                                                } else if is_plan_gated {
                                                    view! { <strong>"9"</strong> "/" {total_nis.to_string()} " NIs active" }.into_any()
                                                } else {
                                                    let pct = global_rollout.get();
                                                    if pct == 100 {
                                                        view! { <strong>{total_nis.to_string()}</strong> "/" {total_nis.to_string()} " NIs active" }.into_any()
                                                    } else if pct == 0 {
                                                        view! { <span style="color:var(--amber)">"⚠ Dark launch"</span> }.into_any()
                                                    } else {
                                                        let active_est = (total_nis as f32 * (pct as f32 / 100.0)).round() as i32;
                                                        view! { <strong>"~" {active_est.to_string()}</strong> "/" {total_nis.to_string()} " NIs active" }.into_any()
                                                    }
                                                }
                                            }}
                                        </div>

                                        <button 
                                            class="btn btn-ghost btn-sm" 
                                            on:click={
                                                let fk = fkey.clone();
                                                move |e| { e.stop_propagation(); open_override_drawer(fk.get_value()); }
                                            }
                                        >
                                            "+ Assign NI Override"
                                        </button>
                                        <span class="expand-caret">"▾"</span>
                                    </div>
                                </div>

                                // Expanded details body
                                <div class="flag-card-body">
                                    <div class="flag-card-tabs">
                                        <button 
                                            class=move || format!("fct {}", if active_tab.get() == "variants" { "active" } else { "" })
                                            on:click={
                                                let tab_sig = active_tab;
                                                move |e| { e.stop_propagation(); tab_sig.set("variants".to_string()); }
                                            }
                                        >
                                            "Variants & Rollout"
                                        </button>
                                        <button 
                                            class=move || format!("fct {}", if active_tab.get() == "overrides" { "active" } else { "" })
                                            on:click={
                                                let tab_sig = active_tab;
                                                move |e| { e.stop_propagation(); tab_sig.set("overrides".to_string()); }
                                            }
                                        >
                                            "NI Overrides "
                                            <span style=move || format!("background:{};border:1px solid {};border-radius:8px;padding:0 5px;font-size:9px;color:{}", 
                                                if count_overrides.get() > 0 { "var(--green-dim)" } else { "var(--bg-elevated)" },
                                                if count_overrides.get() > 0 { "var(--green)" } else { "var(--border-default)" },
                                                if count_overrides.get() > 0 { "var(--green)" } else { "var(--text-muted)" }
                                            )>
                                                {move || count_overrides.get().to_string()}
                                            </span>
                                        </button>
                                        <button 
                                            class=move || format!("fct {}", if active_tab.get() == "audit" { "active" } else { "" })
                                            on:click={
                                                let tab_sig = active_tab;
                                                move |e| { e.stop_propagation(); tab_sig.set("audit".to_string()); }
                                            }
                                        >
                                            "Audit Log"
                                        </button>
                                    </div>

                                    // Tab Pane: Variants & Rollout
                                    <Show when=move || active_tab.get() == "variants">
                                        <div class="fct-pane active" on:click=move |e| e.stop_propagation()>
                                            <table class="variant-table">
                                                <thead>
                                                    <tr>
                                                        <th>"Scope"</th>
                                                        <th>"State"</th>
                                                        <th>"Rollout"</th>
                                                        <th>"Applies To"</th>
                                                        <th>"Changed"</th>
                                                        <th>""</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <Show when=move || has_global fallback=move || view! {
                                                        <tr>
                                                            <td><span class="scope-chip sc-off">"Global"</span></td>
                                                            <td style="color:var(--text-muted);font-style:italic;">"Not configured — NI override only"</td>
                                                            <td>"—"</td>
                                                            <td style="color:var(--text-secondary);font-size:11.5px;">"Only tenants with an explicit NI grant see this feature"</td>
                                                            <td>"—"</td>
                                                            <td>""</td>
                                                        </tr>
                                                    }>
                                                        <tr>
                                                            <td><span class="scope-chip sc-global">"Global"</span></td>
                                                            <td>
                                                                {move || {
                                                                    if !is_enabled.get() || global_rollout.get() == 0 {
                                                                        view! { <span style="color:var(--amber);font-weight:600;">"● Dark (0%)"</span> }.into_any()
                                                                    } else if global_rollout.get() == 100 {
                                                                        view! { <span style="color:var(--green);font-weight:600;">"● Enabled"</span> }.into_any()
                                                                    } else {
                                                                        view! { <span style="color:var(--cobalt);font-weight:600;">"⏳ Canary"</span> }.into_any()
                                                                    }
                                                                }}
                                                            </td>
                                                            <td>
                                                                <div class="rb-inline">
                                                                    <div class="rb">
                                                                        <div class="rf" style=move || format!("width: {}%; background: {}", 
                                                                            global_rollout.get(),
                                                                            if global_rollout.get() == 100 { "var(--green)" } else { "var(--cobalt)" }
                                                                        )></div>
                                                                    </div>
                                                                    <span class="rl">{move || global_rollout.get().to_string()} "%"</span>
                                                                </div>
                                                            </td>
                                                            <td style="color:var(--text-secondary);font-size:11.5px;">
                                                                {move || {
                                                                    let pct = global_rollout.get();
                                                                    if pct == 100 {
                                                                        "All 24 NIs".to_string()
                                                                    } else if pct == 0 {
                                                                        "Awaiting legal + QA".to_string()
                                                                    } else {
                                                                        format!("~{} of 24 NIs", (24.0f32 * (pct as f32 / 100.0f32)).round() as i32)
                                                                    }
                                                                }}
                                                            </td>
                                                            <td style="font-size:11px;color:var(--text-muted);">{f_val.with_value(|v| v.date_created.clone())} " · " {owner.get_value()}</td>
                                                            <td>
                                                                <button 
                                                                    class="btn btn-ghost btn-sm"
                                                                    on:click={
                                                                        let fk = fkey.clone();
                                                                        let cur = global_rollout;
                                                                        move |_| {
                                                                            temp_rollout_val.set(cur.get());
                                                                            show_rollout_modal.set(Some((fk.get_value(), "Global".to_string(), cur.get())));
                                                                        }
                                                                    }
                                                                >
                                                                    "Edit %"
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    </Show>
                                                    <Show when=move || is_plan_gated>
                                                        <tr>
                                                            <td><span class="scope-chip sc-plan">"Plan Gate"</span></td>
                                                            <td><span style="color:var(--violet);font-weight:600;">"● " {plan_gate_tier.get_value()}</span></td>
                                                            <td>
                                                                <div class="rb-inline">
                                                                    <div class="rb">
                                                                        <div class="rf" style="width: 100%; background: var(--violet);"></div>
                                                                    </div>
                                                                    <span class="rl">"9 NIs"</span>
                                                                </div>
                                                            </td>
                                                            <td style="color:var(--text-secondary);font-size:11.5px;">{plan_gate_tier.get_value()} " tier only"</td>
                                                            <td style="font-size:11px;color:var(--text-muted);">{f_val.with_value(|v| v.date_created.clone())} " · " {owner.get_value()}</td>
                                                            <td>
                                                                <button 
                                                                    class="btn btn-ghost btn-sm"
                                                                    on:click=move |_| toast.show_toast("Info", "Plan gate definitions must be modified in Billing rules tier mapping.", "info")
                                                                >
                                                                    "Edit Gate"
                                                                </button>
                                                            </td>
                                                        </tr>
                                                    </Show>
                                                </tbody>
                                            </table>
                                        </div>
                                    </Show>

                                    // Tab Pane: NI Overrides
                                    <Show when=move || active_tab.get() == "overrides">
                                        <div class="fct-pane active" on:click=move |e| e.stop_propagation()>
                                            <div style="padding:10px 0;">
                                                <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:10px;">
                                                    <div style="font-size:10px;font-weight:600;text-transform:uppercase;letter-spacing:.08em;color:var(--text-muted);">"Tenant-Specific Overrides"</div>
                                                    <button 
                                                        class="btn btn-ghost btn-sm"
                                                        on:click={
                                                            let fk = fkey.clone();
                                                            move |_| open_override_drawer(fk.get_value())
                                                        }
                                                    >
                                                        "+ Add Override"
                                                    </button>
                                                </div>

                                                <Show when=move || !overrides.get().is_empty() fallback=move || view! {
                                                    <div style="padding:20px;text-align:center;color:var(--text-muted);font-size:12px;">
                                                        "No NI overrides. Use \"+ Assign NI Override\" to grant or deny access for a specific tenant."
                                                    </div>
                                                }>
                                                    <For 
                                                        each=move || overrides.get()
                                                        key=|o| o.tenant_id.clone()
                                                        children={
                                                            let fk = fkey.clone();
                                                            move |o| {
                                                                let tenant_id = o.tenant_id.clone();
                                                                let o_type = o.override_type.clone();
                                                                let o_type_for_class = o_type.clone();
                                                                let o_type_for_style = o_type.clone();
                                                                 let o_type_badge = o_type.clone();
                                                                 let o_type_source = o_type.clone();
                                                                 let o_created_at_str = o.created_at.clone().map(|s| s.chars().take(10).collect::<String>()).unwrap_or_default();
                                                                let fk_inner = fk.clone();
                                                                view! {
                                                                    <div class="ni-override-row">
                                                                        <div style="flex:1;">
                                                                            <div style="display:flex;align-items:center;gap:8px;">
                                                                                <span class="ni-slug">{o.tenant_id.clone()}</span>
                                                                                <span class=move || format!("ni-plan-badge {}", o_type_badge.to_lowercase())>{o_type.clone()}</span>
                                                                            </div>
                                                                            <div class="ni-source">
                                                                                {if o_type_source == "deny" { "Deny" } else { "Grant" }}
                                                                                " · " {o.rollout_pct.to_string()} "% within NI · "
                                                                                {o.reason.clone()} " · " {o_created_at_str} " · " {o.changed_by.clone()}
                                                                                {o.jira.as_deref().map(|j| format!(" · {}", j)).unwrap_or_default()}
                                                                            </div>
                                                                        </div>
                                                                        <div class=move || format!("override-state {}", if o_type_for_class == "deny" { "ni-deny" } else { "ni-grant" })>
                                                                            <span class="override-dot" style=move || format!("background:{}", if o_type_for_style == "deny" { "var(--red)" } else { "var(--green)" })></span>
                                                                            {if o_type == "deny" { "Blocked" } else { "100% active" }}
                                                                        </div>
                                                                        <button 
                                                                            class="btn btn-ghost btn-sm btn-icon" 
                                                                            title="Remove override"
                                                                            on:click=move |_| handle_remove_override(fk_inner.get_value(), tenant_id.clone())
                                                                        >
                                                                            "✕"
                                                                        </button>
                                                                    </div>
                                                                }
                                                            }
                                                        }
                                                    />
                                                </Show>
                                            </div>
                                        </div>
                                    </Show>

                                    // Tab Pane: Audit Log
                                    <Show when=move || active_tab.get() == "audit">
                                        <div class="fct-pane active" on:click=move |e| e.stop_propagation() style="max-height: 250px; overflow-y: auto;">
                                            <For 
                                                each=move || audit_logs.get()
                                                key=|log| format!("{}-{}-{}", log.created_at.as_deref().unwrap_or(""), log.user_id, log.action)
                                                children=move |log| {
                                                    view! {
                                                        <div style="font-size:11.5px;color:var(--text-secondary);padding:8px 0;border-bottom:1px solid var(--border-subtle);display:flex;gap:8px;">
                                                            <strong>{log.created_at.as_deref().map(|s| s.chars().take(10).collect::<String>()).unwrap_or_default()}</strong>
                                                            <span style="color:var(--text-muted)">"·"</span>
                                                            <span style="color:var(--cobalt);font-weight:600;min-width:60px;">{log.user_id.clone()}</span>
                                                            <span style="color:var(--text-muted)">"—"</span>
                                                            <span>{log.action.clone()}</span>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>
                                    </Show>
                                </div>
                            </div>
                        }
                    }
                />
            </div>

            // ── Assign NI Override Side Panel ──
            <div 
                on:click=move |_| close_override_drawer()
                class=move || format!(
                    "assign-panel-backdrop {}",
                    if show_override_panel.get() { "open" } else { "" }
                )
            ></div>
            
            <div class=move || format!(
                "assign-panel {}",
                if show_override_panel.get() { "open" } else { "" }
            )>
                <div class="assign-panel-hdr">
                    <div>
                        <div class="assign-panel-title">"Assign NI Override"</div>
                        <div style="font-size:11px;color:var(--text-muted);font-family:monospace;margin-top:2px;">
                            {move || active_override_flag_key.get().unwrap_or_default()}
                        </div>
                    </div>
                    <button 
                        on:click=move |_| close_override_drawer()
                        class="btn btn-ghost btn-sm"
                    >
                        "✕ Close"
                    </button>
                </div>

                <div class="assign-panel-body">
                    <div class="arch-note">
                        <strong>"NI Override"</strong> " grants or denies access for a single tenant, bypassing the plan gate."
                        "A " <strong>"grant"</strong> " lets an NI access a feature above their plan tier (e.g. pilot program)."
                        "A " <strong>"deny"</strong> " blocks a specific NI below their plan tier (e.g. compliance hold)."
                        "All changes are audit-logged."
                    </div>

                    // Tenant Search Form
                    <div class="n-form-row">
                        <label class="n-form-label">"Tenant (NI slug or name)"</label>
                        <Show when=move || selected_tenant.get().is_some() fallback=move || view! {
                            <div class="tenant-search-wrap">
                                <input 
                                    type="text" 
                                    placeholder="Search nexus-property, miami-stays…" 
                                    prop:value=override_tenant_input
                                    on:focus=move |_| autocomplete_open.set(true)
                                    on:input=move |ev| {
                                        let val = event_target_value(&ev);
                                        override_tenant_input.set(val);
                                        autocomplete_open.set(true);
                                    }
                                    autocomplete="off"
                                />
                                <Show when=move || autocomplete_open.get()>
                                    <div class="tenant-dropdown open">
                                        {
                                            let term = override_tenant_input.get().to_lowercase();
                                            let filtered_opts: Vec<TenantOption> = tenant_options.get().into_iter().filter(|t| {
                                                term.is_empty() || t.name.to_lowercase().contains(&term) || t.id.contains(&term)
                                            }).collect();

                                            if filtered_opts.is_empty() {
                                                view! {
                                                    <div class="px-4 py-3 text-xs text-on-surface-variant text-center">"No matching tenants found"</div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <For 
                                                        each=move || filtered_opts.clone()
                                                        key=|t| t.slug.clone()
                                                        children=move |t| {
                                                            let t_select = t.clone();
                                                            view! {
                                                                <div 
                                                                    on:click=move |_| {
                                                                        selected_tenant.set(Some(t_select.clone()));
                                                                        autocomplete_open.set(false);
                                                                    }
                                                                    class="tenant-opt"
                                                                >
                                                                    <div class=format!("w-7 h-7 rounded flex items-center justify-center font-bold text-xs border {}", t.bg_class)>
                                                                        {t.icon_char.to_string()}
                                                                    </div>
                                                                    <div>
                                                                        <div class="tenant-opt-slug">{t.slug.clone()}</div>
                                                                        <div class="tenant-opt-meta">{t.name.clone()} " · " <span class="tenant-opt-plan">{t.plan.clone()}</span></div>
                                                                    </div>
                                                                </div>
                                                            }
                                                        }
                                                    />
                                                }.into_any()
                                            }
                                        }
                                    </div>
                                </Show>
                            </div>
                        }>
                            {let st = selected_tenant.get().unwrap();
                             view! {
                                <div class="selected-tenant-chip">
                                    <div class=format!("w-6 h-6 rounded flex items-center justify-center font-bold border text-[10px] {}", st.bg_class)>
                                        {st.icon_char.to_string()}
                                    </div>
                                    <div style="flex:1;">
                                        <div class="slug">{st.slug.clone()}</div>
                                        <div style="font-size:10.5px;color:var(--text-muted);">{st.name.clone()} " · " {st.plan.clone()}</div>
                                    </div>
                                    <button 
                                        on:click=move |_| selected_tenant.set(None)
                                        class="btn btn-ghost btn-sm btn-icon" 
                                        title="Change tenant"
                                    >
                                        "✕"
                                    </button>
                                </div>
                            }}
                        </Show>
                    </div>

                    // Override Type Selection
                    <div class="n-form-row">
                        <label class="n-form-label">"Override Type"</label>
                        <select 
                            class="n-form-select"
                            on:change=move |ev| override_type.set(event_target_value(&ev))
                        >
                            <option value="grant">"Grant — enable for this tenant (above plan tier)"</option>
                            <option value="deny">"Deny — block for this tenant (compliance / hold)"</option>
                        </select>
                        <div class="n-form-hint">
                            {move || if override_type.get() == "grant" {
                                "This tenant will gain access to this feature regardless of their plan tier."
                            } else {
                                "This tenant will be blocked even if their plan tier qualifies."
                            }}
                        </div>
                    </div>

                    // Rollout % (only active for grant)
                    <Show when=move || override_type.get() == "grant">
                        <div class="n-form-row" id="rollout-row">
                            <label class="n-form-label">"Rollout % within this tenant (0–100)"</label>
                            <input 
                                type="number" 
                                min="0" max="100"
                                class="n-form-input"
                                prop:value=override_rollout_pct
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                        override_rollout_pct.set(val.clamp(0, 100));
                                    }
                                }
                            />
                            <div class="n-form-hint">"100 = all users in this NI see it. Lower values = canary within the tenant."</div>
                        </div>
                    </Show>

                    // Reason input
                    <div class="n-form-row">
                        <label class="n-form-label">"Reason " <span style="color:var(--red)">"*"</span></label>
                        <input 
                            type="text"
                            class="n-form-input"
                            placeholder="e.g. Pilot per AM request · ATLAS-3200"
                            prop:value=override_reason
                            on:input=move |ev| override_reason.set(event_target_value(&ev))
                        />
                        <div class="n-form-hint">"Required. Attached to audit log entry."</div>
                    </div>

                    // Optional Jira ticket
                    <div class="n-form-row">
                        <label class="n-form-label">"Jira Ticket (if different from flag's primary ticket)"</label>
                        <input 
                            type="text"
                            class="n-form-input font-mono"
                            placeholder="ATLAS-XXXX"
                            prop:value=override_jira
                            on:input=move |ev| override_jira.set(event_target_value(&ev))
                        />
                    </div>

                    // Effective access cascade preview
                    <div style="background:var(--bg-base);border:1px solid var(--border-default);border-radius:5px;padding:12px 14px;margin-top:4px;">
                        <div style="font-size:10px;font-weight:600;text-transform:uppercase;letter-spacing:.08em;color:var(--text-muted);margin-bottom:10px;">
                            "Effective Access After Override"
                        </div>
                        <div style="display:flex;flex-direction:column;gap:6px;">
                            <div style="display:flex;align-items:center;gap:8px;font-size:11.5px;">
                                <span style="width:70px;color:var(--text-muted);font-size:10px;text-transform:uppercase;letter-spacing:.06em;font-weight:600;">"Global"</span>
                                {move || {
                                    let is_active = flags.get().iter().find(|flg| Some(flg.key.clone()) == active_override_flag_key.get()).map(|flg| flg.is_enabled.get()).unwrap_or(false);
                                    if is_active {
                                        view! {
                                            <>
                                                <span style="color:var(--green);font-weight:600;">"✓ ON"</span>
                                                <span style="font-size:10.5px;color:var(--text-muted);">" — platform-wide enabled"</span>
                                            </>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <>
                                                <span style="color:var(--amber);font-weight:600;">"● OFF"</span>
                                                <span style="font-size:10.5px;color:var(--text-muted);">" — platform-wide dark launched"</span>
                                            </>
                                        }.into_any()
                                    }
                                }}
                            </div>
                            <div style="display:flex;align-items:center;gap:8px;font-size:11.5px;">
                                <span style="width:70px;color:var(--text-muted);font-size:10px;text-transform:uppercase;letter-spacing:.06em;font-weight:600;">"Plan Gate"</span>
                                {move || {
                                    let has_gate = flags.get().iter().find(|flg| Some(flg.key.clone()) == active_override_flag_key.get()).map(|flg| flg.is_plan_gated).unwrap_or(false);
                                    if has_gate {
                                        view! {
                                            <>
                                                <span style="color:var(--amber);font-weight:600;">"⚠ Blocked"</span>
                                                <span style="font-size:10.5px;color:var(--text-muted);">" — tenant's plan may not qualify"</span>
                                            </>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <>
                                                <span style="color:var(--text-secondary);font-weight:600;">"None"</span>
                                                <span style="font-size:10.5px;color:var(--text-muted);">" — no plan gate configured"</span>
                                            </>
                                        }.into_any()
                                    }
                                }}
                            </div>
                            <div style="display:flex;align-items:center;gap:8px;font-size:11.5px;">
                                <span style="width:70px;color:var(--text-muted);font-size:10px;text-transform:uppercase;letter-spacing:.06em;font-weight:600;">"Override"</span>
                                {move || {
                                    if override_type.get() == "deny" {
                                        view! {
                                            <>
                                                <span style="color:var(--red);font-weight:600;">"→ Deny"</span>
                                                <span style="font-size:10.5px;color:var(--text-muted);">" — explicit block bypasses other rules"</span>
                                            </>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <>
                                                <span style="color:var(--cobalt);font-weight:600;">"→ Grant"</span>
                                                <span style="font-size:10.5px;color:var(--text-muted);">" — NI override bypasses plan gate"</span>
                                            </>
                                        }.into_any()
                                    }
                                }}
                            </div>
                            <div style="border-top:1px solid var(--border-strong);margin-top:4px;padding-top:8px;display:flex;align-items:center;gap:8px;font-size:12px;font-weight:700;">
                                <span style="width:70px;color:var(--text-muted);font-size:10px;text-transform:uppercase;letter-spacing:.06em;font-weight:600;">"Effective"</span>
                                {move || {
                                    if override_type.get() == "deny" {
                                        view! { <span style="color:var(--red);">"🚫 Blocked"</span> }.into_any()
                                    } else {
                                        view! { <span style="color:var(--green);">"✅ Active"</span> }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>

                <div class="assign-panel-footer">
                    <button class="btn btn-ghost" on:click=move |_| close_override_drawer()>"Cancel"</button>
                    <button class="btn btn-primary" on:click=handle_save_override>"Save Override · Audit-Log"</button>
                </div>
            </div>

            // ── Create New Flag Modal ──
            <Show when=move || show_new_flag_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-xs flex items-center justify-center p-4">
                    <div class="bg-[#111520] w-full max-w-md p-6 rounded-lg border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_new_flag_modal.set(false)>"✕"</button>
                        <h3 class="text-base font-bold mb-2">"New Feature Flag"</h3>
                        <p class="text-xs text-on-surface-variant mb-6">"Define a new global feature key and registry rollout rules."</p>
                        
                        <div class="space-y-4 mb-6">
                            <div class="n-form-row">
                                <label class="n-form-label">"Flag KeyRegistry Key *"</label>
                                <input 
                                    type="text" 
                                    class="n-form-input font-mono"
                                    placeholder="snake_case_key"
                                    prop:value=new_flag_key
                                    on:input=move |ev| new_flag_key.set(event_target_value(&ev))
                                />
                                <div class="n-form-hint">"Unique platform-wide. Becomes FlagKey::SnakeCaseKey in Rust."</div>
                            </div>
                            <div class="n-form-row">
                                <label class="n-form-label">"Description / Purpose *"</label>
                                <input 
                                    type="text" 
                                    class="n-form-input"
                                    placeholder="What does enabling this flag do?"
                                    prop:value=new_flag_desc
                                    on:input=move |ev| new_flag_desc.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="n-form-row">
                                <label class="n-form-label">"Initial Variant"</label>
                                <select 
                                    class="n-form-select"
                                    on:change=move |ev| {
                                        let val = event_target_value(&ev);
                                        new_flag_has_global.set(val == "global");
                                    }
                                >
                                    <option value="global">"Global — 0% dark launch (enable globally when ready)"</option>
                                    <option value="plan">"Plan-Scoped — set a plan gate (no global rollout)"</option>
                                    <option value="ni">"NI Override only — no global or plan rollout"</option>
                                </select>
                            </div>
                            <div class="n-form-row">
                                <label class="n-form-label">"Jira Ticket / Issue ID *"</label>
                                <input 
                                    type="text" 
                                    class="n-form-input font-mono uppercase"
                                    placeholder="ATLAS-XXXX"
                                    prop:value=new_flag_jira
                                    on:input=move |ev| new_flag_jira.set(event_target_value(&ev))
                                />
                                <div class="n-form-hint">"Required. Every flag must be traceable."</div>
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| show_new_flag_modal.set(false) class="btn btn-ghost">"Cancel"</button>
                            <button on:click=handle_create_flag class="btn btn-primary">"Create Flag"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Edit Rollout Percentage Modal ──
            <Show when=move || show_rollout_modal.get().is_some()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-xs flex items-center justify-center p-4">
                    <div class="bg-[#111520] w-full max-w-sm p-6 rounded-lg border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_rollout_modal.set(None)>"✕"</button>
                        <h3 class="text-base font-bold mb-2">"Edit Rollout % · " {move || show_rollout_modal.get().map(|(key, _, _)| key).unwrap_or_default()}</h3>
                        <p class="text-xs text-on-surface-variant mb-6">
                            "Modify global rollout for key: " <code class="bg-[#0A0C16] px-1 py-0.5 rounded font-mono">{move || show_rollout_modal.get().map(|(key, _, _)| key).unwrap_or_default()}</code>
                        </p>

                        <div class="space-y-4 mb-6">
                            <div class="n-form-row">
                                <label class="n-form-label">"Global Rollout (0–100)"</label>
                                <input 
                                    type="number" min="0" max="100"
                                    class="n-form-input"
                                    prop:value=temp_rollout_val
                                    on:input=move |ev| {
                                        if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                            temp_rollout_val.set(val.clamp(0, 100));
                                        }
                                    }
                                />
                                <div class="n-form-hint">"0 = dark launch · 100 = fully live · between = canary"</div>
                            </div>
                            <div class="n-form-row">
                                <label class="n-form-label">"Reason"</label>
                                <input 
                                    type="text" 
                                    class="n-form-input"
                                    placeholder="e.g. 72h canary stable, no errors — bumping to 100%"
                                />
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| show_rollout_modal.set(None) class="btn btn-ghost">"Cancel"</button>
                            <button on:click=handle_save_rollout class="btn btn-primary">"Update Rollout"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
