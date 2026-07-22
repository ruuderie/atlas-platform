use crate::auth::FolioRole;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

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
    Domain,    // portfolio (building)
    Apartment, // assets (unit/building)
    Map,       // map view
    Sell,      // pricing

    // Document / contract
    Description,    // leases / documents
    Assignment,     // work orders
    ReceiptLong,    // billing / invoices / statements
    AccountBalance, // ledger
    Folder,         // documents / vault
    Verified,       // compliance
    Gavel,          // STR compliance
    Payments,       // distributions / payouts

    // People / CRM
    PersonSearch, // leads
    People,       // team / clients / household
    Group,        // household
    Badge,        // vendor profile / identity
    Handshake,    // deals (brokerage)

    // Operations
    Build,          // maintenance
    Handyman,       // vendors
    Campaign,       // campaigns
    Inventory2,     // catalog
    SyncAlt,        // syndication / channels
    CalendarMonth,  // schedule / calendar
    EventAvailable, // reservations

    // Communication
    Inbox,
    Chat, // guest messaging

    // Analytics
    BarChart,  // meridian / analytics
    ShowChart, // property value tracker

    // Finance
    CreditCard, // tenant payments

    // STR / reviews
    Star,   // reviews
    Report, // incidents / violations

    // Navigation helpers (not sidebar items — used in page chrome)
    ChevronRight, // breadcrumb separator
    ArrowBack,    // back-navigation link
    Login,        // sign-in / check-in icon
    Tune,         // Setup (Salesforce-style config index)
    Search,       // global finder
}

impl NavIcon {
    /// Returns the Material Symbols Outlined CSS class name for this icon.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Home => "home",
            Self::Settings => "settings",
            Self::Logout => "logout",
            Self::Help => "help_outline",
            Self::Person => "person",
            Self::ManageAccounts => "manage_accounts",

            Self::Domain => "domain",
            Self::Apartment => "apartment",
            Self::Map => "map",
            Self::Sell => "sell",

            Self::Description => "description",
            Self::Assignment => "assignment",
            Self::ReceiptLong => "receipt_long",
            Self::AccountBalance => "account_balance",
            Self::Folder => "folder",
            Self::Verified => "verified",
            Self::Gavel => "gavel",
            Self::Payments => "payments",

            Self::PersonSearch => "person_search",
            Self::People => "people",
            Self::Group => "group",
            Self::Badge => "badge",
            Self::Handshake => "handshake",

            Self::Build => "build",
            Self::Handyman => "handyman",
            Self::Campaign => "campaign",
            Self::Inventory2 => "inventory_2",
            Self::SyncAlt => "sync_alt",
            Self::CalendarMonth => "calendar_month",
            Self::EventAvailable => "event_available",

            Self::Inbox => "inbox",
            Self::Chat => "chat",

            Self::BarChart => "bar_chart",
            Self::ShowChart => "show_chart",
            Self::CreditCard => "credit_card",

            Self::Star => "star",
            Self::Report => "report",

            Self::ChevronRight => "chevron_right",
            Self::ArrowBack => "arrow_back",
            Self::Login => "login",
            Self::Tune => "tune",
            Self::Search => "search",
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
    LandlordAssetDetail, // /l/assets/:id — hub | unit | leaf dispatch
    LandlordUnitHistory, // /l/assets/:id/history — unit History tab deep link
    LandlordHistoricalLease, // /l/assets/:id/history/lease
    LandlordUnitPaymentHistory, // /l/assets/:id/history/payments
    LandlordUnitMaintenanceHistory, // /l/assets/:id/history/maintenance
    LandlordAssetArchive, // /l/assets/:id/archive — danger-zone deep link
    LandlordAssetSystems, // /l/assets/:id/systems
    LandlordAssetDocuments, // /l/assets/:id/documents
    LandlordAssetPortal, // /l/assets/:id/portal — CMS stub
    LandlordLeases,
    LandlordLeaseCreate, // /l/leases/new
    LandlordLeaseDetail, // /l/leases/:id
    LandlordOccupantProfile, // /l/leases/:lease_id/occupants/:entry_id
    LandlordApplications, // /l/applications — rental applications inbox
    LandlordAssetsCreate, // /l/assets/new
    LandlordSetup,       // /l/setup — Salesforce-style config index
    LandlordLeads,
    LandlordCampaigns,
    LandlordBilling,
    LandlordLedger,
    LandlordStrCompliance,
    LandlordCatalog,
    LandlordVendors,
    LandlordReservations,
    LandlordMaintenance,
    LandlordMaintenanceNew, // /l/maintenance/new
    LandlordMaintenanceDetail, // /l/maintenance/:id
    LandlordProjectDetail, // /l/projects/:id
    LandlordSyndication,
    LandlordMeridian, // /l/meridian — KPI overview
    LandlordMeridianConfig,
    LandlordRatings,
    LandlordAccountBilling,
    LandlordMap,
    LandlordVault,
    LandlordSystems, // /l/systems — portfolio building systems
    LandlordAppliances, // /l/appliances
    LandlordInspections,
    LandlordViolations,
    LandlordDeals,           // /l/deals
    LandlordDealDetail,      // /l/deals/:id
    LandlordDealStructure,   // /l/deals/:id/structure
    LandlordBuyers,          // /l/buyers
    LandlordMarketplace,     // /l/marketplace
    LandlordTenantProfile, // /l/tenants/:id — landlord view of a counterparty tenant
    LandlordCommunications, // /l/communications — multi-party messaging
    LandlordNotifications, // /l/notifications  — notification inbox + channel prefs
    LandlordTeam,          // /l/team — team access + G-36 network invites
    LandlordReferrals,     // /l/referrals — F&F share link + SMS/email
    VendorReferrals,       // /v/referrals
    PmcReferrals,          // /pmc/referrals
    StrHostReferrals,      // /s/referrals
    AgentReferrals,        // /a/referrals
    BrokerReferrals,       // /b/referrals

    // ── Tenant /t/** ──────────────────────────────────────────────────────────
    TenantDashboard,
    TenantMyLease,
    TenantPayments,
    TenantPaymentHistory,
    TenantMaintenance,
    TenantMaintenanceNew,
    TenantMaintenanceDetail, // /t/maintenance/:id
    TenantReservations,
    TenantRatings,
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
    PmcClientDetail, // /pmc/clients/:id
    PmcMaintenance,
    PmcPortfolioMap,
    PmcOwnerStatements,

    // ── Owner /o/** ───────────────────────────────────────────────────────────
    OwnerDashboard,
    OwnerProperties,
    OwnerPropertyDetail, // /o/properties/:id
    OwnerStatements,
    OwnerDistributions,
    OwnerMaintenance,

    // ── STR Host /s/** ────────────────────────────────────────────────────────
    StrHostDashboard,
    StrHostCalendar,
    StrHostReservations,
    StrHostListingIndex, // /s/listings  (list view — NAV target)
    StrHostListings,     // /s/listings/:id  (detail — linked from cards)
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

    // ── STR Guest /g/** ───────────────────────────────────────────────────────
    GuestDashboard,
    GuestReservation,
    GuestCheckIn,
    GuestHouseRules,
    GuestInbox,
    GuestProfile,

    // ── Property Owner Lite /po/** ──────────────────────────────────
    PropertyOwnerLiteDashboard,
    PropertyOwnerLiteValue,
    PropertyOwnerLiteFindVendor,
}

impl FolioRoute {
    /// The URL path for this route. Must match `app.rs` route definitions exactly.
    pub const fn path(self) -> &'static str {
        match self {
            Self::LandlordDashboard => "/l",
            Self::LandlordPortfolio => "/l/portfolio",
            Self::LandlordAssets => "/l/assets",
            Self::LandlordAssetDetail => "/l/assets/:id",
            Self::LandlordUnitHistory => "/l/assets/:id/history",
            Self::LandlordHistoricalLease => "/l/assets/:id/history/lease",
            Self::LandlordUnitPaymentHistory => "/l/assets/:id/history/payments",
            Self::LandlordUnitMaintenanceHistory => "/l/assets/:id/history/maintenance",
            Self::LandlordAssetArchive => "/l/assets/:id/archive",
            Self::LandlordAssetSystems => "/l/assets/:id/systems",
            Self::LandlordAssetDocuments => "/l/assets/:id/documents",
            Self::LandlordAssetPortal => "/l/assets/:id/portal",
            Self::LandlordLeases => "/l/leases",
            Self::LandlordLeaseCreate => "/l/leases/new",
            Self::LandlordLeaseDetail => "/l/leases/:id",
            Self::LandlordOccupantProfile => "/l/leases/:lease_id/occupants/:entry_id",
            Self::LandlordApplications => "/l/applications",
            Self::LandlordAssetsCreate => "/l/assets/new",
            Self::LandlordSetup => "/l/setup",
            Self::LandlordLeads => "/l/leads",
            Self::LandlordCampaigns => "/l/campaigns",
            Self::LandlordBilling => "/l/billing",
            Self::LandlordLedger => "/l/ledger",
            Self::LandlordStrCompliance => "/l/str",
            Self::LandlordCatalog => "/l/catalog",
            Self::LandlordVendors => "/l/vendors",
            Self::LandlordReservations => "/l/reservations",
            Self::LandlordMaintenance => "/l/maintenance",
            Self::LandlordMaintenanceNew => "/l/maintenance/new",
            Self::LandlordMaintenanceDetail => "/l/maintenance/:id",
            Self::LandlordProjectDetail => "/l/projects/:id",
            Self::LandlordSyndication => "/l/syndication",
            Self::LandlordMeridian => "/l/meridian",
            Self::LandlordMeridianConfig => "/l/meridian/configure",
            Self::LandlordRatings => "/l/ratings",
            Self::LandlordAccountBilling => "/l/account/billing",
            Self::LandlordMap => "/l/map",
            Self::LandlordVault => "/l/vault",
            Self::LandlordSystems => "/l/systems",
            Self::LandlordAppliances => "/l/appliances",
            Self::LandlordInspections => "/l/inspections",
            Self::LandlordViolations => "/l/violations",
            Self::LandlordDeals => "/l/deals",
            Self::LandlordDealDetail => "/l/deals/:id",
            Self::LandlordDealStructure => "/l/deals/:id/structure",
            Self::LandlordBuyers => "/l/buyers",
            Self::LandlordMarketplace => "/l/marketplace",
            Self::LandlordTenantProfile => "/l/tenants/:id",
            Self::LandlordCommunications => "/l/communications",
            Self::LandlordNotifications => "/l/notifications",
            Self::LandlordTeam => "/l/team",
            Self::LandlordReferrals => "/l/referrals",
            Self::VendorReferrals => "/v/referrals",
            Self::PmcReferrals => "/pmc/referrals",
            Self::StrHostReferrals => "/s/referrals",
            Self::AgentReferrals => "/a/referrals",
            Self::BrokerReferrals => "/b/referrals",

            Self::TenantDashboard => "/t",
            Self::TenantMyLease => "/t/my-lease",
            Self::TenantPayments => "/t/payments",
            Self::TenantPaymentHistory => "/t/payments/history",
            Self::TenantMaintenance => "/t/maintenance",
            Self::TenantMaintenanceNew => "/t/maintenance/new",
            Self::TenantMaintenanceDetail => "/t/maintenance/:id",
            Self::TenantReservations => "/t/reservations",
            Self::TenantRatings => "/t/ratings",
            Self::TenantInbox => "/t/inbox",
            Self::TenantDocuments => "/t/docs",
            Self::TenantHousehold => "/t/household",
            Self::TenantProfile => "/t/profile",
            Self::TenantViolations => "/t/violations",
            Self::TenantReports => "/t/reports",

            Self::VendorDashboard => "/v",
            Self::VendorWorkOrders => "/v/work-orders",
            Self::VendorInvoices => "/v/invoices",
            Self::VendorSchedule => "/v/schedule",
            Self::VendorNetworkProfile => "/v/profile",

            Self::PmcDashboard => "/pmc",
            Self::PmcClientBook => "/pmc/clients",
            Self::PmcClientDetail => "/pmc/clients/:id",
            Self::PmcMaintenance => "/pmc/maintenance",
            Self::PmcPortfolioMap => "/pmc/map",
            Self::PmcOwnerStatements => "/pmc/statements",

            Self::OwnerDashboard => "/o",
            Self::OwnerProperties => "/o/properties",
            Self::OwnerPropertyDetail => "/o/properties/:id",
            Self::OwnerStatements => "/o/statements",
            Self::OwnerDistributions => "/o/distributions",
            Self::OwnerMaintenance => "/o/maintenance",

            Self::StrHostDashboard => "/s",
            Self::StrHostCalendar => "/s/calendar",
            Self::StrHostReservations => "/s/reservations",
            Self::StrHostListingIndex => "/s/listings",
            Self::StrHostListings => "/s/listings/:id",
            Self::StrHostPricing => "/s/pricing",
            Self::StrHostChannels => "/s/channels",
            Self::StrHostMessages => "/s/messages",
            Self::StrHostReviews => "/s/reviews",
            Self::StrHostSyndication => "/s/syndication",
            Self::StrHostIncidents => "/s/incidents",
            Self::StrHostViolationFiling => "/s/violations/new",

            Self::AgentDashboard => "/a",
            Self::AgentListings => "/a/listings",
            Self::AgentClients => "/a/clients",
            Self::AgentDeals => "/a/deals",
            Self::AgentSchedule => "/a/schedule",

            Self::BrokerDashboard => "/br",
            Self::BrokerAgents => "/br/agents",
            Self::BrokerListings => "/br/listings",
            Self::BrokerCompliance => "/br/compliance",
            Self::BrokerRevenue => "/br/revenue",

            Self::Settings => "/settings",
            Self::Login => "/login",
            Self::Verify => "/verify",

            Self::GuestDashboard => "/g",
            Self::GuestReservation => "/g/reservation",
            Self::GuestCheckIn => "/g/check-in",
            Self::GuestHouseRules => "/g/house-rules",
            Self::GuestInbox => "/g/inbox",
            Self::GuestProfile => "/g/profile",

            Self::PropertyOwnerLiteDashboard => "/po",
            Self::PropertyOwnerLiteValue => "/po/value",
            Self::PropertyOwnerLiteFindVendor => "/po/find-vendor",
        }
    }

    /// Returns true if this route is a namespace root (matches exactly, not as prefix).
    /// Root routes: /l, /t, /v, /pmc, /o, /s, /a, /br
    pub const fn is_namespace_root(self) -> bool {
        matches!(
            self,
            Self::LandlordDashboard
                | Self::TenantDashboard
                | Self::VendorDashboard
                | Self::PmcDashboard
                | Self::OwnerDashboard
                | Self::StrHostDashboard
                | Self::AgentDashboard
                | Self::BrokerDashboard
                | Self::PropertyOwnerLiteDashboard
        )
    }
}

impl std::fmt::Display for FolioRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.path())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests — FolioRoute routing invariants
//
// Run with: cargo test -p folio --lib components::nav::tests
// No WASM, no async, no browser required.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── path() starts with / ──────────────────────────────────────────────────
    // Regression guard: every route must produce an absolute path so that
    // <a href=route.path()> works correctly in SSR.

    #[test]
    fn all_sampled_paths_are_absolute() {
        let routes = [
            FolioRoute::LandlordDashboard,
            FolioRoute::LandlordAssets,
            FolioRoute::LandlordMeridian,
            FolioRoute::LandlordMeridianConfig,
            FolioRoute::LandlordAccountBilling,
            FolioRoute::TenantDashboard,
            FolioRoute::TenantMyLease,
            FolioRoute::TenantInbox,
            FolioRoute::TenantHousehold,
            FolioRoute::VendorDashboard,
            FolioRoute::PmcDashboard,
            FolioRoute::PmcPortfolioMap,
            FolioRoute::OwnerDashboard,
            FolioRoute::StrHostDashboard,
            FolioRoute::StrHostListingIndex,
            FolioRoute::StrHostSyndication,
            FolioRoute::Settings,
            FolioRoute::Login,
        ];
        for r in routes {
            let p = r.path();
            assert!(
                p.starts_with('/'),
                "FolioRoute::{r:?}.path() = {p:?} — must start with '/'"
            );
        }
    }

    // ── Namespace dashboard paths ─────────────────────────────────────────────

    #[test]
    fn namespace_dashboard_paths() {
        assert_eq!(FolioRoute::LandlordDashboard.path(), "/l");
        assert_eq!(FolioRoute::TenantDashboard.path(), "/t");
        assert_eq!(FolioRoute::VendorDashboard.path(), "/v");
        assert_eq!(FolioRoute::PmcDashboard.path(), "/pmc");
        assert_eq!(FolioRoute::OwnerDashboard.path(), "/o");
        assert_eq!(FolioRoute::StrHostDashboard.path(), "/s");
        assert_eq!(FolioRoute::AgentDashboard.path(), "/a");
        assert_eq!(FolioRoute::BrokerDashboard.path(), "/br");
    }

    // ── Key route paths ───────────────────────────────────────────────────────

    #[test]
    fn meridian_paths() {
        assert_eq!(FolioRoute::LandlordMeridian.path(), "/l/meridian");
        assert_eq!(
            FolioRoute::LandlordMeridianConfig.path(),
            "/l/meridian/configure"
        );
    }

    #[test]
    fn landlord_setup_and_create_paths() {
        assert_eq!(FolioRoute::LandlordSetup.path(), "/l/setup");
        assert_eq!(FolioRoute::LandlordLeaseCreate.path(), "/l/leases/new");
        assert_eq!(FolioRoute::LandlordAssetsCreate.path(), "/l/assets/new");
    }

    #[test]
    fn landlord_unit_history_and_archive_paths() {
        assert_eq!(
            FolioRoute::LandlordUnitHistory.path(),
            "/l/assets/:id/history"
        );
        assert_eq!(
            FolioRoute::LandlordHistoricalLease.path(),
            "/l/assets/:id/history/lease"
        );
        assert_eq!(
            FolioRoute::LandlordUnitPaymentHistory.path(),
            "/l/assets/:id/history/payments"
        );
        assert_eq!(
            FolioRoute::LandlordUnitMaintenanceHistory.path(),
            "/l/assets/:id/history/maintenance"
        );
        assert_eq!(
            FolioRoute::LandlordAssetArchive.path(),
            "/l/assets/:id/archive"
        );
        let primary: Vec<_> = LANDLORD_NAV
            .groups
            .iter()
            .flat_map(|g| g.items.iter().map(|i| i.route))
            .collect();
        assert!(!primary.contains(&FolioRoute::LandlordUnitHistory));
        assert!(!primary.contains(&FolioRoute::LandlordHistoricalLease));
        assert!(!primary.contains(&FolioRoute::LandlordUnitPaymentHistory));
        assert!(!primary.contains(&FolioRoute::LandlordUnitMaintenanceHistory));
        assert!(!primary.contains(&FolioRoute::LandlordAssetArchive));
    }

    #[test]
    fn landlord_nav_is_lean_job_rail() {
        let primary: Vec<_> = LANDLORD_NAV
            .groups
            .iter()
            .flat_map(|g| g.items.iter().map(|i| i.route))
            .collect();
        assert_eq!(primary.len(), 8, "primary rail must stay ~8 job destinations");
        assert!(primary.contains(&FolioRoute::LandlordAssets));
        assert!(primary.contains(&FolioRoute::LandlordDeals));
        assert!(!primary.contains(&FolioRoute::LandlordLeads));
        assert!(!primary.contains(&FolioRoute::LandlordVault));
        let footer: Vec<_> = LANDLORD_NAV.footer_items.iter().map(|i| i.route).collect();
        assert!(footer.contains(&FolioRoute::LandlordSetup));
        assert!(footer.contains(&FolioRoute::LandlordMeridian));
    }

    #[test]
    fn settings_path() {
        assert_eq!(FolioRoute::Settings.path(), "/settings");
    }

    #[test]
    fn pmc_map_path() {
        assert_eq!(FolioRoute::PmcPortfolioMap.path(), "/pmc/map");
    }

    #[test]
    fn str_listing_index_path() {
        // GAP-5 regression: nav "Listings" must go to /s/listings (not /s/listings/:id)
        assert_eq!(FolioRoute::StrHostListingIndex.path(), "/s/listings");
    }

    #[test]
    fn str_listing_detail_path_has_param() {
        assert_eq!(FolioRoute::StrHostListings.path(), "/s/listings/:id");
    }

    #[test]
    fn str_syndication_path() {
        assert_eq!(FolioRoute::StrHostSyndication.path(), "/s/syndication");
    }

    // ── is_namespace_root ─────────────────────────────────────────────────────

    #[test]
    fn dashboards_are_namespace_roots() {
        assert!(FolioRoute::LandlordDashboard.is_namespace_root());
        assert!(FolioRoute::TenantDashboard.is_namespace_root());
        assert!(FolioRoute::VendorDashboard.is_namespace_root());
        assert!(FolioRoute::PmcDashboard.is_namespace_root());
        assert!(FolioRoute::OwnerDashboard.is_namespace_root());
        assert!(FolioRoute::StrHostDashboard.is_namespace_root());
        assert!(FolioRoute::AgentDashboard.is_namespace_root());
        assert!(FolioRoute::BrokerDashboard.is_namespace_root());
    }

    #[test]
    fn non_dashboards_are_not_namespace_roots() {
        // Regression: adding a new route must not accidentally be marked as root
        assert!(!FolioRoute::LandlordAssets.is_namespace_root());
        assert!(!FolioRoute::TenantInbox.is_namespace_root());
        assert!(!FolioRoute::TenantHousehold.is_namespace_root());
        assert!(!FolioRoute::PmcPortfolioMap.is_namespace_root());
        assert!(!FolioRoute::StrHostListingIndex.is_namespace_root());
        assert!(!FolioRoute::Settings.is_namespace_root());
        assert!(!FolioRoute::Login.is_namespace_root());
    }

    // ── No path collisions (sampled) ─────────────────────────────────────────

    #[test]
    fn sampled_paths_are_unique() {
        let routes_and_paths = [
            (FolioRoute::LandlordDashboard, "/l"),
            (FolioRoute::TenantDashboard, "/t"),
            (FolioRoute::VendorDashboard, "/v"),
            (FolioRoute::PmcDashboard, "/pmc"),
            (FolioRoute::OwnerDashboard, "/o"),
            (FolioRoute::StrHostDashboard, "/s"),
            (FolioRoute::Settings, "/settings"),
            (FolioRoute::PmcPortfolioMap, "/pmc/map"),
            (FolioRoute::StrHostListingIndex, "/s/listings"),
            (FolioRoute::LandlordMeridian, "/l/meridian"),
            (FolioRoute::LandlordMeridianConfig, "/l/meridian/configure"),
            (FolioRoute::Login, "/login"),
        ];
        for (route, expected) in routes_and_paths {
            assert_eq!(route.path(), expected);
        }
        let paths: Vec<_> = routes_and_paths.iter().map(|(r, _)| r.path()).collect();
        let unique: std::collections::HashSet<_> = paths.iter().collect();
        assert_eq!(
            paths.len(),
            unique.len(),
            "path collision detected in sampled routes"
        );
    }

    // ── Display impl delegates to path() ─────────────────────────────────────

    #[test]
    fn display_matches_path() {
        let routes = [
            FolioRoute::LandlordDashboard,
            FolioRoute::Settings,
            FolioRoute::TenantInbox,
            FolioRoute::StrHostSyndication,
        ];
        for r in routes {
            assert_eq!(format!("{r}"), r.path(), "Display != path() for {r:?}");
        }
    }
}

// ── NavItem — strongly typed ──────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NavItem {
    pub route: FolioRoute,
    pub label: &'static str,
    pub icon: NavIcon,
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
    pub role_label: &'static str,
    pub groups: &'static [NavGroup],
    pub footer_items: &'static [NavItem],
}

// ── Canonical configs — one per role ─────────────────────────────────────────
//
// Source of truth for nav structure. Edit here only.
// Derived from: designs/stitch/project_pm/folio/l_assets/code.html
// and designs/stitch/project_pm/folio/ROUTES.md

pub(crate) static LANDLORD_NAV: NavConfig = NavConfig {
    role_label: "Landlord Portal",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::LandlordDashboard, "Dashboard", NavIcon::Home),
            NavItem::new(FolioRoute::LandlordAssets, "Assets", NavIcon::Apartment),
            NavItem::new(FolioRoute::LandlordLeases, "Leases", NavIcon::Description),
            NavItem::new(
                FolioRoute::LandlordMaintenance,
                "Maintenance",
                NavIcon::Build,
            ),
            NavItem::new(FolioRoute::LandlordDeals, "Deals", NavIcon::Handshake),
            NavItem::new(FolioRoute::LandlordMap, "Map", NavIcon::Map),
            NavItem::new(
                FolioRoute::LandlordCommunications,
                "Messages",
                NavIcon::Inbox,
            ),
            NavItem::new(FolioRoute::LandlordBilling, "Billing", NavIcon::ReceiptLong),
        ],
    }],
    footer_items: &[
        NavItem::new(FolioRoute::LandlordSetup, "Setup", NavIcon::Tune),
        NavItem::new(
            FolioRoute::LandlordMeridian,
            "Analytics",
            NavIcon::BarChart,
        ),
        NavItem::new(
            FolioRoute::LandlordAccountBilling,
            "Account",
            NavIcon::ManageAccounts,
        ),
        NavItem::new(FolioRoute::Settings, "Settings", NavIcon::Settings),
    ],
};

/// Hired PM operator on `/l` — ops nav without account-admin (Account / Team).
pub(crate) static HIRED_PM_LANDLORD_NAV: NavConfig = NavConfig {
    role_label: "Property Manager",
    groups: LANDLORD_NAV.groups,
    footer_items: &[
        NavItem::new(
            FolioRoute::LandlordMeridian,
            "Analytics",
            NavIcon::BarChart,
        ),
        NavItem::new(FolioRoute::Settings, "Settings", NavIcon::Settings),
    ],
};

pub(crate) static TENANT_NAV: NavConfig = NavConfig {
    role_label: "Tenant",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::TenantDashboard, "Dashboard", NavIcon::Home),
            NavItem::new(FolioRoute::TenantMyLease, "My Lease", NavIcon::Description),
            NavItem::new(FolioRoute::TenantPayments, "Payments", NavIcon::CreditCard),
            NavItem::new(FolioRoute::TenantMaintenance, "Maintenance", NavIcon::Build),
            NavItem::new(
                FolioRoute::TenantReservations,
                "Reservations",
                NavIcon::EventAvailable,
            ),
            NavItem::new(FolioRoute::TenantRatings, "Ratings", NavIcon::BarChart),
            NavItem::new(FolioRoute::TenantInbox, "Inbox", NavIcon::Inbox),
            NavItem::new(FolioRoute::TenantDocuments, "Documents", NavIcon::Folder),
            NavItem::new(FolioRoute::TenantHousehold, "Household", NavIcon::Group),
        ],
    }],
    footer_items: &[
        NavItem::new(FolioRoute::TenantProfile, "Profile", NavIcon::Person),
        NavItem::new(FolioRoute::TenantReports, "Reports", NavIcon::BarChart),
    ],
};

/// Guest Portal nav — STR booking guests (str_guest role).
/// Minimal, booking-focused. Separate from TENANT_NAV because the
/// journey is entirely different (reservation vs lease management).
pub(crate) static GUEST_NAV: NavConfig = NavConfig {
    role_label: "Guest",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::GuestDashboard, "My Stay", NavIcon::Home),
            NavItem::new(
                FolioRoute::GuestReservation,
                "Reservation",
                NavIcon::EventAvailable,
            ),
            NavItem::new(FolioRoute::GuestCheckIn, "Check-In", NavIcon::Login),
            NavItem::new(FolioRoute::GuestHouseRules, "House Rules", NavIcon::Gavel),
            NavItem::new(FolioRoute::GuestInbox, "Inbox", NavIcon::Inbox),
        ],
    }],
    footer_items: &[NavItem::new(
        FolioRoute::GuestProfile,
        "Profile",
        NavIcon::Person,
    )],
};

static VENDOR_NAV: NavConfig = NavConfig {
    role_label: "Vendor",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::VendorDashboard, "Dashboard", NavIcon::Home),
            NavItem::new(
                FolioRoute::VendorWorkOrders,
                "Work Orders",
                NavIcon::Assignment,
            ),
            NavItem::new(FolioRoute::VendorInvoices, "Invoices", NavIcon::ReceiptLong),
            NavItem::new(
                FolioRoute::VendorSchedule,
                "Schedule",
                NavIcon::CalendarMonth,
            ),
            NavItem::new(FolioRoute::VendorReferrals, "Referrals", NavIcon::Campaign),
        ],
    }],
    footer_items: &[NavItem::new(
        FolioRoute::VendorNetworkProfile,
        "Network Profile",
        NavIcon::Badge,
    )],
};

static PMC_NAV: NavConfig = NavConfig {
    role_label: "PMC",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::PmcDashboard, "Dashboard", NavIcon::Home),
            NavItem::new(FolioRoute::PmcClientBook, "Client Book", NavIcon::People),
            NavItem::new(FolioRoute::PmcMaintenance, "Maintenance", NavIcon::Build),
            NavItem::new(FolioRoute::PmcPortfolioMap, "Map", NavIcon::Map),
            NavItem::new(
                FolioRoute::PmcOwnerStatements,
                "Statements",
                NavIcon::ReceiptLong,
            ),
            NavItem::new(FolioRoute::PmcReferrals, "Referrals", NavIcon::Campaign),
        ],
    }],
    footer_items: &[NavItem::new(
        FolioRoute::Settings,
        "Settings",
        NavIcon::Settings,
    )],
};

static OWNER_NAV: NavConfig = NavConfig {
    role_label: "Owner",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::OwnerDashboard, "Dashboard", NavIcon::Home),
            NavItem::new(
                FolioRoute::OwnerProperties,
                "Properties",
                NavIcon::Apartment,
            ),
            NavItem::new(
                FolioRoute::OwnerStatements,
                "Statements",
                NavIcon::ReceiptLong,
            ),
            NavItem::new(
                FolioRoute::OwnerDistributions,
                "Distributions",
                NavIcon::Payments,
            ),
            NavItem::new(FolioRoute::OwnerMaintenance, "Maintenance", NavIcon::Build),
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
            NavItem::new(FolioRoute::AgentListings, "Listings", NavIcon::Apartment),
            NavItem::new(FolioRoute::AgentClients, "Clients", NavIcon::People),
            NavItem::new(FolioRoute::AgentDeals, "Deals", NavIcon::Handshake),
            NavItem::new(
                FolioRoute::AgentSchedule,
                "Schedule",
                NavIcon::CalendarMonth,
            ),
            NavItem::new(FolioRoute::AgentReferrals, "Referrals", NavIcon::Campaign),
        ],
    }],
    footer_items: &[NavItem::new(
        FolioRoute::Settings,
        "Settings",
        NavIcon::Settings,
    )],
};

static BROKER_NAV: NavConfig = NavConfig {
    role_label: "Broker",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::BrokerDashboard, "Dashboard", NavIcon::Home),
            NavItem::new(FolioRoute::BrokerAgents, "Agents", NavIcon::People),
            NavItem::new(FolioRoute::BrokerListings, "Listings", NavIcon::Apartment),
            NavItem::new(
                FolioRoute::BrokerCompliance,
                "Compliance",
                NavIcon::Verified,
            ),
            NavItem::new(FolioRoute::BrokerRevenue, "Revenue", NavIcon::Payments),
            NavItem::new(FolioRoute::BrokerReferrals, "Referrals", NavIcon::Campaign),
        ],
    }],
    footer_items: &[NavItem::new(
        FolioRoute::Settings,
        "Settings",
        NavIcon::Settings,
    )],
};

static PROPERTY_OWNER_LITE_NAV: NavConfig = NavConfig {
    role_label: "Property Owner",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(
                FolioRoute::PropertyOwnerLiteDashboard,
                "My Property",
                NavIcon::Home,
            ),
            NavItem::new(
                FolioRoute::PropertyOwnerLiteValue,
                "Property Value",
                NavIcon::ShowChart,
            ),
            NavItem::new(
                FolioRoute::PropertyOwnerLiteFindVendor,
                "Find a Vendor",
                NavIcon::Handyman,
            ),
        ],
    }],
    footer_items: &[NavItem::new(
        FolioRoute::Settings,
        "Settings",
        NavIcon::Settings,
    )],
};

static STR_HOST_NAV: NavConfig = NavConfig {
    role_label: "STR Host",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::StrHostDashboard, "Dashboard", NavIcon::Home),
            NavItem::new(
                FolioRoute::StrHostCalendar,
                "Calendar",
                NavIcon::CalendarMonth,
            ),
            NavItem::new(
                FolioRoute::StrHostReservations,
                "Reservations",
                NavIcon::EventAvailable,
            ),
            NavItem::new(
                FolioRoute::StrHostListingIndex,
                "Listings",
                NavIcon::Apartment,
            ),
            NavItem::new(FolioRoute::StrHostPricing, "Pricing", NavIcon::Sell),
            NavItem::new(FolioRoute::StrHostChannels, "Channels", NavIcon::SyncAlt),
            NavItem::new(
                FolioRoute::StrHostSyndication,
                "Syndication",
                NavIcon::SyncAlt,
            ),
            NavItem::new(FolioRoute::StrHostMessages, "Messages", NavIcon::Chat),
        ],
    }],
    footer_items: &[
        NavItem::new(FolioRoute::StrHostReviews, "Reviews", NavIcon::Star),
        NavItem::new(FolioRoute::StrHostIncidents, "Incidents", NavIcon::Report),
        NavItem::new(FolioRoute::Settings, "Settings", NavIcon::Settings),
    ],
};

static COHOST_NAV: NavConfig = NavConfig {
    role_label: "Cohost",
    groups: &[NavGroup {
        label: None,
        items: &[
            NavItem::new(FolioRoute::StrHostDashboard, "Dashboard", NavIcon::Home),
            NavItem::new(
                FolioRoute::StrHostCalendar,
                "Calendar",
                NavIcon::CalendarMonth,
            ),
            NavItem::new(
                FolioRoute::StrHostReservations,
                "Reservations",
                NavIcon::EventAvailable,
            ),
            NavItem::new(FolioRoute::StrHostMessages, "Messages", NavIcon::Chat),
        ],
    }],
    footer_items: &[NavItem::new(
        FolioRoute::StrHostIncidents,
        "Incidents",
        NavIcon::Report,
    )],
};

//
// This is the key architectural point: FolioRole IS the index into nav configs.
// Layouts don't import a specific *_NAV static — they pass `role.nav_config()`.
// Adding a new role requires adding it here — the compiler will enforce completeness.

impl FolioRole {
    pub fn nav_config(self) -> &'static NavConfig {
        match self {
            Self::Landlord => &LANDLORD_NAV,
            Self::Tenant => &TENANT_NAV,
            Self::StrGuest => &GUEST_NAV,
            Self::Vendor => &VENDOR_NAV,
            Self::PropertyManager => &PMC_NAV,
            Self::Owner => &OWNER_NAV,
            Self::Cohost => &COHOST_NAV,
            // StrHost removed — Landlord portal shows STR sections when
            // session.has_str_assets = true (asset-level trait, not a role)
            Self::Agent => &AGENT_NAV,
            Self::Broker => &BROKER_NAV,
            Self::PropertyOwnerLite => &PROPERTY_OWNER_LITE_NAV,
        }
    }
}

// ── SidebarNav Component ──────────────────────────────────────────────────────

#[component]
pub fn SidebarNav(
    config: &'static NavConfig,
    #[prop(optional)] user_name: Option<String>,
    #[prop(optional)] initials: Option<String>,
) -> impl IntoView {
    let location = use_location();

    let initials = initials
        .or_else(|| user_name.as_deref().map(derive_initials))
        .unwrap_or_else(|| "?".to_string());

    view! {
        <nav class="folio-sidebar">
            <div class="sidebar-brand">
                <span class="sidebar-logo">"Folio"</span>
                <span class="sidebar-role">{config.role_label}</span>
            </div>

            <div class="sidebar-scroll">
                {config.groups.iter().map(|group| {
                    view! {
                        <div class="nav-group">
                            {group.label.map(|l| view! {
                                <span class="nav-group-label">{l}</span>
                            })}
                            <ul class="nav-list">
                                {group.items.iter().map(|item| {
                                    let route = item.route;
                                    let pathname = location.pathname;
                                    view! {
                                        <li>
                                            <A
                                                href=route.path()
                                                attr:class=move || {
                                                    if is_active(&pathname.get(), route) {
                                                        "nav-link nav-link--active"
                                                    } else {
                                                        "nav-link"
                                                    }
                                                }
                                                attr:aria-current=move || {
                                                    is_active(&pathname.get(), route).then_some("page")
                                                }
                                            >
                                                <span class="material-symbols-outlined nav-link-icon">
                                                    {item.icon.as_str()}
                                                </span>
                                                <span class="nav-link-label">{item.label}</span>
                                            </A>
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
                    view! {
                        <ul class="nav-list nav-list--footer">
                            {config.footer_items.iter().map(|item| {
                                let route = item.route;
                                let pathname = location.pathname;
                                view! {
                                    <li>
                                        <A
                                            href=route.path()
                                            attr:class=move || {
                                                if is_active(&pathname.get(), route) {
                                                    "nav-link nav-link--active"
                                                } else {
                                                    "nav-link"
                                                }
                                            }
                                        >
                                            <span class="material-symbols-outlined nav-link-icon">
                                                {item.icon.as_str()}
                                            </span>
                                            <span class="nav-link-label">{item.label}</span>
                                        </A>
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
        current_path == route_path || current_path.starts_with(&format!("{route_path}/"))
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
