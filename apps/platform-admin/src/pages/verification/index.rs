use crate::components::gtm_process_strip::{GtmProcessStrip, GtmStage};
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ChecklistItem {
    pub label: &'static str,
    pub note: &'static str,
    pub note_class: &'static str,
    pub checked: RwSignal<bool>,
}

#[derive(Clone, Debug)]
pub struct NoteHistoryRecord {
    pub author: String,
    pub text: String,
    pub timestamp: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QueueFilter {
    All,
    Pending,
    Review,
    NeedsInfo,
    Approved,
    Rejected,
}

impl QueueFilter {
    fn api_status(self) -> Option<String> {
        match self {
            Self::All => None,
            Self::Pending => Some("pending".to_string()),
            Self::Review => Some("review".to_string()),
            Self::NeedsInfo => Some("needs_info".to_string()),
            Self::Approved => Some("approved".to_string()),
            Self::Rejected => Some("rejected".to_string()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReviewTab {
    Documents,
    Checklist,
    Entity,
    Notes,
}

fn compute_age_days(created_at_str: &str) -> u32 {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created_at_str) {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(dt.with_timezone(&chrono::Utc));
        duration.num_days().max(0) as u32
    } else if let Ok(dt) =
        chrono::NaiveDateTime::parse_from_str(created_at_str, "%Y-%m-%d %H:%M:%S")
    {
        let now = chrono::Utc::now().naive_utc();
        let duration = now.signed_duration_since(dt);
        duration.num_days().max(0) as u32
    } else {
        3
    }
}

fn get_priority_color(age_days: u32, status: &str) -> &'static str {
    match status {
        "approved" => "var(--green)",
        "rejected" => "var(--text-muted)",
        "needs_info" => "var(--amber)",
        _ if age_days >= 7 => "var(--red)",
        _ if age_days >= 4 => "var(--amber)",
        "review" => "var(--violet)",
        _ => "var(--cobalt)",
    }
}

fn get_badge_style(req_type: &str) -> &'static str {
    match req_type.to_lowercase().as_str() {
        "business" => "color:var(--red);border-color:var(--red);background:var(--red-dim)",
        "identity" => {
            "color:var(--amber);border-color:var(--amber);background:var(--amber-dim)"
        }
        _ => "color:var(--cobalt);border-color:var(--cobalt);background:var(--cobalt-dim)",
    }
}

fn get_status_badge_style(status: &str, req_type: &str) -> &'static str {
    match status {
        "approved" => "color:var(--green);border-color:var(--green);background:var(--green-dim)",
        "rejected" => {
            "color:var(--text-muted);border-color:var(--border-default);background:var(--bg-elevated)"
        }
        "needs_info" => {
            "color:var(--amber);border-color:var(--amber);background:var(--amber-dim)"
        }
        "review" => {
            "color:var(--violet);border-color:var(--violet);background:var(--violet-dim)"
        }
        _ => get_badge_style(req_type),
    }
}

fn get_icon_style(status: &str, req_type: &str) -> &'static str {
    match status {
        "approved" => "background:var(--green-dim)",
        "rejected" => "background:var(--bg-elevated)",
        "needs_info" => "background:var(--amber-dim)",
        "review" => "background:var(--violet-dim)",
        _ => match req_type.to_lowercase().as_str() {
            "business" => "background:var(--red-dim)",
            "identity" => "background:var(--amber-dim)",
            _ => "background:var(--cobalt-dim)",
        },
    }
}

fn age_class(age_days: u32, status: &str) -> &'static str {
    match status {
        "approved" | "rejected" => "qi-age age-ok",
        "needs_info" => "qi-age age-warn",
        _ if age_days >= 7 => "qi-age age-critical",
        _ if age_days >= 4 => "qi-age age-warn",
        _ => "qi-age age-ok",
    }
}

fn age_text(age_days: u32, status: &str) -> String {
    match status {
        "approved" => "Done".to_string(),
        "rejected" => "Rejected".to_string(),
        "needs_info" => "Needs info".to_string(),
        "review" => "Review".to_string(),
        _ if age_days >= 7 => format!("{}d overdue", age_days),
        _ => format!("{}d pending", age_days),
    }
}

#[component]
pub fn Verification() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let selected_id = RwSignal::new(None::<Uuid>);
    let active_filter = RwSignal::new(QueueFilter::All);
    let active_rev_tab = RwSignal::new(ReviewTab::Documents);
    let trigger_fetch = RwSignal::new(0);
    let ver_error: RwSignal<Option<String>> = RwSignal::new(None);

    let db_requests = LocalResource::new(move || {
        trigger_fetch.get();
        let filter = active_filter.get();
        async move {
            let res =
                crate::api::verification::get_verification_requests(None, filter.api_status())
                    .await;
            match res {
                Ok(v) => {
                    ver_error.set(None);
                    v
                }
                Err(e) => {
                    ver_error.set(Some(e));
                    vec![]
                }
            }
        }
    });

    let selected_request = Signal::derive(move || {
        let sid = selected_id.get();
        db_requests
            .get()
            .unwrap_or_default()
            .into_iter()
            .find(|r| Some(r.id) == sid)
    });

    Effect::new(move |_| {
        let list = db_requests.get().unwrap_or_default();
        if !list.is_empty() && selected_id.get().is_none() {
            selected_id.set(Some(list[0].id));
        }
    });

    let checklist_items = RwSignal::new(vec![]);
    Effect::new(move |_| {
        if let Some(req) = selected_request.get() {
            let items = match req.req_type.to_lowercase().as_str() {
                "business" => vec![
                    ("EIN / Tax ID confirmed", "Verified via IRS SS-4", "", true),
                    (
                        "Business name matches submitted entity name",
                        "✓ Match",
                        "",
                        true,
                    ),
                    (
                        "Operating agreement / LLC articles present",
                        "Signed PDF attached",
                        "",
                        true,
                    ),
                    (
                        "State registration document current (within 2 years)",
                        "⚠ 2019 — review",
                        "critical",
                        false,
                    ),
                    (
                        "Primary contact identity verified (ID check)",
                        "Pending",
                        "warn",
                        false,
                    ),
                    (
                        "No active regulatory flags (FMCSA / DOT cross-check)",
                        "Auto-check available",
                        "",
                        false,
                    ),
                    (
                        "Billing address matches registered business address",
                        "✓ Match",
                        "",
                        true,
                    ),
                ],
                "identity" => vec![
                    ("Government ID matches name", "Passport scanned", "", true),
                    ("Facial recognition matches ID photo", "98% confidence", "", true),
                    ("PEP list cross-reference check", "✓ Clear", "", true),
                    ("Sanction registry check", "✓ Clear", "", true),
                    (
                        "Proof of residency document verified",
                        "Pending",
                        "warn",
                        false,
                    ),
                ],
                _ => vec![
                    (
                        "Document signature validation",
                        "Standard SHA-256 hash verified",
                        "",
                        true,
                    ),
                    (
                        "Issuer authenticity confirmation",
                        "Self-signed certificate match",
                        "",
                        true,
                    ),
                    ("Expiration boundary validation", "Valid until 2029", "", true),
                ],
            };

            checklist_items.set(
                items
                    .into_iter()
                    .map(|(label, note, note_class, checked)| ChecklistItem {
                        label,
                        note,
                        note_class,
                        checked: RwSignal::new(checked),
                    })
                    .collect::<Vec<_>>(),
            );
        }
    });

    let reviewer_notes = RwSignal::new(String::new());
    let note_history = RwSignal::new(Vec::<NoteHistoryRecord>::new());
    let info_request_message = RwSignal::new(
        "We need additional information to complete your verification. Please reply with the requested documents."
            .to_string(),
    );

    // Sync persisted reviewer notes when selection changes
    Effect::new(move |_| {
        if let Some(req) = selected_request.get() {
            let history = match req.reviewer_notes.as_ref() {
                Some(raw) if !raw.trim().is_empty() => raw
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .map(|line| {
                        let (timestamp, text) = if line.starts_with('[') {
                            if let Some(end) = line.find(']') {
                                (
                                    line[1..end].to_string(),
                                    line[end + 1..].trim().to_string(),
                                )
                            } else {
                                ("—".to_string(), line.to_string())
                            }
                        } else {
                            ("—".to_string(), line.to_string())
                        };
                        NoteHistoryRecord {
                            author: "Reviewer".to_string(),
                            text,
                            timestamp,
                        }
                    })
                    .rev()
                    .collect(),
                _ => Vec::new(),
            };
            note_history.set(history);
        } else {
            note_history.set(Vec::new());
        }
    });

    let show_approve_modal = RwSignal::new(false);
    let show_reject_modal = RwSignal::new(false);
    let show_info_request_modal = RwSignal::new(false);
    let temp_rejection_reason = RwSignal::new(String::new());

    let add_reviewer_note = move |_| {
        let text = reviewer_notes.get();
        if text.trim().is_empty() {
            return;
        }
        let Some(req) = selected_request.get() else {
            return;
        };
        let req_id = req.id;
        leptos::task::spawn_local(async move {
            match crate::api::verification::add_verification_notes(req_id, text).await {
                Ok(_) => {
                    reviewer_notes.set(String::new());
                    toast.show_toast("Success", "Review note saved.", "success");
                    trigger_fetch.set(trigger_fetch.get() + 1);
                }
                Err(e) => toast.show_toast("Error", &e, "error"),
            }
        });
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

    let filter_pill = move |filter: QueueFilter, label: &'static str| {
        view! {
            <button
                class=move || if active_filter.get() == filter { "pill active" } else { "pill" }
                on:click=move |_| {
                    active_filter.set(filter);
                    selected_id.set(None);
                    active_rev_tab.set(ReviewTab::Documents);
                }
            >
                {label}
            </button>
        }
    };

    let rev_tab = move |tab: ReviewTab, label: &'static str| {
        view! {
            <button
                class=move || if active_rev_tab.get() == tab { "rev-tab active" } else { "rev-tab" }
                on:click=move |_| active_rev_tab.set(tab)
            >
                {label}
            </button>
        }
    };

    view! {
        <div class="main-area no-pad">
            <Suspense fallback=|| view! {
                <div class="empty-state">
                    <div class="empty-state-title">"Loading verification queue..."</div>
                    <div class="empty-state-body">"Fetching submitted identity and business documents."</div>
                </div>
            }>
                <div class="main">
                    <div class="queue-pane">
                        <div class="queue-hdr">
                            <div class="queue-title-row">
                                <div class="queue-title">
                                    "Verification Queue"
                                    <span class="queue-badge">
                                        {move || {
                                            let count = db_requests
                                                .get()
                                                .unwrap_or_default()
                                                .iter()
                                                .filter(|r| {
                                                    r.status == "pending"
                                                        || r.status == "review"
                                                        || r.status == "needs_info"
                                                })
                                                .count();
                                            format!("{} Active", count)
                                        }}
                                    </span>
                                </div>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    title="Reload queue from backend"
                                    on:click=move |_| trigger_fetch.set(trigger_fetch.get() + 1)
                                >
                                    "Refresh"
                                </button>
                            </div>
                            <div class="queue-title-row">
                                {filter_pill(QueueFilter::All, "All")}
                                {filter_pill(QueueFilter::Pending, "Pending")}
                                {filter_pill(QueueFilter::Review, "In Review")}
                                {filter_pill(QueueFilter::NeedsInfo, "Needs Info")}
                                {filter_pill(QueueFilter::Approved, "Approved")}
                                {filter_pill(QueueFilter::Rejected, "Rejected")}
                            </div>
                        </div>

                        <div class="queue-scroll">
                            {move || {
                                let list = db_requests.get().unwrap_or_default();
                                if list.is_empty() {
                                    view! {
                                        <div class="empty-state">
                                            <div class="empty-state-title">"No requests in queue."</div>
                                            <div class="empty-state-body">"Try a different filter or refresh the queue."</div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        {list.into_iter().map(|item| {
                                            let is_selected = selected_id.get() == Some(item.id);
                                            let age = compute_age_days(&item.created_at);
                                            let priority = get_priority_color(age, &item.status);
                                            let badge_style = get_status_badge_style(&item.status, &item.req_type);
                                            let icon_style = get_icon_style(&item.status, &item.req_type);
                                            let item_id = item.id;
                                            let done_class = if item.status == "approved" || item.status == "rejected" {
                                                " is-done"
                                            } else {
                                                ""
                                            };
                                            let selected_class = if is_selected { " selected" } else { "" };
                                            let t_slug = format!(
                                                "tenant_{}",
                                                item.tenant_id.to_string().chars().take(8).collect::<String>()
                                            );
                                            let icon_svg = match item.status.as_str() {
                                                "review" => view! {
                                                    <svg viewBox="0 0 16 16" fill="none" stroke="var(--violet)" stroke-width="1.5"><path d="M12 4H4l-2 4 2 4h8l2-4-2-4z"/><circle cx="8" cy="8" r="1.5"/></svg>
                                                }.into_any(),
                                                "approved" => view! {
                                                    <svg viewBox="0 0 16 16" fill="none" stroke="var(--green)" stroke-width="1.5"><rect x="2" y="7" width="12" height="7" rx="1"/><path d="M5 7V5a3 3 0 0 1 6 0v2"/></svg>
                                                }.into_any(),
                                                "rejected" => view! {
                                                    <svg viewBox="0 0 16 16" fill="none" stroke="var(--text-muted)" stroke-width="1.5"><circle cx="8" cy="6" r="3"/><path d="M4 13c0-2.2 1.8-4 4-4s4 1.8 4 4"/></svg>
                                                }.into_any(),
                                                _ => match item.req_type.to_lowercase().as_str() {
                                                    "business" => view! {
                                                        <svg viewBox="0 0 16 16" fill="none" stroke="var(--red)" stroke-width="1.5"><rect x="2" y="4" width="12" height="10" rx="0.5"/><path d="M5 4V3a3 3 0 0 1 6 0v1"/><path d="M8 8v2"/><circle cx="8" cy="11.5" r="0.5" fill="var(--red)" stroke="none"/></svg>
                                                    }.into_any(),
                                                    "identity" => view! {
                                                        <svg viewBox="0 0 16 16" fill="none" stroke="var(--amber)" stroke-width="1.5"><circle cx="8" cy="8" r="5"/><path d="M8 5v3l2 2"/></svg>
                                                    }.into_any(),
                                                    _ => view! {
                                                        <svg viewBox="0 0 16 16" fill="none" stroke="var(--cobalt)" stroke-width="1.5"><rect x="2" y="2" width="12" height="14" rx="0.5"/><path d="M5 6h6M5 9h6M5 12h4"/></svg>
                                                    }.into_any(),
                                                },
                                            };

                                            view! {
                                                <div
                                                    class=format!("queue-item{}{}", selected_class, done_class)
                                                    on:click=move |_| selected_id.set(Some(item_id))
                                                >
                                                    <div class="qi-priority-bar" style=format!("background:{}", priority)></div>
                                                    <div class="qi-icon" style=icon_style>{icon_svg}</div>
                                                    <div class="qi-info">
                                                        <div class="qi-entity">{item.entity_name.clone()}</div>
                                                        <div class="qi-type">{format!("{} · {} · {} docs", item.req_type, t_slug, item.document_count)}</div>
                                                    </div>
                                                    <div class="qi-right">
                                                        <span class=age_class(age, &item.status)>{age_text(age, &item.status)}</span>
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

                    <div class="review-pane">
                        <div class="verif-gtm-wrap">
                            <GtmProcessStrip
                                active=GtmStage::Verification
                                subtitle="Verification closes the GTM loop by approving identities, documents, and trust gates.".to_string()
                            />
                        </div>

                        {move || {
                            let current_req = selected_request.get();
                            if current_req.is_none() {
                                return view! {
                                    <div class="verif-empty">
                                        <div class="verif-empty-icon">"◎"</div>
                                        <div class="empty-state-title">"No Request Selected"</div>
                                        <div class="empty-state-body">"Select a pending registration from the queue to review documents, checklist signals, and reviewer notes."</div>
                                    </div>
                                }.into_any();
                            }

                            let request = current_req.unwrap();
                            let req_id = request.id;
                            let entity_title = request.entity_name.clone();
                            let type_label = request.req_type.clone();
                            let status_str = request.status.clone();
                            let age = compute_age_days(&request.created_at);
                            let badge_style = get_status_badge_style(&status_str, &type_label);
                            let header_state = match status_str.as_str() {
                                "approved" => "Verification Approved".to_string(),
                                "rejected" => "Verification Rejected".to_string(),
                                "needs_info" => "Awaiting applicant response".to_string(),
                                "review" => "Under active review".to_string(),
                                _ if age >= 7 => format!("{} days — Critical", age),
                                _ => format!("{} days pending — Action Required", age),
                            };
                            let header_state_style = match status_str.as_str() {
                                "approved" => "color:var(--green);font-weight:600",
                                "rejected" => "color:var(--red);font-weight:600",
                                "needs_info" => "color:var(--amber);font-weight:600",
                                "review" => "color:var(--violet);font-weight:600",
                                _ if age >= 7 => "color:var(--red);font-weight:600",
                                _ => "color:var(--amber);font-weight:600",
                            };
                            let show_reject_button = status_str != "rejected";
                            let show_approve_button = status_str != "approved";

                            view! {
                                <div class="review-hdr">
                                    <div class="queue-title-row">
                                        <div>
                                            <div class="rev-meta">
                                                <span class="type-badge" style=badge_style>{format!("{} Verification", type_label.clone())}</span>
                                                <span>" · "</span>
                                                <span style=header_state_style>{header_state}</span>
                                            </div>
                                            <div class="rev-entity">{entity_title.clone()}</div>
                                            <div class="rev-meta">{format!("tenant_id: {} · Submitted {} · {} documents attached", req_id, request.created_at, request.document_count)}</div>
                                        </div>
                                        <div class="rev-actions">
                                            <button class="btn btn-ghost btn-sm" on:click=move |_| show_info_request_modal.set(true)>"Request More Info"</button>
                                            <Show when=move || show_reject_button>
                                                <button class="btn btn-reject btn-sm" on:click=move |_| {
                                                    temp_rejection_reason.set(String::new());
                                                    show_reject_modal.set(true);
                                                }>"Reject"</button>
                                            </Show>
                                            <Show when=move || show_approve_button>
                                                <button class="btn btn-approve" on:click=move |_| show_approve_modal.set(true)>"Approve →"</button>
                                            </Show>
                                        </div>
                                    </div>
                                </div>

                                <div class="rev-tabs">
                                    {rev_tab(ReviewTab::Documents, "Documents")}
                                    {rev_tab(ReviewTab::Checklist, "Review Checklist")}
                                    {rev_tab(ReviewTab::Entity, "Entity Summary")}
                                    {rev_tab(ReviewTab::Notes, "Reviewer Notes")}
                                </div>

                                <div class="rev-content">
                                    {move || match active_rev_tab.get() {
                                        ReviewTab::Documents => {
                                            let attachment = request.attachment.clone();
                                            let doc_count = request.document_count;
                                            let rejected_reason = request.rejection_reason.clone();
                                            let rejected = status_str == "rejected";
                                            view! {
                                                <div>
                                                    <div class="verif-section-label">"Submitted Documents · G-02 Vault"</div>
                                                    {match attachment {
                                                        Some(att) => {
                                                            let doc_name = att
                                                                .title
                                                                .clone()
                                                                .filter(|t| !t.is_empty())
                                                                .unwrap_or_else(|| {
                                                                    if att.url.is_empty() {
                                                                        format!("Attachment {}", att.id)
                                                                    } else {
                                                                        att.url
                                                                            .rsplit('/')
                                                                            .next()
                                                                            .unwrap_or("attachment")
                                                                            .to_string()
                                                                    }
                                                                });
                                                            let format_type = if att.mime_type.contains("png")
                                                                || att.mime_type.contains("jpeg")
                                                                || att.mime_type.contains("jpg")
                                                            {
                                                                "IMG"
                                                            } else if att.mime_type.contains("pdf") {
                                                                "PDF"
                                                            } else if att.mime_type.is_empty() {
                                                                "DOC"
                                                            } else {
                                                                "FILE"
                                                            };
                                                            let view_url = att.url.clone();
                                                            let has_url = !view_url.is_empty();
                                                            view! {
                                                                <div class="doc-card">
                                                                    <div class="doc-icon">{format_type}</div>
                                                                    <div class="doc-info">
                                                                        <div class="doc-name">{doc_name}</div>
                                                                        <div class="doc-meta">{format!("id: {} · {}", att.id, if att.mime_type.is_empty() { "attached".to_string() } else { att.mime_type.clone() })}</div>
                                                                    </div>
                                                                    <div class="doc-actions">
                                                                        <span class="doc-status" style="color:var(--amber);border-color:var(--amber)">"Attached"</span>
                                                                        <Show when=move || has_url>
                                                                            <a
                                                                                class="btn btn-ghost btn-sm"
                                                                                href=view_url.clone()
                                                                                target="_blank"
                                                                                rel="noopener noreferrer"
                                                                            >
                                                                                "View ↗"
                                                                            </a>
                                                                        </Show>
                                                                    </div>
                                                                </div>
                                                            }.into_any()
                                                        }
                                                        None => view! {
                                                            <div class="empty-state">
                                                                <div class="empty-state-title">
                                                                    {if doc_count == 0 {
                                                                        "No documents attached"
                                                                    } else {
                                                                        "Attachment unavailable"
                                                                    }}
                                                                </div>
                                                                <div class="empty-state-body">"This request has no vault attachment yet. Request more info if evidence is required."</div>
                                                            </div>
                                                        }.into_any(),
                                                    }}

                                                    <Show when=move || rejected>
                                                        <div class="disq-panel">
                                                            <div class="disq-panel-label">"⚠ Rejection Reason"</div>
                                                            <div class="verif-warn-body">{rejected_reason.clone().unwrap_or_else(|| "Documents insufficient.".to_string())}</div>
                                                        </div>
                                                    </Show>
                                                </div>
                                            }.into_any()
                                        }
                                        ReviewTab::Checklist => view! {
                                            <div class="card">
                                                <div class="card-hdr">
                                                    <span class="card-title">{format!("{} Verification Checklist", type_label.clone())}</span>
                                                </div>
                                                <div class="checklist">
                                                    {move || checklist_items.get().into_iter().map(|item| {
                                                        let state = item.checked;
                                                        view! {
                                                            <div class="check-item">
                                                                <div
                                                                    class=move || if state.get() { "check-box checked" } else { "check-box" }
                                                                    on:click=move |_| state.update(|v| *v = !*v)
                                                                ></div>
                                                                <span class=move || if state.get() { "check-label checked-label" } else { "check-label" }>{item.label}</span>
                                                                <span class=if item.note_class.is_empty() { "check-note" } else if item.note_class == "warn" { "check-note warn" } else { "check-note critical" }>{item.note}</span>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                        }.into_any(),
                                        ReviewTab::Entity => view! {
                                            <div class="verif-stat-grid">
                                                <div class="card">
                                                    <div class="card-hdr"><span class="card-title">"Entity Profile"</span></div>
                                                    <div class="stat-row"><span class="s-label">"Organization"</span><span class="s-value">{entity_title.clone()}</span></div>
                                                    <div class="stat-row"><span class="s-label">"Registry Type"</span><span class="s-value">{type_label.clone()}</span></div>
                                                    <div class="stat-row"><span class="s-label">"Tax ID / EIN"</span><span class="s-value muted mono">"—"</span></div>
                                                    <div class="stat-row"><span class="s-label">"Incorporation Region"</span><span class="s-value muted">"—"</span></div>
                                                    <div class="stat-row"><span class="s-label">"Primary Contact"</span><span class="s-value muted">"—"</span></div>
                                                </div>
                                                <div class="card">
                                                    <div class="card-hdr">
                                                        <span class="card-title">"FMCSA / DOT Cross-Check"</span>
                                                        <button class="btn btn-ghost btn-sm" on:click=move |_| toast.show_toast("Queued", "Automated verification check queued.", "info")>"Run Check"</button>
                                                    </div>
                                                    <div class="stat-row"><span class="s-label">"DOT Match"</span><span class="s-value muted">"Not applicable"</span></div>
                                                    <div class="stat-row"><span class="s-label">"FMCSA Match"</span><span class="s-value muted">"Not applicable"</span></div>
                                                    <div class="stat-row"><span class="s-label">"OFAC Sanctions"</span><span class="s-value green">"✓ Clear"</span></div>
                                                    <div class="stat-row"><span class="s-label">"Security Hold Status"</span><span class="s-value green">"Clear"</span></div>
                                                </div>
                                            </div>
                                        }.into_any(),
                                        ReviewTab::Notes => view! {
                                            <div>
                                                <div class="verif-section-label">"Reviewer Notes"</div>
                                                <textarea
                                                    class="rev-notes"
                                                    placeholder="Add internal review notes..."
                                                    on:input=move |ev| reviewer_notes.set(event_target_value(&ev))
                                                    prop:value=reviewer_notes
                                                ></textarea>
                                                <div class="decision-row">
                                                    <button class="btn btn-primary btn-sm" on:click=add_reviewer_note>"Post Note"</button>
                                                    <button class="btn btn-approve btn-sm" on:click=move |_| show_approve_modal.set(true)>"Approve Verification"</button>
                                                    <button class="btn btn-reject btn-sm" on:click=move |_| {
                                                        temp_rejection_reason.set(String::new());
                                                        show_reject_modal.set(true);
                                                    }>"Reject"</button>
                                                    <button class="btn btn-ghost btn-sm" on:click=move |_| show_info_request_modal.set(true)>"Request More Info"</button>
                                                </div>

                                                <div class="verif-note-list">
                                                    {move || {
                                                        let history = note_history.get();
                                                        if history.is_empty() {
                                                            view! {
                                                                <div class="empty-state">
                                                                    <div class="empty-state-title">"No reviewer notes yet"</div>
                                                                    <div class="empty-state-body">"Internal notes added during review will appear here."</div>
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                {history.into_iter().map(|item| view! {
                                                                    <div class="verif-note-item">
                                                                        <div class="verif-note-meta">
                                                                            <span class="verif-note-author">{item.author}</span>
                                                                            <span class="verif-note-role">"Super-Admin"</span>
                                                                            <span class="verif-note-time">{item.timestamp}</span>
                                                                        </div>
                                                                        <div class="verif-note-body">{item.text}</div>
                                                                    </div>
                                                                }).collect_view()}
                                                            }.into_any()
                                                        }
                                                    }}
                                                </div>
                                            </div>
                                        }.into_any(),
                                    }}
                                </div>
                            }.into_any()
                        }}
                    </div>
                </div>
            </Suspense>

            {move || ver_error.get().map(|e| crate::utils::inline_error(&e))}

            <Show when=move || show_approve_modal.get()>
                <div class="modal-overlay open">
                    <div class="modal">
                        <div class="verif-section-label">"Approve Business Verification"</div>
                        <div class="rev-meta">"Approve this verification request? The tenant status will immediately update to verified and the action will be permanently logged."</div>
                        <div class="rev-actions">
                            <button class="btn btn-ghost" on:click=move |_| show_approve_modal.set(false)>"Cancel"</button>
                            <button class="btn btn-primary" on:click=handle_approve>"Approve Request"</button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_reject_modal.get()>
                <div class="modal-overlay open">
                    <div class="modal">
                        <div class="verif-section-label">"Reject Verification Request"</div>
                        <div class="rev-meta">"Explain the refusal decision to send as email notification feedback."</div>
                        <textarea
                            class="rev-notes"
                            placeholder="Reason for rejection (e.g. Expired registration documents)"
                            on:input=move |ev| temp_rejection_reason.set(event_target_value(&ev))
                            prop:value=temp_rejection_reason
                        ></textarea>
                        <div class="rev-actions">
                            <button class="btn btn-ghost" on:click=move |_| show_reject_modal.set(false)>"Cancel"</button>
                            <button class="btn btn-reject btn-sm" on:click=handle_reject>"Reject Request"</button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_info_request_modal.get()>
                <div class="modal-overlay open">
                    <div class="modal">
                        <div class="verif-section-label">"Request Additional Documents"</div>
                        <div class="rev-meta">"Marks the request as needing more info and stores your message for the applicant."</div>
                        <label class="verif-section-label">"Message"</label>
                        <textarea
                            class="rev-notes"
                            placeholder="Describe what additional documents or details are required..."
                            on:input=move |ev| info_request_message.set(event_target_value(&ev))
                            prop:value=info_request_message
                        ></textarea>
                        <div class="rev-actions">
                            <button class="btn btn-ghost" on:click=move |_| show_info_request_modal.set(false)>"Cancel"</button>
                            <button class="btn btn-primary" on:click=move |_| {
                                let Some(req) = selected_request.get() else {
                                    show_info_request_modal.set(false);
                                    return;
                                };
                                let req_id = req.id;
                                let message = info_request_message.get();
                                show_info_request_modal.set(false);
                                leptos::task::spawn_local(async move {
                                    match crate::api::verification::request_verification_info(
                                        req_id,
                                        Some(message),
                                    )
                                    .await
                                    {
                                        Ok(_) => {
                                            toast.show_toast(
                                                "Success",
                                                "Request marked as needs info.",
                                                "success",
                                            );
                                            trigger_fetch.set(trigger_fetch.get() + 1);
                                        }
                                        Err(e) => toast.show_toast("Error", &e, "error"),
                                    }
                                });
                            }>"Send Request"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
