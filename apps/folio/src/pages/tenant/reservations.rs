//! Tenant / guest reservations — `/t/reservations`
//! Wired to `GET /api/folio/reservations`.

use leptos::prelude::*;

use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::reservations::{list_reservations, ReservationSummary};

fn tone(status: &str) -> StatusPillTone {
    match status.to_ascii_lowercase().as_str() {
        "confirmed" | "checked_in" => StatusPillTone::Ok,
        "hold" | "pending" => StatusPillTone::Warn,
        "cancelled" | "no_show" => StatusPillTone::Danger,
        _ => StatusPillTone::Neutral,
    }
}

#[component]
pub fn TenantReservations() -> impl IntoView {
    let reservations = Resource::new(|| (), |_| async move { list_reservations().await });

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Reservations".to_string())
                subtitle=Signal::derive(|| "Your short-term stays and upcoming bookings.".to_string())
            />

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
                    Ok(items) if items.is_empty() => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"event_available"</span>
                            <p class="folio-empty__heading">"No stays booked"</p>
                            <p class="folio-empty__sub">"Upcoming and past STR reservations appear here."</p>
                        </div>
                    }.into_any(),
                    Ok(items) => view! {
                        <For
                            each=move || items.clone()
                            key=|r: &ReservationSummary| r.id
                            children=move |r| {
                                let dates = format!(
                                    "{} → {}",
                                    r.check_in.format("%b %d"),
                                    r.check_out.format("%b %d, %Y")
                                );
                                let total = format!("${:.0}", r.total_price_cents as f64 / 100.0);
                                view! {
                                    <div class="hub-activity-rail__row">
                                        <StatusPill label=r.status.clone() tone=tone(&r.status)/>
                                        <div class="hub-activity-rail__body">
                                            <p class="hub-activity-rail__row-title">{dates}</p>
                                            <p class="hub-activity-rail__row-meta">{format!("{total} · {}", r.currency)}</p>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
