use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RecordDocumentModel {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub target_record_id: uuid::Uuid,
    pub file_url: String,
    pub file_name: String,
    pub uploaded_at: String,
}

#[component]
pub fn FileAttachments(
    #[prop(into)] entity_type: String,
    #[prop(optional)] entity_id: Option<uuid::Uuid>,
    #[prop(optional)] files: Option<Signal<Vec<RecordDocumentModel>>>,
    #[prop(optional)] on_upload: Option<Callback<(String, String)>>, // (file_name, file_url)
    #[prop(optional)] on_delete: Option<Callback<uuid::Uuid>>,
    #[prop(optional)] on_download: Option<Callback<String>>, // file_url/file_key -> triggers download link generation
    #[prop(optional)] on_file_drop: Option<Callback<String>>, // backwards compatibility
) -> impl IntoView {
    let _ = entity_id;
    let (is_dragging, set_dragging) = signal(false);
    let (is_uploading, set_uploading) = signal(false);
    let (error_msg, set_error_msg) = signal::<Option<String>>(None);
    
    // File input reference
    let file_input_ref = NodeRef::<leptos::html::Input>::new();

    let trigger_file_select = move |_| {
        if let Some(input) = file_input_ref.get() {
            input.click();
        }
    };

    let handle_file_upload = move |file: web_sys::File| {
        set_uploading.set(true);
        set_error_msg.set(None);
        
        let on_upload = on_upload.clone();
        let on_file_drop = on_file_drop.clone();
        
        leptos::task::spawn_local(async move {
            #[cfg(not(feature = "ssr"))]
            {
                match upload_file_to_s3(file).await {
                    Ok((name, url)) => {
                        if let Some(ref cb) = on_upload {
                            cb.run((name.clone(), url.clone()));
                        }
                        if let Some(ref cb) = on_file_drop {
                            cb.run(name);
                        }
                    }
                    Err(e) => {
                        set_error_msg.set(Some(e));
                    }
                }
            }
            #[cfg(feature = "ssr")]
            {
                let _ = on_upload;
                let _ = on_file_drop;
            }
            set_uploading.set(false);
        });
    };

    let handle_change = move |ev: web_sys::Event| {
        let target = ev.target().and_then(|t| wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlInputElement>(t).ok());
        if let Some(input) = target {
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    handle_file_upload(file);
                }
            }
        }
    };

    view! {
        <div class="bg-surface-container p-6 rounded-2xl border border-outline-variant/30 shadow-xs space-y-6">
            <div class="flex items-center justify-between border-b border-outline-variant/15 pb-3">
                <div class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-primary text-[20px]">"folder_open"</span>
                    <h4 class="text-sm font-bold uppercase tracking-wider jetbrains text-on-surface">"Record Attachments"</h4>
                </div>
                <span class="px-2 py-0.5 border border-outline-variant/30 rounded text-[9px] font-bold uppercase text-outline font-mono">
                    {entity_type.clone()}
                </span>
            </div>

            // Hidden file input
            <input 
                type="file" 
                node_ref=file_input_ref
                on:change=handle_change
                class="hidden"
            />

            // Drag and Drop Zone
            <div 
                class=move || {
                    let mut base = "border-2 border-dashed rounded-xl p-8 flex flex-col items-center justify-center gap-3 transition-all duration-300 cursor-pointer ".to_string();
                    if is_dragging.get() {
                        base.push_str("border-primary bg-primary/5 scale-[0.99]");
                    } else if is_uploading.get() {
                        base.push_str("border-outline-variant bg-surface-container-high pointer-events-none opacity-60");
                    } else {
                        base.push_str("border-outline-variant/50 hover:border-primary/50 hover:bg-primary/5");
                    }
                    base
                }
                on:click=trigger_file_select
                on:dragover=move |e| { e.prevent_default(); set_dragging.set(true); }
                on:dragleave=move |e| { e.prevent_default(); set_dragging.set(false); }
                on:drop=move |e| { 
                    e.prevent_default(); 
                    set_dragging.set(false); 
                    if let Some(dt) = e.data_transfer() {
                        if let Some(files) = dt.files() {
                            if let Some(file) = files.get(0) {
                                handle_file_upload(file);
                            }
                        }
                    }
                }
            >
                <Show 
                    when=move || is_uploading.get()
                    fallback=move || view! {
                        <div class="w-10 h-10 rounded-full bg-primary/10 text-primary flex items-center justify-center">
                            <span class="material-symbols-outlined text-[20px]">"upload_file"</span>
                        </div>
                        <div class="text-center">
                            <p class="text-xs font-semibold text-on-surface">"Drag & drop file here, or "<span class="text-primary hover:underline font-bold">"browse"</span></p>
                            <p class="text-[10px] text-outline mt-1">"PDF, Images, Documents up to 50MB"</p>
                        </div>
                    }
                >
                    <div class="flex flex-col items-center gap-2">
                        <span class="material-symbols-outlined animate-spin text-primary text-[24px]">"progress_activity"</span>
                        <span class="text-xs font-mono font-bold uppercase tracking-wider text-outline">"UPLOADING_TO_VAULT..."</span>
                    </div>
                </Show>
            </div>

            // Error display
            <Show when=move || error_msg.get().is_some()>
                <div class="bg-error/10 border-l-4 border-error p-3 rounded-r-lg jetbrains text-xs text-error font-medium flex items-start gap-2">
                    <span class="material-symbols-outlined text-sm shrink-0">"error"</span>
                    <div>
                        <div class="font-bold uppercase">"Upload Error"</div>
                        <div class="mt-0.5">{move || error_msg.get().unwrap_or_default()}</div>
                    </div>
                </div>
            </Show>

            // Attachments List
            <Show when=move || files.is_some()>
                {
                    let files_sig = files.unwrap();
                    let on_delete = on_delete.clone();
                    let on_download = on_download.clone();
                    
                    view! {
                        <div class="space-y-2">
                            <For 
                                each=move || files_sig.get() 
                                key=|f| f.id 
                                children={
                                    let on_delete = on_delete.clone();
                                    let on_download = on_download.clone();
                                    move |f| {
                                        let doc_id = f.id;
                                        let file_name = f.file_name.clone();
                                        let file_url = f.file_url.clone();
                                        let on_delete = on_delete.clone();
                                        let on_download = on_download.clone();
                                        
                                        let is_pdf = file_name.to_lowercase().ends_with(".pdf");
                                        let is_image = file_name.to_lowercase().ends_with(".png") 
                                            || file_name.to_lowercase().ends_with(".jpg") 
                                            || file_name.to_lowercase().ends_with(".jpeg") 
                                            || file_name.to_lowercase().ends_with(".webp") 
                                            || file_name.to_lowercase().ends_with(".gif");
                                            
                                        let icon = if is_pdf { "picture_as_pdf" } else if is_image { "image" } else { "description" };
                                        let icon_color = if is_pdf { "text-red-500" } else if is_image { "text-blue-500" } else { "text-slate-500" };

                                        view! {
                                            <div class="flex items-center justify-between p-3 bg-surface-container-lowest border border-outline-variant/20 rounded-xl hover:border-outline-variant/60 transition-all group">
                                                <div 
                                                    on:click={
                                                        let on_download = on_download.clone();
                                                        let file_url = file_url.clone();
                                                        move |_| {
                                                            if let Some(ref cb) = on_download {
                                                                cb.run(file_url.clone());
                                                            }
                                                        }
                                                    }
                                                    class="flex items-center gap-3 cursor-pointer flex-1 min-w-0"
                                                >
                                                    <div class="w-8 h-8 rounded-lg bg-surface-container flex items-center justify-center shrink-0 border border-outline-variant/10 group-hover:bg-primary/5 transition-colors">
                                                        <span class=format!("material-symbols-outlined text-[16px] {}", icon_color)>{icon}</span>
                                                    </div>
                                                    <div class="flex flex-col min-w-0">
                                                        <span class="text-xs font-bold text-on-surface truncate group-hover:text-primary transition-colors">{file_name}</span>
                                                        <span class="text-[9px] text-outline font-mono mt-0.5">{f.uploaded_at.clone()}</span>
                                                    </div>
                                                </div>

                                                <div class="flex items-center gap-1 shrink-0 opacity-80 group-hover:opacity-100 transition-opacity">
                                                    <button 
                                                        on:click={
                                                            let on_download = on_download.clone();
                                                            let file_url = file_url.clone();
                                                            move |e| {
                                                                e.stop_propagation();
                                                                if let Some(ref cb) = on_download {
                                                                    cb.run(file_url.clone());
                                                                }
                                                            }
                                                        }
                                                        title="Download Attachment"
                                                        class="p-1 hover:bg-surface-container rounded text-outline hover:text-primary transition-colors flex items-center justify-center"
                                                    >
                                                        <span class="material-symbols-outlined text-[16px]">"download"</span>
                                                    </button>
                                                    <button 
                                                        on:click={
                                                            let on_delete = on_delete.clone();
                                                            move |e| {
                                                                e.stop_propagation();
                                                                if let Some(ref cb) = on_delete {
                                                                    cb.run(doc_id);
                                                                }
                                                            }
                                                        }
                                                        title="Delete Attachment"
                                                        class="p-1 hover:bg-error/10 rounded text-outline hover:text-error transition-colors flex items-center justify-center"
                                                    >
                                                        <span class="material-symbols-outlined text-[16px]">"delete"</span>
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                    }
                                }
                            />

                            <Show when=move || files_sig.get().is_empty()>
                                <div class="text-center py-8 border border-dashed border-outline-variant/20 rounded-xl bg-surface-container-lowest/30">
                                    <span class="material-symbols-outlined text-outline-variant text-[24px] mb-1 block">"draft"</span>
                                    <span class="text-[10px] font-mono text-outline-variant font-bold uppercase">"NO_ATTACHED_FILES"</span>
                                </div>
                            </Show>
                        </div>
                    }
                }
            </Show>
        </div>
    }
}

#[cfg(not(feature = "ssr"))]
pub async fn upload_file_to_s3(file: web_sys::File) -> Result<(String, String), String> {
    use wasm_bindgen_futures::JsFuture;
    
    let filename = file.name();
    let content_type = file.type_();
    let content_type = if content_type.is_empty() { "application/octet-stream".to_string() } else { content_type };
    
    // 1. Get presigned URL
    #[derive(serde::Serialize)]
    struct PresignedReq {
        filename: String,
        content_type: String,
    }
    
    #[derive(serde::Deserialize)]
    struct PresignedResp {
        upload_url: String,
        file_key: String,
    }
    
    let client = reqwest::Client::new();
    let presigned_res = client.post("/api/forms/upload-url")
        .json(&PresignedReq { filename: filename.clone(), content_type: content_type.clone() })
        .send()
        .await
        .map_err(|e| format!("Failed to get presigned URL: {}", e))?;
        
    if !presigned_res.status().is_success() {
        return Err(format!("Presigned URL server error: {}", presigned_res.status()));
    }
    
    let resp: PresignedResp = presigned_res.json()
        .await
        .map_err(|e| format!("Failed to parse presigned response: {}", e))?;
        
    // 2. Read file bytes
    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|e| format!("Failed to read file: {:?}", e))?;
    let array = js_sys::Uint8Array::new(&array_buffer);
    let bytes = array.to_vec();
    
    // 3. PUT bytes to S3
    let upload_res = client.put(&resp.upload_url)
        .header("Content-Type", content_type)
        .body(bytes)
        .send()
        .await
        .map_err(|e| format!("Failed to upload to S3: {}", e))?;
        
    if !upload_res.status().is_success() {
        return Err(format!("S3 upload failed: {}", upload_res.status()));
    }
    
    Ok((filename, resp.file_key))
}
