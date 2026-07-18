// apps/folio/src/pages/landlord/digital_vault.rs
//
// Landlord Digital Vault — /l/vault
//
// Manages all documents in the landlord's vault (lease agreements, permits,
// certificates, inspection reports, etc.). Reuses /api/folio/vault/documents.
// ─────────────────────────────────────────────────────────────────────────────

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

// ── Server functions ──────────────────────────────────────────────────────────

#[server(LLFetchVaultDocs, "/api")]
pub async fn ll_fetch_vault_docs(
    entity_type: Option<String>,
) -> Result<Vec<DocumentSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let url = match entity_type {
        Some(et) => format!("/api/folio/vault/documents?entity_type={et}"),
        None => "/api/folio/vault/documents".to_string(),
    };
    crate::atlas_client::authenticated_get::<Vec<DocumentSummary>>(&url, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[cfg(feature = "ssr")]
fn session_token(
    headers: &axum::http::HeaderMap,
) -> Result<String, server_fn::error::ServerFnError> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            })
        })
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
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
    let refresh = RwSignal::new(0u32);
    let cat_filter = RwSignal::new("all".to_string());
    let selected_doc = RwSignal::new(None::<DocumentSummary>);

    let docs_res = Resource::new(move || refresh.get(), |_| ll_fetch_vault_docs(None));

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <h1 class="page-title">"Digital Vault"</h1>
                    <p class="page-subtitle">"Leases, permits, certificates, and shared files"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>"↻ Refresh"</button>
                    <button class="btn btn-primary btn-sm" disabled=true title="Upload via R2 presign (Phase 7)">
                        "+ Upload Document"
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
                                            <dt>"Entity ID"</dt>
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
