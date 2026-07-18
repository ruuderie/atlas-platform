//! Meridian Analytics — `/l/meridian`
//! Thin KPI overview from `GET /api/folio/analytics/landlord`. Configure link → G-27.

use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LandlordOverview {
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub revenue_this_month_cents: i64,
    pub revenue_this_year_cents: i64,
    pub active_leases: i64,
    pub open_maintenance_cases: i64,
    pub open_violations: i64,
    pub outstanding_payments: i64,
    pub outstanding_balance_cents: i64,
    pub on_time_payment_rate_pct: f64,
}

fn fmt_money(cents: i64) -> String {
    format!("${:.0}", cents as f64 / 100.0)
}

#[component]
pub fn MeridianAnalytics() -> impl IntoView {
    let overview = Resource::new(|| (), |_| async move { fetch_landlord_analytics().await });
    let configure_href = FolioRoute::LandlordMeridianConfig.path();

    view! {
        <div class="main-area">
            <PageHeader
                title=Signal::derive(|| "Meridian".to_string())
                subtitle=Signal::derive(|| {
                    "Portfolio KPIs from live ledger, leases, and cases.".to_string()
                })
            >
                <A href=configure_href attr:class="btn btn-secondary btn-sm">
                    "Configure G-27"
                </A>
            </PageHeader>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading analytics…"</p></div>
            }>
                {move || match overview.get() {
                    Some(Err(e)) => view! {
                        <div class="folio-empty">
                            <p class="folio-empty__heading">"Could not load analytics"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Some(Ok(o)) => view! {
                        <p class="page-subtitle" style="margin-bottom:1rem;">
                            {format!("Generated {}", o.generated_at.format("%Y-%m-%d %H:%M UTC"))}
                        </p>
                        <div class="stat-grid stat-grid--4">
                            <div class="stat-card">
                                <div class="stat-body">
                                    <p class="stat-label">"Revenue (month)"</p>
                                    <p class="stat-value">{fmt_money(o.revenue_this_month_cents)}</p>
                                </div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-body">
                                    <p class="stat-label">"Revenue (year)"</p>
                                    <p class="stat-value">{fmt_money(o.revenue_this_year_cents)}</p>
                                </div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-body">
                                    <p class="stat-label">"Active leases"</p>
                                    <p class="stat-value">{o.active_leases.to_string()}</p>
                                </div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-body">
                                    <p class="stat-label">"On-time pay rate"</p>
                                    <p class="stat-value">{format!("{:.0}%", o.on_time_payment_rate_pct)}</p>
                                </div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-body">
                                    <p class="stat-label">"Open maintenance"</p>
                                    <p class="stat-value">{o.open_maintenance_cases.to_string()}</p>
                                </div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-body">
                                    <p class="stat-label">"Open violations"</p>
                                    <p class="stat-value">{o.open_violations.to_string()}</p>
                                </div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-body">
                                    <p class="stat-label">"Outstanding payments"</p>
                                    <p class="stat-value">{o.outstanding_payments.to_string()}</p>
                                </div>
                            </div>
                            <div class="stat-card">
                                <div class="stat-body">
                                    <p class="stat-label">"Outstanding balance"</p>
                                    <p class="stat-value">{fmt_money(o.outstanding_balance_cents)}</p>
                                </div>
                            </div>
                        </div>
                    }.into_any(),
                    None => view! { <div /> }.into_any(),
                }}
            </Suspense>
        </div>
    }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(FetchLandlordAnalytics, "/api")]
pub async fn fetch_landlord_analytics(
) -> Result<LandlordOverview, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<LandlordOverview>(
        "/api/folio/analytics/landlord",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}
