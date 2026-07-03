use leptos::prelude::*;

#[component]
pub fn Scorecards() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // Sub tab navigation: templates, analytics
    let active_tab = RwSignal::new("templates".to_string());
    
    // Configurator state
    let show_configurator = RwSignal::new(false);
    let selected_template_name = RwSignal::new("Contractor Performance".to_string());
    let selected_template_entity = RwSignal::new("Lead".to_string());

    // Calibration settings modal state
    let show_settings_modal = RwSignal::new(false);

    let handle_new_template = move |_| {
        selected_template_name.set("New Evaluation Template".to_string());
        selected_template_entity.set("Tenant".to_string());
        show_configurator.set(true);
        toast.show_toast("Template", "New evaluation canvas initialized.", "info");
    };

    let handle_save_config = move |_| {
        show_configurator.set(false);
        toast.show_toast("Success", "Scorecard template schema published.", "success");
    };

    view! {
        <div class="main-canvas">
            // ── Page Header ──
            <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                <div>
                    <h1 class="text-2xl font-extrabold tracking-tight text-on-surface">"Scorecards"</h1>
                    <p class="text-xs text-on-surface-variant mt-1">"Universal structured evaluation engine · 6 active templates · 1,204 evaluations"</p>
                </div>
                <div class="flex items-center gap-3">
                    <button 
                        class="btn-ghost px-4 py-2 rounded-lg text-sm font-semibold border border-outline-variant/30 hover:bg-surface-bright/20 hover:text-on-surface transition-all active:scale-95"
                        on:click=move |_| show_settings_modal.set(true)
                    >
                        "Calibration Settings"
                    </button>
                    <button 
                        class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-semibold text-on-primary-container shadow-md shadow-primary/10 hover:opacity-90 active:scale-95 transition-all"
                        on:click=handle_new_template
                    >
                        "+ New Template"
                    </button>
                </div>
            </div>

            // ── Sub Navigation ──
            <div class="flex border-b border-outline-variant/20 overflow-x-auto shrink-0 select-none">
                <button 
                    class=move || if active_tab.get() == "templates" { "px-4 py-2.5 text-sm font-semibold text-primary border-b-2 border-primary transition-all shrink-0 bg-transparent" } else { "px-4 py-2.5 text-sm text-on-surface-variant hover:text-on-surface transition-all shrink-0 bg-transparent" }
                    on:click=move |_| active_tab.set("templates".to_string())
                >
                    "Templates"
                </button>
                <button 
                    class=move || if active_tab.get() == "analytics" { "px-4 py-2.5 text-sm font-semibold text-primary border-b-2 border-primary transition-all shrink-0 bg-transparent" } else { "px-4 py-2.5 text-sm text-on-surface-variant hover:text-on-surface transition-all shrink-0 bg-transparent" }
                    on:click=move |_| active_tab.set("analytics".to_string())
                >
                    "Analytics"
                </button>
            </div>

            // ── VIEW: Templates ──
            <Show when=move || active_tab.get() == "templates">
                <Show 
                    when=move || show_configurator.get()
                    fallback=move || view! {
                        <div class="space-y-6">
                            // Published Section
                            <div class="space-y-4">
                                <h3 class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">"Published Templates (4)"</h3>
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                    // Published Card 1
                                    <div 
                                        class="bg-surface-container-low border border-outline-variant/20 hover:border-primary/40 rounded-xl p-5 shadow-sm hover:shadow-md cursor-pointer transition-all flex flex-col justify-between min-h-[180px]"
                                        on:click=move |_| {
                                            selected_template_name.set("Contractor Performance".to_string());
                                            selected_template_entity.set("Lead".to_string());
                                            show_configurator.set(true);
                                        }
                                    >
                                        <div>
                                            <div class="flex justify-between items-start mb-3">
                                                <span class="px-2 py-0.5 rounded text-[8px] font-bold bg-primary-container/20 text-primary border border-primary/20 uppercase tracking-wider">"Lead"</span>
                                                <span class="inline-flex items-center px-1.5 py-0.5 rounded text-[8px] font-bold bg-emerald-500/10 text-emerald-400 uppercase tracking-wider">"Published"</span>
                                            </div>
                                            <h4 class="text-sm font-bold text-on-surface">"Contractor Performance"</h4>
                                            <p class="text-[11px] text-on-surface-variant/70 mt-2 leading-relaxed">"FMCSA compliance, DOT safety logs, and carrier capacity evaluation."</p>
                                        </div>
                                        <div class="flex items-center justify-between mt-4 border-t border-outline-variant/10 pt-3 text-[10px] text-on-surface-variant">
                                            <span>"5 dims · 842 scores"</span>
                                            <span class="font-bold text-emerald-400">"Avg 7.9"</span>
                                        </div>
                                    </div>

                                    // Published Card 2
                                    <div 
                                        class="bg-surface-container-low border border-outline-variant/20 hover:border-primary/40 rounded-xl p-5 shadow-sm hover:shadow-md cursor-pointer transition-all flex flex-col justify-between min-h-[180px]"
                                        on:click=move |_| {
                                            selected_template_name.set("Listing Quality Index".to_string());
                                            selected_template_entity.set("Property".to_string());
                                            show_configurator.set(true);
                                        }
                                    >
                                        <div>
                                            <div class="flex justify-between items-start mb-3">
                                                <span class="px-2 py-0.5 rounded text-[8px] font-bold bg-primary-container/20 text-primary border border-primary/20 uppercase tracking-wider">"Property"</span>
                                                <span class="inline-flex items-center px-1.5 py-0.5 rounded text-[8px] font-bold bg-emerald-500/10 text-emerald-400 uppercase tracking-wider">"Published"</span>
                                            </div>
                                            <h4 class="text-sm font-bold text-on-surface">"Listing Quality Index"</h4>
                                            <p class="text-[11px] text-on-surface-variant/70 mt-2 leading-relaxed">"Evaluates photography, pricing integrity, response latency, and ratings."</p>
                                        </div>
                                        <div class="flex items-center justify-between mt-4 border-t border-outline-variant/10 pt-3 text-[10px] text-on-surface-variant">
                                            <span>"6 dims · 204 scores"</span>
                                            <span class="font-bold text-emerald-400">"Avg 8.2"</span>
                                        </div>
                                    </div>

                                    // Published Card 3
                                    <div 
                                        class="bg-surface-container-low border border-outline-variant/20 hover:border-primary/40 rounded-xl p-5 shadow-sm hover:shadow-md cursor-pointer transition-all flex flex-col justify-between min-h-[180px]"
                                        on:click=move |_| {
                                            selected_template_name.set("Vendor Reliability".to_string());
                                            selected_template_entity.set("Vendor".to_string());
                                            show_configurator.set(true);
                                        }
                                    >
                                        <div>
                                            <div class="flex justify-between items-start mb-3">
                                                <span class="px-2 py-0.5 rounded text-[8px] font-bold bg-primary-container/20 text-primary border border-primary/20 uppercase tracking-wider">"Vendor"</span>
                                                <span class="inline-flex items-center px-1.5 py-0.5 rounded text-[8px] font-bold bg-emerald-500/10 text-emerald-400 uppercase tracking-wider">"Published"</span>
                                            </div>
                                            <h4 class="text-sm font-bold text-on-surface">"Vendor Reliability"</h4>
                                            <p class="text-[11px] text-on-surface-variant/70 mt-2 leading-relaxed">"Evaluates maintenance ticket speed, price fairness, and tenant reviews."</p>
                                        </div>
                                        <div class="flex items-center justify-between mt-4 border-t border-outline-variant/10 pt-3 text-[10px] text-on-surface-variant">
                                            <span>"4 dims · 98 scores"</span>
                                            <span class="font-bold text-amber-400">"Avg 6.4"</span>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            // Drafts Section
                            <div class="space-y-4 pt-6">
                                <h3 class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">"Draft Templates (2)"</h3>
                                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                    // Draft Card 1
                                    <div 
                                        class="bg-surface-container-low border border-outline-variant/20 hover:border-primary/40 rounded-xl p-5 shadow-sm hover:shadow-md cursor-pointer transition-all flex flex-col justify-between min-h-[180px]"
                                        on:click=move |_| {
                                            selected_template_name.set("Tenant Health Score".to_string());
                                            selected_template_entity.set("Tenant".to_string());
                                            show_configurator.set(true);
                                        }
                                    >
                                        <div>
                                            <div class="flex justify-between items-start mb-3">
                                                <span class="px-2 py-0.5 rounded text-[8px] font-bold bg-primary-container/20 text-primary border border-primary/20 uppercase tracking-wider">"Tenant"</span>
                                                <span class="inline-flex items-center px-1.5 py-0.5 rounded text-[8px] font-bold bg-amber-500/10 text-amber-400 uppercase tracking-wider">"Draft"</span>
                                            </div>
                                            <h4 class="text-sm font-bold text-on-surface">"Tenant Health Score"</h4>
                                            <p class="text-[11px] text-on-surface-variant/70 mt-2 leading-relaxed">"Telemetry checks: MRR growth, license compliance, and ticket load."</p>
                                        </div>
                                        <div class="flex items-center justify-between mt-4 border-t border-outline-variant/10 pt-3 text-[10px] text-on-surface-variant">
                                            <span>"7 dims · 0 scores"</span>
                                            <span class="font-bold text-amber-400">"In Progress"</span>
                                        </div>
                                    </div>

                                    // Draft Card 2
                                    <div 
                                        class="bg-surface-container-low border border-outline-variant/20 hover:border-primary/40 rounded-xl p-5 shadow-sm hover:shadow-md cursor-pointer transition-all flex flex-col justify-between min-h-[180px]"
                                        on:click=move |_| {
                                            selected_template_name.set("Asset Condition Index".to_string());
                                            selected_template_entity.set("Asset".to_string());
                                            show_configurator.set(true);
                                        }
                                    >
                                        <div>
                                            <div class="flex justify-between items-start mb-3">
                                                <span class="px-2 py-0.5 rounded text-[8px] font-bold bg-primary-container/20 text-primary border border-primary/20 uppercase tracking-wider">"Asset"</span>
                                                <span class="inline-flex items-center px-1.5 py-0.5 rounded text-[8px] font-bold bg-amber-500/10 text-amber-400 uppercase tracking-wider">"Draft"</span>
                                            </div>
                                            <h4 class="text-sm font-bold text-on-surface">"Asset Condition Index"</h4>
                                            <p class="text-[11px] text-on-surface-variant/70 mt-2 leading-relaxed">"Scoring framework for physical properties: HVAC, roof, electrical structures."</p>
                                        </div>
                                        <div class="flex items-center justify-between mt-4 border-t border-outline-variant/10 pt-3 text-[10px] text-on-surface-variant">
                                            <span>"5 dims · 0 scores"</span>
                                            <span class="font-bold text-amber-400">"In Progress"</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                >
                    // Configurator details canvas
                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                        <div class="lg:col-span-2 space-y-6">
                            <h2 class="text-lg font-bold text-on-surface">"Template Configurator"</h2>
                            
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="space-y-1">
                                    <label class="text-xs font-semibold text-on-surface-variant">"Template Name"</label>
                                    <input 
                                        type="text" 
                                        class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 focus:ring-1 focus:ring-primary focus:border-primary transition-all"
                                        prop:value=selected_template_name
                                        on:input=move |ev| selected_template_name.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="space-y-1">
                                    <label class="text-xs font-semibold text-on-surface-variant">"Target Entity Type"</label>
                                    <select 
                                        class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 focus:ring-1 focus:ring-primary focus:border-primary transition-all cursor-pointer"
                                        on:change=move |ev| selected_template_entity.set(event_target_value(&ev))
                                    >
                                        <option value="Lead" selected=move || selected_template_entity.get() == "Lead">"Lead"</option>
                                        <option value="Property" selected=move || selected_template_entity.get() == "Property">"Property Listing"</option>
                                        <option value="Vendor" selected=move || selected_template_entity.get() == "Vendor">"Maintenance Vendor"</option>
                                        <option value="Tenant" selected=move || selected_template_entity.get() == "Tenant">"Platform Tenant"</option>
                                        <option value="Asset" selected=move || selected_template_entity.get() == "Asset">"Physical Asset"</option>
                                    </select>
                                </div>
                            </div>

                            <div class="space-y-4">
                                <div class="flex justify-between items-center">
                                    <span class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"Evaluation Dimensions"</span>
                                    <button class="text-xs text-primary hover:underline">"+ Add Dimension"</button>
                                </div>
                                <div class="space-y-3">
                                    <div class="bg-surface-container p-4 rounded-lg border border-outline-variant/20 flex items-center justify-between">
                                        <div>
                                            <h5 class="text-xs font-bold text-on-surface">"1. Response Speed"</h5>
                                            <p class="text-[10px] text-on-surface-variant/70 mt-1">"Response rate / communication latency metrics."</p>
                                        </div>
                                        <span class="text-xs font-bold font-mono text-primary">"Weight: 30%"</span>
                                    </div>
                                    <div class="bg-surface-container p-4 rounded-lg border border-outline-variant/20 flex items-center justify-between">
                                        <div>
                                            <h5 class="text-xs font-bold text-on-surface">"2. Compliance Integrity"</h5>
                                            <p class="text-[10px] text-on-surface-variant/70 mt-1">"Background safety and document check compliance."</p>
                                        </div>
                                        <span class="text-xs font-bold font-mono text-primary">"Weight: 40%"</span>
                                    </div>
                                    <div class="bg-surface-container p-4 rounded-lg border border-outline-variant/20 flex items-center justify-between">
                                        <div>
                                            <h5 class="text-xs font-bold text-on-surface">"3. Financial Solvency"</h5>
                                            <p class="text-[10px] text-on-surface-variant/70 mt-1">"History of invoice clearance and credit ratings."</p>
                                        </div>
                                        <span class="text-xs font-bold font-mono text-primary">"Weight: 30%"</span>
                                    </div>
                                </div>
                            </div>

                            <div class="flex justify-end gap-3 pt-4 border-t border-outline-variant/10">
                                <button 
                                    class="btn-ghost px-4 py-2 rounded-lg text-sm font-semibold border border-outline-variant/30 hover:bg-surface-bright/20 transition-all active:scale-95"
                                    on:click=move |_| show_configurator.set(false)
                                >
                                    "Cancel"
                                </button>
                                <button 
                                    class="btn-primary px-4 py-2 rounded-lg text-sm font-semibold transition-all active:scale-95"
                                    on:click=handle_save_config
                                >
                                    "Save Template Schema"
                                </button>
                            </div>
                        </div>

                        // Configurator info side details
                        <div class="bg-surface-container p-6 rounded-xl border border-outline-variant/20 space-y-4 h-fit">
                            <h4 class="text-xs font-bold text-on-surface-variant uppercase tracking-widest">"Calibration Preview"</h4>
                            <p class="text-xs text-on-surface-variant/70 leading-relaxed">"Modifying weights dynamically recalibrates downstream scoring arrays across the outbox workers."</p>
                            <div class="bg-surface-container-low p-4 border border-outline-variant/10 rounded-lg text-[10px] font-mono text-on-surface-variant/70 space-y-2 select-all">
                                <div>"type: \"calibration_session\""</div>
                                <div>"entity_type: \"" {move || selected_template_entity.get()} "\""</div>
                                <div>"recalc_nodes: \"OutboxWorkerNode02\""</div>
                            </div>
                        </div>
                    </div>
                </Show>
            </Show>

            // ── VIEW: Analytics ──
            <Show when=move || active_tab.get() == "analytics">
                <div class="space-y-6">
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
                            <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Total Evaluations"</span>
                            <span class="text-3xl font-extrabold text-on-surface tracking-tight mt-2">"1,204"</span>
                            <span class="text-[10px] text-on-surface-variant/50 mt-1">"Across 6 active templates"</span>
                        </div>
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
                            <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"Platform Average"</span>
                            <span class="text-3xl font-extrabold text-emerald-400 tracking-tight mt-2">"7.6"</span>
                            <span class="text-[10px] text-emerald-400 font-semibold mt-1">"Outstanding average"</span>
                        </div>
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
                            <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"High Confidence"</span>
                            <span class="text-3xl font-extrabold text-on-surface tracking-tight mt-2">"68%"</span>
                            <span class="text-[10px] text-on-surface-variant/50 mt-1">"Over 25 samples verified"</span>
                        </div>
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col justify-between min-h-[100px]">
                            <span class="text-[10px] font-bold text-on-surface-variant/70 uppercase tracking-widest">"At-Risk Entities"</span>
                            <span class="text-3xl font-extrabold text-red-500 tracking-tight mt-2">"47"</span>
                            <span class="text-[10px] text-red-400 font-semibold mt-1">"Evaluation Score < 5.0"</span>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                        // Score Tier Distribution
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-4">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Score Tier Distribution"</h3>
                            
                            <div class="space-y-3">
                                {
                                    let dist_row = |label: &str, count: &str, pct: &str, color_class: &str, width: &str| {
                                        view! {
                                            <div class="flex items-center justify-between text-xs">
                                                <span class="w-24 text-on-surface-variant">{label.to_string()}</span>
                                                <div class="flex-1 mx-4 h-2 bg-surface-container rounded-full overflow-hidden">
                                                    <div class=format!("h-full rounded-full {}", color_class) style=format!("width: {}", width)></div>
                                                </div>
                                                <span class="w-10 text-right font-semibold font-mono text-on-surface">{count.to_string()}</span>
                                                <span class="w-8 text-right font-mono text-on-surface-variant/60">{pct.to_string()}</span>
                                            </div>
                                        }
                                    };
                                    view! {
                                        {dist_row("Outstanding", "265", "22%", "bg-emerald-500", "22%")}
                                        {dist_row("Above Bar", "457", "38%", "bg-emerald-400", "38%")}
                                        {dist_row("At Bar", "289", "24%", "bg-amber-400", "24%")}
                                        {dist_row("Below Bar", "146", "12%", "bg-orange-500", "12%")}
                                        {dist_row("Avoid", "47", "4%", "bg-red-500", "4%")}
                                    }
                                }
                            </div>
                        </div>

                        // Confidence Distribution
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-4">
                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Confidence Distribution"</h3>
                            
                            <div class="space-y-3">
                                {
                                    let conf_row = |label: &str, count: &str, color_class: &str, width: &str| {
                                        view! {
                                            <div class="flex items-center justify-between text-xs">
                                                <span class="w-24 text-on-surface-variant">{label.to_string()}</span>
                                                <div class="flex-1 mx-4 h-1.5 bg-surface-container rounded-full overflow-hidden">
                                                    <div class=format!("h-full rounded-full {}", color_class) style=format!("width: {}", width)></div>
                                                </div>
                                                <span class="w-10 text-right font-semibold font-mono text-on-surface">{count.to_string()}</span>
                                            </div>
                                        }
                                    };
                                    view! {
                                        {conf_row("Very High", "217", "bg-cobalt", "18%")}
                                        {conf_row("High", "385", "bg-cobalt-light", "32%")}
                                        {conf_row("Medium", "313", "bg-amber-500", "26%")}
                                        {conf_row("Low", "168", "bg-orange-500", "14%")}
                                        {conf_row("Insufficient", "121", "bg-gray-500", "10%")}
                                    }
                                }
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Modal: Calibration Settings ──
            <Show when=move || show_settings_modal.get()>
                <div class="fixed inset-0 z-[1000] flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
                    <div class="bg-surface-container border border-outline-variant/20 rounded-2xl p-6 max-w-md w-full shadow-2xl space-y-4">
                        <div>
                            <h3 class="text-lg font-bold text-on-surface">"Calibration Schedule Configuration"</h3>
                            <p class="text-xs text-on-surface-variant">"Alter recalculation and anomaly filters across evaluations."</p>
                        </div>
                        
                        <div class="space-y-4">
                            <div class="space-y-1">
                                <label class="text-xs font-semibold text-on-surface-variant">"Recalculation Interval"</label>
                                <select class="w-full bg-surface-container-high border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 cursor-pointer outline-none">
                                    <option>"Daily (midnight UTC)"</option>
                                    <option>"Weekly (Sunday midnight)"</option>
                                    <option>"On-demand updates only"</option>
                                </select>
                            </div>
                            <div class="space-y-1">
                                <label class="text-xs font-semibold text-on-surface-variant">"Anomaly Threshold (z-score)"</label>
                                <input type="number" step="0.1" class="w-full bg-surface-container-high border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 outline-none" value="2.0" />
                            </div>
                        </div>

                        <div class="flex justify-end gap-3 pt-4 border-t border-outline-variant/10">
                            <button 
                                class="btn-ghost px-4 py-2 rounded-lg text-sm font-semibold border border-outline-variant/30 hover:bg-surface-bright/20 transition-all"
                                on:click=move |_| show_settings_modal.set(false)
                            >
                                "Cancel"
                            </button>
                            <button 
                                class="btn-primary px-4 py-2 rounded-lg text-sm font-semibold transition-all"
                                on:click=move |_| {
                                    show_settings_modal.set(false);
                                    toast.show_toast("Calibration", "Calibration settings updated.", "success");
                                }
                            >
                                "Save Settings"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
