//! Leads list — `/l/leads`
//!
//! Wired to `GET/POST /api/folio/leads` and row lifecycle actions.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::page_header::PageHeader;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeadRow {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub lead_status: String,
    pub source: Option<String>,
    #[serde(default)]
    pub is_converted: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Pipeline status — mirrors backend `LeadStatus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LeadPipelineStatus {
    New,
    Contacted,
    Qualifying,
    Qualified,
    Disqualified,
    Converted,
}

impl LeadPipelineStatus {
    const ALL_SETTABLE: &'static [Self] = &[
        Self::New,
        Self::Contacted,
        Self::Qualifying,
        Self::Qualified,
    ];

    fn from_api(s: &str) -> Option<Self> {
        match s {
            "new" => Some(Self::New),
            "contacted" => Some(Self::Contacted),
            "qualifying" => Some(Self::Qualifying),
            "qualified" => Some(Self::Qualified),
            "disqualified" => Some(Self::Disqualified),
            "converted" => Some(Self::Converted),
            _ => None,
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Contacted => "contacted",
            Self::Qualifying => "qualifying",
            Self::Qualified => "qualified",
            Self::Disqualified => "disqualified",
            Self::Converted => "converted",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::New => "New",
            Self::Contacted => "Contacted",
            Self::Qualifying => "Qualifying",
            Self::Qualified => "Qualified",
            Self::Disqualified => "Disqualified",
            Self::Converted => "Converted",
        }
    }

    const fn is_terminal(self) -> bool {
        matches!(self, Self::Disqualified | Self::Converted)
    }
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
            Self::Converted => {
                lead.is_converted || lead.lead_status.eq_ignore_ascii_case("converted")
            }
        }
    }
}

#[component]
pub fn Leads() -> impl IntoView {
    let (filter, set_filter) = signal(LeadFilter::All);
    let refresh = RwSignal::new(0u32);
    let leads = Resource::new(move || refresh.get(), |_| async move { list_leads().await });

    let show_add = RwSignal::new(false);
    let first_name = RwSignal::new(String::new());
    let last_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let company = RwSignal::new(String::new());
    let source = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let creating = RwSignal::new(false);
    let create_err = RwSignal::new(None::<String>);
    let action_err = RwSignal::new(None::<String>);

    let on_create = move |_| {
        let fn_ = first_name.get().trim().to_string();
        let ln = last_name.get().trim().to_string();
        let em = email.get().trim().to_string();
        let ph = phone.get().trim().to_string();
        let co = company.get().trim().to_string();
        if fn_.is_empty() && ln.is_empty() && em.is_empty() && co.is_empty() {
            create_err.set(Some(
                "Provide a name, email, or company so the lead can be identified.".into(),
            ));
            return;
        }
        creating.set(true);
        create_err.set(None);
        spawn_local(async move {
            match create_lead(
                empty_to_none(fn_),
                empty_to_none(ln),
                empty_to_none(em),
                empty_to_none(ph),
                empty_to_none(co),
                empty_to_none(source.get().trim().to_string()),
                empty_to_none(message.get().trim().to_string()),
            )
            .await
            {
                Ok(_) => {
                    show_add.set(false);
                    first_name.set(String::new());
                    last_name.set(String::new());
                    email.set(String::new());
                    phone.set(String::new());
                    company.set(String::new());
                    source.set(String::new());
                    message.set(String::new());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => create_err.set(Some(e.to_string())),
            }
            creating.set(false);
        });
    };

    let title = Signal::derive(|| "Leads".to_string());
    let subtitle = Signal::derive(|| "Prospective tenants and buyers.".to_string());

    view! {
        <div class="landlord-list-page">
            <PageHeader title=title subtitle=subtitle>
                <button
                    type="button"
                    class="folio-btn folio-btn--primary press"
                    on:click=move |_| {
                        create_err.set(None);
                        show_add.set(true);
                    }
                >
                    "Add lead"
                </button>
            </PageHeader>

            <Show when=move || action_err.get().is_some()>
                <p style="color:#b91c1c;font-size:0.875rem;margin-bottom:0.75rem;">
                    {move || action_err.get().unwrap_or_default()}
                </p>
            </Show>

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
                                    <button
                                        type="button"
                                        class="folio-btn folio-btn--primary press"
                                        style="margin-top:1rem;"
                                        on:click=move |_| {
                                            create_err.set(None);
                                            show_add.set(true);
                                        }
                                    >
                                        "Add lead"
                                    </button>
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
                                                <th>"Actions"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {filtered.into_iter().map(|l| {
                                                let contact = l.email.clone()
                                                    .or(l.phone.clone())
                                                    .unwrap_or_else(|| "—".into());
                                                let source_s = l.source.clone().unwrap_or_else(|| "—".into());
                                                let created = l.created_at.format("%Y-%m-%d").to_string();
                                                let status = l.lead_status.clone();
                                                let status_enum = LeadPipelineStatus::from_api(&l.lead_status);
                                                let terminal = status_enum
                                                    .map(|s| s.is_terminal())
                                                    .unwrap_or(l.is_converted);
                                                let lead_id = l.id;
                                                let status_for_select = l.lead_status.clone();
                                                view! {
                                                    <tr>
                                                        <td>{l.name}</td>
                                                        <td>{contact}</td>
                                                        <td>
                                                            <span class="landlord-pill landlord-pill--muted">{status}</span>
                                                        </td>
                                                        <td>{source_s}</td>
                                                        <td>{created}</td>
                                                        <td>
                                                            <Show when=move || !terminal>
                                                                <div class="flex flex-wrap gap-1">
                                                                    <button
                                                                        type="button"
                                                                        class="folio-btn folio-btn--ghost press"
                                                                        style="font-size:0.75rem;padding:0.25rem 0.5rem;"
                                                                        on:click=move |_| {
                                                                            action_err.set(None);
                                                                            spawn_local(async move {
                                                                                match qualify_lead(lead_id).await {
                                                                                    Ok(_) => refresh.update(|n| *n += 1),
                                                                                    Err(e) => action_err.set(Some(e.to_string())),
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        "Qualify"
                                                                    </button>
                                                                    <button
                                                                        type="button"
                                                                        class="folio-btn folio-btn--ghost press"
                                                                        style="font-size:0.75rem;padding:0.25rem 0.5rem;"
                                                                        on:click=move |_| {
                                                                            action_err.set(None);
                                                                            spawn_local(async move {
                                                                                match convert_lead(lead_id).await {
                                                                                    Ok(_) => refresh.update(|n| *n += 1),
                                                                                    Err(e) => action_err.set(Some(e.to_string())),
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        "Convert"
                                                                    </button>
                                                                    <button
                                                                        type="button"
                                                                        class="folio-btn folio-btn--ghost press"
                                                                        style="font-size:0.75rem;padding:0.25rem 0.5rem;"
                                                                        on:click=move |_| {
                                                                            action_err.set(None);
                                                                            spawn_local(async move {
                                                                                match disqualify_lead(lead_id, None).await {
                                                                                    Ok(_) => refresh.update(|n| *n += 1),
                                                                                    Err(e) => action_err.set(Some(e.to_string())),
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        "Disqualify"
                                                                    </button>
                                                                    <select
                                                                        class="form-select"
                                                                        style="font-size:0.75rem;padding:0.25rem 0.5rem;width:auto;"
                                                                        on:change=move |ev| {
                                                                            let v = event_target_value(&ev);
                                                                            let Some(st) = LeadPipelineStatus::from_api(&v) else {
                                                                                return;
                                                                            };
                                                                            if st.is_terminal() {
                                                                                return;
                                                                            }
                                                                            action_err.set(None);
                                                                            spawn_local(async move {
                                                                                match advance_lead_status(lead_id, st.as_str().to_string()).await {
                                                                                    Ok(_) => refresh.update(|n| *n += 1),
                                                                                    Err(e) => action_err.set(Some(e.to_string())),
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        {LeadPipelineStatus::ALL_SETTABLE.iter().copied().map(|s| {
                                                                            let selected = status_for_select == s.as_str();
                                                                            view! {
                                                                                <option value=s.as_str() selected=selected>
                                                                                    {s.label()}
                                                                                </option>
                                                                            }
                                                                        }).collect_view()}
                                                                    </select>
                                                                </div>
                                                            </Show>
                                                        </td>
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

            <Show when=move || show_add.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Add lead"</h3>
                            <button type="button" class="modal-close" on:click=move |_| show_add.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="form-field">
                                <label class="form-label">"First name"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    prop:value=first_name
                                    on:input=move |ev| first_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Last name"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    prop:value=last_name
                                    on:input=move |ev| last_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Email"</label>
                                <input
                                    type="email"
                                    class="form-input"
                                    prop:value=email
                                    on:input=move |ev| email.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Phone"</label>
                                <input
                                    type="tel"
                                    class="form-input"
                                    prop:value=phone
                                    on:input=move |ev| phone.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Company"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    prop:value=company
                                    on:input=move |ev| company.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Source"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    placeholder="web_form, referral, …"
                                    prop:value=source
                                    on:input=move |ev| source.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Message"</label>
                                <textarea
                                    class="form-input"
                                    rows="3"
                                    prop:value=message
                                    on:input=move |ev| message.set(event_target_value(&ev))
                                />
                            </div>
                            {move || create_err.get().map(|e| view! {
                                <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button type="button" class="folio-btn folio-btn--ghost" on:click=move |_| show_add.set(false)>
                                "Cancel"
                            </button>
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary"
                                disabled=move || creating.get()
                                on:click=on_create
                            >
                                {move || if creating.get() { "Saving…" } else { "Add lead" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

fn empty_to_none(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[derive(Serialize)]
struct CreateLeadBody {
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    company: Option<String>,
    source: Option<String>,
    message: Option<String>,
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

#[server(CreateLead, "/api")]
pub async fn create_lead(
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    company: Option<String>,
    source: Option<String>,
    message: Option<String>,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let body = CreateLeadBody {
        first_name,
        last_name,
        email,
        phone,
        company,
        source,
        message,
    };
    let lead = crate::atlas_client::authenticated_post::<CreateLeadBody, LeadRow>(
        "/api/folio/leads",
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Create lead failed: {e}")))?;
    Ok(lead.id)
}

#[server(QualifyLead, "/api")]
pub async fn qualify_lead(id: Uuid) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let path = format!("/api/folio/leads/{id}/qualify");
    crate::atlas_client::authenticated_post::<serde_json::Value, LeadRow>(
        &path,
        &token,
        None,
        &serde_json::json!({}),
    )
    .await
    .map(|_| ())
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Qualify failed: {e}")))
}

#[server(ConvertLead, "/api")]
pub async fn convert_lead(id: Uuid) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let path = format!("/api/folio/leads/{id}/convert");
    crate::atlas_client::authenticated_post::<serde_json::Value, LeadRow>(
        &path,
        &token,
        None,
        &serde_json::json!({}),
    )
    .await
    .map(|_| ())
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Convert failed: {e}")))
}

#[server(DisqualifyLead, "/api")]
pub async fn disqualify_lead(
    id: Uuid,
    reason: Option<String>,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let path = format!("/api/folio/leads/{id}/disqualify");
    let body = serde_json::json!({ "reason": reason });
    crate::atlas_client::authenticated_post::<serde_json::Value, LeadRow>(
        &path, &token, None, &body,
    )
    .await
    .map(|_| ())
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Disqualify failed: {e}")))
}

#[server(AdvanceLeadStatus, "/api")]
pub async fn advance_lead_status(
    id: Uuid,
    status: String,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let Some(st) = LeadPipelineStatus::from_api(&status) else {
        return Err(server_fn::error::ServerFnError::new("Invalid lead status"));
    };
    if st.is_terminal() {
        return Err(server_fn::error::ServerFnError::new(
            "Use convert/disqualify for terminal statuses",
        ));
    }
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let path = format!("/api/folio/leads/{id}/status");
    let body = serde_json::json!({ "status": st.as_str() });
    crate::atlas_client::authenticated_post::<serde_json::Value, LeadRow>(
        &path, &token, None, &body,
    )
    .await
    .map(|_| ())
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Status update failed: {e}")))
}
