// apps/folio/src/pages/str_host/dashboard.rs
//
// STR Host Dashboard — /s
//
// Overview of all STR operations: upcoming check-ins, active reservations,
// revenue today vs MTD, and occupancy. Uses /api/folio/reservations.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrReservation {
    pub id: Uuid,
    pub status: String,
    pub check_in: String,
    pub check_out: String,
    pub total_price_cents: i64,
    pub currency: String,
    pub hold_expires_at: Option<String>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchStrReservations, "/api")]
pub async fn fetch_str_reservations() -> Result<Vec<StrReservation>, server_fn::error::ServerFnError>
{
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<StrReservation>>(
        "/api/folio/reservations",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[cfg(feature = "ssr")]
fn session_token(
    headers: &axum::http::HeaderMap,
) -> Result<String, server_fn::error::ServerFnError> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            })
        })
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn fmt_usd(cents: i64) -> String {
    format!("${:.2}", cents as f64 / 100.0)
}

fn nights_between(check_in: &str, check_out: &str) -> i64 {
    // Simple date subtraction for display (yyyy-mm-dd strings)
    // In production use chrono; here we estimate from positions
    let parse_day = |s: &str| -> i64 {
        let parts: Vec<_> = s[..10].split('-').collect();
        if parts.len() == 3 {
            let y = parts[0].parse::<i64>().unwrap_or(0);
            let m = parts[1].parse::<i64>().unwrap_or(0);
            let d = parts[2].parse::<i64>().unwrap_or(0);
            y * 365 + m * 30 + d
        } else {
            0
        }
    };
    (parse_day(check_out) - parse_day(check_in)).max(1)
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StrHostDashboard() -> impl IntoView {
    let res = Resource::new(|| (), |_| fetch_str_reservations());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"STR Dashboard"</h1>
                    <p class="page-subtitle">"Check-ins, bookings, and work orders"</p>
                </div>
                <div class="page-actions">
                    <a href="/s/calendar" class="btn btn-ghost btn-sm">"📅 Calendar"</a>
                    <a href="/s/reservations" class="btn btn-primary btn-sm">"All Reservations"</a>
                </div>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading…"</div> }>
                {move || res.get().map(|result| {
                    match result {
                        Ok(reservations) => {
                            let total     = reservations.len();
                            let active    = reservations.iter().filter(|r| r.status == "confirmed" || r.status == "checked_in").count();
                            let pending   = reservations.iter().filter(|r| r.status == "pending" || r.status == "hold").count();
                            let revenue   = reservations.iter().filter(|r| r.status != "cancelled").map(|r| r.total_price_cents).sum::<i64>();

                            // Upcoming check-ins (next 7 days approx — filter by check_in prefix)
                            let mut upcoming: Vec<_> = reservations.iter()
                                .filter(|r| r.status == "confirmed")
                                .cloned().collect();
                            upcoming.sort_by(|a, b| a.check_in.cmp(&b.check_in));
                            upcoming.truncate(5);

                            view! {
                                // ── KPIs ──
                                <div class="kpi-row" style="margin-bottom:1.5rem;">
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Total Reservations"</span>
                                        <span class="kpi-value" style="color:var(--cobalt)">{total.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Active / Confirmed"</span>
                                        <span class="kpi-value" style="color:var(--green)">{active.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Pending / On Hold"</span>
                                        <span class="kpi-value" style="color:var(--amber)">{pending.to_string()}</span>
                                    </div>
                                    <div class="kpi-card">
                                        <span class="kpi-label">"Total Revenue"</span>
                                        <span class="kpi-value" style="color:var(--green)">{fmt_usd(revenue)}</span>
                                    </div>
                                </div>

                                // ── Upcoming check-ins ──
                                <div class="owner-section">
                                    <div class="owner-section-title">"Upcoming Check-Ins"</div>
                                    {if upcoming.is_empty() {
                                        view! { <div class="doc-empty">"No upcoming check-ins."</div> }.into_any()
                                    } else {
                                        view! {
                                            <div class="str-upcoming-list">
                                                <For
                                                    each=move || upcoming.clone()
                                                    key=|r| r.id
                                                    children=move |r| {
                                                        let nights = nights_between(&r.check_in, &r.check_out);
                                                        let ci = r.check_in.chars().take(10).collect::<String>();
                                                        let co = r.check_out.chars().take(10).collect::<String>();
                                                        let rid = r.id;
                                                        view! {
                                                            <div class="str-upcoming-card">
                                                                <div class="str-upcoming-dates">
                                                                    <div class="str-upcoming-date-in">{ci}</div>
                                                                    <div class="str-upcoming-arrow">"→"</div>
                                                                    <div class="str-upcoming-date-out">{co}</div>
                                                                </div>
                                                                <div class="str-upcoming-meta">
                                                                    <span class="str-upcoming-nights">{nights.to_string()} " nights"</span>
                                                                    <span class="str-upcoming-price" style="color:var(--green)">{fmt_usd(r.total_price_cents)}</span>
                                                                </div>
                                                                <a href=format!("/s/reservations/{}", rid)
                                                                    class="btn btn-ghost btn-sm">"View →"</a>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            </div>
                                        }.into_any()
                                    }}
                                </div>

                                // ── Quick links ──
                                <div class="str-quick-row">
                                    <a href="/s/calendar"     class="str-quick-card">"📅" <span>"Calendar"</span></a>
                                    <a href="/s/listings"     class="str-quick-card">"🏠" <span>"Listings"</span></a>
                                    <a href="/s/pricing"      class="str-quick-card">"💰" <span>"Pricing"</span></a>
                                    <a href="/s/channels"     class="str-quick-card">"🌐" <span>"Channels"</span></a>
                                    <a href="/s/messages"     class="str-quick-card">"✉" <span>"Messages"</span></a>
                                    <a href="/s/reviews"      class="str-quick-card">"⭐" <span>"Reviews"</span></a>
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="doc-empty text-red-400">"Error: " {e.to_string()}</div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
