//! Landlord reservations — `/l/reservations`
//! Wired to `GET /api/folio/reservations`.

use leptos::prelude::*;
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
            Self::Cancelled => s == "cancelled" || s == "no_show",
        }
    }
}

fn status_tone(status: &str) -> StatusPillTone {
    match status.to_ascii_lowercase().as_str() {
        "confirmed" | "checked_in" => StatusPillTone::Ok,
        "hold" | "pending" => StatusPillTone::Warn,
        "cancelled" | "no_show" => StatusPillTone::Danger,
        _ => StatusPillTone::Neutral,
    }
}

#[component]
pub fn LandlordReservations() -> impl IntoView {
    let filter = RwSignal::new(ReservationFilter::All);
    let reservations = Resource::new(|| (), |_| async move { list_reservations().await });

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
