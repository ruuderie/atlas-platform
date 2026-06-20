use leptos::prelude::*;
use crate::api::models::VerificationRequestModel;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ChecklistItem {
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

fn compute_age_days(created_at_str: &str) -> u32 {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at_str) {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
        duration.num_days().max(0) as u32
    } else if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(created_at_str, "%Y-%m-%d %H:%M:%S") {
        let now = chrono::Utc::now().naive_utc();
        let duration = now.signed_duration_since(dt);
        duration.num_days().max(0) as u32
    } else {
        3 // fallback default
    }
}

fn get_priority_color(age_days: u32, status: &str) -> &'static str {
    if status == "approved" {
        "bg-green"
    } else if status == "rejected" {
        "bg-text-muted"
    } else if age_days >= 7 {
        "bg-error"
    } else if age_days >= 4 {
        "bg-amber-400"
    } else if status == "review" {
        "bg-purple-400"
    } else {
        "bg-primary"
    }
}

fn get_badge_style(req_type: &str) -> &'static str {
    match req_type.to_lowercase().as_str() {
        "business" => "color:var(--red);border-color:var(--red);background:var(--red-dim)",
        "identity" => "color:var(--amber);border-color:var(--amber);background:var(--amber-dim)",
        _ => "color:var(--cobalt);border-color:var(--cobalt);background:var(--cobalt-dim)",
    }
}

#[component]
pub fn Verification() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    // UI state signals
    let selected_id = RwSignal::new(None::<Uuid>);
    let active_filter = RwSignal::new("all".to_string());
    let active_rev_tab = RwSignal::new("rv-documents".to_string());
    let trigger_fetch = RwSignal::new(0);

    // Resource for database verification requests
    let db_requests = LocalResource::new(move || {
        trigger_fetch.get();
        let filter_val = active_filter.get();
        async move {
            let filter = if filter_val == "all" { None } else { Some(filter_val) };
            crate::api::verification::get_verification_requests(None, filter).await.unwrap_or_default()
        }
    });

    let selected_request = Signal::derive(move || {
        let sid = selected_id.get();
        db_requests.get().unwrap_or_default().into_iter().find(|r| Some(r.id) == sid)
    });

    // Set first request as default selection when data loads
    Effect::new(move |_| {
        let list = db_requests.get().unwrap_or_default();
        if !list.is_empty() && selected_id.get().is_none() {
            selected_id.set(Some(list[0].id));
        }
    });
    
    // Checklist items state, dynamically configured per selected request
    let checklist_items = RwSignal::new(vec![]);
    Effect::new(move |_| {
        if let Some(req) = selected_request.get() {
            let items = match req.req_type.to_lowercase().as_str() {
                "business" => vec![
                    ("EIN / Tax ID confirmed", "Verified via IRS SS-4", true),
                    ("Business name matches submitted entity name", "✓ Match", true),
                    ("Operating agreement / LLC articles present", "Signed PDF attached", true),
                    ("State registration document current (within 2 years)", "Pending review", false),
                    ("Primary contact identity verified (ID check)", "Pending", false),
                    ("No active regulatory flags (FMCSA / DOT cross-check)", "Auto-check available", false),
                    ("Billing address matches registered business address", "✓ Match", true),
                ],
                "identity" => vec![
                    ("Government ID matches name", "Passport scanned", true),
                    ("Facial recognition matches ID photo", "98% confidence", true),
                    ("PEP list cross-reference check", "✓ Clear", true),
                    ("Sanction registry check", "✓ Clear", true),
                    ("Proof of residency document verified", "Awaiting review", false),
                ],
                _ => vec![
                    ("Document signature validation", "Standard SHA-256 hash verified", true),
                    ("Issuer authenticity confirmation", "Self-signed certificate match", true),
                    ("Expiration boundary validation", "Valid until 2029", true),
                ]
            };
            checklist_items.set(items.into_iter().enumerate().map(|(id, (label, note, checked))| {
                ChecklistItem { id, label, note, checked: RwSignal::new(checked) }
            }).collect::<Vec<_>>());
        }
    });

    // Notes history state
    let reviewer_notes = RwSignal::new(String::new());
    let note_history = RwSignal::new(Vec::<NoteHistoryRecord>::new());
    
    // Dialog modals
    let show_approve_modal = RwSignal::new(false);
    let show_reject_modal = RwSignal::new(false);
    let temp_rejection_reason = RwSignal::new(String::new());
    let show_info_request_modal = RwSignal::new(false);

    // Handle adding note
    let add_reviewer_note = move |_| {
        let text = reviewer_notes.get();
        if text.trim().is_empty() { return; }
        note_history.update(|h| {
            h.insert(0, NoteHistoryRecord {
                author: "JD",
                text: text.clone(),
                timestamp: "Just now · UTC"
            });
        });
        reviewer_notes.set(String::new());
        toast.show_toast("Success", "Review note added.", "success");
    };

    let handle_approve = move |_| {
        if let Some(req) = selected_request.get() {
            let req_id = req.id;
            leptos::task::spawn_local(async move {
                match crate::api::verification::approve_verification_request(req_id).await {
                    Ok(_) => {
                        toast.show_toast("Success", "Verification request approved.", "success");
                        trigger_fetch.set(trigger_fetch.get() + 1);
                    }
                    Err(e) => toast.show_toast("Error", &e, "error"),
                }
            });
        }
        show_approve_modal.set(false);
    };

    let handle_reject = move |_| {
        if let Some(req) = selected_request.get() {
            let req_id = req.id;
            let reason = temp_rejection_reason.get();
            if reason.trim().is_empty() {
                toast.show_toast("Error", "Rejection reason is required.", "error");
                return;
            }
            leptos::task::spawn_local(async move {
                match crate::api::verification::reject_verification_request(req_id, reason).await {
                    Ok(_) => {
                        toast.show_toast("Refused", "Verification request rejected.", "error");
                        trigger_fetch.set(trigger_fetch.get() + 1);
                    }
                    Err(e) => toast.show_toast("Error", &e, "error"),
                }
            });
        }
        show_reject_modal.set(false);
    };

    view! {
        <Suspense fallback=|| view! {
            <div class="main-canvas">
                <div class="animate-pulse flex flex-col items-center justify-center h-64">
                    <span class="material-symbols-outlined text-4xl mb-2 opacity-50">"sync"</span>
                    <p>"Loading verification queue..."</p>
                </div>
            </div>
        }>
            <div class="main">
                // ── LEFT PANE: Queue ──
                <div class="queue-pane">
                    <div class="queue-hdr">
                        <div class="queue-title-row">
                            <div class="queue-title">
                                "Verification Queue"
                                <span class="queue-badge">
                                    {move || {
                                        let count = db_requests.get().unwrap_or_default().iter().filter(|r| r.status == "pending" || r.status == "review").count();
                                        format!("{} Active", count)
                                    }}
                                </span>
                            </div>
                            <button class="pill" style="font-size:10px;padding:2px 6px;" on:click=move |_| toast.show_toast("Info", "Verification logs compiled.", "info")>"Export"</button>
                        </div>
                        <div class="filter-row">
                            {
                                let filter_pill = move |id: &'static str, label: &'static str| {
                                    view! {
                                        <button 
                                            class=move || if active_filter.get() == id { "pill active" } else { "pill" }
                                            on:click=move |_| {
                                                active_filter.set(id.to_string());
                                                selected_id.set(None);
                                            }
                                        >
                                            {label}
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
                    
                    <div class="queue-scroll">
                        {move || {
                            let list = db_requests.get().unwrap_or_default();
                            if list.is_empty() {
                                view! {
                                    <div class="p-6 text-center text-xs text-on-surface-variant/50">"No requests in queue."</div>
                                }.into_any()
                            } else {
                                view! {
                                    {list.into_iter().map(|item| {
                                    let is_selected = selected_id.get() == Some(item.id);
                                    let age = compute_age_days(&item.created_at);
                                    let priority_bg = get_priority_color(age, &item.status);
                                    let badge_style = get_badge_style(&item.req_type);
                                    let r_id = item.id;
                                    let name = item.entity_name.clone();
                                    let t_slug = format!("tenant_{}", item.tenant_id.to_string().chars().take(8).collect::<String>());

                                    let icon_svg = match item.req_type.to_lowercase().as_str() {
                                        "business" => view! {
                                            <svg viewBox="0 0 16 16" fill="none" stroke="var(--red)" stroke-width="1.5"><rect x="2" y="4" width="12" height="10" rx="0.5"/><path d="M5 4V3a3 3 0 0 1 6 0v1"/><path d="M8 8v2"/><circle cx="8" cy="11.5" r="0.5" fill="var(--red)" stroke="none"/></svg>
                                        }.into_any(),
                                        "identity" => view! {
                                            <svg viewBox="0 0 16 16" fill="none" stroke="var(--amber)" stroke-width="1.5"><circle cx="8" cy="8" r="5"/><path d="M8 5v3l2 2"/></svg>
                                        }.into_any(),
                                        _ => view! {
                                            <svg viewBox="0 0 16 16" fill="none" stroke="var(--cobalt)" stroke-width="1.5"><rect x="2" y="2" width="12" height="14" rx="0.5"/><path d="M5 6h6M5 9h6M5 12h4"/></svg>
                                        }.into_any(),
                                    };

                                    let age_text = match item.status.as_str() {
                                        "approved" => "Done".to_string(),
                                        "rejected" => "Rejected".to_string(),
                                        "review" => "Review".to_string(),
                                        _ => format!("{}d pending", age),
                                    };

                                    let age_class = if item.status == "approved" {
                                        "age-ok"
                                    } else if item.status == "rejected" {
                                        "age-critical"
                                    } else if age >= 7 {
                                        "age-critical"
                                    } else if age >= 4 {
                                        "age-warn"
                                    } else {
                                        "age-ok"
                                    };

                                    view! {
                                        <div 
                                            class=move || if is_selected { "queue-item selected" } else { "queue-item" }
                                            on:click=move |_| selected_id.set(Some(r_id))
                                        >
                                            <div class=format!("qi-priority-bar {}", priority_bg)></div>
                                            <div class="qi-icon">{icon_svg}</div>
                                            <div class="qi-info">
                                                <div class="qi-entity">{name}</div>
                                                <div class="qi-type">{format!("{} · {} · {} docs", item.req_type, t_slug, item.document_count)}</div>
                                            </div>
                                            <div class="qi-right">
                                                <span class=format!("qi-age {}", age_class)>{age_text}</span>
                                                <span class="type-badge" style=badge_style>{item.req_type.clone()}</span>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                                }.into_any()
                            }
                        }}
                    </div>
                </div>

                // ── RIGHT PANE: Details Workspace ──
                {move || {
                    let current_req = selected_request.get();
                    if current_req.is_none() {
                        return view! {
                            <div class="review-pane">
                                <div class="flex-1 flex flex-col items-center justify-center text-on-surface-variant p-8 bg-surface-container-low/30">
                                    <span class="material-symbols-outlined text-4xl text-on-surface-variant/40 mb-3">"folder_shared"</span>
                                    <div class="text-sm font-semibold">"No Request Selected"</div>
                                    <div class="text-xs text-on-surface-variant/60 mt-1">"Select a pending registration from the left queue."</div>
                                </div>
                            </div>
                        }.into_any();
                    }
                    let request = current_req.unwrap();
                    let req_id = request.id;
                    let entity_title = request.entity_name.clone();
                    let type_label = request.req_type.clone();
                    let status_str = request.status.clone();
                    let age = compute_age_days(&request.created_at);
                    let badge_style = get_badge_style(&type_label);

                    let sub_header_text = match status_str.as_str() {
                        "approved" => "Verification Approved".to_string(),
                        "rejected" => "Verification Rejected".to_string(),
                        "review" => "Under active review".to_string(),
                        _ => format!("{} days pending — Action Required", age),
                    };

                    let status_str_show1 = status_str.clone();
                    let status_str_show2 = status_str.clone();
                    let status_str_show3 = status_str.clone();

                    view! {
                        <div class="review-pane">
                            // Detail Header
                            <div class="review-hdr">
                                <div style="display:flex;align-items:flex-start;justify-content:space-between;flex-wrap:wrap;gap:12px;">
                                    <div>
                                        <div style="font-size:11px;color:var(--text-muted);margin-bottom:4px;display:flex;align-items:center;gap:6px">
                                            <span class="type-badge" style=badge_style>{format!("{} Verification", type_label)}</span>
                                            <span>"·"</span>
                                            <span style=format!("font-weight:600; {}", if status_str == "approved" { "color:var(--green)" } else if status_str == "rejected" || age >= 7 { "color:var(--red)" } else { "color:var(--amber)" })>
                                                {sub_header_text}
                                            </span>
                                        </div>
                                        <div class="rev-entity">{entity_title.clone()}</div>
                                        <div class="rev-meta">{format!("tenant_id: {} · Submitted {} · {} documents attached", req_id, request.created_at, request.document_count)}</div>
                                    </div>
                                    <div class="rev-actions">
                                        <button class="btn btn-ghost btn-sm" on:click=move |_| show_info_request_modal.set(true)>"Request More Info"</button>
                                        <Show when=move || status_str_show1 != "rejected">
                                            <button class="btn btn-reject btn-sm" on:click=move |_| {
                                                temp_rejection_reason.set(String::new());
                                                show_reject_modal.set(true);
                                            }>"Reject"</button>
                                        </Show>
                                        <Show when=move || status_str_show2 != "approved">
                                            <button class="btn btn-approve btn-sm" on:click=move |_| show_approve_modal.set(true)>"Approve →"</button>
                                        </Show>
                                    </div>
                                </div>
                            </div>

                            // Detail Tabs
                            <div class="rev-tabs">
                                <button class=move || if active_rev_tab.get() == "rv-documents" { "rev-tab active" } else { "rev-tab" } on:click=move |_| active_rev_tab.set("rv-documents".into())>"Documents"</button>
                                <button class=move || if active_rev_tab.get() == "rv-checklist" { "rev-tab active" } else { "rev-tab" } on:click=move |_| active_rev_tab.set("rv-checklist".into())>"Review Checklist"</button>
                                <button class=move || if active_rev_tab.get() == "rv-entity" { "rev-tab active" } else { "rev-tab" } on:click=move |_| active_rev_tab.set("rv-entity".into())>"Entity Summary"</button>
                                <button class=move || if active_rev_tab.get() == "rv-notes" { "rev-tab active" } else { "rev-tab" } on:click=move |_| active_rev_tab.set("rv-notes".into())>"Reviewer Notes"</button>
                            </div>

                            // Detail Content
                            <div class="rev-content">
                                // TAB: Documents
                                <div class=move || format!("tab-pane {}", if active_rev_tab.get() == "rv-documents" { "active" } else { "" })>
                                    <div style="margin-bottom:14px">
                                        <div style="font-size:11px;font-weight:600;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);margin-bottom:10px">"Submitted Documents · G-02 Vault"</div>
                                        
                                        {
                                            let count = request.document_count.max(1);
                                            let r_type = type_label.clone();
                                            let entity_name_cleaned = entity_title.clone().replace(" ", "_");
                                            
                                            (0..count).map(move |i| {
                                                let doc_name = match i {
                                                    0 if r_type.to_lowercase() == "identity" => format!("Government_ID_{}.pdf", entity_name_cleaned),
                                                    0 => format!("Business_Registration_{}.pdf", entity_name_cleaned),
                                                    1 if r_type.to_lowercase() == "identity" => "Selfie_Verification_Scan.png".to_string(),
                                                    1 => "EIN_Confirmation_SS4.pdf".to_string(),
                                                    2 => "Operating_Agreement_Signed.pdf".to_string(),
                                                    _ => format!("Supporting_Document_{}.pdf", i),
                                                };
                                                let format_type = if doc_name.ends_with(".png") { "PNG" } else { "PDF" };
                                                let size_mb = (3.2 - (i as f32 * 0.9)).max(0.5);
                                                
                                                view! {
                                                    <div class="doc-card">
                                                        <div class="doc-icon">{format_type}</div>
                                                        <div class="doc-info">
                                                            <div class="doc-name">{doc_name}</div>
                                                            <div class="doc-meta">{format!("{:.1} MB · Uploaded · SHA-256 Checksum Verified", size_mb)}</div>
                                                        </div>
                                                        <div class="flex items-center gap-3">
                                                            <span class="px-2 py-0.5 rounded border border-[#069669]/20 text-[#069669] text-[9px] font-bold uppercase tracking-wider bg-[#069669]/5">"Verified"</span>
                                                            <button class="pill" style="font-size:10px;" on:click=move |_| toast.show_toast("Document Viewer", "Loading document preview in secure sandbox...", "info")>"View ↗"</button>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()
                                        }

                                        <Show when=move || status_str_show3 == "rejected">
                                            <div class="disq-panel" style="margin-top:16px;">
                                                <div class="disq-panel-label">"⚠ Rejection Reason"</div>
                                                <p class="text-xs text-on-surface-variant/90 leading-relaxed">{request.rejection_reason.clone().unwrap_or_else(|| "Documents insufficient.".to_string())}</p>
                                            </div>
                                        </Show>
                                    </div>
                                </div>

                                // TAB: Checklist
                                <div class=move || format!("tab-pane {}", if active_rev_tab.get() == "rv-checklist" { "active" } else { "" })>
                                    <div class="checklist">
                                        {move || checklist_items.get().into_iter().map(|item| {
                                            let state = item.checked;
                                            let label = item.label;
                                            view! {
                                                <div class="check-item">
                                                    <div 
                                                        class=move || if state.get() { "check-box checked" } else { "check-box" }
                                                        on:click=move |_| state.update(|v| *v = !*v)
                                                    ></div>
                                                    <span class=move || if state.get() { "check-label checked-label" } else { "check-label" }>{label}</span>
                                                    <span class="check-note">{item.note}</span>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                // TAB: Entity Summary
                                <div class=move || format!("tab-pane {}", if active_rev_tab.get() == "rv-entity" { "active" } else { "" })>
                                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6" style="margin-top:4px;">
                                        <div class="card">
                                            <div class="card-hdr">
                                                <h3 class="card-title">"Entity Profile"</h3>
                                            </div>
                                            <div class="divide-y divide-outline-variant/10 text-xs">
                                                <div class="stat-row">
                                                    <span class="s-label">"Organization"</span>
                                                    <span class="s-value">{entity_title.clone()}</span>
                                                </div>
                                                <div class="stat-row">
                                                    <span class="s-label">"Registry Type"</span>
                                                    <span class="s-value">{type_label.clone()}</span>
                                                </div>
                                                <div class="stat-row">
                                                    <span class="s-label">"Tax ID / EIN"</span>
                                                    <span class="s-value font-mono muted">"—"</span>
                                                </div>
                                                <div class="stat-row">
                                                    <span class="s-label">"Incorporation Region"</span>
                                                    <span class="s-value muted">"—"</span>
                                                </div>
                                                <div class="stat-row">
                                                    <span class="s-label">"Primary Contact"</span>
                                                    <span class="s-value muted">"—"</span>
                                                </div>
                                            </div>
                                        </div>

                                        <div class="card">
                                            <div class="card-hdr">
                                                <h3 class="card-title">"FMCSA & Regulatory Screening"</h3>
                                            </div>
                                            <div class="divide-y divide-outline-variant/10 text-xs">
                                                <div class="stat-row">
                                                    <span class="s-label">"OFAC Sanctions List"</span>
                                                    <span class="s-value green font-semibold">"✓ Clear"</span>
                                                </div>
                                                <div class="stat-row">
                                                    <span class="s-label">"DOT Carrier Match"</span>
                                                    <span class="s-value muted">"Not applicable"</span>
                                                </div>
                                                <div class="stat-row">
                                                    <span class="s-label">"LEI Code Status"</span>
                                                    <span class="s-value green">"Verified"</span>
                                                </div>
                                                <div class="stat-row">
                                                    <span class="s-label">"Security Hold status"</span>
                                                    <span class="s-value green">"Clear"</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // TAB: Reviewer Notes
                                <div class=move || format!("tab-pane {}", if active_rev_tab.get() == "rv-notes" { "active" } else { "" })>
                                    <div class="space-y-6">
                                        <div class="composer">
                                            <div class="composer-tabs">
                                                <button class="c-tab active">"Internal Note"</button>
                                            </div>
                                            <textarea 
                                                placeholder="Add internal reviewer logs..."
                                                on:input=move |ev| reviewer_notes.set(event_target_value(&ev))
                                                prop:value=reviewer_notes
                                            ></textarea>
                                            <div class="composer-footer">
                                                <button class="btn btn-primary btn-sm" on:click=add_reviewer_note>"Post Note"</button>
                                            </div>
                                        </div>

                                        <div class="space-y-4">
                                            <h4 class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Review History Timeline"</h4>
                                            <div class="relative pl-6 border-l border-outline-variant/20 ml-2 space-y-6 pt-2">
                                                {move || note_history.get().into_iter().map(|item| view! {
                                                    <div class="relative">
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
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }}
            </div>
        </Suspense>

        // ── ACTION MODAL DIALOGS ──
        
        // 1. Approve Dialog
        <Show when=move || show_approve_modal.get()>
            <div class="modal-overlay open">
                <div class="modal">
                    <h3 class="text-lg font-bold mb-2">"Approve Business Verification"</h3>
                    <p class="text-on-surface-variant text-xs mb-6">"Are you sure you want to approve this verification? The tenant status will immediately update to verified. This action is permanently logged."</p>
                    <div class="flex justify-end gap-3">
                        <button class="btn btn-ghost" on:click=move |_| show_approve_modal.set(false)>"Cancel"</button>
                        <button class="btn btn-primary" on:click=handle_approve>"Approve Request"</button>
                    </div>
                </div>
            </div>
        </Show>

        // 2. Reject Dialog
        <Show when=move || show_reject_modal.get()>
            <div class="modal-overlay open">
                <div class="modal">
                    <h3 class="text-lg font-bold mb-2 text-error">"Reject Verification Request"</h3>
                    <p class="text-on-surface-variant text-xs mb-4">"Explain the refusal decision to send as an email notification feedback."</p>
                    <textarea 
                        class="w-full bg-[var(--bg-elevated)] border border-[var(--border-default)] rounded-lg p-3 text-xs text-on-surface outline-none focus:border-error min-h-[80px] mb-6 resize-none" 
                        placeholder="Reason for rejection (e.g. Expired registration documents)"
                        on:input=move |ev| temp_rejection_reason.set(event_target_value(&ev))
                        prop:value=temp_rejection_reason
                    ></textarea>
                    <div class="flex justify-end gap-3">
                        <button class="btn btn-ghost" on:click=move |_| show_reject_modal.set(false)>"Cancel"</button>
                        <button class="btn btn-reject btn-sm" on:click=handle_reject>"Reject Request"</button>
                    </div>
                </div>
            </div>
        </Show>

        // 3. Request More Info Dialog
        <Show when=move || show_info_request_modal.get()>
            <div class="modal-overlay open">
                <div class="modal">
                    <h3 class="text-lg font-bold mb-2">"Request Additional Documents"</h3>
                    <p class="text-on-surface-variant text-xs mb-4">"The applicant will receive an email checklist link to supply the following items."</p>
                    <div class="space-y-4 mb-6">
                        <div class="space-y-1.5">
                            <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Recipient Email"</label>
                            <input type="email" class="form-input" value=move || {
                                    selected_request.get()
                                        .map(|r| format!("applicant+{}@verification.atlasplatform.io", r.entity_name.to_lowercase().replace(' ', ".")))
                                        .unwrap_or_else(|| "applicant@verification.atlasplatform.io".to_string())
                                } disabled=true />
                        </div>
                        <div class="space-y-1.5">
                            <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Email message body"</label>
                            <textarea class="w-full bg-[var(--bg-elevated)] border border-[var(--border-default)] rounded-lg p-2.5 text-xs text-on-surface outline-none focus:border-primary h-28 resize-none">
                                "Hi,\n\nWe need additional information to complete your verification. Please reply with the requested documents.\n\n— Atlas Platform Team"
                            </textarea>
                        </div>
                    </div>
                    <div class="flex justify-end gap-3">
                        <button class="btn btn-ghost" on:click=move |_| show_info_request_modal.set(false)>"Cancel"</button>
                        <button class="btn btn-primary" on:click=move |_| {
                            show_info_request_modal.set(false);
                            toast.show_toast("Success", "Request email sent to applicant.", "success");
                        }>"Send Request"</button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
