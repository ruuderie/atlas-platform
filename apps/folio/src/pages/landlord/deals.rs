//! Deal Ops hub — `/l/deals?track=wholesale|creative_finance`
//! Dual Acquire / Disposition pipeline over `/api/folio/deals`.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealSummary {
    pub id: Uuid,
    pub track: String,
    pub opportunity_type: String,
    pub name: String,
    pub status: String,
    pub property_address: String,
    pub arv_cents: Option<i64>,
    pub repair_cents: Option<i64>,
    pub offer_cents: Option<i64>,
    pub deal_amount_cents: Option<i64>,
    pub acquisition_structure: Option<String>,
    pub exit_mode: Option<String>,
    pub cya_required: Option<bool>,
    pub cya_signed: Option<bool>,
    pub title_clear: Option<bool>,
    pub asset_id: Option<Uuid>,
    pub currency: String,
    pub created_at: String,
}

#[server(FetchDeals, "/api")]
pub async fn fetch_deals(
    track: Option<String>,
) -> Result<Vec<DealSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let path = match track.as_deref() {
        Some(t) => format!("/api/folio/deals?track={t}"),
        None => "/api/folio/deals".to_string(),
    };
    crate::atlas_client::authenticated_get::<Vec<DealSummary>>(&path, &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(CreateDeal, "/api")]
pub async fn create_deal(
    track: String,
    address: String,
    arv_cents: i64,
    repair_cents: i64,
    as_buyer: bool,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let body = serde_json::json!({
        "track": track,
        "address": address,
        "arv_cents": arv_cents,
        "repair_cents": repair_cents,
        "as_buyer": as_buyer,
        "seller_motivation": "other",
    });
    #[derive(Deserialize)]
    struct Resp { id: Uuid }
    let r = crate::atlas_client::authenticated_post::<serde_json::Value, Resp>(
        "/api/folio/deals",
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(r.id)
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

fn fmt_money(cents: Option<i64>) -> String {
    cents
        .map(|c| format!("${}", c / 100))
        .unwrap_or_else(|| "—".into())
}

fn wholesale_acquire_cols() -> &'static [(&'static str, &'static str)] {
    &[
        ("new", "New"),
        ("prescreened", "Prescreened"),
        ("offer_out", "Offer"),
        ("under_contract", "Contract"),
        ("title_clear", "Title"),
        ("marketing", "Marketing"),
        ("assigned_or_closed", "Closed"),
    ]
}

fn cf_acquire_cols() -> &'static [(&'static str, &'static str)] {
    &[
        ("new", "New"),
        ("prescreened", "Prescreened"),
        ("offer_structured", "Structured"),
        ("cya_closing", "CYA"),
        ("owned_or_optioned", "Owned"),
    ]
}

fn dispose_cols(track: &str) -> &'static [(&'static str, &'static str)] {
    if track == "creative_finance" {
        &[
            ("buyer_lead", "Leads"),
            ("prescreen_pass", "Pass"),
            ("ara_deposit", "ARA"),
            ("installed", "Installed"),
            ("exercise_cashout", "Cash-out"),
        ]
    } else {
        &[
            ("buyer_lead", "Leads"),
            ("prescreen_pass", "Pass"),
            ("deposit_held", "Deposit"),
            ("assigned", "Assigned"),
            ("closed", "Closed"),
        ]
    }
}

fn is_acquire(opp_type: &str) -> bool {
    matches!(
        opp_type,
        "wholesale_lead" | "creative_finance_acquisition"
    )
}

#[component]
pub fn LandlordDeals() -> impl IntoView {
    let query = use_query_map();
    let track = Memo::new(move |_| {
        query
            .get()
            .get("track")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "wholesale".to_string())
    });
    let mode = RwSignal::new("acquire".to_string());
    let refresh = RwSignal::new(0u32);
    let show_add = RwSignal::new(false);
    let addr = RwSignal::new(String::new());
    let arv = RwSignal::new(String::new());
    let repair = RwSignal::new(String::new());
    let creating = RwSignal::new(false);

    let deals = Resource::new(
        move || (track.get(), refresh.get()),
        |(t, _)| fetch_deals(Some(t)),
    );

    let on_create = move |_| {
        let t = track.get();
        let as_buyer = mode.get() == "dispose";
        let address = addr.get();
        if address.trim().is_empty() {
            return;
        }
        let a = (arv.get().parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
        let r = (repair.get().parse::<f64>().unwrap_or(0.0) * 100.0) as i64;
        creating.set(true);
        spawn_local(async move {
            if create_deal(t, address, a, r, as_buyer).await.is_ok() {
                show_add.set(false);
                refresh.update(|n| *n += 1);
            }
            creating.set(false);
        });
    };

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Deal Ops"</h1>
                    <p class="page-subtitle">"Wholesaling and creative finance deals"</p>
                </div>
                <div class="page-actions">
                    <a class="folio-btn folio-btn--ghost press" href=FolioRoute::LandlordBuyers.path()>"Buyers"</a>
                    <button class="folio-btn folio-btn--primary press" on:click=move |_| show_add.set(true)>"+ New"</button>
                </div>
            </div>
            <nav class="folio-related" aria-label="Related">
                <span class="folio-related__label">"Related"</span>
                <ul class="folio-related__list">
                    <li><a class="folio-related__link press" href=FolioRoute::LandlordBuyers.path()>"Buyers"</a></li>
                    <li><a class="folio-related__link press" href="/l/deals?track=wholesale">"Wholesaling"</a></li>
                </ul>
            </nav>

            <div class="flex gap-2 mb-4">
                <a
                    class=move || if track.get() == "wholesale" { "btn btn-primary btn-sm" } else { "btn btn-ghost btn-sm" }
                    href=format!("{}?track=wholesale", FolioRoute::LandlordDeals.path())
                >"Wholesale"</a>
                <a
                    class=move || if track.get() == "creative_finance" { "btn btn-primary btn-sm" } else { "btn btn-ghost btn-sm" }
                    href=format!("{}?track=creative_finance", FolioRoute::LandlordDeals.path())
                >"Creative Finance"</a>
            </div>

            <div class="flex gap-2 mb-4 border-b pb-2">
                <button
                    class=move || if mode.get() == "acquire" { "btn btn-primary btn-sm" } else { "btn btn-ghost btn-sm" }
                    on:click=move |_| mode.set("acquire".into())
                >"Acquire"</button>
                <button
                    class=move || if mode.get() == "dispose" { "btn btn-primary btn-sm" } else { "btn btn-ghost btn-sm" }
                    on:click=move |_| mode.set("dispose".into())
                >"Disposition"</button>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading deals…"</div> }>
                {move || deals.get().map(|res| {
                    match res {
                        Ok(all) => {
                            let t = track.get();
                            let m = mode.get();
                            let filtered: Vec<_> = all.into_iter().filter(|d| {
                                if m == "acquire" { is_acquire(&d.opportunity_type) } else { !is_acquire(&d.opportunity_type) }
                            }).collect();
                            let cols = if m == "acquire" {
                                if t == "creative_finance" { cf_acquire_cols() } else { wholesale_acquire_cols() }
                            } else {
                                dispose_cols(&t)
                            };
                            view! {
                                <div class="wholesale-kanban">
                                    {cols.iter().map(|(sid, label)| {
                                        let col: Vec<_> = filtered.iter().filter(|d| d.status == *sid).cloned().collect();
                                        let count = col.len();
                                        let lab = *label;
                                        view! {
                                            <div class="wholesale-column">
                                                <div class="wholesale-col-header">
                                                    <span class="wholesale-col-title">{lab}</span>
                                                    <span class="wholesale-col-count">{count.to_string()}</span>
                                                </div>
                                                <div class="wholesale-col-body">
                                                    {if col.is_empty() {
                                                        view! { <div class="wholesale-empty">"—"</div> }.into_any()
                                                    } else {
                                                        view! {
                                                            <For
                                                                each=move || col.clone()
                                                                key=|d| d.id
                                                                children=move |d| {
                                                                    let href = FolioRoute::LandlordDealDetail
                                                                        .path()
                                                                        .replace(":id", &d.id.to_string());
                                                                    let addr = d.property_address.clone();
                                                                    let offer = fmt_money(d.offer_cents.or(d.deal_amount_cents));
                                                                    let arv = fmt_money(d.arv_cents);
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
                            }.into_any()
                        }
                        Err(_) => view! { <div class="doc-empty">"Could not load deals."</div> }.into_any(),
                    }
                })}
            </Suspense>

            <Show when=move || show_add.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">{move || if mode.get() == "dispose" { "New buyer lead" } else { "New acquisition" }}</h3>
                            <button class="modal-close" on:click=move |_| show_add.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="form-field">
                                <label class="form-label">"Address / name"</label>
                                <input class="form-input" prop:value=addr on:input=move |ev| addr.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"ARV ($)"</label>
                                <input type="number" class="form-input" prop:value=arv on:input=move |ev| arv.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Repairs ($)"</label>
                                <input type="number" class="form-input" prop:value=repair on:input=move |ev| repair.set(event_target_value(&ev)) />
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add.set(false)>"Cancel"</button>
                            <button class="btn btn-primary" on:click=on_create disabled=move || creating.get()>
                                {move || if creating.get() { "Saving…" } else { "Create" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
