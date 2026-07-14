use crate::auth::{ServerFnError, SessionInfo};
use crate::components::page_header::PageHeader;
use crate::components::stat_card::StatCard;
use crate::pages::landlord::assets::list_assets;
use crate::pages::landlord::leads::list_leads;
use crate::pages::landlord::leases::{list_leases, LeaseStatus, LeaseSummary};
use crate::pages::landlord::maintenance_queue::{
    list_maintenance_tickets, CaseStatus, MaintenanceSummary,
};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn LandlordDashboard() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, ServerFnError>>>()
        .expect("Session context missing");
    let name = move || {
        session
            .get()
            .and_then(|r| r.ok())
            .and_then(|s| s.display_name)
            .unwrap_or_else(|| "there".into())
    };

    let leases = Resource::new(|| (), |_| async move { list_leases().await });
    let tickets = Resource::new(|| (), |_| async move { list_maintenance_tickets().await });
    let assets = Resource::new(|| (), |_| async move { list_assets().await });
    let leads = Resource::new(|| (), |_| async move { list_leads().await });

    let title = Signal::derive(move || format!("Welcome back, {}!", name()));
    let subtitle = Signal::derive(|| {
        "Here's what's happening across your portfolio today.".to_string()
    });

    let active_lease_count = Signal::derive(move || {
        leases
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .iter()
                    .filter(|l| LeaseStatus::from_str(&l.status) == LeaseStatus::Active)
                    .count()
            })
            .unwrap_or(0)
            .to_string()
    });

    let property_count = Signal::derive(move || {
        assets
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .iter()
                    .filter(|a| a.parent_asset_id.is_none())
                    .count()
            })
            .unwrap_or(0)
            .to_string()
    });

    let open_work_orders = Signal::derive(move || {
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

    let revenue_mtd = Signal::derive(move || {
        let cents: i64 = leases
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .iter()
                    .filter(|l| LeaseStatus::from_str(&l.status) == LeaseStatus::Active)
                    .filter_map(|l| l.monthly_rent_cents)
                    .sum()
            })
            .unwrap_or(0);
        format_money(cents)
    });

    let leads_count = Signal::derive(move || {
        leads
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .iter()
                    .filter(|l| !l.is_converted && !l.lead_status.eq_ignore_ascii_case("disqualified"))
                    .count()
            })
            .unwrap_or(0)
            .to_string()
    });

    view! {
        <div class="landlord-dash">
            <PageHeader title=title subtitle=subtitle>
                <A href="/l/leases" attr:class="folio-btn folio-btn--primary">
                    <span class="material-symbols-outlined">"add"</span>
                    "New lease"
                </A>
            </PageHeader>

            <div class="folio-stat-grid">
                <StatCard
                    label="Properties"
                    value=property_count
                    icon="domain"
                    href="/l/portfolio"
                />
                <StatCard
                    label="Active Leases"
                    value=active_lease_count
                    icon="description"
                    href="/l/leases"
                />
                <StatCard
                    label="Open Work Orders"
                    value=open_work_orders
                    icon="build"
                    href="/l/maintenance"
                />
                <StatCard
                    label="Monthly Rent Roll"
                    value=revenue_mtd
                    icon="payments"
                    href="/l/billing"
                />
                <StatCard
                    label="Open Leads"
                    value=leads_count
                    icon="person_search"
                    href="/l/leads"
                />
            </div>

            <div class="landlord-dash__sections">
                <section class="folio-section-card">
                    <div class="folio-section-card__header">
                        <h2 class="folio-section-card__title">"Needs attention"</h2>
                        <A
                            href="/l/maintenance"
                            attr:class="folio-btn folio-btn--ghost"
                            attr:style="padding:0.4rem 0.75rem;font-size:0.75rem"
                        >
                            "View all"
                        </A>
                    </div>
                    <div class="folio-section-card__body">
                        <Suspense fallback=|| view! {
                            <div class="folio-empty">
                                <p class="folio-empty__sub">"Loading…"</p>
                            </div>
                        }>
                            {move || {
                                let lease_items = leases.get().and_then(|r| r.ok()).unwrap_or_default();
                                let ticket_items = tickets.get().and_then(|r| r.ok()).unwrap_or_default();
                                let items = attention_items(&lease_items, &ticket_items);
                                if items.is_empty() {
                                    view! {
                                        <div class="folio-empty">
                                            <span class="material-symbols-outlined folio-empty__icon">"check_circle"</span>
                                            <p class="folio-empty__heading">"You're all caught up"</p>
                                            <p class="folio-empty__sub">
                                                "No expiring leases or open work orders right now. When something needs you, it shows up here."
                                            </p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <ul class="folio-attention-list">
                                            {items.into_iter().map(|item| {
                                                let href = item.href;
                                                let icon = item.icon;
                                                let label = item.label;
                                                let meta = item.meta;
                                                view! {
                                                    <li>
                                                        <A href=href attr:class="folio-attention-item">
                                                            <span class="material-symbols-outlined folio-attention-item__icon">
                                                                {icon}
                                                            </span>
                                                            <div class="folio-attention-item__text">
                                                                <p class="folio-attention-item__label">{label}</p>
                                                                <p class="folio-attention-item__meta">{meta}</p>
                                                            </div>
                                                            <span class="material-symbols-outlined folio-quick-action__chevron">
                                                                "chevron_right"
                                                            </span>
                                                        </A>
                                                    </li>
                                                }
                                            }).collect_view()}
                                        </ul>
                                    }.into_any()
                                }
                            }}
                        </Suspense>
                    </div>
                </section>

                <section class="folio-section-card">
                    <div class="folio-section-card__header">
                        <h2 class="folio-section-card__title">"Quick actions"</h2>
                    </div>
                    <div class="folio-section-card__body">
                        <div class="folio-quick-actions">
                            <A href="/l/leases" attr:class="folio-quick-action">
                                <span class="material-symbols-outlined folio-quick-action__icon">"description"</span>
                                "Add a lease"
                                <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                            </A>
                            <A href="/l/maintenance" attr:class="folio-quick-action">
                                <span class="material-symbols-outlined folio-quick-action__icon">"build"</span>
                                "Create work order"
                                <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                            </A>
                            <A href="/l/vendors" attr:class="folio-quick-action">
                                <span class="material-symbols-outlined folio-quick-action__icon">"handyman"</span>
                                "Dispatch a vendor"
                                <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                            </A>
                            <A href="/l/assets" attr:class="folio-quick-action">
                                <span class="material-symbols-outlined folio-quick-action__icon">"apartment"</span>
                                "Register an asset"
                                <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                            </A>
                        </div>
                    </div>
                </section>
            </div>
        </div>
    }
}

struct AttentionItem {
    icon: &'static str,
    label: String,
    meta: String,
    href: &'static str,
}

fn attention_items(
    leases: &[LeaseSummary],
    tickets: &[MaintenanceSummary],
) -> Vec<AttentionItem> {
    let mut items = Vec::new();
    let today = chrono::Utc::now().date_naive();

    for lease in leases.iter().filter(|l| {
        matches!(
            LeaseStatus::from_str(&l.status),
            LeaseStatus::Active | LeaseStatus::Pending
        )
    }) {
        if let Some(end) = lease.end_date {
            let days = (end - today).num_days();
            if (0..=60).contains(&days) {
                items.push(AttentionItem {
                    icon: "event_upcoming",
                    label: format!("Lease expiring in {days} days"),
                    meta: format!("Ends {}", end.format("%b %d, %Y")),
                    href: "/l/leases",
                });
            }
        }
    }

    for ticket in tickets.iter().filter(|t| {
        matches!(
            CaseStatus::from_str(&t.status),
            CaseStatus::Open | CaseStatus::InProgress
        )
    }) {
        let priority = if ticket.priority.eq_ignore_ascii_case("emergency") {
            "Emergency"
        } else {
            "Open"
        };
        items.push(AttentionItem {
            icon: if ticket.priority.eq_ignore_ascii_case("emergency") {
                "priority_high"
            } else {
                "build"
            },
            label: ticket.subject.clone(),
            meta: format!("{priority} · {}", ticket.status),
            href: "/l/maintenance",
        });
    }

    items.truncate(5);
    items
}

fn format_money(cents: i64) -> String {
    let dollars = cents as f64 / 100.0;
    if cents == 0 {
        "$0".to_string()
    } else {
        format!("${dollars:.0}")
    }
}
