// apps/folio/src/pages/str_host/calendar.rs
//
// STR Calendar — /s/calendar
//
// Visual monthly calendar showing reservations as color-coded blocks.
// Each occupied day shows a chip. Uses /api/folio/reservations.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrReservation {
    pub id:                Uuid,
    pub status:            String,
    pub check_in:          String,
    pub check_out:         String,
    pub total_price_cents: i64,
    pub currency:          String,
    pub hold_expires_at:   Option<String>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchStrReservationsCalendar, "/api")]
pub async fn fetch_str_reservations_calendar() -> Result<Vec<StrReservation>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<StrReservation>>(
        "/api/folio/reservations", &token, None,
    ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[cfg(feature = "ssr")]
fn session_token(headers: &axum::http::HeaderMap) -> Result<String, server_fn::error::ServerFnError> {
    headers.get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find_map(|p| {
            let p = p.trim();
            p.strip_prefix("session=").map(|t| t.to_string())
        }))
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11               => 30,
        2 => if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 29 } else { 28 },
        _                            => 30,
    }
}

/// Check if a given (year, month, day) falls within check_in..check_out range.
fn day_is_booked(year: i32, month: u32, day: u32, r: &StrReservation) -> bool {
    let cell = format!("{:04}-{:02}-{:02}", year, month, day);
    let ci   = &r.check_in[..10.min(r.check_in.len())];
    let co   = &r.check_out[..10.min(r.check_out.len())];
    cell.as_str() >= ci && cell.as_str() < co
}

fn status_color(s: &str) -> &'static str {
    match s {
        "confirmed"   => "rgba(34,197,94,0.25)",
        "checked_in"  => "rgba(34,197,94,0.5)",
        "hold"        => "rgba(251,191,36,0.25)",
        "pending"     => "rgba(251,191,36,0.2)",
        "cancelled"   => "rgba(148,163,184,0.15)",
        _             => "rgba(10,132,255,0.15)",
    }
}

const MONTH_NAMES: &[&str] = &[
    "", "January","February","March","April","May","June",
    "July","August","September","October","November","December"
];
const DOW: &[&str] = &["Sun","Mon","Tue","Wed","Thu","Fri","Sat"];

// Simple day-of-week for 1st of month (Zeller's-adjacent, good enough for display)
fn first_dow(year: i32, month: u32) -> u32 {
    let y = if month < 3 { year - 1 } else { year } as u64;
    let m = if month < 3 { month + 12 } else { month } as u64;
    let k = y % 100;
    let j = y / 100;
    let h = (1 + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    ((h + 6) % 7) as u32  // 0=Sun
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StrCalendar() -> impl IntoView {
    // Current month/year (static initialise — in production use js_sys::Date)
    let year  = RwSignal::new(2026i32);
    let month = RwSignal::new(6u32);

    let res = Resource::new(|| (), |_| fetch_str_reservations_calendar());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Reservation Calendar"</h1>
                    <p class="page-subtitle">"Monthly occupancy view for all STR units"</p>
                </div>
                <div class="page-actions">
                    <a href="/s/reservations" class="btn btn-ghost btn-sm">"List View"</a>
                    <button class="btn btn-primary btn-sm" disabled=true>"+ Block Dates"</button>
                </div>
            </div>

            // ── Month navigation ──
            <div class="str-cal-nav">
                <button class="btn btn-ghost btn-sm" on:click=move |_| {
                    month.update(|m| if *m == 1 { *m = 12; year.update(|y| *y -= 1); } else { *m -= 1; });
                }>"‹"</button>
                <span class="str-cal-month-label">
                    {move || format!("{} {}", MONTH_NAMES[month.get() as usize], year.get())}
                </span>
                <button class="btn btn-ghost btn-sm" on:click=move |_| {
                    month.update(|m| if *m == 12 { *m = 1; year.update(|y| *y += 1); } else { *m += 1; });
                }>"›"</button>
            </div>

            // ── Calendar grid ──
            <Suspense fallback=|| view! { <div class="doc-empty">"Loading calendar…"</div> }>
                {move || res.get().map(|result| {
                    match result {
                        Ok(reservations) => {
                            let yr = year.get();
                            let mo = month.get();
                            let days = days_in_month(yr, mo);
                            let start_dow = first_dow(yr, mo);

                            view! {
                                <div class="str-cal-grid">
                                    // Day-of-week headers
                                    {DOW.iter().map(|d| view! {
                                        <div class="str-cal-dow">{*d}</div>
                                    }).collect::<Vec<_>>()}

                                    // Leading empty cells
                                    {(0..start_dow).map(|_| view! {
                                        <div class="str-cal-cell str-cal-cell--empty"></div>
                                    }).collect::<Vec<_>>()}

                                    // Day cells
                                    {(1..=days).map(|day| {
                                        let booked: Vec<_> = reservations.iter()
                                            .filter(|r| day_is_booked(yr, mo, day, r))
                                            .collect();
                                        let is_booked = !booked.is_empty();
                                        let bg = booked.first().map(|r| status_color(&r.status)).unwrap_or("transparent");
                                        let rid = booked.first().map(|r| r.id);
                                        view! {
                                            <div
                                                class=move || format!("str-cal-cell {}", if is_booked { "str-cal-cell--booked" } else { "" })
                                                style=format!("background:{bg}")
                                            >
                                                <span class="str-cal-day-num">{day.to_string()}</span>
                                                {if is_booked {
                                                    view! {
                                                        <span class="str-cal-booking-dot">
                                                            {booked.first().map(|r| match r.status.as_str() {
                                                                "confirmed"  => "✓",
                                                                "checked_in" => "↳",
                                                                "hold"       => "⏳",
                                                                _            => "•",
                                                            })}
                                                        </span>
                                                    }.into_any()
                                                } else { ().into_any() }}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>

                                // Legend
                                <div class="str-cal-legend">
                                    <span class="str-cal-legend-item" style="background:rgba(34,197,94,0.25)">"Confirmed"</span>
                                    <span class="str-cal-legend-item" style="background:rgba(34,197,94,0.5)">"Checked In"</span>
                                    <span class="str-cal-legend-item" style="background:rgba(251,191,36,0.25)">"Hold/Pending"</span>
                                    <span class="str-cal-legend-item" style="background:rgba(148,163,184,0.15)">"Cancelled"</span>
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
