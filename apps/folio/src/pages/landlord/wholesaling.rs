// apps/folio/src/pages/landlord/wholesaling.rs
//
// Wholesaling — /l/wholesaling
//
// Kanban board for wholesale opportunity tracking.
// Uses /api/folio/wholesale to list leads + advance stage.
// Also exposes the stateless MAO calculator via /api/folio/wholesale/mao.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos::task::spawn_local;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WholesaleSummary {
    pub id:             Uuid,
    pub property_address: String,
    pub stage:          String,  // lead | negotiating | under_contract | closed | dead
    pub arv_cents:      Option<i64>,
    pub repair_cents:   Option<i64>,
    pub offer_cents:    Option<i64>,
    pub created_at:     String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaoResult {
    pub mao_cents:         i64,
    pub arv_cents:         i64,
    pub repair_cents:      i64,
    pub wholesale_fee_cents: i64,
    pub equity_cushion_pct: f64,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchWholesaleLeads, "/api")]
pub async fn fetch_wholesale_leads() -> Result<Vec<WholesaleSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<WholesaleSummary>>(
        "/api/folio/wholesale", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(CalcMao, "/api")]
pub async fn calc_mao(arv: i64, repair: i64, fee: i64) -> Result<MaoResult, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let body = serde_json::json!({
        "arv_cents": arv,
        "repair_cents": repair,
        "wholesale_fee_cents": fee,
        "multiplier": 0.7
    });
    crate::atlas_client::authenticated_post::<MaoResult, serde_json::Value>(
        "/api/folio/wholesale/mao", &token, None, &body,
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

fn fmt_k(cents: i64) -> String {
    let k = cents as f64 / 100_000.0;
    if k >= 1.0 { format!("${:.0}k", k) } else { format!("${}", cents / 100) }
}

const STAGES: &[(&str, &str)] = &[
    ("lead",           "📋 Lead"),
    ("negotiating",    "🤝 Negotiating"),
    ("under_contract", "📝 Under Contract"),
    ("closed",         "✅ Closed"),
    ("dead",           "💀 Dead"),
];

fn stage_color(stage: &str) -> &'static str {
    match stage {
        "lead"           => "#60a5fa",
        "negotiating"    => "#fbbf24",
        "under_contract" => "#a78bfa",
        "closed"         => "#4ade80",
        "dead"           => "#94a3b8",
        _                => "#94a3b8",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn LandlordWholesaling() -> impl IntoView {
    let refresh     = RwSignal::new(0u32);
    let show_calc   = RwSignal::new(false);
    let calc_arv    = RwSignal::new(String::new());
    let calc_repair = RwSignal::new(String::new());
    let calc_fee    = RwSignal::new(String::new());
    let calc_result = RwSignal::new(None::<MaoResult>);
    let calculating = RwSignal::new(false);

    let leads_res = Resource::new(
        move || refresh.get(),
        |_| fetch_wholesale_leads(),
    );

    let handle_calc = move |_: leptos::ev::MouseEvent| {
        let arv    = calc_arv.get().replace(['$', ',', 'k', ' '], "").parse::<f64>().unwrap_or(0.0) as i64 * 100;
        let repair = calc_repair.get().replace(['$', ',', 'k', ' '], "").parse::<f64>().unwrap_or(0.0) as i64 * 100;
        let fee    = calc_fee.get().replace(['$', ',', 'k', ' '], "").parse::<f64>().unwrap_or(0.0) as i64 * 100;
        if arv == 0 { return; }
        calculating.set(true);
        spawn_local(async move {
            if let Ok(res) = calc_mao(arv, repair, fee).await {
                calc_result.set(Some(res));
            }
            calculating.set(false);
        });
    };

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <h1 class="page-title">"Wholesaling"</h1>
                    <p class="page-subtitle">"Off-market deal pipeline and MAO analysis"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| show_calc.set(true)>
                        "🧮 MAO Calculator"
                    </button>
                    <button class="btn btn-primary btn-sm" disabled=true>"+ Add Lead"</button>
                </div>
            </div>

            // ── Kanban ──
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading pipeline…"</div> }>
                {move || leads_res.get().map(|res| {
                    match res {
                        Ok(leads) => view! {
                            <div class="wholesale-kanban">
                                {STAGES.iter().map(|(stage_id, stage_label)| {
                                    let stage_leads: Vec<_> = leads.iter().filter(|l| &l.stage.as_str() == stage_id).cloned().collect();
                                    let count = stage_leads.len();
                                    let color = stage_color(stage_id);
                                    let label = *stage_label;
                                    view! {
                                        <div class="wholesale-column">
                                            <div class="wholesale-col-header" style=format!("border-top:3px solid {color}")>
                                                <span class="wholesale-col-title">{label}</span>
                                                <span class="wholesale-col-count">{count.to_string()}</span>
                                            </div>
                                            <div class="wholesale-col-body">
                                                {if stage_leads.is_empty() {
                                                    view! {
                                                        <div class="wholesale-empty">"—"</div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <For
                                                            each=move || stage_leads.clone()
                                                            key=|l| l.id
                                                            children=move |lead| {
                                                                let addr  = lead.property_address.clone();
                                                                let arv   = lead.arv_cents.map(fmt_k).unwrap_or_else(|| "—".to_string());
                                                                let offer = lead.offer_cents.map(fmt_k).unwrap_or_else(|| "—".to_string());
                                                                let date  = lead.created_at.chars().take(10).collect::<String>();
                                                                view! {
                                                                    <div class="wholesale-card">
                                                                        <div class="wholesale-card-addr">{addr}</div>
                                                                        <div class="wholesale-card-kv">
                                                                            <span class="wholesale-card-k">"ARV"</span>
                                                                            <span class="wholesale-card-v">{arv}</span>
                                                                        </div>
                                                                        <div class="wholesale-card-kv">
                                                                            <span class="wholesale-card-k">"Offer"</span>
                                                                            <span class="wholesale-card-v">{offer}</span>
                                                                        </div>
                                                                        <div class="wholesale-card-date">{date}</div>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                    }.into_any()
                                                }}
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any(),
                        Err(_) => view! {
                            <div class="doc-empty">"Could not load pipeline."</div>
                        }.into_any(),
                    }
                })}
            </Suspense>

            // ── MAO Calculator Modal ─────────────────────────────────────────
            <Show when=move || show_calc.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"🧮 MAO Calculator"</h3>
                            <button class="modal-close" on:click=move |_| { show_calc.set(false); calc_result.set(None); }>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="form-field">
                                <label class="form-label">"ARV — After Repair Value ($)"</label>
                                <input type="number" class="form-input" placeholder="250000"
                                    prop:value=calc_arv
                                    on:input=move |ev| calc_arv.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Estimated Repairs ($)"</label>
                                <input type="number" class="form-input" placeholder="30000"
                                    prop:value=calc_repair
                                    on:input=move |ev| calc_repair.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Wholesale Fee ($)"</label>
                                <input type="number" class="form-input" placeholder="10000"
                                    prop:value=calc_fee
                                    on:input=move |ev| calc_fee.set(event_target_value(&ev)) />
                            </div>

                            {move || calc_result.get().map(|r| view! {
                                <div class="mao-result-card">
                                    <div class="mao-result-label">"Maximum Allowable Offer (MAO)"</div>
                                    <div class="mao-result-value">{fmt_k(r.mao_cents)}</div>
                                    <div class="mao-result-detail">
                                        "Equity cushion: " <strong>{format!("{:.1}%", r.equity_cushion_pct)}</strong>
                                    </div>
                                </div>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| { show_calc.set(false); calc_result.set(None); }>"Close"</button>
                            <button
                                class="btn btn-primary"
                                on:click=handle_calc
                                disabled=move || calculating.get()
                            >
                                {move || if calculating.get() { "Calculating…" } else { "Calculate MAO" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

        </div>
    }
}
