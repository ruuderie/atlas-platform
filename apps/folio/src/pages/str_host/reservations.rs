// apps/folio/src/pages/str_host/reservations.rs
//
// STR Reservation Manifest — /s/reservations
//
// Full list of all reservations with guest manifest link for each.
// Uses /api/folio/reservations + /api/folio/reservations/{id}/manifest.
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

#[server(FetchStrReservationsList, "/api")]
pub async fn fetch_str_reservations_list(
) -> Result<Vec<StrReservation>, server_fn::error::ServerFnError> {
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

fn status_chip_cls(s: &str) -> &'static str {
    match s {
        "confirmed" | "checked_in" => "ph-badge--paid",
        "pending" | "hold" => "ph-badge--pending",
        "cancelled" | "no_show" => "ph-badge--overdue",
        _ => "ph-badge--default",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StrReservationManifest() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let status_filter = RwSignal::new("all".to_string());

    let res = Resource::new(move || refresh.get(), |_| fetch_str_reservations_list());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Reservations"</h1>
                    <p class="page-subtitle">"All STR bookings with guest manifest access"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>"↻"</button>
                    <a href="/s/calendar" class="btn btn-ghost btn-sm">"📅 Calendar"</a>
                </div>
            </div>

            // ── Status filters ──
            <div class="mkt-controls" style="margin-bottom:1rem;">
                <div class="mkt-filters">
                    {
                        let pill = move |v: &'static str, label: &'static str| view! {
                            <button
                                class=move || format!("filter-pill {}", if status_filter.get() == v { "filter-pill--active" } else { "" })
                                on:click=move |_| status_filter.set(v.to_string())
                            >{label}</button>
                        };
                        view! {
                            {pill("all",         "All")}
                            {pill("confirmed",   "Confirmed")}
                            {pill("checked_in",  "Checked In")}
                            {pill("hold",        "On Hold")}
                            {pill("pending",     "Pending")}
                            {pill("cancelled",   "Cancelled")}
                        }
                    }
                </div>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading reservations…"</div> }>
                {move || res.get().map(|result| {
                    match result {
                        Ok(reservations) => {
                            let sf = status_filter.get();
                            let filtered: Vec<_> = reservations.into_iter()
                                .filter(|r| sf == "all" || r.status.contains(&sf))
                                .collect();

                            if filtered.is_empty() {
                                return view! { <div class="doc-empty">"No reservations match filter."</div> }.into_any();
                            }

                            view! {
                                <div class="owner-table-wrap">
                                    <table class="ph-table">
                                        <thead>
                                            <tr>
                                                <th>"Check-In"</th>
                                                <th>"Check-Out"</th>
                                                <th>"Total"</th>
                                                <th>"Status"</th>
                                                <th>"Manifest"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            <For
                                                each=move || filtered.clone()
                                                key=|r| r.id
                                                children=move |r| {
                                                    let sc   = status_chip_cls(&r.status);
                                                    let ci   = r.check_in.chars().take(10).collect::<String>();
                                                    let co   = r.check_out.chars().take(10).collect::<String>();
                                                    let rid  = r.id;
                                                    let exp  = r.hold_expires_at.clone();
                                                    view! {
                                                        <tr class="ph-row">
                                                            <td class="ph-date">{ci}</td>
                                                            <td class="ph-date">{co}</td>
                                                            <td class="ph-amount">{fmt_usd(r.total_price_cents)}</td>
                                                            <td>
                                                                <span class=format!("ph-badge {sc}")>{r.status.replace('_', " ")}</span>
                                                                {exp.map(|e| view! {
                                                                    <span class="text-xs text-amber-400 ml-1">"⏳ " {e.chars().take(10).collect::<String>()}</span>
                                                                })}
                                                            </td>
                                                            <td>
                                                                <a href=format!("/s/reservations/{}", rid) class="btn btn-ghost btn-sm">"Manifest →"</a>
                                                            </td>
                                                        </tr>
                                                    }
                                                }
                                            />
                                        </tbody>
                                    </table>
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
