//! Landlord Setup index — `/l/setup`
//!
//! Salesforce-style config surface: grouped deep links, not an ops dashboard.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::nav::{FolioRoute, NavIcon};
use crate::components::page_header::PageHeader;

struct SetupLink {
    route: FolioRoute,
    title: &'static str,
    purpose: &'static str,
    icon: NavIcon,
}

struct SetupSection {
    heading: &'static str,
    links: &'static [SetupLink],
}

const SECTIONS: &[SetupSection] = &[
    SetupSection {
        heading: "People & access",
        links: &[
            SetupLink {
                route: FolioRoute::LandlordTeam,
                title: "Network",
                purpose: "Team access and invite codes",
                icon: NavIcon::Group,
            },
            SetupLink {
                route: FolioRoute::LandlordReferrals,
                title: "Referrals",
                purpose: "Friends & family share links",
                icon: NavIcon::Campaign,
            },
            SetupLink {
                route: FolioRoute::LandlordNotifications,
                title: "Notifications",
                purpose: "Inbox and channel preferences",
                icon: NavIcon::Inbox,
            },
        ],
    },
    SetupSection {
        heading: "Compliance & channels",
        links: &[
            SetupLink {
                route: FolioRoute::LandlordStrCompliance,
                title: "STR Compliance",
                purpose: "Operating permits and expiry scans",
                icon: NavIcon::Gavel,
            },
            SetupLink {
                route: FolioRoute::LandlordSyndication,
                title: "Syndication",
                purpose: "Listing channel preferences",
                icon: NavIcon::SyncAlt,
            },
            SetupLink {
                route: FolioRoute::LandlordViolations,
                title: "Violations",
                purpose: "File and track compliance cases",
                icon: NavIcon::Report,
            },
        ],
    },
    SetupSection {
        heading: "Catalog & marketing",
        links: &[
            SetupLink {
                route: FolioRoute::LandlordCatalog,
                title: "Catalog",
                purpose: "Pricebook and room types",
                icon: NavIcon::Inventory2,
            },
            SetupLink {
                route: FolioRoute::LandlordCampaigns,
                title: "Campaigns",
                purpose: "Outreach campaigns",
                icon: NavIcon::Campaign,
            },
            SetupLink {
                route: FolioRoute::LandlordLeads,
                title: "Leads",
                purpose: "Prospect pipeline",
                icon: NavIcon::PersonSearch,
            },
            SetupLink {
                route: FolioRoute::LandlordReservations,
                title: "Reservations",
                purpose: "STR stay operations",
                icon: NavIcon::EventAvailable,
            },
        ],
    },
    SetupSection {
        heading: "Money detail",
        links: &[SetupLink {
            route: FolioRoute::LandlordLedger,
            title: "Ledger",
            purpose: "Full billable event audit trail",
            icon: NavIcon::AccountBalance,
        }],
    },
    SetupSection {
        heading: "Quality & ops registry",
        links: &[
            SetupLink {
                route: FolioRoute::LandlordRatings,
                title: "Ratings",
                purpose: "Contractor scorecards",
                icon: NavIcon::Star,
            },
            SetupLink {
                route: FolioRoute::LandlordInspections,
                title: "Inspections",
                purpose: "Proactive inspection schedule",
                icon: NavIcon::Verified,
            },
            SetupLink {
                route: FolioRoute::LandlordSystems,
                title: "Building systems",
                purpose: "Elevators, HVAC, envelope registry",
                icon: NavIcon::Build,
            },
            SetupLink {
                route: FolioRoute::LandlordAppliances,
                title: "Appliances",
                purpose: "Unit appliance lifecycle registry",
                icon: NavIcon::Inventory2,
            },
            SetupLink {
                route: FolioRoute::LandlordVault,
                title: "Digital vault",
                purpose: "Leases, permits, and certificates",
                icon: NavIcon::Folder,
            },
        ],
    },
    SetupSection {
        heading: "Marketplace",
        links: &[
            SetupLink {
                route: FolioRoute::LandlordVendors,
                title: "Vendors",
                purpose: "Your contractor network",
                icon: NavIcon::Handyman,
            },
            SetupLink {
                route: FolioRoute::LandlordMarketplace,
                title: "Marketplace",
                purpose: "Find and add contractors by trade",
                icon: NavIcon::Handshake,
            },
            SetupLink {
                route: FolioRoute::LandlordBuyers,
                title: "Buyers",
                purpose: "Buyer CRM for dispositions",
                icon: NavIcon::People,
            },
        ],
    },
];

#[component]
pub fn LandlordSetup() -> impl IntoView {
    let title = Signal::derive(|| "Setup".to_string());
    let subtitle = Signal::derive(|| {
        "Configuration and less-frequent tools — day-to-day work stays in the sidebar.".to_string()
    });

    view! {
        <div class="folio-setup">
            <PageHeader title=title subtitle=subtitle />
            <div class="folio-setup__sections">
                {SECTIONS.iter().map(|section| {
                    view! {
                        <section class="folio-setup__section">
                            <h2 class="folio-setup__heading">{section.heading}</h2>
                            <ul class="folio-setup__list">
                                {section.links.iter().map(|link| {
                                    let href = link.route.path();
                                    view! {
                                        <li>
                                            <A href=href attr:class="folio-setup__row press">
                                                <span class="material-symbols-outlined folio-setup__icon">
                                                    {link.icon.as_str()}
                                                </span>
                                                <span class="folio-setup__text">
                                                    <span class="folio-setup__title">{link.title}</span>
                                                    <span class="folio-setup__purpose">{link.purpose}</span>
                                                </span>
                                                <span class="material-symbols-outlined folio-setup__chevron">
                                                    {NavIcon::ChevronRight.as_str()}
                                                </span>
                                            </A>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        </section>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
