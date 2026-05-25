use leptos::prelude::*;
use shared_ui::components::crm_stage_bar::{CrmStageBar, CrmStatusOption};
use shared_ui::components::crm_timeline_generic::{
    CrmTimelineGeneric, NoteModel, ActivityModel, ActivityType, ActivityStatus, FileModel
};
use shared_ui::utils::ResourceState;
use shared_ui::components::file_attachments::{FileAttachments, RecordDocumentModel};

use super::*;

#[component]
pub fn LeadTable() -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();
    let leads_res = Resource::new(move || refresh.get(), |_| get_leads());
    let statuses_res = Resource::new(|| (), |_| get_lead_crm_statuses());

    let location = leptos_router::hooks::use_location();
    let navigate = leptos_router::hooks::use_navigate();

    // Parse lead ID from URL path: e.g., "/admin/leads/123-456"
    let id_from_url = move || {
        let path = location.pathname.get();
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 3 && parts[1] == "leads" {
            uuid::Uuid::parse_str(parts[2]).ok()
        } else {
            None
        }
    };

    let (selected_lead, set_selected_lead) = signal::<Option<LeadRecord>>(None);

    Effect::new(move |_| {
        if let Some(Ok(items)) = leads_res.get() {
            if let Some(target_id) = id_from_url() {
                if let Some(matched) = items.iter().find(|l| l.id == target_id) {
                    if selected_lead.get_untracked().map(|l| l.id) != Some(target_id) {
                        set_selected_lead.set(Some(matched.clone()));
                    }
                } else {
                    set_selected_lead.set(None);
                }
            } else {
                set_selected_lead.set(None);
            }
        }
    });

    view! {
        <Transition fallback=move || view! { <div class="jetbrains text-sm text-outline">"QUERYING_DB..."</div> }>
            {move || {
                let navigate = navigate.clone();
                let res = leads_res.get();
                let statuses = statuses_res.get().and_then(|r| r.ok()).unwrap_or_default();
                view! {
                    <div class="relative w-full">
                        <Show
                            when=move || selected_lead.get().is_none()
                            fallback={
                                let navigate = navigate.clone();
                                let statuses = statuses.clone();
                                move || {
                                    let navigate = navigate.clone();
                                    let statuses = statuses.clone();
                                    view! {
                                        {move || selected_lead.get().map(|lead| {
                                            let navigate = navigate.clone();
                                            view! {
                                                <LeadCrmPane 
                                                    lead_record=lead
                                                    stages=statuses.clone()
                                                    on_close=Callback::new(move |_: ()| {
                                                        let _ = navigate("/admin/leads", Default::default());
                                                    })
                                                />
                                            }
                                        })}
                                    }
                                }
                            }
                        >
                            // Table container
                            <div class="overflow-x-auto bg-surface-container-lowest border border-outline-variant/30 rounded-xl p-6 shadow-sm">
                                <table class="w-full text-left jetbrains text-sm">
                                    <thead>
                                        <tr class="text-outline border-b border-outline-variant/30 uppercase text-xs tracking-wider">
                                            <th class="py-4 px-4 font-semibold">"Name"</th>
                                            <th class="py-4 px-4 font-semibold">"Contact"</th>
                                            <th class="py-4 px-4 font-semibold">"Company / Title"</th>
                                            <th class="py-4 px-4 font-semibold">"Status"</th>
                                            <th class="py-4 px-4 font-semibold">"Source"</th>
                                            <th class="py-4 px-4 font-semibold">"Created"</th>
                                            <th class="py-4 px-4 font-semibold text-right">"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody class="divide-y divide-outline-variant/20">
                                        {match ResourceState::from(res.clone()) {
                                            ResourceState::Ready(items) => {
                                                if items.is_empty() {
                                                    view! {
                                                        <tr>
                                                            <td colspan="7" class="py-12 text-center text-outline-variant">
                                                                "NO_ACTIVE_LEADS"
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                } else {
                                                    items.into_iter().map(|lead| {
                                                        let c = lead.clone();
                                                        let navigate = navigate.clone();
                                                        let email_disp = lead.email.clone().unwrap_or_else(|| "-".to_string());
                                                        let phone_disp = lead.phone.clone().unwrap_or_else(|| "-".to_string());
                                                        let company_disp = lead.company.clone().unwrap_or_else(|| "-".to_string());
                                                        let title_disp = lead.title.clone().unwrap_or_else(|| "-".to_string());
                                                        let status_disp = lead.lead_status.clone().unwrap_or_else(|| "New".to_string());
                                                        let source_disp = lead.source.clone().unwrap_or_else(|| "Unknown".to_string());
                                                        
                                                        // Dynamic pipeline-based status badge styling
                                                        let matched_color = statuses.iter()
                                                            .find(|s| s.status_key.to_lowercase() == status_disp.to_lowercase())
                                                            .map(|s| s.color.as_str())
                                                            .unwrap_or("slate");
                                                            
                                                        let badge_classes = match matched_color {
                                                            "blue" => "bg-blue-500/10 text-blue-500 border-blue-500/20",
                                                            "purple" => "bg-purple-500/10 text-purple-500 border-purple-500/20",
                                                            "indigo" => "bg-indigo-500/10 text-indigo-500 border-indigo-500/20",
                                                            "orange" => "bg-orange-500/10 text-orange-500 border-orange-500/20",
                                                            "emerald" => "bg-emerald-500/10 text-emerald-500 border-emerald-500/20",
                                                            "rose" => "bg-rose-500/10 text-rose-500 border-rose-500/20",
                                                            _ => "bg-slate-500/10 text-slate-400 border-slate-500/20",
                                                        };
 
                                                        view! {
                                                            <tr 
                                                                class="hover:bg-surface-container-high transition-all duration-150 cursor-pointer"
                                                                on:click={
                                                                    let navigate = navigate.clone();
                                                                    move |_| {
                                                                        let _ = navigate(&format!("/admin/leads/{}", c.id), Default::default());
                                                                    }
                                                                }
                                                            >
                                                                <td class="py-4 px-4 font-bold text-primary">{lead.name}</td>
                                                                <td class="py-4 px-4">
                                                                    <div class="text-xs text-outline">{email_disp}</div>
                                                                    <div class="text-[10px] text-outline-variant">{phone_disp}</div>
                                                                </td>
                                                                <td class="py-4 px-4 text-xs">
                                                                    <div class="font-semibold">{company_disp}</div>
                                                                    <div class="text-outline-variant text-[10px]">{title_disp}</div>
                                                                </td>
                                                                <td class="py-4 px-4">
                                                                    <span class=format!("px-2 py-0.5 border rounded text-[10px] font-bold {}", badge_classes)>
                                                                        {status_disp}
                                                                    </span>
                                                                </td>
                                                                <td class="py-4 px-4 text-outline text-xs">{source_disp}</td>
                                                                <td class="py-4 px-4 text-outline-variant text-xs">{lead.created_at.chars().take(10).collect::<String>()}</td>
                                                                <td class="py-4 px-4 text-right">
                                                                    <button 
                                                                        on:click={
                                                                            let navigate = navigate.clone();
                                                                            move |e| {
                                                                                e.stop_propagation();
                                                                                let id = lead.id;
                                                                                let navigate = navigate.clone();
                                                                                leptos::task::spawn_local(async move {
                                                                                    if let Ok(_) = delete_lead(id).await {
                                                                                        set_refresh.set(refresh.get_untracked() + 1);
                                                                                        if selected_lead.get().map(|s| s.id) == Some(id) {
                                                                                            let _ = navigate("/admin/leads", Default::default());
                                                                                        }
                                                                                    }
                                                                                });
                                                                            }
                                                                        } 
                                                                        class="text-error hover:underline text-xs tracking-wider uppercase font-bold"
                                                                    >
                                                                        "Drop"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect::<Vec<_>>().into_any()
                                                }
                                            }
                                            ResourceState::Loading => view! { <tr class="hidden"></tr> }.into_any(),
                                            ResourceState::Error(_) => view! { <tr><td colspan="7" class="py-8 text-center text-error">"ERR_NO_DATA"</td></tr> }.into_any(),
                                        }}
                                    </tbody>
                                </table>
                            </div>
                        </Show>
                    </div>
                }
            }.into_any()
            }
        </Transition>
    }
}

#[component]
fn LeadCrmPane(
    lead_record: LeadRecord,
    stages: Vec<CrmStatusOption>,
    on_close: Callback<()>,
) -> impl IntoView {
    let refresh = expect_context::<ReadSignal<i32>>();
    let set_refresh = expect_context::<WriteSignal<i32>>();

    let (composer_open, set_composer_open) = signal(false);

    let default_templates = vec![
        shared_ui::components::email_composer::EmailTemplate {
            name: "Intake Follow-Up".to_string(),
            subject: "Following up on your intake inquiry".to_string(),
            body: "<p>Hello,</p><p>Thank you for reaching out. We received your details and are currently reviewing your inquiry. We will get back to you shortly with next steps.</p><p>Best regards,<br/>The Operations Team</p>".to_string(),
        },
        shared_ui::components::email_composer::EmailTemplate {
            name: "Proposal Presentation".to_string(),
            subject: "Custom Proposal Presentation".to_string(),
            body: "<p>Hello,</p><p>We are excited to share our custom proposal based on our initial discussion. Please review the attached details and let us know if you have any questions or when you would be available for a quick walkthrough.</p><p>Best regards,<br/>The Consulting Team</p>".to_string(),
        },
    ];
    
    // Internal signals for notes, activities and stages
    let (current_stage, set_current_stage) = signal(lead_record.lead_status.clone().unwrap_or_else(|| "New".to_string()));
    
    let lead_id = lead_record.id;
    let notes_res = Resource::new(move || refresh.get(), move |_| get_lead_notes(lead_id));
    let activities_res = Resource::new(move || refresh.get(), move |_| get_lead_activities(lead_id));
    let attachments_res = Resource::new(move || refresh.get(), move |_| get_lead_attachments(lead_id));

    // Avatar Url State
    let (avatar_url_signal, set_avatar_url_signal) = signal(lead_record.avatar_url.clone());
    let avatar_input_ref = NodeRef::<leptos::html::Input>::new();
    
    let trigger_avatar_upload = move |_| {
        if let Some(input) = avatar_input_ref.get() {
            input.click();
        }
    };

    // Field signals for properties editing
    let (name, set_name) = signal(lead_record.name.clone());
    let (first_name, set_first_name) = signal(lead_record.first_name.clone().unwrap_or_default());
    let (last_name, set_last_name) = signal(lead_record.last_name.clone().unwrap_or_default());
    let (email, set_email) = signal(lead_record.email.clone().unwrap_or_default());
    let (phone, set_phone) = signal(lead_record.phone.clone().unwrap_or_default());
    let (company, set_company) = signal(lead_record.company.clone().unwrap_or_default());
    let (title, set_title) = signal(lead_record.title.clone().unwrap_or_default());
    let (source, set_source) = signal(lead_record.source.clone().unwrap_or_default());
    let (message, set_message) = signal(lead_record.message.clone().unwrap_or_default());
    
    let (edit_mode, set_edit_mode) = signal(false);

    let handle_stage_change = move |new_stage: String| {
        set_current_stage.set(new_stage.clone());
        let stage_cl = new_stage.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = update_lead_stage(lead_id, stage_cl.clone()).await {
                let _ = add_lead_activity(
                    lead_id,
                    ActivityType::Log,
                    "Logged: Stage Change".to_string(),
                    Some(format!("Stage updated to {}", stage_cl)),
                    ActivityStatus::Completed,
                    None,
                    Some(chrono::Utc::now().to_rfc3339()),
                    Vec::new()
                ).await;
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    };

    let (save_error, set_save_error) = signal::<Option<String>>(None);

    let handle_save_details = move |_| {
        let fn_val = first_name.get();
        let ln_val = last_name.get();
        if fn_val.is_empty() && ln_val.is_empty() {
            set_save_error.set(Some("First Name or Last Name is required".to_string()));
            return;
        }
        let n = format!("{} {}", fn_val, ln_val).trim().to_string();
        set_name.set(n.clone());
        set_save_error.set(None);

        let fn_opt = Some(fn_val).filter(|s| !s.is_empty());
        let ln_opt = Some(ln_val).filter(|s| !s.is_empty());
        let em_val = Some(email.get()).filter(|s| !s.is_empty());
        let ph_val = Some(phone.get()).filter(|s| !s.is_empty());
        let co_val = Some(company.get()).filter(|s| !s.is_empty());
        let ti_val = Some(title.get()).filter(|s| !s.is_empty());
        let so_val = Some(source.get()).filter(|s| !s.is_empty());
        let me_val = Some(message.get()).filter(|s| !s.is_empty());
        let av_opt = avatar_url_signal.get_untracked();

        leptos::task::spawn_local(async move {
            match update_lead_details(
                lead_id, n, fn_opt, ln_opt, em_val, ph_val, co_val, ti_val, so_val, me_val, av_opt
            ).await {
                Ok(_) => {
                    set_edit_mode.set(false);
                    set_refresh.set(refresh.get_untracked() + 1);
                }
                Err(e) => {
                    set_save_error.set(Some(format!("Save failed: {}", e)));
                }
            }
        });
    };

    let add_note_cb = Callback::new(move |(content, is_private, files): (String, bool, Vec<FileModel>)| {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = add_lead_note(lead_id, content, is_private, files).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let log_activity_cb = Callback::new(move |(act_type, title, desc, status, due_date, completed_at, files): (ActivityType, String, Option<String>, ActivityStatus, Option<String>, Option<String>, Vec<FileModel>)| {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = add_lead_activity(lead_id, act_type, title, desc, status, due_date, completed_at, files).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let update_activity_status_cb = Callback::new(move |(act_id, status): (uuid::Uuid, ActivityStatus)| {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = update_lead_activity_status(act_id, status).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let delete_note_cb = Callback::new(move |note_id: uuid::Uuid| {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = delete_lead_note(note_id).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let delete_activity_cb = Callback::new(move |act_id: uuid::Uuid| {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        leptos::task::spawn_local(async move {
            if let Ok(_) = delete_lead_activity(act_id).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let add_attachment_cb = Callback::new(move |(file_name, file_url): (String, String)| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = add_lead_attachment(lead_id, file_name, file_url).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let delete_attachment_cb = Callback::new(move |doc_id: uuid::Uuid| {
        leptos::task::spawn_local(async move {
            if let Ok(_) = delete_lead_attachment(doc_id).await {
                set_refresh.set(refresh.get_untracked() + 1);
            }
        });
    });

    let download_attachment_cb = Callback::new(move |file_key: String| {
        leptos::task::spawn_local(async move {
            if let Ok(download_url) = crate::pages::admin::contacts::get_attachment_download_url(file_key).await {
                #[cfg(not(feature = "ssr"))]
                if let Some(win) = web_sys::window() {
                    let _ = win.open_with_url_and_target(&download_url, "_blank");
                }
            }
        });
    });

    let handle_avatar_change = {
        let set_refresh = set_refresh.clone();
        let refresh = refresh.clone();
        let name = name.clone();
        let first_name = first_name.clone();
        let last_name = last_name.clone();
        let email = email.clone();
        let phone = phone.clone();
        let company = company.clone();
        let title = title.clone();
        let source = source.clone();
        let message = message.clone();
        let set_avatar_url_signal = set_avatar_url_signal.clone();
        move |ev: web_sys::Event| {
            #[cfg(not(feature = "ssr"))]
            {
                use leptos::wasm_bindgen::JsCast;
                let target = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                if let Some(input) = target {
                    if let Some(files) = input.files() {
                        if let Some(file) = files.get(0) {
                            let name_val = name.get_untracked();
                            let fn_val = Some(first_name.get_untracked()).filter(|s| !s.is_empty());
                            let ln_val = Some(last_name.get_untracked()).filter(|s| !s.is_empty());
                            let em_val = Some(email.get_untracked()).filter(|s| !s.is_empty());
                            let ph_val = Some(phone.get_untracked()).filter(|s| !s.is_empty());
                            let co_val = Some(company.get_untracked()).filter(|s| !s.is_empty());
                            let ti_val = Some(title.get_untracked()).filter(|s| !s.is_empty());
                            let so_val = Some(source.get_untracked()).filter(|s| !s.is_empty());
                            let me_val = Some(message.get_untracked()).filter(|s| !s.is_empty());
                            let set_refresh = set_refresh.clone();
                            let refresh = refresh.clone();
                            let set_avatar_url_signal = set_avatar_url_signal.clone();
                            
                            leptos::task::spawn_local(async move {
                                if let Ok((_, key)) = shared_ui::components::file_attachments::upload_file_to_s3(file).await {
                                    if let Ok(_) = update_lead_details(
                                        lead_id, name_val, fn_val, ln_val, em_val, ph_val, co_val, ti_val, so_val, me_val, Some(key.clone())
                                    ).await {
                                        set_avatar_url_signal.set(Some(key));
                                        set_refresh.set(refresh.get_untracked() + 1);
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }
    };

    view! {
        <div class="w-full bg-background flex flex-col animate-slide-in font-sans text-on-surface">
            // Breadcrumb navigation header
            <div class="flex items-center gap-2 mb-6 text-xs font-mono text-outline-variant">
                <button 
                    on:click=move |_| on_close.run(()) 
                    class="hover:text-primary transition-colors flex items-center gap-1 font-bold uppercase tracking-wider"
                >
                    <span class="material-symbols-outlined text-[14px]">"arrow_back"</span>
                    "Back to Leads"
                </button>
            </div>

            // Salesforce-style layout container
            <div class="flex flex-col lg:flex-row gap-6 w-full items-start">
                
                // LEFT COLUMN (65% width) - Core info and status
                <div class="w-full lg:w-[65%] space-y-6 flex flex-col">
                    
                    // Main Highlight Panel / Avatar & Quick Details
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs flex flex-col md:flex-row md:items-center justify-between gap-4">
                        <div class="flex items-center gap-4">
                            <input 
                                type="file" 
                                node_ref=avatar_input_ref
                                on:change=handle_avatar_change
                                class="hidden"
                            />
                            <div 
                                on:click=trigger_avatar_upload
                                class="w-14 h-14 rounded-full bg-primary/10 text-primary flex items-center justify-center shrink-0 border border-primary/20 relative group cursor-pointer overflow-hidden"
                            >
                                <Show 
                                    when=move || avatar_url_signal.get().is_some()
                                    fallback=move || {
                                        let name_val = name.get();
                                        let initials: String = name_val.split_whitespace()
                                            .filter_map(|s| s.chars().next())
                                            .take(2)
                                            .collect::<String>()
                                            .to_uppercase();
                                        view! {
                                            <span class="font-bold text-lg">{initials}</span>
                                        }
                                    }
                                >
                                    <img 
                                        src=move || avatar_url_signal.get().unwrap_or_default()
                                        class="w-full h-full object-cover animate-fade-in"
                                    />
                                </Show>
                                <div class="absolute inset-0 bg-black/40 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
                                    <span class="material-symbols-outlined text-white text-[18px]">"photo_camera"</span>
                                </div>
                            </div>
                            <div>
                                <h2 class="text-xl font-bold text-on-surface leading-tight">{move || name.get()}</h2>
                                <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-outline mt-1 font-mono">
                                    <div class="flex items-center gap-1">
                                        <span class="material-symbols-outlined text-[14px]">"mail"</span>
                                        <span>{move || if email.get().is_empty() { "-".to_string() } else { email.get() }}</span>
                                    </div>
                                    <div class="flex items-center gap-1">
                                        <span class="material-symbols-outlined text-[14px]">"call"</span>
                                        <span>{move || if phone.get().is_empty() { "-".to_string() } else { phone.get() }}</span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        // Quick Actions Row
                        <div class="flex items-center gap-2 self-end md:self-auto">
                            <Show when=move || !email.get().is_empty()>
                                <button
                                    on:click=move |_| set_composer_open.set(true)
                                    class="bg-primary text-on-primary px-3 py-1.5 rounded-lg jetbrains text-[10px] font-bold uppercase tracking-wider hover:bg-primary-container transition-colors flex items-center gap-1 shadow-xs"
                                >
                                    <span class="material-symbols-outlined text-xs">"mail"</span>
                                    "Send Email"
                                </button>
                            </Show>
                            <Show when=move || !lead_record.is_converted>
                                <button
                                    on:click=move |_| {
                                        leptos::task::spawn_local(async move {
                                            if let Ok(_) = convert_lead(lead_id).await {
                                                 let _ = add_lead_activity(
                                                     lead_id,
                                                     ActivityType::Log,
                                                     "Logged: Conversion".to_string(),
                                                     Some("Lead converted to contact successfully.".to_string()),
                                                     ActivityStatus::Completed,
                                                     None,
                                                     Some(chrono::Utc::now().to_rfc3339()),
                                                     Vec::new()
                                                 ).await;
                                                set_refresh.set(refresh.get_untracked() + 1);
                                                on_close.run(());
                                            }
                                        });
                                    }
                                    class="bg-emerald-600 text-white px-3 py-1.5 rounded-lg jetbrains text-[10px] font-bold uppercase tracking-wider hover:bg-emerald-700 transition-colors flex items-center gap-1 shadow-xs"
                                >
                                    <span class="material-symbols-outlined text-xs">"person_add"</span>
                                    "Convert"
                                </button>
                            </Show>
                            <button
                                on:click=move |_| set_edit_mode.update(|m| *m = !*m)
                                class="bg-surface-container-high border border-outline-variant/40 px-3 py-1.5 rounded-lg jetbrains text-[10px] font-bold uppercase tracking-wider text-on-surface hover:bg-surface-container-lowest transition-colors flex items-center gap-1 shadow-xs"
                            >
                                <span class="material-symbols-outlined text-xs">"edit"</span>
                                {move || if edit_mode.get() { "Cancel" } else { "Edit Details" }}
                            </button>
                        </div>
                    </div>

                    // Chevron Pipeline Stage Bar Card
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs">
                        <label class="block text-[10px] font-bold uppercase text-outline-variant tracking-wider font-mono mb-3">"Lead Status / Pipeline Stage"</label>
                        <CrmStageBar
                            stages=stages
                            current_stage=current_stage.into()
                            on_stage_change=handle_stage_change
                        />
                    </div>

                    // Details Section
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs space-y-4">
                        <div class="flex justify-between items-center border-b border-outline-variant/15 pb-2">
                            <span class="text-[10px] jetbrains font-bold uppercase text-outline">"Lead Details"</span>
                        </div>

                        <Show
                            when=move || edit_mode.get()
                            fallback=move || view! {
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-mono bg-surface-container-lowest p-4 rounded-xl border border-outline-variant/10">
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"First Name"</span>
                                        <span class="text-on-surface font-semibold">{move || if first_name.get().is_empty() { "-".to_string() } else { first_name.get() }}</span>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"Last Name"</span>
                                        <span class="text-on-surface font-semibold">{move || if last_name.get().is_empty() { "-".to_string() } else { last_name.get() }}</span>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"Email"</span>
                                        <div class="flex items-center gap-2">
                                            <span class="text-on-surface font-semibold break-all">{move || if email.get().is_empty() { "-".to_string() } else { email.get() }}</span>
                                            <Show when=move || !email.get().is_empty()>
                                                <button 
                                                    on:click=move |_| set_composer_open.set(true)
                                                    class="text-primary hover:text-primary-container p-0.5 rounded transition-colors flex items-center justify-center"
                                                    title="Compose Email"
                                                >
                                                    <span class="material-symbols-outlined text-[14px]">"mail"</span>
                                                </button>
                                            </Show>
                                        </div>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"Phone"</span>
                                        <span class="text-on-surface font-semibold">{move || if phone.get().is_empty() { "-".to_string() } else { phone.get() }}</span>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"Company"</span>
                                        <span class="text-on-surface font-semibold">{move || if company.get().is_empty() { "-".to_string() } else { company.get() }}</span>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"Title"</span>
                                        <span class="text-on-surface font-semibold">{move || if title.get().is_empty() { "-".to_string() } else { title.get() }}</span>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"Source"</span>
                                        <span class="text-on-surface font-semibold">{move || if source.get().is_empty() { "-".to_string() } else { source.get() }}</span>
                                    </div>
                                    <div>
                                        <span class="text-outline-variant text-[10px] block uppercase">"Converted"</span>
                                        <span class="text-on-surface font-semibold">{move || if lead_record.is_converted { "Yes".to_string() } else { "No".to_string() }}</span>
                                    </div>
                                </div>
                            }
                        >
                            <div class="space-y-3 bg-surface-container-lowest p-4 rounded-xl border border-outline-variant/20">
                                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"First Name *"</label>
                                        <input 
                                            type="text" 
                                            prop:value=first_name
                                            on:input=move |ev| set_first_name.set(event_target_value(&ev))
                                            class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Last Name"</label>
                                        <input 
                                            type="text" 
                                            prop:value=last_name
                                            on:input=move |ev| set_last_name.set(event_target_value(&ev))
                                            class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Email"</label>
                                        <input 
                                            type="email" 
                                            prop:value=email
                                            on:input=move |ev| set_email.set(event_target_value(&ev))
                                            class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Phone"</label>
                                        <input 
                                            type="text" 
                                            prop:value=phone
                                            on:input=move |ev| set_phone.set(event_target_value(&ev))
                                            class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Company"</label>
                                        <input 
                                            type="text" 
                                            prop:value=company
                                            on:input=move |ev| set_company.set(event_target_value(&ev))
                                            class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Title"</label>
                                        <input 
                                            type="text" 
                                            prop:value=title
                                            on:input=move |ev| set_title.set(event_target_value(&ev))
                                            class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                        />
                                    </div>
                                    <div>
                                        <label class="block text-[10px] jetbrains uppercase text-outline mb-1">"Source"</label>
                                        <input 
                                            type="text" 
                                            prop:value=source
                                            on:input=move |ev| set_source.set(event_target_value(&ev))
                                            class="w-full bg-surface-container border border-outline-variant/30 px-3 py-1.5 text-xs text-on-surface focus:outline-none focus:border-primary rounded"
                                        />
                                    </div>
                                </div>
                                <Show when=move || save_error.get().is_some()>
                                    <div class="bg-error/10 border-l-4 border-error p-3 jetbrains text-xs text-error font-medium">
                                        {move || save_error.get().unwrap_or_default()}
                                    </div>
                                </Show>
                                <div class="flex justify-end">
                                    <button
                                        on:click=handle_save_details
                                        class="bg-primary text-on-primary px-4 py-2 text-xs jetbrains font-bold uppercase tracking-wider hover:bg-primary-container rounded-lg"
                                    >
                                        "Save Changes"
                                    </button>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>

                // RIGHT COLUMN (35% width) - Activity Feed & Timeline
                <div class="w-full lg:w-[35%] space-y-6">
                    <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs flex flex-col">
                        <label class="block text-[10px] font-bold uppercase text-outline-variant tracking-wider font-mono mb-4">"Timeline (Notes & Activities)"</label>
                        <CrmTimelineGeneric
                            notes=Signal::derive(move || notes_res.get().and_then(|r| r.ok()).unwrap_or_default())
                            activities=Signal::derive(move || activities_res.get().and_then(|r| r.ok()).unwrap_or_default())
                            on_add_note=add_note_cb
                            on_add_activity=log_activity_cb
                            on_update_activity_status=update_activity_status_cb
                            on_delete_note=delete_note_cb
                            on_delete_activity=delete_activity_cb
                        />
                    </div>
                    <FileAttachments
                        entity_type="Lead".to_string()
                        entity_id=lead_id
                        files=Signal::derive(move || attachments_res.get().and_then(|r| r.ok()).unwrap_or_default())
                        on_upload=add_attachment_cb
                        on_delete=delete_attachment_cb
                        on_download=download_attachment_cb
                    />
                </div>

            </div>

            <shared_ui::components::email_composer::EmailComposer
                open=composer_open
                to_email=email
                templates=default_templates.clone()
                record_files=Signal::derive(move || attachments_res.get().and_then(|r| r.ok()).unwrap_or_default())
                on_close=Callback::new(move |_: ()| set_composer_open.set(false))
                on_send=Callback::new({
                    let set_refresh = set_refresh.clone();
                    let refresh = refresh.clone();
                    let to_email = email.clone();
                    move |(subj, bdy, atts): (String, String, Vec<String>)| {
                        let set_refresh = set_refresh.clone();
                        let refresh = refresh.clone();
                        let to_addr = to_email.get();
                        leptos::task::spawn_local(async move {
                            if let Ok(_) = send_crm_email(to_addr, subj, bdy, None, Some(lead_id), atts).await {
                                set_composer_open.set(false);
                                set_refresh.set(refresh.get_untracked() + 1);
                            }
                        });
                    }
                })
            />
        </div>
    }
}
