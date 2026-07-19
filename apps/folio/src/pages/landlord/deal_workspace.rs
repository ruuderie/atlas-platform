//! Deal workspace — `/l/deals/:id`
//! CYA / title checklist, economics, convert / assign actions.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use serde::Deserialize;
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::pages::landlord::deals::fetch_deals;

#[derive(Debug, Deserialize)]
struct IdResp {
    id: Uuid,
}

#[derive(Debug, Deserialize)]
struct ConvertResp {
    asset_id: Uuid,
    contract_id: Uuid,
}

#[server(PostDealAction, "/api")]
pub async fn post_deal_action(
    id: Uuid,
    action: String,
    body: serde_json::Value,
) -> Result<serde_json::Value, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let path = format!("/api/folio/deals/{id}/{action}");
    crate::atlas_client::authenticated_post::<serde_json::Value, serde_json::Value>(
        &path, &token, None, &body,
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

fn is_acquire(opp_type: &str) -> bool {
    matches!(
        opp_type,
        "wholesale_lead" | "creative_finance_acquisition"
    )
}

/// Next pipeline stage for this deal's track / opportunity type.
fn next_stage(track: &str, opportunity_type: &str, current: &str) -> Option<&'static str> {
    let cols: &[&str] = if is_acquire(opportunity_type) {
        if track == "creative_finance" {
            &[
                "new",
                "prescreened",
                "offer_structured",
                "cya_closing",
                "owned_or_optioned",
            ]
        } else {
            &[
                "new",
                "prescreened",
                "offer_out",
                "under_contract",
                "title_clear",
                "marketing",
                "assigned_or_closed",
            ]
        }
    } else if track == "creative_finance" {
        &[
            "buyer_lead",
            "prescreen_pass",
            "ara_deposit",
            "installed",
            "exercise_cashout",
        ]
    } else {
        &[
            "buyer_lead",
            "prescreen_pass",
            "deposit_held",
            "assigned",
            "closed",
        ]
    };
    let idx = cols.iter().position(|s| *s == current)?;
    cols.get(idx + 1).copied()
}

#[component]
pub fn DealWorkspace() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| {
        params
            .get()
            .get("id")
            .and_then(|s| Uuid::parse_str(s.as_str()).ok())
    });
    let refresh = RwSignal::new(0u32);
    let msg = RwSignal::new(String::new());

    let deal = Resource::new(
        move || (id.get(), refresh.get()),
        |(maybe_id, _)| async move {
            let Some(deal_id) = maybe_id else {
                return Err(server_fn::error::ServerFnError::new("missing id"));
            };
            let all = fetch_deals(None).await?;
            all.into_iter()
                .find(|d| d.id == deal_id)
                .ok_or_else(|| server_fn::error::ServerFnError::new("deal not found"))
        },
    );

    let mark_cya = move |_| {
        let Some(deal_id) = id.get() else { return };
        spawn_local(async move {
            let body = serde_json::json!({ "signed": true });
            match post_deal_action(deal_id, "cya".into(), body).await {
                Ok(_) => {
                    msg.set("CYA marked signed".into());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => msg.set(e.to_string()),
            }
        });
    };

    let mark_title = move |_| {
        let Some(deal_id) = id.get() else { return };
        spawn_local(async move {
            let body = serde_json::json!({
                "title_search_ordered": true,
                "title_clear": true
            });
            match post_deal_action(deal_id, "title".into(), body).await {
                Ok(_) => {
                    msg.set("Title cleared".into());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => msg.set(e.to_string()),
            }
        });
    };

    let do_convert = move |_| {
        let Some(deal_id) = id.get() else { return };
        spawn_local(async move {
            let body = serde_json::json!({});
            match post_deal_action(deal_id, "convert".into(), body).await {
                Ok(v) => {
                    let parsed: Result<ConvertResp, _> = serde_json::from_value(v);
                    msg.set(match parsed {
                        Ok(c) => format!(
                            "Converted · asset {} · contract {}",
                            c.asset_id, c.contract_id
                        ),
                        Err(e) => format!("Converted (parse detail failed): {e}"),
                    });
                    refresh.update(|n| *n += 1);
                }
                Err(e) => msg.set(e.to_string()),
            }
        });
    };

    let do_assign = move |_| {
        let Some(deal_id) = id.get() else { return };
        spawn_local(async move {
            let body = serde_json::json!({
                "assignment_fee_cents": 1000000,
                "deposit_cents": 50000,
                "expires_days": 14
            });
            match post_deal_action(deal_id, "assign".into(), body).await {
                Ok(v) => {
                    let _ = serde_json::from_value::<IdResp>(v);
                    msg.set("Assignment created".into());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => msg.set(e.to_string()),
            }
        });
    };

    let convert_cf = move |_| {
        let Some(deal_id) = id.get() else { return };
        spawn_local(async move {
            match post_deal_action(deal_id, "convert-to-cf".into(), serde_json::json!({})).await {
                Ok(_) => {
                    msg.set("Converted to Creative Finance".into());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => msg.set(e.to_string()),
            }
        });
    };

    view! {
        <div class="main-area">
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading…"</div> }>
                {move || deal.get().map(|res| match res {
                    Ok(d) => {
                        let structure_href = FolioRoute::LandlordDealStructure
                            .path()
                            .replace(":id", &d.id.to_string());
                        let is_cf = d.track == "creative_finance";
                        let is_ws = d.track == "wholesale";
                        let advance_to = next_stage(&d.track, &d.opportunity_type, &d.status)
                            .map(|s| s.to_string());
                        view! {
                            <div class="page-header">
                                <div>
                                    <a class="text-sm" href=FolioRoute::LandlordDeals.path()>"← Deal Ops"</a>
                                    <h1 class="page-title">{d.property_address.clone()}</h1>
                                    <p class="page-subtitle">
                                        {d.track.clone()}" · "{d.status.clone()}" · "{d.opportunity_type.clone()}
                                    </p>
                                </div>
                                <div class="page-actions" style="display:flex;gap:0.5rem;flex-wrap:wrap;">
                                    {advance_to.map(|stage| {
                                        let stage_click = stage.clone();
                                        let label = format!("Advance → {}", stage.replace('_', " "));
                                        view! {
                                            <button
                                                type="button"
                                                class="folio-btn folio-btn--primary press"
                                                on:click=move |_| {
                                                    let Some(deal_id) = id.get() else { return };
                                                    let stage = stage_click.clone();
                                                    spawn_local(async move {
                                                        let body = serde_json::json!({ "stage": stage });
                                                        match post_deal_action(deal_id, "advance".into(), body).await {
                                                            Ok(_) => {
                                                                msg.set("Stage advanced".into());
                                                                refresh.update(|n| *n += 1);
                                                            }
                                                            Err(e) => msg.set(e.to_string()),
                                                        }
                                                    });
                                                }
                                            >
                                                {label}
                                            </button>
                                        }
                                    })}
                                    <a class="folio-btn folio-btn--ghost press" href=structure_href>"Structure offer"</a>
                                </div>
                            </div>

                            <Show when=move || !msg.get().is_empty()>
                                <div class="doc-empty mb-4">{move || msg.get()}</div>
                            </Show>

                            <div class="grid gap-4" style="grid-template-columns: 2fr 1fr;">
                                <div class="card p-4">
                                    <h3 class="font-bold mb-2">"Checklist"</h3>
                                    <ul class="space-y-2 text-sm">
                                        <li>"Status: " {d.status.clone()}</li>
                                        <li>"Structure: " {d.acquisition_structure.clone().unwrap_or_else(|| "—".into())}</li>
                                        <li>"Exit: " {d.exit_mode.clone().unwrap_or_else(|| "—".into())}</li>
                                        <li>
                                            "CYA required: "
                                            {d.cya_required.map(|b| if b {"yes"} else {"no"}).unwrap_or("—")}
                                            " · signed: "
                                            {d.cya_signed.map(|b| if b {"yes"} else {"no"}).unwrap_or("—")}
                                        </li>
                                        <li>
                                            "Title clear: "
                                            {d.title_clear.map(|b| if b {"yes"} else {"no"}).unwrap_or("—")}
                                        </li>
                                    </ul>
                                    <div class="flex flex-wrap gap-2 mt-4">
                                        <Show when=move || is_cf>
                                            <button class="btn btn-primary btn-sm" on:click=mark_cya>"Mark CYA signed"</button>
                                            <button class="btn btn-ghost btn-sm" on:click=do_convert>"Convert → asset + contract"</button>
                                        </Show>
                                        <Show when=move || is_ws>
                                            <button class="btn btn-primary btn-sm" on:click=mark_title>"Mark title clear"</button>
                                            <button class="btn btn-ghost btn-sm" on:click=do_assign>"Create assignment"</button>
                                            <button class="btn btn-ghost btn-sm" on:click=convert_cf>"Convert → CF"</button>
                                        </Show>
                                    </div>
                                </div>
                                <div class="card p-4">
                                    <h3 class="font-bold mb-2">"Economics"</h3>
                                    <p class="text-sm">"ARV: $" {(d.arv_cents.unwrap_or(0) / 100).to_string()}</p>
                                    <p class="text-sm">"Repairs: $" {(d.repair_cents.unwrap_or(0) / 100).to_string()}</p>
                                    <p class="text-sm">"Offer: $" {(d.offer_cents.or(d.deal_amount_cents).unwrap_or(0) / 100).to_string()}</p>
                                </div>
                            </div>
                        }.into_any()
                    }
                    Err(e) => view! { <div class="doc-empty">{e.to_string()}</div> }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
