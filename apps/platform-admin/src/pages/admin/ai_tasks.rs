use leptos::prelude::*;
use serde_json::json;

#[derive(Clone, Debug)]
pub struct AiTaskItem {
    pub id: String,
    pub task_type: String,
    pub entity: String,
    pub status: RwSignal<String>,
    pub status_class: RwSignal<&'static str>,
    pub runtime: RwSignal<String>,
    pub tokens: RwSignal<String>,
    pub completed: RwSignal<String>,
    pub model: String,
    pub params: serde_json::Value,
    pub initial_logs: Vec<String>,
    pub streamable: bool,
}

#[component]
pub fn AiTasks() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // Global Queue Pause state
    let queue_paused = RwSignal::new(false);
    
    // Tasks list database state
    let tasks_data = RwSignal::new(vec![
        AiTaskItem {
            id: "ait_e19fca6a".to_string(),
            task_type: "fmcsa_lead_scoring".to_string(),
            entity: "Ruud Logistics Corp (Lead)".to_string(),
            status: RwSignal::new("Running".to_string()),
            status_class: RwSignal::new("bg-blue-500/10 border-blue-500/30 text-blue-400"),
            runtime: RwSignal::new("1.82s".to_string()),
            tokens: RwSignal::new("642".to_string()),
            completed: RwSignal::new("Just now".to_string()),
            model: "gpt-4o-mini".to_string(),
            params: json!({ "lead_id": "l_ruud_992", "target": "lead_score_card", "run_index": 2 }),
            initial_logs: vec![
                "[INFO] Initializing agent context...".to_string(),
                "[INFO] Resolving FMCSA DOT registry ID: 44219A".to_string(),
                "[INFO] Fetched carrier details: 42 trucks, 22 drivers, 91.2% safety rating.".to_string(),
                "[INFO] Triggering GPT-4o score vector compilation...".to_string(),
            ],
            streamable: true,
        },
        AiTaskItem {
            id: "ait_ed8c0441".to_string(),
            task_type: "scorecard_aggregation".to_string(),
            entity: "Nexus Property Group (Account)".to_string(),
            status: RwSignal::new("Queued".to_string()),
            status_class: RwSignal::new("bg-slate-500/10 border-slate-500/30 text-slate-400"),
            runtime: RwSignal::new("—".to_string()),
            tokens: RwSignal::new("—".to_string()),
            completed: RwSignal::new("Pending".to_string()),
            model: "gpt-4o".to_string(),
            params: json!({ "account_id": "acc_nexus_88ea", "target": "recalculate_g27_split" }),
            initial_logs: vec![
                "[QUEUE] Task placed in priority queue by cron outbox sweeper.".to_string(),
                "[QUEUE] Waiting for available runner thread in pool atlas-llm-pool-04...".to_string(),
                "[QUEUE] Awaiting compliance checkpoint locks.".to_string(),
            ],
            streamable: false,
        },
        AiTaskItem {
            id: "ait_ad9043a1".to_string(),
            task_type: "auto_compliance_check".to_string(),
            entity: "João Carlos Silva (Lead)".to_string(),
            status: RwSignal::new("Success".to_string()),
            status_class: RwSignal::new("bg-emerald-500/10 border-emerald-500/30 text-emerald-400"),
            runtime: RwSignal::new("842ms".to_string()),
            tokens: RwSignal::new("4,204".to_string()),
            completed: RwSignal::new("4 mins ago".to_string()),
            model: "gpt-4o-mini".to_string(),
            params: json!({ "lead_id": "l_joao_cs", "ruleset": "brazil_cpf_legal_status" }),
            initial_logs: vec![
                "[INFO] Running CNPJ registration checks on João Carlos Silva...".to_string(),
                "[INFO] CPF check status returned PASSED from state database query.".to_string(),
                "[SUCCESS] Compliance check completed successfully. Generated score: 9.4.".to_string(),
            ],
            streamable: false,
        },
        AiTaskItem {
            id: "ait_bd7affaa".to_string(),
            task_type: "vector_similarity_search".to_string(),
            entity: "Biscayne STR Co. (Account)".to_string(),
            status: RwSignal::new("Success".to_string()),
            status_class: RwSignal::new("bg-emerald-500/10 border-emerald-500/30 text-emerald-400"),
            runtime: RwSignal::new("224ms".to_string()),
            tokens: RwSignal::new("820".to_string()),
            completed: RwSignal::new("12 mins ago".to_string()),
            model: "text-embedding-3-small".to_string(),
            params: json!({ "vector_dimension": 1536, "limit": 5, "min_confidence": 0.82 }),
            initial_logs: vec![
                "[INFO] Initializing similarity vector space embedding comparison...".to_string(),
                "[INFO] Found 3 matching records in tenant scope. Confidence: 0.94.".to_string(),
                "[SUCCESS] Task executed in 224ms. No data leakage flags detected.".to_string(),
            ],
            streamable: false,
        },
        AiTaskItem {
            id: "ait_005c6922".to_string(),
            task_type: "sentiment_analysis".to_string(),
            entity: "Vizcaya STR Partners (Account)".to_string(),
            status: RwSignal::new("Failed".to_string()),
            status_class: RwSignal::new("bg-red-500/10 border-red-500/30 text-red-400"),
            runtime: RwSignal::new("3.44s".to_string()),
            tokens: RwSignal::new("1,120".to_string()),
            completed: RwSignal::new("2 hours ago".to_string()),
            model: "gpt-4o".to_string(),
            params: json!({ "account_id": "acc_vizcaya_01", "source_logs": "support_tickets" }),
            initial_logs: vec![
                "[INFO] Initializing ticket log scraper context...".to_string(),
                "[WARNING] API Connection timed out after 3 retries.".to_string(),
                "[ERROR] OpenAI Connection Error: API connection failed after 3 retries (502 Bad Gateway).".to_string(),
                "[ERROR] Retrying outbox worker task aborted. Job re-scheduled for outbox retry interval in 120s.".to_string(),
            ],
            streamable: false,
        },
    ]);

    // Active Selection & Filter state
    let active_filter = RwSignal::new("all".to_string());
    let search_query = RwSignal::new("".to_string());
    let selected_task_id = RwSignal::new(None::<String>);
    let detail_tab = RwSignal::new("console".to_string());
    
    // Console log lines state (can append dynamically during streaming)
    let console_logs = RwSignal::new(Vec::<String>::new());
    let streaming_active = RwSignal::new(false);
    
    // Find active selection item
    let selected_item = Signal::derive(move || {
        let sid = selected_task_id.get();
        sid.and_then(|id| {
            tasks_data.get().iter().find(|t| t.id == id).cloned()
        })
    });

    // Handle selection change
    let select_task = move |task_id: String| {
        if let Some(t) = tasks_data.get().iter().find(|item| item.id == task_id) {
            selected_task_id.set(Some(task_id.clone()));
            console_logs.set(t.initial_logs.clone());
            detail_tab.set("console".to_string());
            
            // If the selected task is Running and streamable, simulate real-time log streaming
            if t.status.get() == "Running" && t.streamable {
                streaming_active.set(true);
                let logs_signal = console_logs;
                let active_status = t.status;
                let active_class = t.status_class;
                let active_runtime = t.runtime;
                let active_completed = t.completed;
                let t_id = task_id.clone();
                let toast_clone = toast.clone();
                
                leptos::task::spawn_local(async move {
                    let stream_lines = vec![
                        "[INFO] Executing prompt formatting pipeline...".to_string(),
                        "[INFO] Contact score weights calculated: Cleanliness 9.3, Compliance 9.6.".to_string(),
                        "[INFO] Executing aggregate calibration checker...".to_string(),
                        "[SUCCESS] Generated final scorecard vector output safely.".to_string(),
                        "[INFO] Saving state to database...".to_string(),
                        "[SUCCESS] Task execution completed in 4.22s. Logs closed.".to_string(),
                    ];
                    
                    for line in stream_lines {
                        // Check if selection changed or streaming aborted
                        if selected_task_id.get_untracked() != Some(t_id.clone()) || !streaming_active.get_untracked() {
                            return;
                        }
                        gloo_timers::future::TimeoutFuture::new(1000).await;
                        logs_signal.update(|logs| logs.push(line));
                    }
                    
                    if selected_task_id.get_untracked() == Some(t_id) && streaming_active.get_untracked() {
                        streaming_active.set(false);
                        active_status.set("Success".to_string());
                        active_class.set("bg-emerald-500/10 border-emerald-500/30 text-emerald-400");
                        active_runtime.set("4.22s".to_string());
                        active_completed.set("Just now".to_string());
                        toast_clone.show_toast("Success", "Task completed successfully.", "success");
                    }
                });
            } else {
                streaming_active.set(false);
            }
        }
    };

    // Actions
    let handle_abort = move |task: AiTaskItem| {
        streaming_active.set(false);
        task.status.set("Failed".to_string());
        task.status_class.set("bg-red-500/10 border-red-500/30 text-red-400");
        task.runtime.set("Stopped".to_string());
        console_logs.update(|logs| logs.push("[ABORTED] Task manually terminated by Super-Admin.".to_string()));
        toast.show_toast("Error", "Task execution terminated.", "error");
    };

    let handle_rerun = move |task: AiTaskItem| {
        task.status.set("Running".to_string());
        task.status_class.set("bg-blue-500/10 border-blue-500/30 text-blue-400");
        task.runtime.set("1.82s".to_string());
        task.completed.set("Just now".to_string());
        toast.show_toast("Info", "Task re-enqueued for execution.", "info");
        select_task(task.id.clone());
    };

    let toggle_pause = move |_| {
        queue_paused.update(|p| *p = !*p);
        if queue_paused.get() {
            toast.show_toast("Warning", "Background task scheduler PAUSED.", "warn");
        } else {
            toast.show_toast("Success", "Background task scheduler RESUMED.", "success");
        }
    };

    let retry_all_failed = move |_| {
        let mut count = 0;
        for t in tasks_data.get() {
            if t.status.get() == "Failed" {
                t.status.set("Queued".to_string());
                t.status_class.set("bg-slate-500/10 border-slate-500/30 text-slate-400");
                t.runtime.set("—".to_string());
                t.tokens.set("—".to_string());
                t.completed.set("Pending".to_string());
                count += 1;
            }
        }
        if count > 0 {
            toast.show_toast("Success", &format!("Re-enqueued {} failed tasks.", count), "success");
        } else {
            toast.show_toast("Info", "No failed tasks found.", "info");
        }
    };

    // Filter list
    let filtered_tasks = Signal::derive(move || {
        let filter = active_filter.get();
        let query = search_query.get().to_lowercase();
        
        tasks_data.get().into_iter().filter(|t| {
            let matches_status = filter == "all" || t.status.get() == filter;
            let matches_query = query.is_empty() 
                || t.id.to_lowercase().contains(&query)
                || t.task_type.to_lowercase().contains(&query)
                || t.entity.to_lowercase().contains(&query);
            matches_status && matches_query
        }).collect::<Vec<_>>()
    });

    view! {
        <div class="max-w-6xl mx-auto space-y-8 animate-in slide-in-from-bottom-4 duration-500 ease-out fade-in">
            // Header
            <header class="flex justify-between items-center bg-surface-container border border-outline-variant/10 p-6 rounded-2xl shadow-sm">
                <div>
                    <h1 class="text-3xl font-light tracking-tight text-on-surface mb-2 font-headline">"AI Task Monitor"</h1>
                    <p class="text-on-surface-variant text-sm tracking-wide">"Background processor metrics, completions, and prompt logs · G-08"</p>
                </div>
                <div class="flex gap-3">
                    <button 
                        on:click=toggle_pause
                        class=move || if queue_paused.get() { "px-4 py-2 text-sm font-semibold rounded-lg bg-primary text-on-primary hover:opacity-90 active:scale-95 transition-all shadow-md" } else { "px-4 py-2 text-sm font-semibold rounded-lg bg-[#05183c] border border-outline-variant/30 text-[#91aaeb] hover:bg-[#05183c]/60 active:scale-95 transition-all shadow-sm" }
                    >
                        {move || if queue_paused.get() { "Resume Queue" } else { "Pause Queue" }}
                    </button>
                    <button 
                        on:click=retry_all_failed
                        class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-bold text-on-primary shadow-lg shadow-primary/20 hover:scale-105 active:scale-95 transition-all"
                    >
                        "Retry Failed Tasks"
                    </button>
                </div>
            </header>

            // KPI Grid
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Active / Queued Tasks"</span>
                    <span class="text-3xl font-bold font-mono text-on-surface">"4"</span>
                    <span class="text-xs text-success">"3.5% load increase"</span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Success Rate (24h)"</span>
                    <span class="text-3xl font-bold font-mono text-success">"99.24%"</span>
                    <span class="text-xs text-success">"↑ 0.12% vs yesterday"</span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Avg Runtime"</span>
                    <span class="text-3xl font-bold font-mono text-on-surface">"1.42s"</span>
                    <span class="text-xs text-success">"↓ 0.08s faster"</span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Daily Token Usage"</span>
                    <span class="text-3xl font-bold font-mono text-on-surface">"1,248k"</span>
                    <span class="text-xs text-error">"↑ 14% vs avg"</span>
                </div>
            </div>

            // Filter & Search bar
            <div class="flex items-center justify-between gap-4 bg-surface-container border border-outline-variant/10 p-3 rounded-xl">
                <div class="flex items-center gap-1.5 overflow-x-auto">
                    {
                        let filter_pill = move |status_id: &str, label: &str| {
                            let sid = status_id.to_string();
                            let lbl = label.to_string();
                            let active_cls = sid.clone();
                            let click_cls = sid.clone();
                            view! {
                                <button 
                                    on:click=move |_| active_filter.set(click_cls.clone())
                                    class=move || if active_filter.get() == active_cls { "px-3 py-1.5 text-xs font-semibold rounded bg-[#05183c] text-[#7bd0ff] border border-[#7bd0ff]/20 shrink-0" } else { "px-3 py-1.5 text-xs font-semibold rounded text-[#91aaeb] hover:bg-[#05183c]/30 hover:text-[#dee5ff] transition-all shrink-0 bg-transparent border border-transparent" }
                                >
                                    {lbl.clone()}
                                </button>
                            }
                        };
                        view! {
                            {filter_pill("all", "All")}
                            {filter_pill("Running", "Running")}
                            {filter_pill("Queued", "Queued")}
                            {filter_pill("Success", "Success")}
                            {filter_pill("Failed", "Failed")}
                        }
                    }
                </div>
                <div class="relative shrink-0 w-64">
                    <span class="material-symbols-outlined absolute left-3 top-2.5 text-[#91aaeb]/60 text-sm">"search"</span>
                    <input 
                        type="text" 
                        placeholder="Search key, entity..." 
                        class="w-full bg-[#05183c] border border-outline-variant/30 text-on-surface text-xs rounded-lg pl-8 pr-3 py-2 focus:ring-1 focus:ring-primary focus:border-primary transition-all placeholder:text-[#91aaeb]/40"
                        on:input=move |ev| search_query.set(event_target_value(&ev))
                        prop:value=search_query
                    />
                </div>
            </div>

            // Table Grid
            <div class="bg-surface border border-outline-variant/10 rounded-xl overflow-hidden">
                <div class="overflow-x-auto w-full">
                    <table class="w-full text-left text-sm whitespace-nowrap">
                        <thead class="bg-surface-container-highest/60 text-[#91aaeb] text-xs font-medium uppercase tracking-wider">
                            <tr>
                                <th class="px-6 py-4">"Task ID"</th>
                                <th class="px-6 py-4">"Type"</th>
                                <th class="px-6 py-4">"Target Entity"</th>
                                <th class="px-6 py-4">"Status"</th>
                                <th class="px-6 py-4">"Runtime"</th>
                                <th class="px-6 py-4">"Tokens Used"</th>
                                <th class="px-6 py-4">"Completed"</th>
                                <th class="px-6 py-4"></th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-outline-variant/10 text-on-surface">
                            <For 
                                each=move || filtered_tasks.get()
                                key=|t| t.id.clone() 
                                children=move |t| {
                                    let tid = t.id.clone();
                                    let tid_click = t.id.clone();
                                    let is_selected = Signal::derive(move || selected_task_id.get() == Some(tid.clone()));
                                    
                                    view! {
                                        <tr 
                                            on:click=move |_| select_task(tid_click.clone())
                                            class=move || if is_selected.get() { "hover:bg-surface-bright/5 cursor-pointer bg-primary-container/10 border-l-2 border-primary" } else { "hover:bg-surface-bright/5 cursor-pointer" }
                                        >
                                            <td class="px-6 py-4 font-mono text-xs">{t.id.clone()}</td>
                                            <td class="px-6 py-4 font-semibold text-primary">{t.task_type.clone()}</td>
                                            <td class="px-6 py-4 max-w-xs truncate">{t.entity.clone()}</td>
                                            <td class="px-6 py-4">
                                                <span class=move || format!("px-2 py-0.5 rounded text-[10px] uppercase font-bold border {}", t.status_class.get())>
                                                    {move || t.status.get()}
                                                </span>
                                            </td>
                                            <td class="px-6 py-4 font-mono text-xs text-on-surface-variant">{move || t.runtime.get()}</td>
                                            <td class="px-6 py-4 font-mono text-xs text-on-surface-variant">{move || t.tokens.get()}</td>
                                            <td class="px-6 py-4 text-xs text-on-surface-variant">{move || t.completed.get()}</td>
                                            <td class="px-6 py-4 text-right">
                                                <button 
                                                    on:click=move |e| { e.stop_propagation(); select_task(t.id.clone()); }
                                                    class="px-2.5 py-1 text-xs bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 rounded transition-all"
                                                >
                                                    "Inspect"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </div>
            </div>

            // Sliding drawer details panel
            <div 
                class=move || if selected_task_id.get().is_some() { "panel-backdrop open" } else { "panel-backdrop" }
                on:click=move |_| selected_task_id.set(None)
                style="position: fixed; inset: 0; background: rgba(0,0,0,0.5); z-index: 300; opacity: 0; pointer-events: none; transition: opacity 0.2s;"
            ></div>
            <div 
                class=move || if selected_task_id.get().is_some() { "detail-panel open" } else { "detail-panel" }
                style="position: fixed; top: 0; right: -560px; width: 560px; height: 100vh; background: #111520; border-left: 1px solid rgba(255,255,255,0.08); z-index: 400; display: flex; flex-direction: column; transition: right 0.24s cubic-bezier(0.25, 0.46, 0.45, 0.94); overflow: hidden;"
            >
                {move || selected_item.get().map(|item| {
                    let item_val = StoredValue::new(item);
                    let is_running = Signal::derive(move || item_val.with_value(|v| v.status.get() == "Running"));
                    
                    view! {
                        <div class="panel-header" style="padding: 16px 20px 0; border-bottom: 1px solid rgba(255,255,255,0.08); flex-shrink: 0;">
                            <div class="panel-header-top" style="display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; margin-bottom: 12px;">
                                <div class="panel-identity" style="flex: 1; min-width: 0;">
                                    <div class="panel-title-text font-mono" style="font-size: 18px; font-weight: 700; color: #E8EAF0;">{item_val.with_value(|v| v.id.clone())}</div>
                                    <div class="panel-subtitle-text" style="font-size: 12.5px; color: #8B92A8; margin-top: 3px;">{item_val.with_value(|v| v.task_type.clone())} " · " {item_val.with_value(|v| v.entity.clone())}</div>
                                </div>
                                <button 
                                    class="panel-close" 
                                    on:click=move |_| selected_task_id.set(None)
                                    style="width: 28px; height: 28px; border-radius: 5px; border: 1px solid rgba(255,255,255,0.08); background: transparent; color: #525A72; cursor: pointer; display: flex; align-items: center; justify-content: center; transition: all 0.12s;"
                                >
                                    "✕"
                                </button>
                            </div>
                            
                            <div class="panel-actions" style="display: flex; align-items: center; gap: 6px; padding-bottom: 12px;">
                                {move || if is_running.get() {
                                    view! {
                                        <button 
                                            on:click=move |_| handle_abort(item_val.get_value())
                                            class="px-3 py-1.5 text-xs font-semibold bg-red-600/20 border border-red-500/30 text-red-400 rounded-lg hover:bg-red-600/30 transition-colors"
                                        >
                                            "Abort Task"
                                        </button>
                                    }.into_any()
                                } else {
                                    view! {
                                        <button 
                                            on:click=move |_| handle_rerun(item_val.get_value())
                                            class="btn-primary-gradient px-3 py-1.5 text-xs font-bold text-on-primary rounded-lg shadow-sm"
                                        >
                                            "Rerun Task"
                                        </button>
                                    }.into_any()
                                }}
                            </div>

                            <div class="panel-tabs" style="display: flex; gap: 0; border-bottom: 1px solid rgba(255,255,255,0.08); padding: 0 20px;">
                                <button 
                                    class=move || if detail_tab.get() == "console" { "panel-tab active" } else { "panel-tab" }
                                    on:click=move |_| detail_tab.set("console".to_string())
                                    style="padding: 10px 12px; font-size: 12px; font-weight: 450; color: #8B92A8; cursor: pointer; border-bottom: 2px solid transparent; border-top:none; border-left:none; border-right:none; background:none; font-family:inherit;"
                                >
                                    "Console Output Logs"
                                </button>
                                <button 
                                    class=move || if detail_tab.get() == "details" { "panel-tab active" } else { "panel-tab" }
                                    on:click=move |_| detail_tab.set("details".to_string())
                                    style="padding: 10px 12px; font-size: 12px; font-weight: 450; color: #8B92A8; cursor: pointer; border-bottom: 2px solid transparent; border-top:none; border-left:none; border-right:none; background:none; font-family:inherit;"
                                >
                                    "Details & Metadata"
                                </button>
                            </div>
                        </div>

                        <div class="panel-content" style="flex: 1; overflow-y: auto; padding: 16px 20px;">
                            <Show when=move || detail_tab.get() == "console">
                                <div class="terminal-box" style="background: #05070B; border: 1.5px solid rgba(255,255,255,0.14); border-radius: 6px; padding: 14px; font-family: monospace; font-size: 11.5px; color: #39FF14; line-height: 1.6; height: 320px; overflow-y: auto;">
                                    <For 
                                        each=move || console_logs.get() 
                                        key=|line| line.clone() 
                                        children=move |line| {
                                            let cls = if line.contains("[SUCCESS]") { "text-emerald-400" }
                                                else if line.contains("[WARNING]") { "text-amber-400" }
                                                else if line.contains("[ERROR]") || line.contains("[ABORTED]") { "text-red-400" }
                                                else if line.contains("[INFO]") { "text-sky-400" }
                                                else { "text-emerald-400" };
                                            view! { <div class=cls>{line}</div> }
                                        }
                                    />
                                    <Show when=move || streaming_active.get()>
                                        <div class="inline-block w-1.5 h-3 bg-[#39FF14] animate-pulse"></div>
                                    </Show>
                                </div>
                                <div style="font-size:11px; color:#525A72; display:flex; justify-content:space-between">
                                    <span>"Encoding: UTF-8 · Host: atlas-llm-pool-04"</span>
                                    <Show when=move || streaming_active.get()>
                                        <span class="text-emerald-400 animate-pulse">"● Log Streaming Active"</span>
                                    </Show>
                                </div>
                            </Show>

                            <Show when=move || detail_tab.get() == "details">
                                <div class="grid grid-cols-2 gap-y-4 gap-x-8 text-sm">
                                    <div class="col-span-2 text-[10px] font-bold text-[#8B92A8] uppercase tracking-widest border-b border-white/5 pb-2">"Execution Metadata"</div>
                                    
                                    <div class="space-y-1">
                                        <span class="text-xs text-[#8B92A8]">"Model Context"</span>
                                        <p class="font-medium text-[#E8EAF0]">{item_val.with_value(|v| v.model.clone())}</p>
                                    </div>
                                    <div class="space-y-1">
                                        <span class="text-xs text-[#8B92A8]">"Runtime Latency"</span>
                                        <p class="font-mono text-xs text-[#E8EAF0]">{move || item_val.with_value(|v| v.runtime.get())}</p>
                                    </div>
                                    <div class="space-y-1">
                                        <span class="text-xs text-[#8B92A8]">"Context Tokens"</span>
                                        <p class="font-mono text-xs text-[#E8EAF0]">{move || item_val.with_value(|v| v.tokens.get())}</p>
                                    </div>
                                    <div class="space-y-1">
                                        <span class="text-xs text-[#8B92A8]">"Trigger Source"</span>
                                        <p class="font-medium text-[#E8EAF0]">"OutboxWorker queue"</p>
                                    </div>
                                    
                                    <div class="col-span-2 text-[10px] font-bold text-[#8B92A8] uppercase tracking-widest border-b border-white/5 pb-2 mt-4">"Parameters"</div>
                                    <div class="col-span-2">
                                        <span class="text-xs text-[#8B92A8] block mb-2">"JSON Payload"</span>
                                        <pre style="font-family:monospace; font-size:11px; background:#1C2236; padding:12px; border-radius:6px; color:#8B92A8; overflow-x:auto;">
                                            {item_val.with_value(|v| serde_json::to_string_pretty(&v.params).unwrap_or_default())}
                                        </pre>
                                    </div>
                                </div>
                            </Show>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
