use leptos::prelude::*;

#[component]
pub fn ScorecardSession() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // Wizard step state: 1 to 6
    let current_step = RwSignal::new(1);
    
    // Ratings inputs
    let cleanliness_score = RwSignal::new(7.5);
    let communication_score = RwSignal::new(8.8);
    let _compliance_checked = RwSignal::new(vec![true, true, false]); // baño, cocina, etc.
    let financial_score = RwSignal::new(9.2);
    let _financial_checked = RwSignal::new(vec![true]);
    let evaluator_notes = RwSignal::new("Maria Silva has demonstrated excellent property cleanliness scores across Chicago STR units. Response times are well within acceptable limits. Background checks verified.".to_string());
    
    // Checklist inputs
    let _clean_check_1 = RwSignal::new(true);
    let _clean_check_2 = RwSignal::new(false);
    let conf_tier = RwSignal::new("Verified".to_string());
    let response_threshold = RwSignal::new("Under 15 minutes".to_string());

    let move_step = move |dir: i32| {
        let next = current_step.get() + dir;
        if next < 1 { return; }
        if next > 6 {
            // Submit scorecard session
            toast.show_toast("Scorecard", "Scorecard session submitted. Recalculating metrics...", "success");
            // Redirect to scorecards
            let window = web_sys::window().unwrap();
            let _ = window.location().set_href("/billing/scorecards");
            return;
        }
        current_step.set(next);
    };

    let step_label = move |step: i32| {
        match step {
            1 => "Cleanliness",
            2 => "Communication",
            3 => "Vendor Compliance",
            4 => "Financial Status",
            5 => "Calibration & Notes",
            _ => "Review & Submit",
        }
    };

    view! {
        <div class="space-y-6">
            // ── Breadcrumb ──
            <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                <a href="/billing/scorecards" class="hover:text-primary transition-colors">"Scorecards"</a>
                <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                <span class="text-primary/70">"Active Rating Session"</span>
            </nav>

            // ── Session Header ──
            <div class="bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                <h1 class="text-xl font-extrabold tracking-tight text-on-surface">"Lead Rating Session · Maria Silva"</h1>
                <p class="text-xs text-on-surface-variant mt-1">"Rater: Jamie Delaney · Scorecard: sc_maria_001 · Target: G-31 atlas_lead"</p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-4 gap-6">
                // ── Steps sidebar on the left ──
                <div class="bg-surface-container-low border border-outline-variant/20 p-6 rounded-xl shadow-sm h-fit space-y-4">
                    <h3 class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">"Progress"</h3>
                    
                    // Progress Bar
                    <div class="w-full h-1.5 bg-surface-container rounded-full overflow-hidden border border-outline-variant/10">
                        <div 
                            class="h-full bg-primary transition-all duration-300"
                            style=move || format!("width: {}%;", (current_step.get() * 100) / 6)
                        ></div>
                    </div>

                    // Step items list
                    <div class="space-y-3 pt-2">
                        {
                            (1..=6).map(|step| {
                                view! {
                                    <div 
                                        class=move || {
                                            if current_step.get() == step {
                                                "flex items-center gap-3 cursor-pointer text-xs font-bold text-primary"
                                            } else if current_step.get() > step {
                                                "flex items-center gap-3 cursor-pointer text-xs text-emerald-400 font-medium"
                                            } else {
                                                "flex items-center gap-3 cursor-pointer text-xs text-on-surface-variant/70 font-medium"
                                            }
                                        }
                                        on:click=move |_| current_step.set(step)
                                    >
                                        <span class=move || {
                                            if current_step.get() == step {
                                                "w-6 h-6 rounded-full border border-primary bg-primary/10 flex items-center justify-center font-bold font-mono text-[10px]"
                                            } else if current_step.get() > step {
                                                "w-6 h-6 rounded-full bg-emerald-500 text-white flex items-center justify-center font-bold font-mono text-[10px]"
                                            } else {
                                                "w-6 h-6 rounded-full border border-outline-variant/40 bg-surface-container flex items-center justify-center font-bold font-mono text-[10px]"
                                            }
                                        }>
                                            {if current_step.get() > step { "✓".to_string() } else { step.to_string() }}
                                        </span>
                                        <span>{step_label(step)}</span>
                                    </div>
                                }
                            }).collect_view()
                        }
                    </div>
                </div>

                // ── Rater card contents on the right ──
                <div class="md:col-span-3 bg-surface-container-low border border-outline-variant/20 rounded-xl shadow-sm flex flex-col justify-between min-h-[360px]">
                    <div class="p-6 border-b border-outline-variant/20 bg-surface-container-high/20 flex justify-between items-center">
                        <h3 class="text-sm font-bold text-on-surface">
                            {move || format!("Step {}: {}", current_step.get(), step_label(current_step.get()))}
                        </h3>
                        <span class="text-xs text-on-surface-variant/60 font-mono">
                            {move || format!("Step {} of 6", current_step.get())}
                        </span>
                    </div>

                    // Active Panes
                    <div class="p-6 flex-1">
                        // Pane 1: Cleanliness
                        <Show when=move || current_step.get() == 1>
                            <div class="space-y-6">
                                <div class="space-y-2">
                                    <div class="flex justify-between text-xs">
                                        <span class="font-semibold text-on-surface-variant">"Overall cleanliness score"</span>
                                        <span class="font-bold text-primary font-mono">{move || cleanliness_score.get()} " / 10"</span>
                                    </div>
                                    <input 
                                        type="range" 
                                        min="0" 
                                        max="100" 
                                        class="w-full bg-surface-container h-1.5 rounded-lg appearance-none cursor-pointer outline-none border border-outline-variant/20"
                                        prop:value=move || (cleanliness_score.get() * 10.0) as i32
                                        on:input=move |ev| {
                                            let val: f64 = event_target_value(&ev).parse().unwrap_or(0.0);
                                            cleanliness_score.set(val / 10.0);
                                        }
                                    />
                                </div>
                                <div class="space-y-2">
                                    <span class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"Confidence Tier"</span>
                                    <div class="grid grid-cols-2 gap-4">
                                        <div 
                                            class=move || if conf_tier.get() == "Verified" { "p-3 rounded-lg border border-primary bg-primary/5 cursor-pointer text-xs" } else { "p-3 rounded-lg border border-outline-variant/30 bg-surface-container cursor-pointer text-xs" }
                                            on:click=move |_| conf_tier.set("Verified".to_string())
                                        >
                                            <div class="font-bold">"Verified"</div>
                                            <div class="text-[10px] text-on-surface-variant/70 mt-1">"Direct audit by field agent."</div>
                                        </div>
                                        <div 
                                            class=move || if conf_tier.get() == "Inferred" { "p-3 rounded-lg border border-primary bg-primary/5 cursor-pointer text-xs" } else { "p-3 rounded-lg border border-outline-variant/30 bg-surface-container cursor-pointer text-xs" }
                                            on:click=move |_| conf_tier.set("Inferred".to_string())
                                        >
                                            <div class="font-bold">"Inferred"</div>
                                            <div class="text-[10px] text-on-surface-variant/70 mt-1">"AI metrics calculation."</div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </Show>

                        // Pane 2: Communication
                        <Show when=move || current_step.get() == 2>
                            <div class="space-y-6">
                                <div class="space-y-2">
                                    <div class="flex justify-between text-xs">
                                        <span class="font-semibold text-on-surface-variant">"Response rate / communication time"</span>
                                        <span class="font-bold text-primary font-mono">{move || communication_score.get()} " / 10"</span>
                                    </div>
                                    <input 
                                        type="range" 
                                        min="0" 
                                        max="100" 
                                        class="w-full bg-surface-container h-1.5 rounded-lg appearance-none cursor-pointer outline-none border border-outline-variant/20"
                                        prop:value=move || (communication_score.get() * 10.0) as i32
                                        on:input=move |ev| {
                                            let val: f64 = event_target_value(&ev).parse().unwrap_or(0.0);
                                            communication_score.set(val / 10.0);
                                        }
                                    />
                                </div>
                                <div class="space-y-2">
                                    <span class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"Average Response speed"</span>
                                    <div class="grid grid-cols-2 gap-4">
                                        <div 
                                            class=move || if response_threshold.get() == "Under 15 minutes" { "p-3 rounded-lg border border-primary bg-primary/5 cursor-pointer text-xs" } else { "p-3 rounded-lg border border-outline-variant/30 bg-surface-container cursor-pointer text-xs" }
                                            on:click=move |_| response_threshold.set("Under 15 minutes".to_string())
                                        >
                                            <div class="font-bold">"Under 15 minutes"</div>
                                            <div class="text-[10px] text-on-surface-variant/70 mt-1">"Outstanding response timing."</div>
                                        </div>
                                        <div 
                                            class=move || if response_threshold.get() == "15 - 60 minutes" { "p-3 rounded-lg border border-primary bg-primary/5 cursor-pointer text-xs" } else { "p-3 rounded-lg border border-outline-variant/30 bg-surface-container cursor-pointer text-xs" }
                                            on:click=move |_| response_threshold.set("15 - 60 minutes".to_string())
                                        >
                                            <div class="font-bold">"15 - 60 minutes"</div>
                                            <div class="text-[10px] text-on-surface-variant/70 mt-1">"Standard platform response rate."</div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </Show>

                        // Pane 3: Vendor Compliance
                        <Show when=move || current_step.get() == 3>
                            <div class="space-y-4 text-xs">
                                <span class="font-bold text-on-surface-variant uppercase tracking-wider">"Licensing & Regulatory Checks"</span>
                                <div class="space-y-2">
                                    <label class="flex items-center gap-3 bg-surface-container p-3 rounded-lg border border-outline-variant/20 cursor-pointer">
                                        <input type="checkbox" checked=true class="rounded border-outline-variant text-primary focus:ring-primary h-4 w-4" />
                                        <span>"Valid municipal STR registration matches city records"</span>
                                    </label>
                                    <label class="flex items-center gap-3 bg-surface-container p-3 rounded-lg border border-outline-variant/20 cursor-pointer">
                                        <input type="checkbox" checked=true class="rounded border-outline-variant text-primary focus:ring-primary h-4 w-4" />
                                        <span>"Background check on primary account holder PASSED"</span>
                                    </label>
                                    <label class="flex items-center gap-3 bg-surface-container p-3 rounded-lg border border-outline-variant/20 cursor-pointer">
                                        <input type="checkbox" checked=false class="rounded border-outline-variant text-primary focus:ring-primary h-4 w-4" />
                                        <span>"Insurance certificates uploaded and active in Vault"</span>
                                    </label>
                                </div>
                            </div>
                        </Show>

                        // Pane 4: Financial Status
                        <Show when=move || current_step.get() == 4>
                            <div class="space-y-6">
                                <div class="space-y-2">
                                    <div class="flex justify-between text-xs">
                                        <span class="font-semibold text-on-surface-variant">"Financial solvency score"</span>
                                        <span class="font-bold text-primary font-mono">{move || financial_score.get()} " / 10"</span>
                                    </div>
                                    <input 
                                        type="range" 
                                        min="0" 
                                        max="100" 
                                        class="w-full bg-surface-container h-1.5 rounded-lg appearance-none cursor-pointer outline-none border border-outline-variant/20"
                                        prop:value=move || (financial_score.get() * 10.0) as i32
                                        on:input=move |ev| {
                                            let val: f64 = event_target_value(&ev).parse().unwrap_or(0.0);
                                            financial_score.set(val / 10.0);
                                        }
                                    />
                                </div>
                                <div class="space-y-2 text-xs">
                                    <span class="font-bold text-on-surface-variant uppercase tracking-wider">"Outstanding Ledgers (G-03)"</span>
                                    <label class="flex items-center gap-3 bg-surface-container p-3 rounded-lg border border-outline-variant/20 cursor-pointer">
                                        <input type="checkbox" checked=true class="rounded border-outline-variant text-primary focus:ring-primary h-4 w-4" />
                                        <span>"No overdue billing invoices in past 6 months"</span>
                                    </label>
                                </div>
                            </div>
                        </Show>

                        // Pane 5: Notes
                        <Show when=move || current_step.get() == 5>
                            <div class="space-y-4">
                                <div class="space-y-1">
                                    <label class="text-xs font-semibold text-on-surface-variant">"Internal Evaluator Summary Notes"</label>
                                    <textarea 
                                        class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 h-36 resize-none focus:ring-1 focus:ring-primary focus:border-primary outline-none transition-all"
                                        on:input=move |ev| evaluator_notes.set(event_target_value(&ev))
                                    >
                                        {evaluator_notes.get_untracked()}
                                    </textarea>
                                </div>
                            </div>
                        </Show>

                        // Pane 6: Final Review
                        <Show when=move || current_step.get() == 6>
                            <div class="space-y-6 text-xs">
                                <p class="text-on-surface-variant">"Review the scores and compliance metrics before submitting."</p>
                                
                                <div class="grid grid-cols-2 gap-4">
                                    <div class="bg-surface-container p-4 rounded-lg border border-outline-variant/20 flex items-center justify-between">
                                        <div>
                                            <div class="text-[10px] text-on-surface-variant uppercase tracking-widest font-bold">"Cleanliness"</div>
                                            <div class="text-sm font-bold text-on-surface mt-1">{move || cleanliness_score.get()} " / 10"</div>
                                        </div>
                                    </div>
                                    <div class="bg-surface-container p-4 rounded-lg border border-outline-variant/20 flex items-center justify-between">
                                        <div>
                                            <div class="text-[10px] text-on-surface-variant uppercase tracking-widest font-bold">"Communication"</div>
                                            <div class="text-sm font-bold text-on-surface mt-1">{move || communication_score.get()} " / 10"</div>
                                        </div>
                                    </div>
                                    <div class="bg-surface-container p-4 rounded-lg border border-outline-variant/20 flex items-center justify-between">
                                        <div>
                                            <div class="text-[10px] text-on-surface-variant uppercase tracking-widest font-bold">"Compliance"</div>
                                            <div class="text-sm font-bold text-emerald-400 mt-1">"PASSED"</div>
                                        </div>
                                    </div>
                                    <div class="bg-surface-container p-4 rounded-lg border border-outline-variant/20 flex items-center justify-between">
                                        <div>
                                            <div class="text-[10px] text-on-surface-variant uppercase tracking-widest font-bold">"Financial Status"</div>
                                            <div class="text-sm font-bold text-on-surface mt-1">{move || financial_score.get()} " / 10"</div>
                                        </div>
                                    </div>
                                </div>

                                <div class="bg-surface-container p-4 rounded-lg border border-outline-variant/20 space-y-2">
                                    <div class="font-bold text-[10px] text-on-surface-variant uppercase tracking-widest">"Evaluator Notes"</div>
                                    <p class="text-on-surface-variant leading-relaxed italic">"\"" {move || evaluator_notes.get()} "\""</p>
                                </div>
                            </div>
                        </Show>
                    </div>

                    // Rater footer with back/next
                    <div class="p-4 border-t border-outline-variant/20 bg-surface-container-high/10 flex justify-between">
                        <button 
                            class=move || if current_step.get() == 1 { "invisible" } else { "btn-ghost px-4 py-2 rounded-lg text-sm font-semibold border border-outline-variant/30 hover:bg-surface-bright/20" }
                            on:click=move |_| move_step(-1)
                        >
                            "← Back"
                        </button>
                        <button 
                            class="btn-primary px-4 py-2 rounded-lg text-sm font-semibold shadow-md active:scale-95 transition-all"
                            on:click=move |_| move_step(1)
                        >
                            {move || if current_step.get() == 6 { "Submit Scorecard" } else { "Next Step →" }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
