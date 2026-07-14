//! Portfolio list — `/l/portfolio`
//!
//! Wired to `GET /api/folio/portfolios`.

use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

use crate::components::page_header::PageHeader;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PortfolioSummary {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub name: String,
    pub description: Option<String>,
    pub asset_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[component]
pub fn Portfolio() -> impl IntoView {
    let portfolios = Resource::new(|| (), |_| async move { list_portfolios().await });

    let title = Signal::derive(|| "Portfolio".to_string());
    let subtitle = Signal::derive(|| "Manage your property portfolio.".to_string());

    view! {
        <div class="landlord-list-page">
            <PageHeader title=title subtitle=subtitle>
                <A href="/l/assets" attr:class="folio-btn folio-btn--primary">
                    <span class="material-symbols-outlined">"apartment"</span>
                    "View assets"
                </A>
            </PageHeader>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading portfolios…"</p></div>
            }>
                {move || portfolios.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load portfolios"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(items) if items.is_empty() => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"domain"</span>
                            <p class="folio-empty__heading">"No portfolios yet"</p>
                            <p class="folio-empty__sub">
                                "Your first portfolio is created when you add a property during onboarding."
                            </p>
                        </div>
                    }.into_any(),
                    Ok(items) => view! {
                        <div class="landlord-card-grid">
                            {items.into_iter().map(|p| {
                                let name = p.name.clone();
                                let desc = p.description.clone().unwrap_or_default();
                                let count = p.asset_count;
                                let created = p.created_at.format("%b %Y").to_string();
                                view! {
                                    <div class="landlord-card landlord-card--static">
                                        <div class="landlord-card__top">
                                            <span class="material-symbols-outlined landlord-card__icon">"domain"</span>
                                            <span class="landlord-pill landlord-pill--muted">{created}</span>
                                        </div>
                                        <h3 class="landlord-card__title">{name}</h3>
                                        <p class="landlord-card__meta">
                                            {if desc.is_empty() { "Real estate portfolio".to_string() } else { desc }}
                                        </p>
                                        <p class="landlord-card__stat">
                                            <span class="landlord-card__stat-value">{count.to_string()}</span>
                                            " assets"
                                        </p>
                                        <A href="/l/assets" attr:class="folio-btn folio-btn--ghost" attr:style="margin-top:0.75rem;align-self:flex-start">
                                            "Browse assets"
                                        </A>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(ListPortfolios, "/api")]
pub async fn list_portfolios() -> Result<Vec<PortfolioSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<PortfolioSummary>>(
        "/api/folio/portfolios",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Portfolio list failed: {e}")))
}
