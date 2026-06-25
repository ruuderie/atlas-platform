use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_location};
use uuid::Uuid;
use crate::api::crm::{
    get_lead_by_id, get_account_by_id, get_deal_by_id, get_contact_by_id,
    convert_lead, add_contact_note, get_contact_notes, get_contact_activities,
    log_call_activity,
};
use crate::api::models::{LeadModel, AccountModel, DealModel, ContactModel};
use crate::api::communications::{SendEmailPayload, send_email};
use crate::api::files::{get_admin_presign, put_to_presigned_url};

#[derive(Clone, Debug)]
pub enum EntityDetail {
    Lead(LeadModel),
    Contact(ContactModel),
    Account(AccountModel),
    Deal(DealModel),
    Unknown,
}

#[component]
pub fn CrmDetail() -> impl IntoView {
    let params = use_params_map();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let active_network = use_context::<ReadSignal<Option<Uuid>>>().expect("active network context");

    let location = use_location();
    // Derive entity type from URL path: /leads/:id → "lead", /contacts/:id → "contact", etc.
    let entity_type = move || {
        let path = location.pathname.get();
        if path.starts_with("/leads/")    { "lead".to_string() }
        else if path.starts_with("/contacts/") { "contact".to_string() }
        else if path.starts_with("/accounts/") { "account".to_string() }
        else if path.starts_with("/pipeline/") { "deal".to_string() }
        else { String::new() }
    };
    let record_id = move || params.get().get("id").map(|s| s.to_string()).unwrap_or_default();

    let (trigger_refresh, set_trigger_refresh) = signal(0);
    let active_tab = RwSignal::new("overview".to_string());
    let note_content = RwSignal::new("".to_string());

    // ── Email modal state ─────────────────────────────────────────────────────
    let show_email_modal = RwSignal::new(false);
    let email_to    = RwSignal::new("".to_string());
    let email_subj  = RwSignal::new("".to_string());
    let email_body  = RwSignal::new("".to_string());
    let is_sending  = RwSignal::new(false);

    // ── Call modal state ──────────────────────────────────────────────────────
    let show_call_modal     = RwSignal::new(false);
    let call_duration       = RwSignal::new("5".to_string());
    let call_direction      = RwSignal::new("outbound".to_string());
    let call_outcome        = RwSignal::new("connected".to_string());
    let call_notes          = RwSignal::new("".to_string());
    let call_transcript_key = RwSignal::new(Option::<String>::None);
    let call_transcript_name = RwSignal::new("".to_string());
    let is_logging_call     = RwSignal::new(false);
    let is_uploading_transcript = RwSignal::new(false);

    let details_res = LocalResource::new(
        move || {
            trigger_refresh.get();
            let entity = entity_type();
            let id = record_id();
            async move {
                match entity.as_str() {
                    "lead"    => get_lead_by_id(&id).await.map(EntityDetail::Lead).unwrap_or(EntityDetail::Unknown),
                    "contact" => get_contact_by_id(&id).await.map(EntityDetail::Contact).unwrap_or(EntityDetail::Unknown),
                    "account" => get_account_by_id(&id).await.map(EntityDetail::Account).unwrap_or(EntityDetail::Unknown),
                    "deal"    => get_deal_by_id(&id).await.map(EntityDetail::Deal).unwrap_or(EntityDetail::Unknown),
                    _ => EntityDetail::Unknown,
                }
            }
        }
    );

    let notes_res = LocalResource::new(move || {
        trigger_refresh.get();
        let entity = entity_type();
        let id = record_id();
        async move {
            if entity == "contact" || entity == "lead" {
                get_contact_notes(&id).await.unwrap_or_default()
            } else {
                Vec::new()
            }
        }
    });

    let activities_res = LocalResource::new(move || {
        trigger_refresh.get();
        let entity = entity_type();
        let id = record_id();
        async move {
            if entity == "contact" || entity == "lead" {
                get_contact_activities(&id).await.unwrap_or_default()
            } else {
                Vec::new()
            }
        }
    });

    // ── Handlers ──────────────────────────────────────────────────────────────

    let handle_convert_lead = move |_| {
        let id = record_id();
        let toast = toast.clone();
        let navigate = leptos_router::hooks::use_navigate();
        leptos::task::spawn_local(async move {
            match convert_lead(&id).await {
                Ok(contact) => {
                    toast.show_toast("CRM", "Lead qualified and converted to Contact!", "success");
                    navigate(&format!("/contacts/{}", contact.id), Default::default());
                }
                Err(e) => {
                    toast.show_toast("Error", &format!("Failed to convert lead: {}", e), "error");
                }
            }
        });
    };

    let handle_add_note = move |_| {
        let id = record_id();
        let content = note_content.get();
        if content.is_empty() { return; }
        let toast = toast.clone();
        leptos::task::spawn_local(async move {
            match add_contact_note(&id, &content).await {
                Ok(_) => {
                    toast.show_toast("CRM", "Note added successfully!", "success");
                    note_content.set("".to_string());
                    set_trigger_refresh.update(|v| *v += 1);
                }
                Err(e) => {
                    toast.show_toast("Error", &format!("Failed to add note: {}", e), "error");
                }
            }
        });
    };

    // Pre-populate email To field from the current record's email
    let open_email_modal = move |_| {
        let email = match details_res.get() {
            Some(EntityDetail::Lead(ref l))    => l.email.clone().unwrap_or_default(),
            Some(EntityDetail::Contact(ref c)) => c.email.clone().unwrap_or_default(),
            _ => String::new(),
        };
        email_to.set(email);
        email_subj.set("".to_string());
        email_body.set("".to_string());
        show_email_modal.set(true);
    };

    let handle_send_email = move |_| {
        let tenant_id = match active_network.get() {
            Some(id) => id,
            None => {
                toast.show_toast("Error", "No active network selected.", "error");
                return;
            }
        };
        let to = email_to.get();
        let subj = email_subj.get();
        let body = email_body.get();
        if to.is_empty() || subj.is_empty() {
            toast.show_toast("Error", "To and Subject are required.", "error");
            return;
        }
        is_sending.set(true);
        let toast2 = toast.clone();
        leptos::task::spawn_local(async move {
            let payload = SendEmailPayload {
                tenant_id,
                to_email: to.clone(),
                subject: subj,
                body_html: body,
                attachments: vec![],
            };
            match send_email(payload).await {
                Ok(r) => {
                    let msg = if r.message.contains("mocked") {
                        format!("Email queued (preview mode — SMTP not configured): {}", to)
                    } else {
                        format!("Email sent to {}", to)
                    };
                    toast2.show_toast("Email", &msg, "success");
                    show_email_modal.set(false);
                }
                Err(e) => toast2.show_toast("Error", &e, "error"),
            }
            is_sending.set(false);
        });
    };

    // Transcript upload handler — called when user picks a file
    let handle_transcript_file = move |ev: leptos::ev::Event| {
        use wasm_bindgen::JsCast;
        let input: web_sys::HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
        let files = input.files().unwrap();
        if files.length() == 0 { return; }
        let file = files.item(0).unwrap();
        let name = file.name();
        let mime = file.type_();
        is_uploading_transcript.set(true);
        let toast2 = toast.clone();
        leptos::task::spawn_local(async move {
            // 1. Get presigned URL
            let presign = match get_admin_presign(&name, &mime, "transcripts").await {
                Ok(p) => p,
                Err(e) => {
                    toast2.show_toast("Error", &format!("Presign failed: {}", e), "error");
                    is_uploading_transcript.set(false);
                    return;
                }
            };
            // 2. Read file bytes
            use wasm_bindgen_futures::JsFuture;
            let array_buf = JsFuture::from(file.array_buffer()).await.unwrap();
            let bytes = js_sys::Uint8Array::new(&array_buf).to_vec();
            // 3. PUT to R2
            if let Err(e) = put_to_presigned_url(&presign.upload_url, bytes, &mime).await {
                toast2.show_toast("Error", &format!("Upload failed: {}", e), "error");
                is_uploading_transcript.set(false);
                return;
            }
            // 4. Store file_key for use when logging call
            call_transcript_key.set(Some(presign.file_key));
            call_transcript_name.set(name);
            toast2.show_toast("Transcript", "Transcript uploaded. Ready to log call.", "success");
            is_uploading_transcript.set(false);
        });
    };

    let handle_log_call = move |_| {
        let entity = entity_type();
        let id = record_id();
        let duration: u32 = call_duration.get().parse().unwrap_or(5);
        let direction = call_direction.get();
        let outcome = call_outcome.get();
        let notes = call_notes.get();
        let transcript_key = call_transcript_key.get();
        let file_paths: Vec<String> = transcript_key.into_iter().collect();

        let lead_id    = if entity == "lead"    { Some(id.clone()) } else { None };
        let contact_id = if entity == "contact" { Some(id.clone()) } else { None };
        let account_id = if entity == "account" { Some(id.clone()) } else { None };

        is_logging_call.set(true);
        let toast2 = toast.clone();
        leptos::task::spawn_local(async move {
            match log_call_activity(
                lead_id.as_deref(),
                contact_id.as_deref(),
                account_id.as_deref(),
                duration,
                &direction,
                &outcome,
                &notes,
                file_paths,
            ).await {
                Ok(_) => {
                    toast2.show_toast("CRM", "Call logged successfully.", "success");
                    show_call_modal.set(false);
                    call_notes.set("".to_string());
                    call_transcript_key.set(None);
                    call_transcript_name.set("".to_string());
                    set_trigger_refresh.update(|v| *v += 1);
                }
                Err(e) => toast2.show_toast("Error", &e, "error"),
            }
            is_logging_call.set(false);
        });
    };

    let record_name = move || match details_res.get() {
        Some(EntityDetail::Lead(ref l))    => l.name.clone(),
        Some(EntityDetail::Contact(ref c)) => c.name.clone(),
        Some(EntityDetail::Account(ref a)) => a.name.clone(),
        Some(EntityDetail::Deal(ref d))    => d.name.clone(),
        _ => "Record Details".to_string(),
    };

    let avatar_initials = move || {
        let name = record_name();
        name.split_whitespace()
            .map(|w| w.chars().next().unwrap_or('?'))
            .collect::<String>()
            .chars()
            .take(2)
            .collect::<String>()
            .to_uppercase()
    };

    view! {
        <div class="main-area" style="overflow-y: auto;">
            <Suspense fallback=move || view! { <div class="p-8 text-center text-on-surface-variant">"Loading details..."</div> }>
                {move || match details_res.get() {
                    Some(EntityDetail::Lead(l)) => view! {
                        // ── Lead Layout ──
                        <div class="rec-hdr">
                            <div class="breadcrumb">
                                <a href="/leads">"Leads"</a>" › "{l.name.clone()}
                            </div>
                            <div class="rec-identity">
                                <div class="rec-left">
                                    <div class="lead-avatar">{avatar_initials}</div>
                                    <div>
                                        <div class="rec-name">
                                            {l.name.clone()}
                                        </div>
                                        <div class="rec-meta">
                                            {l.id.clone()}
                                        </div>
                                        <div class="rec-actions-row">
                                            <button class="btn btn-ghost btn-sm"
                                                on:click=open_email_modal
                                            >"✉ Email"</button>
                                            <button class="btn btn-ghost btn-sm"
                                                on:click=move |_| show_call_modal.set(true)
                                            >"📞 Log Call"</button>
                                            <button class="btn btn-convert btn-sm" on:click=handle_convert_lead>"⇉ Convert Lead"</button>
                                        </div>
                                    </div>
                                </div>
                                <div style="text-align:right;font-size:11px;color:var(--text-muted)">
                                    <div>"Lead Owner: —"</div>
                                    <div style="margin-top:3px">"G-27 Score: "<span style="color:var(--text-muted);font-weight:600">"—"</span></div>
                                </div>
                            </div>
                        </div>

                        {
                            let status_str    = l.lead_status.clone().unwrap_or_else(|| "New".to_string());
                            let stages        = ["New", "Contacted", "Qualified", "Proposal"];
                            let current_idx   = stages.iter().position(|&s| s == status_str.as_str()).unwrap_or(0);
                            let is_converted  = status_str == "Converted";
                            let is_disqualified = status_str == "Disqualified";
                            let sc = move |i: usize| -> &'static str {
                                if is_converted { return "sf-step done"; }
                                if i < current_idx { "sf-step done" }
                                else if i == current_idx { "sf-step current" }
                                else { "sf-step future" }
                            };
                            let terminal_class = if is_disqualified { "sf-step terminal-lost" }
                                else if is_converted { "sf-step terminal-won" }
                                else { "sf-step future" };
                            let terminal_label = if is_disqualified { "Disqualified" } else { "Converted" };
                            view! {
                                <div class="status-flow">
                                    <div class=sc(0)><div class="sf-pill">"New"</div></div>
                                    <div class="sf-arrow">"→"</div>
                                    <div class=sc(1)><div class="sf-pill">"Contacted"</div></div>
                                    <div class="sf-arrow">"→"</div>
                                    <div class=sc(2)><div class="sf-pill">"Qualified"</div></div>
                                    <div class="sf-arrow">"→"</div>
                                    <div class=sc(3)><div class="sf-pill">"Proposal"</div></div>
                                    <div class="sf-arrow">"→"</div>
                                    <div class=terminal_class><div class="sf-pill">{terminal_label}</div></div>
                                </div>
                            }
                        }

                        <div class="kpi-strip">
                            <div class="kpi"><div class="kpi-label">"Annual Revenue"</div><div class="kpi-value mono" style="color:var(--green)">"—"</div><div class="kpi-sub">"Not yet loaded"</div></div>
                            <div class="kpi"><div class="kpi-label">"Employees"</div><div class="kpi-value mono">"—"</div></div>
                            <div class="kpi"><div class="kpi-label">"Activities"</div><div class="kpi-value mono">{move || activities_res.get().unwrap_or_default().len().to_string()}</div></div>
                            <div class="kpi"><div class="kpi-label">"G-27 Score"</div><div class="kpi-value" style="font-size:15px;color:var(--text-muted)">"—"</div><div class="kpi-sub">"Not yet loaded"</div></div>
                        </div>

                        <div class="tab-bar">
                            <button class=move || format!("tab {}", if active_tab.get() == "overview" { "active" } else { "" }) on:click=move |_| active_tab.set("overview".to_string())>"Overview"</button>
                            <button class=move || format!("tab {}", if active_tab.get() == "details" { "active" } else { "" }) on:click=move |_| active_tab.set("details".to_string())>"All Fields · G-31"</button>
                        </div>

                        <div class="content-body">
                            {move || match active_tab.get().as_str() {
                                "overview" => view! {
                                    <div class="col-7-5">
                                        <div>
                                            <div class="convert-panel">
                                                <div class="convert-panel-hdr">"⇉ This lead is ready to convert"</div>
                                                <div class="convert-panel-body">
                                                    "This lead meets conversion parameters. Converting will atomically create an Account, a Contact, and an Opportunity."
                                                </div>
                                                <button class="btn btn-convert btn-sm" on:click=handle_convert_lead>"Qualification Conversion →"</button>
                                            </div>

                                            <div class="card">
                                                <div class="composer">
                                                    <div class="composer-tabs">
                                                        <button class="c-tab active">"Note"</button>
                                                    </div>
                                                    <textarea
                                                        class="w-full bg-[#1C2236] border border-outline-variant/30 rounded p-2 text-sm text-text-primary"
                                                        placeholder="Log activity on this record..."
                                                        prop:value=move || note_content.get()
                                                        on:input=move |e| note_content.set(event_target_value(&e))
                                                    ></textarea>
                                                    <div class="composer-footer">
                                                        <button class="btn btn-primary btn-sm" on:click=handle_add_note>"Save Note"</button>
                                                    </div>
                                                </div>
                                            </div>

                                            <div class="card">
                                                <div class="card-hdr"><span class="card-title">"Activity Timeline · G-29"</span></div>
                                                <div>
                                                    {move || notes_res.get().unwrap_or_default().into_iter().map(|n| view! {
                                                        <div class="activity-item">
                                                            <div class="act-icon" style="background:var(--cobalt-dim);color:var(--cobalt)">"📝"</div>
                                                            <div class="act-body">
                                                                <div class="act-title">"Internal Note added"</div>
                                                                <div class="act-meta">"System User · " {n.created_at}</div>
                                                                <div class="act-desc">{n.content}</div>
                                                            </div>
                                                        </div>
                                                    }).collect_view()}
                                                    {move || activities_res.get().unwrap_or_default().into_iter().map(|a| view! {
                                                        <div class="activity-item">
                                                            <div class="act-icon" style="background:var(--green-dim);color:var(--green)">"🤝"</div>
                                                            <div class="act-body">
                                                                <div class="act-title">{a.activity_type}</div>
                                                                <div class="act-meta">"System User · " {a.created_at}</div>
                                                                <div class="act-desc">{a.description}</div>
                                                            </div>
                                                        </div>
                                                    }).collect_view()}
                                                </div>
                                            </div>
                                        </div>
                                        <div>
                                            <div class="card">
                                                <div class="card-hdr"><span class="card-title">"Lead Info"</span></div>
                                                <div class="stat-row"><span class="s-label">"Status"</span><span class="s-value cobalt">{l.lead_status.clone().unwrap_or_else(|| "New".to_string())}</span></div>
                                                <div class="stat-row"><span class="s-label">"Source"</span><span class="s-value">{l.source.clone().unwrap_or_else(|| "—".to_string())}</span></div>
                                                <div class="stat-row"><span class="s-label">"Lead Owner"</span><span class="s-value">"—"</span></div>
                                            </div>
                                            <div class="card">
                                                <div class="card-hdr"><span class="card-title">"Contact Info"</span></div>
                                                <div class="stat-row"><span class="s-label">"Email"</span><span class="s-value">{l.email.clone().unwrap_or_else(|| "—".to_string())}</span></div>
                                                <div class="stat-row"><span class="s-label">"Phone"</span><span class="s-value">{l.phone.clone().unwrap_or_else(|| "—".to_string())}</span></div>
                                                <div class="stat-row"><span class="s-label">"Company"</span><span class="s-value">{l.company.clone().unwrap_or_else(|| "—".to_string())}</span></div>
                                            </div>
                                        </div>
                                    </div>
                                }.into_any(),
                                "details" => view! {
                                    <div class="card">
                                        <div class="card-hdr"><span class="card-title">"All Fields · G-31"</span></div>
                                        <div class="field-grid">
                                            <div class="field-row"><span class="f-label">"ID"</span><span class="f-value mono">{l.id.clone()}</span></div>
                                            <div class="field-row"><span class="f-label">"Name"</span><span class="f-value">{l.name.clone()}</span></div>
                                            <div class="field-row"><span class="f-label">"Email"</span><span class="f-value">{l.email.clone().unwrap_or_default()}</span></div>
                                        </div>
                                    </div>
                                }.into_any(),
                                _ => view! {}.into_any()
                            }}
                        </div>
                    }.into_any(),

                    Some(EntityDetail::Account(a)) => view! {
                        // ── Account Layout ──
                        <div class="rec-hdr">
                            <div class="breadcrumb">
                                <a href="/accounts">"Accounts"</a>" › "{a.name.clone()}
                            </div>
                            <div class="rec-identity">
                                <div class="rec-left">
                                    <div class="lead-avatar" style="background:var(--cobalt-dim);color:var(--cobalt);border-color:var(--cobalt);">{avatar_initials}</div>
                                    <div>
                                        <div class="rec-name">
                                            {a.name.clone()}
                                            <span class="tag" style="color:var(--cobalt);border-color:var(--cobalt)">"Organization"</span>
                                            <span class="tag" style="color:var(--green);border-color:var(--green);background:var(--green-dim)">"Active"</span>
                                        </div>
                                        <div class="rec-meta">
                                            "atlas_accounts · G-31 · " {a.id.clone()} " · Brazil · CNPJ 47.382.910/0001-88"
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="kpi-strip">
                            <div class="kpi"><div class="kpi-label">"Open Opportunities"</div><div class="kpi-value">"—"</div></div>
                            <div class="kpi"><div class="kpi-label">"Contacts"</div><div class="kpi-value">"—"</div></div>
                            <div class="kpi"><div class="kpi-label">"Annual Revenue"</div><div class="kpi-value">"—"</div></div>
                            <div class="kpi"><div class="kpi-label">"Employees"</div><div class="kpi-value">"—"</div></div>
                        </div>

                        <div class="content-body">
                            <div class="col-7-5">
                                <div>
                                    <div class="card">
                                        <div class="card-hdr"><span class="card-title">"Account Info"</span></div>
                                        <div class="stat-row"><span class="s-label">"Type"</span><span class="s-value">"—"</span></div>
                                        <div class="stat-row"><span class="s-label">"Industry"</span><span class="s-value">"—"</span></div>
                                        <div class="stat-row"><span class="s-label">"Website"</span><span class="s-value">"—"</span></div>
                                    </div>
                                </div>
                                <div>
                                    <div class="card">
                                        <div class="card-hdr"><span class="card-title">"Primary Contact"</span></div>
                                        <div class="card-body">
                                            <div style="font-size:12px;color:var(--text-muted);">"No primary contact linked yet."</div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any(),

                    Some(EntityDetail::Contact(c)) => view! {
                        // ── Contact Layout ──
                        <div class="rec-hdr">
                            <div class="breadcrumb">
                                <a href="/contacts">"Contacts"</a>" › "{c.name.clone()}
                            </div>
                            <div class="rec-identity">
                                <div class="rec-left">
                                    <div class="lead-avatar" style="background:var(--cobalt-dim);color:var(--cobalt);border-color:var(--cobalt);">{avatar_initials}</div>
                                    <div>
                                        <div class="rec-name">
                                            {c.name.clone()}
                                            <span class="tag tag-verified">"Verified"</span>
                                        </div>
                                        <div class="rec-meta">
                                            "atlas_contacts · G-31 · " {c.id.clone()}
                                        </div>
                                        <div class="rec-actions-row">
                                            <button class="btn btn-ghost btn-sm"
                                                on:click=open_email_modal
                                            >"✉ Email"</button>
                                            <button class="btn btn-ghost btn-sm"
                                                on:click=move |_| show_call_modal.set(true)
                                            >"📞 Log Call"</button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="kpi-strip">
                            <div class="kpi"><div class="kpi-label">"Verification Status"</div><div class="kpi-value" style="color:var(--green)">"Verified"</div><div class="kpi-sub">"G-06 Cleared"</div></div>
                            <div class="kpi"><div class="kpi-label">"Last Active"</div><div class="kpi-value">"2 hours ago"</div><div class="kpi-sub">"Active Portal"</div></div>
                        </div>

                        <div class="content-body">
                            <div class="col-7-5">
                                <div>
                                    <div class="card">
                                        <div class="card-hdr"><span class="card-title">"Contact Info"</span></div>
                                        <div class="stat-row"><span class="s-label">"Email"</span><span class="s-value">{c.email.clone().unwrap_or_default()}</span></div>
                                        <div class="stat-row"><span class="s-label">"Phone"</span><span class="s-value">{c.phone.clone().unwrap_or_default()}</span></div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any(),

                    Some(EntityDetail::Deal(d)) => view! {
                        // ── Opportunity/Deal Layout ──
                        <div class="rec-hdr">
                            <div class="breadcrumb">
                                <a href="/pipeline">"Pipeline"</a>" › "{d.name.clone()}
                            </div>
                            <div class="rec-identity">
                                <div class="rec-left">
                                    <div class="lead-avatar" style="background:var(--amber-dim);color:var(--amber);border-color:var(--amber);">{avatar_initials}</div>
                                    <div>
                                        <div class="rec-name">
                                            {d.name.clone()}
                                            <span class="tag" style="color:var(--violet);border-color:var(--violet)">{d.stage.clone()}</span>
                                        </div>
                                        <div class="rec-meta">
                                            "atlas_deals · G-11 · " {d.id.clone()}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="kpi-strip">
                            <div class="kpi"><div class="kpi-label">"Deal Value"</div><div class="kpi-value mono" style="color:var(--cobalt)">{format!("${:.2}", d.amount)}</div></div>
                            <div class="kpi"><div class="kpi-label">"Win Probability"</div><div class="kpi-value">"—"</div></div>
                        </div>

                        <div class="content-body">
                            <div class="col-7-5">
                                <div>
                                    <div class="card">
                                        <div class="card-hdr"><span class="card-title">"Deal Details"</span></div>
                                        <div class="stat-row"><span class="s-label">"Stage"</span><span class="s-value">{d.stage.clone()}</span></div>
                                        <div class="stat-row"><span class="s-label">"Status"</span><span class="s-value">{d.status.clone()}</span></div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_any(),

                    _ => view! {
                        <div class="p-8 text-center text-on-surface-variant">"Record not found."</div>
                    }.into_any()
                }}
            </Suspense>

            // ── Email Compose Modal ───────────────────────────────────────────
            <Show when=move || show_email_modal.get()>
                <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[1000]">
                    <div class="bg-[#111520] border border-outline-variant/40 rounded-xl p-6 w-full max-w-lg shadow-2xl">
                        <div class="flex justify-between items-center mb-4">
                            <h3 class="text-base font-bold text-on-surface">"Compose Email"</h3>
                            <button class="text-on-surface-variant hover:text-on-surface font-bold"
                                on:click=move |_| show_email_modal.set(false)>"✕"</button>
                        </div>
                        <div class="space-y-3">
                            <div>
                                <label class="text-xs text-muted-foreground block mb-1">"To"</label>
                                <input type="email" class="w-full bg-surface-container border border-outline-variant/30 rounded px-3 py-2 text-sm text-on-surface"
                                    prop:value=move || email_to.get()
                                    on:input=move |e| email_to.set(event_target_value(&e))
                                    placeholder="recipient@example.com"
                                />
                            </div>
                            <div>
                                <label class="text-xs text-muted-foreground block mb-1">"Subject"</label>
                                <input type="text" class="w-full bg-surface-container border border-outline-variant/30 rounded px-3 py-2 text-sm text-on-surface"
                                    prop:value=move || email_subj.get()
                                    on:input=move |e| email_subj.set(event_target_value(&e))
                                    placeholder="Subject line"
                                />
                            </div>
                            <div>
                                <label class="text-xs text-muted-foreground block mb-1">"Message"</label>
                                <textarea class="w-full bg-surface-container border border-outline-variant/30 rounded px-3 py-2 text-sm text-on-surface h-32 resize-none"
                                    placeholder="Email body..."
                                    prop:value=move || email_body.get()
                                    on:input=move |e| email_body.set(event_target_value(&e))
                                ></textarea>
                            </div>
                            <div class="flex justify-end gap-2 pt-2">
                                <button class="btn btn-ghost btn-sm" on:click=move |_| show_email_modal.set(false)>"Cancel"</button>
                                <button class="btn btn-primary btn-sm"
                                    on:click=handle_send_email
                                    disabled=move || is_sending.get()
                                >{move || if is_sending.get() { "Sending…" } else { "Send Email" }}</button>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Log Call Modal ────────────────────────────────────────────────
            <Show when=move || show_call_modal.get()>
                <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[1000]">
                    <div class="bg-[#111520] border border-outline-variant/40 rounded-xl p-6 w-full max-w-md shadow-2xl">
                        <div class="flex justify-between items-center mb-4">
                            <h3 class="text-base font-bold text-on-surface">"📞 Log Call"</h3>
                            <button class="text-on-surface-variant hover:text-on-surface font-bold"
                                on:click=move |_| show_call_modal.set(false)>"✕"</button>
                        </div>
                        <div class="space-y-3">
                            <div class="grid grid-cols-2 gap-3">
                                <div>
                                    <label class="text-xs text-muted-foreground block mb-1">"Direction"</label>
                                    <select class="w-full bg-surface-container border border-outline-variant/30 rounded px-3 py-2 text-sm text-on-surface"
                                        on:change=move |e| call_direction.set(event_target_value(&e))
                                    >
                                        <option value="outbound" selected=true>"Outbound"</option>
                                        <option value="inbound">"Inbound"</option>
                                    </select>
                                </div>
                                <div>
                                    <label class="text-xs text-muted-foreground block mb-1">"Outcome"</label>
                                    <select class="w-full bg-surface-container border border-outline-variant/30 rounded px-3 py-2 text-sm text-on-surface"
                                        on:change=move |e| call_outcome.set(event_target_value(&e))
                                    >
                                        <option value="connected" selected=true>"Connected"</option>
                                        <option value="voicemail">"Voicemail"</option>
                                        <option value="no_answer">"No Answer"</option>
                                    </select>
                                </div>
                            </div>
                            <div>
                                <label class="text-xs text-muted-foreground block mb-1">"Duration (minutes)"</label>
                                <input type="number" min="0" class="w-full bg-surface-container border border-outline-variant/30 rounded px-3 py-2 text-sm text-on-surface"
                                    prop:value=move || call_duration.get()
                                    on:input=move |e| call_duration.set(event_target_value(&e))
                                />
                            </div>
                            <div>
                                <label class="text-xs text-muted-foreground block mb-1">"Notes"</label>
                                <textarea class="w-full bg-surface-container border border-outline-variant/30 rounded px-3 py-2 text-sm text-on-surface h-20 resize-none"
                                    placeholder="Call summary..."
                                    prop:value=move || call_notes.get()
                                    on:input=move |e| call_notes.set(event_target_value(&e))
                                ></textarea>
                            </div>
                            <div>
                                <label class="text-xs text-muted-foreground block mb-1">
                                    "Transcript (optional · .txt / .pdf)"
                                </label>
                                {move || {
                                    let key = call_transcript_key.get();
                                    let name = call_transcript_name.get();
                                    if let Some(_) = key {
                                        view! {
                                            <div class="flex items-center gap-2 text-xs text-green-400">
                                                <span class="material-symbols-outlined text-[14px]">"check_circle"</span>
                                                {name}
                                                <button class="text-red-400 ml-auto" on:click=move |_| {
                                                    call_transcript_key.set(None);
                                                    call_transcript_name.set("".to_string());
                                                }>"✕"</button>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <label class="flex items-center gap-2 cursor-pointer border border-dashed border-outline-variant/40 rounded px-3 py-2 text-xs text-muted-foreground hover:border-outline-variant/80 transition-colors">
                                                {move || if is_uploading_transcript.get() { "Uploading…" } else { "Click to attach transcript" }}
                                                <input type="file" accept=".txt,.pdf" class="hidden"
                                                    on:change=handle_transcript_file
                                                    disabled=move || is_uploading_transcript.get()
                                                />
                                            </label>
                                        }.into_any()
                                    }
                                }}
                            </div>
                            <div class="flex justify-end gap-2 pt-2">
                                <button class="btn btn-ghost btn-sm" on:click=move |_| show_call_modal.set(false)>"Cancel"</button>
                                <button class="btn btn-primary btn-sm"
                                    on:click=handle_log_call
                                    disabled=move || is_logging_call.get() || is_uploading_transcript.get()
                                >{move || if is_logging_call.get() { "Logging…" } else { "Log Call" }}</button>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
