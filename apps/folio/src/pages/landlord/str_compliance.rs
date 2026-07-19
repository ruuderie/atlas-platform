//! STR Compliance — `/l/str`
//! Wired to `GET /api/folio/str/permits` (+ optional expiry scan).

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::vendors::{list_assets_for_picker, AssetPickerItem};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrPermitSummary {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    pub permit_number: String,
    pub jurisdiction_code: String,
    pub status: String,
    pub expires_at: Option<chrono::NaiveDate>,
    pub permit_category: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResponse {
    pub cases_opened: u32,
    pub warning_days: u32,
}

/// STR permit category — mirrors backend `StrPermitCategory`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrPermitCategory {
    PrincipalResidence,
    Hosted,
    NonHosted,
}

impl StrPermitCategory {
    pub const ALL: &'static [Self] = &[
        Self::PrincipalResidence,
        Self::Hosted,
        Self::NonHosted,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PrincipalResidence => "principal_residence",
            Self::Hosted => "hosted",
            Self::NonHosted => "non_hosted",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::PrincipalResidence => "Principal Residence",
            Self::Hosted => "Hosted",
            Self::NonHosted => "Non-Hosted",
        }
    }
}

fn status_tone(status: &str) -> StatusPillTone {
    match status.to_ascii_lowercase().as_str() {
        "active" | "approved" | "valid" => StatusPillTone::Ok,
        "expiring" | "pending" => StatusPillTone::Warn,
        "expired" | "revoked" | "denied" => StatusPillTone::Danger,
        _ => StatusPillTone::Neutral,
    }
}

#[component]
pub fn StrCompliance() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let scan_msg = RwSignal::new(String::new());
    let scanning = RwSignal::new(false);
    let show_register = RwSignal::new(false);
    let asset_id = RwSignal::new(String::new());
    let permit_category =
        RwSignal::new(StrPermitCategory::PrincipalResidence.as_str().to_string());
    let permit_number = RwSignal::new(String::new());
    let expires_at = RwSignal::new(String::new());
    let jurisdiction_code = RwSignal::new(String::new());
    let registering = RwSignal::new(false);
    let reg_err = RwSignal::new(None::<String>);

    let permits = Resource::new(
        move || refresh.get(),
        |_| async move { list_str_permits().await },
    );
    let assets = Resource::new(
        move || show_register.get(),
        |open| async move {
            if !open {
                return Ok::<Vec<AssetPickerItem>, server_fn::error::ServerFnError>(vec![]);
            }
            list_assets_for_picker().await
        },
    );

    let run_scan = move |_| {
        scanning.set(true);
        scan_msg.set(String::new());
        spawn_local(async move {
            match trigger_permit_scan(30).await {
                Ok(r) => {
                    scan_msg.set(format!(
                        "Scan complete — {} compliance case(s) opened ({}-day window).",
                        r.cases_opened, r.warning_days
                    ));
                    refresh.update(|n| *n += 1);
                }
                Err(e) => scan_msg.set(e.to_string()),
            }
            scanning.set(false);
        });
    };

    let on_register = move |_| {
        let aid = asset_id.get().trim().to_string();
        let num = permit_number.get().trim().to_string();
        let exp = expires_at.get().trim().to_string();
        let jur = jurisdiction_code.get().trim().to_string();
        let cat = permit_category.get();
        if aid.is_empty() || num.is_empty() || exp.is_empty() || jur.is_empty() {
            reg_err.set(Some("All fields are required.".into()));
            return;
        }
        registering.set(true);
        reg_err.set(None);
        spawn_local(async move {
            match register_str_permit(aid, cat, num, exp, jur).await {
                Ok(_) => {
                    show_register.set(false);
                    permit_number.set(String::new());
                    expires_at.set(String::new());
                    jurisdiction_code.set(String::new());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => reg_err.set(Some(e.to_string())),
            }
            registering.set(false);
        });
    };

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "STR Compliance".to_string())
                subtitle=Signal::derive(|| "Short-term rental permits and regulatory status.".to_string())
            >
                <button
                    type="button"
                    class="folio-btn folio-btn--ghost press"
                    on:click=move |_| {
                        reg_err.set(None);
                        show_register.set(true);
                    }
                >
                    "+ Register Permit"
                </button>
                <button
                    type="button"
                    class="folio-btn folio-btn--primary press"
                    prop:disabled=move || scanning.get()
                    on:click=run_scan
                >
                    {move || if scanning.get() { "Scanning…" } else { "Scan expiries" }}
                </button>
            </PageHeader>

            <Show when=move || !scan_msg.get().is_empty()>
                <p class="folio-empty__sub" style="margin-bottom:1rem;">{move || scan_msg.get()}</p>
            </Show>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading permits…"</p></div>
            }>
                {move || permits.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load permits"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(items) if items.is_empty() => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"gavel"</span>
                            <p class="folio-empty__heading">"No STR permits registered"</p>
                            <p class="folio-empty__sub">
                                "Register operating permits for short-term units to track expiry and compliance cases."
                            </p>
                        </div>
                    }.into_any(),
                    Ok(items) => {
                        let today = chrono::Utc::now().date_naive();
                        let expiring = items.iter().filter(|p| {
                            p.expires_at.map(|d| (d - today).num_days() <= 60).unwrap_or(false)
                        }).count();
                        view! {
                            <div class="assets-kpi-strip">
                                <div class="assets-kpi">
                                    <p class="assets-kpi__label">"Permits"</p>
                                    <p class="assets-kpi__value">{items.len().to_string()}</p>
                                </div>
                                <div class="assets-kpi">
                                    <p class="assets-kpi__label">"Expiring ≤60d"</p>
                                    <p class="assets-kpi__value">{expiring.to_string()}</p>
                                </div>
                            </div>
                            <div class="landlord-card-grid">
                                {items.into_iter().map(|p| {
                                    let expires = p.expires_at
                                        .map(|d| d.format("%b %d, %Y").to_string())
                                        .unwrap_or_else(|| "No expiry".into());
                                    let cat = p.permit_category.clone().unwrap_or_else(|| "—".into());
                                    let tone = status_tone(&p.status);
                                    view! {
                                        <div class="landlord-card landlord-card--static">
                                            <div class="landlord-card__top">
                                                <span class="material-symbols-outlined landlord-card__icon">"gavel"</span>
                                                <StatusPill label=p.status.clone() tone=tone/>
                                            </div>
                                            <h3 class="landlord-card__title">{p.permit_number.clone()}</h3>
                                            <p class="landlord-card__meta">{p.jurisdiction_code.clone()}</p>
                                            <p class="landlord-card__meta">{cat.replace('_', " ")}</p>
                                            <p class="landlord-card__stat">
                                                <span class="landlord-card__stat-value" style="font-size:1rem;">{expires}</span>
                                                " expires"
                                            </p>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                })}
            </Suspense>

            <Show when=move || show_register.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Register STR Permit"</h3>
                            <button
                                type="button"
                                class="modal-close"
                                on:click=move |_| show_register.set(false)
                            >
                                "✕"
                            </button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="form-field">
                                <label class="form-label">"Asset *"</label>
                                <select
                                    class="form-select"
                                    on:change=move |ev| asset_id.set(event_target_value(&ev))
                                >
                                    <option value="">"Select asset…"</option>
                                    {move || assets.get().and_then(|r| r.ok()).unwrap_or_default().into_iter().map(|a| {
                                        let id = a.id.to_string();
                                        let label = format!("{} ({})", a.name, a.asset_type.replace('_', " "));
                                        view! { <option value=id>{label}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Permit Category *"</label>
                                <select
                                    class="form-select"
                                    on:change=move |ev| permit_category.set(event_target_value(&ev))
                                >
                                    {StrPermitCategory::ALL.iter().copied().map(|c| {
                                        view! { <option value=c.as_str()>{c.label()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Permit Number *"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    prop:value=permit_number
                                    on:input=move |ev| permit_number.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Expires At *"</label>
                                <input
                                    type="date"
                                    class="form-input"
                                    prop:value=expires_at
                                    on:input=move |ev| expires_at.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Jurisdiction Code *"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    placeholder="US-FL-MIAMI-DADE"
                                    prop:value=jurisdiction_code
                                    on:input=move |ev| jurisdiction_code.set(event_target_value(&ev))
                                />
                            </div>
                            {move || reg_err.get().map(|e| view! {
                                <p class="folio-empty__sub" style="color:var(--folio-danger, #b91c1c);">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button
                                type="button"
                                class="folio-btn folio-btn--ghost"
                                on:click=move |_| show_register.set(false)
                            >
                                "Cancel"
                            </button>
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary"
                                disabled=move || registering.get()
                                on:click=on_register
                            >
                                {move || if registering.get() { "Saving…" } else { "Register" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[derive(Serialize)]
struct RegisterStrPermitBody {
    asset_id: Uuid,
    permit_category: String,
    permit_number: String,
    expires_at: chrono::NaiveDate,
    jurisdiction_code: String,
}

#[derive(Deserialize)]
struct RegisterStrPermitResponse {
    id: Uuid,
}

/// POST /api/folio/str/permits
#[server(RegisterStrPermit, "/api")]
pub async fn register_str_permit(
    asset_id: String,
    permit_category: String,
    permit_number: String,
    expires_at: String,
    jurisdiction_code: String,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let asset_id = Uuid::parse_str(asset_id.trim())
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid asset ID"))?;
    if StrPermitCategory::ALL
        .iter()
        .all(|c| c.as_str() != permit_category.as_str())
    {
        return Err(server_fn::error::ServerFnError::new("Invalid permit category"));
    }
    let expires_at = chrono::NaiveDate::parse_from_str(expires_at.trim(), "%Y-%m-%d")
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid expiry date (use YYYY-MM-DD)"))?;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let body = RegisterStrPermitBody {
        asset_id,
        permit_category,
        permit_number: permit_number.trim().to_string(),
        expires_at,
        jurisdiction_code: jurisdiction_code.trim().to_string(),
    };
    let resp = crate::atlas_client::authenticated_post::<
        RegisterStrPermitBody,
        RegisterStrPermitResponse,
    >("/api/folio/str/permits", &token, None, &body)
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Register permit failed: {e}")))?;
    Ok(resp.id)
}

#[server(ListStrPermits, "/api")]
pub async fn list_str_permits() -> Result<Vec<StrPermitSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<StrPermitSummary>>(
        "/api/folio/str/permits",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("STR permits failed: {e}")))
}

#[server(TriggerStrPermitScan, "/api")]
pub async fn trigger_permit_scan(
    warning_days: u32,
) -> Result<ScanResponse, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    #[derive(Serialize)]
    struct Body {
        warning_days: Option<u32>,
    }
    crate::atlas_client::authenticated_post::<Body, ScanResponse>(
        "/api/folio/str/scan",
        &token,
        None,
        &Body {
            warning_days: Some(warning_days),
        },
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Permit scan failed: {e}")))
}
