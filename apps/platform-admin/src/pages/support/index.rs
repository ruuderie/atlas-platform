use crate::api::models::{SupportThreadDetail, SupportThreadSummary};
use leptos::prelude::*;
use leptos::task::spawn_local;

fn status_class(is_active: bool) -> &'static str {
    if is_active {
        "text-red-400 border-red-500/30 bg-red-500/5"
    } else {
        "text-slate-400 border-slate-500/30 bg-slate-500/5"
    }
}

fn msg_time(ts: &str) -> String {
    // ISO8601 → "YYYY-MM-DD HH:MM"
    let date = ts.chars().take(10).collect::<String>();
    let time = ts.chars().skip(11).take(5).collect::<String>();
    if time.is_empty() {
        date
    } else {
        format!("{date} {time}")
    }
}

fn short_id(id: &str) -> String {
    let upper = id.replace('-', "");
    format!(
        "SUP-{}",
        upper.chars().take(6).collect::<String>().to_uppercase()
    )
}

fn initials(name: &Option<String>) -> String {
    name.as_deref()
        .map(|n| {
            n.split_whitespace()
                .filter_map(|w| w.chars().next())
                .take(2)
                .collect::<String>()
                .to_uppercase()
        })
        .unwrap_or_else(|| "?".to_string())
}

#[component]
pub fn SupportQueue() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // ── Server data ──────────────────────────────────────────────────────────
    let refresh = RwSignal::new(0u32);
    let status_filter = RwSignal::new("open".to_string());

    let threads_resource = LocalResource::new(move || {
        let sf = status_filter.get();
        async move {
            let _ = refresh.get();
            crate::api::admin::get_support_threads(&sf).await
        }
    });

    // Selected thread state
    let selected_id: RwSignal<Option<String>> = RwSignal::new(None);

    let thread_detail_resource = LocalResource::new(move || async move {
        match selected_id.get() {
            Some(id) => Some(crate::api::admin::get_support_thread(id).await),
            None => None,
        }
    });

    // ── Reply / action state ─────────────────────────────────────────────────
    let reply_text = RwSignal::new(String::new());
    let show_internal_modal = RwSignal::new(false);
    let show_escalate_modal = RwSignal::new(false);
    let show_impersonate_modal = RwSignal::new(false);
    let internal_note_input = RwSignal::new(String::new());
    let escalate_reason = RwSignal::new("SLA breach imminent".to_string());
    let escalate_target = RwSignal::new("Engineering On-Call".to_string());
    let escalate_notes = RwSignal::new(String::new());
    let sending = RwSignal::new(false);

    // ── Handlers ─────────────────────────────────────────────────────────────
    let handle_close = {
        let toast = toast.clone();
        move |_| {
            let Some(id) = selected_id.get() else {
                return;
            };
            let toast = toast.clone();
            spawn_local(async move {
                match crate::api::admin::close_support_thread(id).await {
                    Ok(_) => {
                        selected_id.set(None);
                        refresh.update(|n| *n += 1);
                        toast.show_toast("Success", "Thread closed and user notified.", "success");
                    }
                    Err(e) => toast.show_toast("Error", &e, "error"),
                }
            });
        }
    };

    let handle_send_reply = {
        let toast = toast.clone();
        move |_| {
            let txt = reply_text.get();
            if txt.trim().is_empty() {
                toast.show_toast("Error", "Reply cannot be empty.", "error");
                return;
            }
            let Some(id) = selected_id.get() else {
                return;
            };
            let toast = toast.clone();
            sending.set(true);
            spawn_local(async move {
                match crate::api::admin::send_support_reply(id.clone(), txt).await {
                    Ok(_) => {
                        reply_text.set(String::new());
                        sending.set(false);
                        // Reload thread detail to show new message
                        thread_detail_resource.refetch();
                        toast.show_toast("Sent", "Reply delivered to user.", "success");
                    }
                    Err(e) => {
                        sending.set(false);
                        toast.show_toast("Error", &e, "error");
                    }
                }
            });
        }
    };

    let handle_save_internal_note = {
        let toast = toast.clone();
        move |_| {
            let note = internal_note_input.get();
            if note.trim().is_empty() {
                toast.show_toast("Error", "Note cannot be empty.", "error");
                return;
            }
            show_internal_modal.set(false);
            internal_note_input.set(String::new());
            toast.show_toast(
                "Saved",
                "Internal note registered (hidden from user).",
                "success",
            );
        }
    };

    let handle_save_escalation = {
        let toast = toast.clone();
        move |_| {
            let target = escalate_target.get();
            show_escalate_modal.set(false);
            escalate_notes.set(String::new());
            toast.show_toast(
                "Escalated",
                &format!("Thread assigned to {}.", target),
                "warn",
            );
        }
    };

    let handle_confirm_impersonate = {
        let toast = toast.clone();
        move |_| {
            show_impersonate_modal.set(false);
            toast.show_toast(
                "Warning",
                "⚠ Impersonation token active. Audit log registered.",
                "warn",
            );
        }
    };

    view! {
        <div class="main-area">

            // ── Page Header ──
            <div class="page-header">
                <div>
                    <div class="page-title">"Support Inbox"</div>
                    <div class="page-subtitle">"Platform-wide support threads from Folio users · Click any thread to open the workspace"</div>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-ghost btn-sm"
                        title="Refresh"
                        on:click=move |_| refresh.update(|n| *n += 1)
                    >
                        <svg class="w-3 h-3 inline-block mr-1" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8">
                            <path d="M13.5 8A5.5 5.5 0 1 1 8 2.5M13.5 2.5v3h-3"/>
                        </svg>
                        "Refresh"
                    </button>
                    <button class="btn btn-ghost btn-sm" on:click=move |_| toast.show_toast("Info", "Exporting inbox to CSV…", "info")>"↓ Export"</button>
                </div>
            </div>

            // ── KPI Row ──
            <div class="kpi-row">
                <div class="kpi-card">
                    <span class="kpi-label">"Open Threads"</span>
                    <span class="kpi-value" style="color:var(--red)">
                        <Suspense fallback=|| view! { "—" }>
                            {move || threads_resource.get().map(|r| {
                                r.as_deref().ok().map(|v| v.iter().filter(|t| t.is_active).count()).unwrap_or(0).to_string()
                            })}
                        </Suspense>
                    </span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Closed Threads"</span>
                    <span class="kpi-value" style="color:var(--green)">
                        <Suspense fallback=|| view! { "—" }>
                            {move || threads_resource.get().map(|r| {
                                r.as_deref().ok().map(|v| v.iter().filter(|t| !t.is_active).count()).unwrap_or(0).to_string()
                            })}
                        </Suspense>
                    </span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Total Messages"</span>
                    <span class="kpi-value" style="color:var(--cobalt)">
                        <Suspense fallback=|| view! { "—" }>
                            {move || threads_resource.get().map(|r| {
                                r.as_deref().ok().map(|v| v.iter().map(|t| t.message_count).sum::<u64>()).unwrap_or(0).to_string()
                            })}
                        </Suspense>
                    </span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Selected Thread"</span>
                    <span class="kpi-value" style="color:var(--amber)">
                        {move || selected_id.get().map(|id| short_id(&id)).unwrap_or_else(|| "None".to_string())}
                    </span>
                </div>
            </div>

            // ── 2-Panel Workspace ──
            <div class="flags-body" style="display:flex;flex-direction:row;padding:0;gap:0;overflow:hidden;border:1px solid var(--border-default);border-radius:8px;background:var(--bg-surface);">

            // ── Left panel: thread list ──────────────────────────────────────
            <div class="w-80 flex-shrink-0 border-r border-outline-variant/10 flex flex-col bg-surface-container/20">
                <div class="p-4 border-b border-outline-variant/10 flex-shrink-0">
                    <div class="flex items-center justify-between font-bold text-sm">
                        <span>"Inbox"</span>
                        <Suspense fallback=|| view! { <span class="px-2 py-0.5 text-[10px] font-bold rounded-full bg-surface-container border border-outline-variant/20 text-on-surface-variant">"..."</span> }>
                            {move || threads_resource.get().map(|r| {
                                let open = r.as_deref().ok().map(|v| v.iter().filter(|t| t.is_active).count()).unwrap_or(0);
                                view! {
                                    <span class="px-2 py-0.5 text-[10px] font-bold rounded-full bg-red-500/10 border border-red-500/30 text-red-400">
                                        {open.to_string()} " Open"
                                    </span>
                                }
                            })}
                        </Suspense>
                    </div>
                </div>

                // Status filter pills
                <div class="p-3 border-b border-outline-variant/5 flex gap-1.5 overflow-x-auto scrollbar-none flex-shrink-0">
                    {
                        let pill = move |scope: &'static str, label: &'static str| {
                            let active = move || status_filter.get() == scope;
                            view! {
                                <button
                                    on:click=move |_| {
                                        status_filter.set(scope.to_string());
                                        selected_id.set(None);
                                        refresh.update(|n| *n += 1);
                                    }
                                    class=move || if active() { "pill active" } else { "pill" }
                                >{label}</button>
                            }
                        };
                        view! {
                            {pill("open", "Open")}
                            {pill("closed", "Closed")}
                            {pill("all", "All")}
                        }
                    }
                </div>

                // Thread list
                <div class="flex-1 overflow-y-auto divide-y divide-outline-variant/5">
                    <Suspense fallback=|| view! {
                        <div class="p-6 text-center text-xs text-on-surface-variant">"Loading threads..."</div>
                    }>
                        {move || threads_resource.get().map(|result| {
                            match result.as_deref() {
                                Ok(threads) => {
                                    if threads.is_empty() {
                                        return view! {
                                            <div class="p-6 text-center text-xs text-on-surface-variant">"No threads found."</div>
                                        }.into_any();
                                    }
                                    let threads_cloned = threads.to_vec();
                                    view! {
                                        <For
                                            each=move || threads_cloned.clone()
                                            key=|t| t.id.clone()
                                            children=move |thread| {
                                                let tid = thread.id.clone();
                                                let is_sel = {
                                                    let cid = tid.clone();
                                                    Signal::derive(move || selected_id.get().as_deref() == Some(&cid))
                                                };
                                                let short  = short_id(&thread.id);
                                                let sc     = status_class(thread.is_active).to_string();
                                                let preview= thread.last_message.clone().unwrap_or_else(|| "No messages yet".to_string());
                                                let time   = thread.last_at.as_deref().map(msg_time).unwrap_or_else(|| thread.created_at.chars().take(10).collect());
                                                let name   = thread.submitter_name.clone().unwrap_or_else(|| thread.submitter_email.clone().unwrap_or_else(|| "Unknown".to_string()));
                                                let count  = thread.message_count;

                                                view! {
                                                    <div
                                                        on:click={
                                                            let cid = tid.clone();
                                                            move |_| selected_id.set(Some(cid.clone()))
                                                        }
                                                        class=move || format!(
                                                            "p-4 cursor-pointer transition-all border-l-2 {}",
                                                            if is_sel.get() { "bg-surface-bright/10 border-primary" } else { "border-transparent hover:bg-surface-bright/5" }
                                                        )
                                                    >
                                                        <div class="space-y-1">
                                                            <div class="flex items-center justify-between text-[10px] font-semibold text-on-surface-variant">
                                                                <span class=format!("px-1.5 rounded font-bold border text-[9px] {}", sc)>
                                                                    {if thread.is_active { "Open" } else { "Closed" }}
                                                                </span>
                                                                <span>{time}</span>
                                                            </div>
                                                            <div class="flex items-start gap-2 justify-between">
                                                                <h4 class="text-xs font-bold text-on-surface line-clamp-1">
                                                                    {short} " — " {name}
                                                                </h4>
                                                                <span class="px-1.5 py-0.5 text-[9px] font-bold rounded-full bg-surface-container border border-outline-variant/20 text-on-surface-variant flex-shrink-0">
                                                                    {count.to_string()}
                                                                </span>
                                                            </div>
                                                            <p class="text-[10px] text-on-surface-variant truncate">{preview}</p>
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

            // ── Right panel: thread workspace ────────────────────────────────
            <div class="flex-1 flex flex-col bg-surface overflow-hidden">
                {move || {
                    match selected_id.get() {
                        None => view! {
                            <div class="flex-1 flex items-center justify-center">
                                <div class="text-center text-on-surface-variant">
                                    <svg class="w-12 h-12 mx-auto mb-3 opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
                                        <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>
                                    </svg>
                                    <p class="text-sm font-semibold">"Select a thread to view the conversation"</p>
                                    <p class="text-xs mt-1">"Pick a support request from the list on the left"</p>
                                </div>
                            </div>
                        }.into_any(),
                        Some(_) => view! {
                            <Suspense fallback=|| view! {
                                <div class="flex-1 flex items-center justify-center text-xs text-on-surface-variant">"Loading thread..."</div>
                            }>
                                {move || thread_detail_resource.get().map(|opt| {
                                    match opt {
                                        Some(Ok(thread)) => {
                                            let short = short_id(&thread.id);
                                            let sc    = status_class(thread.is_active).to_string();
                                            let name  = thread.submitter_name.clone()
                                                .unwrap_or_else(|| thread.submitter_email.clone().unwrap_or_else(|| "Unknown User".to_string()));
                                            let email = thread.submitter_email.clone().unwrap_or_default();
                                            let tid   = thread.id.clone();
                                            let msgs  = thread.messages.clone();
                                            let is_open = thread.is_active;

                                            view! {
                                                // Header
                                                <div class="p-5 border-b border-outline-variant/10 flex justify-between items-start flex-shrink-0 gap-4">
                                                    <div class="space-y-1">
                                                        <h2 class="text-base font-bold text-on-surface">
                                                            {short.clone()} " — " {name.clone()}
                                                        </h2>
                                                        <div class="flex items-center gap-3 text-xs">
                                                            <span class=format!("px-2 py-0.5 rounded text-[10px] font-bold border {}", sc)>
                                                                {if is_open { "Open" } else { "Closed" }}
                                                            </span>
                                                            <span class="text-on-surface-variant font-mono text-[10px] bg-surface-container/60 border border-outline-variant/20 px-2 py-0.5 rounded">
                                                                {email.clone()}
                                                            </span>
                                                            <span class="text-on-surface-variant">
                                                                {thread.created_at.chars().take(10).collect::<String>()}
                                                            </span>
                                                        </div>
                                                    </div>

                                                    <div class="flex items-center gap-2 flex-shrink-0">
                                                        <button
                                                            on:click=move |_| show_impersonate_modal.set(true)
                                                            class="btn btn-ghost btn-sm"
                                                        >
                                                            <span class="material-symbols-outlined text-sm">"key"</span>
                                                            "Impersonate"
                                                        </button>
                                                        <Show when=move || is_open>
                                                            <button
                                                                on:click=handle_close
                                                                class="btn btn-primary btn-sm"
                                                            >
                                                                <span class="material-symbols-outlined text-sm">"check"</span>
                                                                "Close Thread"
                                                            </button>
                                                        </Show>
                                                    </div>
                                                </div>

                                                // Thread metadata bar
                                                <div class="px-5 py-2.5 bg-surface-container-low border-b border-outline-variant/10 flex items-center justify-between text-xs flex-shrink-0">
                                                    <div class="flex items-center gap-6">
                                                        <span>"Thread: " <strong class="text-on-surface font-mono">{short}</strong></span>
                                                        <span>"Messages: " <strong class="text-on-surface">{thread.message_count.to_string()}</strong></span>
                                                        <span>"Tenant: " <strong class="text-on-surface font-mono text-[10px]">{thread.tenant_id.chars().take(8).collect::<String>()}{"…"}</strong></span>
                                                    </div>
                                                    <span class="text-on-surface-variant text-[10px]">"User ID: " {thread.entity_id.chars().take(8).collect::<String>()} "…"</span>
                                                </div>

                                                // Message thread
                                                <div class="flex-1 overflow-y-auto p-5 space-y-4">
                                                    {if msgs.is_empty() {
                                                        view! {
                                                            <div class="text-center text-xs text-on-surface-variant py-8">
                                                                "No messages in this thread yet."
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <For
                                                                each=move || msgs.clone()
                                                                key=|m| m.id.clone()
                                                                children=move |msg| {
                                                                    let is_op  = msg.is_operator;
                                                                    let is_sys = msg.message_type == "system";
                                                                    let sender = msg.sender_name.clone()
                                                                        .unwrap_or_else(|| if is_op { "Platform Support".to_string() } else { "User".to_string() });
                                                                    let ini    = initials(&msg.sender_name);
                                                                    let time   = msg_time(&msg.created_at);
                                                                    let content= msg.content.clone();

                                                                    if is_sys {
                                                                        view! {
                                                                            <div class="flex items-center gap-3 my-2">
                                                                                <div class="flex-1 h-px bg-outline-variant/10"/>
                                                                                <span class="text-[10px] text-on-surface-variant px-3 py-1 bg-surface-container rounded-full border border-outline-variant/15">
                                                                                    {content}
                                                                                </span>
                                                                                <div class="flex-1 h-px bg-outline-variant/10"/>
                                                                            </div>
                                                                        }.into_any()
                                                                    } else if is_op {
                                                                        // Operator bubble — right aligned
                                                                        view! {
                                                                            <div class="flex gap-3 justify-end">
                                                                                <div class="space-y-1 max-w-[75%]">
                                                                                    <div class="text-[10px] text-on-surface-variant text-right">
                                                                                        "Platform Support · " {time}
                                                                                    </div>
                                                                                    <div class="p-3 rounded-2xl rounded-tr-none text-xs leading-relaxed bg-primary/15 border border-primary/30 text-on-surface">
                                                                                        <div class="text-[9px] font-bold text-primary uppercase tracking-wider mb-1">"🛡 Atlas Support"</div>
                                                                                        {content}
                                                                                    </div>
                                                                                </div>
                                                                                <div class="w-7 h-7 rounded-full flex items-center justify-center text-[10px] font-bold text-white flex-shrink-0 border border-white/5"
                                                                                    style="background: linear-gradient(135deg, #0A84FF, #5E5CE6)">
                                                                                    "🛡"
                                                                                </div>
                                                                            </div>
                                                                        }.into_any()
                                                                    } else {
                                                                        // User bubble — left aligned
                                                                        view! {
                                                                            <div class="flex gap-3 max-w-[75%]">
                                                                                <div class="w-7 h-7 rounded-full flex items-center justify-center text-[10px] font-bold text-white flex-shrink-0 bg-amber-500/80 border border-white/5">
                                                                                    {ini}
                                                                                </div>
                                                                                <div class="space-y-1">
                                                                                    <div class="text-[10px] text-on-surface-variant">
                                                                                        {sender} " · " {time}
                                                                                    </div>
                                                                                    <div class="p-3 rounded-2xl rounded-tl-none text-xs leading-relaxed border bg-surface-container border-outline-variant/15 text-on-surface">
                                                                                        {content}
                                                                                    </div>
                                                                                </div>
                                                                            </div>
                                                                        }.into_any()
                                                                    }
                                                                }
                                                            />
                                                        }.into_any()
                                                    }}
                                                </div>

                                                // Reply compose
                                                <Show when=move || is_open>
                                                    <div class="p-4 border-t border-outline-variant/10 bg-surface-container/20 flex-shrink-0 space-y-3">
                                                        <textarea
                                                            rows="2"
                                                            placeholder="Type a reply to the user… (appears as Platform Support)"
                                                            class="w-full bg-surface-container-low border border-outline-variant/30 text-on-surface text-sm rounded-lg p-3 focus:ring-1 focus:ring-primary focus:border-primary placeholder:text-on-surface-variant/40 resize-none outline-none"
                                                            prop:value=reply_text
                                                            on:input=move |ev| reply_text.set(event_target_value(&ev))
                                                        ></textarea>

                                                        <div class="flex flex-wrap justify-between items-center gap-3">
                                                            <div class="flex items-center gap-2">
                                                                <button
                                                                    on:click=move |_| show_internal_modal.set(true)
                                                                    class="btn btn-ghost btn-sm"
                                                                >
                                                                    <span class="material-symbols-outlined text-[14px]">"lock"</span>
                                                                    "Internal Note"
                                                                </button>
                                                                <button
                                                                    on:click=move |_| show_escalate_modal.set(true)
                                                                    class="btn btn-warn btn-sm"
                                                                >
                                                                    <span class="material-symbols-outlined text-[14px]">"campaign"</span>
                                                                    "Escalate"
                                                                </button>
                                                            </div>
                                                            <button
                                                                on:click=handle_send_reply
                                                                disabled=move || sending.get()
                                                                class="btn btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
                                                            >
                                                                {move || if sending.get() { "Sending…" } else { "Send Reply" }}
                                                                <span class="material-symbols-outlined text-sm">"send"</span>
                                                            </button>
                                                        </div>
                                                    </div>
                                                </Show>
                                            }.into_any()
                                        }
                                        Some(Err(_)) => view! {
                                            <div class="flex-1 flex items-center justify-center text-xs text-red-400">
                                                "Error loading thread details."
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
            </div> // end 2-panel wrapper

            // ── Internal Note Modal ──────────────────────────────────────────
            <Show when=move || show_internal_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_internal_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2">"Add Internal Staff Note"</h3>
                        <div class="p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg text-xs text-amber-400 mb-4 font-semibold flex items-center gap-2">
                            <span class="material-symbols-outlined text-sm">"lock"</span>
                            "Internal only — NEVER visible to the user."
                        </div>
                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-semibold text-on-surface-variant">"Note details *"</label>
                                <textarea
                                    rows="4"
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary placeholder:text-on-surface-variant/40"
                                    placeholder="Enter diagnostics, call highlights, resolution steps..."
                                    prop:value=internal_note_input
                                    on:input=move |ev| internal_note_input.set(event_target_value(&ev))
                                ></textarea>
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| show_internal_modal.set(false) class="btn btn-ghost">"Cancel"</button>
                            <button on:click=handle_save_internal_note class="btn btn-primary">"Save Internal Note"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Escalate Modal ───────────────────────────────────────────────
            <Show when=move || show_escalate_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_escalate_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2">"Escalate Support Thread"</h3>
                        <p class="text-xs text-on-surface-variant mb-4">"Escalations flag the thread for senior review and update the internal audit log."</p>
                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-semibold text-on-surface-variant">"Escalation Reason"</label>
                                <select
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full"
                                    on:change=move |ev| escalate_reason.set(event_target_value(&ev))
                                >
                                    <option value="SLA breach imminent">"SLA breach imminent"</option>
                                    <option value="Requires engineering access">"Requires engineering database access"</option>
                                    <option value="Billing dispute — finance">"Billing dispute — needs finance review"</option>
                                    <option value="Security / compliance hold">"Security / compliance hold"</option>
                                </select>
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-semibold text-on-surface-variant">"Assign To"</label>
                                <select
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full"
                                    on:change=move |ev| escalate_target.set(event_target_value(&ev))
                                >
                                    <option value="Engineering On-Call">"Engineering On-Call"</option>
                                    <option value="Senior Support">"Senior Support"</option>
                                    <option value="Billing Team">"Billing Team"</option>
                                    <option value="Security">"Security Team"</option>
                                </select>
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-semibold text-on-surface-variant">"Context / Notes"</label>
                                <textarea
                                    rows="3"
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full placeholder:text-on-surface-variant/40"
                                    placeholder="Provide context for the escalation team..."
                                    prop:value=escalate_notes
                                    on:input=move |ev| escalate_notes.set(event_target_value(&ev))
                                ></textarea>
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| show_escalate_modal.set(false) class="btn btn-ghost">"Cancel"</button>
                            <button on:click=handle_save_escalation class="btn btn-warn">"Escalate Thread"</button>
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
                            "You are about to start a diagnostics session as this user. All actions will be audit-logged under your staff profile."
                        </div>
                        <p class="text-xs text-on-surface-variant mb-6 leading-relaxed">
                            "This grants access to view private listings, customer billing data, and run platform adjustments. Use strictly for resolving support cases."
                        </p>
                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| show_impersonate_modal.set(false) class="btn btn-ghost">"Cancel"</button>
                            <button on:click=handle_confirm_impersonate class="btn btn-danger">"Audit & Impersonate"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div> // end main-area
    }
}
