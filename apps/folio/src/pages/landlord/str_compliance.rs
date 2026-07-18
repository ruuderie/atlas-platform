//! STR Compliance — `/l/str`
//! Wired to `GET /api/folio/str/permits` (+ optional expiry scan).

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};

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
    let permits = Resource::new(
        move || refresh.get(),
        |_| async move { list_str_permits().await },
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

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "STR Compliance".to_string())
                subtitle=Signal::derive(|| "Short-term rental permits and regulatory status.".to_string())
            >
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
        </div>
    }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
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
