use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct FileItem {
    pub name: String,
    pub file_type: String,
    pub size: String,
    pub timestamp: String,
}

#[component]
pub fn FileAttachments(
    #[prop(into)] entity_type: String,
) -> impl IntoView {
    let (is_dragging, set_dragging) = signal(false);
    let (files, _) = signal(vec![
        FileItem { name: "contract.pdf".into(), file_type: "PDF".into(), size: "2.4 MB".into(), timestamp: "2 mins ago".into() },
        FileItem { name: "logo.png".into(), file_type: "Image".into(), size: "845 KB".into(), timestamp: "1 hour ago".into() },
    ]);

    view! {
        <div class="file-attachments">
            <div class="attachments-header">
                <h4>"Attachments" <span class="badge badge-default" style="margin-left: 0.5rem;">{entity_type}</span></h4>
            </div>
            
            <div 
                class=move || if is_dragging.get() { "drop-zone active" } else { "drop-zone" }
                on:dragover=move |e| { e.prevent_default(); set_dragging.set(true); }
                on:dragleave=move |e| { e.prevent_default(); set_dragging.set(false); }
                on:drop=move |e| { e.prevent_default(); set_dragging.set(false); }
            >
                <div class="upload-icon">"☁️"</div>
                <p>"Drag & drop files here or " <a>"browse"</a></p>
                <p class="text-xs text-muted">"Supports PDF, images, and documents up to 50MB"</p>
            </div>

            <div class="file-list">
                <For each=move || files.get() key=|f| f.name.clone() children=move |f| {
                    view! {
                        <div class="file-item">
                            <div class="file-icon">
                                {if f.file_type == "PDF" { "📄" } else { "🖼️" }}
                            </div>
                            <div class="file-details">
                                <span class="file-name">{f.name.clone()}</span>
                                <span class="file-meta">{f.size.clone()} " • " {f.timestamp.clone()}</span>
                            </div>
                            <button class="btn btn-small">"✕"</button>
                        </div>
                    }
                } />
            </div>
        </div>
    }
}
