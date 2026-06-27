// apps/folio/src/pages/tenant/documents.rs
//
// Tenant Documents — /t/docs
//
// Lists all documents in the tenant's vault (atlas_documents via G-14).
// Allows viewing document details and category filtering.
//
// Endpoints:
//   GET /api/folio/vault/documents — List all tenant documents
//   GET /api/folio/vault/documents/{id} — Document detail
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub id:                      Uuid,
    pub document_category:       String,
    pub related_entity_type:     Option<String>,
    pub related_entity_id:       Option<Uuid>,
    pub is_counterparty_visible: bool,
    pub requires_signature:      bool,
    pub is_signed:               bool,
    pub version_number:          i32,
    pub created_at:              String,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchTenantDocs, "/api")]
pub async fn fetch_tenant_docs() -> Result<Vec<DocumentSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<DocumentSummary>>(
        "/api/folio/vault/documents", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[cfg(feature = "ssr")]
fn session_token(headers: &axum::http::HeaderMap) -> Result<String, server_fn::error::ServerFnError> {
    headers.get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find_map(|p| {
            let p = p.trim();
            p.strip_prefix("session=").map(|t| t.to_string())
        }))
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn doc_icon(category: &str) -> &'static str {
    match category.to_lowercase().as_str() {
        c if c.contains("lease") || c.contains("agreement") => "📋",
        c if c.contains("id")  || c.contains("passport")   => "🪪",
        c if c.contains("permit") || c.contains("str")     => "📜",
        c if c.contains("insurance")                        => "🛡",
        c if c.contains("notice")                           => "📣",
        c if c.contains("inspection")                       => "🔍",
        c if c.contains("receipt") || c.contains("invoice")=> "🧾",
        _                                                   => "📄",
    }
}

fn doc_label(category: &str) -> String {
    category.replace('_', " ")
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

fn needs_action(doc: &DocumentSummary) -> bool {
    doc.requires_signature && !doc.is_signed
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantDocuments() -> impl IntoView {
    let refresh        = RwSignal::new(0u32);
    let filter_cat     = RwSignal::new("all".to_string());
    let selected_doc   = RwSignal::new(None::<DocumentSummary>);
    let show_detail    = RwSignal::new(false);

    let docs_res = Resource::new(
        move || refresh.get(),
        |_| fetch_tenant_docs(),
    );

    view! {
        <div class="main-area">

            // ── Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Documents"</h1>
                    <p class="page-subtitle">"Your lease agreements, notices, permits, and shared files"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>
                        "↻ Refresh"
                    </button>
                </div>
            </div>

            // ── Action required banner ──
            <Suspense fallback=|| ()>
                {move || docs_res.get().map(|res| {
                    let action_count = res.as_ref().ok()
                        .map(|docs| docs.iter().filter(|d| needs_action(d)).count())
                        .unwrap_or(0);
                    if action_count > 0 {
                        view! {
                            <div class="doc-action-banner">
                                <span class="doc-action-icon">"✍️"</span>
                                <span>
                                    "You have "
                                    <strong>{action_count.to_string()}</strong>
                                    " document(s) awaiting your signature."
                                </span>
                                <button class="doc-action-btn" on:click=move |_| filter_cat.set("pending_signature".to_string())>
                                    "View →"
                                </button>
                            </div>
                        }.into_any()
                    } else { ().into_any() }
                })}
            </Suspense>

            // ── Stats ──
            <Suspense fallback=|| ()>
                {move || docs_res.get().map(|res| {
                    match res.as_ref() {
                        Ok(docs) => {
                            let total    = docs.len();
                            let signed   = docs.iter().filter(|d| d.is_signed).count();
                            let unsigned = docs.iter().filter(|d| d.requires_signature && !d.is_signed).count();
                            view! {
                                <div class="kpi-row" style="margin-bottom:1.25rem;">
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Total Documents"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">{total.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Signed"</span>
                                        <span class="kpi-value" style="color:var(--green)">{signed.to_string()}</span>
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
                    let pill = move |scope: &'static str, label: &'static str| {
                        view! {
                            <button
                                class=move || format!("filter-pill {}", if filter_cat.get() == scope { "filter-pill--active" } else { "" })
                                on:click=move |_| filter_cat.set(scope.to_string())
                            >{label}</button>
                        }
                    };
                    view! {
                        {pill("all",               "All")}
                        {pill("lease_agreement",   "Leases")}
                        {pill("notice",            "Notices")}
                        {pill("id_document",       "ID Docs")}
                        {pill("pending_signature", "Needs Signature")}
                    }
                }
            </div>

            // ── Document grid ──
            <Suspense fallback=|| view! {
                <div class="doc-empty">"Loading documents…"</div>
            }>
                {move || docs_res.get().map(|res| {
                    match res {
                        Ok(docs) => {
                            let cf = filter_cat.get();
                            let visible: Vec<_> = docs.into_iter().filter(|d| {
                                match cf.as_str() {
                                    "all" => true,
                                    "pending_signature" => d.requires_signature && !d.is_signed,
                                    cat   => d.document_category.contains(cat),
                                }
                            }).collect();

                            if visible.is_empty() {
                                return view! {
                                    <div class="doc-empty">"No documents found for this filter."</div>
                                }.into_any();
                            }

                            view! {
                                <div class="doc-grid">
                                    <For
                                        each=move || visible.clone()
                                        key=|d| d.id
                                        children=move |doc| {
                                            let doc_for_click = doc.clone();
                                            let icon    = doc_icon(&doc.document_category);
                                            let label   = doc_label(&doc.document_category);
                                            let date    = doc.created_at.chars().take(10).collect::<String>();
                                            let ver     = doc.version_number;
                                            let signed  = doc.is_signed;
                                            let req_sig = doc.requires_signature;
                                            let visible_to_me = doc.is_counterparty_visible;

                                            view! {
                                                <div
                                                    class="doc-card"
                                                    on:click=move |_| {
                                                        selected_doc.set(Some(doc_for_click.clone()));
                                                        show_detail.set(true);
                                                    }
                                                >
                                                    <div class="doc-card-icon">{icon}</div>
                                                    <div class="doc-card-body">
                                                        <div class="doc-card-title">{label}</div>
                                                        <div class="doc-card-meta">"Added " {date}</div>
                                                        <div class="doc-card-meta">"Version " {ver.to_string()}</div>
                                                    </div>
                                                    <div class="doc-card-badges">
                                                        {if req_sig && !signed {
                                                            view! { <span class="doc-badge doc-badge--action">"Needs Signature"</span> }.into_any()
                                                        } else if req_sig && signed {
                                                            view! { <span class="doc-badge doc-badge--signed">"✓ Signed"</span> }.into_any()
                                                        } else {
                                                            ().into_any()
                                                        }}
                                                        {if visible_to_me {
                                                            view! { <span class="doc-badge doc-badge--shared">"Shared"</span> }.into_any()
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
                            <div class="doc-empty text-red-400">"Error loading documents: " {e.to_string()}</div>
                        }.into_any(),
                    }
                })}
            </Suspense>

            // ── Document detail modal ────────────────────────────────────────
            <Show when=move || show_detail.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:32rem;">
                        {move || selected_doc.get().map(|doc| {
                            let icon   = doc_icon(&doc.document_category);
                            let label  = doc_label(&doc.document_category);
                            let date   = doc.created_at.chars().take(10).collect::<String>();

                            view! {
                                <div class="modal-header">
                                    <h3 class="modal-title">{icon} " " {label.clone()}</h3>
                                    <button class="modal-close" on:click=move |_| show_detail.set(false)>"✕"</button>
                                </div>
                                <div class="modal-body">
                                    <dl class="doc-detail-list">
                                        <dt>"Category"</dt><dd>{label}</dd>
                                        <dt>"Version"</dt><dd>{doc.version_number.to_string()}</dd>
                                        <dt>"Added"</dt><dd>{date}</dd>
                                        <dt>"Signature required"</dt>
                                        <dd>{if doc.requires_signature { "Yes" } else { "No" }}</dd>
                                        <dt>"Signed"</dt>
                                        <dd>{if doc.is_signed { "✓ Yes" } else { "✗ Not yet" }}</dd>
                                        <dt>"Shared with me"</dt>
                                        <dd>{if doc.is_counterparty_visible { "Yes" } else { "No" }}</dd>
                                        {doc.related_entity_type.clone().map(|et| view! {
                                            <dt>"Related to"</dt><dd>{et}</dd>
                                        })}
                                    </dl>

                                    {if doc.requires_signature && !doc.is_signed {
                                        view! {
                                            <div class="doc-sign-prompt">
                                                <p>"This document requires your electronic signature. Please contact your property manager to complete the signing process."</p>
                                            </div>
                                        }.into_any()
                                    } else { ().into_any() }}
                                </div>
                                <div class="modal-footer">
                                    <button class="btn btn-ghost" on:click=move |_| show_detail.set(false)>"Close"</button>
                                </div>
                            }
                        })}
                    </div>
                </div>
            </Show>

        </div>
    }
}
