use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[cfg(not(feature = "ssr"))]
use crate::components::file_attachments::upload_file_to_s3;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivityType {
    Log,
    Task,
    Event,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivityStatus {
    Open,
    Pending,
    Completed,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum AssociatedEntityType {
    Account,
    Customer,
    Lead,
    Deal,
    Case,
    Contact,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AssociatedEntity {
    pub entity_type: AssociatedEntityType,
    pub entity_id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FileModel {
    pub id: Uuid,
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub hash_sha256: String,
    pub storage_type: String,
    pub storage_path: String,
    pub views: i32,
    pub downloads: i32,
    pub bandwidth_used: i64,
    pub bandwidth_used_paid: i64,
    pub date_upload: String,
    pub date_last_view: Option<String>,
    pub is_anonymous: bool,
    pub user_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ActivityModel {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub account_id: Option<Uuid>,
    pub deal_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub case_id: Option<Uuid>,
    pub activity_type: ActivityType,
    pub title: String,
    pub description: Option<String>,
    pub status: ActivityStatus,
    pub due_date: Option<String>,
    pub completed_at: Option<String>,
    pub associated_entities: Vec<AssociatedEntity>,
    pub created_by: Uuid,
    pub assigned_to: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
    pub files: Vec<FileModel>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NoteModel {
    pub id: Uuid,
    pub content: String,
    pub created_by: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub is_private: bool,
    pub created_at: String,
    pub updated_at: String,
    pub files: Vec<FileModel>,
}

// Staged file structure for upload tracking
#[derive(Clone, Debug, PartialEq)]
pub struct StagedFile {
    pub id: Uuid,
    pub name: String,
    pub key: String,
    pub is_uploading: bool,
    pub error: Option<String>,
}

#[derive(Clone, PartialEq)]
pub enum TimelineFeedItem {
    Note(NoteModel),
    Activity(ActivityModel),
}

#[component]
pub fn CrmTimelineGeneric(
    notes: Signal<Vec<NoteModel>>,
    activities: Signal<Vec<ActivityModel>>,
    #[prop(into)] on_add_note: Callback<(String, bool, Vec<FileModel>)>, // (content, is_private, files)
    #[prop(into)] on_add_activity: Callback<(ActivityType, String, Option<String>, ActivityStatus, Option<String>, Option<String>, Vec<FileModel>)>, // (type, title, desc, status, due_date, completed_at, files)
    #[prop(into)] on_update_activity_status: Callback<(Uuid, ActivityStatus)>,
    #[prop(into)] on_delete_note: Callback<Uuid>,
    #[prop(into)] on_delete_activity: Callback<Uuid>,
) -> impl IntoView {
    // Current Active Tab: "note" | "log" | "task" | "event"
    let (active_tab, set_active_tab) = signal("note".to_string());
    
    // Notes Staged Fields
    let (note_text, set_note_text) = signal(String::new());
    let (note_private, set_note_private) = signal(false);

    // Logs Staged Fields
    let (log_title, set_log_title) = signal("Phone Call".to_string());
    let (log_desc, set_log_desc) = signal(String::new());
    let (log_date, set_log_date) = signal(chrono::Local::now().format("%Y-%m-%dT%H:%M").to_string());

    // Tasks Staged Fields
    let (task_title, set_task_title) = signal(String::new());
    let (task_desc, set_task_desc) = signal(String::new());
    let (task_due, set_task_due) = signal(String::new());

    // Events Staged Fields
    let (event_title, set_event_title) = signal(String::new());
    let (event_desc, set_event_desc) = signal(String::new());
    let (event_start, set_event_start) = signal(String::new());
    let (event_end, set_event_end) = signal(String::new());

    // Staged Files Dropzone State
    let (staged_files, set_staged_files) = signal::<Vec<StagedFile>>(Vec::new());
    let (is_dragging, set_dragging) = signal(false);
    let file_input_ref = NodeRef::<leptos::html::Input>::new();

    // Helper to turn SagedFiles into FileModels
    let get_file_models = move || -> Vec<FileModel> {
        staged_files.get().into_iter()
            .filter(|f| !f.is_uploading && f.error.is_none() && !f.key.is_empty())
            .map(|f| FileModel {
                id: f.id,
                name: f.name.clone(),
                size: 0,
                mime_type: "application/octet-stream".to_string(),
                hash_sha256: "".to_string(),
                storage_type: "S3".to_string(),
                storage_path: f.key.clone(),
                views: 0,
                downloads: 0,
                bandwidth_used: 0,
                bandwidth_used_paid: 0,
                date_upload: chrono::Utc::now().to_rfc3339(),
                date_last_view: None,
                is_anonymous: false,
                user_id: None,
            })
            .collect()
    };

    let handle_file_upload = move |file: web_sys::File| {
        let file_id = Uuid::new_v4();
        let filename = file.name();
        
        let new_staged = StagedFile {
            id: file_id,
            name: filename.clone(),
            key: String::new(),
            is_uploading: true,
            error: None,
        };
        
        set_staged_files.update(|v| v.push(new_staged));
        
        leptos::task::spawn_local(async move {
            #[cfg(not(feature = "ssr"))]
            {
                match upload_file_to_s3(file).await {
                    Ok((_, key)) => {
                        set_staged_files.update(|v| {
                            if let Some(f) = v.iter_mut().find(|x| x.id == file_id) {
                                f.is_uploading = false;
                                f.key = key;
                            }
                        });
                    }
                    Err(err) => {
                        set_staged_files.update(|v| {
                            if let Some(f) = v.iter_mut().find(|x| x.id == file_id) {
                                f.is_uploading = false;
                                f.error = Some(err);
                            }
                        });
                    }
                }
            }
            #[cfg(feature = "ssr")]
            {
                let _ = file_id;
            }
        });
    };

    let on_file_change = move |ev: web_sys::Event| {
        let target: web_sys::HtmlInputElement = event_target(&ev);
        let files = target.files();
            
        if let Some(file_list) = files {
            let len = file_list.length();
            for i in 0..len {
                if let Some(file) = file_list.item(i) {
                    handle_file_upload(file);
                }
            }
        }
    };

    let on_drag_over = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        set_dragging.set(true);
    };

    let on_drag_leave = move |_| {
        set_dragging.set(false);
    };

    let on_drop = move |ev: web_sys::DragEvent| {
        ev.prevent_default();
        set_dragging.set(false);
        let data_transfer = ev.data_transfer();
        if let Some(dt) = data_transfer {
            let files = dt.files();
            if let Some(file_list) = files {
                let len = file_list.length();
                for i in 0..len {
                    if let Some(file) = file_list.item(i) {
                        handle_file_upload(file);
                    }
                }
            }
        }
    };

    let remove_staged = move |id: Uuid| {
        set_staged_files.update(|v| v.retain(|x| x.id != id));
    };

    let do_submit_note = move || {
        let text = note_text.get();
        if !text.trim().is_empty() {
            on_add_note.run((text, note_private.get(), get_file_models()));
            set_note_text.set(String::new());
            set_staged_files.set(Vec::new());
        }
    };

    let submit_note = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        do_submit_note();
    };

    let do_submit_log = move || {
        let desc = log_desc.get();
        if !desc.trim().is_empty() {
            let title_str = format!("Logged: {}", log_title.get());
            let completed_at = Some(chrono::DateTime::parse_from_rfc3339(&format!("{}:00Z", log_date.get()))
                .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339())
                .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339()));
            
            on_add_activity.run((
                ActivityType::Log,
                title_str,
                Some(desc),
                ActivityStatus::Completed,
                None,
                completed_at,
                get_file_models()
            ));
            set_log_desc.set(String::new());
            set_staged_files.set(Vec::new());
        }
    };

    let submit_log = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        do_submit_log();
    };

    let do_submit_task = move || {
        let title = task_title.get();
        let due = task_due.get();
        if !title.trim().is_empty() && !due.is_empty() {
            let due_date = Some(chrono::DateTime::parse_from_rfc3339(&format!("{}:00Z", due))
                .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339())
                .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339()));

            on_add_activity.run((
                ActivityType::Task,
                title,
                Some(task_desc.get()),
                ActivityStatus::Open,
                due_date,
                None,
                get_file_models()
            ));
            set_task_title.set(String::new());
            set_task_desc.set(String::new());
            set_task_due.set(String::new());
            set_staged_files.set(Vec::new());
        }
    };

    let submit_task = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        do_submit_task();
    };

    let do_submit_event = move || {
        let title = event_title.get();
        let start = event_start.get();
        let end = event_end.get();
        if !title.trim().is_empty() && !start.is_empty() && !end.is_empty() {
            let start_date = Some(chrono::DateTime::parse_from_rfc3339(&format!("{}:00Z", start))
                .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339())
                .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339()));

            let end_date = Some(chrono::DateTime::parse_from_rfc3339(&format!("{}:00Z", end))
                .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339())
                .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339()));

            on_add_activity.run((
                ActivityType::Event,
                title,
                Some(event_desc.get()),
                ActivityStatus::Open,
                start_date,
                end_date,
                get_file_models()
            ));
            set_event_title.set(String::new());
            set_event_desc.set(String::new());
            set_event_start.set(String::new());
            set_event_end.set(String::new());
            set_staged_files.set(Vec::new());
        }
    };

    let submit_event = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        do_submit_event();
    };

    // Sort Combined Timeline descending chronologically (completed_at -> due_date -> created_at)
    let combined_feed = move || {
        let mut feed = Vec::new();
        for n in notes.get() {
            feed.push((n.created_at.clone(), TimelineFeedItem::Note(n)));
        }
        for a in activities.get() {
            let primary_time = match a.activity_type {
                ActivityType::Log => a.completed_at.clone().unwrap_or_else(|| a.created_at.clone()),
                ActivityType::Task => a.due_date.clone().unwrap_or_else(|| a.created_at.clone()),
                ActivityType::Event => a.due_date.clone().unwrap_or_else(|| a.created_at.clone()),
            };
            feed.push((primary_time, TimelineFeedItem::Activity(a)));
        }
        feed.sort_by(|a, b| b.0.cmp(&a.0));
        feed.into_iter().map(|(_, item)| item).collect::<Vec<_>>()
    };

    let get_activity_icon = |act_type: &ActivityType, title: &str| -> &'static str {
        match act_type {
            ActivityType::Log => {
                let t = title.to_lowercase();
                if t.contains("call") { "call" }
                else if t.contains("meeting") { "calendar_today" }
                else if t.contains("email") { "mail" }
                else { "chat_bubble" }
            }
            ActivityType::Task => "task_alt",
            ActivityType::Event => "celebration",
        }
    };

    let get_activity_icon_color = |act_type: &ActivityType, title: &str| -> &'static str {
        match act_type {
            ActivityType::Log => {
                let t = title.to_lowercase();
                if t.contains("call") { "bg-blue-500/10 text-blue-500 border-blue-500/30" }
                else if t.contains("meeting") { "bg-indigo-500/10 text-indigo-500 border-indigo-500/30" }
                else if t.contains("email") { "bg-purple-500/10 text-purple-500 border-purple-500/30" }
                else { "bg-slate-500/10 text-slate-500 border-slate-500/30" }
            }
            ActivityType::Task => "bg-orange-500/10 text-orange-500 border-orange-500/30",
            ActivityType::Event => "bg-emerald-500/10 text-emerald-500 border-emerald-500/30",
        }
    };

    let format_date = |iso_str: &str| -> String {
        if iso_str.len() >= 16 {
            let replaced = iso_str.replace("T", " ");
            replaced[..16].to_string()
        } else {
            iso_str.to_string()
        }
    };

    view! {
        <div class="space-y-6">
            // Tab Selector
            <div class="border-b border-outline-variant/30 flex gap-4 overflow-x-auto pb-1">
                <button
                    on:click=move |_| { set_active_tab.set("note".to_string()); set_staged_files.set(Vec::new()); }
                    class={move || format!(
                        "pb-2 text-xs jetbrains font-bold uppercase tracking-wider transition-colors border-b-2 whitespace-nowrap {}",
                        if active_tab.get() == "note" { "border-primary text-primary" } else { "border-transparent text-outline hover:text-on-surface" }
                    )}
                >
                    "Add Note"
                </button>
                <button
                    on:click=move |_| { set_active_tab.set("log".to_string()); set_staged_files.set(Vec::new()); }
                    class={move || format!(
                        "pb-2 text-xs jetbrains font-bold uppercase tracking-wider transition-colors border-b-2 whitespace-nowrap {}",
                        if active_tab.get() == "log" { "border-primary text-primary" } else { "border-transparent text-outline hover:text-on-surface" }
                    )}
                >
                    "Log Call/Call log"
                </button>
                <button
                    on:click=move |_| { set_active_tab.set("task".to_string()); set_staged_files.set(Vec::new()); }
                    class={move || format!(
                        "pb-2 text-xs jetbrains font-bold uppercase tracking-wider transition-colors border-b-2 whitespace-nowrap {}",
                        if active_tab.get() == "task" { "border-primary text-primary" } else { "border-transparent text-outline hover:text-on-surface" }
                    )}
                >
                    "Create Task"
                </button>
                <button
                    on:click=move |_| { set_active_tab.set("event".to_string()); set_staged_files.set(Vec::new()); }
                    class={move || format!(
                        "pb-2 text-xs jetbrains font-bold uppercase tracking-wider transition-colors border-b-2 whitespace-nowrap {}",
                        if active_tab.get() == "event" { "border-primary text-primary" } else { "border-transparent text-outline hover:text-on-surface" }
                    )}
                >
                    "Schedule Event"
                </button>
            </div>

            // Unified Publisher Composer
            <div class="bg-surface-container p-4 rounded-xl border border-outline-variant/20 shadow-inner">
                <Show when=move || active_tab.get() == "note">
                    <form on:submit=submit_note class="space-y-4">
                        <div class="flex justify-between items-center">
                            <label class="block text-[10px] jetbrains uppercase text-outline">"Note Content"</label>
                            
                            // Privacy Toggle lock icon
                            <button
                                type="button"
                                on:click=move |_| set_note_private.update(|v| *v = !*v)
                                class={move || format!(
                                    "flex items-center gap-1.5 px-2 py-1 rounded text-xs font-semibold jetbrains transition-colors {}",
                                    if note_private.get() { "bg-error/10 text-error hover:bg-error/20" } else { "bg-secondary-container-low text-outline hover:text-on-surface" }
                                )}
                            >
                                <span class="material-symbols-outlined text-sm">
                                    {move || if note_private.get() { "lock" } else { "lock_open" }}
                                </span>
                                {move || if note_private.get() { "Private" } else { "Public" }}
                            </button>
                        </div>
                        <textarea
                            prop:value=note_text
                            on:input=move |ev| set_note_text.set(event_target_value(&ev))
                            placeholder="Type a note (Markdown supported)..."
                            rows="3"
                            class="w-full bg-surface-container-low border border-outline-variant/30 p-3 text-sm focus:outline-none focus:border-primary text-on-surface resize-none rounded-lg"
                        ></textarea>
                    </form>
                </Show>

                <Show when=move || active_tab.get() == "log">
                    <form on:submit=submit_log class="space-y-4">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Log Action"</label>
                                <select
                                    prop:value=log_title
                                    on:change=move |ev| set_log_title.set(event_target_value(&ev))
                                    class="bg-surface-container-low border border-outline-variant/30 px-3 py-2 text-xs font-semibold rounded w-full text-on-surface focus:outline-none focus:border-primary"
                                >
                                    <option value="Phone Call">"Phone Call"</option>
                                    <option value="Meeting Held">"Meeting Held"</option>
                                    <option value="Email Sent">"Email Sent"</option>
                                    <option value="Meeting Notes">"Meeting Notes"</option>
                                </select>
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Date/Time (Retroactive)"</label>
                                <input
                                    type="datetime-local"
                                    prop:value=log_date
                                    on:input=move |ev| set_log_date.set(event_target_value(&ev))
                                    class="bg-surface-container-low border border-outline-variant/30 px-3 py-1.5 text-xs font-semibold rounded w-full text-on-surface focus:outline-none focus:border-primary"
                                />
                            </div>
                        </div>
                        <div>
                            <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Description"</label>
                            <textarea
                                prop:value=log_desc
                                on:input=move |ev| set_log_desc.set(event_target_value(&ev))
                                placeholder="Describe the outcome..."
                                rows="3"
                                class="w-full bg-surface-container-low border border-outline-variant/30 p-3 text-sm focus:outline-none focus:border-primary text-on-surface resize-none rounded-lg"
                            ></textarea>
                        </div>
                    </form>
                </Show>

                <Show when=move || active_tab.get() == "task">
                    <form on:submit=submit_task class="space-y-4">
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Task Title"</label>
                                <input
                                    type="text"
                                    prop:value=task_title
                                    on:input=move |ev| set_task_title.set(event_target_value(&ev))
                                    placeholder="Follow-up call, schedule demo..."
                                    class="bg-surface-container-low border border-outline-variant/30 px-3 py-1.5 text-xs font-semibold rounded w-full text-on-surface focus:outline-none focus:border-primary"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Due Date/Time"</label>
                                <input
                                    type="datetime-local"
                                    prop:value=task_due
                                    on:input=move |ev| set_task_due.set(event_target_value(&ev))
                                    class="bg-surface-container-low border border-outline-variant/30 px-3 py-1.5 text-xs font-semibold rounded w-full text-on-surface focus:outline-none focus:border-primary"
                                />
                            </div>
                        </div>
                        <div>
                            <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Description"</label>
                            <textarea
                                prop:value=task_desc
                                on:input=move |ev| set_task_desc.set(event_target_value(&ev))
                                placeholder="e.g. Discuss the proposal contract edits..."
                                rows="3"
                                class="w-full bg-surface-container-low border border-outline-variant/30 p-3 text-sm focus:outline-none focus:border-primary text-on-surface resize-none rounded-lg"
                            ></textarea>
                        </div>
                    </form>
                </Show>

                <Show when=move || active_tab.get() == "event">
                    <form on:submit=submit_event class="space-y-4">
                        <div>
                            <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Event Title"</label>
                            <input
                                type="text"
                                prop:value=event_title
                                on:input=move |ev| set_event_title.set(event_target_value(&ev))
                                placeholder="Kickoff call, design alignment..."
                                class="bg-surface-container-low border border-outline-variant/30 px-3 py-1.5 text-xs font-semibold rounded w-full text-on-surface focus:outline-none focus:border-primary"
                            />
                        </div>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Start Date/Time"</label>
                                <input
                                    type="datetime-local"
                                    prop:value=event_start
                                    on:input=move |ev| set_event_start.set(event_target_value(&ev))
                                    class="bg-surface-container-low border border-outline-variant/30 px-3 py-1.5 text-xs font-semibold rounded w-full text-on-surface focus:outline-none focus:border-primary"
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"End Date/Time"</label>
                                <input
                                    type="datetime-local"
                                    prop:value=event_end
                                    on:input=move |ev| set_event_end.set(event_target_value(&ev))
                                    class="bg-surface-container-low border border-outline-variant/30 px-3 py-1.5 text-xs font-semibold rounded w-full text-on-surface focus:outline-none focus:border-primary"
                                />
                            </div>
                        </div>
                        <div>
                            <label class="block text-[10px] jetbrains uppercase text-outline mb-1.5">"Description"</label>
                            <textarea
                                prop:value=event_desc
                                on:input=move |ev| set_event_desc.set(event_target_value(&ev))
                                placeholder="e.g. High-level roadmap review..."
                                rows="3"
                                class="w-full bg-surface-container-low border border-outline-variant/30 p-3 text-sm focus:outline-none focus:border-primary text-on-surface resize-none rounded-lg"
                            ></textarea>
                        </div>
                    </form>
                </Show>

                // Upload Dropzone Widget
                <div class="mt-4">
                    <input
                        type="file"
                        node_ref=file_input_ref
                        on:change=on_file_change
                        multiple=true
                        class="hidden"
                    />
                    <div
                        on:dragover=on_drag_over
                        on:dragleave=on_drag_leave
                        on:drop=on_drop
                        on:click=move |_| {
                            if let Some(input) = file_input_ref.get() {
                                input.click();
                            }
                        }
                        class={move || format!(
                            "border-2 border-dashed rounded-lg p-4 text-center cursor-pointer transition-all hover:bg-surface-container-low/40 {}",
                            if is_dragging.get() { "border-primary bg-primary/5" } else { "border-outline-variant/40" }
                        )}
                    >
                        <div class="flex flex-col items-center gap-1.5 text-outline">
                           <span class="material-symbols-outlined text-2xl">"cloud_upload"</span>
                            <span class="text-xs font-semibold">"Drag & Drop or Click to Attach Files"</span>
                            <span class="text-[9px] jetbrains">"PDFs, images, docs supported"</span>
                        </div>
                    </div>

                    // Staged files list with spinners/thumbnails
                    <Show when=move || !staged_files.get().is_empty()>
                        <div class="mt-3 space-y-2">
                            <For
                                each=staged_files
                                key=|f| f.id
                                children=move |f| {
                                    let fid = f.id;
                                    let name = f.name.clone();
                                    let is_uploading = f.is_uploading;
                                    let error = f.error.clone();
                                    view! {
                                        <div class="flex items-center justify-between bg-surface-container-low border border-outline-variant/20 px-3 py-2 rounded-lg text-xs">
                                            <div class="flex items-center gap-2 max-w-[80%] overflow-hidden">
                                                <span class="material-symbols-outlined text-sm text-outline">"insert_drive_file"</span>
                                                <span class="truncate font-semibold text-on-surface">{name}</span>
                                                <Show when=move || is_uploading>
                                                    <span class="text-[9px] text-primary animate-pulse font-semibold jetbrains">"(uploading...)"</span>
                                                </Show>
                                                <Show when={
                                                    let error = error.clone();
                                                    move || error.is_some()
                                                }>
                                                    <span class="text-[9px] text-error font-semibold jetbrains">
                                                        {
                                                            let error = error.clone();
                                                            move || format!("(Error: {})", error.clone().unwrap_or_default())
                                                        }
                                                    </span>
                                                </Show>
                                            </div>
                                            <button
                                                type="button"
                                                on:click=move |_| remove_staged(fid)
                                                class="text-outline hover:text-error transition-colors"
                                            >
                                                <span class="material-symbols-outlined text-sm">"close"</span>
                                            </button>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </Show>
                </div>

                // Submit Button
                <div class="mt-4 flex justify-end">
                    <button
                        type="button"
                        on:click=move |_| {
                            // Submit the corresponding active form
                            let tab = active_tab.get();
                            if tab == "note" {
                                do_submit_note();
                            } else if tab == "log" {
                                do_submit_log();
                            } else if tab == "task" {
                                do_submit_task();
                            } else if tab == "event" {
                                do_submit_event();
                            }
                        }
                        class="bg-primary text-on-primary px-5 py-2 text-xs jetbrains font-bold uppercase tracking-wider hover:bg-primary-container transition-colors rounded-lg shadow-sm"
                    >
                        {move || match active_tab.get().as_str() {
                            "note" => "Save Note",
                            "log" => "Log Activity",
                            "task" => "Create Task",
                            "event" => "Schedule Event",
                            _ => "Submit",
                        }}
                    </button>
                </div>
            </div>

            // Timeline chronological Feed
            <div class="relative pl-6 border-l-2 border-outline-variant/20 ml-3 space-y-6 pt-2">
                <For
                    each=combined_feed
                    key=|item| match item {
                        TimelineFeedItem::Note(n) => format!("note-{}", n.id),
                        TimelineFeedItem::Activity(a) => format!("act-{}", a.id),
                    }
                    children=move |item| {
                        match item {
                            TimelineFeedItem::Note(note) => {
                                let note_id = note.id;
                                let is_private = note.is_private;
                                let created_at = note.created_at.clone();
                                let content = note.content.clone();
                                let files = note.files.clone();
                                
                                let files_for_show = files.clone();
                                let files_for_map = files.clone();
                                
                                view! {
                                    <div class="relative group">
                                        // Bullet circle icon
                                        <div class="absolute -left-[35px] top-1.5 w-6 h-6 rounded-full flex items-center justify-center border bg-surface-container-lowest text-primary border-outline-variant/40">
                                            <span class="material-symbols-outlined text-sm">
                                                {if is_private { "lock" } else { "description" }}
                                            </span>
                                        </div>
                                        // Box content
                                        <div class="bg-surface-container-lowest border border-outline-variant/25 p-4 rounded-xl shadow-xs hover:shadow-md transition-shadow relative">
                                            <div class="flex justify-between items-center mb-2">
                                                <div class="flex items-center gap-1.5">
                                                    <span class="text-xs font-bold text-primary jetbrains uppercase tracking-wide">"NOTE"</span>
                                                    <Show when=move || is_private>
                                                        <span class="bg-error/10 text-error border border-error/20 text-[9px] px-1.5 py-0.5 rounded font-bold jetbrains uppercase">"PRIVATE"</span>
                                                    </Show>
                                                </div>
                                                <div class="flex items-center gap-2">
                                                    <span class="text-[10px] text-outline font-semibold jetbrains">{format_date(&created_at)}</span>
                                                    <button
                                                        on:click=move |_| on_delete_note.run(note_id)
                                                        class="opacity-0 group-hover:opacity-100 text-outline hover:text-error transition-all"
                                                    >
                                                        <span class="material-symbols-outlined text-xs">"delete"</span>
                                                    </button>
                                                </div>
                                            </div>
                                            <p class="text-sm text-on-surface whitespace-pre-wrap leading-relaxed">{content.clone()}</p>
                                            
                                            // Associated Files list
                                            <Show when=move || !files_for_show.is_empty()>
                                                <div class="mt-3 flex flex-wrap gap-2 border-t border-outline-variant/20 pt-2">
                                                    {
                                                        let files = files_for_map.clone();
                                                        files.iter().map(|f| {
                                                            let storage_path = f.storage_path.clone();
                                                            let name = f.name.clone();
                                                            view! {
                                                                <a
                                                                    href=storage_path
                                                                    target="_blank"
                                                                    class="flex items-center gap-1 px-2 py-1 rounded bg-surface-container border border-outline-variant/25 text-[10px] font-semibold text-outline hover:text-on-surface transition-colors"
                                                                >
                                                                    <span class="material-symbols-outlined text-xs">"attachment"</span>
                                                                    <span>{name}</span>
                                                                </a>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }
                                                </div>
                                            </Show>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            TimelineFeedItem::Activity(act) => {
                                let act_id = act.id;
                                let act_type = act.activity_type.clone();
                                let status = act.status.clone();
                                let title = act.title.clone();
                                let description = act.description.clone();
                                let completed_at = act.completed_at.clone();
                                let due_date = act.due_date.clone();
                                let created_at = act.created_at.clone();
                                let files = act.files.clone();
                                
                                let is_completed = status == ActivityStatus::Completed;
                                let icon = get_activity_icon(&act_type, &title);
                                let icon_color = get_activity_icon_color(&act_type, &title);
                                let label = match act_type {
                                    ActivityType::Log => "LOGGED CALL".to_string(),
                                    ActivityType::Task => "TASK".to_string(),
                                    ActivityType::Event => "EVENT".to_string(),
                                };
                                
                                let display_time = match act_type {
                                    ActivityType::Log => completed_at.clone().unwrap_or_else(|| created_at.clone()),
                                    ActivityType::Task => due_date.clone().unwrap_or_else(|| created_at.clone()),
                                    ActivityType::Event => due_date.clone().unwrap_or_else(|| created_at.clone()),
                                };

                                let act_type_for_show = act_type.clone();
                                let act_type_for_event = act_type.clone();
                                let status_for_click = status.clone();
                                let status_for_class = status.clone();
                                let status_for_format = status.clone();
                                let description_for_show = description.clone();
                                let description_for_content = description.clone();
                                let completed_at_for_show = completed_at.clone();
                                let completed_at_for_event = completed_at.clone();
                                let due_date_for_event = due_date.clone();
                                let files_for_show = files.clone();
                                let files_for_map = files.clone();

                                view! {
                                    <div class="relative group">
                                        // Bullet circle icon
                                        <div class={format!("absolute -left-[35px] top-1.5 w-6 h-6 rounded-full flex items-center justify-center border {}", icon_color)}>
                                            <span class="material-symbols-outlined text-sm">{icon}</span>
                                        </div>
                                        // Box content
                                        <div class="bg-surface-container-lowest border border-outline-variant/25 p-4 rounded-xl shadow-xs hover:shadow-md transition-shadow relative">
                                            <div class="flex justify-between items-center mb-2">
                                                <div class="flex items-center gap-2">
                                                    <span class="text-xs font-bold text-secondary-container text-secondary jetbrains uppercase tracking-wide">
                                                        {label}
                                                    </span>
                                                    
                                                    // Task completion check checkbox
                                                    <Show when={
                                                        let act_type = act_type_for_show.clone();
                                                        move || act_type == ActivityType::Task
                                                    }>
                                                        <button
                                                            on:click={
                                                                let status = status_for_click.clone();
                                                                move |_| {
                                                                    let next_status = if status == ActivityStatus::Completed { ActivityStatus::Open } else { ActivityStatus::Completed };
                                                                    on_update_activity_status.run((act_id, next_status));
                                                                }
                                                            }
                                                            class={move || format!(
                                                                "flex items-center justify-center w-4 h-4 rounded border transition-colors {}",
                                                                if is_completed { "bg-emerald-500 border-emerald-500 text-on-emerald" } else { "border-outline-variant hover:border-primary" }
                                                            )}
                                                        >
                                                            <Show when=move || is_completed>
                                                                <span class="material-symbols-outlined text-[10px] font-bold">"check"</span>
                                                            </Show>
                                                        </button>
                                                    </Show>

                                                    <span class={
                                                        let status = status_for_class.clone();
                                                        move || format!(
                                                            "text-[9px] px-1.5 py-0.5 rounded font-bold jetbrains uppercase border {}",
                                                            match status {
                                                                ActivityStatus::Completed => "bg-emerald-500/10 text-emerald-500 border-emerald-500/20",
                                                                ActivityStatus::Open => "bg-blue-500/10 text-blue-500 border-blue-500/20",
                                                                ActivityStatus::Pending => "bg-amber-500/10 text-amber-500 border-amber-500/20",
                                                            }
                                                        )
                                                    }>
                                                        {
                                                            let status = status_for_format.clone();
                                                            move || format!("{:?}", status)
                                                        }
                                                    </span>
                                                </div>
                                                <div class="flex items-center gap-2">
                                                    <span class="text-[10px] text-outline font-semibold jetbrains">{format_date(&display_time)}</span>
                                                    <button
                                                        on:click=move |_| on_delete_activity.run(act_id)
                                                        class="opacity-0 group-hover:opacity-100 text-outline hover:text-error transition-all"
                                                    >
                                                        <span class="material-symbols-outlined text-xs">"delete"</span>
                                                    </button>
                                                </div>
                                            </div>

                                            <h4 class={move || format!("text-sm font-bold text-on-surface {}", if is_completed { "line-through text-outline" } else { "" })}>
                                                {title.clone()}
                                            </h4>

                                            <Show when={
                                                let description = description_for_show.clone();
                                                move || description.is_some()
                                            }>
                                                <p class="text-xs text-outline mt-1 leading-relaxed">
                                                    {
                                                        let description = description_for_content.clone();
                                                        move || description.clone().unwrap_or_default()
                                                    }
                                                </p>
                                            </Show>

                                            // Event bounding display
                                            <Show when={
                                                let act_type = act_type_for_event.clone();
                                                let completed_at = completed_at_for_show.clone();
                                                move || act_type == ActivityType::Event && completed_at.is_some()
                                            }>
                                                <div class="mt-2 text-[10px] font-semibold text-outline jetbrains">
                                                    {
                                                        let due_date = due_date_for_event.clone();
                                                        let completed_at = completed_at_for_event.clone();
                                                        move || format!("Event duration: {} to {}", format_date(&due_date.clone().unwrap_or_default()), format_date(&completed_at.clone().unwrap_or_default()))
                                                    }
                                                </div>
                                            </Show>

                                            // Associated Files list
                                            <Show when={
                                                let files = files_for_show.clone();
                                                move || !files.is_empty()
                                            }>
                                                <div class="mt-3 flex flex-wrap gap-2 border-t border-outline-variant/20 pt-2">
                                                    {
                                                        let files = files_for_map.clone();
                                                        files.iter().map(|f| {
                                                            let storage_path = f.storage_path.clone();
                                                            let name = f.name.clone();
                                                            view! {
                                                                <a
                                                                    href=storage_path
                                                                    target="_blank"
                                                                    class="flex items-center gap-1 px-2 py-1 rounded bg-surface-container border border-outline-variant/25 text-[10px] font-semibold text-outline hover:text-on-surface transition-colors"
                                                                >
                                                                    <span class="material-symbols-outlined text-xs">"attachment"</span>
                                                                    <span>{name}</span>
                                                                </a>
                                                            }
                                                        }).collect::<Vec<_>>()
                                                    }
                                                </div>
                                            </Show>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }
                    }
                />

                <Show when=move || combined_feed().is_empty()>
                    <div class="text-center py-8 text-outline text-xs jetbrains">
                        "NO_TIMELINE_ENTRIES_YET"
                    </div>
                </Show>
            </div>
        </div>
    }
}
