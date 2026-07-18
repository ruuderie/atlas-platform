//! Tenant dashboard — `/t`
//! Wired to leases, maintenance, and ledger APIs.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::stat_card::StatCard;
use crate::pages::landlord::leases::{list_leases, LeaseStatus};
use crate::pages::landlord::maintenance_queue::{
    list_maintenance_tickets, CaseStatus,
};
use crate::pages::tenant::payment_history::fetch_ledger;

#[component]
pub fn TenantDashboard() -> impl IntoView {
    let leases = Resource::new(|| (), |_| async move { list_leases().await });
    let tickets = Resource::new(|| (), |_| async move { list_maintenance_tickets().await });
    let ledger = Resource::new(|| (), |_| async move { fetch_ledger().await });

    let active_lease = Signal::derive(move || {
        leases
            .get()
            .and_then(|r| r.ok())
            .and_then(|items| {
                items
                    .into_iter()
                    .find(|l| LeaseStatus::from_str(&l.status) == LeaseStatus::Active)
            })
    });

    let rent = Signal::derive(move || {
        active_lease
            .get()
            .and_then(|l| l.monthly_rent_cents)
            .map(|c| format!("${:.0}", c as f64 / 100.0))
            .unwrap_or_else(|| "—".into())
    });

    let open_maint = Signal::derive(move || {
        tickets
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .iter()
                    .filter(|t| {
                        matches!(
                            CaseStatus::from_str(&t.status),
                            CaseStatus::Open | CaseStatus::InProgress
                        )
                    })
                    .count()
            })
            .unwrap_or(0)
            .to_string()
    });

    let due_soon = Signal::derive(move || {
        ledger
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .iter()
                    .filter(|e| {
                        let s = e.status.to_ascii_lowercase();
                        s == "pending" || s == "overdue" || s == "processing"
                    })
                    .count()
            })
            .unwrap_or(0)
            .to_string()
    });

    let lease_status = Signal::derive(move || {
        active_lease
            .get()
            .map(|l| LeaseStatus::from_str(&l.status).as_str().to_string())
            .unwrap_or_else(|| "No active lease".into())
    });

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "My Dashboard".to_string())
                subtitle=Signal::derive(|| "Lease, payments, and requests.".to_string())
            >
                <A href=FolioRoute::TenantMaintenanceNew.path() attr:class="folio-btn folio-btn--primary press">
                    "Request maintenance"
                </A>
            </PageHeader>

            <div class="folio-stat-grid">
                <StatCard label="Lease" value=lease_status icon="description" href=FolioRoute::TenantMyLease.path()/>
                <StatCard label="Monthly rent" value=rent icon="payments" href=FolioRoute::TenantPayments.path()/>
                <StatCard label="Open requests" value=open_maint icon="build" href=FolioRoute::TenantMaintenance.path()/>
                <StatCard label="Unpaid invoices" value=due_soon icon="receipt_long" href=FolioRoute::TenantPaymentHistory.path()/>
            </div>

            <div class="landlord-dash__sections" style="margin-top:1.25rem;">
                <section class="folio-section-card">
                    <div class="folio-section-card__header">
                        <h2 class="folio-section-card__title">"Quick links"</h2>
                    </div>
                    <div class="folio-section-card__body">
                        <div class="folio-quick-actions">
                            <A href=FolioRoute::TenantMyLease.path() attr:class="folio-quick-action">
                                <span class="material-symbols-outlined folio-quick-action__icon">"description"</span>
                                "View lease"
                                <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                            </A>
                            <A href=FolioRoute::TenantPayments.path() attr:class="folio-quick-action">
                                <span class="material-symbols-outlined folio-quick-action__icon">"credit_card"</span>
                                "Payments"
                                <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                            </A>
                            <A href=FolioRoute::TenantInbox.path() attr:class="folio-quick-action">
                                <span class="material-symbols-outlined folio-quick-action__icon">"inbox"</span>
                                "Inbox"
                                <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                            </A>
                        </div>
                    </div>
                </section>
            </div>
        </div>
    }
}
