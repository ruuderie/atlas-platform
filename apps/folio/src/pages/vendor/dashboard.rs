//! Vendor dashboard — `/v`
//! Wired to vendor work-orders + invoices list endpoints.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::stat_card::StatCard;
use crate::pages::vendor::invoices::{list_vendor_invoices, VendorInvoiceSummary};
use crate::pages::vendor::work_orders::{list_vendor_work_orders, WorkOrderSummary};

#[component]
pub fn VendorDashboard() -> impl IntoView {
    let orders = Resource::new(|| (), |_| async move { list_vendor_work_orders("active".into()).await });
    let invoices = Resource::new(|| (), |_| async move { list_vendor_invoices("all".into()).await });

    let open_wo = Signal::derive(move || {
        orders
            .get()
            .and_then(|r| r.ok())
            .map(|items: Vec<WorkOrderSummary>| items.len())
            .unwrap_or(0)
            .to_string()
    });

    let pending_inv = Signal::derive(move || {
        invoices
            .get()
            .and_then(|r| r.ok())
            .map(|items: Vec<VendorInvoiceSummary>| {
                items
                    .iter()
                    .filter(|i| i.status.eq_ignore_ascii_case("pending"))
                    .count()
            })
            .unwrap_or(0)
            .to_string()
    });

    let paid_inv = Signal::derive(move || {
        invoices
            .get()
            .and_then(|r| r.ok())
            .map(|items: Vec<VendorInvoiceSummary>| {
                items
                    .iter()
                    .filter(|i| i.status.eq_ignore_ascii_case("paid"))
                    .count()
            })
            .unwrap_or(0)
            .to_string()
    });

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Vendor Dashboard".to_string())
                subtitle=Signal::derive(|| "Active jobs and invoice status.".to_string())
            >
                <A href=FolioRoute::VendorSchedule.path() attr:class="folio-btn folio-btn--ghost press">
                    "Schedule"
                </A>
            </PageHeader>

            <div class="folio-stat-grid">
                <StatCard label="Active jobs" value=open_wo icon="build" href=FolioRoute::VendorWorkOrders.path()/>
                <StatCard label="Pending invoices" value=pending_inv icon="receipt_long" href=FolioRoute::VendorInvoices.path()/>
                <StatCard label="Paid invoices" value=paid_inv icon="payments" href=FolioRoute::VendorInvoices.path()/>
            </div>

            <div class="folio-quick-actions" style="margin-top:1.25rem;">
                <A href=FolioRoute::VendorWorkOrders.path() attr:class="folio-quick-action">
                    <span class="material-symbols-outlined folio-quick-action__icon">"assignment"</span>
                    "Work orders"
                    <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                </A>
                <A href=FolioRoute::VendorNetworkProfile.path() attr:class="folio-quick-action">
                    <span class="material-symbols-outlined folio-quick-action__icon">"badge"</span>
                    "Network profile"
                    <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                </A>
            </div>
        </div>
    }
}
