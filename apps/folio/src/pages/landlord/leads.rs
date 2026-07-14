//! Leads list — `/l/leads`
//!
//! Wired to `GET /api/folio/leads`.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::page_header::PageHeader;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeadRow {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub lead_status: String,
    pub source: Option<String>,
    #[serde(default)]
    pub is_converted: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LeadFilter {
    All,
    New,
    Qualified,
    Converted,
}

impl LeadFilter {
    const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::New => "New",
            Self::Qualified => "Qualified",
            Self::Converted => "Converted",
        }
    }

    fn matches(self, lead: &LeadRow) -> bool {
        match self {
            Self::All => true,
            Self::New => {
                lead.lead_status.eq_ignore_ascii_case("new")
                    || lead.lead_status.eq_ignore_ascii_case("contacted")
            }
            Self::Qualified => {
                lead.lead_status.eq_ignore_ascii_case("qualified")
                    || lead.lead_status.eq_ignore_ascii_case("qualifying")
            }
            Self::Converted => lead.is_converted || lead.lead_status.eq_ignore_ascii_case("converted"),
        }
    }
}

#[component]
pub fn Leads() -> impl IntoView {
    let (filter, set_filter) = signal(LeadFilter::All);
    let leads = Resource::new(|| (), |_| async move { list_leads().await });

    let title = Signal::derive(|| "Leads".to_string());
    let subtitle = Signal::derive(|| "Prospective tenants and buyers.".to_string());

    view! {
        <div class="landlord-list-page">
            <PageHeader title=title subtitle=subtitle />

            <div class="landlord-filter-bar">
                <div class="landlord-filter-chips">
                    {[
                        LeadFilter::All,
                        LeadFilter::New,
                        LeadFilter::Qualified,
                        LeadFilter::Converted,
                    ]
                        .into_iter()
                        .map(|f| view! {
                            <button
                                type="button"
                                class=move || if filter.get() == f {
                                    "landlord-chip landlord-chip--active"
                                } else {
                                    "landlord-chip"
                                }
                                on:click=move |_| set_filter.set(f)
                            >
                                {f.label()}
                            </button>
                        })
                        .collect_view()}
                </div>
            </div>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading leads…"</p></div>
            }>
                {move || leads.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load leads"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(all) => {
                        let f = filter.get();
                        let filtered: Vec<_> = all.into_iter().filter(|l| f.matches(l)).collect();
                        if filtered.is_empty() {
                            view! {
                                <div class="folio-empty">
                                    <span class="material-symbols-outlined folio-empty__icon">"person_search"</span>
                                    <p class="folio-empty__heading">"No leads yet"</p>
                                    <p class="folio-empty__sub">
                                        "Inbound interest from listings and campaigns will show up here."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="landlord-table-wrap">
                                    <table class="landlord-table">
                                        <thead>
                                            <tr>
                                                <th>"Name"</th>
                                                <th>"Contact"</th>
                                                <th>"Status"</th>
                                                <th>"Source"</th>
                                                <th>"Created"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {filtered.into_iter().map(|l| {
                                                let contact = l.email.clone()
                                                    .or(l.phone.clone())
                                                    .unwrap_or_else(|| "—".into());
                                                let source = l.source.clone().unwrap_or_else(|| "—".into());
                                                let created = l.created_at.format("%Y-%m-%d").to_string();
                                                let status = l.lead_status.clone();
                                                view! {
                                                    <tr>
                                                        <td>{l.name}</td>
                                                        <td>{contact}</td>
                                                        <td>
                                                            <span class="landlord-pill landlord-pill--muted">{status}</span>
                                                        </td>
                                                        <td>{source}</td>
                                                        <td>{created}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </table>
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

#[server(ListLeads, "/api")]
pub async fn list_leads() -> Result<Vec<LeadRow>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<LeadRow>>("/api/folio/leads", &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Lead list failed: {e}")))
}
