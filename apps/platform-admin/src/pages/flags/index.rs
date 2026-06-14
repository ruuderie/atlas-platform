use leptos::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MockOverride {
    pub tenant_slug: String,
    pub tenant_plan: String,
    pub override_type: String, // "grant" or "deny"
    pub rollout_pct: i32,
    pub reason: String,
    pub jira: String,
    pub changed_by: String,
    pub changed_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MockAuditLog {
    pub date: String,
    pub user: String,
    pub action: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MockFlag {
    pub key: String,
    pub description: String,
    pub is_enabled: RwSignal<bool>,
    pub has_global: bool,
    pub global_rollout: RwSignal<i32>,
    pub is_plan_gated: bool,
    pub plan_gate_tier: String, // "Enterprise", "Growth", "Starter"
    pub overrides: RwSignal<Vec<MockOverride>>,
    pub audit_logs: RwSignal<Vec<MockAuditLog>>,
    pub jira: String,
    pub owner: String,
    pub date_created: String,
    pub expanded: RwSignal<bool>,
    pub active_tab: RwSignal<String>, // "variants", "overrides", "audit"
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockTenant {
    pub slug: String,
    pub name: String,
    pub plan: String,
    pub icon_char: char,
    pub bg_class: &'static str,
    pub text_class: &'static str,
}

#[component]
pub fn FeatureFlags() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // Static Tenant Autocomplete Options
    let mock_tenants_vec = vec![
        MockTenant { slug: "nexus-property-group".to_string(), name: "Nexus Property Group".to_string(), plan: "Enterprise".to_string(), icon_char: 'N', bg_class: "bg-blue-500/10 border-blue-500/30", text_class: "text-blue-400" },
        MockTenant { slug: "miami-stays".to_string(), name: "Miami Stays STR".to_string(), plan: "Growth".to_string(), icon_char: 'M', bg_class: "bg-amber-500/10 border-amber-500/30", text_class: "text-amber-400" },
        MockTenant { slug: "leira-chicago".to_string(), name: "Leira Chicago PM".to_string(), plan: "Starter".to_string(), icon_char: 'L', bg_class: "bg-slate-500/10 border-slate-500/30", text_class: "text-slate-400" },
        MockTenant { slug: "nexus-brasil".to_string(), name: "Nexus Brasil".to_string(), plan: "Enterprise".to_string(), icon_char: 'B', bg_class: "bg-emerald-500/10 border-emerald-500/30", text_class: "text-emerald-400" },
        MockTenant { slug: "ruud-commercial".to_string(), name: "Ruud Commercial".to_string(), plan: "Growth".to_string(), icon_char: 'R', bg_class: "bg-violet-500/10 border-violet-500/30", text_class: "text-violet-400" },
    ];
    let mock_tenants = StoredValue::new(mock_tenants_vec);

    // Master list of Flags
    let flags = RwSignal::new(vec![
        MockFlag {
            key: "ota_sync_v2".to_string(),
            description: "OTA sync v2 engine — Airbnb & VRBO availability propagation at 10-min cadence. Replaces legacy polling.".to_string(),
            is_enabled: RwSignal::new(true),
            has_global: true,
            global_rollout: RwSignal::new(100),
            is_plan_gated: false,
            plan_gate_tier: "".to_string(),
            overrides: RwSignal::new(vec![]),
            audit_logs: RwSignal::new(vec![
                MockAuditLog { date: "Jun 8".to_string(), user: "priya.s".to_string(), action: "Global rollout 50% → 100% · ATLAS-2841".to_string() },
                MockAuditLog { date: "Apr 14".to_string(), user: "dan.h".to_string(), action: "Flag created at 0% · ATLAS-2841".to_string() },
            ]),
            jira: "ATLAS-2841".to_string(),
            owner: "priya.s".to_string(),
            date_created: "Apr 14".to_string(),
            expanded: RwSignal::new(false),
            active_tab: RwSignal::new("variants".to_string()),
        },
        MockFlag {
            key: "ledger_v3_engine".to_string(),
            description: "Ledger v3 — multi-currency split engine with BTC, Lightning, Pix, Stripe. Target 100% by Jun 30.".to_string(),
            is_enabled: RwSignal::new(true),
            has_global: true,
            global_rollout: RwSignal::new(80),
            is_plan_gated: false,
            plan_gate_tier: "".to_string(),
            overrides: RwSignal::new(vec![
                MockOverride {
                    tenant_slug: "nexus-brasil".to_string(),
                    tenant_plan: "Enterprise".to_string(),
                    override_type: "grant".to_string(),
                    rollout_pct: 100,
                    reason: "Pilot per AM request".to_string(),
                    jira: "ATLAS-3050".to_string(),
                    changed_by: "dan.h".to_string(),
                    changed_at: "Jun 5".to_string(),
                },
                MockOverride {
                    tenant_slug: "leira-chicago".to_string(),
                    tenant_plan: "Starter".to_string(),
                    override_type: "deny".to_string(),
                    rollout_pct: 0,
                    reason: "Compliance hold pending legal review".to_string(),
                    jira: "ATLAS-3190".to_string(),
                    changed_by: "alex.r".to_string(),
                    changed_at: "Jun 11".to_string(),
                },
            ]),
            audit_logs: RwSignal::new(vec![
                MockAuditLog { date: "Jun 11".to_string(), user: "alex.r".to_string(), action: "NI Deny added: leira-chicago · ATLAS-3190".to_string() },
                MockAuditLog { date: "Jun 10".to_string(), user: "priya.s".to_string(), action: "Global 60% → 80% · target 100% Jun 30".to_string() },
                MockAuditLog { date: "Jun 5".to_string(), user: "dan.h".to_string(), action: "NI Grant added: nexus-brasil · 100%".to_string() },
                MockAuditLog { date: "May 1".to_string(), user: "priya.s".to_string(), action: "Flag created at 0% · ATLAS-3050".to_string() },
            ]),
            jira: "ATLAS-3050".to_string(),
            owner: "priya.s".to_string(),
            date_created: "May 1".to_string(),
            expanded: RwSignal::new(true),
            active_tab: RwSignal::new("variants".to_string()),
        },
        MockFlag {
            key: "mls_import".to_string(),
            description: "MLS data import for brokerage listings. Plan-gated to Enterprise tier. Starter and Growth see upgrade CTA.".to_string(),
            is_enabled: RwSignal::new(true),
            has_global: true,
            global_rollout: RwSignal::new(100),
            is_plan_gated: true,
            plan_gate_tier: "Enterprise".to_string(),
            overrides: RwSignal::new(vec![]),
            audit_logs: RwSignal::new(vec![
                MockAuditLog { date: "May 28".to_string(), user: "alex.r".to_string(), action: "Plan gate set to Enterprise · ATLAS-3012".to_string() },
            ]),
            jira: "ATLAS-3012".to_string(),
            owner: "alex.r".to_string(),
            date_created: "May 28".to_string(),
            expanded: RwSignal::new(false),
            active_tab: RwSignal::new("variants".to_string()),
        },
        MockFlag {
            key: "btc_payment_pilot".to_string(),
            description: "Bitcoin & Lightning STR booking payments. No global or plan rollout — NI override only.".to_string(),
            is_enabled: RwSignal::new(true),
            has_global: false,
            global_rollout: RwSignal::new(0),
            is_plan_gated: false,
            plan_gate_tier: "".to_string(),
            overrides: RwSignal::new(vec![
                MockOverride {
                    tenant_slug: "miami-stays".to_string(),
                    tenant_plan: "Growth".to_string(),
                    override_type: "grant".to_string(),
                    rollout_pct: 100,
                    reason: "Pilot per AM request".to_string(),
                    jira: "ATLAS-3101".to_string(),
                    changed_by: "alex.r".to_string(),
                    changed_at: "Jun 1".to_string(),
                },
            ]),
            audit_logs: RwSignal::new(vec![
                MockAuditLog { date: "Jun 1".to_string(), user: "alex.r".to_string(), action: "NI Grant added: miami-stays · 100% · ATLAS-3101".to_string() },
            ]),
            jira: "ATLAS-3101".to_string(),
            owner: "alex.r".to_string(),
            date_created: "Jun 1".to_string(),
            expanded: RwSignal::new(false),
            active_tab: RwSignal::new("variants".to_string()),
        },
        MockFlag {
            key: "g31_referral_engine".to_string(),
            description: "G-31 tenant-to-tenant lead referral network. Dark launched — awaiting legal sign-off.".to_string(),
            is_enabled: RwSignal::new(false),
            has_global: true,
            global_rollout: RwSignal::new(0),
            is_plan_gated: false,
            plan_gate_tier: "".to_string(),
            overrides: RwSignal::new(vec![]),
            audit_logs: RwSignal::new(vec![
                MockAuditLog { date: "Jun 9".to_string(), user: "dan.h".to_string(), action: "Flag created at 0% · dark launch · ATLAS-3200".to_string() },
            ]),
            jira: "ATLAS-3200".to_string(),
            owner: "dan.h".to_string(),
            date_created: "Jun 9".to_string(),
            expanded: RwSignal::new(false),
            active_tab: RwSignal::new("variants".to_string()),
        },
    ]);

    // Filtering & Searching UI State
    let search_query = RwSignal::new(String::new());
    let filter_scope = RwSignal::new("all".to_string()); // "all", "global", "plan", "ni", "off"

    // Side Drawer / Panel state for adding override
    let active_override_flag_key = RwSignal::new(None::<String>);
    let show_override_panel = RwSignal::new(false);
    
    // Forms state for NI override
    let override_tenant_input = RwSignal::new(String::new());
    let selected_tenant = RwSignal::new(None::<MockTenant>);
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
    let show_rollout_modal = RwSignal::new(None::<(String, String, i32)>); // (flag_key, scope_name, current_val)
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
                || f.overrides.get().iter().any(|o| o.tenant_slug.to_lowercase().contains(&query));
            
            // Scope Filter Match
            let matches_scope = match scope.as_str() {
                "global" => f.has_global,
                "plan" => f.is_plan_gated,
                "ni" => !f.overrides.get().is_empty(),
                "off" => !f.is_enabled.get() || (f.has_global && f.global_rollout.get() == 0),
                _ => true,
            };
            
            matches_query && matches_scope
        }).collect::<Vec<MockFlag>>()
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

    // Save Override
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

        let new_ovr = MockOverride {
            tenant_slug: tenant.slug.clone(),
            tenant_plan: tenant.plan.clone(),
            override_type: override_type.get(),
            rollout_pct: if override_type.get() == "deny" { 0 } else { override_rollout_pct.get() },
            reason: reason.clone(),
            jira: if override_jira.get().trim().is_empty() { "None".to_string() } else { override_jira.get() },
            changed_by: "alex.r".to_string(),
            changed_at: "Just now".to_string(),
        };

        if let Some(f) = flags.get().iter().find(|flg| flg.key == fkey) {
            // Add to list
            f.overrides.update(|list| {
                list.retain(|o| o.tenant_slug != tenant.slug); // Avoid duplicates
                list.push(new_ovr.clone());
            });

            // Write to audit log
            let action_desc = format!(
                "NI {} added: {} (Reason: {})",
                if new_ovr.override_type == "deny" { "Deny" } else { "Grant" },
                new_ovr.tenant_slug,
                new_ovr.reason
            );
            f.audit_logs.update(|logs| {
                logs.insert(0, MockAuditLog {
                    date: "Just now".to_string(),
                    user: "alex.r".to_string(),
                    action: action_desc,
                });
            });
            
            toast.show_toast("Success", &format!("Tenant override saved for {}.", tenant.slug), "success");
        }

        close_override_drawer();
    };

    // Remove Override
    let handle_remove_override = move |flag_key: String, tenant_slug: String| {
        if let Some(f) = flags.get().iter().find(|flg| flg.key == flag_key) {
            f.overrides.update(|list| {
                list.retain(|o| o.tenant_slug != tenant_slug);
            });
            
            f.audit_logs.update(|logs| {
                logs.insert(0, MockAuditLog {
                    date: "Just now".to_string(),
                    user: "alex.r".to_string(),
                    action: format!("NI Override removed: {}", tenant_slug),
                });
            });

            toast.show_toast("Warning", &format!("Override removed for {}.", tenant_slug), "warn");
        }
    };

    // Handle Global Enable/Disable Toggle
    let handle_global_toggle = move |flag_key: String, checked: bool| {
        if let Some(f) = flags.get().iter().find(|flg| flg.key == flag_key) {
            f.is_enabled.set(checked);
            let action_desc = format!("Global kill-switch toggled to {}", if checked { "ON" } else { "OFF" });
            f.audit_logs.update(|logs| {
                logs.insert(0, MockAuditLog {
                    date: "Just now".to_string(),
                    user: "alex.r".to_string(),
                    action: action_desc,
                });
            });
            toast.show_toast("Info", &format!("Global switch updated for {}.", flag_key), "info");
        }
    };

    // New flag submit handler
    let handle_create_flag = move |_| {
        let key = new_flag_key.get();
        let desc = new_flag_desc.get();
        let jira = new_flag_jira.get();

        if key.trim().is_empty() || desc.trim().is_empty() {
            toast.show_toast("Error", "Flag key and description are required.", "error");
            return;
        }

        let key_normalized = key.trim().to_lowercase().replace(" ", "_");
        let duplicate_exists = flags.get().iter().any(|f| f.key == key_normalized);
        if duplicate_exists {
            toast.show_toast("Error", "A flag with this key already exists.", "error");
            return;
        }

        flags.update(|list| {
            list.insert(0, MockFlag {
                key: key_normalized.clone(),
                description: desc,
                is_enabled: RwSignal::new(true),
                has_global: new_flag_has_global.get(),
                global_rollout: RwSignal::new(if new_flag_has_global.get() { 100 } else { 0 }),
                is_plan_gated: false,
                plan_gate_tier: "".to_string(),
                overrides: RwSignal::new(vec![]),
                audit_logs: RwSignal::new(vec![
                    MockAuditLog { date: "Just now".to_string(), user: "alex.r".to_string(), action: "Flag created at 100% rollout.".to_string() }
                ]),
                jira: if jira.trim().is_empty() { "None".to_string() } else { jira.trim().to_uppercase() },
                owner: "alex.r".to_string(),
                date_created: "Jun 2026".to_string(),
                expanded: RwSignal::new(true),
                active_tab: RwSignal::new("variants".to_string()),
            });
        });

        show_new_flag_modal.set(false);
        toast.show_toast("Success", &format!("Feature flag {} created.", key_normalized), "success");
    };

    // Rollout save handler
    let handle_save_rollout = move |_| {
        if let Some((flag_key, _scope, _)) = show_rollout_modal.get() {
            if let Some(f) = flags.get().iter().find(|flg| flg.key == flag_key) {
                let prev = f.global_rollout.get();
                let next = temp_rollout_val.get();
                f.global_rollout.set(next);
                
                f.audit_logs.update(|logs| {
                    logs.insert(0, MockAuditLog {
                        date: "Just now".to_string(),
                        user: "alex.r".to_string(),
                        action: format!("Global rollout % updated from {}% → {}% · ATLAS-Manual", prev, next),
                    });
                });
                
                toast.show_toast("Success", &format!("Rollout updated to {}% for {}.", next, flag_key), "success");
            }
        }
        show_rollout_modal.set(None);
    };

    view! {
        <div class="max-w-6xl mx-auto space-y-8 animate-in slide-in-from-bottom-4 duration-500 ease-out fade-in text-on-surface">
            // Page Header
            <header class="flex justify-between items-center bg-surface-container border border-outline-variant/10 p-6 rounded-2xl shadow-sm">
                <div>
                    <h1 class="text-3xl font-light tracking-tight text-on-surface mb-2 font-headline">"Feature Flags"</h1>
                    <p class="text-on-surface-variant text-sm tracking-wide">"Manage real-time rollout percentages, plan gates, and tenant-level overrides"</p>
                </div>
                <div class="flex gap-3">
                    <button 
                        on:click=move |_| toast.show_toast("Info", "Exporting flag audit logs to CSV...", "info")
                        class="px-4 py-2 text-sm font-semibold rounded-lg bg-[#05183c] border border-outline-variant/30 text-[#91aaeb] hover:bg-[#05183c]/60 active:scale-95 transition-all shadow-sm"
                    >
                        "Export Audit"
                    </button>
                    <button 
                        on:click=move |_| {
                            new_flag_key.set(String::new());
                            new_flag_desc.set(String::new());
                            new_flag_jira.set(String::new());
                            new_flag_has_global.set(true);
                            show_new_flag_modal.set(true);
                        }
                        class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-bold text-on-primary shadow-lg shadow-primary/20 hover:scale-105 active:scale-95 transition-all"
                    >
                        "+ New Flag"
                    </button>
                </div>
            </header>

            // Stat Strip Indicators
            <div class="grid grid-cols-2 md:grid-cols-6 gap-4">
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-1">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Total Flags"</span>
                    <span class="text-2xl font-bold font-mono text-on-surface">{move || flags.get().len().to_string()}</span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-1">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Globally On"</span>
                    <span class="text-2xl font-bold font-mono text-emerald-400">
                        {move || flags.get().iter().filter(|f| f.is_enabled.get() && f.has_global && f.global_rollout.get() == 100).count().to_string()}
                    </span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-1">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Dark / Off"</span>
                    <span class="text-2xl font-bold font-mono text-on-surface-variant/50">
                        {move || flags.get().iter().filter(|f| !f.is_enabled.get() || (f.has_global && f.global_rollout.get() == 0)).count().to_string()}
                    </span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-1">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Plan-Gated"</span>
                    <span class="text-2xl font-bold font-mono text-purple-400">
                        {move || flags.get().iter().filter(|f| f.is_plan_gated).count().to_string()}
                    </span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-1">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"NI Overrides"</span>
                    <span class="text-2xl font-bold font-mono text-emerald-400">
                        {move || flags.get().iter().map(|f| f.overrides.get().len()).sum::<usize>().to_string()}
                    </span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-1">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Canary (<100%)"</span>
                    <span class="text-2xl font-bold font-mono text-amber-500">
                        {move || flags.get().iter().filter(|f| f.is_enabled.get() && f.has_global && f.global_rollout.get() > 0 && f.global_rollout.get() < 100).count().to_string()}
                    </span>
                </div>
            </div>

            // Filter Bar & Search
            <div class="flex flex-wrap items-center justify-between gap-4 bg-surface-container/30 border border-outline-variant/10 px-5 py-3 rounded-xl">
                <div class="relative w-full md:w-80">
                    <span class="material-symbols-outlined absolute left-3 top-1/2 -translate-y-1/2 text-on-surface-variant text-sm">"search"</span>
                    <input 
                        type="text" 
                        placeholder="Search key, description, Jira, tenant..." 
                        class="bg-[#06122d] border border-outline-variant/30 text-on-surface text-sm rounded-lg pl-10 pr-4 py-2 w-full focus:ring-1 focus:ring-primary focus:border-primary placeholder:text-on-surface-variant/40"
                        prop:value=search_query
                        on:input=move |ev| search_query.set(event_target_value(&ev))
                    />
                </div>
                <div class="flex flex-wrap gap-2">
                    {
                        let filter_pill = move |scope: &'static str, label: &'static str, active_bg: &'static str| {
                            let s_str = scope.to_string();
                            let active_s = s_str.clone();
                            let click_s = s_str.clone();
                            view! {
                                <button 
                                    on:click=move |_| filter_scope.set(click_s.clone())
                                    class=move || format!(
                                        "px-3.5 py-1.5 rounded-lg text-xs font-semibold border transition-all {}",
                                        if filter_scope.get() == active_s {
                                            active_bg.to_string()
                                        } else {
                                            "bg-[#05183c]/20 border-outline-variant/30 text-on-surface-variant hover:text-on-surface hover:bg-[#05183c]/50".to_string()
                                        }
                                    )
                                >
                                    {label}
                                </button>
                            }
                        };
                        view! {
                            {filter_pill("all", "All Flags", "bg-primary-container border-primary text-primary")}
                            {filter_pill("global", "Has Global", "bg-blue-500/10 border-blue-500/30 text-blue-400")}
                            {filter_pill("plan", "Plan-Gated", "bg-purple-500/10 border-purple-500/30 text-purple-400")}
                            {filter_pill("ni", "Has NI Overrides", "bg-emerald-500/10 border-emerald-500/30 text-emerald-400")}
                            {filter_pill("off", "Off / Dark", "bg-amber-500/10 border-amber-500/30 text-amber-400")}
                        }
                    }
                </div>
            </div>

            // Flags list
            <div class="space-y-4">
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
                                "bg-surface border rounded-xl overflow-hidden shadow-sm transition-all {}",
                                if is_expanded.get() { "border-outline-variant" } else { "border-outline-variant/10 hover:border-outline-variant/30" }
                            )>
                                // Header row
                                <div 
                                    on:click=move |_| is_expanded.update(|v| *v = !*v)
                                    class="px-5 py-4 flex flex-wrap items-center justify-between gap-4 cursor-pointer select-none"
                                >
                                    <div class="flex items-center gap-4">
                                        // Toggle Switch
                                        <div on:click=move |e: leptos::ev::MouseEvent| e.stop_propagation() class="flex items-center">
                                            <label class="relative inline-flex items-center cursor-pointer">
                                                <input 
                                                    type="checkbox" 
                                                    class="sr-only peer"
                                                    prop:checked=is_enabled
                                                    on:change={
                                                        let fk = fkey.clone();
                                                        move |ev| handle_global_toggle(fk.get_value(), event_target_checked(&ev))
                                                    }
                                                />
                                                <div class="w-9 h-5 bg-[#2b4680]/30 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-emerald-600"></div>
                                            </label>
                                        </div>

                                        <div class="space-y-1">
                                            <div class="flex items-center gap-3">
                                                <span class="font-mono text-sm font-bold tracking-tight text-on-surface">{fkey.get_value()}</span>
                                                <Show when={
                                                    let jr = jira.clone();
                                                    move || {
                                                        let j = jr.get_value();
                                                        !j.is_empty() && j != "None"
                                                    }
                                                }>
                                                    <span class="text-[10px] font-bold font-mono px-2 py-0.5 rounded bg-blue-500/10 border border-blue-500/20 text-blue-400">{jira.get_value()}</span>
                                                </Show>
                                            </div>
                                            <p class="text-xs text-on-surface-variant max-w-2xl">{description.get_value()}</p>
                                        </div>
                                    </div>

                                    <div class="flex items-center gap-4" on:click=move |e: leptos::ev::MouseEvent| e.stop_propagation()>
                                        <div class="flex items-center gap-2">
                                            <Show when=move || has_global>
                                                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded border border-blue-500/30 text-blue-400 bg-blue-500/5">
                                                    "Global: " {move || global_rollout.get().to_string()} "%"
                                                </span>
                                            </Show>
                                            <Show when=move || is_plan_gated>
                                                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded border border-purple-500/30 text-purple-400 bg-purple-500/5">
                                                    {plan_gate_tier.get_value()} " Gate"
                                                </span>
                                            </Show>
                                            <Show when=move || has_overrides.get()>
                                                <span class="text-[10px] font-bold uppercase tracking-wider px-2 py-0.5 rounded border border-emerald-500/30 text-emerald-400 bg-emerald-500/5">
                                                    {move || count_overrides.get().to_string()} " Overrides"
                                                </span>
                                            </Show>
                                        </div>

                                        <button 
                                            on:click={
                                                let fk = fkey.clone();
                                                move |_| open_override_drawer(fk.get_value())
                                            }
                                            class="px-2.5 py-1 text-xs font-semibold bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 rounded transition-all"
                                        >
                                            "+ Override"
                                        </button>
                                        
                                        <button 
                                            on:click=move |_| is_expanded.update(|v| *v = !*v)
                                            class="text-on-surface-variant hover:text-on-surface p-1 transition-colors"
                                        >
                                            <span class=move || format!("material-symbols-outlined text-lg transition-transform duration-200 {}", if is_expanded.get() { "-scale-y-100" } else { "" })>
                                                "expand_more"
                                            </span>
                                        </button>
                                    </div>
                                </div>

                                // Expanded details body
                                <Show when=move || is_expanded.get()>
                                    <div class="border-t border-outline-variant/10 bg-[#05122d]/30">
                                        // Tabs
                                        <div class="flex border-b border-outline-variant/10 px-5 pt-1">
                                            {
                                                let tab_btn = move |tab_id: &'static str, label: &'static str| {
                                                    let id_str = tab_id.to_string();
                                                    let active_id = id_str.clone();
                                                    let click_id = id_str.clone();
                                                    view! {
                                                        <button 
                                                            on:click=move |_| active_tab.set(click_id.clone())
                                                            class=move || format!(
                                                                "px-4 py-2.5 text-xs font-semibold border-b-2 transition-all {}",
                                                                if active_tab.get() == active_id { "border-primary text-on-surface" } else { "border-transparent text-on-surface-variant hover:text-on-surface" }
                                                            )
                                                        >
                                                            {label}
                                                        </button>
                                                    }
                                                };
                                                view! {
                                                    {tab_btn("variants", "Variants & Rollout")}
                                                    {tab_btn("overrides", "NI Overrides")}
                                                    {tab_btn("audit", "Audit Log")}
                                                }
                                            }
                                        </div>

                                        // Tab Pane: Variants & Rollout
                                        <Show when=move || active_tab.get() == "variants">
                                            <div class="p-5 overflow-x-auto w-full">
                                                <table class="w-full text-left text-xs whitespace-nowrap">
                                                    <thead class="bg-surface-container-highest/60 text-[#91aaeb] font-semibold uppercase tracking-wider">
                                                        <tr>
                                                            <th class="px-4 py-3">"Scope"</th>
                                                            <th class="px-4 py-3">"State"</th>
                                                            <th class="px-4 py-3">"Rollout %"</th>
                                                            <th class="px-4 py-3">"Applies To"</th>
                                                            <th class="px-4 py-3">"Changed By"</th>
                                                            <th class="px-4 py-3"></th>
                                                        </tr>
                                                    </thead>
                                                    <tbody class="divide-y divide-outline-variant/10 text-on-surface">
                                                        <Show when=move || has_global>
                                                            <tr class="hover:bg-surface-bright/5">
                                                                <td class="px-4 py-3"><span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold border border-blue-500/30 text-blue-400 bg-blue-500/5">"Global"</span></td>
                                                                <td class="px-4 py-3">
                                                                    {move || if global_rollout.get() == 0 {
                                                                        view! { <span class="text-amber-500">"● Dark (0%)"</span> }.into_any()
                                                                    } else if global_rollout.get() == 100 {
                                                                        view! { <span class="text-emerald-400">"● Enabled"</span> }.into_any()
                                                                    } else {
                                                                        view! { <span class="text-blue-400">"⏳ Canary"</span> }.into_any()
                                                                    }}
                                                                </td>
                                                                <td class="px-4 py-3">
                                                                    <div class="flex items-center gap-2.5">
                                                                        <div class="w-20 bg-[#06122d] border border-outline-variant/30 h-2 rounded-full overflow-hidden">
                                                                            <div class="bg-blue-500 h-full rounded-full" style=move || format!("width: {}%", global_rollout.get())></div>
                                                                        </div>
                                                                        <span class="font-mono">{move || global_rollout.get().to_string()} "%"</span>
                                                                    </div>
                                                                </td>
                                                                <td class="px-4 py-3 text-on-surface-variant">"All connected NIs"</td>
                                                                <td class="px-4 py-3 text-on-surface-variant">{owner.get_value()}</td>
                                                                <td class="px-4 py-3 text-right">
                                                                    <button 
                                                                        on:click={
                                                                            let fk = fkey.clone();
                                                                            move |_| {
                                                                                temp_rollout_val.set(global_rollout.get());
                                                                                show_rollout_modal.set(Some((fk.get_value(), "Global".to_string(), global_rollout.get())));
                                                                            }
                                                                        }
                                                                        class="px-2 py-1 bg-surface-container hover:bg-surface-container-high border border-outline-variant/30 rounded transition-all"
                                                                    >
                                                                        "Edit %"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        </Show>
                                                        <Show when=move || is_plan_gated>
                                                            <tr class="hover:bg-surface-bright/5">
                                                                <td class="px-4 py-3"><span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold border border-purple-500/30 text-purple-400 bg-purple-500/5">"Plan Gate"</span></td>
                                                                <td class="px-4 py-3"><span class="text-purple-400">"● " {plan_gate_tier.get_value()}</span></td>
                                                                <td class="px-4 py-3">"—"</td>
                                                                <td class="px-4 py-3 text-on-surface-variant">"Only plans at or above " {plan_gate_tier.get_value()}</td>
                                                                <td class="px-4 py-3 text-on-surface-variant">{owner.get_value()}</td>
                                                                <td class="px-4 py-3 text-right">
                                                                    <button 
                                                                        on:click=move |_| toast.show_toast("Info", "Plan gate definitions must be modified in Billing rules tier mapping.", "info")
                                                                        class="px-2 py-1 bg-surface-container hover:bg-surface-container-high border border-outline-variant/30 rounded transition-all"
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
                                            <div class="p-5 space-y-3">
                                                <Show when=move || !overrides.get().is_empty() fallback=move || view! {
                                                    <div class="text-center py-6 text-xs text-on-surface-variant">
                                                        "No custom tenant-specific overrides registered for this flag key."
                                                    </div>
                                                }>
                                                    <div class="divide-y divide-outline-variant/10 border border-outline-variant/10 rounded-xl overflow-hidden bg-surface-container/20">
                                                        <For 
                                                            each=move || overrides.get()
                                                            key=|o| o.tenant_slug.clone()
                                                            children={
                                                                let fk = fkey.clone();
                                                                move |o| {
                                                                    let tenant_slug = o.tenant_slug.clone();
                                                                    let o_type = o.override_type.clone();
                                                                    let fk_inner = fk.clone();
                                                                    view! {
                                                                        <div class="px-4 py-3 flex items-center justify-between text-xs hover:bg-surface-bright/5 transition-all">
                                                                            <div class="flex items-center gap-4">
                                                                                <div class="space-y-1">
                                                                                    <div class="flex items-center gap-2">
                                                                                        <span class="font-mono font-bold text-on-surface">{o.tenant_slug.clone()}</span>
                                                                                        <span class="px-1.5 py-0.5 rounded text-[9px] uppercase font-bold bg-[#06122d] border border-outline-variant/20 text-on-surface-variant">{o.tenant_plan.clone()}</span>
                                                                                    </div>
                                                                                    <p class="text-[11px] text-on-surface-variant">
                                                                                        {o.reason.clone()} " · changed by " <strong class="text-on-surface">{o.changed_by.clone()}</strong>
                                                                                    </p>
                                                                                </div>
                                                                            </div>

                                                                            <div class="flex items-center gap-4">
                                                                                {if o_type == "deny" {
                                                                                    view! {
                                                                                        <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold bg-red-500/10 border border-red-500/30 text-red-400">
                                                                                            "🚫 Blocked"
                                                                                        </span>
                                                                                    }.into_any()
                                                                                } else {
                                                                                    view! {
                                                                                        <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold bg-emerald-500/10 border border-emerald-500/30 text-emerald-400">
                                                                                            "✓ Granted (" {o.rollout_pct.to_string()} "%)"
                                                                                        </span>
                                                                                    }.into_any()
                                                                                }}
                                                                                <button 
                                                                                    on:click=move |_| handle_remove_override(fk_inner.get_value(), tenant_slug.clone())
                                                                                    class="p-1 text-on-surface-variant hover:text-red-400 hover:bg-red-500/10 rounded transition-all"
                                                                                    title="Remove override"
                                                                                >
                                                                                    <span class="material-symbols-outlined text-sm">"delete"</span>
                                                                                </button>
                                                                            </div>
                                                                        </div>
                                                                    }
                                                                 }
                                                            }
                                                        />
                                                    </div>
                                                </Show>
                                            </div>
                                        </Show>

                                        // Tab Pane: Audit Log
                                        <Show when=move || active_tab.get() == "audit">
                                            <div class="p-5 space-y-2 max-h-60 overflow-y-auto">
                                                <For 
                                                    each=move || audit_logs.get()
                                                    key=|log| format!("{}-{}-{}", log.date, log.user, log.action)
                                                    children=move |log| {
                                                        view! {
                                                            <div class="flex items-start gap-4 text-xs border-b border-outline-variant/10 pb-2 last:border-b-0">
                                                                <span class="font-mono text-on-surface-variant w-14 flex-shrink-0">{log.date.clone()}</span>
                                                                <span class="font-semibold text-primary w-20 flex-shrink-0 truncate">{log.user.clone()}</span>
                                                                <span class="text-on-surface flex-1">{log.action.clone()}</span>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            </div>
                                        </Show>
                                    </div>
                                </Show>
                            </div>
                        }
                    }
                />
            </div>

            // Assign NI Override Side Panel Drawer
            <div 
                on:click=move |_| close_override_drawer()
                class=move || format!(
                    "fixed inset-0 z-[300] bg-black/60 backdrop-blur-xs transition-opacity duration-200 {}",
                    if show_override_panel.get() { "opacity-100 pointer-events-auto" } else { "opacity-0 pointer-events-none" }
                )
            ></div>
            
            <div class=move || format!(
                "fixed top-0 right-0 h-screen w-[460px] bg-card border-l border-outline-variant/30 z-[400] flex flex-col shadow-2xl transition-transform duration-300 ease-out {}",
                if show_override_panel.get() { "translate-x-0" } else { "translate-x-full" }
            )>
                <div class="px-5 py-4 border-b border-outline-variant/20 flex items-center justify-between flex-shrink-0">
                    <div>
                        <h3 class="text-base font-bold">"Assign NI Override"</h3>
                        <p class="text-[11px] text-on-surface-variant font-mono mt-0.5">
                            {move || active_override_flag_key.get().unwrap_or_default()}
                        </p>
                    </div>
                    <button 
                        on:click=move |_| close_override_drawer()
                        class="text-on-surface-variant hover:text-on-surface text-sm p-1 rounded hover:bg-surface-bright/20"
                    >
                        "✕ Close"
                    </button>
                </div>

                <div class="flex-1 overflow-y-auto p-5 space-y-5">
                    <div class="p-3 bg-surface-container border border-outline-variant/20 rounded-lg text-xs leading-relaxed text-on-surface-variant">
                        <strong>"NI Override"</strong> " grants or denies access for a single tenant, bypassing the plan gate. A " <strong>"grant"</strong> " lets an NI access a feature above their plan tier. A " <strong>"deny"</strong> " blocks a specific NI below their plan tier. All changes are audit-logged."
                    </div>

                    // Tenant Autocomplete search
                    <div class="flex flex-col gap-1.5 relative">
                        <label class="text-xs font-semibold text-on-surface-variant">"Tenant (NI slug or name)"</label>
                        <Show when=move || selected_tenant.get().is_some() fallback=move || view! {
                            <div class="relative">
                                <input 
                                    type="text" 
                                    class="bg-surface-container-highest border border-outline/25 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    placeholder="Search nexus-property, miami-stays..."
                                    prop:value=override_tenant_input
                                    on:focus=move |_| autocomplete_open.set(true)
                                    on:input=move |ev| {
                                        let val = event_target_value(&ev);
                                        override_tenant_input.set(val);
                                        autocomplete_open.set(true);
                                    }
                                />
                                // Autocomplete Dropdown list
                                <Show when=move || autocomplete_open.get()>
                                    <div class="absolute left-0 right-0 top-12 mt-1 bg-surface-container border border-outline-variant/40 rounded-xl overflow-hidden shadow-xl z-55 max-h-60 overflow-y-auto">
                                        {
                                            let term = override_tenant_input.get().to_lowercase();
                                            let filtered_opts: Vec<MockTenant> = mock_tenants.get_value().into_iter().filter(|t| {
                                                term.is_empty() || t.slug.to_lowercase().contains(&term) || t.name.to_lowercase().contains(&term)
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
                                                                    class="flex items-center gap-3 px-4 py-2.5 cursor-pointer hover:bg-surface-bright/20 border-b border-outline-variant/10 last:border-b-0"
                                                                >
                                                                    <div class=format!("w-7 h-7 rounded flex items-center justify-center font-bold text-xs border {}", t.bg_class)>
                                                                        {t.icon_char}
                                                                    </div>
                                                                    <div>
                                                                        <div class="text-xs font-mono font-semibold text-on-surface">{t.slug.clone()}</div>
                                                                        <div class="text-[10px] text-on-surface-variant">{t.name.clone()} " · " <span class="text-primary">{t.plan.clone()}</span></div>
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
                        }.into_any()>
                            // Selected Tenant Chip view
                            {let st = selected_tenant.get().unwrap();
                             view! {
                                <div class="flex items-center justify-between p-3 rounded-lg border border-primary bg-primary-container/20 text-xs">
                                    <div class="flex items-center gap-3">
                                        <div class=format!("w-6 h-6 rounded flex items-center justify-center font-bold border text-[10px] {}", st.bg_class)>
                                            {st.icon_char}
                                        </div>
                                        <div>
                                            <div class="font-mono font-bold text-primary">{st.slug.clone()}</div>
                                            <div class="text-[10px] text-on-surface-variant">{st.name.clone()} " · " {st.plan.clone()}</div>
                                        </div>
                                    </div>
                                    <button 
                                        on:click=move |_| selected_tenant.set(None)
                                        class="text-on-surface-variant hover:text-on-surface font-semibold px-2 py-1 bg-surface-container rounded hover:bg-surface-container-high"
                                    >
                                        "Change"
                                    </button>
                                </div>
                            }.into_any()}
                        </Show>
                    </div>

                    // Override Type Selection
                    <div class="flex flex-col gap-1.5">
                        <label class="text-xs font-semibold text-on-surface-variant">"Override Type"</label>
                        <select 
                            class="bg-surface-container-highest border border-outline/25 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                            on:change=move |ev| override_type.set(event_target_value(&ev))
                        >
                            <option value="grant">"Grant — enable for this tenant (above plan tier)"</option>
                            <option value="deny">"Deny — block for this tenant (compliance / hold)"</option>
                        </select>
                        <p class="text-[10px] text-on-surface-variant/75">
                            {move || if override_type.get() == "grant" {
                                "This tenant will gain access to this feature regardless of their plan tier."
                            } else {
                                "This tenant will be strictly blocked from this feature regardless of global state."
                            }}
                        </p>
                    </div>

                    // Rollout % (only active for grant)
                    <Show when=move || override_type.get() == "grant">
                        <div class="flex flex-col gap-1.5">
                            <label class="text-xs font-semibold text-on-surface-variant">"Rollout % within this tenant (0–100)"</label>
                            <input 
                                type="number" 
                                min="0" max="100"
                                class="bg-surface-container-highest border border-outline/25 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                prop:value=override_rollout_pct
                                on:input=move |ev| {
                                    if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                        override_rollout_pct.set(val.clamp(0, 100));
                                    }
                                }
                            />
                            <p class="text-[10px] text-on-surface-variant/70">"100 = all users in this NI see it. Lower values = canary within the tenant."</p>
                        </div>
                    </Show>

                    // Audit reason input
                    <div class="flex flex-col gap-1.5">
                        <label class="text-xs font-semibold text-on-surface-variant">"Reason (Required) *"</label>
                        <input 
                            type="text"
                            class="bg-surface-container-highest border border-outline/25 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                            placeholder="e.g. Pilot per AM request · ATLAS-3200"
                            prop:value=override_reason
                            on:input=move |ev| override_reason.set(event_target_value(&ev))
                        />
                    </div>

                    // Optional Jira ticket
                    <div class="flex flex-col gap-1.5">
                        <label class="text-xs font-semibold text-on-surface-variant">"Jira Ticket (Optional)"</label>
                        <input 
                            type="text"
                            class="bg-surface-container-highest border border-outline/25 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary font-mono"
                            placeholder="ATLAS-XXXX"
                            prop:value=override_jira
                            on:input=move |ev| override_jira.set(event_target_value(&ev))
                        />
                    </div>

                    // Effective access cascade preview
                    <div class="p-4 bg-[#05070B] border border-outline-variant/30 rounded-xl space-y-3">
                        <div class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest border-b border-white/5 pb-1.5">"Effective Access Cascade Preview"</div>
                        <div class="space-y-2 text-xs">
                            <div class="flex items-center justify-between">
                                <span class="text-on-surface-variant">"Global switch status:"</span>
                                {move || {
                                    let is_active = flags.get().iter().find(|flg| Some(flg.key.clone()) == active_override_flag_key.get()).map(|flg| flg.is_enabled.get()).unwrap_or(false);
                                    if is_active {
                                        view! { <span class="text-emerald-400 font-semibold">"✓ ON"</span> }.into_any()
                                    } else {
                                        view! { <span class="text-amber-500 font-semibold">"● OFF"</span> }.into_any()
                                    }
                                }}
                            </div>
                            <div class="flex items-center justify-between">
                                <span class="text-on-surface-variant">"Plan gate check:"</span>
                                {move || {
                                    let has_gate = flags.get().iter().find(|flg| Some(flg.key.clone()) == active_override_flag_key.get()).map(|flg| flg.is_plan_gated).unwrap_or(false);
                                    if has_gate {
                                        view! { <span class="text-amber-400">"⚠ Gate active"</span> }.into_any()
                                    } else {
                                        view! { <span class="text-on-surface-variant">"None"</span> }.into_any()
                                    }
                                }}
                            </div>
                            <div class="flex items-center justify-between">
                                <span class="text-on-surface-variant">"Bypass override:"</span>
                                {move || {
                                    if override_type.get() == "deny" {
                                        view! { <span class="text-red-400 font-semibold">"→ Force Deny"</span> }.into_any()
                                    } else {
                                        view! { <span class="text-emerald-400 font-semibold">"→ Force Grant"</span> }.into_any()
                                    }
                                }}
                            </div>
                            <div class="border-t border-white/5 pt-2 flex items-center justify-between font-bold text-sm">
                                <span>"Effective result:"</span>
                                {move || {
                                    if override_type.get() == "deny" {
                                        view! { <span class="text-red-400">"🚫 Blocked"</span> }.into_any()
                                    } else {
                                        view! { <span class="text-emerald-400">"✅ Active"</span> }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>

                <div class="px-5 py-4 border-t border-outline-variant/20 flex justify-end gap-3 flex-shrink-0">
                    <button 
                        on:click=move |_| close_override_drawer()
                        class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface"
                    >
                        "Cancel"
                    </button>
                    <button 
                        on:click=handle_save_override
                        class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-bold text-on-primary"
                    >
                        "Save Override"
                    </button>
                </div>
            </div>

            // Create New Flag Modal dialog
            <Show when=move || show_new_flag_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_new_flag_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Create Feature Flag"</h3>
                        <p class="text-xs text-on-surface-variant mb-6">"Define a new global feature key registry rollout rules."</p>
                        
                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-medium text-on-surface-variant">"Flag Registry Key *"</label>
                                <input 
                                    type="text" 
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    placeholder="e.g. ota_sync_v2"
                                    prop:value=new_flag_key
                                    on:input=move |ev| new_flag_key.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-medium text-on-surface-variant">"Description / Purpose *"</label>
                                <input 
                                    type="text" 
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    placeholder="Brief description of the feature..."
                                    prop:value=new_flag_desc
                                    on:input=move |ev| new_flag_desc.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-medium text-on-surface-variant">"Jira Ticket / Issue ID"</label>
                                <input 
                                    type="text" 
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary font-mono uppercase"
                                    placeholder="ATLAS-XXXX"
                                    prop:value=new_flag_jira
                                    on:input=move |ev| new_flag_jira.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="flex items-center gap-3 pt-2">
                                <input 
                                    type="checkbox" 
                                    id="new-flag-global"
                                    class="w-4 h-4 rounded text-primary focus:ring-primary bg-[#05122d] border-outline-variant/30"
                                    prop:checked=new_flag_has_global
                                    on:change=move |ev| new_flag_has_global.set(event_target_checked(&ev))
                                />
                                <label for="new-flag-global" class="text-xs text-on-surface-variant">"Configure global default variant at 100% rollout"</label>
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| show_new_flag_modal.set(false) class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface">"Cancel"</button>
                            <button on:click=handle_create_flag class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-bold text-on-primary">"Create Registry"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // Edit Rollout Percentage Modal
            <Show when=move || show_rollout_modal.get().is_some()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-sm p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_rollout_modal.set(None)>"✕"</button>
                        <h3 class="text-base font-semibold mb-2">"Edit Global Rollout %"</h3>
                        <p class="text-xs text-on-surface-variant mb-6">
                            "Modify global rollout for key: " <code class="bg-[#05070B] px-1 py-0.5 rounded font-mono">{move || show_rollout_modal.get().map(|(key, _, _)| key).unwrap_or_default()}</code>
                        </p>

                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-2">
                                <div class="flex justify-between items-center text-xs text-on-surface-variant">
                                    <span>"Percentage Rollout (0-100)"</span>
                                    <span class="font-mono font-bold text-primary">{move || temp_rollout_val.get().to_string()} "%"</span>
                                </div>
                                <input 
                                    type="range" min="0" max="100" step="5"
                                    class="w-full h-1 bg-[#06122d] rounded-lg appearance-none cursor-pointer"
                                    prop:value=temp_rollout_val
                                    on:input=move |ev| {
                                        if let Ok(val) = event_target_value(&ev).parse::<i32>() {
                                            temp_rollout_val.set(val);
                                        }
                                    }
                                />
                            </div>
                        </div>

                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| show_rollout_modal.set(None) class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface">"Cancel"</button>
                            <button on:click=handle_save_rollout class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-bold text-on-primary">"Save Rollout"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
