//! Shared drag-and-drop photo card for property hub, units, systems, projects.

use crate::pages::landlord::digital_vault::{
    presign_vault_upload, register_vault_document, VaultDocumentType,
};
use leptos::prelude::*;
use leptos::task::spawn_local;
use uuid::Uuid;

/// Entity surface for vault `related_entity_type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhotoEntityKind {
    Asset,
    Case,
}

impl PhotoEntityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asset => "atlas_assets",
            Self::Case => "atlas_cases",
        }
    }
}

fn begin_photo_upload(
    entity_kind: PhotoEntityKind,
    entity_id: Uuid,
    parent_asset_id: Option<Uuid>,
    as_cover: bool,
    filename: String,
    content_type: String,
    bytes: Vec<u8>,
    uploading: RwSignal<bool>,
    err: RwSignal<Option<String>>,
    on_uploaded: Option<Callback<()>>,
) {
    if uploading.get_untracked() {
        return;
    }
    if !content_type.starts_with("image/") {
        err.set(Some("Images only (JPEG, PNG, WebP, HEIC)".into()));
        return;
    }
    uploading.set(true);
    err.set(None);
    spawn_local(async move {
        let doc_type = if as_cover {
            VaultDocumentType::Cover.as_str()
        } else {
            VaultDocumentType::Photo.as_str()
        };
        let result = async {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (
                    &filename,
                    &content_type,
                    &bytes,
                    entity_kind,
                    entity_id,
                    parent_asset_id,
                    doc_type,
                );
                return Err("Photo upload requires the browser client".into());
            }
            #[cfg(target_arch = "wasm32")]
            {
                let presign = presign_vault_upload(filename, content_type.clone())
                    .await
                    .map_err(|e| e.to_string())?;
                put_bytes_to_presign(&presign.upload_url, &content_type, &bytes).await?;
                register_vault_document(
                    entity_kind.as_str().to_string(),
                    entity_id.to_string(),
                    doc_type.to_string(),
                    presign.r2_key,
                    Some(content_type),
                    parent_asset_id.map(|id| id.to_string()),
                )
                .await
                .map_err(|e| e.to_string())?;
                Ok::<(), String>(())
            }
        }
        .await;
        uploading.set(false);
        match result {
            Ok(()) => {
                if let Some(cb) = on_uploaded {
                    cb.run(());
                }
            }
            Err(e) => err.set(Some(e)),
        }
    });
}

#[component]
pub fn PhotoMediaCard(
    entity_kind: PhotoEntityKind,
    entity_id: Uuid,
    #[prop(into)] gallery_href: Signal<String>,
    #[prop(into)] photo_count: Signal<usize>,
    #[prop(into)] has_cover: Signal<bool>,
    #[prop(default = false)] cover_eligible: bool,
    /// Building id when this card is a unit. Pass `Uuid::nil()` when none.
    #[prop(default = Uuid::nil())]
    parent_asset_id: Uuid,
    #[prop(optional, into)] empty_label: Option<String>,
    #[prop(optional)] on_uploaded: Option<Callback<()>>,
) -> impl IntoView {
    let parent_asset_id = (!parent_asset_id.is_nil()).then_some(parent_asset_id);
    let uploading = RwSignal::new(false);
    let err = RwSignal::new(None::<String>);
    let drag_over = RwSignal::new(false);
    let file_input: NodeRef<leptos::html::Input> = NodeRef::new();

    let empty_label = empty_label.unwrap_or_else(|| {
        if cover_eligible {
            "No cover yet".into()
        } else {
            "No photos yet".into()
        }
    });

    view! {
        <div
            class=move || {
                if drag_over.get() {
                    "hub-media-card hub-media-card--photos hub-media-card--drop-active"
                } else {
                    "hub-media-card hub-media-card--photos"
                }
            }
            style="position:relative;"
            on:dragover=move |ev| {
                ev.prevent_default();
                drag_over.set(true);
            }
            on:dragleave=move |_| drag_over.set(false)
            on:drop=move |ev| {
                ev.prevent_default();
                drag_over.set(false);
                #[cfg(target_arch = "wasm32")]
                {
                    let as_cover = cover_eligible && !has_cover.get_untracked();
                    if let Some(dt) = ev.data_transfer() {
                        if let Some(files) = dt.files() {
                            if let Some(file) = files.get(0) {
                                spawn_local(async move {
                                    match read_web_file(&file).await {
                                        Ok((bytes, name, ct)) => begin_photo_upload(
                                            entity_kind,
                                            entity_id,
                                            parent_asset_id,
                                            as_cover,
                                            name,
                                            ct,
                                            bytes,
                                            uploading,
                                            err,
                                            on_uploaded,
                                        ),
                                        Err(e) => err.set(Some(e)),
                                    }
                                });
                            }
                        }
                    }
                }
            }
        >
            <input
                type="file"
                accept="image/jpeg,image/png,image/webp,image/heic,image/*"
                node_ref=file_input
                style="display:none;"
                on:change=move |_| {
                    #[cfg(target_arch = "wasm32")]
                    {
                        let as_cover = cover_eligible && !has_cover.get_untracked();
                        if let Some(input) = file_input.get() {
                            spawn_local(async move {
                                match read_input_element(&input).await {
                                    Ok((bytes, name, ct)) => begin_photo_upload(
                                        entity_kind,
                                        entity_id,
                                        parent_asset_id,
                                        as_cover,
                                        name,
                                        ct,
                                        bytes,
                                        uploading,
                                        err,
                                        on_uploaded,
                                    ),
                                    Err(e) => err.set(Some(e)),
                                }
                            });
                        }
                    }
                }
            />
            <button
                type="button"
                class="hub-media-card__photo-hit"
                style="display:block;width:100%;border:none;background:transparent;padding:0;cursor:pointer;text-align:inherit;"
                disabled=move || uploading.get()
                on:click=move |_| {
                    if let Some(input) = file_input.get() {
                        input.click();
                    }
                }
            >
                {move || {
                    let n = photo_count.get();
                    let busy = uploading.get();
                    if busy {
                        view! {
                            <div class="hub-media-card__photo-empty">
                                <span class="material-symbols-outlined">"progress_activity"</span>
                                <span>"Uploading…"</span>
                            </div>
                        }.into_any()
                    } else if n == 0 {
                        let label = empty_label.clone();
                        view! {
                            <div class="hub-media-card__photo-empty">
                                <span class="material-symbols-outlined">"add_a_photo"</span>
                                <span>{label}</span>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="hub-media-card__photo-empty hub-media-card__photo-empty--has">
                                <span class="material-symbols-outlined">"photo_library"</span>
                                <span>{format!("{n} photos")}</span>
                            </div>
                        }.into_any()
                    }
                }}
            </button>
            <div class="hub-media-card__photo-foot">
                <div>
                    <p class="hub-media-card__title">"Photos"</p>
                    <p class="hub-media-card__sub">
                        {move || {
                            if uploading.get() {
                                "Uploading…".to_string()
                            } else if let Some(e) = err.get() {
                                e
                            } else {
                                "Drop photos or open gallery".to_string()
                            }
                        }}
                    </p>
                </div>
                <a
                    class="folio-btn folio-btn--primary folio-btn--sm press"
                    href=move || gallery_href.get()
                    on:click=move |ev| ev.stop_propagation()
                >
                    "Open"
                </a>
            </div>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
async fn put_bytes_to_presign(
    upload_url: &str,
    content_type: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let body = js_sys::Uint8Array::from(bytes);
    let resp = gloo_net::http::Request::put(upload_url)
        .header("Content-Type", content_type)
        .body(body)
        .map_err(|e| format!("Upload request build failed: {e}"))?
        .send()
        .await
        .map_err(|e| format!("Upload failed: {e}"))?;
    if !resp.ok() {
        return Err(format!("Upload failed: HTTP {}", resp.status()));
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn read_web_file(file: &web_sys::File) -> Result<(Vec<u8>, String, String), String> {
    use wasm_bindgen_futures::JsFuture;
    let name = file.name();
    let content_type = {
        let t = file.type_();
        if t.is_empty() {
            "application/octet-stream".to_string()
        } else {
            t
        }
    };
    let buf = JsFuture::from(file.array_buffer())
        .await
        .map_err(|e| format!("Could not read file: {e:?}"))?;
    let array = js_sys::Uint8Array::new(&buf);
    let mut bytes = vec![0u8; array.length() as usize];
    array.copy_to(&mut bytes);
    Ok((bytes, name, content_type))
}

#[cfg(target_arch = "wasm32")]
async fn read_input_element(
    input: &web_sys::HtmlInputElement,
) -> Result<(Vec<u8>, String, String), String> {
    let files = input.files().ok_or_else(|| "No file selected".to_string())?;
    let file = files.get(0).ok_or_else(|| "No file selected".to_string())?;
    read_web_file(&file).await
}
