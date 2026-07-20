// apps/folio/src/pages/landlord/wholesaling.rs
//
// Wholesaling — /l/wholesaling (legacy; prefer /l/deals?track=wholesale)
// Kanban + MAO calculator aligned to /api/folio/wholesale DTOs.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WholesaleSummary {
    pub id: Uuid,
    pub property_address: String,
    pub stage: String,
    pub status: Option<String>,
    pub arv_cents: Option<i64>,
    pub repair_cents: Option<i64>,
    pub offer_cents: Option<i64>,
    pub deal_amount_cents: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaoResult {
    pub mao_cents: i64,
    pub arv_cents: i64,
    pub repair_cents: i64,
    pub wholesale_fee_cents: i64,
    pub equity_cushion_pct: f64,
    pub is_viable: Option<bool>,
    pub currency: Option<String>,
}

#[server(FetchWholesaleLeads, "/api")]
pub async fn fetch_wholesale_leads(
) -> Result<Vec<WholesaleSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<WholesaleSummary>>(
        "/api/folio/wholesale",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(CalcMao, "/api")]
pub async fn calc_mao(
    arv: i64,
    repair: i64,
    fee: i64,
) -> Result<MaoResult, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let body = serde_json::json!({
        "arv_cents": arv,
        "repair_cents": repair,
        "wholesale_fee_cents": fee,
        "multiplier": "0.70"
    });
    crate::atlas_client::authenticated_post::<serde_json::Value, MaoResult>(
        "/api/folio/wholesale/mao",
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(CreateWholesaleLead, "/api")]
pub async fn create_wholesale_lead(
    address: String,
    arv_cents: i64,
    repair_cents: i64,
    seller_motivation: String,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let body = serde_json::json!({
        "address": address,
        "arv_cents": arv_cents,
        "repair_cents": repair_cents,
        "seller_motivation": seller_motivation,
    });
    #[derive(Deserialize)]
    struct Resp {
        id: Uuid,
    }
    let resp = crate::atlas_client::authenticated_post::<serde_json::Value, Resp>(
        "/api/folio/wholesale",
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(resp.id)
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

fn fmt_k(cents: i64) -> String {
    let k = cents as f64 / 100_000.0;
    if k >= 1.0 {
        format!("${:.0}k", k)
    } else {
        format!("${}", cents / 100)
    }
}

/// Kanban columns use canonical wholesale acquire stages.
const STAGES: &[(&str, &str)] = &[
    ("new", "New"),
    ("prescreened", "Prescreened"),
    ("offer_out", "Offer out"),
    ("under_contract", "Under contract"),
    ("title_clear", "Title clear"),
    ("marketing", "Marketing"),
    ("assigned_or_closed", "Closed"),
    ("dead", "Dead"),
];

fn stage_color(stage: &str) -> &'static str {
    match stage {
        "new" => "#60a5fa",
        "prescreened" | "qualified" => "#38bdf8",
        "offer_out" | "negotiating" => "#fbbf24",
        "under_contract" => "#a78bfa",
        "title_clear" => "#c084fc",
        "marketing" => "#fb923c",
        "assigned_or_closed" | "closed" => "#4ade80",
        "dead" | "converted_to_cf" => "#94a3b8",
        _ => "#94a3b8",
    }
}

fn normalize_stage(s: &str) -> String {
    match s {
        "lead" | "new" => "new".into(),
        "qualified" => "prescreened".into(),
        "negotiating" => "offer_out".into(),
        "closed" => "assigned_or_closed".into(),
        other => other.to_string(),
    }
}

#[component]
pub fn LandlordWholesaling() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let show_calc = RwSignal::new(false);
    let show_add = RwSignal::new(false);
    let calc_arv = RwSignal::new(String::new());
    let calc_repair = RwSignal::new(String::new());
    let calc_fee = RwSignal::new(String::from("5000"));
    let calc_result = RwSignal::new(None::<MaoResult>);
    let calculating = RwSignal::new(false);
    let new_addr = RwSignal::new(String::new());
    let new_arv = RwSignal::new(String::new());
    let new_repair = RwSignal::new(String::new());
    let creating = RwSignal::new(false);

    let leads_res = Resource::new(move || refresh.get(), |_| fetch_wholesale_leads());

    let handle_calc = move |_: leptos::ev::MouseEvent| {
        let arv = (calc_arv.get().parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
        let repair = (calc_repair.get().parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
        let fee = (calc_fee.get().parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
        if arv == 0 {
            return;
        }
        calculating.set(true);
        spawn_local(async move {
            if let Ok(res) = calc_mao(arv, repair, fee).await {
                calc_result.set(Some(res));
            }
            calculating.set(false);
        });
    };

    let handle_create = move |_: leptos::ev::MouseEvent| {
        let address = new_addr.get();
        if address.trim().is_empty() {
            return;
        }
        let arv = (new_arv.get().parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
        let repair = (new_repair.get().parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
        creating.set(true);
        spawn_local(async move {
            if create_wholesale_lead(address, arv, repair, "other".into())
                .await
                .is_ok()
            {
                show_add.set(false);
                refresh.update(|n| *n += 1);
            }
            creating.set(false);
        });
    };

    view! {
        <div class="main-area">
            <PageHeader
                title=Signal::derive(|| "Wholesaling".to_string())
                subtitle=Signal::derive(|| "Cash-offer leads".to_string())
            >
                <a
                    class="folio-btn folio-btn--ghost folio-btn--sm"
                    href=format!("{}?track=wholesale", FolioRoute::LandlordDeals.path())
                >
                    "Open Deal Ops →"
                </a>
                <button class="folio-btn folio-btn--ghost folio-btn--sm" on:click=move |_| show_calc.set(true)>
                    "MAO Calculator"
                </button>
                <button class="folio-btn folio-btn--primary folio-btn--sm" on:click=move |_| show_add.set(true)>
                    "+ Add Lead"
                </button>
            </PageHeader>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading pipeline…"</div> }>
                {move || leads_res.get().map(|res| {
                    match res {
                        Ok(leads) => view! {
                            <div class="wholesale-kanban">
                                {STAGES.iter().map(|(stage_id, stage_label)| {
                                    let stage_leads: Vec<_> = leads.iter().filter(|l| {
                                        normalize_stage(&l.stage) == *stage_id
                                    }).cloned().collect();
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
                                                    view! { <div class="wholesale-empty">"—"</div> }.into_any()
                                                } else {
                                                    view! {
                                                        <For
                                                            each=move || stage_leads.clone()
                                                            key=|l| l.id
                                                            children=move |lead| {
                                                                let addr = lead.property_address.clone();
                                                                let arv = lead.arv_cents.map(fmt_k).unwrap_or_else(|| "—".to_string());
                                                                let offer = lead.offer_cents.or(lead.deal_amount_cents).map(fmt_k).unwrap_or_else(|| "—".to_string());
                                                                let date = lead.created_at.chars().take(10).collect::<String>();
                                                                let href = FolioRoute::LandlordDealDetail
                                                                    .path()
                                                                    .replace(":id", &lead.id.to_string());
                                                                view! {
                                                                    <a class="wholesale-card" href=href>
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
                                                                    </a>
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

            <Show when=move || show_calc.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"MAO Calculator"</h3>
                            <button class="modal-close" on:click=move |_| { show_calc.set(false); calc_result.set(None); }>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="folio-field">
                                <label class="folio-field__label">"ARV ($)"</label>
                                <input type="number" class="folio-input" prop:value=calc_arv
                                    on:input=move |ev| calc_arv.set(event_target_value(&ev)) />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Repairs ($)"</label>
                                <input type="number" class="folio-input" prop:value=calc_repair
                                    on:input=move |ev| calc_repair.set(event_target_value(&ev)) />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Wholesale fee ($)"</label>
                                <input type="number" class="folio-input" prop:value=calc_fee
                                    on:input=move |ev| calc_fee.set(event_target_value(&ev)) />
                            </div>
                            {move || calc_result.get().map(|r| view! {
                                <div class="mao-result-card">
                                    <div class="mao-result-label">"Maximum Allowable Offer"</div>
                                    <div class="mao-result-value">{fmt_k(r.mao_cents)}</div>
                                    <div class="mao-result-detail">
                                        "Equity cushion: " <strong>{format!("{:.1}%", r.equity_cushion_pct)}</strong>
                                    </div>
                                </div>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button class="folio-btn folio-btn--ghost" on:click=move |_| { show_calc.set(false); calc_result.set(None); }>"Close"</button>
                            <button class="folio-btn folio-btn--primary" on:click=handle_calc disabled=move || calculating.get()>
                                {move || if calculating.get() { "Calculating…" } else { "Calculate MAO" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || show_add.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"New wholesale lead"</h3>
                            <button class="modal-close" on:click=move |_| show_add.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="folio-field">
                                <label class="folio-field__label">"Address"</label>
                                <input class="folio-input" prop:value=new_addr
                                    on:input=move |ev| new_addr.set(event_target_value(&ev)) />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"ARV ($)"</label>
                                <input type="number" class="folio-input" prop:value=new_arv
                                    on:input=move |ev| new_arv.set(event_target_value(&ev)) />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Repairs ($)"</label>
                                <input type="number" class="folio-input" prop:value=new_repair
                                    on:input=move |ev| new_repair.set(event_target_value(&ev)) />
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="folio-btn folio-btn--ghost" on:click=move |_| show_add.set(false)>"Cancel"</button>
                            <button class="folio-btn folio-btn--primary" on:click=handle_create disabled=move || creating.get()>
                                {move || if creating.get() { "Saving…" } else { "Create" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
