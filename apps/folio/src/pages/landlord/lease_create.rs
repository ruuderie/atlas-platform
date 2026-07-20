//! Create lease — `/l/leases/new`
//! Optional `?asset_id=` + `?user_id=` prefill from unit detail / tenant profile.

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_query_map};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::pages::landlord::leases::{activate_lease, create_occupancy};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TenantBranch {
    AtlasUser,
    OfflinePerson,
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
    #[serde(default)]
    address_line_1: Option<String>,
    #[serde(default)]
    city: Option<String>,
    #[serde(default)]
    state_province: Option<String>,
}

/// People a landlord can put on a live lease — never typed as raw UUIDs.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TenantCandidate {
    pub user_id: Uuid,
    pub label: String,
    pub source: String,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

/// Applicants + prior lease counterparties, labeled by name/email.
#[server(ListTenantCandidates, "/api")]
pub async fn list_tenant_candidates() -> Result<Vec<TenantCandidate>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    use std::collections::HashMap;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;

    #[derive(Deserialize)]
    struct AppRow {
        applicant_user_id: Uuid,
        status: String,
    }
    #[derive(Deserialize)]
    struct LeaseRow {
        counterparty_user_id: Option<Uuid>,
        status: String,
    }
    #[derive(Deserialize)]
    struct UserRow {
        id: Uuid,
        first_name: String,
        last_name: String,
        email: String,
    }

    let apps: Vec<AppRow> =
        crate::atlas_client::authenticated_get("/api/folio/applications", &token, None)
            .await
            .unwrap_or_default();
    let leases: Vec<LeaseRow> =
        crate::atlas_client::authenticated_get("/api/folio/leases", &token, None)
            .await
            .unwrap_or_default();

    let mut sources: HashMap<Uuid, String> = HashMap::new();
    for a in apps {
        sources
            .entry(a.applicant_user_id)
            .or_insert_with(|| format!("Application · {}", a.status.replace('_', " ")));
    }
    for l in leases {
        if let Some(uid) = l.counterparty_user_id {
            sources
                .entry(uid)
                .or_insert_with(|| format!("Prior lease · {}", l.status.replace('_', " ")));
        }
    }

    let mut out = Vec::new();
    for (uid, source) in sources {
        let label = match crate::atlas_client::authenticated_get::<UserRow>(
            &format!("/api/folio/users/{uid}"),
            &token,
            None,
        )
        .await
        {
            Ok(u) => {
                let name = format!("{} {}", u.first_name, u.last_name)
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");
                if u.email.is_empty() {
                    if name.is_empty() {
                        source.clone()
                    } else {
                        format!("{name} · {source}")
                    }
                } else if name.is_empty() {
                    format!("{} · {source}", u.email)
                } else {
                    format!("{name} · {} · {source}", u.email)
                }
            }
            Err(_) => source.clone(),
        };
        out.push(TenantCandidate {
            user_id: uid,
            label,
            source,
        });
    }
    out.sort_by(|a, b| a.label.to_lowercase().cmp(&b.label.to_lowercase()));
    Ok(out)
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
        #[serde(default)]
        address_line_1: Option<String>,
        #[serde(default)]
        city: Option<String>,
        #[serde(default)]
        state_province: Option<String>,
    }
    let rows: Vec<Raw> = crate::atlas_client::authenticated_get("/api/folio/assets", &token, None)
        .await
        .map_err(ServerFnError::new)?;
    Ok(rows
        .into_iter()
        .map(|r| AssetOpt {
            id: r.id,
            name: r.name,
            address_line_1: r.address_line_1,
            city: r.city,
            state_province: r.state_province,
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
    let tenants = Resource::new(|| (), |_| list_tenant_candidates());

    let asset_id = RwSignal::new(String::new());
    let counterparty = RwSignal::new(String::new());
    let tenant_branch = RwSignal::new(TenantBranch::AtlasUser);
    let offline_name = RwSignal::new(String::new());
    let offline_phone = RwSignal::new(String::new());
    let offline_email = RwSignal::new(String::new());
    let rent = RwSignal::new(String::new());
    let currency = RwSignal::new("USD".to_string());
    let guarantee = RwSignal::new(GuaranteeType::SecurityDeposit);
    let start_date = RwSignal::new(String::new());
    let end_date = RwSignal::new(String::new());
    let auto_renew = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let pending = RwSignal::new(false);

    Effect::new(move |_| {
        let map = q.get();
        if let Some(aid) = map.get("asset_id") {
            if !aid.is_empty() {
                asset_id.set(aid);
            }
        }
        if let Some(uid) = map.get("user_id") {
            if !uid.is_empty() {
                counterparty.set(uid);
                tenant_branch.set(TenantBranch::AtlasUser);
            }
        }
    });

    let title = Signal::derive(|| "New lease".to_string());
    let subtitle = Signal::derive(|| {
        "Create a rental contract against a unit — Atlas tenant or offline person.".to_string()
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
                    let start = start_date.get();
                    let end = {
                        let e = end_date.get();
                        if e.is_empty() { None } else { Some(e) }
                    };
                    let rent_raw = rent.get();
                    let rent_trimmed = rent_raw.trim().to_string();
                    let rent_cents = if rent_trimmed.is_empty() {
                        None
                    } else {
                        match rent_trimmed.parse::<f64>() {
                            Ok(v) if v >= 0.0 => Some((v * 100.0).round() as i64),
                            _ => {
                                error.set(Some("Enter monthly rent (e.g. 1850).".into()));
                                return;
                            }
                        }
                    };

                    pending.set(true);
                    let nav = navigate.clone();

                    match tenant_branch.get() {
                        TenantBranch::OfflinePerson => {
                            let name = offline_name.get().trim().to_string();
                            if name.is_empty() {
                                error.set(Some("Enter the person’s name.".into()));
                                pending.set(false);
                                return;
                            }
                            let phone = offline_phone.get();
                            let email = offline_email.get();
                            let currency_v = currency.get();
                            let guarantee_v = guarantee.get().as_str().to_string();
                            let auto = auto_renew.get();
                            spawn_local(async move {
                                match create_occupancy(
                                    aid,
                                    name,
                                    Some(phone).filter(|s| !s.trim().is_empty()),
                                    Some(email).filter(|s| !s.trim().is_empty()),
                                    None,
                                    if start.is_empty() { None } else { Some(start.clone()) },
                                )
                                .await
                                {
                                    Ok(lease_id) => {
                                        if let Some(cents) = rent_cents {
                                            if start.is_empty() {
                                                error.set(Some(
                                                    "Start date is required to activate with rent."
                                                        .into(),
                                                ));
                                                pending.set(false);
                                                return;
                                            }
                                            match activate_lease(
                                                lease_id,
                                                cents,
                                                currency_v,
                                                guarantee_v,
                                                start,
                                                end,
                                                auto,
                                                None,
                                            )
                                            .await
                                            {
                                                Ok(()) => {
                                                    let path = FolioRoute::LandlordLeaseDetail
                                                        .path()
                                                        .replace(":id", &lease_id.to_string());
                                                    nav(&path, Default::default());
                                                }
                                                Err(e) => {
                                                    error.set(Some(e.to_string()));
                                                    pending.set(false);
                                                }
                                            }
                                        } else {
                                            // Occupancy only — back to unit to attach later.
                                            let path = FolioRoute::LandlordAssetDetail
                                                .path()
                                                .replace(":id", &aid.to_string());
                                            nav(&path, Default::default());
                                        }
                                    }
                                    Err(e) => {
                                        error.set(Some(e.to_string()));
                                        pending.set(false);
                                    }
                                }
                            });
                        }
                        TenantBranch::AtlasUser => {
                            let Ok(cid) = Uuid::parse_str(counterparty.get().trim()) else {
                                error.set(Some("Select a tenant.".into()));
                                pending.set(false);
                                return;
                            };
                            let Some(cents) = rent_cents else {
                                error.set(Some("Enter monthly rent (e.g. 1850).".into()));
                                pending.set(false);
                                return;
                            };
                            if start.is_empty() {
                                error.set(Some("Start date is required.".into()));
                                pending.set(false);
                                return;
                            }
                            let currency_v = currency.get();
                            let guarantee_v = guarantee.get().as_str().to_string();
                            let auto = auto_renew.get();
                            spawn_local(async move {
                                match create_lease(
                                    aid,
                                    cid,
                                    cents,
                                    currency_v,
                                    guarantee_v,
                                    start,
                                    end,
                                    auto,
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
                    }
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
                                        let label = crate::utils::format_asset_place_label(
                                            &a.name,
                                            a.address_line_1.as_deref(),
                                            a.city.as_deref(),
                                            a.state_province.as_deref(),
                                        );
                                        view! { <option value=id>{label}</option> }
                                    }).collect_view()}
                                </select>
                            }.into_any(),
                            Err(e) => view! { <p class="folio-error">{e.to_string()}</p> }.into_any(),
                        })}
                    </Suspense>
                </label>

                <fieldset class="folio-field">
                    <legend class="folio-field__label">"Tenant"</legend>
                    <div class="folio-segment-bar" style="margin-bottom:0.75rem;">
                        <button
                            type="button"
                            class=move || {
                                if tenant_branch.get() == TenantBranch::AtlasUser {
                                    "folio-segment folio-segment--active"
                                } else {
                                    "folio-segment"
                                }
                            }
                            on:click=move |_| tenant_branch.set(TenantBranch::AtlasUser)
                        >
                            "Atlas person"
                        </button>
                        <button
                            type="button"
                            class=move || {
                                if tenant_branch.get() == TenantBranch::OfflinePerson {
                                    "folio-segment folio-segment--active"
                                } else {
                                    "folio-segment"
                                }
                            }
                            on:click=move |_| tenant_branch.set(TenantBranch::OfflinePerson)
                        >
                            "New person"
                        </button>
                    </div>

                    <Show when=move || tenant_branch.get() == TenantBranch::AtlasUser>
                        <Suspense fallback=|| view! { <p class="folio-muted">"Loading people…"</p> }>
                            {move || tenants.get().map(|res| match res {
                                Ok(list) if list.is_empty() => view! {
                                    <div class="folio-empty--compact" style="text-align:left;">
                                        <p>
                                            "No applicants or prior tenants yet."
                                        </p>
                                        <div class="unit-actions" style="margin-top:0.75rem;">
                                            <button
                                                type="button"
                                                class="folio-btn folio-btn--primary press"
                                                on:click=move |_| {
                                                    tenant_branch.set(TenantBranch::OfflinePerson)
                                                }
                                            >
                                                "Add offline person"
                                            </button>
                                            <a
                                                class="folio-btn folio-btn--ghost press"
                                                href=FolioRoute::LandlordApplications.path()
                                            >
                                                "Applications"
                                            </a>
                                        </div>
                                    </div>
                                }.into_any(),
                                Ok(list) => view! {
                                    <select
                                        class="folio-input"
                                        prop:value=move || counterparty.get()
                                        on:change=move |e| counterparty.set(event_target_value(&e))
                                    >
                                        <option value="">"Select tenant…"</option>
                                        {list.into_iter().map(|t| {
                                            let id = t.user_id.to_string();
                                            let label = t.label;
                                            view! { <option value=id>{label}</option> }
                                        }).collect_view()}
                                    </select>
                                }.into_any(),
                                Err(e) => view! { <p class="folio-error">{e.to_string()}</p> }.into_any(),
                            })}
                        </Suspense>
                    </Show>

                    <Show when=move || tenant_branch.get() == TenantBranch::OfflinePerson>
                        <div class="space-y-3">
                            <label class="folio-field">
                                <span class="folio-field__label">"Name"</span>
                                <input
                                    class="folio-input"
                                    type="text"
                                    prop:value=move || offline_name.get()
                                    on:input=move |e| offline_name.set(event_target_value(&e))
                                />
                            </label>
                            <label class="folio-field">
                                <span class="folio-field__label">"Phone (optional)"</span>
                                <input
                                    class="folio-input"
                                    type="tel"
                                    prop:value=move || offline_phone.get()
                                    on:input=move |e| offline_phone.set(event_target_value(&e))
                                />
                            </label>
                            <label class="folio-field">
                                <span class="folio-field__label">"Email (optional)"</span>
                                <input
                                    class="folio-input"
                                    type="email"
                                    prop:value=move || offline_email.get()
                                    on:input=move |e| offline_email.set(event_target_value(&e))
                                />
                            </label>
                            <p class="folio-muted" style="font-size:0.8rem;">
                                "Leave rent blank to save occupancy only, then attach terms on the unit."
                            </p>
                        </div>
                    </Show>
                </fieldset>

                <div class="folio-form__row">
                    <label class="folio-field">
                        <span class="folio-field__label">
                            {move || {
                                if tenant_branch.get() == TenantBranch::OfflinePerson {
                                    "Monthly rent (optional)"
                                } else {
                                    "Monthly rent"
                                }
                            }}
                        </span>
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
                    {move || {
                        if pending.get() {
                            "Saving…"
                        } else if tenant_branch.get() == TenantBranch::OfflinePerson
                            && rent.get().trim().is_empty()
                        {
                            "Save occupancy"
                        } else {
                            "Create lease"
                        }
                    }}
                </button>
            </form>
        </div>
    }
}
