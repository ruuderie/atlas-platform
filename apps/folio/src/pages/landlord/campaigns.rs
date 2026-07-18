//! Campaigns — `/l/campaigns`
//! Wired to `GET /api/folio/campaigns`.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CampaignSummary {
    pub id: Uuid,
    pub name: String,
    pub campaign_type: String,
    pub status: String,
    #[serde(default)]
    pub spent_cents: i64,
    pub budget_cents: Option<i64>,
    pub currency: Option<String>,
    pub goal_type: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct CampaignListResponse {
    campaigns: Vec<CampaignSummary>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CampaignStatusFilter {
    All,
    Active,
    Draft,
    Paused,
    Completed,
}

impl CampaignStatusFilter {
    const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Draft => "Draft",
            Self::Paused => "Paused",
            Self::Completed => "Completed",
        }
    }

    fn matches(self, status: &str) -> bool {
        match self {
            Self::All => true,
            Self::Active => status.eq_ignore_ascii_case("active"),
            Self::Draft => status.eq_ignore_ascii_case("draft"),
            Self::Paused => status.eq_ignore_ascii_case("paused"),
            Self::Completed => {
                status.eq_ignore_ascii_case("completed") || status.eq_ignore_ascii_case("archived")
            }
        }
    }
}

fn status_tone(status: &str) -> StatusPillTone {
    match status.to_ascii_lowercase().as_str() {
        "active" => StatusPillTone::Ok,
        "paused" => StatusPillTone::Warn,
        "draft" => StatusPillTone::Neutral,
        "completed" | "archived" => StatusPillTone::Info,
        _ => StatusPillTone::Neutral,
    }
}

fn fmt_money(cents: i64, currency: Option<&str>) -> String {
    let sym = match currency.unwrap_or("USD") {
        "USD" | "usd" => "$",
        other => other,
    };
    format!("{sym}{:.0}", cents as f64 / 100.0)
}

#[component]
pub fn Campaigns() -> impl IntoView {
    let filter = RwSignal::new(CampaignStatusFilter::All);
    let campaigns = Resource::new(|| (), |_| async move { list_campaigns().await });

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Campaigns".to_string())
                subtitle=Signal::derive(|| "Marketing campaigns and outreach.".to_string())
            >
                <a class="folio-btn folio-btn--ghost press" href=FolioRoute::LandlordLeads.path()>
                    "Leads"
                </a>
            </PageHeader>

            <div class="landlord-filter-chips" style="margin-bottom:1rem;">
                {[
                    CampaignStatusFilter::All,
                    CampaignStatusFilter::Active,
                    CampaignStatusFilter::Draft,
                    CampaignStatusFilter::Paused,
                    CampaignStatusFilter::Completed,
                ]
                    .into_iter()
                    .map(|f| {
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    if filter.get() == f {
                                        "landlord-chip landlord-chip--active"
                                    } else {
                                        "landlord-chip"
                                    }
                                }
                                on:click=move |_| filter.set(f)
                            >
                                {f.label()}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading campaigns…"</p></div>
            }>
                {move || campaigns.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load campaigns"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(items) => {
                        let f = filter.get();
                        let filtered: Vec<_> = items.into_iter().filter(|c| f.matches(&c.status)).collect();
                        if filtered.is_empty() {
                            view! {
                                <div class="folio-empty">
                                    <span class="material-symbols-outlined folio-empty__icon">"campaign"</span>
                                    <p class="folio-empty__heading">"No campaigns yet"</p>
                                    <p class="folio-empty__sub">
                                        "Create outreach from leads or open-house workflows — campaigns appear here when saved."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="landlord-card-grid">
                                    {filtered.into_iter().map(|c| {
                                        let spent = fmt_money(c.spent_cents, c.currency.as_deref());
                                        let budget = c.budget_cents
                                            .map(|b| fmt_money(b, c.currency.as_deref()))
                                            .unwrap_or_else(|| "—".into());
                                        let tone = status_tone(&c.status);
                                        view! {
                                            <div class="landlord-card landlord-card--static">
                                                <div class="landlord-card__top">
                                                    <span class="material-symbols-outlined landlord-card__icon">"campaign"</span>
                                                    <StatusPill label=c.status.clone() tone=tone/>
                                                </div>
                                                <h3 class="landlord-card__title">{c.name.clone()}</h3>
                                                <p class="landlord-card__meta">{c.campaign_type.replace('_', " ")}</p>
                                                <p class="landlord-card__meta">
                                                    {c.goal_type.clone().unwrap_or_else(|| "No goal set".into())}
                                                </p>
                                                <p class="landlord-card__stat">
                                                    <span class="landlord-card__stat-value">{spent}</span>
                                                    {format!(" spent / {budget} budget")}
                                                </p>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
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

#[server(ListLandlordCampaigns, "/api")]
pub async fn list_campaigns() -> Result<Vec<CampaignSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let resp = crate::atlas_client::authenticated_get::<CampaignListResponse>(
        "/api/folio/campaigns",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Campaign list failed: {e}")))?;
    Ok(resp.campaigns)
}
