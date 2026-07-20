//! Historical lease backfill — `/l/assets/:id/history/lease`
//! Offline person or Atlas user → `POST /api/folio/leases/historical`.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_params_map};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::pages::landlord::lease_create::list_tenant_candidates;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CounterpartyKind {
    OfflinePerson,
    AtlasUser,
}

impl CounterpartyKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::OfflinePerson => "offline_person",
            Self::AtlasUser => "atlas_user",
        }
    }
}

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
struct CreateHistoricalLeaseBody {
    asset_id: Uuid,
    counterparty_kind: String,
    counterparty_user_id: Option<Uuid>,
    offline_name: Option<String>,
    offline_phone: Option<String>,
    offline_email: Option<String>,
    offline_notes: Option<String>,
    monthly_rent_cents: i64,
    currency: String,
    guarantee_type: String,
    start_date: String,
    end_date: Option<String>,
}

#[derive(Deserialize)]
struct IdResp {
    id: Uuid,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(CreateHistoricalLease, "/api")]
async fn create_historical_lease(
    asset_id: Uuid,
    counterparty_kind: String,
    counterparty_user_id: Option<Uuid>,
    offline_name: Option<String>,
    offline_phone: Option<String>,
    offline_email: Option<String>,
    offline_notes: Option<String>,
    monthly_rent_cents: i64,
    currency: String,
    guarantee_type: String,
    start_date: String,
    end_date: Option<String>,
) -> Result<Uuid, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let body = CreateHistoricalLeaseBody {
        asset_id,
        counterparty_kind,
        counterparty_user_id,
        offline_name,
        offline_phone,
        offline_email,
        offline_notes,
        monthly_rent_cents,
        currency,
        guarantee_type,
        start_date,
        end_date,
    };
    let resp: IdResp = crate::atlas_client::authenticated_post(
        "/api/folio/leases/historical",
        &token,
        None,
        &body,
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(resp.id)
}

#[component]
pub fn HistoricalLeaseCreate() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();
    let asset_id = Memo::new(move |_| {
        params
            .get()
            .get("id")
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or(Uuid::nil())
    });

    let kind = RwSignal::new(CounterpartyKind::OfflinePerson);
    let offline_name = RwSignal::new(String::new());
    let offline_phone = RwSignal::new(String::new());
    let offline_email = RwSignal::new(String::new());
    let offline_notes = RwSignal::new(String::new());
    let counterparty_user = RwSignal::new(String::new());
    let rent = RwSignal::new(String::new());
    let currency = RwSignal::new("USD".to_string());
    let guarantee = RwSignal::new(GuaranteeType::SecurityDeposit);
    let start_date = RwSignal::new(String::new());
    let end_date = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let pending = RwSignal::new(false);
    let tenants = Resource::new(|| (), |_| list_tenant_candidates());

    let history_href = Memo::new(move |_| {
        FolioRoute::LandlordUnitHistory
            .path()
            .replace(":id", &asset_id.get().to_string())
    });

    view! {
        <div class="folio-form-page">
            <PageHeader
                title=Signal::derive(|| "Historical lease".to_string())
                subtitle=Signal::derive(|| {
                    "Backfill a past lease for this unit. Offline tenants are allowed."
                        .to_string()
                })
            >
                <a class="folio-btn folio-btn--ghost press" href=move || history_href.get()>
                    "Cancel"
                </a>
            </PageHeader>

            <form
                class="folio-form"
                on:submit=move |ev| {
                    ev.prevent_default();
                    error.set(None);
                    let aid = asset_id.get();
                    if aid.is_nil() {
                        error.set(Some("Missing unit id.".into()));
                        return;
                    }
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
                    let k = kind.get();
                    let (uid, oname, ophone, oemail, onotes) = match k {
                        CounterpartyKind::AtlasUser => {
                            let Ok(u) = Uuid::parse_str(counterparty_user.get().trim()) else {
                                error.set(Some("Select a tenant.".into()));
                                return;
                            };
                            (Some(u), None, None, None, None)
                        }
                        CounterpartyKind::OfflinePerson => {
                            let name = offline_name.get().trim().to_string();
                            if name.is_empty() {
                                error.set(Some("Offline tenant name is required.".into()));
                                return;
                            }
                            let phone = {
                                let p = offline_phone.get().trim().to_string();
                                if p.is_empty() { None } else { Some(p) }
                            };
                            let email = {
                                let e = offline_email.get().trim().to_string();
                                if e.is_empty() { None } else { Some(e) }
                            };
                            let notes = {
                                let n = offline_notes.get().trim().to_string();
                                if n.is_empty() { None } else { Some(n) }
                            };
                            (None, Some(name), phone, email, notes)
                        }
                    };
                    let g = guarantee.get().as_str().to_string();
                    let cur = currency.get();
                    let kind_s = k.as_str().to_string();
                    pending.set(true);
                    let nav = navigate.clone();
                    let back = history_href.get();
                    spawn_local(async move {
                        match create_historical_lease(
                            aid,
                            kind_s,
                            uid,
                            oname,
                            ophone,
                            oemail,
                            onotes,
                            rent_cents,
                            cur,
                            g,
                            start,
                            end,
                        )
                        .await
                        {
                            Ok(_) => nav(&back, Default::default()),
                            Err(e) => {
                                error.set(Some(e.to_string()));
                                pending.set(false);
                            }
                        }
                    });
                }
            >
                <fieldset class="folio-form__section">
                    <legend>"Counterparty"</legend>
                    <div class="unit-actions" style="margin-bottom:1rem;">
                        <button
                            type="button"
                            class=move || if kind.get() == CounterpartyKind::OfflinePerson {
                                "folio-btn folio-btn--primary press"
                            } else {
                                "folio-btn folio-btn--ghost press"
                            }
                            on:click=move |_| kind.set(CounterpartyKind::OfflinePerson)
                        >
                            "Offline person"
                        </button>
                        <button
                            type="button"
                            class=move || if kind.get() == CounterpartyKind::AtlasUser {
                                "folio-btn folio-btn--primary press"
                            } else {
                                "folio-btn folio-btn--ghost press"
                            }
                            on:click=move |_| kind.set(CounterpartyKind::AtlasUser)
                        >
                            "Known tenant"
                        </button>
                    </div>

                    <Show when=move || kind.get() == CounterpartyKind::OfflinePerson>
                        <label class="folio-field__label">
                            "Full name"
                            <input
                                class="folio-input"
                                type="text"
                                prop:value=move || offline_name.get()
                                on:input=move |ev| offline_name.set(event_target_value(&ev))
                            />
                        </label>
                        <label class="folio-field__label">
                            "Phone"
                            <input
                                class="folio-input"
                                type="tel"
                                prop:value=move || offline_phone.get()
                                on:input=move |ev| offline_phone.set(event_target_value(&ev))
                            />
                        </label>
                        <label class="folio-field__label">
                            "Email"
                            <input
                                class="folio-input"
                                type="email"
                                prop:value=move || offline_email.get()
                                on:input=move |ev| offline_email.set(event_target_value(&ev))
                            />
                        </label>
                        <label class="folio-field__label">
                            "Notes"
                            <textarea
                                class="folio-input"
                                prop:value=move || offline_notes.get()
                                on:input=move |ev| offline_notes.set(event_target_value(&ev))
                            />
                        </label>
                    </Show>

                    <Show when=move || kind.get() == CounterpartyKind::AtlasUser>
                        <label class="folio-field__label">
                            "Tenant"
                            <Suspense fallback=|| view! { <p class="proj-section__hint">"Loading people…"</p> }>
                                {move || tenants.get().map(|res| match res {
                                    Ok(list) if list.is_empty() => view! {
                                        <p class="proj-section__hint">
                                            "No applicants or prior tenants yet. Use Offline person for paper leases."
                                        </p>
                                    }.into_any(),
                                    Ok(list) => view! {
                                        <select
                                            class="folio-input"
                                            prop:value=move || counterparty_user.get()
                                            on:change=move |ev| counterparty_user.set(event_target_value(&ev))
                                        >
                                            <option value="">"Select tenant…"</option>
                                            {list.into_iter().map(|t| {
                                                let id = t.user_id.to_string();
                                                let label = t.label;
                                                view! { <option value=id>{label}</option> }
                                            }).collect::<Vec<_>>()}
                                        </select>
                                    }.into_any(),
                                    Err(e) => view! { <p style="color:#b91c1c;">{e.to_string()}</p> }.into_any(),
                                })}
                            </Suspense>
                        </label>
                    </Show>
                </fieldset>

                <fieldset class="folio-form__section">
                    <legend>"Terms"</legend>
                    <label class="folio-field__label">
                        "Monthly rent"
                        <input
                            class="folio-input"
                            type="text"
                            inputmode="decimal"
                            placeholder="1850"
                            prop:value=move || rent.get()
                            on:input=move |ev| rent.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="folio-field__label">
                        "Currency"
                        <input
                            class="folio-input"
                            type="text"
                            prop:value=move || currency.get()
                            on:input=move |ev| currency.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="folio-field__label">
                        "Guarantee"
                        <select
                            class="folio-input"
                            on:change=move |ev| {
                                let v = event_target_value(&ev);
                                if let Some(g) = GuaranteeType::ALL.iter().copied().find(|g| g.as_str() == v) {
                                    guarantee.set(g);
                                }
                            }
                        >
                            {GuaranteeType::ALL.iter().map(|g| {
                                let val = g.as_str();
                                let label = g.label();
                                view! { <option value=val>{label}</option> }
                            }).collect::<Vec<_>>()}
                        </select>
                    </label>
                    <label class="folio-field__label">
                        "Start date"
                        <input
                            class="folio-input"
                            type="date"
                            prop:value=move || start_date.get()
                            on:input=move |ev| start_date.set(event_target_value(&ev))
                        />
                    </label>
                    <label class="folio-field__label">
                        "End date"
                        <input
                            class="folio-input"
                            type="date"
                            prop:value=move || end_date.get()
                            on:input=move |ev| end_date.set(event_target_value(&ev))
                        />
                    </label>
                </fieldset>

                {move || error.get().map(|e| view! {
                    <p class="folio-form__error" style="color:#b91c1c;">{e}</p>
                })}

                <div class="unit-actions">
                    <button
                        type="submit"
                        class="folio-btn folio-btn--primary press"
                        disabled=move || pending.get()
                    >
                        {move || if pending.get() { "Saving…" } else { "Save historical lease" }}
                    </button>
                    <a class="folio-btn folio-btn--ghost press" href=FolioRoute::LandlordVault.path()>
                        "Attach via vault"
                    </a>
                </div>
            </form>
        </div>
    }
}
