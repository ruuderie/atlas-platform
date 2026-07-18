//! Tenant maintenance — `/t/maintenance`
//! Wired to `GET /api/folio/maintenance`.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::maintenance_queue::{
    list_maintenance_tickets, CaseStatus, MaintenanceSummary,
};

fn tone(status: &str) -> StatusPillTone {
    match CaseStatus::from_str(status) {
        CaseStatus::Open | CaseStatus::InProgress => StatusPillTone::Warn,
        CaseStatus::Completed => StatusPillTone::Ok,
        _ => StatusPillTone::Neutral,
    }
}

#[component]
pub fn MaintenanceRequests() -> impl IntoView {
    let tickets = Resource::new(|| (), |_| async move { list_maintenance_tickets().await });

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Maintenance".to_string())
                subtitle=Signal::derive(|| "Submit and track maintenance requests.".to_string())
            >
                <A href=FolioRoute::TenantMaintenanceNew.path() attr:class="folio-btn folio-btn--primary press">
                    "New request"
                </A>
            </PageHeader>

            <Suspense fallback=|| view! {
                <div class="folio-empty"><p class="folio-empty__sub">"Loading requests…"</p></div>
            }>
                {move || tickets.get().map(|result| match result {
                    Err(e) => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"error"</span>
                            <p class="folio-empty__heading">"Could not load maintenance"</p>
                            <p class="folio-empty__sub">{e.to_string()}</p>
                        </div>
                    }.into_any(),
                    Ok(items) if items.is_empty() => view! {
                        <div class="folio-empty">
                            <span class="material-symbols-outlined folio-empty__icon">"build"</span>
                            <p class="folio-empty__heading">"No requests yet"</p>
                            <p class="folio-empty__sub">"Report a plumbing, HVAC, or general issue to get started."</p>
                            <A href=FolioRoute::TenantMaintenanceNew.path() attr:class="folio-btn folio-btn--primary press">
                                "Create request"
                            </A>
                        </div>
                    }.into_any(),
                    Ok(items) => view! {
                        <For
                            each=move || items.clone()
                            key=|t: &MaintenanceSummary| t.id
                            children=move |t| {
                                let href = FolioRoute::TenantMaintenanceDetail
                                    .path()
                                    .replace(":id", &t.id.to_string());
                                view! {
                                    <a class="hub-activity-rail__row press" href=href>
                                        <StatusPill label=t.status.clone() tone=tone(&t.status)/>
                                        <div class="hub-activity-rail__body">
                                            <p class="hub-activity-rail__row-title">{t.subject.clone()}</p>
                                            <p class="hub-activity-rail__row-meta">
                                                {format!("{} · {}", t.priority, t.created_at.format("%b %d"))}
                                            </p>
                                        </div>
                                    </a>
                                }
                            }
                        />
                    }.into_any(),
                })}
            </Suspense>
        </div>
    }
}
