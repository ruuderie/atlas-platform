// apps/folio/src/pages/landlord/digital_vault.rs
//
// Landlord Digital Vault — /l/vault
//
// Manages all documents in the landlord's vault (lease agreements, permits,
// certificates, inspection reports, etc.). Reuses /api/folio/vault/documents.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::html::Input;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub id: Uuid,
    pub document_category: String,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub is_counterparty_visible: bool,
    pub requires_signature: bool,
    pub is_signed: bool,
    pub version_number: i32,
    pub created_at: String,
}

/// Document type — mirrors backend `PmDocumentType` snake_case values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VaultDocumentType {
    LeaseAgreement,
    StrPermit,
    InsurancePolicy,
    InspectionReport,
    TitleDeed,
    IdDocument,
    MaintenanceReceipt,
    SecurityDepositReceipt,
    ContractorLicense,
    CondominioStatement,
}

impl VaultDocumentType {
    pub const ALL: &'static [Self] = &[
        Self::LeaseAgreement,
        Self::StrPermit,
        Self::InsurancePolicy,
        Self::InspectionReport,
        Self::TitleDeed,
        Self::IdDocument,
        Self::MaintenanceReceipt,
        Self::SecurityDepositReceipt,
        Self::ContractorLicense,
        Self::CondominioStatement,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LeaseAgreement => "lease_agreement",
            Self::StrPermit => "str_permit",
            Self::InsurancePolicy => "insurance_policy",
            Self::InspectionReport => "inspection_report",
            Self::TitleDeed => "title_deed",
            Self::IdDocument => "id_document",
            Self::MaintenanceReceipt => "maintenance_receipt",
            Self::SecurityDepositReceipt => "security_deposit_receipt",
            Self::ContractorLicense => "contractor_license",
            Self::CondominioStatement => "condominio_statement",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::LeaseAgreement => "Lease Agreement",
            Self::StrPermit => "STR Permit",
            Self::InsurancePolicy => "Insurance Policy",
            Self::InspectionReport => "Inspection Report",
            Self::TitleDeed => "Title Deed",
            Self::IdDocument => "ID Document",
            Self::MaintenanceReceipt => "Maintenance Receipt",
            Self::SecurityDepositReceipt => "Security Deposit Receipt",
            Self::ContractorLicense => "Contractor License",
            Self::CondominioStatement => "Condomínio Statement",
        }
    }
}

// ── Server functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VaultEntityOpt {
    id: Uuid,
    label: String,
}

#[cfg(feature = "ssr")]
async fn load_vault_entity_opts(
    entity_type: &str,
    token: &str,
) -> Result<Vec<VaultEntityOpt>, String> {
    match entity_type {
        "atlas_assets" => {
            #[derive(Deserialize)]
            struct Raw {
                id: Uuid,
                name: String,
            }
            let rows: Vec<Raw> =
                crate::atlas_client::authenticated_get("/api/folio/assets", token, None).await?;
            Ok(rows
                .into_iter()
                .map(|r| VaultEntityOpt {
                    id: r.id,
                    label: r.name,
                })
                .collect())
        }
        "atlas_contracts" => {
            #[derive(Deserialize)]
            struct Raw {
                id: Uuid,
                status: String,
                start_date: Option<chrono::NaiveDate>,
            }
            let rows: Vec<Raw> =
                crate::atlas_client::authenticated_get("/api/folio/leases", token, None).await?;
            Ok(rows
                .into_iter()
                .map(|r| VaultEntityOpt {
                    id: r.id,
                    label: format!(
                        "{} · {}",
                        r.status.replace('_', " "),
                        r.start_date
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "—".into())
                    ),
                })
                .collect())
        }
        "atlas_applications" => {
            #[derive(Deserialize)]
            struct Raw {
                id: Uuid,
                status: String,
                submitted_at: Option<chrono::DateTime<chrono::Utc>>,
            }
            let rows: Vec<Raw> =
                crate::atlas_client::authenticated_get("/api/folio/applications", token, None)
                    .await?;
            Ok(rows
                .into_iter()
                .map(|r| VaultEntityOpt {
                    id: r.id,
                    label: format!(
                        "Application · {} · {}",
                        r.status.replace('_', " "),
                        r.submitted_at
                            .map(|d| d.date_naive().to_string())
                            .unwrap_or_else(|| "—".into())
                    ),
                })
                .collect())
        }
        "atlas_service_providers" => {
            #[derive(Deserialize)]
            struct Raw {
                id: Uuid,
                business_name: String,
            }
            let rows: Vec<Raw> =
                crate::atlas_client::authenticated_get("/api/folio/vendors", token, None).await?;
            Ok(rows
                .into_iter()
                .map(|r| VaultEntityOpt {
                    id: r.id,
                    label: r.business_name,
                })
                .collect())
        }
        _ => Ok(Vec::new()),
    }
}

#[server(ListVaultEntityOptions, "/api")]
async fn list_vault_entity_options(
    entity_type: String,
) -> Result<Vec<VaultEntityOpt>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    load_vault_entity_opts(&entity_type, &token)
        .await
        .map_err(server_fn::error::ServerFnError::new)
}

#[server(LLFetchVaultDocs, "/api")]
pub async fn ll_fetch_vault_docs(
    entity_type: Option<String>,
) -> Result<Vec<DocumentSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let url = match entity_type {
        Some(et) => format!("/api/folio/vault/documents?entity_type={et}"),
        None => "/api/folio/vault/documents".to_string(),
    };
    crate::atlas_client::authenticated_get::<Vec<DocumentSummary>>(&url, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[derive(Serialize)]
struct RegisterDocumentBody {
    entity_type: String,
    entity_id: Uuid,
    document_type: String,
    r2_key: String,
    mime_type: Option<String>,
    size_bytes: Option<i64>,
}

#[derive(Deserialize)]
struct RegisterDocumentResponse {
    id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresignUploadResponse {
    pub upload_url: String,
    pub r2_key: String,
}

#[derive(Serialize)]
struct PresignUploadBody {
    filename: String,
    content_type: String,
}

/// POST /api/folio/vault/presign — R2 PUT URL + object key.
#[server(PresignVaultUpload, "/api")]
pub async fn presign_vault_upload(
    filename: String,
    content_type: String,
) -> Result<PresignUploadResponse, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    if filename.trim().is_empty() {
        return Err(server_fn::error::ServerFnError::new("Filename is required"));
    }
    if content_type.trim().is_empty() || !content_type.contains('/') {
        return Err(server_fn::error::ServerFnError::new("Content type is required"));
    }
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let body = PresignUploadBody {
        filename: filename.trim().to_string(),
        content_type: content_type.trim().to_string(),
    };
    crate::atlas_client::authenticated_post::<PresignUploadBody, PresignUploadResponse>(
        "/api/folio/vault/presign",
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Presign failed: {e}")))
}

/// POST /api/folio/vault/documents — register metadata after R2 upload.
#[server(RegisterVaultDocument, "/api")]
pub async fn register_vault_document(
    entity_type: String,
    entity_id: String,
    document_type: String,
    r2_key: String,
    mime_type: Option<String>,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    if entity_type.trim().is_empty() {
        return Err(server_fn::error::ServerFnError::new("Entity type is required"));
    }
    if r2_key.trim().is_empty() {
        return Err(server_fn::error::ServerFnError::new("R2 key is required"));
    }
    if VaultDocumentType::ALL
        .iter()
        .all(|t| t.as_str() != document_type.as_str())
    {
        return Err(server_fn::error::ServerFnError::new("Invalid document type"));
    }
    let entity_id = Uuid::parse_str(entity_id.trim())
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid entity ID"))?;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let mime = mime_type
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let body = RegisterDocumentBody {
        entity_type: entity_type.trim().to_string(),
        entity_id,
        document_type,
        r2_key: r2_key.trim().to_string(),
        mime_type: mime,
        size_bytes: None,
    };
    let resp =
        crate::atlas_client::authenticated_post::<RegisterDocumentBody, RegisterDocumentResponse>(
            "/api/folio/vault/documents",
            &token,
            None,
            &body,
        )
        .await
        .map_err(|e| {
            server_fn::error::ServerFnError::new(format!("Register document failed: {e}"))
        })?;
    Ok(resp.id)
}

/// Browser PUT to the R2 presigned URL (hydrate / WASM only).
#[cfg(target_arch = "wasm32")]
async fn put_file_to_presign(
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
async fn read_input_file(
    input: &web_sys::HtmlInputElement,
) -> Result<(Vec<u8>, String, String), String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    let files = input.files().ok_or_else(|| "No file selected".to_string())?;
    let file = files.get(0).ok_or_else(|| "No file selected".to_string())?;
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

// ── Helpers ───────────────────────────────────────────────────────────────────

fn doc_icon(cat: &str) -> &'static str {
    match cat.to_lowercase().as_str() {
        c if c.contains("lease") || c.contains("agreement") => "📋",
        c if c.contains("permit") => "📜",
        c if c.contains("insurance") => "🛡",
        c if c.contains("inspection") => "🔍",
        c if c.contains("certificate") => "🏆",
        c if c.contains("tax") => "💼",
        c if c.contains("notice") => "📣",
        c if c.contains("id") => "🪪",
        _ => "📄",
    }
}

fn doc_label(category: &str) -> String {
    category
        .replace('_', " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn LandlordDigitalVault() -> impl IntoView {
    let q = leptos_router::hooks::use_query_map();
    let refresh = RwSignal::new(0u32);
    let cat_filter = RwSignal::new("all".to_string());
    let selected_doc = RwSignal::new(None::<DocumentSummary>);
    let show_register = RwSignal::new(false);

    let reg_entity_type = RwSignal::new("atlas_assets".to_string());
    let reg_entity_id = RwSignal::new(String::new());
    let reg_doc_type = RwSignal::new(VaultDocumentType::LeaseAgreement.as_str().to_string());
    let file_input_el: NodeRef<Input> = NodeRef::new();
    let selected_file_name = RwSignal::new(String::new());
    let registering = RwSignal::new(false);
    let reg_err = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        let map = q.get();
        if let Some(et) = map.get("entity_type") {
            if !et.is_empty() {
                reg_entity_type.set(et);
            }
        }
        if let Some(eid) = map.get("entity_id") {
            if !eid.is_empty() {
                reg_entity_id.set(eid);
                show_register.set(true);
            }
        }
    });

    let docs_res = Resource::new(move || refresh.get(), |_| ll_fetch_vault_docs(None));
    let entity_opts = Resource::new(
        move || reg_entity_type.get(),
        |et| async move { list_vault_entity_options(et).await },
    );

    let on_register = move |_| {
        let entity_type = reg_entity_type.get().trim().to_string();
        let entity_id = reg_entity_id.get().trim().to_string();
        let document_type = reg_doc_type.get();
        if entity_type.is_empty() || entity_id.is_empty() {
            reg_err.set(Some("Choose what this document belongs to.".into()));
            return;
        }
        let Some(input) = file_input_el.get() else {
            reg_err.set(Some("Choose a file to upload.".into()));
            return;
        };
        registering.set(true);
        reg_err.set(None);
        spawn_local(async move {
            #[cfg(target_arch = "wasm32")]
            {
                match read_input_file(&input).await {
                    Ok((bytes, filename, content_type)) => {
                        match presign_vault_upload(filename, content_type.clone()).await {
                            Ok(presign) => {
                                match put_file_to_presign(
                                    &presign.upload_url,
                                    &content_type,
                                    &bytes,
                                )
                                .await
                                {
                                    Ok(()) => {
                                        match register_vault_document(
                                            entity_type,
                                            entity_id,
                                            document_type,
                                            presign.r2_key,
                                            Some(content_type),
                                        )
                                        .await
                                        {
                                            Ok(_) => {
                                                show_register.set(false);
                                                reg_entity_id.set(String::new());
                                                selected_file_name.set(String::new());
                                                refresh.update(|n| *n += 1);
                                            }
                                            Err(e) => reg_err.set(Some(e.to_string())),
                                        }
                                    }
                                    Err(e) => reg_err.set(Some(e)),
                                }
                            }
                            Err(e) => reg_err.set(Some(e.to_string())),
                        }
                    }
                    Err(e) => reg_err.set(Some(e)),
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = (input, entity_type, entity_id, document_type);
                reg_err.set(Some(
                    "File upload runs in the browser after hydrate.".into(),
                ));
            }
            registering.set(false);
        });
    };

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <h1 class="page-title">"Digital Vault"</h1>
                    <p class="page-subtitle">"Leases, permits, certificates, and shared files"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>"↻ Refresh"</button>
                    <button
                        class="btn btn-primary btn-sm"
                        on:click=move |_| {
                            reg_err.set(None);
                            show_register.set(true);
                        }
                    >
                        "+ Register Document"
                    </button>
                </div>
            </div>

            // ── KPI / stats ──
            <Suspense fallback=|| ()>
                {move || docs_res.get().map(|res| {
                    match res.as_ref() {
                        Ok(docs) => {
                            let total    = docs.len();
                            let shared   = docs.iter().filter(|d| d.is_counterparty_visible).count();
                            let unsigned = docs.iter().filter(|d| d.requires_signature && !d.is_signed).count();
                            view! {
                                <div class="kpi-row" style="margin-bottom:1.25rem;">
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Total Documents"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">{total.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Shared with Tenants"</span>
                                        <span class="kpi-value" style="color:var(--green)">{shared.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Awaiting Signature"</span>
                                        <span class="kpi-value" style="color:var(--amber)">{unsigned.to_string()}</span>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(_) => ().into_any(),
                    }
                })}
            </Suspense>

            // ── Filter pills ──
            <div class="doc-filter-row">
                {
                    let pill = move |scope: &'static str, label: &'static str| view! {
                        <button
                            class=move || format!("filter-pill {}", if cat_filter.get() == scope { "filter-pill--active" } else { "" })
                            on:click=move |_| cat_filter.set(scope.to_string())
                        >{label}</button>
                    };
                    view! {
                        {pill("all",              "All")}
                        {pill("lease_agreement",  "Leases")}
                        {pill("permit",           "Permits")}
                        {pill("insurance",        "Insurance")}
                        {pill("inspection_report","Inspections")}
                        {pill("certificate",      "Certificates")}
                    }
                }
            </div>

            // ── Document grid ──
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading vault…"</div> }>
                {move || docs_res.get().map(|res| {
                    match res {
                        Ok(docs) => {
                            let cf = cat_filter.get();
                            let visible: Vec<_> = docs.into_iter().filter(|d| {
                                cf == "all" || d.document_category.contains(&cf)
                            }).collect();

                            if visible.is_empty() {
                                return view! { <div class="doc-empty">"No documents found."</div> }.into_any();
                            }

                            view! {
                                <div class="doc-grid">
                                    <For
                                        each=move || visible.clone()
                                        key=|d| d.id
                                        children=move |doc| {
                                            let d2 = doc.clone();
                                            let icon   = doc_icon(&doc.document_category);
                                            let label  = doc_label(&doc.document_category);
                                            let date   = doc.created_at.chars().take(10).collect::<String>();
                                            let shared = doc.is_counterparty_visible;
                                            let sig    = doc.requires_signature;
                                            let signed = doc.is_signed;
                                            let entity = doc.related_entity_type.clone();

                                            view! {
                                                <div class="doc-card" on:click=move |_| selected_doc.set(Some(d2.clone()))>
                                                    <div class="doc-card-icon">{icon}</div>
                                                    <div class="doc-card-body">
                                                        <div class="doc-card-title">{label}</div>
                                                        {entity.map(|et| view! {
                                                            <div class="doc-card-meta">{et.replace('_', " ")}</div>
                                                        })}
                                                        <div class="doc-card-meta">"v" {doc.version_number.to_string()} " · " {date}</div>
                                                    </div>
                                                    <div class="doc-card-badges">
                                                        {if sig && !signed {
                                                            view! { <span class="doc-badge doc-badge--action">"Needs Sig"</span> }.into_any()
                                                        } else if sig {
                                                            view! { <span class="doc-badge doc-badge--signed">"✓ Signed"</span> }.into_any()
                                                        } else { ().into_any() }}
                                                        {if shared {
                                                            view! { <span class="doc-badge doc-badge--shared">"Tenant Visible"</span> }.into_any()
                                                        } else { ().into_any() }}
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="doc-empty text-red-400">"Error: " {e.to_string()}</div>
                        }.into_any(),
                    }
                })}
            </Suspense>

            // ── Register document modal ──
            <Show when=move || show_register.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:32rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Register Document"</h3>
                            <button class="modal-close" on:click=move |_| show_register.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <p class="folio-empty__sub" style="margin:0;">
                                "Upload a file — we get a secure upload link, store the object, then register it in your vault."
                            </p>
                            <div class="form-field">
                                <label class="form-label">"File *"</label>
                                <input
                                    type="file"
                                    class="form-input"
                                    node_ref=file_input_el
                                    on:change=move |ev| {
                                        let el = event_target::<web_sys::HtmlInputElement>(&ev);
                                        let name = el
                                            .files()
                                            .and_then(|f| f.get(0))
                                            .map(|f| f.name())
                                            .unwrap_or_default();
                                        selected_file_name.set(name);
                                    }
                                />
                                {move || {
                                    let n = selected_file_name.get();
                                    if n.is_empty() {
                                        ().into_any()
                                    } else {
                                        view! { <p class="folio-empty__sub" style="margin:0.35rem 0 0;">{n}</p> }.into_any()
                                    }
                                }}
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Belongs to *"</label>
                                <select
                                    class="form-select"
                                    prop:value=move || reg_entity_type.get()
                                    on:change=move |ev| {
                                        reg_entity_type.set(event_target_value(&ev));
                                        reg_entity_id.set(String::new());
                                    }
                                >
                                    <option value="atlas_assets">"Property / unit"</option>
                                    <option value="atlas_contracts">"Lease"</option>
                                    <option value="atlas_applications">"Application"</option>
                                    <option value="atlas_service_providers">"Vendor"</option>
                                </select>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Record *"</label>
                                <Suspense fallback=|| view! { <p class="folio-empty__sub">"Loading…"</p> }>
                                    {move || entity_opts.get().map(|res| match res {
                                        Ok(list) if list.is_empty() => view! {
                                            <p class="folio-empty__sub">"Nothing to attach to yet for this type."</p>
                                        }.into_any(),
                                        Ok(list) => view! {
                                            <select
                                                class="form-select"
                                                prop:value=move || reg_entity_id.get()
                                                on:change=move |ev| reg_entity_id.set(event_target_value(&ev))
                                            >
                                                <option value="">"Select…"</option>
                                                {list.into_iter().map(|o| {
                                                    let id = o.id.to_string();
                                                    let label = o.label;
                                                    view! { <option value=id>{label}</option> }
                                                }).collect_view()}
                                            </select>
                                        }.into_any(),
                                        Err(e) => view! {
                                            <p class="text-red-400" style="font-size:0.875rem;">{e.to_string()}</p>
                                        }.into_any(),
                                    })}
                                </Suspense>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Document Type *"</label>
                                <select
                                    class="form-select"
                                    on:change=move |ev| reg_doc_type.set(event_target_value(&ev))
                                >
                                    {VaultDocumentType::ALL.iter().copied().map(|t| {
                                        view! { <option value=t.as_str()>{t.label()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            {move || reg_err.get().map(|e| view! {
                                <p class="text-red-400" style="font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_register.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                disabled=move || {
                                    registering.get()
                                        || reg_entity_id.get().trim().is_empty()
                                        || selected_file_name.get().trim().is_empty()
                                }
                                on:click=on_register
                            >
                                {move || if registering.get() { "Uploading…" } else { "Upload & register" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Detail modal ──
            <Show when=move || selected_doc.get().is_some()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:32rem;">
                        {move || selected_doc.get().map(|doc| {
                            let icon  = doc_icon(&doc.document_category);
                            let label = doc_label(&doc.document_category);
                            view! {
                                <div class="modal-header">
                                    <h3 class="modal-title">{icon} " " {label.clone()}</h3>
                                    <button class="modal-close" on:click=move |_| selected_doc.set(None)>"✕"</button>
                                </div>
                                <div class="modal-body">
                                    <dl class="doc-detail-list">
                                        <dt>"Category"</dt><dd>{label}</dd>
                                        <dt>"Version"</dt><dd>{doc.version_number.to_string()}</dd>
                                        <dt>"Added"</dt><dd>{doc.created_at.chars().take(10).collect::<String>()}</dd>
                                        <dt>"Signature Required"</dt><dd>{if doc.requires_signature { "Yes" } else { "No" }}</dd>
                                        <dt>"Signed"</dt><dd>{if doc.is_signed { "✓ Yes" } else { "✗ No" }}</dd>
                                        <dt>"Tenant Visible"</dt><dd>{if doc.is_counterparty_visible { "Yes" } else { "No" }}</dd>
                                        {doc.related_entity_type.clone().map(|et| view! {
                                            <dt>"Entity Type"</dt><dd>{et}</dd>
                                        })}
                                        {doc.related_entity_id.map(|eid| view! {
                                            <dt>"Linked record"</dt>
                                            <dd class="font-mono text-xs opacity-60">{eid.to_string()}</dd>
                                        })}
                                    </dl>
                                </div>
                                <div class="modal-footer">
                                    <button class="btn btn-ghost" on:click=move |_| selected_doc.set(None)>"Close"</button>
                                </div>
                            }
                        })}
                    </div>
                </div>
            </Show>

        </div>
    }
}
