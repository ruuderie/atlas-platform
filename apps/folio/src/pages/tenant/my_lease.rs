//! My Lease — `/t/my-lease`
//! Wired to `GET /api/folio/leases` (active lease for the tenant account).

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::leases::{list_leases, LeaseStatus, LeaseSummary};

fn tone_for(status: LeaseStatus) -> StatusPillTone {
    match status {
        LeaseStatus::Active => StatusPillTone::Ok,
        LeaseStatus::Pending => StatusPillTone::Warn,
        LeaseStatus::Expired | LeaseStatus::Terminated => StatusPillTone::Danger,
        _ => StatusPillTone::Neutral,
    }
}

#[component]
pub fn MyLease() -> impl IntoView {
    let leases = Resource::new(|| (), |_| async move { list_leases().await });

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "My Lease".to_string())
                subtitle=Signal::derive(|| "Your current lease agreement.".to_string())
            >
                <A href=FolioRoute::TenantDocuments.path() attr:class="folio-btn folio-btn--ghost press">
                    "Documents"
                </A>
            </PageHeader>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading lease…"</p></div>
            }>
                {move || leases.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load lease"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(items) => {
                        let active: Option<LeaseSummary> = items.iter()
                            .find(|l| LeaseStatus::from_str(&l.status) == LeaseStatus::Active)
                            .cloned()
                            .or_else(|| items.first().cloned());
                        match active {
                            None => view! {
                                <div class="folio-empty">
                                    <span class="material-symbols-outlined folio-empty__icon">"description"</span>
                                    <p class="folio-empty__heading">"No lease on file"</p>
                                    <p class="folio-empty__sub">
                                        "When your landlord activates a lease for your account, it appears here."
                                    </p>
                                </div>
                            }.into_any(),
                            Some(l) => {
                                let st = LeaseStatus::from_str(&l.status);
                                let rent = l.monthly_rent_cents
                                    .map(|c| format!("${:.0} / mo", c as f64 / 100.0))
                                    .unwrap_or_else(|| "—".into());
                                let dates = match (l.start_date, l.end_date) {
                                    (Some(s), Some(e)) => format!("{s} → {e}"),
                                    (Some(s), None) => format!("From {s}"),
                                    _ => "Dates not set".into(),
                                };
                                let detail = FolioRoute::LandlordLeaseDetail
                                    .path()
                                    .replace(":id", &l.id.to_string());
                                // Tenant namespace has no lease detail route — use payments/docs.
                                let _ = detail;
                                view! {
                                    <div class="landlord-card landlord-card--static" style="max-width:32rem;">
                                        <div class="landlord-card__top">
                                            <span class="material-symbols-outlined landlord-card__icon">"description"</span>
                                            <StatusPill label=st.as_str().to_string() tone=tone_for(st)/>
                                        </div>
                                        <h3 class="landlord-card__title">{rent}</h3>
                                        <p class="landlord-card__meta">{dates}</p>
                                        <p class="landlord-card__meta">{l.currency.clone()}</p>
                                        <div class="unit-actions" style="margin-top:1rem;">
                                            <A href=FolioRoute::TenantPayments.path() attr:class="folio-btn folio-btn--primary press">
                                                "Payments"
                                            </A>
                                            <A href=FolioRoute::TenantHousehold.path() attr:class="folio-btn folio-btn--ghost press">
                                                "Household"
                                            </A>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }
                    }
                })}
            </Suspense>
        </div>
    }
}
