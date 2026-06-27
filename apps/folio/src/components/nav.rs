use leptos::prelude::*;
use leptos_router::hooks::use_location;
use crate::auth::FolioRole;

// ── NavIcon — every icon used in Folio, compile-checked ───────────────────────
//
// Using an enum instead of &'static str means:
//   - A typo ("appartment") is a compile error, not a silent missing icon at runtime
//   - Renaming an icon requires updating one match arm — all usages update automatically
//   - The icon set is exhaustively documented and discoverable
//
// Icon names map to Material Symbols Outlined identifiers.
// Reference: https://fonts.google.com/icons?icon.style=Outlined

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavIcon {
    // Navigation / layout
    Home,
    Settings,
    Logout,
    Help,
    Person,
    ManageAccounts,

    // Property / real estate
    Domain,          // portfolio (building)
    Apartment,       // assets (unit/building)
    Map,             // map view
    Sell,            // pricing

    // Document / contract
    Description,     // leases / documents
    Assignment,      // work orders
    ReceiptLong,     // billing / invoices / statements
    AccountBalance,  // ledger
    Folder,          // documents / vault
    Verified,        // compliance
    Gavel,           // STR compliance
    Payments,        // distributions / payouts

    // People / CRM
    PersonSearch,    // leads
    People,          // team / clients / household
    Group,           // household
    Badge,           // vendor profile / identity
    Handshake,       // deals (brokerage)

    // Operations
    Build,           // maintenance
    Handyman,        // vendors
    Campaign,        // campaigns
    Inventory2,      // catalog
    SyncAlt,         // syndication / channels
    CalendarMonth,   // schedule / calendar
    EventAvailable,  // reservations

    // Communication
    Inbox,
    Chat,            // guest messaging

    // Analytics
    BarChart,        // meridian / analytics

    // Finance
    CreditCard,      // tenant payments

    // STR / reviews
    Star,            // reviews
    Report,          // incidents / violations
}

impl NavIcon {
    /// Returns the Material Symbols Outlined CSS class name for this icon.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Home           => "home",
            Self::Settings       => "settings",
            Self::Logout         => "logout",
            Self::Help           => "help_outline",
            Self::Person         => "person",
            Self::ManageAccounts => "manage_accounts",

            Self::Domain         => "domain",
            Self::Apartment      => "apartment",
            Self::Map            => "map",
            Self::Sell           => "sell",

            Self::Description    => "description",
            Self::Assignment     => "assignment",
            Self::ReceiptLong    => "receipt_long",
            Self::AccountBalance => "account_balance",
            Self::Folder         => "folder",
            Self::Verified       => "verified",
            Self::Gavel          => "gavel",
            Self::Payments       => "payments",

            Self::PersonSearch   => "person_search",
            Self::People         => "people",
            Self::Group          => "group",
            Self::Badge          => "badge",
            Self::Handshake      => "handshake",

            Self::Build          => "build",
            Self::Handyman       => "handyman",
            Self::Campaign       => "campaign",
            Self::Inventory2     => "inventory_2",
            Self::SyncAlt        => "sync_alt",
            Self::CalendarMonth  => "calendar_month",
            Self::EventAvailable => "event_available",

            Self::Inbox          => "inbox",
            Self::Chat           => "chat",

            Self::BarChart       => "bar_chart",
            Self::CreditCard     => "credit_card",

            Self::Star           => "star",
            Self::Report         => "report",
        }
    }
}

impl std::fmt::Display for NavIcon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── FolioRoute — every navigable route as a typed value ──────────────────────
//
// Using an enum instead of &'static str means:
//   - If a route path changes, update `path()` in one place — compiler enforces all callers
//   - Routes are discoverable: you can ask "what routes exist for Landlord?"
//   - Nav items can't reference routes that don't exist in this enum
//   - Active-path matching logic is centralised on the enum, not scattered in views
//
// IMPORTANT: These path() values must stay in sync with the route definitions
// in `apps/folio/src/app.rs`. The Leptos router is the runtime authority;
// this enum is the compile-time authority. They must agree.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolioRoute {
    // ── Landlord /l/** ────────────────────────────────────────────────────────
    LandlordDashboard,
    LandlordPortfolio,
    LandlordAssets,
    LandlordAssetDetail,      // /l/assets/:id — not a nav item, but typed for use in pages
    LandlordLeases,
    LandlordLeaseDetail,      // /l/leases/:id
    LandlordLeads,
    LandlordCampaigns,
    LandlordBilling,
    LandlordLedger,
    LandlordStrCompliance,
    LandlordCatalog,
    LandlordVendors,
    LandlordReservations,
    LandlordMaintenance,
    LandlordSyndication,
    LandlordMeridian,
    LandlordMeridianConfig,
    LandlordAccountBilling,
    LandlordMap,
    LandlordVault,
    LandlordInspections,
    LandlordViolations,

    // ── Tenant /t/** ──────────────────────────────────────────────────────────
    TenantDashboard,
    TenantMyLease,
    TenantPayments,
    TenantPaymentHistory,
    TenantMaintenance,
    TenantMaintenanceNew,
    TenantMaintenanceDetail,  // /t/maintenance/:id
    TenantReservations,
    TenantInbox,
    TenantDocuments,
    TenantHousehold,
    TenantProfile,
    TenantViolations,
    TenantReports,

    // ── Vendor /v/** ──────────────────────────────────────────────────────────
    VendorDashboard,
    VendorWorkOrders,
    VendorInvoices,
    VendorSchedule,
    VendorNetworkProfile,

    // ── PMC /pmc/** ───────────────────────────────────────────────────────────
    PmcDashboard,
    PmcClientBook,
    PmcClientDetail,           // /pmc/clients/:id
    PmcMaintenance,
    PmcPortfolioMap,
    PmcOwnerStatements,

    // ── Owner /o/** ───────────────────────────────────────────────────────────
    OwnerDashboard,
    OwnerProperties,
    OwnerPropertyDetail,       // /o/properties/:id
    OwnerStatements,
    OwnerDistributions,
    OwnerMaintenance,

    // ── STR Host /s/** ────────────────────────────────────────────────────────
    StrHostDashboard,
    StrHostCalendar,
    StrHostReservations,
    StrHostListings,
    StrHostPricing,
    StrHostChannels,
    StrHostMessages,
    StrHostReviews,
    StrHostSyndication,
    StrHostIncidents,
    StrHostViolationFiling,

    // ── Agent /a/** ───────────────────────────────────────────────────────────
    AgentDashboard,
    AgentListings,
    AgentClients,
    AgentDeals,
    AgentSchedule,

    // ── Broker /br/** ─────────────────────────────────────────────────────────
    BrokerDashboard,
    BrokerAgents,
    BrokerListings,
    BrokerCompliance,
    BrokerRevenue,

    // ── Shared ────────────────────────────────────────────────────────────────
    Settings,
    Login,
    Verify,
}

impl FolioRoute {
    /// The URL path for this route. Must match `app.rs` route definitions exactly.
    pub const fn path(self) -> &'static str {
        match self {
            Self::LandlordDashboard      => "/l",
            Self::LandlordPortfolio      => "/l/portfolio",
            Self::LandlordAssets         => "/l/assets",
            Self::LandlordAssetDetail    => "/l/assets/:id",
            Self::LandlordLeases         => "/l/leases",
            Self::LandlordLeaseDetail    => "/l/leases/:id",
            Self::LandlordLeads          => "/l/leads",
            Self::LandlordCampaigns      => "/l/campaigns",
            Self::LandlordBilling        => "/l/billing",
            Self::LandlordLedger         => "/l/ledger",
            Self::LandlordStrCompliance  => "/l/str",
            Self::LandlordCatalog        => "/l/catalog",
            Self::LandlordVendors        => "/l/vendors",
            Self::LandlordReservations   => "/l/reservations",
            Self::LandlordMaintenance    => "/l/maintenance",
            Self::LandlordSyndication    => "/l/syndication",
            Self::LandlordMeridian       => "/l/meridian",
            Self::LandlordMeridianConfig => "/l/meridian/configure",
            Self::LandlordAccountBilling => "/l/account/billing",
            Self::LandlordMap            => "/l/map",
            Self::LandlordVault          => "/l/vault",
            Self::LandlordInspections    => "/l/inspections",
            Self::LandlordViolations     => "/l/violations",

            Self::TenantDashboard        => "/t",
            Self::TenantMyLease          => "/t/my-lease",
            Self::TenantPayments         => "/t/payments",
            Self::TenantPaymentHistory   => "/t/payments/history",
            Self::TenantMaintenance      => "/t/maintenance",
            Self::TenantMaintenanceNew   => "/t/maintenance/new",
            Self::TenantMaintenanceDetail=> "/t/maintenance/:id",
            Self::TenantReservations     => "/t/reservations",
            Self::TenantInbox            => "/t/inbox",
            Self::TenantDocuments        => "/t/docs",
            Self::TenantHousehold        => "/t/household",
            Self::TenantProfile          => "/t/profile",
            Self::TenantViolations       => "/t/violations",
            Self::TenantReports          => "/t/reports",

            Self::VendorDashboard        => "/v",
            Self::VendorWorkOrders       => "/v/work-orders",
            Self::VendorInvoices         => "/v/invoices",
            Self::VendorSchedule         => "/v/schedule",
            Self::VendorNetworkProfile   => "/v/profile",

            Self::PmcDashboard           => "/pmc",
            Self::PmcClientBook          => "/pmc/clients",
            Self::PmcClientDetail        => "/pmc/clients/:id",
            Self::PmcMaintenance         => "/pmc/maintenance",
            Self::PmcPortfolioMap        => "/pmc/map",
            Self::PmcOwnerStatements     => "/pmc/statements",

            Self::OwnerDashboard         => "/o",
            Self::OwnerProperties        => "/o/properties",
            Self::OwnerPropertyDetail    => "/o/properties/:id",
            Self::OwnerStatements        => "/o/statements",
            Self::OwnerDistributions     => "/o/distributions",
            Self::OwnerMaintenance       => "/o/maintenance",

            Self::StrHostDashboard       => "/s",
            Self::StrHostCalendar        => "/s/calendar",
            Self::StrHostReservations    => "/s/reservations",
            Self::StrHostListings        => "/s/listings/:id",
            Self::StrHostPricing         => "/s/pricing",
            Self::StrHostChannels        => "/s/channels",
            Self::StrHostMessages        => "/s/messages",
            Self::StrHostReviews         => "/s/reviews",
            Self::StrHostSyndication     => "/s/syndication",
            Self::StrHostIncidents       => "/s/incidents",
            Self::StrHostViolationFiling => "/s/violations/new",

            Self::AgentDashboard         => "/a",
            Self::AgentListings          => "/a/listings",
            Self::AgentClients           => "/a/clients",
            Self::AgentDeals             => "/a/deals",
            Self::AgentSchedule          => "/a/schedule",

            Self::BrokerDashboard        => "/br",
            Self::BrokerAgents           => "/br/agents",
            Self::BrokerListings         => "/br/listings",
            Self::BrokerCompliance       => "/br/compliance",
            Self::BrokerRevenue          => "/br/revenue",

            Self::Settings               => "/settings",
            Self::Login                  => "/login",
            Self::Verify                 => "/verify",
        }
    }

    /// Returns true if this route is a namespace root (matches exactly, not as prefix).
    /// Root routes: /l, /t, /v, /pmc, /o, /s, /a, /br
    pub const fn is_namespace_root(self) -> bool {
        matches!(self,
            Self::LandlordDashboard
            | Self::TenantDashboard
            | Self::VendorDashboard
            | Self::PmcDashboard
            | Self::OwnerDashboard
            | Self::StrHostDashboard
            | Self::AgentDashboard
            | Self::BrokerDashboard
        )
    }
}

impl std::fmt::Display for FolioRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.path())
    }
}

// ── NavItem — strongly typed ──────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavItem {
    pub route: FolioRoute,
    pub label: &'static str,
    pub icon:  NavIcon,
}

impl NavItem {
    pub const fn new(route: FolioRoute, label: &'static str, icon: NavIcon) -> Self {
        Self { route, label, icon }
    }
}

// ── NavGroup ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub struct NavGroup {
    pub label: Option<&'static str>,
    pub items: &'static [NavItem],
}

// ── NavConfig — per-role, derived via FolioRole::nav_config() ────────────────

#[derive(Clone, Copy, Debug)]
pub struct NavConfig {
    pub role_label:   &'static str,
    pub groups:       &'static [NavGroup],
    pub footer_items: &'static [NavItem],
}

// ── Canonical configs — one per role ─────────────────────────────────────────
//
// Source of truth for nav structure. Edit here only.
// Derived from: designs/stitch/project_pm/folio/l_assets/code.html
// and designs/stitch/project_pm/folio/ROUTES.md

static LANDLORD_NAV: NavConfig = NavConfig {
    role_label: "Landlord",
    groups: &[
        NavGroup {
            label: None,
            items: &[
                NavItem::new(FolioRoute::LandlordDashboard,   "Dashboard",   NavIcon::Home),
                NavItem::new(FolioRoute::LandlordPortfolio,   "Portfolio",   NavIcon::Domain),
                NavItem::new(FolioRoute::LandlordAssets,      "Assets",      NavIcon::Apartment),
                NavItem::new(FolioRoute::LandlordLeases,      "Leases",      NavIcon::Description),
                NavItem::new(FolioRoute::LandlordLeads,       "Leads",       NavIcon::PersonSearch),
            ],
        },
        NavGroup {
            label: Some("Operations"),
            items: &[
                NavItem::new(FolioRoute::LandlordMaintenance,  "Maintenance",  NavIcon::Build),
                NavItem::new(FolioRoute::LandlordCampaigns,    "Campaigns",    NavIcon::Campaign),
                NavItem::new(FolioRoute::LandlordVendors,      "Vendors",      NavIcon::Handyman),
                NavItem::new(FolioRoute::LandlordReservations, "Reservations", NavIcon::EventAvailable),
                NavItem::new(FolioRoute::LandlordCatalog,      "Catalog",      NavIcon::Inventory2),
            ],
        },
        NavGroup {
            label: Some("Finance"),
            items: &[
                NavItem::new(FolioRoute::LandlordBilling, "Billing", NavIcon::ReceiptLong),
                NavItem::new(FolioRoute::LandlordLedger,  "Ledger",  NavIcon::AccountBalance),
            ],
        },
        NavGroup {
            label: Some("Compliance"),
            items: &[
                NavItem::new(FolioRoute::LandlordStrCompliance, "STR Compliance", NavIcon::Gavel),
                NavItem::new(FolioRoute::LandlordSyndication,   "Syndication",    NavIcon::SyncAlt),
            ],
        },
    ],
    footer_items: &[
        NavItem::new(FolioRoute::LandlordMeridian,       "Analytics",  NavIcon::BarChart),
        NavItem::new(FolioRoute::LandlordAccountBilling, "Account",    NavIcon::ManageAccounts),
        NavItem::new(FolioRoute::Settings,               "Settings",   NavIcon::Settings),
    ],
};

static TENANT_NAV: NavConfig = NavConfig {
    role_label: "Tenant",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::TenantDashboard,    "Dashboard",    NavIcon::Home),
            NavItem::new(FolioRoute::TenantMyLease,      "My Lease",     NavIcon::Description),
            NavItem::new(FolioRoute::TenantPayments,     "Payments",     NavIcon::CreditCard),
            NavItem::new(FolioRoute::TenantMaintenance,  "Maintenance",  NavIcon::Build),
            NavItem::new(FolioRoute::TenantReservations, "Reservations", NavIcon::EventAvailable),
            NavItem::new(FolioRoute::TenantInbox,        "Inbox",        NavIcon::Inbox),
            NavItem::new(FolioRoute::TenantDocuments,    "Documents",    NavIcon::Folder),
            NavItem::new(FolioRoute::TenantHousehold,    "Household",    NavIcon::Group),
        ],
    }],
    footer_items: &[
        NavItem::new(FolioRoute::TenantProfile, "Profile", NavIcon::Person),
        NavItem::new(FolioRoute::TenantReports, "Reports", NavIcon::BarChart),
    ],
};

static VENDOR_NAV: NavConfig = NavConfig {
    role_label: "Vendor",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::VendorDashboard,   "Dashboard",   NavIcon::Home),
            NavItem::new(FolioRoute::VendorWorkOrders,  "Work Orders", NavIcon::Assignment),
            NavItem::new(FolioRoute::VendorInvoices,    "Invoices",    NavIcon::ReceiptLong),
            NavItem::new(FolioRoute::VendorSchedule,    "Schedule",    NavIcon::CalendarMonth),
        ],
    }],
    footer_items: &[
        NavItem::new(FolioRoute::VendorNetworkProfile, "Network Profile", NavIcon::Badge),
    ],
};

static PMC_NAV: NavConfig = NavConfig {
    role_label: "PMC",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::PmcDashboard,      "Dashboard",   NavIcon::Home),
            NavItem::new(FolioRoute::PmcClientBook,     "Client Book", NavIcon::People),
            NavItem::new(FolioRoute::PmcMaintenance,    "Maintenance", NavIcon::Build),
            NavItem::new(FolioRoute::PmcPortfolioMap,   "Map",         NavIcon::Map),
            NavItem::new(FolioRoute::PmcOwnerStatements,"Statements",  NavIcon::ReceiptLong),
        ],
    }],
    footer_items: &[
        NavItem::new(FolioRoute::Settings, "Settings", NavIcon::Settings),
    ],
};

static OWNER_NAV: NavConfig = NavConfig {
    role_label: "Owner",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::OwnerDashboard,    "Dashboard",    NavIcon::Home),
            NavItem::new(FolioRoute::OwnerProperties,   "Properties",   NavIcon::Apartment),
            NavItem::new(FolioRoute::OwnerStatements,   "Statements",   NavIcon::ReceiptLong),
            NavItem::new(FolioRoute::OwnerDistributions,"Distributions",NavIcon::Payments),
            NavItem::new(FolioRoute::OwnerMaintenance,  "Maintenance",  NavIcon::Build),
        ],
    }],
    footer_items: &[],
};

static AGENT_NAV: NavConfig = NavConfig {
    role_label: "Agent",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::AgentDashboard, "Dashboard", NavIcon::Home),
            NavItem::new(FolioRoute::AgentListings,  "Listings",  NavIcon::Apartment),
            NavItem::new(FolioRoute::AgentClients,   "Clients",   NavIcon::People),
            NavItem::new(FolioRoute::AgentDeals,     "Deals",     NavIcon::Handshake),
            NavItem::new(FolioRoute::AgentSchedule,  "Schedule",  NavIcon::CalendarMonth),
        ],
    }],
    footer_items: &[
        NavItem::new(FolioRoute::Settings, "Settings", NavIcon::Settings),
    ],
};

static BROKER_NAV: NavConfig = NavConfig {
    role_label: "Broker",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::BrokerDashboard,   "Dashboard",  NavIcon::Home),
            NavItem::new(FolioRoute::BrokerAgents,      "Agents",     NavIcon::People),
            NavItem::new(FolioRoute::BrokerListings,    "Listings",   NavIcon::Apartment),
            NavItem::new(FolioRoute::BrokerCompliance,  "Compliance", NavIcon::Verified),
            NavItem::new(FolioRoute::BrokerRevenue,     "Revenue",    NavIcon::Payments),
        ],
    }],
    footer_items: &[
        NavItem::new(FolioRoute::Settings, "Settings", NavIcon::Settings),
    ],
};

static STR_HOST_NAV: NavConfig = NavConfig {
    role_label: "STR Host",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::StrHostDashboard,    "Dashboard",    NavIcon::Home),
            NavItem::new(FolioRoute::StrHostCalendar,     "Calendar",     NavIcon::CalendarMonth),
            NavItem::new(FolioRoute::StrHostReservations, "Reservations", NavIcon::EventAvailable),
            NavItem::new(FolioRoute::StrHostListings,     "Listings",     NavIcon::Apartment),
            NavItem::new(FolioRoute::StrHostPricing,      "Pricing",      NavIcon::Sell),
            NavItem::new(FolioRoute::StrHostChannels,     "Channels",     NavIcon::SyncAlt),
            NavItem::new(FolioRoute::StrHostMessages,     "Messages",     NavIcon::Chat),
        ],
    }],
    footer_items: &[
        NavItem::new(FolioRoute::StrHostReviews,   "Reviews",   NavIcon::Star),
        NavItem::new(FolioRoute::StrHostIncidents, "Incidents", NavIcon::Report),
        NavItem::new(FolioRoute::Settings,         "Settings",  NavIcon::Settings),
    ],
};

// ── Role → Config binding ─────────────────────────────────────────────────────
//
// This is the key architectural point: FolioRole IS the index into nav configs.
// Layouts don't import a specific *_NAV static — they pass `role.nav_config()`.
// Adding a new role requires adding it here — the compiler will enforce completeness.

impl FolioRole {
    pub fn nav_config(self) -> &'static NavConfig {
        match self {
            Self::Landlord        => &LANDLORD_NAV,
            Self::Tenant          => &TENANT_NAV,
            Self::Vendor          => &VENDOR_NAV,
            Self::PropertyManager => &PMC_NAV,
            Self::Owner           => &OWNER_NAV,
            Self::Agent           => &AGENT_NAV,
            Self::Broker          => &BROKER_NAV,
        }
    }
}

// ── SidebarNav Component ──────────────────────────────────────────────────────

#[component]
pub fn SidebarNav(
    config: &'static NavConfig,
    #[prop(optional)] user_name: Option<String>,
    #[prop(optional)] initials:  Option<String>,
) -> impl IntoView {
    let location = use_location();

    let initials = initials.or_else(|| {
        user_name.as_deref().map(derive_initials)
    }).unwrap_or_else(|| "?".to_string());

    view! {
        <nav class="folio-sidebar">
            <div class="sidebar-brand">
                <span class="sidebar-logo">"Folio"</span>
                <span class="sidebar-role">{config.role_label}</span>
            </div>

            <div class="sidebar-scroll">
                {config.groups.iter().map(|group| {
                    let path = location.pathname.get();
                    view! {
                        <div class="nav-group">
                            {group.label.map(|l| view! {
                                <span class="nav-group-label">{l}</span>
                            })}
                            <ul class="nav-list">
                                {group.items.iter().map(|item| {
                                    let active = is_active(&path, item.route);
                                    view! {
                                        <li>
                                            <a
                                                href=item.route.path()
                                                class=if active { "nav-link nav-link--active" } else { "nav-link" }
                                                aria-current=if active { Some("page") } else { None }
                                            >
                                                <span class="material-symbols-outlined nav-link-icon">
                                                    {item.icon.as_str()}
                                                </span>
                                                <span class="nav-link-label">{item.label}</span>
                                            </a>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        </div>
                    }
                }).collect_view()}
            </div>

            <div class="sidebar-footer">
                {(!config.footer_items.is_empty()).then(|| {
                    let path = location.pathname.get();
                    view! {
                        <ul class="nav-list nav-list--footer">
                            {config.footer_items.iter().map(|item| {
                                let active = is_active(&path, item.route);
                                view! {
                                    <li>
                                        <a
                                            href=item.route.path()
                                            class=if active { "nav-link nav-link--active" } else { "nav-link" }
                                        >
                                            <span class="material-symbols-outlined nav-link-icon">
                                                {item.icon.as_str()}
                                            </span>
                                            <span class="nav-link-label">{item.label}</span>
                                        </a>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    }
                })}

                <div class="sidebar-user">
                    <div class="sidebar-avatar">{initials.clone()}</div>
                    <div class="sidebar-user-info">
                        <span class="sidebar-user-name">
                            {user_name.unwrap_or_else(|| "—".to_string())}
                        </span>
                        <span class="sidebar-user-role">{config.role_label}</span>
                    </div>
                    <button
                        class="sidebar-logout-btn"
                        title="Sign out"
                        on:click=move |_| {
                            leptos::task::spawn_local(async {
                                let _ = crate::auth::revoke_session().await;
                                let _ = web_sys::window().map(|w| {
                                    let _ = w.location().set_href(FolioRoute::Login.path());
                                });
                            });
                        }
                    >
                        <span class="material-symbols-outlined" style="font-size:18px">
                            {NavIcon::Logout.as_str()}
                        </span>
                    </button>
                </div>
            </div>
        </nav>
    }
}

// ── Active path logic ─────────────────────────────────────────────────────────

fn is_active(current_path: &str, route: FolioRoute) -> bool {
    let route_path = route.path();
    if route.is_namespace_root() {
        // Exact match only — /l should not match /l/assets
        current_path == route_path
    } else {
        // Prefix match — /l/assets matches /l/assets AND /l/assets/some-id
        current_path == route_path
            || current_path.starts_with(&format!("{route_path}/"))
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn derive_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}
