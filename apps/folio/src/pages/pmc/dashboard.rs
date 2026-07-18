//! PMC dashboard — `/pmc`
//! Wired to `GET /api/folio/pm/analytics`.

use crate::auth::{ServerFnError, SessionInfo};
use crate::components::nav::FolioRoute;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PmAnalytics {
    tenant_id: Uuid,
    total_active_leases: i64,
    total_portfolios: i64,
    #[serde(default)]
    clients: Vec<ClientMetric>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ClientMetric {
    account_id: Uuid,
    active_leases: i64,
    portfolio_count: i64,
}

#[component]
pub fn PmcDashboard() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, ServerFnError>>>()
        .expect("Session context missing");
    let name = move || {
        session
            .get()
            .and_then(|r| r.ok())
            .and_then(|s| s.display_name)
            .unwrap_or_else(|| "there".into())
    };

    let analytics = Resource::new(|| (), |_| async move { fetch_pm_analytics().await });

    let clients = Signal::derive(move || {
        analytics
            .get()
            .and_then(|r| r.ok())
            .map(|a| {
                if a.clients.is_empty() {
                    a.total_portfolios.to_string()
                } else {
                    a.clients.len().to_string()
                }
            })
            .unwrap_or_else(|| "—".into())
    });

    let leases = Signal::derive(move || {
        analytics
            .get()
            .and_then(|r| r.ok())
            .map(|a| a.total_active_leases.to_string())
            .unwrap_or_else(|| "—".into())
    });

    view! {
        <div class="landlord-list-page">
            <div class="page-header">
                <h1 class="page-title">{move || format!("PMC Dashboard — {}", name())}</h1>
                <p class="page-subtitle">"Clients, onboarding, and activity across accounts."</p>
            </div>
            <Suspense fallback=|| view! { <div class="folio-empty"><p class="folio-empty__sub">"Loading…"</p></div> }>
                {move || match analytics.get() {
                    Some(Err(e)) => view! {
                        <div class="folio-empty">
                            <p class="folio-empty__heading">"Could not load analytics"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    _ => view! {
                        <div class="stat-grid">
                            <a href=FolioRoute::PmcClientBook.path() class="stat-card stat-card--link">
                                <span class="stat-icon">"🏢"</span>
                                <div class="stat-body">
                                    <p class="stat-label">"Client accounts / portfolios"</p>
                                    <p class="stat-value">{move || clients.get()}</p>
                                </div>
                            </a>
                            <a href=FolioRoute::PmcClientBook.path() class="stat-card stat-card--link">
                                <span class="stat-icon">"📋"</span>
                                <div class="stat-body">
                                    <p class="stat-label">"Active leases"</p>
                                    <p class="stat-value">{move || leases.get()}</p>
                                </div>
                            </a>
                        </div>
                    }.into_any(),
                }}
            </Suspense>
        </div>
    }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(FetchPmAnalytics, "/api")]
async fn fetch_pm_analytics() -> Result<PmAnalytics, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<PmAnalytics>("/api/folio/pm/analytics", &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}
