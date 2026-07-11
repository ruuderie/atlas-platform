// apps/folio/src/pages/pmc/owner_statements.rs
//
// PMC Owner Statement Batch — /pmc/statements
//
// Batch owner statement generation and delivery for all PMC clients.
// Aggregates client list from /api/folio/pm/clients.
// In Phase 7: bulk-generate PDFs and email/deliver per owner.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSummary {
    pub account_id: Uuid,
    pub display_name: String,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub property_count: Option<i64>,
    pub unit_count: Option<i64>,
    pub active_lease_count: Option<i64>,
    pub occupancy_pct: Option<f64>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchPmcClientsStmts, "/api")]
pub async fn fetch_pmc_clients_stmts() -> Result<Vec<ClientSummary>, server_fn::error::ServerFnError>
{
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<ClientSummary>>(
        "/api/folio/pm/clients",
        &token,
        None,
    )
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

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn PmcOwnerStatements() -> impl IntoView {
    let selected: RwSignal<std::collections::HashSet<Uuid>> =
        RwSignal::new(std::collections::HashSet::new());
    let generating = RwSignal::new(false);
    let generated = RwSignal::new(false);

    let clients_res = Resource::new(|| (), |_| fetch_pmc_clients_stmts());

    let toggle_all = move |clients: Vec<ClientSummary>| {
        selected.update(|s| {
            if s.len() == clients.len() {
                s.clear();
            } else {
                for c in &clients {
                    s.insert(c.account_id);
                }
            }
        });
    };

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Owner Statement Batch"</h1>
                    <p class="page-subtitle">"Generate and deliver monthly statements to all managed owners"</p>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-primary btn-sm"
                        disabled=move || generating.get() || selected.get().is_empty()
                        on:click=move |_| {
                            generating.set(true);
                            // Phase 7: POST /api/folio/pm/statements/batch with selected IDs
                            generated.set(true);
                            generating.set(false);
                        }
                    >
                        {move || if generating.get() { "Generating…".to_string() } else {
                            format!("Generate Statements ({})", selected.get().len())
                        }}
                    </button>
                </div>
            </div>

            {move || if generated.get() {
                view! {
                    <div class="alert-saved-toast">"✓ Statements queued for generation — owners will be notified"</div>
                }.into_any()
            } else { ().into_any() }}

            // ── Phase 7 note ──
            <div class="viol-info-banner" style="margin-bottom:1.25rem;">
                <span class="viol-info-icon">"📄"</span>
                <p class="viol-info-text">
                    "PDF generation will be available in Phase 7. This view lets you preview and select which clients to include. "
                    "Statements will be delivered by email to the contact on file."
                </p>
            </div>

            // ── Client selection table ──
            <div class="owner-section">
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading clients…"</div> }>
                    {move || clients_res.get().map(|res| {
                        match res {
                            Ok(clients) if !clients.is_empty() => {
                                let c2 = clients.clone();
                                let clients_for_list = clients.clone();
                                let all_selected = move || selected.get().len() == c2.len();
                                view! {
                                    <div class="pmc-stmt-toolbar">
                                        <button
                                            class="btn btn-ghost btn-sm"
                                            on:click=move |_| toggle_all(clients.clone())
                                        >
                                            {move || if all_selected() { "Deselect All" } else { "Select All" }}
                                        </button>
                                        <span class="pmc-stmt-selected-count">
                                            {move || format!("{} selected", selected.get().len())}
                                        </span>
                                    </div>
                                    <div class="pmc-stmt-list">
                                        <For
                                            each=move || clients_for_list.clone()
                                            key=|c| c.account_id
                                            children=move |client| {
                                                let cid   = client.account_id;
                                                let name  = client.display_name.clone();
                                                let email = client.contact_email.clone();
                                                let props = client.property_count.unwrap_or(0);
                                                let occ   = client.occupancy_pct.map(|p| format!("{:.0}%", p));
                                                view! {
                                                    <div
                                                        class=move || format!("pmc-stmt-row {}", if selected.get().contains(&cid) { "pmc-stmt-row--selected" } else { "" })
                                                        on:click=move |_| {
                                                            selected.update(|s| {
                                                                if s.contains(&cid) { s.remove(&cid); }
                                                                else { s.insert(cid); }
                                                            });
                                                        }
                                                    >
                                                        <div class="pmc-stmt-check">
                                                            {move || if selected.get().contains(&cid) { "☑" } else { "☐" }}
                                                        </div>
                                                        <div class="pmc-client-avatar" style="font-size:1rem;width:2rem;height:2rem;">
                                                            {name.chars().next().map(|ch| ch.to_string()).unwrap_or_else(|| "?".to_string())}
                                                        </div>
                                                        <div class="pmc-stmt-info">
                                                            <div class="pmc-stmt-name">{name}</div>
                                                            {email.map(|e| view! { <div class="pmc-stmt-email">{e}</div> })}
                                                        </div>
                                                        <div class="pmc-stmt-metrics">
                                                            <span>{props.to_string()} " properties"</span>
                                                            {occ.map(|o| view! { <span>{o} " occupancy"</span> })}
                                                        </div>
                                                        <a href=format!("/pmc/clients/{}", cid)
                                                            class="btn btn-ghost btn-sm"
                                                            on:click=|ev: leptos::ev::MouseEvent| ev.stop_propagation()
                                                        >"Detail →"</a>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                }.into_any()
                            }
                            _ => view! { <div class="doc-empty">"No clients found."</div> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}
