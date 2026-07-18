//! Property Owner Lite — Property Value Tracker — `/po/value`
//! Wired to value-history GET/POST for the first owned property asset.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::pages::landlord::assets::{list_assets, AssetSummary};

const VALUE_SOURCES: &[(&str, &str, &str)] = &[
    ("manual", "My Estimate", "edit"),
    ("purchase_price", "Purchase Price", "sell"),
    ("zillow_avm", "Zillow AVM", "bar_chart"),
    ("county_record", "County Record", "account_balance"),
    ("certified_appraisal", "Appraisal", "verified"),
    ("bank_appraisal", "Bank Appraisal", "domain"),
    ("agent_cma", "Agent CMA", "real_estate_agent"),
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValueHistoryEntry {
    pub id: Uuid,
    pub source: String,
    pub source_ref: Option<String>,
    pub value_cents: i64,
    pub currency_code: String,
    pub valued_on: chrono::NaiveDate,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LogValueInput {
    source: String,
    source_ref: Option<String>,
    value_cents: i64,
    currency_code: Option<String>,
    valued_on: String,
    note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LogValueResponse {
    id: Uuid,
}

#[component]
pub fn PropertyValuePage() -> impl IntoView {
    let (selected_source, set_source) = signal("manual");
    let (value_input, set_value) = signal(String::new());
    let (date_input, set_date) = signal(String::new());
    let (note_input, set_note) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (success_msg, set_success) = signal(Option::<String>::None);
    let (error_msg, set_error) = signal(Option::<String>::None);
    let refresh = RwSignal::new(0u32);

    let property_id = Resource::new(|| (), |_| async move {
        let assets = list_assets().await?;
        Ok::<Option<Uuid>, server_fn::error::ServerFnError>(
            assets
                .into_iter()
                .find(|a: &AssetSummary| a.parent_asset_id.is_none())
                .map(|a| a.id),
        )
    });

    let history = Resource::new(
        move || (refresh.get(), property_id.get()),
        |(_, pid)| async move {
            match pid {
                Some(Ok(Some(id))) => fetch_value_history(id).await,
                Some(Ok(None)) => Ok(Vec::new()),
                Some(Err(e)) => Err(e),
                None => Ok(Vec::new()),
            }
        },
    );

    view! {
        <div class="page-header">
            <div>
                <h1 class="page-title">"Property Value"</h1>
                <p class="page-subtitle">
                    "Log valuations from any source and watch your equity grow over time."
                </p>
            </div>
        </div>

        <Suspense fallback=|| view! { <div class="folio-empty"><p class="folio-empty__sub">"Loading property…"</p></div> }>
            {move || match property_id.get() {
                Some(Ok(None)) => view! {
                    <div class="folio-empty">
                        <p class="folio-empty__heading">"No property on file"</p>
                        <p class="folio-empty__sub">"Complete onboarding to attach a property before logging valuations."</p>
                    </div>
                }.into_any(),
                Some(Err(e)) => view! {
                    <div class="folio-empty">
                        <p class="folio-empty__heading">"Could not load property"</p>
                        <p class="folio-empty__sub">{e.to_string()}</p>
                    </div>
                }.into_any(),
                _ => view! {
                    <div class="split-panel">
                        <div class="split-panel__main">
                            <div class="card" style="margin-bottom:16px">
                                <div class="card-header">
                                    <span class="card-title">"History"</span>
                                </div>
                                <Suspense fallback=|| view! { <p class="folio-empty__sub" style="padding:1rem;">"Loading history…"</p> }>
                                    {move || history.get().map(|res| match res {
                                        Err(e) => view! { <p class="folio-empty__sub" style="padding:1rem;">{e.to_string()}</p> }.into_any(),
                                        Ok(items) if items.is_empty() => view! {
                                            <div class="chart-empty-state" style="padding:2rem;">
                                                <span class="ms msf chart-empty-state__icon">"show_chart"</span>
                                                <p>"No valuations logged yet."</p>
                                            </div>
                                        }.into_any(),
                                        Ok(items) => view! {
                                            <table class="data-table">
                                                <thead>
                                                    <tr>
                                                        <th>"Date"</th>
                                                        <th>"Source"</th>
                                                        <th>"Value"</th>
                                                        <th>"Note"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {items.into_iter().map(|e| {
                                                        let val = format!("${:.0}", e.value_cents as f64 / 100.0);
                                                        view! {
                                                            <tr>
                                                                <td>{e.valued_on.to_string()}</td>
                                                                <td>{e.source.replace('_', " ")}</td>
                                                                <td>{val}</td>
                                                                <td>{e.note.clone().unwrap_or_default()}</td>
                                                            </tr>
                                                        }
                                                    }).collect_view()}
                                                </tbody>
                                            </table>
                                        }.into_any(),
                                    })}
                                </Suspense>
                            </div>
                        </div>

                        <div class="split-panel__aside">
                            <div class="card">
                                <div class="card-header">
                                    <span class="card-title">"Log a Valuation"</span>
                                </div>
                                <div class="card-body">
                                    <div class="form-group">
                                        <label class="form-label">"Valuation Source"</label>
                                        <div class="source-grid">
                                            {VALUE_SOURCES.iter().map(|(slug, label, icon)| {
                                                let slug_str = *slug;
                                                let is_sel = move || selected_source.get() == slug_str;
                                                view! {
                                                    <button
                                                        type="button"
                                                        class:source-btn=true
                                                        class:source-btn--selected=is_sel
                                                        on:click=move |_| set_source.set(slug_str)
                                                    >
                                                        <span class="ms msf source-btn__icon">{*icon}</span>
                                                        <span class="source-btn__label">{*label}</span>
                                                    </button>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                    <div class="form-group">
                                        <label class="form-label">"Estimated Value (USD)"</label>
                                        <input
                                            type="number"
                                            min="1"
                                            step="1000"
                                            class="form-input"
                                            placeholder="450000"
                                            on:input=move |e| set_value.set(event_target_value(&e))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label class="form-label">"Valuation Date"</label>
                                        <input
                                            type="date"
                                            class="form-input"
                                            on:input=move |e| set_date.set(event_target_value(&e))
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label class="form-label">"Note (optional)"</label>
                                        <input
                                            type="text"
                                            class="form-input"
                                            on:input=move |e| set_note.set(event_target_value(&e))
                                        />
                                    </div>
                                    <Show when=move || success_msg.get().is_some()>
                                        <div class="alert alert-success">{move || success_msg.get().unwrap_or_default()}</div>
                                    </Show>
                                    <Show when=move || error_msg.get().is_some()>
                                        <div class="folio-empty__sub">{move || error_msg.get().unwrap_or_default()}</div>
                                    </Show>
                                    <button
                                        type="button"
                                        class="btn btn-primary w-full"
                                        prop:disabled=move || submitting.get()
                                        on:click=move |_| {
                                            let Some(Ok(Some(pid))) = property_id.get() else {
                                                set_error.set(Some("No property to attach valuation to.".into()));
                                                return;
                                            };
                                            let dollars: f64 = value_input.get().parse().unwrap_or(0.0);
                                            if dollars <= 0.0 || date_input.get().is_empty() {
                                                set_error.set(Some("Enter a value and date.".into()));
                                                return;
                                            }
                                            set_submitting.set(true);
                                            set_error.set(None);
                                            let source = selected_source.get().to_string();
                                            let note = {
                                                let n = note_input.get();
                                                if n.is_empty() { None } else { Some(n) }
                                            };
                                            let valued_on = date_input.get();
                                            spawn_local(async move {
                                                match log_property_value(
                                                    pid,
                                                    source,
                                                    (dollars * 100.0) as i64,
                                                    valued_on,
                                                    note,
                                                ).await {
                                                    Ok(_) => {
                                                        set_success.set(Some("Valuation logged.".into()));
                                                        refresh.update(|n| *n += 1);
                                                    }
                                                    Err(e) => set_error.set(Some(e.to_string())),
                                                }
                                                set_submitting.set(false);
                                            });
                                        }
                                    >
                                        {move || if submitting.get() { "Saving…" } else { "Log Valuation" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any(),
            }}
        </Suspense>
    }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(FetchPoValueHistory, "/api")]
pub async fn fetch_value_history(
    property_id: Uuid,
) -> Result<Vec<ValueHistoryEntry>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<ValueHistoryEntry>>(
        &format!("/api/folio/properties/{property_id}/value-history"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[server(LogPoPropertyValue, "/api")]
pub async fn log_property_value(
    property_id: Uuid,
    source: String,
    value_cents: i64,
    valued_on: String,
    note: Option<String>,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let body = LogValueInput {
        source,
        source_ref: None,
        value_cents,
        currency_code: Some("USD".into()),
        valued_on,
        note,
    };
    let resp = crate::atlas_client::authenticated_post::<LogValueInput, LogValueResponse>(
        &format!("/api/folio/properties/{property_id}/value"),
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(resp.id)
}
