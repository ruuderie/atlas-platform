// apps/folio/src/pages/landlord/tenant_profile.rs
//
// Tenant Profile page — /l/tenants/:id   (Landlord view)
//
// Shows the landlord a read-only dossier for a counterparty tenant:
//   - Identity: name, email, phone, member since
//   - Active lease(s) with this tenant (from GET /api/folio/leases filtered client-side)
//   - Rental application history (GET /api/folio/applications)
//   - Maintenance cases filed against their unit (GET /api/folio/cases)
//
// The user ID is the `counterparty_user_id` from atlas_contract.
// The backend authorises the lookup — only counterparties on the
// landlord's own contracts are accessible (403 otherwise).

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::components::nav::{FolioRoute, NavIcon};
use crate::pages::landlord::leases::{LeaseStatus, LeaseSummary};

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CounterpartyUser {
    pub id: uuid::Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationRecord {
    pub id: uuid::Uuid,
    pub applicant_user_id: uuid::Uuid,
    pub target_asset_id: Option<uuid::Uuid>,
    pub status: String,
    pub screening_status: String,
    pub screening_passed: Option<bool>,
    pub monthly_income_cents: Option<i64>,
    pub submitted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub decided_at: Option<chrono::DateTime<chrono::Utc>>,
    pub decision_reason: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Rental application decision status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationStatus {
    Submitted,
    Approved,
    Denied,
    Withdrawn,
    Pending,
    Unknown,
}

/// PATCH …/decision body vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationDecision {
    Approved,
    Denied,
}

impl ApplicationDecision {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Denied => "denied",
        }
    }
}

impl ApplicationStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "submitted" => Self::Submitted,
            "approved" => Self::Approved,
            "denied" => Self::Denied,
            "withdrawn" => Self::Withdrawn,
            "pending" => Self::Pending,
            _ => Self::Unknown,
        }
    }

    pub const fn is_decidable(self) -> bool {
        matches!(self, Self::Submitted | Self::Pending)
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Submitted => "Submitted",
            Self::Approved => "Approved",
            Self::Denied => "Denied",
            Self::Withdrawn => "Withdrawn",
            Self::Pending => "Pending",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Approved => "app-status--approved",
            Self::Submitted => "app-status--submitted",
            Self::Pending => "app-status--pending",
            Self::Denied => "app-status--denied",
            Self::Withdrawn => "app-status--withdrawn",
            Self::Unknown => "app-status--unknown",
        }
    }

    pub const fn material_icon(self) -> &'static str {
        match self {
            Self::Approved => "check_circle",
            Self::Submitted => "send",
            Self::Pending => "schedule",
            Self::Denied => "cancel",
            Self::Withdrawn => "undo",
            Self::Unknown => "help",
        }
    }
}

impl std::fmt::Display for ApplicationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Screening result displayed on the application row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreeningResult {
    Passed,
    Failed,
    Pending,
    NotRun,
}

impl ScreeningResult {
    pub fn from_parts(passed: Option<bool>, status: &str) -> Self {
        match (passed, status) {
            (Some(true), _) => Self::Passed,
            (Some(false), _) => Self::Failed,
            (None, "pending") => Self::Pending,
            _ => Self::NotRun,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "Passed",
            Self::Failed => "Failed",
            Self::Pending => "Pending",
            Self::NotRun => "N/A",
        }
    }

    pub const fn pill_class(self) -> &'static str {
        match self {
            Self::Passed => "screening--passed",
            Self::Failed => "screening--failed",
            Self::Pending => "screening--pending",
            Self::NotRun => "screening--na",
        }
    }
}

// ── Page ──────────────────────────────────────────────────────────────────────

/// Landlord read-only dossier for a counterparty tenant:
/// identity, leases, applications, and a maintenance summary.
#[component]
pub fn TenantProfile() -> impl IntoView {
    let params = use_params_map();
    let user_id = move || params.with(|p| p.get("id").unwrap_or_default());

    let profile = Resource::new(user_id, |id| async move { get_counterparty_user(id).await });

    // All leases — we filter client-side for those matching this counterparty.
    let all_leases = Resource::new(
        || (),
        |_| async move { crate::pages::landlord::leases::list_leases().await },
    );

    let app_refresh = RwSignal::new(0u32);
    let applications = Resource::new(move || app_refresh.get(), |_| async move { list_applications().await });
    let decide_err = RwSignal::new(None::<String>);
    let decide_pending = RwSignal::new(None::<uuid::Uuid>);
    let deny_for = RwSignal::new(None::<uuid::Uuid>);
    let deny_reason = RwSignal::new(String::new());

    view! {
        <div class="tp-page">
            // ── Back nav ──────────────────────────────────────────────
            <a href=FolioRoute::LandlordLeases.path() class="tp-back-link">
                <span class="material-symbols-outlined" style="font-size:16px;">
                    {NavIcon::ArrowBack.as_str()}
                </span>
                "Back to Leases"
            </a>

            // ── Profile header ────────────────────────────────────────
            <Suspense fallback=|| view! { <div class="tp-header-skeleton"/> }>
                {move || profile.get().map(|res| match res {
                    Err(e) => view! {
                        <div class="tp-error">
                            <span class="material-symbols-outlined" style="font-size:2.5rem;">
                                {NavIcon::Report.as_str()}
                            </span>
                            <p class="tp-error-text">{format!("Could not load tenant profile: {e}")}</p>
                        </div>
                    }.into_any(),
                    Ok(user) => {
                        let initials = format!(
                            "{}{}",
                            user.first_name.chars().next().unwrap_or(' ').to_uppercase().to_string(),
                            user.last_name.chars().next().unwrap_or(' ').to_uppercase().to_string()
                        );
                        let full_name = format!("{} {}", user.first_name, user.last_name);
                        let member_since = user.created_at.format("%B %Y").to_string();

                        view! {
                            <div class="tp-header">
                                <div class="tp-avatar">{initials}</div>
                                <div class="tp-header-info">
                                    <h1 class="tp-name">{full_name}</h1>
                                    <div class="tp-meta-row">
                                        <span class="tp-meta-item">
                                            <span class="material-symbols-outlined" style="font-size:14px;">
                                                "mail"
                                            </span>
                                            {user.email.clone()}
                                        </span>
                                        <span class="tp-meta-sep">"\u{00b7}"</span>
                                        <span class="tp-meta-item">
                                            <span class="material-symbols-outlined" style="font-size:14px;">
                                                "phone"
                                            </span>
                                            {user.phone.clone()}
                                        </span>
                                        <span class="tp-meta-sep">"\u{00b7}"</span>
                                        <span class="tp-meta-item">
                                            <span class="material-symbols-outlined" style="font-size:14px;">
                                                "calendar_month"
                                            </span>
                                            {format!("Member since {member_since}")}
                                        </span>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    }
                })}
            </Suspense>

            // ── Body ──────────────────────────────────────────────────
            <div class="tp-body">
                <div class="tp-col-main">

                    // ── Lease history ─────────────────────────────────
                    <div class="tp-card">
                        <h2 class="tp-card-title">
                            <span class="material-symbols-outlined" style="font-size:18px;">
                                {NavIcon::Description.as_str()}
                            </span>
                            "Lease History"
                        </h2>
                        <Suspense fallback=|| view! { <div class="tp-skel tp-skel--table"/> }>
                            {move || {
                                let uid = user_id();
                                all_leases.get().map(|res| match res {
                                    Err(e) => view! {
                                        <p class="tp-empty-text">{format!("Could not load leases: {e}")}</p>
                                    }.into_any(),
                                    Ok(leases) => {
                                        let related: Vec<LeaseSummary> = leases
                                            .into_iter()
                                            .filter(|l| {
                                                l.counterparty_user_id
                                                    .map(|id| id.to_string() == uid)
                                                    .unwrap_or(false)
                                            })
                                            .collect();

                                        if related.is_empty() {
                                            view! {
                                                <div class="tp-empty-inline">
                                                    <span class="material-symbols-outlined tp-empty-icon">
                                                        {NavIcon::Description.as_str()}
                                                    </span>
                                                    <p class="tp-empty-text">"No leases found for this tenant."</p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="tp-lease-list">
                                                    {related.into_iter().map(|l| {
                                                        let status = LeaseStatus::from_str(&l.status);
                                                        let id_str = l.id.to_string();
                                                        let detail_href = format!("/l/leases/{id_str}");
                                                        let rent = l.monthly_rent_cents.map(|c| {
                                                            format!("{} {:.2}", l.currency, c as f64 / 100.0)
                                                        }).unwrap_or_else(|| "\u{2014}".to_string());
                                                        let start = l.start_date
                                                            .map(|d| d.to_string())
                                                            .unwrap_or_else(|| "\u{2014}".to_string());
                                                        let end = l.end_date
                                                            .map(|d| d.to_string())
                                                            .unwrap_or_else(|| "Open".to_string());
                                                        view! {
                                                            <div class="tp-lease-row">
                                                                <div class="tp-lease-status">
                                                                    <span class={format!("lease-status-badge {}", status.pill_class())}>
                                                                        <span class="material-symbols-outlined" style="font-size:11px;">
                                                                            {status.material_icon()}
                                                                        </span>
                                                                        {status.as_str()}
                                                                    </span>
                                                                </div>
                                                                <div class="tp-lease-details">
                                                                    <span class="tp-lease-rent">{rent}</span>
                                                                    <span class="tp-lease-period">
                                                                        {format!("{start} \u{2192} {end}")}
                                                                    </span>
                                                                </div>
                                                                <a href=detail_href class="tp-lease-link">
                                                                    "View lease"
                                                                    <span class="material-symbols-outlined" style="font-size:13px;">
                                                                        {NavIcon::ChevronRight.as_str()}
                                                                    </span>
                                                                </a>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                    }
                                })
                            }}
                        </Suspense>
                    </div>

                    // ── Application history ───────────────────────────
                    <div class="tp-card">
                        <h2 class="tp-card-title">
                            <span class="material-symbols-outlined" style="font-size:18px;">
                                {NavIcon::Assignment.as_str()}
                            </span>
                            "Rental Applications"
                        </h2>
                        {move || decide_err.get().map(|e| view! {
                            <p class="tp-empty-text" style="color:#b91c1c;">{e}</p>
                        })}
                        <Suspense fallback=|| view! { <div class="tp-skel tp-skel--table"/> }>
                            {move || {
                                let uid = user_id();
                                applications.get().map(|res| match res {
                                    Err(e) => view! {
                                        <p class="tp-empty-text">{format!("Could not load applications: {e}")}</p>
                                    }.into_any(),
                                    Ok(apps) => {
                                        let related: Vec<ApplicationRecord> = apps
                                            .into_iter()
                                            .filter(|a| a.applicant_user_id.to_string() == uid)
                                            .collect();

                                        if related.is_empty() {
                                            view! {
                                                <div class="tp-empty-inline">
                                                    <span class="material-symbols-outlined tp-empty-icon">
                                                        {NavIcon::Assignment.as_str()}
                                                    </span>
                                                    <p class="tp-empty-text">"No rental applications on record."</p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <table class="tp-app-table">
                                                    <thead>
                                                        <tr>
                                                            <th class="tp-app-th">"Status"</th>
                                                            <th class="tp-app-th">"Screening"</th>
                                                            <th class="tp-app-th">"Income"</th>
                                                            <th class="tp-app-th">"Submitted"</th>
                                                            <th class="tp-app-th">"Decided"</th>
                                                            <th class="tp-app-th">"Actions"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        {related.into_iter().map(|app| {
                                                            let app_id = app.id;
                                                            let status = ApplicationStatus::from_str(&app.status);
                                                            let decidable = status.is_decidable();
                                                            let screening = ScreeningResult::from_parts(
                                                                app.screening_passed,
                                                                &app.screening_status,
                                                            );
                                                            let income = app.monthly_income_cents
                                                                .map(|c| format!("${:.0}/mo", c as f64 / 100.0))
                                                                .unwrap_or_else(|| "\u{2014}".to_string());
                                                            let submitted = app.submitted_at
                                                                .map(|d| d.format("%Y-%m-%d").to_string())
                                                                .unwrap_or_else(|| "\u{2014}".to_string());
                                                            let decided = app.decided_at
                                                                .map(|d| d.format("%Y-%m-%d").to_string())
                                                                .unwrap_or_else(|| "\u{2014}".to_string());
                                                            let offer_href = match app.target_asset_id {
                                                                Some(aid) => format!(
                                                                    "{}?asset_id={}&user_id={}",
                                                                    FolioRoute::LandlordLeaseCreate.path(),
                                                                    aid,
                                                                    app.applicant_user_id
                                                                ),
                                                                None => format!(
                                                                    "{}?user_id={}",
                                                                    FolioRoute::LandlordLeaseCreate.path(),
                                                                    app.applicant_user_id
                                                                ),
                                                            };
                                                            let is_approved = status == ApplicationStatus::Approved;
                                                            view! {
                                                                <tr class="tp-app-row">
                                                                    <td class="tp-app-td">
                                                                        <span class={format!("app-status-badge {}", status.pill_class())}>
                                                                            <span class="material-symbols-outlined" style="font-size:10px;">
                                                                                {status.material_icon()}
                                                                            </span>
                                                                            {status.as_str()}
                                                                        </span>
                                                                    </td>
                                                                    <td class="tp-app-td">
                                                                        <span class={format!("screening-badge {}", screening.pill_class())}>
                                                                            {screening.as_str()}
                                                                        </span>
                                                                    </td>
                                                                    <td class="tp-app-td tp-app-td--mono">{income}</td>
                                                                    <td class="tp-app-td">{submitted}</td>
                                                                    <td class="tp-app-td">{decided}</td>
                                                                    <td class="tp-app-td">
                                                                        {if is_approved {
                                                                            view! {
                                                                                <a
                                                                                    class="folio-btn folio-btn--primary press"
                                                                                    style="font-size:0.75rem;padding:0.25rem 0.5rem;"
                                                                                    href=offer_href.clone()
                                                                                >
                                                                                    "Offer lease"
                                                                                </a>
                                                                            }.into_any()
                                                                        } else if decidable {
                                                                            view! {
                                                                                <div class="unit-actions" style="gap:0.35rem;">
                                                                                    <button
                                                                                        type="button"
                                                                                        class="folio-btn folio-btn--primary press"
                                                                                        style="font-size:0.75rem;padding:0.25rem 0.5rem;"
                                                                                        disabled=move || decide_pending.get() == Some(app_id)
                                                                                        on:click=move |_| {
                                                                                            decide_pending.set(Some(app_id));
                                                                                            decide_err.set(None);
                                                                                            deny_for.set(None);
                                                                                            spawn_local(async move {
                                                                                                match decide_application(
                                                                                                    app_id.to_string(),
                                                                                                    ApplicationDecision::Approved.as_str().to_string(),
                                                                                                    None,
                                                                                                ).await {
                                                                                                    Ok(_) => app_refresh.update(|n| *n += 1),
                                                                                                    Err(e) => decide_err.set(Some(e.to_string())),
                                                                                                }
                                                                                                decide_pending.set(None);
                                                                                            });
                                                                                        }
                                                                                    >
                                                                                        "Approve"
                                                                                    </button>
                                                                                    <button
                                                                                        type="button"
                                                                                        class="folio-btn folio-btn--ghost press"
                                                                                        style="font-size:0.75rem;padding:0.25rem 0.5rem;"
                                                                                        disabled=move || decide_pending.get() == Some(app_id)
                                                                                        on:click=move |_| {
                                                                                            deny_for.set(Some(app_id));
                                                                                            deny_reason.set(String::new());
                                                                                            decide_err.set(None);
                                                                                        }
                                                                                    >
                                                                                        "Deny"
                                                                                    </button>
                                                                                </div>
                                                                                <Show when=move || deny_for.get() == Some(app_id)>
                                                                                    <div style="margin-top:0.5rem;display:flex;flex-direction:column;gap:0.35rem;">
                                                                                        <input
                                                                                            type="text"
                                                                                            class="folio-input"
                                                                                            placeholder="Denial reason (required)"
                                                                                            prop:value=move || deny_reason.get()
                                                                                            on:input=move |ev| deny_reason.set(event_target_value(&ev))
                                                                                        />
                                                                                        <button
                                                                                            type="button"
                                                                                            class="folio-btn folio-btn--primary press"
                                                                                            style="font-size:0.75rem;padding:0.25rem 0.5rem;"
                                                                                            disabled=move || {
                                                                                                decide_pending.get() == Some(app_id)
                                                                                                    || deny_reason.get().trim().is_empty()
                                                                                            }
                                                                                            on:click=move |_| {
                                                                                                let reason = deny_reason.get().trim().to_string();
                                                                                                if reason.is_empty() {
                                                                                                    decide_err.set(Some("Denial requires a reason.".into()));
                                                                                                    return;
                                                                                                }
                                                                                                decide_pending.set(Some(app_id));
                                                                                                spawn_local(async move {
                                                                                                    match decide_application(
                                                                                                        app_id.to_string(),
                                                                                                        ApplicationDecision::Denied.as_str().to_string(),
                                                                                                        Some(reason),
                                                                                                    ).await {
                                                                                                        Ok(_) => {
                                                                                                            deny_for.set(None);
                                                                                                            app_refresh.update(|n| *n += 1);
                                                                                                        }
                                                                                                        Err(e) => decide_err.set(Some(e.to_string())),
                                                                                                    }
                                                                                                    decide_pending.set(None);
                                                                                                });
                                                                                            }
                                                                                        >
                                                                                            "Confirm deny"
                                                                                        </button>
                                                                                    </div>
                                                                                </Show>
                                                                            }.into_any()
                                                                        } else {
                                                                            ().into_any()
                                                                        }}
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }).collect_view()}
                                                    </tbody>
                                                </table>
                                            }.into_any()
                                        }
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                </div>

                // Right column
                <div class="tp-col-side">
                    <div class="tp-card">
                        <h2 class="tp-card-title">
                            <span class="material-symbols-outlined" style="font-size:18px;">
                                "quick_reference"
                            </span>
                            "Quick Actions"
                        </h2>
                        <div class="tp-quick-links">
                            <a
                                href=FolioRoute::LandlordMaintenance.path()
                                class="tp-quick-link"
                            >
                                <span class="material-symbols-outlined" style="font-size:18px;">
                                    {NavIcon::Build.as_str()}
                                </span>
                                "Open Work Order"
                            </a>
                            <a
                                href=FolioRoute::LandlordLeases.path()
                                class="tp-quick-link"
                            >
                                <span class="material-symbols-outlined" style="font-size:18px;">
                                    {NavIcon::Description.as_str()}
                                </span>
                                "View All Leases"
                            </a>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ── Server functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

/// GET /api/folio/users/:id — counterparty identity lookup.
#[server(GetCounterpartyUser, "/api")]
pub async fn get_counterparty_user(
    user_id: String,
) -> Result<CounterpartyUser, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    if user_id.is_empty() {
        return Err(server_fn::error::ServerFnError::new("Missing user ID"));
    }
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<CounterpartyUser>(
        &format!("/api/folio/users/{user_id}"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("User lookup failed: {e}")))
}

/// GET /api/folio/applications — landlord view of all applications.
#[server(ListApplications, "/api")]
pub async fn list_applications() -> Result<Vec<ApplicationRecord>, server_fn::error::ServerFnError>
{
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<ApplicationRecord>>(
        "/api/folio/applications",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Applications failed: {e}")))
}

#[derive(Serialize)]
struct DecideApplicationBody {
    decision: String,
    reason: Option<String>,
}

/// PATCH /api/folio/applications/{id}/decision
#[server(DecideApplication, "/api")]
pub async fn decide_application(
    application_id: String,
    decision: String,
    reason: Option<String>,
) -> Result<ApplicationRecord, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let decision_enum = match decision.as_str() {
        "approved" => ApplicationDecision::Approved,
        "denied" => ApplicationDecision::Denied,
        _ => {
            return Err(server_fn::error::ServerFnError::new(
                "Decision must be approved or denied",
            ))
        }
    };
    if decision_enum == ApplicationDecision::Denied
        && reason.as_deref().map(str::trim).unwrap_or("").is_empty()
    {
        return Err(server_fn::error::ServerFnError::new(
            "Denial requires a reason",
        ));
    }
    let application_id = uuid::Uuid::parse_str(application_id.trim())
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid application ID"))?;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let body = DecideApplicationBody {
        decision: decision_enum.as_str().to_string(),
        reason,
    };
    crate::atlas_client::authenticated_patch::<DecideApplicationBody, ApplicationRecord>(
        &format!("/api/folio/applications/{application_id}/decision"),
        &token,
        body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Decision failed: {e}")))
}
