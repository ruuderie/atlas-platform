// apps/folio/src/pages/landlord/asset_alerts.rs
//
// Asset Alerts — /l/assets/:id/alerts
//
// Per-asset alert preferences (GET/PUT …/alert-prefs).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::components::nav::FolioRoute;

// ── Alert type vocabulary (matches backend AssetAlertType) ────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetAlertTypeId {
    PaymentOverdue,
    PaymentFailed,
    Vacancy,
    LeaseExpiring,
    MaintenanceOpen,
    InspectionDue,
    StrPermitExpiry,
    ViolationFiled,
}

impl AssetAlertTypeId {
    pub const ALL: &'static [Self] = &[
        Self::PaymentOverdue,
        Self::PaymentFailed,
        Self::Vacancy,
        Self::LeaseExpiring,
        Self::MaintenanceOpen,
        Self::InspectionDue,
        Self::StrPermitExpiry,
        Self::ViolationFiled,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PaymentOverdue => "payment_overdue",
            Self::PaymentFailed => "payment_failed",
            Self::Vacancy => "vacancy",
            Self::LeaseExpiring => "lease_expiring",
            Self::MaintenanceOpen => "maintenance_open",
            Self::InspectionDue => "inspection_due",
            Self::StrPermitExpiry => "str_permit_expiry",
            Self::ViolationFiled => "violation_filed",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::PaymentOverdue => "Payment Overdue",
            Self::PaymentFailed => "Payment Failed",
            Self::Vacancy => "Unit Vacant",
            Self::LeaseExpiring => "Lease Expiring",
            Self::MaintenanceOpen => "Open Maintenance",
            Self::InspectionDue => "Inspection Due",
            Self::StrPermitExpiry => "STR Permit Expiry",
            Self::ViolationFiled => "Violation Filed",
        }
    }

    pub const fn desc(self) -> &'static str {
        match self {
            Self::PaymentOverdue => "Tenant payment not received by due date",
            Self::PaymentFailed => "Payment attempt rejected by payment rail",
            Self::Vacancy => "Unit is unoccupied for more than N days",
            Self::LeaseExpiring => "Active lease expires within 60 days",
            Self::MaintenanceOpen => "Maintenance request unresolved for 7+ days",
            Self::InspectionDue => "Scheduled inspection approaching or overdue",
            Self::StrPermitExpiry => "Short-term rental permit expiring within 30 days",
            Self::ViolationFiled => "New compliance violation filed on this asset",
        }
    }

    pub const fn icon(self) -> &'static str {
        match self {
            Self::PaymentOverdue => "💰",
            Self::PaymentFailed => "❌",
            Self::Vacancy => "🚪",
            Self::LeaseExpiring => "📋",
            Self::MaintenanceOpen => "🔧",
            Self::InspectionDue => "🔍",
            Self::StrPermitExpiry => "📜",
            Self::ViolationFiled => "⚠️",
        }
    }

    pub const fn category(self) -> &'static str {
        match self {
            Self::PaymentOverdue | Self::PaymentFailed => "Financial",
            Self::Vacancy | Self::LeaseExpiring => "Occupancy",
            Self::MaintenanceOpen | Self::InspectionDue => "Maintenance",
            Self::StrPermitExpiry | Self::ViolationFiled => "Compliance",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|t| t.as_str() == s)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AlertPrefsBody {
    enabled: Vec<String>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(GetAssetAlertPrefs, "/api")]
pub async fn get_asset_alert_prefs(
    asset_id: String,
) -> Result<Vec<String>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let asset_id = uuid::Uuid::parse_str(asset_id.trim())
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid asset ID"))?;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let resp = crate::atlas_client::authenticated_get::<AlertPrefsBody>(
        &format!("/api/folio/assets/{asset_id}/alert-prefs"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Load alert prefs failed: {e}")))?;
    Ok(resp.enabled)
}

#[server(PutAssetAlertPrefs, "/api")]
pub async fn put_asset_alert_prefs(
    asset_id: String,
    enabled: Vec<String>,
) -> Result<Vec<String>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let asset_id = uuid::Uuid::parse_str(asset_id.trim())
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid asset ID"))?;
    for id in &enabled {
        if AssetAlertTypeId::parse(id).is_none() {
            return Err(server_fn::error::ServerFnError::new(format!(
                "Unknown alert type: {id}"
            )));
        }
    }
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let body = AlertPrefsBody { enabled };
    let resp = crate::atlas_client::authenticated_put::<AlertPrefsBody, AlertPrefsBody>(
        &format!("/api/folio/assets/{asset_id}/alert-prefs"),
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Save alert prefs failed: {e}")))?;
    Ok(resp.enabled)
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn AssetAlerts() -> impl IntoView {
    let params = use_params_map();
    let asset_id = Signal::derive(move || params.with(|p| p.get("id").unwrap_or_default()));
    let aid_disp = Signal::derive(move || {
        let asset_id = asset_id.get();
        if asset_id.len() > 8 {
            format!("…{}", &asset_id[asset_id.len() - 8..])
        } else {
            asset_id
        }
    });

    let enabled: RwSignal<std::collections::HashSet<String>> = RwSignal::new(std::collections::HashSet::new());
    let save_pending = RwSignal::new(false);
    let saved = RwSignal::new(false);
    let load_err = RwSignal::new(None::<String>);
    let save_err = RwSignal::new(None::<String>);

    let prefs = Resource::new(
        move || asset_id.get(),
        |id| async move {
            if id.is_empty() {
                return Err(server_fn::error::ServerFnError::new("Missing asset ID"));
            }
            get_asset_alert_prefs(id).await
        },
    );

    Effect::new(move |_| {
        if let Some(res) = prefs.get() {
            match res {
                Ok(ids) => {
                    enabled.set(ids.into_iter().collect());
                    load_err.set(None);
                }
                Err(e) => load_err.set(Some(e.to_string())),
            }
        }
    });

    let on_save = move |_| {
        let id = asset_id.get();
        let mut ids: Vec<String> = enabled.get().into_iter().collect();
        ids.sort();
        save_pending.set(true);
        saved.set(false);
        save_err.set(None);
        spawn_local(async move {
            match put_asset_alert_prefs(id, ids).await {
                Ok(next) => {
                    enabled.set(next.into_iter().collect());
                    saved.set(true);
                }
                Err(e) => save_err.set(Some(e.to_string())),
            }
            save_pending.set(false);
        });
    };

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <a href=FolioRoute::LandlordAssets.path() class="back-link">"← Back to Assets"</a>
                    <h1 class="page-title">"Asset Alerts"</h1>
                    <p class="page-subtitle">"Alerts for asset " <code class="font-mono text-xs">{move || aid_disp.get()}</code></p>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-primary btn-sm"
                        disabled=move || save_pending.get()
                        on:click=on_save
                    >
                        {move || if save_pending.get() { "Saving…" } else { "Save Preferences" }}
                    </button>
                </div>
            </div>

            {move || load_err.get().map(|e| view! {
                <div class="viol-info-banner" style="margin-bottom:1rem;color:#b91c1c;">
                    <p class="viol-info-text">{format!("Could not load preferences: {e}")}</p>
                </div>
            })}

            {move || if saved.get() {
                view! {
                    <div class="alert-saved-toast">"✓ Alert preferences saved"</div>
                }.into_any()
            } else { ().into_any() }}

            {move || save_err.get().map(|e| view! {
                <div class="alert-saved-toast" style="color:#b91c1c;">{format!("Save failed: {e}")}</div>
            })}

            <div class="viol-info-banner" style="margin-bottom:1.25rem;">
                <span class="viol-info-icon">"📡"</span>
                <p class="viol-info-text">
                    "Alerts are delivered via your notification channels (Email, Telegram, SMS). "
                    <a href=FolioRoute::LandlordNotifications.path() style="color:#60a5fa;">"Manage delivery channels →"</a>
                </p>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading preferences…"</div> }>
                {move || prefs.get().map(|_| {
                    let categories = ["Financial", "Occupancy", "Maintenance", "Compliance"];
                    view! {
                        {
                            categories.into_iter().map(|cat| {
                                let type_rows: Vec<_> = AssetAlertTypeId::ALL
                                    .iter()
                                    .copied()
                                    .filter(|t| t.category() == cat)
                                    .collect();
                                view! {
                                    <div class="alerts-section">
                                        <div class="alerts-section-title">{cat}</div>
                                        <div class="alerts-type-list">
                                            {type_rows.into_iter().map(|at| {
                                                let at_id = at.as_str().to_string();
                                                let at_id_check = at_id.clone();
                                                view! {
                                                    <div class="alerts-type-row">
                                                        <span class="alerts-type-icon">{at.icon()}</span>
                                                        <div class="alerts-type-body">
                                                            <div class="alerts-type-title">{at.title()}</div>
                                                            <div class="alerts-type-desc">{at.desc()}</div>
                                                        </div>
                                                        <label class="syndic-toggle-wrap">
                                                            <input
                                                                type="checkbox"
                                                                class="syndic-toggle-input"
                                                                prop:checked=move || enabled.get().contains(&at_id_check)
                                                                on:change=move |ev: web_sys::Event| {
                                                                    let el = event_target::<web_sys::HtmlInputElement>(&ev);
                                                                    let key = at_id.clone();
                                                                    enabled.update(|s| {
                                                                        if el.checked() { s.insert(key); }
                                                                        else { s.remove(&key); }
                                                                    });
                                                                    saved.set(false);
                                                                }
                                                            />
                                                            <span class="syndic-toggle-track"></span>
                                                        </label>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        }
                    }.into_any()
                })}
            </Suspense>

        </div>
    }
}
