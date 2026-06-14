use leptos::prelude::*;

#[derive(Clone, Debug)]
pub struct VerificationRequest {
    pub id: String,
    pub entity_name: String,
    pub req_type: String,
    pub status: String,
    pub age_days: u32,
    pub document_count: u32,
    pub priority_color: &'static str,
    pub badge_color: &'static str,
}

#[derive(Clone, Debug)]
pub struct ChecklistItem {
    #[allow(dead_code)]
    pub id: usize,
    pub label: &'static str,
    pub note: &'static str,
    pub checked: RwSignal<bool>,
}

#[derive(Clone, Debug)]
pub struct NoteHistoryRecord {
    pub author: &'static str,
    pub text: String,
    pub timestamp: &'static str,
}

#[component]
pub fn Verification() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // UI state signals
    let selected_index = RwSignal::new(0);
    let active_filter = RwSignal::new("all".to_string());
    let active_rev_tab = RwSignal::new("rv-documents".to_string());
    
    // Notes history state
    let reviewer_notes = RwSignal::new("EIN verified via SS-4 document. Operating agreement looks current. Main concern is the 2019 state registration — reached out to contact on Jun 08 to request updated document. Awaiting response.".to_string());
    let note_history = RwSignal::new(vec![
        NoteHistoryRecord {
            author: "JD",
            text: "Contacted Ruud Erie to request updated 2024 state registration document".to_string(),
            timestamp: "Jun 08, 2026 · 11:30 UTC"
        },
        NoteHistoryRecord {
            author: "JD",
            text: "Opened verification request — EIN and operating agreement verified".to_string(),
            timestamp: "Jun 03, 2026 · 16:10 UTC"
        }
    ]);
    
    // Dialog modals
    let show_approve_modal = RwSignal::new(false);
    let show_reject_modal = RwSignal::new(false);
    let show_info_request_modal = RwSignal::new(false);

    // Hardcoded verification queue items matching the mockup
    let queue_items = vec![
        VerificationRequest {
            id: "t_8a91f3d2".to_string(),
            entity_name: "Nexus Property Group".to_string(),
            req_type: "Business".to_string(),
            status: "pending".to_string(),
            age_days: 7,
            document_count: 3,
            priority_color: "bg-error",
            badge_color: "text-error border-error/30 bg-error-container/20",
        },
        VerificationRequest {
            id: "t_l_jcs_001".to_string(),
            entity_name: "João Carlos Silva".to_string(),
            req_type: "Identity".to_string(),
            status: "pending".to_string(),
            age_days: 4,
            document_count: 2,
            priority_color: "bg-amber-400",
            badge_color: "text-amber-400 border-amber-500/20 bg-amber-500/10",
        },
        VerificationRequest {
            id: "t_comp_ruud".to_string(),
            entity_name: "Ruud Logistics Corp".to_string(),
            req_type: "Document".to_string(),
            status: "pending".to_string(),
            age_days: 1,
            document_count: 1,
            priority_color: "bg-primary",
            badge_color: "text-primary border-primary/20 bg-primary/10",
        },
        VerificationRequest {
            id: "t_str_vizcaya".to_string(),
            entity_name: "Vizcaya STR Partners".to_string(),
            req_type: "Business".to_string(),
            status: "review".to_string(),
            age_days: 3,
            document_count: 4,
            priority_color: "bg-purple-400",
            badge_color: "text-purple-400 border-purple-500/20 bg-purple-500/10",
        },
        VerificationRequest {
            id: "t_ind_ana".to_string(),
            entity_name: "Ana Carvalho".to_string(),
            req_type: "Identity".to_string(),
            status: "review".to_string(),
            age_days: 2,
            document_count: 2,
            priority_color: "bg-purple-400",
            badge_color: "text-amber-400 border-amber-500/20 bg-amber-500/10",
        },
        VerificationRequest {
            id: "t_approved_merid".to_string(),
            entity_name: "Meridian Brokerage LLC".to_string(),
            req_type: "Business".to_string(),
            status: "approved".to_string(),
            age_days: 12,
            document_count: 3,
            priority_color: "bg-emerald-400",
            badge_color: "text-emerald-400 border-emerald-500/20 bg-emerald-500/10",
        },
        VerificationRequest {
            id: "t_rejected_unk".to_string(),
            entity_name: "Unknown Entity Corp".to_string(),
            req_type: "Identity".to_string(),
            status: "rejected".to_string(),
            age_days: 18,
            document_count: 1,
            priority_color: "bg-on-surface-variant/30",
            badge_color: "text-on-surface-variant/40 border-outline-variant/30 bg-surface-container",
        }
    ];

    // Checklist items matching the mockup
    let checklist_items = vec![
        ChecklistItem { id: 1, label: "EIN / Tax ID confirmed", note: "Verified via IRS SS-4", checked: RwSignal::new(true) },
        ChecklistItem { id: 2, label: "Business name matches submitted entity name", note: "✓ Match", checked: RwSignal::new(true) },
        ChecklistItem { id: 3, label: "Operating agreement / LLC articles present", note: "Signed PDF attached", checked: RwSignal::new(true) },
        ChecklistItem { id: 4, label: "State registration document current (within 2 years)", note: "⚠ 2019 — review", checked: RwSignal::new(false) },
        ChecklistItem { id: 5, label: "Primary contact identity verified (ID check)", note: "Pending", checked: RwSignal::new(false) },
        ChecklistItem { id: 6, label: "No active regulatory flags (FMCSA / DOT cross-check)", note: "Auto-check available", checked: RwSignal::new(false) },
        ChecklistItem { id: 7, label: "Billing address matches registered business address", note: "✓ Match", checked: RwSignal::new(true) },
    ];

    // Derived filtered queue
    let filtered_items = Signal::derive({
        let items = queue_items.clone();
        move || {
            let filter = active_filter.get();
            if filter == "all" {
                items.clone()
            } else {
                items.iter()
                    .filter(|item| item.status == filter)
                    .cloned()
                    .collect::<Vec<_>>()
            }
        }
    });

    // Handle adding note
    let add_reviewer_note = move |text: String| {
        if text.trim().is_empty() { return; }
        note_history.update(|h| {
            h.insert(0, NoteHistoryRecord {
                author: "JD",
                text: text.clone(),
                timestamp: "Just now · UTC"
            });
        });
        toast.show_toast("Success", "Review note added.", "success");
    };

    view! {
        <div class="h-[calc(100vh-140px)] flex border border-outline-variant/20 rounded-2xl overflow-hidden bg-surface-container-low shadow-sm">
            // ── LEFT PANE: Queue ──
            <div class="w-[380px] border-r border-outline-variant/20 flex flex-col overflow-hidden bg-surface-container-low/60">
                <div class="p-4 border-b border-outline-variant/20 bg-surface-container-high/20 shrink-0">
                    <div class="flex items-center justify-between mb-3">
                        <div class="text-sm font-extrabold text-on-surface tracking-tight flex items-center gap-2">
                            "Verification Queue"
                            <span class="px-2 py-0.5 rounded-full text-[10px] font-black bg-error-container/20 text-error border border-error/30">"3 Pending"</span>
                        </div>
                        <button class="btn-ghost text-[10px] px-2 py-1 border border-outline-variant/30 rounded" on:click=move |_| toast.show_toast("Info", "Verification logs compiled.", "info")>"Export"</button>
                    </div>
                    <div class="flex flex-wrap gap-1">
                        {
                            let filter_pill = move |id: &str, label: &str| {
                                let id = id.to_string();
                                let label = label.to_string();
                                let id_class = id.clone();
                                let id_click = id.clone();
                                view! {
                                    <button 
                                        class=move || if active_filter.get() == id_class { "px-2.5 py-1 text-[10.5px] rounded bg-surface-container-highest border border-outline-variant text-on-surface font-semibold transition-all shrink-0" } else { "px-2.5 py-1 text-[10.5px] rounded text-on-surface-variant hover:text-on-surface transition-all shrink-0 bg-transparent border border-transparent" }
                                        on:click=move |_| active_filter.set(id_click.clone())
                                    >
                                        {label.clone()}
                                    </button>
                                }
                            };
                            view! {
                                {filter_pill("all", "All")}
                                {filter_pill("pending", "Pending")}
                                {filter_pill("review", "In Review")}
                                {filter_pill("approved", "Approved")}
                                {filter_pill("rejected", "Rejected")}
                            }
                        }
                    </div>
                </div>
                
                <div class="flex-1 overflow-y-auto divide-y divide-outline-variant/10">
                    {move || {
                        let selected = selected_index.get();
                        filtered_items.get().into_iter().enumerate().map(|(idx, item)| {
                            let is_selected = selected == idx;
                            view! {
                                <div 
                                    class=move || if is_selected { "p-4 flex gap-3 cursor-pointer bg-surface-container-highest/60 border-l-2 border-primary transition-colors" } else { "p-4 flex gap-3 cursor-pointer hover:bg-surface-bright/5 transition-colors border-l-2 border-transparent" }
                                    on:click=move |_| selected_index.set(idx)
                                >
                                    <div class=format!("w-1 rounded shrink-0 {}", item.priority_color)></div>
                                    <div class="flex-1 min-w-0">
                                        <div class="text-xs font-bold text-on-surface truncate">{item.entity_name.clone()}</div>
                                        <div class="text-[10px] text-on-surface-variant/70 mt-0.5 truncate">{format!("{} · {} · {} docs", item.req_type, item.id, item.document_count)}</div>
                                    </div>
                                    <div class="flex flex-col items-end gap-1.5 shrink-0">
                                        <span class=format!("text-[10px] font-bold {}", if item.age_days >= 7 { "text-error" } else if item.age_days >= 4 { "text-amber-400" } else { "text-on-surface-variant/60" })>{format!("{}d", item.age_days)}</span>
                                        <span class=format!("px-1.5 py-0.5 rounded border text-[9px] font-bold uppercase tracking-wider {}", item.badge_color)>{item.req_type.clone()}</span>
                                    </div>
                                </div>
                            }
                        }).collect_view()
                    }}
                </div>
            </div>

            // ── RIGHT PANE: Details Workspace ──
            {move || {
                let checklist_items = checklist_items.clone();
                let current_idx = selected_index.get();
                let list = filtered_items.get();
                if list.is_empty() || current_idx >= list.len() {
                    return view! {
                        <div class="flex-1 flex flex-col items-center justify-center text-on-surface-variant p-8 bg-surface-container-low/30">
                            <span class="material-symbols-outlined text-4xl text-on-surface-variant/40 mb-3">"folder_shared"</span>
                            <div class="text-sm font-semibold">"No Request Selected"</div>
                            <div class="text-xs text-on-surface-variant/60 mt-1">"Select a pending registration from the left queue."</div>
                        </div>
                    }.into_any();
                }
                let request = &list[current_idx];
                let req_id = request.id.clone();
                let entity_title = request.entity_name.clone();
                let type_label = request.req_type.clone();
                let age = request.age_days;

                view! {
                    <div class="flex-1 flex flex-col overflow-hidden">
                        // Detail Header actions
                        <div class="p-6 border-b border-outline-variant/20 shrink-0 bg-surface-container-low">
                            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                                <div>
                                    <div class="flex items-center gap-2 text-[10px] text-on-surface-variant/80 mb-1">
                                        <span class=format!("px-2 py-0.5 rounded border font-bold uppercase tracking-wider {}", request.badge_color)>{format!("{} Verification", type_label)}</span>
                                        <span>"·"</span>
                                        <span class=format!("font-bold {}", if age >= 7 { "text-error" } else { "text-amber-400" })>{format!("{} days pending", age)}</span>
                                    </div>
                                    <h2 class="text-xl font-bold text-on-surface tracking-tight">{entity_title.clone()}</h2>
                                    <div class="text-[11px] text-on-surface-variant/70 font-mono mt-0.5">{format!("tenant_id: {} · Submitted Jun 03, 2026 · 14:44 UTC", req_id)}</div>
                                </div>
                                <div class="flex items-center gap-2">
                                    <button class="btn-ghost text-xs px-3.5 py-2 border border-outline-variant/30 rounded-lg hover:bg-surface-bright/20 transition-all font-semibold" on:click=move |_| show_info_request_modal.set(true)>"Request More Info"</button>
                                    <button class="bg-error-container/20 border border-error/30 text-error hover:bg-error-container/30 text-xs px-3.5 py-2 rounded-lg font-semibold transition-all" on:click=move |_| show_reject_modal.set(true)>"Reject"</button>
                                    <button class="btn-primary-gradient text-xs px-4 py-2 rounded-lg font-semibold text-on-primary-container shadow hover:opacity-90 active:scale-95 transition-all" on:click=move |_| show_approve_modal.set(true)>"Approve"</button>
                                </div>
                            </div>
                        </div>

                        // Detail Tabs navigation
                        <div class="flex border-b border-outline-variant/20 px-6 overflow-x-auto shrink-0 select-none bg-surface-container-low/40">
                            {
                                let rev_tab_btn = move |id: &str, label: &str| {
                                    let id = id.to_string();
                                    let label = label.to_string();
                                    let id_class = id.clone();
                                    let id_click = id.clone();
                                    view! {
                                        <button 
                                            class=move || if active_rev_tab.get() == id_class { "px-4 py-2.5 text-xs font-bold text-primary border-b-2 border-primary transition-all shrink-0 bg-transparent" } else { "px-4 py-2.5 text-xs text-on-surface-variant hover:text-on-surface transition-all shrink-0 bg-transparent" }
                                            on:click=move |_| active_rev_tab.set(id_click.clone())
                                        >
                                            {label.clone()}
                                        </button>
                                    }
                                };
                                view! {
                                    {rev_tab_btn("rv-documents", "Documents")}
                                    {rev_tab_btn("rv-checklist", "Review Checklist")}
                                    {rev_tab_btn("rv-entity", "Entity Summary")}
                                    {rev_tab_btn("rv-notes", "Reviewer Notes")}
                                }
                            }
                        </div>

                        // Detail Workspace Viewport
                        <div class="flex-1 overflow-y-auto p-6 bg-surface-container-low/10">
                            // TAB: Documents
                            <Show when=move || active_rev_tab.get() == "rv-documents">
                                <div class="space-y-4">
                                    <h4 class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Submitted Documents · G-02 Vault"</h4>
                                    
                                    <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex items-center justify-between gap-4">
                                        <div class="w-10 h-10 rounded-lg bg-surface-container-high border border-outline-variant/30 flex items-center justify-center text-xs font-bold text-on-surface-variant/80 shrink-0">"PDF"</div>
                                        <div class="flex-1 min-w-0">
                                            <div class="text-xs font-bold text-on-surface truncate">"Business_Registration_NexusPG.pdf"</div>
                                            <div class="text-[10px] text-on-surface-variant/60 font-mono mt-0.5 truncate">"4.2 MB · Uploaded Jun 03, 2026 · SHA-256: 8a3f…c14b"</div>
                                        </div>
                                        <div class="flex items-center gap-3">
                                            <span class="px-2 py-0.5 rounded border border-amber-500/20 text-amber-400 text-[9px] font-bold uppercase tracking-wider bg-amber-500/5">"⚠ Review"</span>
                                            <button class="btn-ghost text-[10px] font-bold px-2 py-1 border border-outline-variant/30 rounded" on:click=move |_| toast.show_toast("Document Viewer", "Loading document preview in sandbox...", "info")>"View ↗"</button>
                                        </div>
                                    </div>

                                    <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex items-center justify-between gap-4">
                                        <div class="w-10 h-10 rounded-lg bg-surface-container-high border border-outline-variant/30 flex items-center justify-center text-xs font-bold text-on-surface-variant/80 shrink-0">"PDF"</div>
                                        <div class="flex-1 min-w-0">
                                            <div class="text-xs font-bold text-on-surface truncate">"EIN_Confirmation_SS4.pdf"</div>
                                            <div class="text-[10px] text-on-surface-variant/60 font-mono mt-0.5 truncate">"1.1 MB · Uploaded Jun 03, 2026 · SHA-256: 2c8d…f091"</div>
                                        </div>
                                        <div class="flex items-center gap-3">
                                            <span class="px-2 py-0.5 rounded border border-emerald-500/20 text-emerald-400 text-[9px] font-bold uppercase tracking-wider bg-emerald-500/5">"✓ Verified"</span>
                                            <button class="btn-ghost text-[10px] font-bold px-2 py-1 border border-outline-variant/30 rounded" on:click=move |_| toast.show_toast("Document Viewer", "Loading document preview in sandbox...", "info")>"View ↗"</button>
                                        </div>
                                    </div>

                                    <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 flex items-center justify-between gap-4">
                                        <div class="w-10 h-10 rounded-lg bg-surface-container-high border border-outline-variant/30 flex items-center justify-center text-xs font-bold text-on-surface-variant/80 shrink-0">"PNG"</div>
                                        <div class="flex-1 min-w-0">
                                            <div class="text-xs font-bold text-on-surface truncate">"Operating_Agreement_Signed.png"</div>
                                            <div class="text-[10px] text-on-surface-variant/60 font-mono mt-0.5 truncate">"2.8 MB · Uploaded Jun 03, 2026 · SHA-256: 71a2…0e43"</div>
                                        </div>
                                        <div class="flex items-center gap-3">
                                            <span class="px-2 py-0.5 rounded border border-outline-variant/40 text-on-surface-variant/50 text-[9px] font-bold uppercase tracking-wider bg-surface-container">"○ Pending"</span>
                                            <button class="btn-ghost text-[10px] font-bold px-2 py-1 border border-outline-variant/30 rounded" on:click=move |_| toast.show_toast("Document Viewer", "Loading document preview in sandbox...", "info")>"View ↗"</button>
                                        </div>
                                    </div>

                                    <div class="bg-amber-500/10 border border-amber-500/20 p-4 rounded-xl space-y-1">
                                        <div class="text-xs font-bold text-amber-400">"⚠ Document concern — Business Registration"</div>
                                        <p class="text-xs text-on-surface-variant/90 leading-relaxed">"The business registration PDF is dated 2019 and may be expired. Confirm whether this state of registration is still active or request an updated document."</p>
                                    </div>
                                </div>
                            </Show>

                            // TAB: Review Checklist
                            <Show when=move || active_rev_tab.get() == "rv-checklist">
                                <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                                    <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                        <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Business Identity Verification Checklist"</h3>
                                    </div>
                                    <div class="divide-y divide-outline-variant/10">
                                        {
                                            checklist_items.iter().cloned().map(|item| {
                                                let state = item.checked;
                                                let label = item.label;
                                                view! {
                                                    <div class="p-4 flex items-center justify-between gap-4 text-xs hover:bg-surface-bright/5 transition-colors">
                                                        <div class="flex items-center gap-3">
                                                            <div 
                                                                class=move || if state.get() { "w-4.5 h-4.5 rounded border border-primary bg-primary flex items-center justify-center text-on-primary-container text-[10px] font-bold cursor-pointer transition-all shrink-0" } else { "w-4.5 h-4.5 rounded border border-outline-variant bg-transparent flex items-center justify-center text-transparent cursor-pointer transition-all shrink-0 hover:border-primary" }
                                                                on:click=move |_| state.update(|v| *v = !*v)
                                                            >
                                                                "✓"
                                                            </div>
                                                            <span class=move || if state.get() { "text-on-surface-variant/60 line-through" } else { "text-on-surface font-medium" }>{label}</span>
                                                        </div>
                                                        <span class=format!("text-[10px] font-mono {}", if item.note.contains("⚠") { "text-error font-bold" } else if item.note.contains("Pending") { "text-amber-400" } else { "text-on-surface-variant/60" })>{item.note}</span>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }
                                    </div>
                                </div>
                            </Show>

                            // TAB: Entity Summary
                            <Show when=move || active_rev_tab.get() == "rv-entity">
                                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                                        <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"Entity Profile"</h3>
                                        </div>
                                        <div class="divide-y divide-outline-variant/10 text-xs">
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"Organization"</span>
                                                <span class="font-bold">"Nexus Property Group LLC"</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"EIN"</span>
                                                <span class="font-mono text-on-surface/80">"82-4419201"</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"State of Incorporation"</span>
                                                <span>"Florida, USA"</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"NAICS"</span>
                                                <span class="font-mono text-on-surface-variant/80">"531110 — Lessors of Residential Buildings"</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"DUNS"</span>
                                                <span class="font-mono text-on-surface-variant/80">"88-044-9120"</span>
                                            </div>
                                            <div class="flex justify-between items-start px-5 py-3 gap-8">
                                                <span class="text-on-surface-variant">"Primary Contact"</span>
                                                <span class="text-right">"Ruud Salym Erie · ruud@nexusproperties.com"</span>
                                            </div>
                                            <div class="flex justify-between items-start px-5 py-3 gap-8">
                                                <span class="text-on-surface-variant">"Business Address"</span>
                                                <span class="text-right text-on-surface-variant/80">"2100 Ponce de Leon Blvd, Coral Gables, FL"</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"Tenant Status"</span>
                                                <span class="text-emerald-400 font-semibold">"● Active (since Feb 2024)"</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"Platform MRR"</span>
                                                <span class="font-mono font-bold text-primary">$4,800</span>
                                            </div>
                                        </div>
                                    </div>

                                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                                        <div class="flex justify-between items-center px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40">
                                            <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant">"FMCSA / DOT Cross-Check"</h3>
                                            <button 
                                                class="btn-ghost text-[10px] font-bold px-2 py-1 border border-outline-variant/30 rounded"
                                                on:click=move |_| toast.show_toast("Cross check initiated", "DOT/FMCSA screening database query launched.", "success")
                                            >
                                                "Run Check"
                                            </button>
                                        </div>
                                        <div class="divide-y divide-outline-variant/10 text-xs">
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"DOT Match"</span>
                                                <span class="text-on-surface-variant/50 font-italic">"Not applicable (non-carrier)"</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"FMCSA Match"</span>
                                                <span class="text-on-surface-variant/50 font-italic">"Not applicable (non-carrier)"</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"OFAC Sanctions"</span>
                                                <span class="text-emerald-400 font-bold">"✓ Clear"</span>
                                            </div>
                                            <div class="flex justify-between items-center px-5 py-3">
                                                <span class="text-on-surface-variant">"Last Checked"</span>
                                                <span class="text-on-surface-variant/60 font-mono">"Jun 03, 2026"</span>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </Show>

                            // TAB: Reviewer Notes
                            <Show when=move || active_rev_tab.get() == "rv-notes">
                                <div class="space-y-6">
                                    <div class="space-y-2">
                                        <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Write Internal Review Note"</label>
                                        <div class="flex gap-2 items-start">
                                            <textarea 
                                                class="flex-1 bg-surface-container border border-outline-variant/40 rounded-lg p-3 text-xs text-on-surface outline-none focus:border-primary min-h-[80px] resize-y" 
                                                placeholder="Add internal reviewer logs..."
                                                on:input=move |ev| reviewer_notes.set(event_target_value(&ev))
                                                prop:value=reviewer_notes
                                            ></textarea>
                                            <button 
                                                class="btn-primary-gradient px-4 py-2 text-xs rounded-lg font-semibold text-on-primary-container shrink-0 hover:opacity-95"
                                                on:click=move |_| {
                                                    let note = reviewer_notes.get();
                                                    add_reviewer_note(note);
                                                }
                                            >
                                                "Post Note"
                                            </button>
                                        </div>
                                    </div>

                                    <div class="space-y-4">
                                        <h4 class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Review History Timeline"</h4>
                                        <div class="relative pl-6 border-l border-outline-variant/20 ml-2 space-y-6 pt-2">
                                            {move || note_history.get().into_iter().map(|item| view! {
                                                <div class="relative">
                                                    // Timeline dot
                                                    <div class="absolute -left-[30px] top-0 w-2.5 h-2.5 rounded-full bg-primary border-2 border-surface-container"></div>
                                                    <div>
                                                        <div class="flex items-center gap-2">
                                                            <span class="text-xs font-bold text-on-surface">{item.author}</span>
                                                            <span class="px-1.5 py-0.2 bg-surface-container border border-outline-variant/20 rounded text-[9px] font-bold uppercase tracking-widest text-on-surface-variant/60">"Super-Admin"</span>
                                                            <span class="text-[10px] text-on-surface-variant/40 font-mono ml-auto">{item.timestamp}</span>
                                                        </div>
                                                        <p class="text-xs text-on-surface-variant mt-1 leading-relaxed bg-surface-container p-3 rounded-lg border border-outline-variant/10">{item.text}</p>
                                                    </div>
                                                </div>
                                            }).collect_view()}
                                        </div>
                                    </div>
                                </div>
                            </Show>
                        </div>
                    </div>
                }.into_any()
            }}

            // ── ACTION MODAL DIALOGS ──
            
            // 1. Approve Dialog
            <Show when=move || show_approve_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-md p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_approve_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2">"Approve Business Verification"</h3>
                        <p class="text-on-surface-variant text-xs mb-6">"Are you sure you want to approve this verification? The tenant status will immediately update to verified. This action is permanently logged."</p>
                        <div class="flex justify-end gap-3">
                            <button class="btn-ghost px-4 py-2 border border-outline-variant/30 rounded-lg text-xs font-semibold hover:bg-surface-bright/20" on:click=move |_| show_approve_modal.set(false)>"Cancel"</button>
                            <button 
                                class="bg-emerald-500 border border-emerald-500/20 text-on-primary hover:opacity-90 px-4 py-2 rounded-lg text-xs font-semibold"
                                on:click=move |_| {
                                    show_approve_modal.set(false);
                                    toast.show_toast("Success", "Verification request approved.", "success");
                                }
                            >
                                "Approve Request"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // 2. Reject Dialog
            <Show when=move || show_reject_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-md p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_reject_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2 text-error">"Reject Verification Request"</h3>
                        <p class="text-on-surface-variant text-xs mb-4">"Explain the refusal decision to send as an email notification feedback."</p>
                        <textarea class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-3 text-xs text-on-surface outline-none focus:border-error min-h-[80px] mb-6 resize-none" placeholder="Reason for rejection (e.g. Expired registration documents)"></textarea>
                        <div class="flex justify-end gap-3">
                            <button class="btn-ghost px-4 py-2 border border-outline-variant/30 rounded-lg text-xs font-semibold hover:bg-surface-bright/20" on:click=move |_| show_reject_modal.set(false)>"Cancel"</button>
                            <button 
                                class="bg-error border border-error/20 text-on-error hover:opacity-95 px-4 py-2 rounded-lg text-xs font-semibold"
                                on:click=move |_| {
                                    show_reject_modal.set(false);
                                    toast.show_toast("Refused", "Verification request rejected.", "error");
                                }
                            >
                                "Reject Request"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // 3. Request More Info Dialog
            <Show when=move || show_info_request_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-lg p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_info_request_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold mb-2">"Request Additional Documents"</h3>
                        <p class="text-on-surface-variant text-xs mb-4">"The applicant will receive an email checklist link to supply the following items."</p>
                        <div class="space-y-4 mb-6">
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Recipient Email"</label>
                                <input type="email" class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary" value="applicant@nexusproperties.com" disabled=true />
                            </div>
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Email message body"</label>
                                <textarea class="w-full bg-surface-container border border-outline-variant/40 rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary h-28 resize-none">
                                    "Hi,\n\nWe need additional information to complete your verification. Please reply with the requested documents.\n\n— Atlas Platform Team"
                                </textarea>
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button class="btn-ghost px-4 py-2 border border-outline-variant/30 rounded-lg text-xs font-semibold hover:bg-surface-bright/20" on:click=move |_| show_info_request_modal.set(false)>"Cancel"</button>
                            <button 
                                class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-semibold text-on-primary-container"
                                on:click=move |_| {
                                    show_info_request_modal.set(false);
                                    toast.show_toast("Success", "Request email sent to applicant.", "success");
                                }
                            >
                                "Send Request"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
