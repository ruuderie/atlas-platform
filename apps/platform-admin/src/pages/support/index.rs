use leptos::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MockTicket {
    pub id: String,
    pub subject: String,
    pub submitter: String,
    pub tenant_slug: String,
    pub tenant_plan: String,
    pub plan_color_class: &'static str,
    pub time_ago: String,
    pub priority_color: &'static str,
    pub sla_alert: Option<String>,
    pub status: RwSignal<String>,
    pub status_class: RwSignal<&'static str>,
    pub mrr: &'static str,
    pub am: &'static str,
    pub health: &'static str,
    pub health_color: &'static str,
    pub assigned_to: RwSignal<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MockMessage {
    pub author: String,
    pub author_role: String,
    pub avatar_text: String,
    pub avatar_bg: &'static str,
    pub time: String,
    pub content: String,
    pub is_outbound: bool,
    pub is_internal: bool,
}

#[component]
pub fn SupportQueue() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // Static list of support tickets
    let tickets = RwSignal::new(vec![
        MockTicket {
            id: "ATL-1204".to_string(),
            subject: "Stripe ACH disbursements failing".to_string(),
            submitter: "billing@leiraprops.com (NI Operator)".to_string(),
            tenant_slug: "leira-chicago".to_string(),
            tenant_plan: "Enterprise".to_string(),
            plan_color_class: "text-purple-400 border-purple-500/30 bg-purple-500/5",
            time_ago: "2h ago".to_string(),
            priority_color: "bg-red-500",
            sla_alert: Some("SLA: 2h 18m remaining (Enterprise 4h SLA)".to_string()),
            status: RwSignal::new("Open".to_string()),
            status_class: RwSignal::new("text-red-400 border-red-500/30 bg-red-500/5"),
            mrr: "$4,200",
            am: "Priya S.",
            health: "Good",
            health_color: "text-emerald-400",
            assigned_to: RwSignal::new("Alex R. (me)".to_string()),
        },
        MockTicket {
            id: "ATL-1203".to_string(),
            subject: "OTA sync returning 403 for Airbnb".to_string(),
            submitter: "STR host (escalated from Folio support)".to_string(),
            tenant_slug: "miami-stays".to_string(),
            tenant_plan: "Growth".to_string(),
            plan_color_class: "text-blue-400 border-blue-500/30 bg-blue-500/5",
            time_ago: "4h ago".to_string(),
            priority_color: "bg-amber-500",
            sla_alert: None,
            status: RwSignal::new("Open".to_string()),
            status_class: RwSignal::new("text-red-400 border-red-500/30 bg-red-500/5"),
            mrr: "$1,850",
            am: "Dan H.",
            health: "Warning",
            health_color: "text-amber-500",
            assigned_to: RwSignal::new("Unassigned".to_string()),
        },
        MockTicket {
            id: "ATL-1201".to_string(),
            subject: "PDF statement generation timeout".to_string(),
            submitter: "NI Operator PMC user".to_string(),
            tenant_slug: "westside-rentals".to_string(),
            tenant_plan: "Starter".to_string(),
            plan_color_class: "text-slate-400 border-slate-500/30 bg-slate-500/5",
            time_ago: "1d ago".to_string(),
            priority_color: "bg-blue-500",
            sla_alert: None,
            status: RwSignal::new("Open".to_string()),
            status_class: RwSignal::new("text-red-400 border-red-500/30 bg-red-500/5"),
            mrr: "$350",
            am: "None",
            health: "Fair",
            health_color: "text-slate-300",
            assigned_to: RwSignal::new("Unassigned".to_string()),
        },
        MockTicket {
            id: "ATL-1199".to_string(),
            subject: "Map search not loading in production".to_string(),
            submitter: "tech@anchor.com (NI Operator)".to_string(),
            tenant_slug: "anchor-realty-tx".to_string(),
            tenant_plan: "Enterprise".to_string(),
            plan_color_class: "text-purple-400 border-purple-500/30 bg-purple-500/5",
            time_ago: "2d ago".to_string(),
            priority_color: "bg-emerald-500",
            sla_alert: None,
            status: RwSignal::new("In Progress".to_string()),
            status_class: RwSignal::new("text-blue-400 border-blue-500/30 bg-blue-500/5"),
            mrr: "$6,500",
            am: "Priya S.",
            health: "Good",
            health_color: "text-emerald-400",
            assigned_to: RwSignal::new("Priya S.".to_string()),
        },
    ]);

    // Active Selection State
    let selected_ticket_id = RwSignal::new("ATL-1204".to_string());
    
    // Derived selected ticket detail
    let selected_ticket = Signal::derive(move || {
        let sid = selected_ticket_id.get();
        tickets.get().into_iter().find(|t| t.id == sid)
    });

    // Chat threads by Ticket ID
    let thread_1204 = RwSignal::new(vec![
        MockMessage {
            author: "billing@leiraprops.com".to_string(),
            author_role: "NI Operator".to_string(),
            avatar_text: "LB".to_string(),
            avatar_bg: "linear-gradient(135deg, #7C3AED, #0A84FF)",
            time: "Jun 11 · 9:42 AM".to_string(),
            content: "Our ACH disbursements to owner accounts have been failing since 6:00 AM. Stripe shows \"bank_account_verification_required\" on all payouts. We have 24 owners expecting payments today. This is critical.".to_string(),
            is_outbound: false,
            is_internal: false,
        },
        MockMessage {
            author: "Alex R.".to_string(),
            author_role: "Atlas Platform Support".to_string(),
            avatar_text: "AR".to_string(),
            avatar_bg: "linear-gradient(135deg, #0A84FF, #00C853)",
            time: "Jun 11 · 10:05 AM".to_string(),
            content: "Hi — I've pulled your Stripe Connect account and can see the issue. Stripe Connect rolled out enhanced KYC requirements last night and your connected accounts need re-verification. I'm pushing the re-verification batch now. Should unblock within 30 min.".to_string(),
            is_outbound: true,
            is_internal: false,
        },
        MockMessage {
            author: "Alex R.".to_string(),
            author_role: "Internal Note".to_string(),
            avatar_text: "🔒".to_string(),
            avatar_bg: "linear-gradient(135deg, #FF9100, #F5A623)",
            time: "Jun 11 · 10:06 AM".to_string(),
            content: "Checked Stripe dashboard — this is the KYC enforcement that rolled out in all EU/US regions at 5:53 AM. Batch re-verify triggered via Stripe admin API. Monitoring. If not resolved by noon, escalate to Stripe account rep (Jordan M.).".to_string(),
            is_outbound: false,
            is_internal: true,
        },
        MockMessage {
            author: "billing@leiraprops.com".to_string(),
            author_role: "NI Operator".to_string(),
            avatar_text: "LB".to_string(),
            avatar_bg: "linear-gradient(135deg, #7C3AED, #0A84FF)",
            time: "Jun 11 · 10:20 AM".to_string(),
            content: "Thank you Alex — 8 of 24 are now showing as unblocked. Still waiting on 16. Can you confirm ETA?".to_string(),
            is_outbound: false,
            is_internal: false,
        },
    ]);

    let thread_1203 = RwSignal::new(vec![
        MockMessage {
            author: "STR host".to_string(),
            author_role: "Folio Submitter".to_string(),
            avatar_text: "SH".to_string(),
            avatar_bg: "linear-gradient(135deg, #F5A623, #FF3D00)",
            time: "Jun 11 · 8:12 AM".to_string(),
            content: "I try to link Airbnb channel but it keeps returning 403 Forbidden. Is there an API ban on my account?".to_string(),
            is_outbound: false,
            is_internal: false,
        },
    ]);

    let thread_1201 = RwSignal::new(vec![
        MockMessage {
            author: "operator@westside.com".to_string(),
            author_role: "NI Operator".to_string(),
            avatar_text: "WO".to_string(),
            avatar_bg: "linear-gradient(135deg, #0A84FF, #7C3AED)",
            time: "Jun 10 · 2:30 PM".to_string(),
            content: "Trying to generate PDF statements for May. The generator spins for 3 minutes then returns a timeout error. Please check ledger logs.".to_string(),
            is_outbound: false,
            is_internal: false,
        },
    ]);

    let thread_1199 = RwSignal::new(vec![
        MockMessage {
            author: "tech@anchor.com".to_string(),
            author_role: "NI Tech Operator".to_string(),
            avatar_text: "AT".to_string(),
            avatar_bg: "linear-gradient(135deg, #7C3AED, #0A84FF)",
            time: "Jun 9 · 11:15 AM".to_string(),
            content: "The map search feature on our storefront returns no results. No console errors, just blank maps.".to_string(),
            is_outbound: false,
            is_internal: false,
        },
        MockMessage {
            author: "Priya S.".to_string(),
            author_role: "Atlas Platform Support".to_string(),
            avatar_text: "PS".to_string(),
            avatar_bg: "linear-gradient(135deg, #FF3D00, #7C3AED)",
            time: "Jun 9 · 4:10 PM".to_string(),
            content: "We are investigating the PostGIS index replication delay in the Texas region. Index rebuild pending.".to_string(),
            is_outbound: true,
            is_internal: false,
        },
    ]);

    // Active thread derived signal
    let active_thread = Signal::derive(move || {
        let sid = selected_ticket_id.get();
        match sid.as_str() {
            "ATL-1204" => thread_1204.get(),
            "ATL-1203" => thread_1203.get(),
            "ATL-1201" => thread_1201.get(),
            _ => thread_1199.get(),
        }
    });

    // List Filtering State
    let filter_selection = RwSignal::new("all".to_string()); // "all", "enterprise", "escalated", "mine", "sla"
    
    let filtered_tickets = Signal::derive(move || {
        let f = filter_selection.get();
        tickets.get().into_iter().filter(|t| {
            match f.as_str() {
                "enterprise" => t.tenant_plan == "Enterprise",
                "escalated" => t.priority_color == "bg-red-500",
                "mine" => t.assigned_to.get() == "Alex R. (me)",
                "sla" => t.sla_alert.is_some(),
                _ => true,
            }
        }).collect::<Vec<MockTicket>>()
    });

    // Form response fields
    let reply_text = RwSignal::new(String::new());

    // Modal dialog controls
    let show_internal_modal = RwSignal::new(false);
    let show_escalate_modal = RwSignal::new(false);
    let show_impersonate_modal = RwSignal::new(false);

    let internal_note_input = RwSignal::new(String::new());
    let escalate_reason = RwSignal::new("SLA breach imminent".to_string());
    let escalate_target = RwSignal::new("Jordan M. (Supervisor)".to_string());
    let escalate_notes = RwSignal::new(String::new());

    // Send reply action
    let handle_send_reply = move |_| {
        let txt = reply_text.get();
        if txt.trim().is_empty() {
            toast.show_toast("Error", "Reply content cannot be empty.", "error");
            return;
        }

        let sid = selected_ticket_id.get();
        let new_msg = MockMessage {
            author: "Alex R.".to_string(),
            author_role: "Atlas Platform Support".to_string(),
            avatar_text: "AR".to_string(),
            avatar_bg: "linear-gradient(135deg, #0A84FF, #00C853)",
            time: "Just now".to_string(),
            content: txt.clone(),
            is_outbound: true,
            is_internal: false,
        };

        match sid.as_str() {
            "ATL-1204" => thread_1204.update(|m| m.push(new_msg)),
            "ATL-1203" => thread_1203.update(|m| m.push(new_msg)),
            "ATL-1201" => thread_1201.update(|m| m.push(new_msg)),
            _ => thread_1199.update(|m| m.push(new_msg)),
        }

        reply_text.set(String::new());
        toast.show_toast("Success", "Reply sent to tenant operator.", "success");
    };

    // Save internal note action
    let handle_save_internal_note = move |_| {
        let note = internal_note_input.get();
        if note.trim().is_empty() {
            toast.show_toast("Error", "Note content cannot be empty.", "error");
            return;
        }

        let sid = selected_ticket_id.get();
        let new_msg = MockMessage {
            author: "Alex R.".to_string(),
            author_role: "Internal Note".to_string(),
            avatar_text: "🔒".to_string(),
            avatar_bg: "linear-gradient(135deg, #FF9100, #F5A623)",
            time: "Just now".to_string(),
            content: note,
            is_outbound: false,
            is_internal: true,
        };

        match sid.as_str() {
            "ATL-1204" => thread_1204.update(|m| m.push(new_msg)),
            "ATL-1203" => thread_1203.update(|m| m.push(new_msg)),
            "ATL-1201" => thread_1201.update(|m| m.push(new_msg)),
            _ => thread_1199.update(|m| m.push(new_msg)),
        }

        show_internal_modal.set(false);
        internal_note_input.set(String::new());
        toast.show_toast("Success", "Internal note registered (hidden from tenant).", "success");
    };

    // Save escalation action
    let handle_save_escalation = move |_| {
        let notes = escalate_notes.get();
        let target = escalate_target.get();
        let reason = escalate_reason.get();
        
        let sid = selected_ticket_id.get();

        // Update ticket status
        if let Some(t) = tickets.get().iter().find(|tick| tick.id == sid) {
            t.status.set("Escalated".to_string());
            t.status_class.set("text-red-400 border-red-500/30 bg-red-500/10");
        }

        // Add internal note about escalation
        let escalation_msg = MockMessage {
            author: "System Log".to_string(),
            author_role: "Escalation Action".to_string(),
            avatar_text: "🚨".to_string(),
            avatar_bg: "linear-gradient(135deg, #E5484D, #FF3D00)",
            time: "Just now".to_string(),
            content: format!("Ticket escalated to {} (Reason: {}). Notes: {}", target, reason, notes),
            is_outbound: false,
            is_internal: true,
        };

        match sid.as_str() {
            "ATL-1204" => thread_1204.update(|m| m.push(escalation_msg)),
            "ATL-1203" => thread_1203.update(|m| m.push(escalation_msg)),
            "ATL-1201" => thread_1201.update(|m| m.push(escalation_msg)),
            _ => thread_1199.update(|m| m.push(escalation_msg)),
        }

        show_escalate_modal.set(false);
        escalate_notes.set(String::new());
        toast.show_toast("Warning", &format!("Ticket escalated to {}.", target), "warn");
    };

    // Confirm Impersonation action
    let handle_confirm_impersonate = move |_| {
        let t = selected_ticket.get().unwrap();
        show_impersonate_modal.set(false);
        toast.show_toast("Warning", &format!("⚠ Impersonation token active for {}. Audit log registered.", t.tenant_slug), "warn");
    };

    view! {
        <div class="h-[calc(100vh-140px)] flex bg-surface border border-outline-variant/10 rounded-2xl overflow-hidden shadow-lg text-on-surface">
            // Left ticket list panel
            <div class="w-80 flex-shrink-0 border-r border-outline-variant/10 flex flex-col bg-surface-container/20">
                <div class="p-4 border-b border-outline-variant/10 flex-shrink-0">
                    <div class="flex items-center justify-between font-bold text-sm">
                        <span>"Support Queue"</span>
                        <span class="px-2 py-0.5 text-[10px] font-bold rounded-full bg-red-500/10 border border-red-500/30 text-red-400">
                            {move || tickets.get().iter().filter(|t| t.status.get() == "Open").count().to_string()} " Open"
                        </span>
                    </div>
                    <p class="text-[10.5px] text-on-surface-variant mt-1">"Tenant operational issues and infrastructure tickets"</p>
                </div>

                // Queue filtering pills
                <div class="p-3 border-b border-outline-variant/5 flex gap-1.5 overflow-x-auto scrollbar-none flex-shrink-0">
                    {
                        let click_s = "all".to_string(); // bypass compiler move binding
                        let click_e = "enterprise".to_string();
                        let click_esc = "escalated".to_string();
                        let click_m = "mine".to_string();
                        let click_sla = "sla".to_string();
                        view! {
                            <button on:click=move |_| filter_selection.set(click_s.clone()) class=move || format!("px-2.5 py-1 text-[10px] font-bold border rounded-lg whitespace-nowrap transition-all {}", if filter_selection.get() == "all" { "bg-primary-container border-primary text-primary" } else { "bg-[#05183c]/20 border-outline-variant/20 text-on-surface-variant hover:text-on-surface" })>"All Sites"</button>
                            <button on:click=move |_| filter_selection.set(click_e.clone()) class=move || format!("px-2.5 py-1 text-[10px] font-bold border rounded-lg whitespace-nowrap transition-all {}", if filter_selection.get() == "enterprise" { "bg-primary-container border-primary text-primary" } else { "bg-[#05183c]/20 border-outline-variant/20 text-on-surface-variant hover:text-on-surface" })>"Enterprise ★"</button>
                            <button on:click=move |_| filter_selection.set(click_esc.clone()) class=move || format!("px-2.5 py-1 text-[10px] font-bold border rounded-lg whitespace-nowrap transition-all {}", if filter_selection.get() == "escalated" { "bg-red-500/15 border-red-500/40 text-red-400" } else { "bg-[#05183c]/20 border-outline-variant/20 text-on-surface-variant hover:text-on-surface" })>"Escalated"</button>
                            <button on:click=move |_| filter_selection.set(click_m.clone()) class=move || format!("px-2.5 py-1 text-[10px] font-bold border rounded-lg whitespace-nowrap transition-all {}", if filter_selection.get() == "mine" { "bg-primary-container border-primary text-primary" } else { "bg-[#05183c]/20 border-outline-variant/20 text-on-surface-variant hover:text-on-surface" })>"Mine"</button>
                            <button on:click=move |_| filter_selection.set(click_sla.clone()) class=move || format!("px-2.5 py-1 text-[10px] font-bold border rounded-lg whitespace-nowrap transition-all {}", if filter_selection.get() == "sla" { "bg-primary-container border-primary text-primary" } else { "bg-[#05183c]/20 border-outline-variant/20 text-on-surface-variant hover:text-on-surface" })>"SLA at Risk"</button>
                        }
                    }
                </div>

                // Ticket Items Scroller
                <div class="flex-1 overflow-y-auto divide-y divide-outline-variant/5">
                    <For 
                        each=move || filtered_tickets.get()
                        key=|t| t.id.clone()
                        children=move |t| {
                            let t_val = StoredValue::new(t);
                            let tid = t_val.with_value(|v| v.id.clone());
                            let is_sel = Signal::derive({
                                let tid_check = tid.clone();
                                move || selected_ticket_id.get() == tid_check
                            });
                            
                            view! {
                                <div 
                                    on:click={
                                        let tid_click = tid.clone();
                                        move |_| selected_ticket_id.set(tid_click.clone())
                                    }
                                    class=move || format!(
                                        "p-4 cursor-pointer transition-all border-l-2 {}",
                                        if is_sel.get() { "bg-surface-bright/10 border-primary" } else { "border-transparent hover:bg-surface-bright/5" }
                                    )
                                >
                                    <div class="space-y-1">
                                        <div class="flex items-center justify-between text-[10px] font-semibold text-on-surface-variant">
                                            <div class="flex items-center gap-1.5">
                                                <span class=move || t_val.with_value(|v| format!("px-1.5 py-0.2 rounded font-bold border {}", v.plan_color_class))>
                                                    {move || t_val.with_value(|v| v.tenant_plan.clone())}
                                                </span>
                                                <span class="font-mono text-on-surface">{move || t_val.with_value(|v| v.tenant_slug.clone())}</span>
                                            </div>
                                            <span>{move || t_val.with_value(|v| v.time_ago.clone())}</span>
                                        </div>
                                        
                                        <div class="flex items-start gap-2 justify-between">
                                            <h4 class="text-xs font-bold text-on-surface line-clamp-1">"#" {tid.clone()} " — " {move || t_val.with_value(|v| v.subject.clone())}</h4>
                                            <span class=move || t_val.with_value(|v| format!("w-2 h-2 rounded-full mt-1.5 flex-shrink-0 {}", v.priority_color))></span>
                                        </div>

                                        <p class="text-[10px] text-on-surface-variant truncate">{move || t_val.with_value(|v| v.submitter.clone())}</p>
                                        
                                        <Show when=move || t_val.with_value(|v| v.sla_alert.is_some())>
                                            <div class="flex items-center gap-1 text-[9.5px] font-semibold text-red-400 mt-1">
                                                <span class="material-symbols-outlined text-[11px]">"warning"</span>
                                                {move || t_val.with_value(|v| v.sla_alert.clone().unwrap_or_default())}
                                            </div>
                                        </Show>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </div>

            // Right ticket workspace details panel
            <div class="flex-1 flex flex-col bg-surface overflow-hidden">
                {move || selected_ticket.get().map(|t| {
                    let t_resolve = t.clone();
                    
                    view! {
                        // Header detail section
                        <div class="p-5 border-b border-outline-variant/10 flex justify-between items-start flex-shrink-0 gap-4">
                            <div class="space-y-1">
                                <h2 class="text-base font-bold text-on-surface">"#" {t.id.clone()} " — " {t.subject.clone()}</h2>
                                <div class="flex items-center gap-3 text-xs">
                                    <span class=format!("px-2 py-0.5 rounded text-[10px] font-bold border {}", t.plan_color_class)>
                                        {t.tenant_plan.clone()}
                                    </span>
                                    <span class=move || format!("px-2 py-0.5 rounded text-[10px] font-bold border {}", t.status_class.get())>
                                        {move || t.status.get()}
                                    </span>
                                    <span class="font-mono bg-surface-container/60 border border-outline-variant/20 px-2 py-0.5 rounded text-on-surface-variant font-bold">
                                        {t.tenant_slug.clone()}
                                    </span>
                                    <span class="text-on-surface-variant">"Submitted 2 days ago"</span>
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
                                
                                <select 
                                    class="bg-surface-container border border-outline-variant/30 text-on-surface text-xs rounded-lg px-2.5 py-1.5 focus:ring-1 focus:ring-primary focus:border-primary font-medium"
                                    on:change={
                                        let t_assign = t.clone();
                                        move |ev| {
                                            let assigned = event_target_value(&ev);
                                            t_assign.assigned_to.set(assigned.clone());
                                            toast.show_toast("Info", &format!("Ticket owner updated to {}.", assigned), "info");
                                        }
                                    }
                                >
                                    <option value="Unassigned" selected=move || t.assigned_to.get() == "Unassigned">"Unassigned"</option>
                                    <option value="Alex R. (me)" selected=move || t.assigned_to.get() == "Alex R. (me)">"Alex R. (me)"</option>
                                    <option value="Priya S." selected=move || t.assigned_to.get() == "Priya S.">"Priya S."</option>
                                    <option value="Dan H." selected=move || t.assigned_to.get() == "Dan H.">"Dan H."</option>
                                </select>

                                <button 
                                    on:click=move |_| {
                                        t_resolve.status.set("Resolved".to_string());
                                        t_resolve.status_class.set("text-emerald-400 border-emerald-500/30 bg-emerald-500/5");
                                        toast.show_toast("Success", "Ticket marked as resolved.", "success");
                                    }
                                    class="px-3 py-1.5 text-xs font-bold text-on-primary bg-emerald-600 hover:bg-emerald-700 active:scale-95 transition-all rounded-lg flex items-center gap-1"
                                >
                                    <span class="material-symbols-outlined text-sm">"check"</span>
                                    "Resolve"
                                </button>
                            </div>
                        </div>

                        // Customer infrastructure metrics bar
                        <div class="px-5 py-2.5 bg-surface-container-low border-b border-outline-variant/10 flex items-center justify-between text-xs flex-shrink-0">
                            <div class="flex items-center gap-6">
                                <span>"NI: " <strong class="text-on-surface">{t.tenant_slug.clone()}</strong></span>
                                <span>"MRR: " <strong class="text-on-surface">{t.mrr}</strong></span>
                                <span>"AM: " <strong class="text-on-surface">{t.am}</strong></span>
                                <span>"Health: " <strong class=t.health_color>{t.health}</strong></span>
                                <Show when=move || t.sla_alert.is_some()>
                                    <span class="text-red-400">"SLA: " <strong>"2h remaining"</strong></span>
                                </Show>
                            </div>
                            <a 
                                href=format!("/apps/{}", t.tenant_slug)
                                class="text-primary hover:underline font-semibold flex items-center gap-0.5"
                            >
                                "View NI Details"
                                <span class="material-symbols-outlined text-[10px]">"arrow_forward"</span>
                            </a>
                        </div>

                        // Conversation message threads scroll box
                        <div class="flex-1 overflow-y-auto p-5 space-y-4">
                            <For 
                                each=move || active_thread.get()
                                key=|msg| format!("{}-{}-{}", msg.author, msg.time, msg.content.chars().take(20).collect::<String>())
                                children=move |msg| {
                                    view! {
                                        <div class=format!("flex gap-3 max-w-[80%] {}", if msg.is_outbound { "ml-auto flex-row-reverse" } else { "" })>
                                            <div 
                                                class="w-7 h-7 rounded-full flex items-center justify-center text-[10px] font-bold text-white flex-shrink-0 align-self-end border border-white/5"
                                                style=format!("background: {}", msg.avatar_bg)
                                            >
                                                {msg.avatar_text.clone()}
                                            </div>
                                            
                                            <div class="space-y-1">
                                                <div class=format!("text-[10px] text-on-surface-variant {}", if msg.is_outbound { "text-right" } else { "" })>
                                                    {msg.author.clone()} " · " {msg.author_role.clone()} " · " {msg.time.clone()}
                                                </div>
                                                
                                                <div class=format!(
                                                    "p-3 rounded-2xl text-xs leading-relaxed border {}",
                                                    if msg.is_internal {
                                                        "bg-amber-500/10 border-amber-500/30 text-on-surface"
                                                    } else if msg.is_outbound {
                                                        "bg-primary-container border-primary/20 text-on-surface rounded-tr-none"
                                                    } else {
                                                        "bg-surface-container border-outline-variant/15 text-on-surface rounded-tl-none"
                                                    }
                                                )>
                                                    <Show when=move || msg.is_internal>
                                                        <div class="text-[9px] font-bold text-amber-500 uppercase tracking-wider mb-1">"🔒 Private Staff Note"</div>
                                                    </Show>
                                                    {msg.content.clone()}
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }
                            />
                        </div>

                        // Reply composition area
                        <div class="p-4 border-t border-outline-variant/10 bg-surface-container/20 flex-shrink-0 space-y-3">
                            <textarea 
                                rows="2"
                                placeholder=format!("Send a reply to {} operator... (External communication)", t.tenant_slug)
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

                                    <button 
                                        on:click=move |_| toast.show_toast("Info", &format!("Accessing Stripe connect credentials for {}...", t.tenant_slug), "info")
                                        class="px-3 py-1.5 text-xs font-semibold bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 rounded-lg flex items-center gap-1"
                                    >
                                        <span class="material-symbols-outlined text-[14px]">"credit_card"</span>
                                        "Stripe Console"
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
                    }
                })}
            </div>

            // Add Internal Note Modal
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

            // Escalate Ticket Modal
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

            // Impersonate Tenant Caution Modal
            <Show when=move || show_impersonate_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_impersonate_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold text-red-400 mb-2 flex items-center gap-1.5">
                            <span class="material-symbols-outlined">"warning"</span>
                            "Impersonate tenant operator view"
                        </h3>
                        
                        <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-lg text-xs text-red-400 mb-4 leading-relaxed">
                            "You are about to start a diagnostics session impersonating: " <strong class="underline">{move || selected_ticket.get().map(|t| t.tenant_slug.clone()).unwrap_or_default()}</strong> ". All actions will be audit-logged under your staff profile."
                        </div>

                        <p class="text-xs text-on-surface-variant mb-6 leading-relaxed">
                            "This grants access to view private listings, customer billing cards, and run platform adjustments. Use strictly for resolving tickets."
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
