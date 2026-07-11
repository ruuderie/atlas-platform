use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct AiTaskItem {
    pub id: String,
    pub task_type: String,
    pub entity: String,
    pub status: RwSignal<String>,
    pub status_class: RwSignal<String>,
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

    // Global Queue Pause state — hydrated from API
    let queue_paused = RwSignal::new(false);
    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(status) = crate::api::admin::get_ai_queue_status().await {
                queue_paused.set(status.paused);
            }
        });
    });

    // Tasks list database state
    let tasks_trigger = RwSignal::new(0);
    let tasks_res = LocalResource::new(move || {
        tasks_trigger.get();
        async move {
            let list = crate::api::admin::get_ai_tasks().await.unwrap_or_default();
            let items: Vec<AiTaskItem> = list
                .into_iter()
                .map(|t| AiTaskItem {
                    id: t.id,
                    task_type: t.task_type,
                    entity: t.entity,
                    status: RwSignal::new(t.status),
                    status_class: RwSignal::new(t.status_class),
                    runtime: RwSignal::new(t.runtime),
                    tokens: RwSignal::new(t.tokens),
                    completed: RwSignal::new(t.completed),
                    model: t.model,
                    params: t.params,
                    initial_logs: t.initial_logs,
                    streamable: t.streamable,
                })
                .collect();
            items
        }
    });

    let tasks_data = RwSignal::new(Vec::<AiTaskItem>::new());
    let tasks_loading = RwSignal::new(true);
    Effect::new(move |_| {
        if let Some(list) = tasks_res.get() {
            tasks_data.set(list);
            tasks_loading.set(false);
        }
    });

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
        sid.and_then(|id| tasks_data.get().iter().find(|t| t.id == id).cloned())
    });

    // Handle selection change
    let select_task = move |task_id: String| {
        if let Some(t) = tasks_data.get().iter().find(|item| item.id == task_id) {
            selected_task_id.set(Some(task_id.clone()));
            console_logs.set(t.initial_logs.clone());
            detail_tab.set("console".to_string());

            // If the selected task is Running and streamable, poll the logs endpoint every 2s
            if t.status.get() == "Running" && t.streamable {
                streaming_active.set(true);
                let logs_signal = console_logs;
                let t_id = task_id.clone();
                let toast_clone = toast.clone();

                leptos::task::spawn_local(async move {
                    // Poll every 2s while still active
                    loop {
                        // Stop if selection changed or streaming was aborted
                        if selected_task_id.get_untracked() != Some(t_id.clone())
                            || !streaming_active.get_untracked()
                        {
                            return;
                        }

                        match crate::api::billing::get_ai_task_logs(&t_id).await {
                            Ok(lines) => {
                                logs_signal.set(lines.clone());
                                // If the last log line signals completion, stop polling
                                let done = lines.iter().any(|l| {
                                    l.contains("[SUCCESS] Task execution completed")
                                        || l.contains("[ERROR]")
                                        || l.contains("[ABORT]")
                                });
                                if done {
                                    streaming_active.set(false);
                                    toast_clone.show_toast("Done", "Task completed.", "success");
                                    return;
                                }
                            }
                            Err(_) => {
                                // Network error — keep trying, just don't panic
                            }
                        }

                        gloo_timers::future::TimeoutFuture::new(2000).await;
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
        let t_toast = toast.clone();
        let task_id = task.id.clone();
        leptos::task::spawn_local(async move {
            match crate::api::admin::abort_ai_task(task_id).await {
                Ok(_) => {
                    tasks_trigger.set(tasks_trigger.get() + 1);
                    t_toast.show_toast("Success", "Task execution terminated.", "success");
                }
                Err(e) => {
                    t_toast.show_toast("Error", &format!("Failed to abort: {}", e), "error");
                }
            }
        });
    };

    let handle_rerun = move |task: AiTaskItem| {
        let t_toast = toast.clone();
        let task_id = task.id.clone();
        let select_id = task_id.clone();
        leptos::task::spawn_local(async move {
            match crate::api::admin::rerun_ai_task(task_id).await {
                Ok(_) => {
                    tasks_trigger.set(tasks_trigger.get() + 1);
                    t_toast.show_toast("Info", "Task re-enqueued for execution.", "info");
                    select_task(select_id);
                }
                Err(e) => {
                    t_toast.show_toast("Error", &format!("Failed to rerun: {}", e), "error");
                }
            }
        });
    };

    let toggle_pause = move |_| {
        let t_toast = toast.clone();
        let currently_paused = queue_paused.get();
        leptos::task::spawn_local(async move {
            let result = if currently_paused {
                crate::api::admin::resume_ai_queue().await
            } else {
                crate::api::admin::pause_ai_queue().await
            };
            match result {
                Ok(status) => {
                    queue_paused.set(status.paused);
                    if status.paused {
                        t_toast.show_toast("Warning", "Background task scheduler PAUSED.", "warn");
                    } else {
                        t_toast.show_toast(
                            "Success",
                            "Background task scheduler RESUMED.",
                            "success",
                        );
                    }
                }
                Err(e) => {
                    t_toast.show_toast("Error", &format!("Queue control failed: {}", e), "error");
                }
            }
        });
    };

    let retry_all_failed = move |_| {
        let t_toast = toast.clone();
        let tasks = tasks_data.get_untracked();
        leptos::task::spawn_local(async move {
            let mut count = 0;
            for t in tasks {
                if t.status.get_untracked() == "Failed" {
                    if crate::api::admin::rerun_ai_task(t.id).await.is_ok() {
                        count += 1;
                    }
                }
            }
            if count > 0 {
                tasks_trigger.set(tasks_trigger.get() + 1);
                t_toast.show_toast(
                    "Success",
                    &format!("Re-enqueued {} failed tasks.", count),
                    "success",
                );
            } else {
                t_toast.show_toast("Info", "No failed tasks found.", "info");
            }
        });
    };

    // Filter list
    let filtered_tasks = Signal::derive(move || {
        let filter = active_filter.get();
        let query = search_query.get().to_lowercase();

        tasks_data
            .get()
            .into_iter()
            .filter(|t| {
                let matches_status = filter == "all" || t.status.get() == filter;
                let matches_query = query.is_empty()
                    || t.id.to_lowercase().contains(&query)
                    || t.task_type.to_lowercase().contains(&query)
                    || t.entity.to_lowercase().contains(&query);
                matches_status && matches_query
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class="main-canvas">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"AI Task Monitor"</h1>
                    <p class="page-subtitle">"Background processor metrics, completions, and prompt logs · G-08"</p>
                </div>
                <div class="page-actions">
                    <button
                        on:click=toggle_pause
                        class=move || if queue_paused.get() { "btn btn-primary" } else { "btn btn-ghost" }
                    >
                        {move || if queue_paused.get() { "Resume Queue" } else { "Pause Queue" }}
                    </button>
                    <button on:click=retry_all_failed class="btn btn-primary">"Retry Failed"</button>
                </div>
            </div>

            // ── KPI Row ──
            <div class="kpi-row">
                <div class="kpi-card">
                    <div class="kpi-label">"Active / Queued Tasks"</div>
                    <div class="kpi-value mono">
                        {move || {
                            let count = tasks_data.get().iter()
                                .filter(|t| t.status.get() == "Running" || t.status.get() == "Queued")
                                .count();
                            count.to_string()
                        }}
                    </div>
                    <div class="kpi-delta">"Live from queue"</div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Completed (session)"</div>
                    <div class="kpi-value mono" style="color:var(--green)">
                        {move || {
                            tasks_data.get().iter()
                                .filter(|t| t.status.get() == "Success")
                                .count()
                                .to_string()
                        }}
                    </div>
                    <div class="kpi-delta up">"Tasks completed"</div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Failed (session)"</div>
                    <div class="kpi-value mono" style="color:var(--red)">
                        {move || {
                            tasks_data.get().iter()
                                .filter(|t| t.status.get() == "Failed")
                                .count()
                                .to_string()
                        }}
                    </div>
                    <div class="kpi-delta">"Tasks with errors"</div>
                </div>
                <div class="kpi-card">
                    <div class="kpi-label">"Total Tasks"</div>
                    <div class="kpi-value mono">
                        {move || tasks_data.get().len().to_string()}
                    </div>
                    <div class="kpi-delta">"Loaded this session"</div>
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
                                    class=move || if active_filter.get() == active_cls { "pill active" } else { "pill" }
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
                    <span class="material-symbols-outlined absolute left-3 top-2.5 text-on-surface-variant/60 text-sm">"search"</span>
                    <input
                        type="text"
                        placeholder="Search key, entity..."
                        class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-xs rounded-lg pl-8 pr-3 py-2 focus:ring-1 focus:ring-primary focus:border-primary transition-all placeholder:text-on-surface-variant/40"
                        on:input=move |ev| search_query.set(event_target_value(&ev))
                        prop:value=search_query
                    />
                </div>
            </div>

            // Table Grid
            <div class="bg-surface border border-outline-variant/10 rounded-xl overflow-hidden">
                <div class="overflow-x-auto w-full">
                    <table class="w-full text-left text-sm whitespace-nowrap">
                        <thead class="bg-surface-container-highest/60 text-on-surface-variant text-xs font-medium uppercase tracking-wider">
                            <tr>
                                <th class="px-6 py-4 col-hide-mobile">"Task ID"</th>
                                <th class="px-6 py-4">"Type"</th>
                                <th class="px-6 py-4">"Target Entity"</th>
                                <th class="px-6 py-4">"Status"</th>
                                <th class="px-6 py-4 col-hide-mobile">"Runtime"</th>
                                <th class="px-6 py-4 col-hide-mobile">"Tokens Used"</th>
                                <th class="px-6 py-4 col-hide-mobile">"Completed"</th>
                                <th class="px-6 py-4"></th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-outline-variant/10 text-on-surface">
                            {move || {
                                // ── Loading skeleton ──────────────────────────
                                if tasks_loading.get() {
                                    return (0..5u8).map(|_| view! {
                                        <tr class="animate-pulse">
                                            <td class="px-6 py-4 col-hide-mobile">
                                                <div class="h-3 bg-surface-container-highest/40 rounded w-24"></div>
                                            </td>
                                            <td class="px-6 py-4">
                                                <div class="h-3 bg-surface-container-highest/40 rounded w-20"></div>
                                            </td>
                                            <td class="px-6 py-4">
                                                <div class="h-3 bg-surface-container-highest/40 rounded w-32"></div>
                                            </td>
                                            <td class="px-6 py-4">
                                                <div class="h-5 bg-surface-container-highest/40 rounded-full w-16"></div>
                                            </td>
                                            <td class="px-6 py-4 col-hide-mobile">
                                                <div class="h-3 bg-surface-container-highest/40 rounded w-10"></div>
                                            </td>
                                            <td class="px-6 py-4 col-hide-mobile">
                                                <div class="h-3 bg-surface-container-highest/40 rounded w-12"></div>
                                            </td>
                                            <td class="px-6 py-4 col-hide-mobile">
                                                <div class="h-3 bg-surface-container-highest/40 rounded w-20"></div>
                                            </td>
                                            <td class="px-6 py-4"></td>
                                        </tr>
                                    }).collect_view().into_any();
                                }

                                let tasks = filtered_tasks.get();

                                // ── Empty state ───────────────────────────────
                                if tasks.is_empty() {
                                    let is_filtered = active_filter.get() != "all" || !search_query.get().is_empty();
                                    return view! {
                                        <tr>
                                            <td colspan="8" class="px-6 py-16 text-center">
                                                <span class="material-symbols-outlined block text-4xl text-on-surface-variant/20 mb-3">
                                                    {if is_filtered { "filter_list_off" } else { "memory" }}
                                                </span>
                                                <p class="text-sm font-semibold text-on-surface-variant/50 mb-1">
                                                    {if is_filtered {
                                                        "No tasks match the current filter"
                                                    } else {
                                                        "No AI tasks yet"
                                                    }}
                                                </p>
                                                <p class="text-xs text-on-surface-variant/35 max-w-xs mx-auto">
                                                    {if is_filtered {
                                                        "Try clearing the search or selecting a different status filter."
                                                    } else {
                                                        "The AI background processor runs tasks like enrichment, scoring, and content generation. Tasks appear here automatically when triggered by platform events."
                                                    }}
                                                </p>
                                                {if is_filtered {
                                                    view! {
                                                        <button
                                                            class="btn btn-ghost btn-sm mt-4"
                                                            on:click=move |_| {
                                                                active_filter.set("all".to_string());
                                                                search_query.set(String::new());
                                                            }
                                                        >
                                                            "Clear filters"
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    view! { <></> }.into_any()
                                                }}
                                            </td>
                                        </tr>
                                    }.into_any();
                                }

                                // ── Data rows ─────────────────────────────────
                                tasks.into_iter().map(|t| {
                                    let tid = t.id.clone();
                                    let tid_click = t.id.clone();
                                    let is_selected = Signal::derive(move || selected_task_id.get() == Some(tid.clone()));
                                    view! {
                                        <tr
                                            on:click=move |_| select_task(tid_click.clone())
                                            class=move || if is_selected.get() { "hover:bg-surface-bright/5 cursor-pointer bg-primary-container/10 border-l-2 border-primary" } else { "hover:bg-surface-bright/5 cursor-pointer" }
                                        >
                                            <td class="px-6 py-4 font-mono text-xs col-hide-mobile">{t.id.clone()}</td>
                                            <td class="px-6 py-4 font-semibold text-primary">{t.task_type.clone()}</td>
                                            <td class="px-6 py-4 max-w-xs truncate">{t.entity.clone()}</td>
                                            <td class="px-6 py-4">
                                                <span class=move || format!("px-2 py-0.5 rounded text-[10px] uppercase font-bold border {}", t.status_class.get())>
                                                    {move || t.status.get()}
                                                </span>
                                            </td>
                                            <td class="px-6 py-4 font-mono text-xs text-on-surface-variant col-hide-mobile">{move || t.runtime.get()}</td>
                                            <td class="px-6 py-4 font-mono text-xs text-on-surface-variant col-hide-mobile">{move || t.tokens.get()}</td>
                                            <td class="px-6 py-4 text-xs text-on-surface-variant col-hide-mobile">{move || t.completed.get()}</td>
                                            <td class="px-6 py-4 text-right">
                                                <button
                                                    on:click=move |e| { e.stop_propagation(); select_task(t.id.clone()); }
                                                    class="btn btn-ghost btn-sm"
                                                >
                                                    "Inspect"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any()
                            }}
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
                style="position: fixed; top: 0; right: -560px; width: 560px; height: 100vh; background: var(--bg-surface); border-left: 1px solid rgba(255,255,255,0.08); z-index: 400; display: flex; flex-direction: column; transition: right 0.24s cubic-bezier(0.25, 0.46, 0.45, 0.94); overflow: hidden;"
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
                                            class="btn btn-danger btn-sm"
                                        >
                                            "Abort Task"
                                        </button>
                                    }.into_any()
                                } else {
                                    view! {
                                        <button
                                            on:click=move |_| handle_rerun(item_val.get_value())
                                            class="btn btn-primary btn-sm"
                                        >
                                            "Rerun Task"
                                        </button>
                                    }.into_any()
                                }}
                            </div>

                            <div class="tab-bar">
                                <button
                                    class=move || if detail_tab.get() == "console" { "tab active" } else { "tab" }
                                    on:click=move |_| detail_tab.set("console".to_string())
                                >
                                    "Console Output Logs"
                                </button>
                                <button
                                    class=move || if detail_tab.get() == "details" { "tab active" } else { "tab" }
                                    on:click=move |_| detail_tab.set("details".to_string())
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
                                        <div class="inline-block w-1.5 h-3 bg-emerald-400 animate-pulse"></div>
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
                                    <div class="col-span-2 text-[10px] font-bold text-on-surface-variant uppercase tracking-widest border-b border-white/5 pb-2">"Execution Metadata"</div>

                                    <div class="space-y-1">
                                        <span class="text-xs text-on-surface-variant">"Model Context"</span>
                                        <p class="font-medium text-on-surface">{item_val.with_value(|v| v.model.clone())}</p>
                                    </div>
                                    <div class="space-y-1">
                                        <span class="text-xs text-on-surface-variant">"Runtime Latency"</span>
                                        <p class="font-mono text-xs text-on-surface">{move || item_val.with_value(|v| v.runtime.get())}</p>
                                    </div>
                                    <div class="space-y-1">
                                        <span class="text-xs text-on-surface-variant">"Context Tokens"</span>
                                        <p class="font-mono text-xs text-on-surface">{move || item_val.with_value(|v| v.tokens.get())}</p>
                                    </div>
                                    <div class="space-y-1">
                                        <span class="text-xs text-on-surface-variant">"Trigger Source"</span>
                                        <p class="font-medium text-on-surface">"OutboxWorker queue"</p>
                                    </div>

                                    <div class="col-span-2 text-[10px] font-bold text-on-surface-variant uppercase tracking-widest border-b border-white/5 pb-2 mt-4">"Parameters"</div>
                                    <div class="col-span-2">
                                        <span class="text-xs text-on-surface-variant block mb-2">"JSON Payload"</span>
                                        <pre style="font-family:monospace; font-size:11px; background:var(--bg-elevated); padding:12px; border-radius:6px; color:var(--text-secondary); overflow-x:auto;">
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
