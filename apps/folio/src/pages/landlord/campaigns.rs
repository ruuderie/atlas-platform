//! Campaigns — `/l/campaigns`
//! Wired to `GET/POST /api/folio/campaigns`.

use leptos::prelude::*;
use leptos::task::spawn_local;
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

/// Campaign channel type — mirrors backend `CampaignType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CampaignTypeOpt {
    ColdEmail,
    DirectMail,
    Ppc,
    Social,
    EventBased,
    Sms,
    Content,
    Referral,
    Retargeting,
}

impl CampaignTypeOpt {
    const ALL: &'static [Self] = &[
        Self::ColdEmail,
        Self::DirectMail,
        Self::Ppc,
        Self::Social,
        Self::EventBased,
        Self::Sms,
        Self::Content,
        Self::Referral,
        Self::Retargeting,
    ];

    const fn as_str(self) -> &'static str {
        match self {
            Self::ColdEmail => "cold_email",
            Self::DirectMail => "direct_mail",
            Self::Ppc => "ppc",
            Self::Social => "social",
            Self::EventBased => "event_based",
            Self::Sms => "sms",
            Self::Content => "content",
            Self::Referral => "referral",
            Self::Retargeting => "retargeting",
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::ColdEmail => "Cold email",
            Self::DirectMail => "Direct mail",
            Self::Ppc => "PPC",
            Self::Social => "Social",
            Self::EventBased => "Event",
            Self::Sms => "SMS",
            Self::Content => "Content",
            Self::Referral => "Referral",
            Self::Retargeting => "Retargeting",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|t| t.as_str() == s)
    }
}

/// Optional goal — mirrors backend `CampaignGoalType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CampaignGoalOpt {
    None,
    LeadCapture,
    Booking,
    Application,
    Sale,
    Registration,
    Subscription,
    Signup,
    OnboardingComplete,
}

impl CampaignGoalOpt {
    const ALL: &'static [Self] = &[
        Self::None,
        Self::LeadCapture,
        Self::Booking,
        Self::Application,
        Self::Sale,
        Self::Registration,
        Self::Subscription,
        Self::Signup,
        Self::OnboardingComplete,
    ];

    const fn as_api(self) -> Option<&'static str> {
        match self {
            Self::None => None,
            Self::LeadCapture => Some("lead_capture"),
            Self::Booking => Some("booking"),
            Self::Application => Some("application"),
            Self::Sale => Some("sale"),
            Self::Registration => Some("registration"),
            Self::Subscription => Some("subscription"),
            Self::Signup => Some("signup"),
            Self::OnboardingComplete => Some("onboarding_complete"),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::None => "No goal",
            Self::LeadCapture => "Lead capture",
            Self::Booking => "Booking",
            Self::Application => "Application",
            Self::Sale => "Sale",
            Self::Registration => "Registration",
            Self::Subscription => "Subscription",
            Self::Signup => "Signup",
            Self::OnboardingComplete => "Onboarding complete",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "lead_capture" => Self::LeadCapture,
            "booking" => Self::Booking,
            "application" => Self::Application,
            "sale" => Self::Sale,
            "registration" => Self::Registration,
            "subscription" => Self::Subscription,
            "signup" => Self::Signup,
            "onboarding_complete" => Self::OnboardingComplete,
            _ => Self::None,
        }
    }
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
    let refresh = RwSignal::new(0u32);
    let campaigns = Resource::new(move || refresh.get(), |_| async move { list_campaigns().await });

    let show_add = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let campaign_type = RwSignal::new(CampaignTypeOpt::ColdEmail.as_str().to_string());
    let goal_type = RwSignal::new(String::new());
    let budget = RwSignal::new(String::new());
    let creating = RwSignal::new(false);
    let create_err = RwSignal::new(None::<String>);

    let on_create = move |_| {
        let n = name.get().trim().to_string();
        if n.is_empty() {
            create_err.set(Some("Name is required.".into()));
            return;
        }
        let Some(ct) = CampaignTypeOpt::from_str(&campaign_type.get()) else {
            create_err.set(Some("Invalid campaign type.".into()));
            return;
        };
        let goal = CampaignGoalOpt::from_str(&goal_type.get());
        let budget_cents = budget
            .get()
            .trim()
            .parse::<f64>()
            .ok()
            .map(|d| (d * 100.0).round() as i64);
        creating.set(true);
        create_err.set(None);
        spawn_local(async move {
            match create_campaign(
                n,
                ct.as_str().to_string(),
                goal.as_api().map(|s| s.to_string()),
                budget_cents,
            )
            .await
            {
                Ok(_) => {
                    show_add.set(false);
                    name.set(String::new());
                    budget.set(String::new());
                    goal_type.set(String::new());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => create_err.set(Some(e.to_string())),
            }
            creating.set(false);
        });
    };

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Campaigns".to_string())
                subtitle=Signal::derive(|| "Marketing campaigns and outreach.".to_string())
            >
                <a class="folio-btn folio-btn--ghost press" href=FolioRoute::LandlordLeads.path()>
                    "Leads"
                </a>
                <button
                    type="button"
                    class="folio-btn folio-btn--primary press"
                    on:click=move |_| {
                        create_err.set(None);
                        show_add.set(true);
                    }
                >
                    "New campaign"
                </button>
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
                                    <button
                                        type="button"
                                        class="folio-btn folio-btn--primary press"
                                        style="margin-top:1rem;"
                                        on:click=move |_| {
                                            create_err.set(None);
                                            show_add.set(true);
                                        }
                                    >
                                        "New campaign"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="landlord-card-grid">
                                    {filtered.into_iter().map(|c| {
                                        let spent = fmt_money(c.spent_cents, c.currency.as_deref());
                                        let budget_s = c.budget_cents
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
                                                    {format!(" spent / {budget_s} budget")}
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

            <Show when=move || show_add.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"New campaign"</h3>
                            <button type="button" class="modal-close" on:click=move |_| show_add.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="folio-field">
                                <label class="folio-field__label">"Name *"</label>
                                <input
                                    type="text"
                                    class="folio-input"
                                    placeholder="Spring open house"
                                    prop:value=name
                                    on:input=move |ev| name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Type *"</label>
                                <select
                                    class="folio-select"
                                    on:change=move |ev| campaign_type.set(event_target_value(&ev))
                                >
                                    {CampaignTypeOpt::ALL.iter().copied().map(|t| {
                                        view! { <option value=t.as_str()>{t.label()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Goal"</label>
                                <select
                                    class="folio-select"
                                    on:change=move |ev| goal_type.set(event_target_value(&ev))
                                >
                                    {CampaignGoalOpt::ALL.iter().copied().map(|g| {
                                        let val = g.as_api().unwrap_or("");
                                        view! { <option value=val>{g.label()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            <div class="folio-field">
                                <label class="folio-field__label">"Budget ($)"</label>
                                <input
                                    type="number"
                                    class="folio-input"
                                    min="0"
                                    step="1"
                                    placeholder="Optional"
                                    prop:value=budget
                                    on:input=move |ev| budget.set(event_target_value(&ev))
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
                                disabled=move || creating.get() || name.get().trim().is_empty()
                                on:click=on_create
                            >
                                {move || if creating.get() { "Saving…" } else { "Create" }}
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
struct CreateCampaignBody {
    name: String,
    campaign_type: String,
    goal_type: Option<String>,
    budget_cents: Option<i64>,
    currency: Option<String>,
}

#[derive(Deserialize)]
struct CreateCampaignResponse {
    campaign: CampaignIdOnly,
}

#[derive(Deserialize)]
struct CampaignIdOnly {
    id: Uuid,
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

#[server(CreateLandlordCampaign, "/api")]
pub async fn create_campaign(
    name: String,
    campaign_type: String,
    goal_type: Option<String>,
    budget_cents: Option<i64>,
) -> Result<Uuid, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    if name.trim().is_empty() {
        return Err(server_fn::error::ServerFnError::new("Name is required"));
    }
    if CampaignTypeOpt::from_str(&campaign_type).is_none() {
        return Err(server_fn::error::ServerFnError::new("Invalid campaign type"));
    }

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let body = CreateCampaignBody {
        name: name.trim().to_string(),
        campaign_type,
        goal_type,
        budget_cents,
        currency: budget_cents.map(|_| "USD".to_string()),
    };
    let resp = crate::atlas_client::authenticated_post::<CreateCampaignBody, CreateCampaignResponse>(
        "/api/folio/campaigns",
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Create campaign failed: {e}")))?;
    Ok(resp.campaign.id)
}
