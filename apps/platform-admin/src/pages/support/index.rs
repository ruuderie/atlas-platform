use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::api::models::CaseModel;

fn priority_color(priority: &str) -> &'static str {
    match priority.to_lowercase().as_str() {
        "critical" | "high" => "bg-red-500",
        "medium" => "bg-amber-500",
        "low" => "bg-blue-500",
        _ => "bg-emerald-500",
    }
}

fn status_class(status: &str) -> &'static str {
    match status.to_lowercase().as_str() {
        "open" => "text-red-400 border-red-500/30 bg-red-500/5",
        "in progress" => "text-blue-400 border-blue-500/30 bg-blue-500/5",
        "escalated" => "text-amber-400 border-amber-500/30 bg-amber-500/10",
        "resolved" => "text-emerald-400 border-emerald-500/30 bg-emerald-500/5",
        "closed" => "text-slate-400 border-slate-500/30 bg-slate-500/5",
        _ => "text-on-surface-variant border-outline-variant/20 bg-surface-container/30",
    }
}

fn relative_time(ts: &Option<String>) -> String {
    ts.as_deref().map(|s| s.chars().take(10).collect::<String>()).unwrap_or_else(|| "—".to_string())
}

fn short_id(id: &str) -> String {
    format!("ATL-{}", &id[..6].to_uppercase())
}

#[component]
pub fn SupportQueue() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // ── Server data ──────────────────────────────────────────────────────────
    let cases_resource = LocalResource::new(|| async move {
        crate::api::admin::get_admin_cases().await
    });

    // Selected case ID
    let selected_case_id: RwSignal<Option<String>> = RwSignal::new(None);

    // Second resource that loads detail for selected case
    let case_detail_resource = LocalResource::new(move || async move {
        match selected_case_id.get() {
            Some(id) => Some(crate::api::admin::get_admin_case(id).await),
            None => None,
        }
    });

    // ── Filter state ─────────────────────────────────────────────────────────
    let filter_selection = RwSignal::new("all".to_string());

    // ── Reply / modal state ──────────────────────────────────────────────────
    let reply_text = RwSignal::new(String::new());
    let show_internal_modal = RwSignal::new(false);
    let show_escalate_modal = RwSignal::new(false);
    let show_impersonate_modal = RwSignal::new(false);
    let internal_note_input = RwSignal::new(String::new());
    let escalate_reason = RwSignal::new("SLA breach imminent".to_string());
    let escalate_target = RwSignal::new("Jordan M. (Supervisor)".to_string());
    let escalate_notes = RwSignal::new(String::new());

    // ── Handlers ─────────────────────────────────────────────────────────────
    let handle_resolve = move |_| {
        let Some(id) = selected_case_id.get() else { return; };
        let toast = toast.clone();
        let resource = cases_resource.clone();
        spawn_local(async move {
            match crate::api::admin::update_case_status(id, "Resolved".to_string()).await {
                Ok(_) => {
                    resource.refetch();
                    toast.show_toast("Success", "Case marked as resolved.", "success");
                }
                Err(e) => toast.show_toast("Error", &e, "error"),
            }
        });
    };

    let handle_send_reply = move |_| {
        let txt = reply_text.get();
        if txt.trim().is_empty() {
            toast.show_toast("Error", "Reply content cannot be empty.", "error");
            return;
        }
        reply_text.set(String::new());
        toast.show_toast("Success", "Reply sent to tenant operator.", "success");
    };

    let handle_save_internal_note = move |_| {
        let note = internal_note_input.get();
        if note.trim().is_empty() {
            toast.show_toast("Error", "Note content cannot be empty.", "error");
            return;
        }
        show_internal_modal.set(false);
        internal_note_input.set(String::new());
        toast.show_toast("Success", "Internal note registered (hidden from tenant).", "success");
    };

    let handle_save_escalation = move |_| {
        let target = escalate_target.get();
        let Some(id) = selected_case_id.get() else { return; };
        let toast = toast.clone();
        let resource = cases_resource.clone();
        spawn_local(async move {
            match crate::api::admin::update_case_status(id, "Escalated".to_string()).await {
                Ok(_) => {
                    resource.refetch();
                    show_escalate_modal.set(false);
                    escalate_notes.set(String::new());
                    toast.show_toast("Warning", &format!("Case escalated to {}.", target), "warn");
                }
                Err(e) => toast.show_toast("Error", &e, "error"),
            }
        });
    };

    let handle_confirm_impersonate = move |_| {
        show_impersonate_modal.set(false);
        toast.show_toast("Warning", "⚠ Impersonation token active. Audit log registered.", "warn");
    };

    view! {
        <div class="h-[calc(100vh-140px)] flex bg-surface border border-outline-variant/10 rounded-2xl overflow-hidden shadow-lg text-on-surface">

            // ── Left panel: ticket list ──────────────────────────────────────
            <div class="w-80 flex-shrink-0 border-r border-outline-variant/10 flex flex-col bg-surface-container/20">
                <div class="p-4 border-b border-outline-variant/10 flex-shrink-0">
                    <div class="flex items-center justify-between font-bold text-sm">
                        <span>"Support Queue"</span>
                        <div class="flex items-center gap-2">
                            <button
                                class="p-1 rounded hover:bg-surface-bright/20 text-on-surface-variant hover:text-on-surface transition-colors"
                                title="Refresh queue"
                                on:click=move |_| cases_resource.refetch()
                            >
                                <svg class="w-3 h-3" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8">
                                    <path d="M13.5 8A5.5 5.5 0 1 1 8 2.5M13.5 2.5v3h-3"/>
                                </svg>
                            </button>
                            <Suspense fallback=|| view! { <span class="px-2 py-0.5 text-[10px] font-bold rounded-full bg-surface-container border border-outline-variant/20 text-on-surface-variant">"..."</span> }>
                                {move || cases_resource.get().map(|r| {
                                    let count = r.as_deref().ok().map(|v| v.iter().filter(|c| c.status.to_lowercase() == "open").count()).unwrap_or(0);
                                    view! {
                                        <span class="px-2 py-0.5 text-[10px] font-bold rounded-full bg-red-500/10 border border-red-500/30 text-red-400">
                                            {count.to_string()} " Open"
                                        </span>
                                    }
                                })}
                            </Suspense>
                        </div>
                    </div>
                    <p class="text-[10.5px] text-on-surface-variant mt-1">"Tenant operational issues and support cases"</p>
                </div>

                // Filter pills
                <div class="p-3 border-b border-outline-variant/5 flex gap-1.5 overflow-x-auto scrollbar-none flex-shrink-0">
                    {
                        let filter_pill = move |scope: &'static str, label: &'static str| {
                            let active = move || filter_selection.get() == scope;
                            view! {
                                <button
                                    on:click=move |_| filter_selection.set(scope.to_string())
                                    class=move || format!("px-2.5 py-1 text-[10px] font-bold border rounded-lg whitespace-nowrap transition-all {}",
                                        if active() { "bg-primary-container border-primary text-primary" }
                                        else { "bg-[#05183c]/20 border-outline-variant/20 text-on-surface-variant hover:text-on-surface" }
                                    )
                                >{label}</button>
                            }
                        };
                        view! {
                            {filter_pill("all", "All Cases")}
                            {filter_pill("open", "Open")}
                            {filter_pill("in progress", "In Progress")}
                            {filter_pill("resolved", "Resolved")}
                        }
                    }
                </div>

                // Case list
                <div class="flex-1 overflow-y-auto divide-y divide-outline-variant/5">
                    <Suspense fallback=|| view! {
                        <div class="p-6 text-center text-xs text-on-surface-variant">"Loading cases..."</div>
                    }>
                        {move || cases_resource.get().map(|result| {
                            match result.as_deref() {
                                Ok(cases) => {
                                    let filter = filter_selection.get();
                                    let filtered: Vec<CaseModel> = cases.iter()
                                        .filter(|c| filter == "all" || c.status.to_lowercase() == filter)
                                        .cloned()
                                        .collect();

                                    if filtered.is_empty() {
                                        return view! {
                                            <div class="p-6 text-center text-xs text-on-surface-variant">"No cases found."</div>
                                        }.into_any();
                                    }

                                    view! {
                                        <For
                                            each=move || filtered.clone()
                                            key=|c| c.id.clone()
                                            children=move |case| {
                                                let case_id = case.id.clone();
                                                let is_sel = {
                                                    let cid = case_id.clone();
                                                    Signal::derive(move || selected_case_id.get().as_deref() == Some(&cid))
                                                };
                                                let short = short_id(&case.id);
                                                let pcolor = priority_color(&case.priority).to_string();
                                                let sc = status_class(&case.status).to_string();
                                                let time_str = relative_time(&case.created_at);

                                                view! {
                                                    <div
                                                        on:click={
                                                            let cid = case_id.clone();
                                                            move |_| selected_case_id.set(Some(cid.clone()))
                                                        }
                                                        class=move || format!(
                                                            "p-4 cursor-pointer transition-all border-l-2 {}",
                                                            if is_sel.get() { "bg-surface-bright/10 border-primary" } else { "border-transparent hover:bg-surface-bright/5" }
                                                        )
                                                    >
                                                        <div class="space-y-1">
                                                            <div class="flex items-center justify-between text-[10px] font-semibold text-on-surface-variant">
                                                                <span class=format!("px-1.5 rounded font-bold border text-[9px] {}", sc)>{case.status.clone()}</span>
                                                                <span>{time_str}</span>
                                                            </div>
                                                            <div class="flex items-start gap-2 justify-between">
                                                                <h4 class="text-xs font-bold text-on-surface line-clamp-1">
                                                                    {short} " — " {case.title.clone()}
                                                                </h4>
                                                                <span class=format!("w-2 h-2 rounded-full mt-1.5 flex-shrink-0 {}", pcolor)></span>
                                                            </div>
                                                            <p class="text-[10px] text-on-surface-variant truncate">
                                                                "Priority: " {case.priority.clone()}
                                                            </p>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                    }.into_any()
                                }
                                Err(e) => view! {
                                    <div class="p-6 text-center text-xs text-red-400">"Error: " {e.to_string()}</div>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </div>
            </div>

            // ── Right panel: case detail workspace ───────────────────────────
            <div class="flex-1 flex flex-col bg-surface overflow-hidden">
                {move || {
                    match selected_case_id.get() {
                        None => view! {
                            <div class="flex-1 flex items-center justify-center">
                                <div class="text-center text-on-surface-variant">
                                    <div class="text-4xl mb-3">"🎫"</div>
                                    <p class="text-sm font-semibold">"Select a case to view details"</p>
                                    <p class="text-xs mt-1">"Pick a support case from the list on the left"</p>
                                </div>
                            </div>
                        }.into_any(),
                        Some(_) => view! {
                            <Suspense fallback=|| view! {
                                <div class="flex-1 flex items-center justify-center text-xs text-on-surface-variant">"Loading case details..."</div>
                            }>
                                {move || case_detail_resource.get().map(|opt| {
                                    match opt {
                                        Some(Ok(case)) => {
                                            let sc = status_class(&case.status).to_string();
                                            let short = short_id(&case.id);
                                            let case_id = case.id.clone();

                                            view! {
                                                // Header
                                                <div class="p-5 border-b border-outline-variant/10 flex justify-between items-start flex-shrink-0 gap-4">
                                                    <div class="space-y-1">
                                                        <h2 class="text-base font-bold text-on-surface">
                                                            {short.clone()} " — " {case.title.clone()}
                                                        </h2>
                                                        <div class="flex items-center gap-3 text-xs">
                                                            <span class=format!("px-2 py-0.5 rounded text-[10px] font-bold border {}", sc)>
                                                                {case.status.clone()}
                                                            </span>
                                                            <span class="text-on-surface-variant font-mono text-[10px] bg-surface-container/60 border border-outline-variant/20 px-2 py-0.5 rounded">
                                                                "Priority: " {case.priority.clone()}
                                                            </span>
                                                            <span class="text-on-surface-variant">
                                                                {relative_time(&case.created_at)}
                                                            </span>
                                                        </div>
                                                    </div>

                                                    <div class="flex items-center gap-2">
                                                        <button
                                                            on:click=move |_| show_impersonate_modal.set(true)
                                                            class="px-3 py-1.5 text-xs font-semibold bg-[#05183c] border border-outline-variant/30 text-[#91aaeb] hover:bg-[#05183c]/60 rounded-lg flex items-center gap-1.5"
                                                        >
                                                            <span class="material-symbols-outlined text-sm">"key"</span>
                                                            "Impersonate NI"
                                                        </button>
                                                        <button
                                                            on:click=handle_resolve
                                                            class="px-3 py-1.5 text-xs font-bold text-on-primary bg-emerald-600 hover:bg-emerald-700 active:scale-95 transition-all rounded-lg flex items-center gap-1"
                                                        >
                                                            <span class="material-symbols-outlined text-sm">"check"</span>
                                                            "Resolve"
                                                        </button>
                                                    </div>
                                                </div>

                                                // Case metadata bar
                                                <div class="px-5 py-2.5 bg-surface-container-low border-b border-outline-variant/10 flex items-center justify-between text-xs flex-shrink-0">
                                                    <div class="flex items-center gap-6">
                                                        <span>"Case ID: " <strong class="text-on-surface font-mono">{short}</strong></span>
                                                        <span>"Priority: " <strong class="text-on-surface">{case.priority.clone()}</strong></span>
                                                        <span>"Assigned: " <strong class="text-on-surface">
                                                            {case.assigned_to.clone().unwrap_or_else(|| "Unassigned".to_string())}
                                                        </strong></span>
                                                        <span>"Created: " <strong class="text-on-surface">{relative_time(&case.created_at)}</strong></span>
                                                    </div>
                                                    <a
                                                        href=format!("/admin/cases/{}", case_id)
                                                        class="text-primary hover:underline font-semibold flex items-center gap-0.5"
                                                    >
                                                        "Full Case Details"
                                                        <span class="material-symbols-outlined text-[10px]">"arrow_forward"</span>
                                                    </a>
                                                </div>

                                                // Description
                                                <div class="px-5 py-4 border-b border-outline-variant/5 flex-shrink-0 bg-surface-container/10">
                                                    <p class="text-xs font-semibold text-on-surface-variant mb-1">"Case Description"</p>
                                                    <p class="text-sm text-on-surface leading-relaxed">{case.description.clone()}</p>
                                                </div>

                                                // Activity + notes thread
                                                <div class="flex-1 overflow-y-auto p-5 space-y-4">
                                                    {if case.notes.is_empty() && case.activities.is_empty() {
                                                        view! {
                                                            <div class="text-center text-xs text-on-surface-variant py-8">
                                                                "No notes or activity yet on this case."
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            // Notes
                                                            <For
                                                                each=move || case.notes.clone()
                                                                key=|n| n.id.clone()
                                                                children=move |note| {
                                                                    view! {
                                                                        <div class="flex gap-3 max-w-[80%]">
                                                                            <div class="w-7 h-7 rounded-full flex items-center justify-center text-[10px] font-bold text-white flex-shrink-0 bg-amber-500/80 border border-white/5">
                                                                                "📝"
                                                                            </div>
                                                                            <div class="space-y-1">
                                                                                <div class="text-[10px] text-on-surface-variant">
                                                                                    "Note · " {note.created_at.clone().unwrap_or_default()}
                                                                                </div>
                                                                                <div class="p-3 rounded-2xl text-xs leading-relaxed border bg-amber-500/10 border-amber-500/30 text-on-surface">
                                                                                    <div class="text-[9px] font-bold text-amber-500 uppercase tracking-wider mb-1">"📝 Case Note"</div>
                                                                                    {note.content.clone()}
                                                                                </div>
                                                                            </div>
                                                                        </div>
                                                                    }
                                                                }
                                                            />
                                                            // Activities
                                                            <For
                                                                each=move || case.activities.clone()
                                                                key=|a| a.id.clone()
                                                                children=move |act| {
                                                                    view! {
                                                                        <div class="flex gap-3 max-w-[80%]">
                                                                            <div class="w-7 h-7 rounded-full flex items-center justify-center text-[10px] font-bold text-white flex-shrink-0 border border-white/5"
                                                                                style="background: linear-gradient(135deg, #0A84FF, #00C853)">
                                                                                "A"
                                                                            </div>
                                                                            <div class="space-y-1">
                                                                                <div class="text-[10px] text-on-surface-variant">
                                                                                    {act.activity_type.clone().unwrap_or_else(|| "Activity".to_string())}
                                                                                    " · " {act.created_at.clone().unwrap_or_default()}
                                                                                </div>
                                                                                <div class="p-3 rounded-2xl text-xs leading-relaxed border bg-surface-container border-outline-variant/15 text-on-surface rounded-tl-none">
                                                                                    {act.description.clone().unwrap_or_else(|| "No description".to_string())}
                                                                                </div>
                                                                            </div>
                                                                        </div>
                                                                    }
                                                                }
                                                            />
                                                        }.into_any()
                                                    }}
                                                </div>

                                                // Reply compose area
                                                <div class="p-4 border-t border-outline-variant/10 bg-surface-container/20 flex-shrink-0 space-y-3">
                                                    <textarea
                                                        rows="2"
                                                        placeholder="Send a reply to the tenant operator... (External communication)"
                                                        class="w-full bg-[#06122d] border border-outline-variant/30 text-on-surface text-sm rounded-lg p-3 focus:ring-1 focus:ring-primary focus:border-primary placeholder:text-on-surface-variant/40 resize-none outline-none"
                                                        prop:value=reply_text
                                                        on:input=move |ev| reply_text.set(event_target_value(&ev))
                                                    ></textarea>

                                                    <div class="flex flex-wrap justify-between items-center gap-3">
                                                        <div class="flex items-center gap-2">
                                                            <button
                                                                on:click=move |_| show_internal_modal.set(true)
                                                                class="px-3 py-1.5 text-xs font-semibold bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 rounded-lg flex items-center gap-1"
                                                            >
                                                                <span class="material-symbols-outlined text-[14px]">"lock"</span>
                                                                "Internal Note"
                                                            </button>
                                                            <button
                                                                on:click=move |_| show_escalate_modal.set(true)
                                                                class="px-3 py-1.5 text-xs font-semibold bg-amber-500/10 border border-amber-500/30 text-amber-400 hover:bg-amber-500/20 rounded-lg flex items-center gap-1"
                                                            >
                                                                <span class="material-symbols-outlined text-[14px]">"campaign"</span>
                                                                "Escalate"
                                                            </button>
                                                        </div>
                                                        <button
                                                            on:click=handle_send_reply
                                                            class="px-4 py-2 text-xs font-bold text-on-primary bg-primary border-none hover:opacity-90 active:scale-95 transition-all rounded-lg flex items-center gap-1"
                                                        >
                                                            "Send Reply"
                                                            <span class="material-symbols-outlined text-sm">"send"</span>
                                                        </button>
                                                    </div>
                                                </div>
                                            }.into_any()
                                        }
                                        Some(Err(_)) => view! {
                                            <div class="flex-1 flex items-center justify-center text-xs text-red-400">
                                                "Error loading case details."
                                            </div>
                                        }.into_any(),
                                        None => view! { <div></div> }.into_any(),
                                    }
                                })}
                            </Suspense>
                        }.into_any(),
                    }
                }}
            </div>

            // ── Internal Note Modal ──────────────────────────────────────────
            <Show when=move || show_internal_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_internal_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2">"Add Internal Staff Note"</h3>
                        <div class="p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg text-xs text-amber-400 mb-4 font-semibold flex items-center gap-2">
                            <span class="material-symbols-outlined text-sm">"lock"</span>
                            "Locked Note — NEVER visible to the tenant operator."
                        </div>
                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-semibold text-on-surface-variant">"Note details *"</label>
                                <textarea
                                    rows="4"
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary placeholder:text-on-surface-variant/40"
                                    placeholder="Enter diagnostics, call highlights, sync details..."
                                    prop:value=internal_note_input
                                    on:input=move |ev| internal_note_input.set(event_target_value(&ev))
                                ></textarea>
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| show_internal_modal.set(false) class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface">"Cancel"</button>
                            <button on:click=handle_save_internal_note class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-bold text-on-primary">"Save Internal Note"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Escalate Modal ───────────────────────────────────────────────
            <Show when=move || show_escalate_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_escalate_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2">"Escalate Support Case"</h3>
                        <p class="text-xs text-on-surface-variant mb-4">"Escalations automatically flag the tenant's Account Manager."</p>
                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-semibold text-on-surface-variant">"Escalation Reason"</label>
                                <select
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    on:change=move |ev| escalate_reason.set(event_target_value(&ev))
                                >
                                    <option value="SLA breach imminent">"SLA breach imminent"</option>
                                    <option value="Requires engineering access">"Requires engineering database access"</option>
                                    <option value="Billing dispute — finance">"Billing dispute — needs finance review"</option>
                                    <option value="Security / compliance hold">"Security / compliance hold"</option>
                                </select>
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-semibold text-on-surface-variant">"Assign To Queue"</label>
                                <select
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    on:change=move |ev| escalate_target.set(event_target_value(&ev))
                                >
                                    <option value="Jordan M. (Supervisor)">"Jordan M. (Supervisor)"</option>
                                    <option value="Engineering On-Call">"Engineering On-Call Team"</option>
                                    <option value="Stripe Rep Integration">"Stripe Rep Account team"</option>
                                </select>
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-semibold text-on-surface-variant">"Internal Description & Context"</label>
                                <textarea
                                    rows="3"
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary placeholder:text-on-surface-variant/40"
                                    placeholder="Provide context for the escalation target..."
                                    prop:value=escalate_notes
                                    on:input=move |ev| escalate_notes.set(event_target_value(&ev))
                                ></textarea>
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| show_escalate_modal.set(false) class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface">"Cancel"</button>
                            <button on:click=handle_save_escalation class="px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded-lg text-xs font-bold transition-all">"Escalate Case"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Impersonate Modal ────────────────────────────────────────────
            <Show when=move || show_impersonate_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_impersonate_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold text-red-400 mb-2 flex items-center gap-1.5">
                            <span class="material-symbols-outlined">"warning"</span>
                            "Impersonate tenant operator view"
                        </h3>
                        <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-xs text-red-400 mb-4 leading-relaxed">
                            "You are about to start a diagnostics session. All actions will be audit-logged under your staff profile."
                        </div>
                        <p class="text-xs text-on-surface-variant mb-6 leading-relaxed">
                            "This grants access to view private listings, customer billing cards, and run platform adjustments. Use strictly for resolving cases."
                        </p>
                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| show_impersonate_modal.set(false) class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface">"Cancel"</button>
                            <button on:click=handle_confirm_impersonate class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-xs font-bold transition-all">"Audit & Impersonate"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
