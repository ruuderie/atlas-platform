use leptos::prelude::*;
use wasm_bindgen::JsCast;
use super::file_attachments::RecordDocumentModel;

#[derive(Clone, Debug, PartialEq)]
pub struct EmailTemplate {
    pub name: String,
    pub subject: String,
    pub body: String,
}

#[component]
pub fn EmailComposer(
    open: ReadSignal<bool>,
    to_email: ReadSignal<String>,
    #[prop(optional)] templates: Vec<EmailTemplate>,
    #[prop(into, optional)] record_files: Option<Signal<Vec<RecordDocumentModel>>>,
    on_close: Callback<()>,
    on_send: Callback<(String, String, Vec<String>)>, // (subject, body, attachments)
) -> impl IntoView {
    let (subject, set_subject) = signal(String::new());
    let (body, set_body) = signal(String::new());
    let (sending, set_sending) = signal(false);
    let (uploading_attachment, set_uploading_attachment) = signal(false);
    
    // Tracks both pre-existing record file keys and newly uploaded local S3 file keys
    let (selected_attachments, set_selected_attachments) = signal::<Vec<String>>(Vec::new());
    let (custom_attachments, set_custom_attachments) = signal::<Vec<(String, String)>>(Vec::new()); // (file_name, file_key)

    // Hidden input reference for local upload
    let local_file_input_ref = NodeRef::<leptos::html::Input>::new();

    // Keep inputs in sync when composer opens/closes
    Effect::new(move |_| {
        if open.get() {
            set_subject.set(String::new());
            set_body.set(String::new());
            set_sending.set(false);
            set_selected_attachments.set(Vec::new());
            set_custom_attachments.set(Vec::new());
        }
    });

    let templates_stored = StoredValue::new(templates.clone());
    let handle_template_select = move |ev: web_sys::Event| {
        let val = event_target_value(&ev);
        templates_stored.with_value(|tpls| {
            if let Some(tpl) = tpls.iter().find(|t| t.name == val) {
                set_subject.set(tpl.subject.clone());
                set_body.set(tpl.body.clone());
            }
        });
    };

    let handle_send = {
        let on_send = on_send.clone();
        move |_| {
            if subject.get_untracked().is_empty() || body.get_untracked().is_empty() {
                return;
            }
            set_sending.set(true);
            
            let attachments = selected_attachments.get_untracked();
            on_send.run((subject.get_untracked(), body.get_untracked(), attachments));
        }
    };

    let toggle_attachment = move |file_key: String| {
        set_selected_attachments.update(|list| {
            if let Some(pos) = list.iter().position(|k| k == &file_key) {
                list.remove(pos);
            } else {
                list.push(file_key);
            }
        });
    };

    let trigger_local_upload = move |_| {
        if let Some(input) = local_file_input_ref.get() {
            input.click();
        }
    };

    let handle_local_upload = move |ev: web_sys::Event| {
        let target = ev.target().and_then(|t| wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlInputElement>(t).ok());
        if let Some(input) = target {
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    set_uploading_attachment.set(true);
                    leptos::task::spawn_local(async move {
                        #[cfg(not(feature = "ssr"))]
                        {
                            match super::file_attachments::upload_file_to_s3(file).await {
                                Ok((name, key)) => {
                                    set_custom_attachments.update(|list| list.push((name.clone(), key.clone())));
                                    set_selected_attachments.update(|list| list.push(key));
                                }
                                Err(e) => {
                                    leptos::logging::error!("Composer upload error: {}", e);
                                }
                            }
                        }
                        set_uploading_attachment.set(false);
                    });
                }
            }
        }
    };

    // Rich Text helper functions to insert HTML formatting tags into textarea
    let insert_formatting = move |tag_open: &str, tag_close: &str| {
        #[cfg(not(feature = "ssr"))]
        if let Some(win) = web_sys::window() {
            if let Some(doc) = win.document() {
                if let Some(el) = doc.get_element_by_id("email-body-textarea") {
                    if let Ok(textarea) = el.dyn_into::<web_sys::HtmlTextAreaElement>() {
                        let start = textarea.selection_start().ok().flatten().unwrap_or(0);
                        let end = textarea.selection_end().ok().flatten().unwrap_or(0);
                        let current_val = textarea.value();
                        
                        let before = &current_val[..start as usize];
                        let selected = &current_val[start as usize..end as usize];
                        let after = &current_val[end as usize..];
                        
                        let new_val = format!("{}{}{}{}{}", before, tag_open, selected, tag_close, after);
                        textarea.set_value(&new_val);
                        set_body.set(new_val);
                        let _ = textarea.focus();
                    }
                }
            }
        }
    };

    let files_signal = record_files.unwrap_or_else(|| Signal::derive(|| Vec::new()));

    view! {
        <div 
            class="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-[90] transition-opacity duration-300"
            style:display=move || if open.get() { "flex" } else { "none" }
        >
            <div class="bg-surface-container border border-outline-variant/30 rounded-2xl w-full max-w-lg shadow-2xl p-6 relative flex flex-col font-sans text-on-surface animate-slide-in max-h-[90vh] overflow-y-auto">
                
                // Header
                <div class="flex items-center justify-between border-b border-outline-variant/30 pb-4 mb-4">
                    <div>
                        <h3 class="text-lg font-bold text-primary">"Compose Email"</h3>
                        <p class="text-xs text-outline-variant mt-0.5">"Headless CRM Email Dispatcher"</p>
                    </div>
                    <button 
                        on:click=move |_| on_close.run(()) 
                        class="p-1.5 hover:bg-surface-container-high rounded-full text-outline-variant hover:text-on-surface transition-colors"
                    >
                        <span class="material-symbols-outlined text-[18px]">"close"</span>
                    </button>
                </div>

                // Form fields
                <div class="space-y-4 flex-1">
                    
                    // Template Dropdown (if templates are supplied)
                    <Show when=move || !templates_stored.with_value(|t| t.is_empty())>
                        <div>
                            <label class="block text-[10px] tracking-wider uppercase font-semibold text-outline-variant mb-1">
                                "Template Injector"
                            </label>
                            <select 
                                on:change=handle_template_select
                                class="w-full bg-surface-container border border-outline-variant/30 rounded-lg px-3 py-2 text-xs text-on-surface focus:outline-none focus:border-primary transition-colors cursor-pointer"
                            >
                                <option value="">"-- Select a template to merge --"</option>
                                {templates_stored.with_value(|t| {
                                    t.iter().map(|tpl| view! {
                                        <option value=tpl.name.clone()>{tpl.name.clone()}</option>
                                    }).collect::<Vec<_>>()
                                })}
                            </select>
                        </div>
                    </Show>

                    // Recipient (read-only info)
                    <div>
                        <label class="block text-[10px] tracking-wider uppercase font-semibold text-outline-variant mb-1">
                            "Recipient Address"
                        </label>
                        <div class="bg-surface-container-lowest border border-outline-variant/20 rounded-lg px-3 py-2 text-xs font-mono text-outline">
                            {move || to_email.get()}
                        </div>
                    </div>

                    // Subject field
                    <div>
                        <label class="block text-[10px] tracking-wider uppercase font-semibold text-outline-variant mb-1">
                            "Subject Line"
                        </label>
                        <input 
                            type="text"
                            placeholder="Enter email subject..."
                            prop:value=subject
                            on:input=move |ev| set_subject.set(event_target_value(&ev))
                            class="w-full bg-surface-container-lowest border border-outline-variant/30 rounded-lg px-3 py-2 text-xs text-on-surface focus:outline-none focus:border-primary transition-colors"
                        />
                    </div>

                    // Rich text formatting toolbar
                    <div>
                        <label class="block text-[10px] tracking-wider uppercase font-semibold text-outline-variant mb-1">
                            "Body Content"
                        </label>
                        <div class="bg-surface-container-lowest border border-outline-variant/30 rounded-xl overflow-hidden flex flex-col focus-within:border-primary transition-all">
                            
                            // Toolbar
                            <div class="bg-surface-container border-b border-outline-variant/30 px-2 py-1 flex items-center gap-1">
                                <button 
                                    on:click=move |_| insert_formatting("<b>", "</b>")
                                    type="button"
                                    title="Bold"
                                    class="p-1 hover:bg-surface-container-high rounded text-outline hover:text-on-surface transition-colors flex items-center justify-center"
                                >
                                    <span class="material-symbols-outlined text-[16px]">"format_bold"</span>
                                </button>
                                <button 
                                    on:click=move |_| insert_formatting("<i>", "</i>")
                                    type="button"
                                    title="Italic"
                                    class="p-1 hover:bg-surface-container-high rounded text-outline hover:text-on-surface transition-colors flex items-center justify-center"
                                >
                                    <span class="material-symbols-outlined text-[16px]">"format_italic"</span>
                                </button>
                                <button 
                                    on:click=move |_| insert_formatting("<u>", "</u>")
                                    type="button"
                                    title="Underline"
                                    class="p-1 hover:bg-surface-container-high rounded text-outline hover:text-on-surface transition-colors flex items-center justify-center"
                                >
                                    <span class="material-symbols-outlined text-[16px]">"format_underlined"</span>
                                </button>
                                <div class="w-px h-4 bg-outline-variant/30 mx-1"></div>
                                <button 
                                    on:click=move |_| insert_formatting("<h3 class=\"text-lg font-bold text-primary\">", "</h3>")
                                    type="button"
                                    title="Heading"
                                    class="p-1 hover:bg-surface-container-high rounded text-outline hover:text-on-surface transition-colors flex items-center justify-center"
                                >
                                    <span class="material-symbols-outlined text-[16px]">"title"</span>
                                </button>
                                <button 
                                    on:click=move |_| insert_formatting("<a href=\"#\" class=\"text-primary hover:underline\">", "</a>")
                                    type="button"
                                    title="Link"
                                    class="p-1 hover:bg-surface-container-high rounded text-outline hover:text-on-surface transition-colors flex items-center justify-center"
                                >
                                    <span class="material-symbols-outlined text-[16px]">"link"</span>
                                </button>
                                <button 
                                    on:click=move |_| insert_formatting("<pre class=\"bg-surface-container p-2 font-mono text-[11px] rounded\">", "</pre>")
                                    type="button"
                                    title="Code Block"
                                    class="p-1 hover:bg-surface-container-high rounded text-outline hover:text-on-surface transition-colors flex items-center justify-center"
                                >
                                    <span class="material-symbols-outlined text-[16px]">"code"</span>
                                </button>
                            </div>

                            // Textarea input
                            <textarea 
                                id="email-body-textarea"
                                placeholder="Write email body using rich text tools or plain HTML..."
                                prop:value=body
                                on:input=move |ev| set_body.set(event_target_value(&ev))
                                rows="8"
                                class="w-full bg-transparent border-0 px-3 py-2 text-xs text-on-surface focus:ring-0 focus:outline-none resize-none"
                            ></textarea>
                        </div>
                    </div>

                    // S3/R2 File Attachments Section
                    <div class="border-t border-outline-variant/20 pt-4">
                        <div class="flex items-center justify-between mb-2">
                            <label class="block text-[10px] tracking-wider uppercase font-semibold text-outline-variant">
                                "Attachments"
                            </label>
                            <div class="flex items-center gap-2">
                                <input 
                                    type="file" 
                                    node_ref=local_file_input_ref
                                    on:change=handle_local_upload
                                    class="hidden"
                                />
                                <button 
                                    type="button"
                                    on:click=trigger_local_upload
                                    disabled=uploading_attachment
                                    class="text-[9px] jetbrains font-bold uppercase text-primary hover:underline flex items-center gap-0.5"
                                >
                                    <span class="material-symbols-outlined text-[12px]">"add"</span>
                                    {move || if uploading_attachment.get() { "Uploading..." } else { "Attach Local File" }}
                                </button>
                            </div>
                        </div>

                        // Attachments checkboxes
                        <div class="bg-surface-container-lowest border border-outline-variant/30 rounded-xl p-3 max-h-36 overflow-y-auto space-y-2">
                            
                            // 1. Existing Record Attachments
                            <For 
                                each=move || files_signal.get() 
                                key=|f| f.id 
                                children={
                                    let toggle_attachment = toggle_attachment.clone();
                                    move |f| {
                                        let key = f.file_url.clone();
                                        let key_cl = key.clone();
                                        let checked = move || selected_attachments.get().contains(&key);
                                        let toggle = toggle_attachment.clone();
                                        view! {
                                            <label class="flex items-center gap-2 text-xs font-medium cursor-pointer py-1 select-none hover:bg-surface-container-high/40 px-2 rounded-lg transition-colors">
                                                <input 
                                                    type="checkbox"
                                                    prop:checked=checked
                                                    on:change={
                                                        let key_cl = key_cl.clone();
                                                        let toggle = toggle.clone();
                                                        move |_| toggle(key_cl.clone())
                                                    }
                                                    class="rounded border-outline-variant/40 text-primary focus:ring-primary w-3.5 h-3.5"
                                                />
                                                <span class="material-symbols-outlined text-outline text-[14px]">"description"</span>
                                                <span class="truncate flex-1 text-on-surface">{f.file_name}</span>
                                            </label>
                                        }
                                    }
                                }
                            />

                            // 2. Newly Uploaded Local Files
                            <For 
                                each=move || custom_attachments.get() 
                                key=|(_, k)| k.clone() 
                                children={
                                    let toggle_attachment = toggle_attachment.clone();
                                    move |(name, key)| {
                                        let key_cl = key.clone();
                                        let checked = move || selected_attachments.get().contains(&key);
                                        let toggle = toggle_attachment.clone();
                                        view! {
                                            <label class="flex items-center gap-2 text-xs font-medium cursor-pointer py-1 select-none hover:bg-surface-container-high/40 px-2 rounded-lg transition-colors bg-primary/5 border border-primary/10">
                                                <input 
                                                    type="checkbox"
                                                    prop:checked=checked
                                                    on:change={
                                                        let key_cl = key_cl.clone();
                                                        let toggle = toggle.clone();
                                                        move |_| toggle(key_cl.clone())
                                                    }
                                                    class="rounded border-outline-variant/40 text-primary focus:ring-primary w-3.5 h-3.5"
                                                />
                                                <span class="material-symbols-outlined text-primary text-[14px]">"cloud_done"</span>
                                                <span class="truncate flex-1 text-primary font-bold">{name}</span>
                                            </label>
                                        }
                                    }
                                }
                            />

                            <Show when=move || files_signal.get().is_empty() && custom_attachments.get().is_empty()>
                                <div class="text-center py-4 text-[10px] text-outline-variant font-mono uppercase">
                                    "NO_ATTACHMENTS_STAGED"
                                </div>
                            </Show>
                        </div>
                    </div>
                </div>

                // Footer actions
                <div class="flex items-center justify-end gap-3 border-t border-outline-variant/30 pt-4 mt-6">
                    <button 
                        on:click=move |_| on_close.run(())
                        disabled=sending
                        class="px-4 py-2 text-xs font-medium border border-outline-variant/30 hover:bg-surface-container-high hover:border-outline rounded-lg transition-colors cursor-pointer"
                    >
                        "Cancel"
                    </button>
                    <button 
                        on:click=handle_send
                        disabled=move || sending.get() || subject.get().is_empty() || body.get().is_empty()
                        class="px-5 py-2 text-xs font-bold text-on-primary bg-primary hover:bg-primary-container disabled:opacity-50 disabled:cursor-not-allowed rounded-lg shadow-sm flex items-center gap-1.5 transition-colors cursor-pointer"
                    >
                        <Show 
                            when=move || sending.get()
                            fallback=move || view! {
                                <span class="material-symbols-outlined text-[14px]">"send"</span>
                                <span>"Send Email"</span>
                            }
                        >
                            <span class="material-symbols-outlined animate-spin text-[14px]">"progress_activity"</span>
                            <span>"Sending..."</span>
                        </Show>
                    </button>
                </div>

            </div>
        </div>
    }
}


