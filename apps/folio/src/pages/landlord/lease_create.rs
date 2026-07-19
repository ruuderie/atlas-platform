//! Create lease — `/l/leases/new`
//! Optional `?asset_id=` prefill from unit detail.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_query_map};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GuaranteeType {
    SecurityDeposit,
    Guarantor,
    Fiador,
    SeguroFianca,
    TituloCapitalizacao,
    None,
}

impl GuaranteeType {
    const ALL: &'static [Self] = &[
        Self::SecurityDeposit,
        Self::Guarantor,
        Self::Fiador,
        Self::SeguroFianca,
        Self::TituloCapitalizacao,
        Self::None,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::SecurityDeposit => "security_deposit",
            Self::Guarantor => "guarantor",
            Self::Fiador => "fiador",
            Self::SeguroFianca => "seguro_fianca",
            Self::TituloCapitalizacao => "titulo_capitalizacao",
            Self::None => "none",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::SecurityDeposit => "Security deposit",
            Self::Guarantor => "Guarantor",
            Self::Fiador => "Fiador",
            Self::SeguroFianca => "Seguro fiança",
            Self::TituloCapitalizacao => "Título capitalização",
            Self::None => "None",
        }
    }
}

#[derive(Serialize)]
struct CreateLeaseBody {
    asset_id: Uuid,
    counterparty_user_id: Uuid,
    monthly_rent_cents: i64,
    currency: String,
    guarantee_type: String,
    start_date: String,
    end_date: Option<String>,
    auto_renew: bool,
}

#[derive(Deserialize)]
struct IdResp {
    id: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AssetOpt {
    id: Uuid,
    name: String,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(ListAssetsForLeaseCreate, "/api")]
async fn list_assets_for_lease_create() -> Result<Vec<AssetOpt>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    #[derive(Deserialize)]
    struct Raw {
        id: Uuid,
        name: String,
    }
    let rows: Vec<Raw> = crate::atlas_client::authenticated_get("/api/folio/assets", &token, None)
        .await
        .map_err(ServerFnError::new)?;
    Ok(rows
        .into_iter()
        .map(|r| AssetOpt {
            id: r.id,
            name: r.name,
        })
        .collect())
}

#[server(CreateLease, "/api")]
async fn create_lease(
    asset_id: Uuid,
    counterparty_user_id: Uuid,
    monthly_rent_cents: i64,
    currency: String,
    guarantee_type: String,
    start_date: String,
    end_date: Option<String>,
    auto_renew: bool,
) -> Result<Uuid, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let body = CreateLeaseBody {
        asset_id,
        counterparty_user_id,
        monthly_rent_cents,
        currency,
        guarantee_type,
        start_date,
        end_date,
        auto_renew,
    };
    let resp: IdResp =
        crate::atlas_client::authenticated_post("/api/folio/leases", &token, None, &body)
            .await
            .map_err(ServerFnError::new)?;
    Ok(resp.id)
}

#[component]
pub fn LeaseCreate() -> impl IntoView {
    let q = use_query_map();
    let navigate = use_navigate();
    let assets = Resource::new(|| (), |_| list_assets_for_lease_create());

    let asset_id = RwSignal::new(String::new());
    let counterparty = RwSignal::new(String::new());
    let rent = RwSignal::new(String::new());
    let currency = RwSignal::new("USD".to_string());
    let guarantee = RwSignal::new(GuaranteeType::SecurityDeposit);
    let start_date = RwSignal::new(String::new());
    let end_date = RwSignal::new(String::new());
    let auto_renew = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let pending = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(aid) = q.get().get("asset_id") {
            if !aid.is_empty() {
                asset_id.set(aid);
            }
        }
    });

    let title = Signal::derive(|| "New lease".to_string());
    let subtitle = Signal::derive(|| {
        "Create a rental contract against a unit. Tenant must already have an Atlas account."
            .to_string()
    });

    view! {
        <div class="folio-form-page">
            <PageHeader title=title subtitle=subtitle>
                <a class="folio-btn folio-btn--ghost press" href=FolioRoute::LandlordLeases.path()>
                    "Cancel"
                </a>
            </PageHeader>

            <form
                class="folio-form"
                on:submit=move |ev| {
                    ev.prevent_default();
                    error.set(None);
                    let Ok(aid) = Uuid::parse_str(&asset_id.get()) else {
                        error.set(Some("Select a unit / asset.".into()));
                        return;
                    };
                    let Ok(cid) = Uuid::parse_str(counterparty.get().trim()) else {
                        error.set(Some("Enter a valid tenant user UUID.".into()));
                        return;
                    };
                    let rent_cents = match rent.get().trim().parse::<f64>() {
                        Ok(v) if v >= 0.0 => (v * 100.0).round() as i64,
                        _ => {
                            error.set(Some("Enter monthly rent (e.g. 1850).".into()));
                            return;
                        }
                    };
                    let start = start_date.get();
                    if start.is_empty() {
                        error.set(Some("Start date is required.".into()));
                        return;
                    }
                    let end = {
                        let e = end_date.get();
                        if e.is_empty() { None } else { Some(e) }
                    };
                    pending.set(true);
                    let nav = navigate.clone();
                    spawn_local(async move {
                        match create_lease(
                            aid,
                            cid,
                            rent_cents,
                            currency.get(),
                            guarantee.get().as_str().to_string(),
                            start,
                            end,
                            auto_renew.get(),
                        )
                        .await
                        {
                            Ok(id) => {
                                let path = FolioRoute::LandlordLeaseDetail
                                    .path()
                                    .replace(":id", &id.to_string());
                                nav(&path, Default::default());
                            }
                            Err(e) => {
                                error.set(Some(e.to_string()));
                                pending.set(false);
                            }
                        }
                    });
                }
            >
                <label class="folio-field">
                    <span class="folio-field__label">"Unit / asset"</span>
                    <Suspense fallback=|| view! { <p class="folio-muted">"Loading assets…"</p> }>
                        {move || assets.get().map(|res| match res {
                            Ok(list) => view! {
                                <select
                                    class="folio-input"
                                    prop:value=move || asset_id.get()
                                    on:change=move |e| asset_id.set(event_target_value(&e))
                                >
                                    <option value="">"Select…"</option>
                                    {list.into_iter().map(|a| {
                                        let id = a.id.to_string();
                                        let name = a.name;
                                        view! { <option value=id.clone()>{name} " (" {id.clone()} ")"</option> }
                                    }).collect_view()}
                                </select>
                            }.into_any(),
                            Err(e) => view! { <p class="folio-error">{e.to_string()}</p> }.into_any(),
                        })}
                    </Suspense>
                </label>

                <label class="folio-field">
                    <span class="folio-field__label">"Tenant user ID"</span>
                    <input
                        class="folio-input"
                        type="text"
                        placeholder="UUID of counterparty user"
                        prop:value=move || counterparty.get()
                        on:input=move |e| counterparty.set(event_target_value(&e))
                    />
                </label>

                <div class="folio-form__row">
                    <label class="folio-field">
                        <span class="folio-field__label">"Monthly rent"</span>
                        <input
                            class="folio-input"
                            type="text"
                            inputmode="decimal"
                            placeholder="1850.00"
                            prop:value=move || rent.get()
                            on:input=move |e| rent.set(event_target_value(&e))
                        />
                    </label>
                    <label class="folio-field">
                        <span class="folio-field__label">"Currency"</span>
                        <select
                            class="folio-input"
                            prop:value=move || currency.get()
                            on:change=move |e| currency.set(event_target_value(&e))
                        >
                            <option value="USD">"USD"</option>
                            <option value="BRL">"BRL"</option>
                        </select>
                    </label>
                </div>

                <label class="folio-field">
                    <span class="folio-field__label">"Guarantee"</span>
                    <select
                        class="folio-input"
                        on:change=move |e| {
                            let v = event_target_value(&e);
                            if let Some(g) = GuaranteeType::ALL.iter().copied().find(|g| g.as_str() == v) {
                                guarantee.set(g);
                            }
                        }
                    >
                        {GuaranteeType::ALL.iter().map(|g| {
                            view! { <option value=g.as_str() selected=(*g == GuaranteeType::SecurityDeposit)>{g.label()}</option> }
                        }).collect_view()}
                    </select>
                </label>

                <div class="folio-form__row">
                    <label class="folio-field">
                        <span class="folio-field__label">"Start date"</span>
                        <input
                            class="folio-input"
                            type="date"
                            prop:value=move || start_date.get()
                            on:input=move |e| start_date.set(event_target_value(&e))
                        />
                    </label>
                    <label class="folio-field">
                        <span class="folio-field__label">"End date (optional)"</span>
                        <input
                            class="folio-input"
                            type="date"
                            prop:value=move || end_date.get()
                            on:input=move |e| end_date.set(event_target_value(&e))
                        />
                    </label>
                </div>

                <label class="folio-field folio-field--check">
                    <input
                        type="checkbox"
                        prop:checked=move || auto_renew.get()
                        on:change=move |e| {
                            let el = event_target::<web_sys::HtmlInputElement>(&e);
                            auto_renew.set(el.checked());
                        }
                    />
                    <span>"Auto-renew"</span>
                </label>

                {move || error.get().map(|msg| view! { <p class="folio-error">{msg}</p> })}

                <button
                    type="submit"
                    class="folio-btn folio-btn--primary press"
                    prop:disabled=move || pending.get()
                >
                    {move || if pending.get() { "Creating…" } else { "Create lease" }}
                </button>
            </form>
        </div>
    }
}
