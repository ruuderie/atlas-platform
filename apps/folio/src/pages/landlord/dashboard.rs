//! Landlord dashboard — mode-aware LTR / STR / All attention surface.

use crate::auth::{ServerFnError, SessionInfo};
use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::stat_card::StatCard;
use crate::pages::landlord::assets::list_assets;
use crate::pages::landlord::leads::list_leads;
use crate::pages::landlord::leases::{list_leases, LeaseStatus, LeaseSummary};
use crate::pages::landlord::maintenance_queue::{
    list_maintenance_tickets, CaseStatus, MaintenanceSummary,
};
use crate::pages::landlord::str_compliance::list_str_permits;
use leptos::prelude::*;
use leptos_router::components::A;

/// Dashboard attention lens — typed at the UI boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DashboardMode {
    All,
    Ltr,
    Str,
}

impl DashboardMode {
    const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Ltr => "LTR",
            Self::Str => "STR",
        }
    }
}

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

    let mode = RwSignal::new(DashboardMode::Ltr);
    let mode_user_set = RwSignal::new(false);

    // Infer default mode once data + session are available.
    Effect::new(move |_| {
        if mode_user_set.get() {
            return;
        }
        let has_str = session
            .get()
            .and_then(|r| r.ok())
            .map(|s| s.has_str_assets)
            .unwrap_or(false);
        let asset_list = assets.get().and_then(|r| r.ok());
        let lease_list = leases.get().and_then(|r| r.ok());
        let Some(asset_list) = asset_list else { return };
        let Some(lease_list) = lease_list else { return };

        let str_count = asset_list.iter().filter(|a| a.str_eligible).count();
        let active_ltr = lease_list
            .iter()
            .filter(|l| LeaseStatus::from_str(&l.status) == LeaseStatus::Active)
            .count();
        let inferred = if (has_str || str_count > 0) && active_ltr == 0 {
            DashboardMode::Str
        } else if str_count > 0 && active_ltr > 0 {
            DashboardMode::All
        } else {
            DashboardMode::Ltr
        };
        mode.set(inferred);
    });

    let title = Signal::derive(move || format!("Welcome back, {}!", name()));
    let subtitle = Signal::derive(move || match mode.get() {
        DashboardMode::Ltr => "Long-term rentals — leases, rent roll, and unit turns.".to_string(),
        DashboardMode::Str => {
            "Short-term holdings — listings, turnover, and open work orders.".to_string()
        }
        DashboardMode::All => {
            "Leases, listings, and open work orders.".to_string()
        }
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

    let open_work_orders = Signal::derive(move || {
        let m = mode.get();
        let str_ids: std::collections::HashSet<_> = assets
            .get()
            .and_then(|r| r.ok())
            .map(|items| {
                items
                    .into_iter()
                    .filter(|a| a.str_eligible)
                    .map(|a| a.id)
                    .collect()
            })
            .unwrap_or_default();
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
                    .filter(|t| match m {
                        DashboardMode::Str => t
                            .asset_id
                            .map(|id| str_ids.contains(&id))
                            .unwrap_or(false),
                        DashboardMode::Ltr => t
                            .asset_id
                            .map(|id| !str_ids.contains(&id))
                            .unwrap_or(true),
                        DashboardMode::All => true,
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

    let str_eligible_count = Signal::derive(move || {
        assets
            .get()
            .and_then(|r| r.ok())
            .map(|items| items.iter().filter(|a| a.str_eligible).count())
            .unwrap_or(0)
            .to_string()
    });

    let str_listed_count = Signal::derive(move || {
        assets
            .get()
            .and_then(|r| r.ok())
            .map(|items| items.iter().filter(|a| a.str_listing_active).count())
            .unwrap_or(0)
            .to_string()
    });

    let permits = Resource::new(|| (), |_| async move { list_str_permits().await });
    let compliance_count = Signal::derive(move || {
        permits
            .get()
            .and_then(|r| r.ok())
            .map(|items| items.len().to_string())
            .unwrap_or_else(|| "—".into())
    });

    let map_maint_href = format!(
        "{}?layer=maintenance",
        FolioRoute::LandlordMap.path()
    );

    view! {
        <div class="landlord-dash">
            <PageHeader title=title subtitle=subtitle>
                <A href=FolioRoute::LandlordLeaseCreate.path() attr:class="folio-btn folio-btn--primary press">
                    <span class="material-symbols-outlined">"add"</span>
                    "New lease"
                </A>
            </PageHeader>

            <div class="dash-mode-bar" role="tablist" aria-label="Dashboard mode">
                {[DashboardMode::All, DashboardMode::Ltr, DashboardMode::Str]
                    .into_iter()
                    .map(|m| {
                        view! {
                            <button
                                type="button"
                                role="tab"
                                class=move || {
                                    if mode.get() == m {
                                        "dash-mode-btn dash-mode-btn--active"
                                    } else {
                                        "dash-mode-btn"
                                    }
                                }
                                on:click=move |_| {
                                    mode_user_set.set(true);
                                    mode.set(m);
                                }
                            >
                                {m.label()}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            <Show when=move || matches!(mode.get(), DashboardMode::Ltr | DashboardMode::All)>
                <div class="folio-stat-grid" style="margin-bottom:1rem;">
                    <StatCard
                        label="Properties"
                        value=property_count
                        icon="domain"
                        href=FolioRoute::LandlordAssets.path()
                    />
                    <StatCard
                        label="Active Leases"
                        value=active_lease_count
                        icon="description"
                        href=FolioRoute::LandlordLeases.path()
                    />
                    <StatCard
                        label="Monthly Rent Roll"
                        value=revenue_mtd
                        icon="payments"
                        href=FolioRoute::LandlordBilling.path()
                    />
                    <StatCard
                        label="Open Leads"
                        value=leads_count
                        icon="person_search"
                        href=FolioRoute::LandlordLeads.path()
                    />
                    <StatCard
                        label="Open Work Orders"
                        value=open_work_orders
                        icon="build"
                        href=FolioRoute::LandlordMaintenance.path()
                    />
                </div>
            </Show>

            <Show when=move || matches!(mode.get(), DashboardMode::Str | DashboardMode::All)>
                <div class="folio-stat-grid" style="margin-bottom:1rem;">
                    <StatCard
                        label="STR-eligible"
                        value=str_eligible_count
                        icon="vacation"
                        href=FolioRoute::LandlordAssets.path()
                    />
                    <StatCard
                        label="Listed"
                        value=str_listed_count
                        icon="campaign"
                        href=FolioRoute::LandlordReservations.path()
                    />
                    <StatCard
                        label="STR work orders"
                        value=open_work_orders
                        icon="build"
                        href=FolioRoute::LandlordMaintenance.path()
                    />
                    <StatCard
                        label="Compliance"
                        value=compliance_count
                        icon="gavel"
                        href=FolioRoute::LandlordStrCompliance.path()
                    />
                </div>
            </Show>

            <A href=map_maint_href attr:class="dash-map-peek press">
                <div>
                    <p class="dash-map-peek__title">"Ops map — maintenance layer"</p>
                    <p class="dash-map-peek__sub">"See open work orders on the map by property pin."</p>
                </div>
                <span class="material-symbols-outlined">"map"</span>
            </A>

            <div class="landlord-dash__sections" style="margin-top:1.25rem;">
                <section class="folio-section-card">
                    <div class="folio-section-card__header">
                        <h2 class="folio-section-card__title">"Needs attention"</h2>
                        <A
                            href=FolioRoute::LandlordMaintenance.path()
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
                                let m = mode.get();
                                let lease_items = leases.get().and_then(|r| r.ok()).unwrap_or_default();
                                let ticket_items = tickets.get().and_then(|r| r.ok()).unwrap_or_default();
                                let str_ids: std::collections::HashSet<_> = assets
                                    .get()
                                    .and_then(|r| r.ok())
                                    .map(|items| {
                                        items
                                            .into_iter()
                                            .filter(|a| a.str_eligible)
                                            .map(|a| a.id)
                                            .collect()
                                    })
                                    .unwrap_or_default();
                                let items = attention_items(m, &lease_items, &ticket_items, &str_ids);
                                if items.is_empty() {
                                    view! {
                                        <div class="folio-empty">
                                            <span class="material-symbols-outlined folio-empty__icon">"check_circle"</span>
                                            <p class="folio-empty__heading">"You're all caught up"</p>
                                            <p class="folio-empty__sub">
                                                "Nothing in this lens needs you right now."
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
                            <A href=FolioRoute::LandlordLeaseCreate.path() attr:class="folio-quick-action">
                                <span class="material-symbols-outlined folio-quick-action__icon">"description"</span>
                                "Add a lease"
                                <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                            </A>
                            <A href=FolioRoute::LandlordMaintenanceNew.path() attr:class="folio-quick-action">
                                <span class="material-symbols-outlined folio-quick-action__icon">"build"</span>
                                "Create work order"
                                <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                            </A>
                            <A href=FolioRoute::LandlordMap.path() attr:class="folio-quick-action">
                                <span class="material-symbols-outlined folio-quick-action__icon">"map"</span>
                                "Open ops map"
                                <span class="material-symbols-outlined folio-quick-action__chevron">"chevron_right"</span>
                            </A>
                            <A href=FolioRoute::LandlordAssets.path() attr:class="folio-quick-action">
                                <span class="material-symbols-outlined folio-quick-action__icon">"apartment"</span>
                                "Browse assets"
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
    mode: DashboardMode,
    leases: &[LeaseSummary],
    tickets: &[MaintenanceSummary],
    str_ids: &std::collections::HashSet<uuid::Uuid>,
) -> Vec<AttentionItem> {
    let mut items = Vec::new();
    let today = chrono::Utc::now().date_naive();

    if matches!(mode, DashboardMode::Ltr | DashboardMode::All) {
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
                        href: FolioRoute::LandlordLeases.path(),
                    });
                }
            }
        }
    }

    for ticket in tickets.iter().filter(|t| {
        matches!(
            CaseStatus::from_str(&t.status),
            CaseStatus::Open | CaseStatus::InProgress
        )
    }) {
        let on_str = ticket
            .asset_id
            .map(|id| str_ids.contains(&id))
            .unwrap_or(false);
        let include = match mode {
            DashboardMode::All => true,
            DashboardMode::Str => on_str,
            DashboardMode::Ltr => !on_str,
        };
        if !include {
            continue;
        }
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
            href: FolioRoute::LandlordMaintenance.path(),
        });
    }

    if matches!(mode, DashboardMode::Str | DashboardMode::All) {
        items.push(AttentionItem {
            icon: "event_available",
            label: "Upcoming check-ins".into(),
            meta: "Connect reservations to populate turnover.".into(),
            href: FolioRoute::LandlordReservations.path(),
        });
    }

    items.truncate(6);
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
