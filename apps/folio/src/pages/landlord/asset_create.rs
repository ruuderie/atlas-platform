//! Create property — `/l/assets/new`

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PropertyType {
    SingleFamily,
    MultiFamily,
    Condo,
    Townhouse,
    Str,
    Commercial,
}

impl PropertyType {
    const ALL: &'static [Self] = &[
        Self::SingleFamily,
        Self::MultiFamily,
        Self::Condo,
        Self::Townhouse,
        Self::Str,
        Self::Commercial,
    ];

    fn as_str(self) -> &'static str {
        match self {
            Self::SingleFamily => "single_family",
            Self::MultiFamily => "multi_family",
            Self::Condo => "condo",
            Self::Townhouse => "townhouse",
            Self::Str => "str",
            Self::Commercial => "commercial",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::SingleFamily => "Single family",
            Self::MultiFamily => "Multi-family",
            Self::Condo => "Condo",
            Self::Townhouse => "Townhouse",
            Self::Str => "STR",
            Self::Commercial => "Commercial",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PortfolioOpt {
    id: Uuid,
    name: String,
}

#[derive(Serialize)]
struct CreateAssetBody {
    portfolio_id: Uuid,
    parent_asset_id: Option<Uuid>,
    property_type: String,
    name: String,
    address_line_1: String,
    address_line_2: Option<String>,
    city: String,
    state_province: String,
    postal_code: String,
    country_code: String,
    folio_number: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Deserialize)]
struct IdResp {
    id: Uuid,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(ListPortfoliosForAssetCreate, "/api")]
async fn list_portfolios() -> Result<Vec<PortfolioOpt>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    #[derive(Deserialize)]
    struct Raw {
        id: Uuid,
        name: String,
    }
    let rows: Vec<Raw> =
        crate::atlas_client::authenticated_get("/api/folio/portfolios", &token, None)
            .await
            .map_err(ServerFnError::new)?;
    Ok(rows
        .into_iter()
        .map(|r| PortfolioOpt {
            id: r.id,
            name: r.name,
        })
        .collect())
}

#[server(CreateAsset, "/api")]
async fn create_asset(
    portfolio_id: Uuid,
    property_type: String,
    name: String,
    address_line_1: String,
    address_line_2: Option<String>,
    city: String,
    state_province: String,
    postal_code: String,
    country_code: String,
    folio_number: Option<String>,
) -> Result<Uuid, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let body = CreateAssetBody {
        portfolio_id,
        parent_asset_id: None,
        property_type,
        name,
        address_line_1,
        address_line_2,
        city,
        state_province,
        postal_code,
        country_code,
        folio_number,
        latitude: None,
        longitude: None,
    };
    let resp: IdResp =
        crate::atlas_client::authenticated_post("/api/folio/assets", &token, None, &body)
            .await
            .map_err(ServerFnError::new)?;
    Ok(resp.id)
}

#[component]
pub fn AssetCreate() -> impl IntoView {
    let navigate = use_navigate();
    let portfolios = Resource::new(|| (), |_| list_portfolios());

    let portfolio_id = RwSignal::new(String::new());
    let property_type = RwSignal::new(PropertyType::SingleFamily);
    let name = RwSignal::new(String::new());
    let address1 = RwSignal::new(String::new());
    let address2 = RwSignal::new(String::new());
    let city = RwSignal::new(String::new());
    let state = RwSignal::new(String::new());
    let postal = RwSignal::new(String::new());
    let country = RwSignal::new("US".to_string());
    let folio = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let pending = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(Ok(list)) = portfolios.get() {
            if portfolio_id.get().is_empty() {
                if let Some(first) = list.first() {
                    portfolio_id.set(first.id.to_string());
                }
            }
        }
    });

    let title = Signal::derive(|| "Add property".to_string());
    let subtitle =
        Signal::derive(|| "Register a property in your holdings inventory.".to_string());

    view! {
        <div class="folio-form-page">
            <PageHeader title=title subtitle=subtitle>
                <a class="folio-btn folio-btn--ghost press" href=FolioRoute::LandlordAssets.path()>
                    "Cancel"
                </a>
            </PageHeader>

            <form
                class="folio-form"
                on:submit=move |ev| {
                    ev.prevent_default();
                    error.set(None);
                    let Ok(pid) = Uuid::parse_str(&portfolio_id.get()) else {
                        error.set(Some("Select a portfolio.".into()));
                        return;
                    };
                    let n = name.get().trim().to_string();
                    let a1 = address1.get().trim().to_string();
                    let c = city.get().trim().to_string();
                    let st = state.get().trim().to_string();
                    let pc = postal.get().trim().to_string();
                    if n.is_empty() || a1.is_empty() || c.is_empty() || st.is_empty() || pc.is_empty() {
                        error.set(Some("Name, address, city, state, and postal code are required.".into()));
                        return;
                    }
                    let a2 = {
                        let v = address2.get().trim().to_string();
                        if v.is_empty() { None } else { Some(v) }
                    };
                    let fn_opt = {
                        let v = folio.get().trim().to_string();
                        if v.is_empty() { None } else { Some(v) }
                    };
                    pending.set(true);
                    let nav = navigate.clone();
                    spawn_local(async move {
                        match create_asset(
                            pid,
                            property_type.get().as_str().to_string(),
                            n,
                            a1,
                            a2,
                            c,
                            st,
                            pc,
                            country.get(),
                            fn_opt,
                        )
                        .await
                        {
                            Ok(id) => {
                                let path = FolioRoute::LandlordAssetDetail
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
                    <span class="folio-field__label">"Portfolio"</span>
                    <Suspense fallback=|| view! { <p class="folio-muted">"Loading…"</p> }>
                        {move || portfolios.get().map(|res| match res {
                            Ok(list) if list.is_empty() => view! {
                                <p class="folio-error">"No portfolio yet — complete onboarding first."</p>
                            }.into_any(),
                            Ok(list) => view! {
                                <select
                                    class="folio-input"
                                    prop:value=move || portfolio_id.get()
                                    on:change=move |e| portfolio_id.set(event_target_value(&e))
                                >
                                    {list.into_iter().map(|p| {
                                        let id = p.id.to_string();
                                        view! { <option value=id.clone()>{p.name}</option> }
                                    }).collect_view()}
                                </select>
                            }.into_any(),
                            Err(e) => view! { <p class="folio-error">{e.to_string()}</p> }.into_any(),
                        })}
                    </Suspense>
                </label>

                <label class="folio-field">
                    <span class="folio-field__label">"Property type"</span>
                    <select
                        class="folio-input"
                        on:change=move |e| {
                            let v = event_target_value(&e);
                            if let Some(t) = PropertyType::ALL.iter().copied().find(|t| t.as_str() == v) {
                                property_type.set(t);
                            }
                        }
                    >
                        {PropertyType::ALL.iter().map(|t| {
                            view! {
                                <option value=t.as_str() selected=(*t == PropertyType::SingleFamily)>
                                    {t.label()}
                                </option>
                            }
                        }).collect_view()}
                    </select>
                </label>

                <label class="folio-field">
                    <span class="folio-field__label">"Name"</span>
                    <input
                        class="folio-input"
                        type="text"
                        placeholder="e.g. 123 Oak St"
                        prop:value=move || name.get()
                        on:input=move |e| name.set(event_target_value(&e))
                    />
                </label>

                <label class="folio-field">
                    <span class="folio-field__label">"Address"</span>
                    <input
                        class="folio-input"
                        type="text"
                        prop:value=move || address1.get()
                        on:input=move |e| address1.set(event_target_value(&e))
                    />
                </label>
                <label class="folio-field">
                    <span class="folio-field__label">"Address line 2"</span>
                    <input
                        class="folio-input"
                        type="text"
                        prop:value=move || address2.get()
                        on:input=move |e| address2.set(event_target_value(&e))
                    />
                </label>

                <div class="folio-form__row">
                    <label class="folio-field">
                        <span class="folio-field__label">"City"</span>
                        <input class="folio-input" type="text" prop:value=move || city.get() on:input=move |e| city.set(event_target_value(&e)) />
                    </label>
                    <label class="folio-field">
                        <span class="folio-field__label">"State"</span>
                        <input class="folio-input" type="text" prop:value=move || state.get() on:input=move |e| state.set(event_target_value(&e)) />
                    </label>
                    <label class="folio-field">
                        <span class="folio-field__label">"Postal"</span>
                        <input class="folio-input" type="text" prop:value=move || postal.get() on:input=move |e| postal.set(event_target_value(&e)) />
                    </label>
                </div>

                <div class="folio-form__row">
                    <label class="folio-field">
                        <span class="folio-field__label">"Country"</span>
                        <input class="folio-input" type="text" maxlength="2" prop:value=move || country.get() on:input=move |e| country.set(event_target_value(&e).to_uppercase()) />
                    </label>
                    <label class="folio-field">
                        <span class="folio-field__label">"Folio number (optional)"</span>
                        <input class="folio-input" type="text" prop:value=move || folio.get() on:input=move |e| folio.set(event_target_value(&e)) />
                    </label>
                </div>

                {move || error.get().map(|msg| view! { <p class="folio-error">{msg}</p> })}

                <button type="submit" class="folio-btn folio-btn--primary press" prop:disabled=move || pending.get()>
                    {move || if pending.get() { "Creating…" } else { "Add property" }}
                </button>
            </form>
        </div>
    }
}
