//! Landlord reservations — `/l/reservations`
//! Wired to `GET /api/folio/reservations` + lifecycle POSTs.

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReservationSummary {
    pub id: Uuid,
    pub status: String,
    pub check_in: chrono::DateTime<chrono::Utc>,
    pub check_out: chrono::DateTime<chrono::Utc>,
    pub total_price_cents: i64,
    pub currency: String,
    pub hold_expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReservationFilter {
    All,
    Confirmed,
    Hold,
    CheckedIn,
    Cancelled,
}

impl ReservationFilter {
    const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Confirmed => "Confirmed",
            Self::Hold => "Hold",
            Self::CheckedIn => "Checked in",
            Self::Cancelled => "Cancelled",
        }
    }

    fn matches(self, status: &str) -> bool {
        let s = status.to_ascii_lowercase();
        match self {
            Self::All => true,
            Self::Confirmed => s == "confirmed",
            Self::Hold => s == "hold" || s == "pending",
            Self::CheckedIn => s == "checked_in",
            Self::Cancelled => s == "cancelled" || s == "no_show" || s == "checked_out",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReservationAction {
    Confirm,
    CheckIn,
    CheckOut,
    Cancel,
}

impl ReservationAction {
    const fn label(self) -> &'static str {
        match self {
            Self::Confirm => "Confirm",
            Self::CheckIn => "Check in",
            Self::CheckOut => "Check out",
            Self::Cancel => "Cancel",
        }
    }
}

fn status_tone(status: &str) -> StatusPillTone {
    match status.to_ascii_lowercase().as_str() {
        "confirmed" | "checked_in" => StatusPillTone::Ok,
        "hold" | "pending" => StatusPillTone::Warn,
        "cancelled" | "no_show" | "checked_out" => StatusPillTone::Danger,
        _ => StatusPillTone::Neutral,
    }
}

fn is_terminal(status: &str) -> bool {
    matches!(
        status.to_ascii_lowercase().as_str(),
        "cancelled" | "no_show" | "checked_out"
    )
}

fn primary_action(status: &str) -> Option<ReservationAction> {
    match status.to_ascii_lowercase().as_str() {
        "hold" | "pending" => Some(ReservationAction::Confirm),
        "confirmed" => Some(ReservationAction::CheckIn),
        "checked_in" => Some(ReservationAction::CheckOut),
        _ => None,
    }
}

#[component]
pub fn LandlordReservations() -> impl IntoView {
    let filter = RwSignal::new(ReservationFilter::All);
    let refresh = RwSignal::new(0u32);
    let action_err = RwSignal::new(None::<String>);
    let action_pending = RwSignal::new(None::<Uuid>);
    let reservations = Resource::new(move || refresh.get(), |_| async move { list_reservations().await });

    let run_action = move |id: Uuid, action: ReservationAction| {
        action_pending.set(Some(id));
        action_err.set(None);
        spawn_local(async move {
            let result = match action {
                ReservationAction::Confirm => reservation_confirm(id.to_string()).await,
                ReservationAction::CheckIn => reservation_check_in(id.to_string()).await,
                ReservationAction::CheckOut => reservation_check_out(id.to_string()).await,
                ReservationAction::Cancel => {
                    reservation_cancel(id.to_string(), None).await
                }
            };
            match result {
                Ok(_) => refresh.update(|n| *n += 1),
                Err(e) => action_err.set(Some(e.to_string())),
            }
            action_pending.set(None);
        });
    };

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Reservations".to_string())
                subtitle=Signal::derive(|| "STR bookings across your properties.".to_string())
            >
                <a class="folio-btn folio-btn--ghost press" href=FolioRoute::LandlordMap.path()>
                    "Map"
                </a>
            </PageHeader>

            {move || action_err.get().map(|e| view! {
                <p style="color:#b91c1c;margin-bottom:0.75rem;font-size:0.875rem;">{e}</p>
            })}

            <div class="landlord-filter-chips" style="margin-bottom:1rem;">
                {[
                    ReservationFilter::All,
                    ReservationFilter::Confirmed,
                    ReservationFilter::Hold,
                    ReservationFilter::CheckedIn,
                    ReservationFilter::Cancelled,
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
                <div class="folio-empty"><p class="folio-empty__sub">"Loading reservations…"</p></div>
            }>
                {move || reservations.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load reservations"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(items) => {
                        let f = filter.get();
                        let filtered: Vec<_> = items.into_iter().filter(|r| f.matches(&r.status)).collect();
                        if filtered.is_empty() {
                            view! {
                                <div class="folio-empty">
                                    <span class="material-symbols-outlined folio-empty__icon">"event_available"</span>
                                    <p class="folio-empty__heading">"No reservations"</p>
                                    <p class="folio-empty__sub">
                                        "When guests book STR units, stays show up here."
                                    </p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="landlord-card-grid">
                                    {filtered.into_iter().map(|r| {
                                        let check_in = r.check_in.format("%b %d, %Y").to_string();
                                        let check_out = r.check_out.format("%b %d, %Y").to_string();
                                        let total = format!(
                                            "${:.0} {}",
                                            r.total_price_cents as f64 / 100.0,
                                            r.currency
                                        );
                                        let tone = status_tone(&r.status);
                                        let rid = r.id;
                                        let primary = primary_action(&r.status);
                                        let can_cancel = !is_terminal(&r.status);
                                        view! {
                                            <div class="landlord-card landlord-card--static">
                                                <div class="landlord-card__top">
                                                    <span class="material-symbols-outlined landlord-card__icon">"event_available"</span>
                                                    <StatusPill label=r.status.clone() tone=tone/>
                                                </div>
                                                <h3 class="landlord-card__title">{format!("{check_in} → {check_out}")}</h3>
                                                <p class="landlord-card__meta">{total}</p>
                                                <p class="landlord-card__meta" style="font-family:monospace;font-size:0.7rem;">
                                                    {r.id.to_string().chars().take(8).collect::<String>()}
                                                </p>
                                                <div class="unit-actions" style="margin-top:0.75rem;">
                                                    {primary.map(|action| {
                                                        let a = action;
                                                        view! {
                                                            <button
                                                                type="button"
                                                                class="folio-btn folio-btn--primary press"
                                                                disabled=move || action_pending.get() == Some(rid)
                                                                on:click=move |_| run_action(rid, a)
                                                            >
                                                                {a.label()}
                                                            </button>
                                                        }
                                                    })}
                                                    {can_cancel.then(|| view! {
                                                        <button
                                                            type="button"
                                                            class="folio-btn folio-btn--ghost press"
                                                            disabled=move || action_pending.get() == Some(rid)
                                                            on:click=move |_| run_action(rid, ReservationAction::Cancel)
                                                        >
                                                            "Cancel"
                                                        </button>
                                                    })}
                                                </div>
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

#[server(ListLandlordReservations, "/api")]
pub async fn list_reservations() -> Result<Vec<ReservationSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<ReservationSummary>>(
        "/api/folio/reservations",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Reservation list failed: {e}")))
}

#[cfg(feature = "ssr")]
async fn post_reservation_action(
    id: &str,
    suffix: &str,
    body: serde_json::Value,
) -> Result<ReservationSummary, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let rid = Uuid::parse_str(id.trim())
        .map_err(|_| server_fn::error::ServerFnError::new("Invalid reservation ID"))?;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_post::<serde_json::Value, ReservationSummary>(
        &format!("/api/folio/reservations/{rid}/{suffix}"),
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Reservation {suffix} failed: {e}")))
}

#[server(ReservationConfirm, "/api")]
pub async fn reservation_confirm(
    reservation_id: String,
) -> Result<ReservationSummary, server_fn::error::ServerFnError> {
    post_reservation_action(&reservation_id, "confirm", serde_json::json!({})).await
}

#[server(ReservationCheckIn, "/api")]
pub async fn reservation_check_in(
    reservation_id: String,
) -> Result<ReservationSummary, server_fn::error::ServerFnError> {
    post_reservation_action(&reservation_id, "check-in", serde_json::json!({})).await
}

#[server(ReservationCheckOut, "/api")]
pub async fn reservation_check_out(
    reservation_id: String,
) -> Result<ReservationSummary, server_fn::error::ServerFnError> {
    post_reservation_action(&reservation_id, "check-out", serde_json::json!({})).await
}

#[server(ReservationCancel, "/api")]
pub async fn reservation_cancel(
    reservation_id: String,
    reason: Option<String>,
) -> Result<ReservationSummary, server_fn::error::ServerFnError> {
    post_reservation_action(
        &reservation_id,
        "cancel",
        serde_json::json!({ "reason": reason }),
    )
    .await
}
