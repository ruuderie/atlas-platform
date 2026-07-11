use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{ParentRoute, Redirect, Route, Router, Routes};
use leptos_router::path;

use crate::auth::{FolioRole, SessionInfo};
use crate::pages::login::Login;
use crate::pages::not_found::NotFound;

use crate::pages::marketing::beta_program_page::BetaProgramPage;
use crate::pages::marketing::broker_landing_page::BrokerLandingPage;
use crate::pages::marketing::founding_member_page::FoundingMemberPage;
use crate::pages::marketing::market_landing_page::MarketLandingPage;
use crate::pages::marketing::property_manager_landing_page::PropertyManagerLandingPage;
use crate::pages::marketing::vendor_landing_page::VendorLandingPage;

// Landlord pages
use crate::pages::landlord::{
    account_billing::LandlordAccountBilling, asset_alerts::AssetAlerts, asset_detail::AssetDetail,
    assets::Assets, billing::Billing, building_systems::BuildingSystems, campaigns::Campaigns,
    catalog::Catalog, communications::Communications,
    contractor_marketplace::ContractorMarketplace, dashboard::LandlordDashboard,
    digital_vault::LandlordDigitalVault, inspections::Inspections, leads::Leads,
    lease_detail::LeaseDetail, leases::Leases, ledger::Ledger,
    listing_preview::ListingNetworkPreview, maintenance_queue::MaintenanceQueue,
    map_portfolio::MapPortfolio, meridian_config::MeridianConfigurator,
    notifications::NotificationsPage, portfolio::Portfolio, ratings::LandlordRatings,
    reservations::LandlordReservations, str_compliance::StrCompliance,
    syndication::LandlordSyndication, team::LandlordTeam, tenant_profile::TenantProfile,
    unit_appliances::UnitAppliances, vendors::Vendors, violations::Violations,
    wholesaling::LandlordWholesaling,
};

// Tenant pages
use crate::pages::tenant::{
    application_status::TenantApplicationStatus, dashboard::TenantDashboard,
    documents::TenantDocuments, household::TenantHousehold, inbox::TenantInbox,
    maintenance::MaintenanceRequests, maintenance_detail::TenantMaintenanceDetail,
    my_lease::MyLease, payment_history::TenantPaymentHistory, payments::TenantPayments,
    profile::TenantProfile as TenantProfilePage, ratings::TenantRatings, reports::TenantReports,
    reservations::TenantReservations, violations::TenantViolations,
};

// Vendor pages
use crate::pages::vendor::{
    dashboard::VendorDashboard, invoices::VendorInvoices, network_profile::VendorNetworkProfile,
    schedule::VendorSchedule, work_orders::WorkOrders,
};

// PMC pages
use crate::pages::pmc::{
    client_book::ClientBook, client_detail::PmcClientDetail, dashboard::PmcDashboard,
    maintenance_dispatch::PmcMaintenanceDispatch, owner_statements::PmcOwnerStatements,
    portfolio_map::PmcPortfolioMap,
};

// Owner pages
use crate::pages::owner::{
    dashboard::OwnerDashboard, distributions::OwnerDistributions,
    maintenance::OwnerMaintenanceApproval, property::OwnerPropertyDetail,
    statements::OwnerStatements,
};

// STR Host pages
use crate::pages::str_host::{
    calendar::StrCalendar, channels::StrChannelManager, dashboard::StrHostDashboard,
    incidents::StrIncidents, listing::StrListingDetail, listing_index::StrListingIndex,
    messages::StrGuestMessaging, pricing::StrPricingRules, reservations::StrReservationManifest,
    reviews::StrReviews, syndication::StrSyndication, violation_file::StrViolationFiling,
};

// Wizard pages (public + token-gated)
use crate::pages::auth::passkey_setup::PasskeySetup;
use crate::pages::marketing::cohost_marketplace::CohostMarketplace;
use crate::pages::marketing::inquiry_confirm::InquiryConfirm;
use crate::pages::marketing::lead_portal::LeadPortal;
use crate::pages::marketing::ltr_listings::LtrListings;
use crate::pages::marketing::ni_signup::NiSignup;
use crate::pages::marketing::renter_application::RenterApplication;
use crate::pages::marketing::str_listings::StrListings;
use crate::pages::onboarding::agent_wizard::AgentWizard;
use crate::pages::onboarding::broker_wizard::BrokerWizard;
use crate::pages::onboarding::cohost_wizard::CohostWizard;
use crate::pages::onboarding::invite_join::InviteJoin;
use crate::pages::onboarding::landlord_wizard::LandlordWizard;
use crate::pages::onboarding::owner_wizard::OwnerWizard;
use crate::pages::onboarding::pmc_wizard::PmcWizard;
use crate::pages::onboarding::property_owner_wizard::PropertyOwnerWizard;
use crate::pages::onboarding::str_guest_wizard::StrGuestWizard;
use crate::pages::onboarding::tenant_wizard::TenantApplicantWizard;
use crate::pages::onboarding::vendor_wizard::VendorWizard;
use crate::pages::onboarding::wizard::OnboardingWizard;
use crate::pages::pmc::onboard::PmcOnboard;
use crate::pages::settings::Settings;
use crate::pages::tenant::maintenance_triage::TenantMaintenanceTriage;
use crate::pages::vendor::job_link::VendorJobLink;
use crate::pages::vendor::onboard::VendorOnboard;

// Agent pages
use crate::pages::agent::dashboard::{
    AgentClients, AgentDashboard, AgentDeals, AgentListings, AgentSchedule,
};

// Broker pages
use crate::pages::broker::dashboard::{
    BrokerAgents, BrokerCompliance, BrokerDashboard, BrokerListings, BrokerRevenue,
};

// Property Owner Lite pages
use crate::pages::property_owner::{
    dashboard::PropertyOwnerDashboard, find_vendor::FindVendorPage,
    property_value::PropertyValuePage, review_submit::ReviewSubmitPage,
};

// Public zero-auth pages
use crate::pages::r#pub::renter_help::RenterHelpPage;

// Layouts — each already renders <Outlet/> for its child routes
use crate::components::layouts::{
    brokerage_layouts::{AgentLayout, BrokerLayout},
    landlord_layout::LandlordLayout,
    owner_layout::OwnerLayout,
    pmc_layout::PmcLayout,
    property_owner_layout::PropertyOwnerLayout,
    tenant_layout::TenantLayout,
    vendor_layout::VendorLayout,
};

/// Root application shell. Sets up meta context and the router.
///
/// # Session strategy
/// `check_session()` is intentionally NOT called here. Marketing pages
/// (`/`, `/beta`, `/brokers`, `/founding`, etc.) are public and must
/// render with zero auth overhead — no round-trip to the backend.
///
/// Session checks happen lazily, only where they are needed:
/// - `HomeDispatch` (`/`)       — must branch authenticated vs anonymous
/// - `role_shell_view` (`/l`, `/t`, etc.) — must guard authenticated routes
///
/// See docs/leptos_architecture_decisions.md § 5 for the full rationale.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Router>
            <Routes fallback=|| view! { <NotFound/> }>
                // ── Public ────────────────────────────────────────────────────
                <Route path=path!("/login")              view=Login/>
                // /verify is handled by the SSR-only Axum handler in main.rs — not a Leptos route.

                // Auth + first-run — no layout chrome
                <Route path=path!("/auth/passkey-setup") view=PasskeySetup/>
                // /onboarding → LandlordWizard (split-panel, invite-code aware)
                <Route path=path!("/onboarding")         view=LandlordWizard/>
                // Persona-specific onboard paths (all invite-code aware via ?code=)
                <Route path=path!("/onboard/tenant")     view=TenantApplicantWizard/>
                <Route path=path!("/onboard/str-guest")  view=StrGuestWizard/>
                <Route path=path!("/onboard/cohost")     view=CohostWizard/>
                <Route path=path!("/onboard/owner")      view=OwnerWizard/>
                <Route path=path!("/onboard/property-owner") view=PropertyOwnerWizard/>
                <Route path=path!("/onboard/agent")      view=AgentWizard/>
                <Route path=path!("/onboard/broker")     view=BrokerWizard/>
                <Route path=path!("/onboard/pmc")        view=PmcWizard/>
                <Route path=path!("/onboard/vendor")     view=VendorWizard/>
                // Public invite join page — no auth, resolves code and shows context card
                <Route path=path!("/join/:code")          view=InviteJoin/>

                // ── Marketing landing pages (zero-auth SSR) ───────────────────
                // /lp              → product master page (folio.app)
                // /lp/:variant_slug → market variant    (miami.folio.app → /lp/miami-fl)
                // Placed before role dispatch so CDN requests match without auth.
                <Route path=path!("/lp")               view=MarketLandingPage/>
                <Route path=path!("/lp/:variant_slug") view=MarketLandingPage/>
                // /brokers → independent broker/PMC landing page (app_id = "folio-broker")
                <Route path=path!("/brokers")              view=BrokerLandingPage/>
                // /property-managers → property manager / PMC landing page (app_id = "folio-pm")
                <Route path=path!("/property-managers")    view=PropertyManagerLandingPage/>
                // /vendors → vendor portal landing page (app_id = "folio-vendor")
                <Route path=path!("/vendors")              view=VendorLandingPage/>
                // /founding → Lifetime / founding member offer page (fundraising)
                <Route path=path!("/founding")             view=FoundingMemberPage/>
                // /beta → Beta program application page
                <Route path=path!("/beta")                 view=BetaProgramPage/>

                // ── Home dispatch: / → marketing page (unauth) or role portal (auth) ──
                <Route path=path!("/") view=HomeDispatch/>

                // ── Landlord namespace /l/** ───────────────────────────────────
                // LandlordShell: checks role, redirects if wrong, then renders
                // LandlordLayout which contains <Outlet/> for child routes.
                <ParentRoute path=path!("/l") view=LandlordShell>
                    <Route path=path!("")               view=LandlordDashboard/>
                    <Route path=path!("/portfolio")     view=Portfolio/>
                    <Route path=path!("/assets")        view=Assets/>
                    <Route path=path!("/assets/:id")    view=AssetDetail/>
                    <Route path=path!("/assets/:id/preview") view=ListingNetworkPreview/>
                    <Route path=path!("/assets/:id/alerts")  view=AssetAlerts/>
                    <Route path=path!("/leases")        view=Leases/>
                    <Route path=path!("/leases/:id")    view=LeaseDetail/>
                    <Route path=path!("/tenants/:id")   view=TenantProfile/>
                    <Route path=path!("/maintenance")   view=MaintenanceQueue/>
                    <Route path=path!("/ledger")        view=Ledger/>
                    <Route path=path!("/violations")    view=Violations/>
                    <Route path=path!("/inspections")   view=Inspections/>
                    <Route path=path!("/systems")       view=BuildingSystems/>
                    <Route path=path!("/appliances")    view=UnitAppliances/>
                    <Route path=path!("/communications")view=Communications/>
                    <Route path=path!("/map")           view=MapPortfolio/>
                    <Route path=path!("/notifications") view=NotificationsPage/>
                    <Route path=path!("/leads")         view=Leads/>
                    <Route path=path!("/campaigns")     view=Campaigns/>
                    <Route path=path!("/billing")       view=Billing/>
                    <Route path=path!("/str")           view=StrCompliance/>
                    <Route path=path!("/catalog")       view=Catalog/>
                    <Route path=path!("/vendors")       view=Vendors/>
                    <Route path=path!("/reservations")  view=LandlordReservations/>
                    <Route path=path!("/marketplace")   view=ContractorMarketplace/>
                    <Route path=path!("/vault")         view=LandlordDigitalVault/>
                    <Route path=path!("/syndication")   view=LandlordSyndication/>
                    <Route path=path!("/wholesaling")   view=LandlordWholesaling/>
                    <Route path=path!("/account/billing")view=LandlordAccountBilling/>
                    <Route path=path!("/meridian/configure") view=MeridianConfigurator/>
                    <Route path=path!("/ratings") view=LandlordRatings/>
                    <Route path=path!("/team")           view=LandlordTeam/>
                </ParentRoute>

                // ── Tenant namespace /t/** ─────────────────────────────────────
                <ParentRoute path=path!("/t") view=TenantShell>
                    <Route path=path!("")                    view=TenantDashboard/>
                    <Route path=path!("/my-lease")           view=MyLease/>
                    <Route path=path!("/payments")           view=TenantPayments/>
                    <Route path=path!("/payments/history")   view=TenantPaymentHistory/>
                    <Route path=path!("/maintenance")          view=MaintenanceRequests/>
                    <Route path=path!("/maintenance/new")      view=TenantMaintenanceTriage/>
                    <Route path=path!("/maintenance/:id")      view=TenantMaintenanceDetail/>
                    <Route path=path!("/reservations")         view=TenantReservations/>
                    <Route path=path!("/ratings")              view=TenantRatings/>
                    <Route path=path!("/inbox")                view=TenantInbox/>
                    <Route path=path!("/household")            view=TenantHousehold/>
                    <Route path=path!("/docs")                 view=TenantDocuments/>
                    <Route path=path!("/violations")           view=TenantViolations/>
                    <Route path=path!("/profile")              view=TenantProfilePage/>
                    <Route path=path!("/application")          view=TenantApplicationStatus/>
                    <Route path=path!("/reports")              view=TenantReports/>
                </ParentRoute>

                // ── Vendor namespace /v/** ─────────────────────────────────────
                <ParentRoute path=path!("/v") view=VendorShell>
                    <Route path=path!("")              view=VendorDashboard/>
                    <Route path=path!("/work-orders")  view=WorkOrders/>
                    <Route path=path!("/invoices")     view=VendorInvoices/>
                    <Route path=path!("/schedule")     view=VendorSchedule/>
                    <Route path=path!("/profile")      view=VendorNetworkProfile/>
                    <Route path=path!("/onboard")      view=VendorOnboard/>
                </ParentRoute>

                // ── STR Host namespace /s/** ───────────────────────────────────
                // Active when the landlord has STR assets or mode = str_host.
                <ParentRoute path=path!("/s") view=LandlordShell>
                    <Route path=path!("")                  view=StrHostDashboard/>
                    <Route path=path!("/calendar")         view=StrCalendar/>
                    <Route path=path!("/reservations")     view=StrReservationManifest/>
                    <Route path=path!("/listings/:id")     view=StrListingDetail/>
                    <Route path=path!("/pricing")          view=StrPricingRules/>
                    <Route path=path!("/channels")         view=StrChannelManager/>
                    <Route path=path!("/messages")         view=StrGuestMessaging/>
                    <Route path=path!("/reviews")          view=StrReviews/>
                    <Route path=path!("/incidents")        view=StrIncidents/>
                    <Route path=path!("/violations/new")   view=StrViolationFiling/>
                    <Route path=path!("/listings")         view=StrListingIndex/>      // index — nav target
                    <Route path=path!("/syndication")      view=StrSyndication/>       // per-listing channel distribution
                    // /s/listings/:id  — detail, linked from cards (no shell nav item)
                </ParentRoute>

                // ── PMC namespace /pmc/** ──────────────────────────────────────
                // Only accessible when folio_mode = "pmc" on the instance.
                // PmcShell checks role = PropertyManager; backend guards check folio_mode.
                <ParentRoute path=path!("/pmc") view=PmcShell>
                    <Route path=path!("")             view=PmcDashboard/>
                    <Route path=path!("/clients")     view=ClientBook/>
                    <Route path=path!("/clients/:id") view=PmcClientDetail/>
                    <Route path=path!("/maintenance") view=PmcMaintenanceDispatch/>
                    <Route path=path!("/statements")  view=PmcOwnerStatements/>
                    <Route path=path!("/map")         view=PmcPortfolioMap/>
                </ParentRoute>

                // ── Owner namespace /o/** ──────────────────────────────────────
                // Read-only portal for beneficial property owners.
                // Owner cannot create, edit, or delete any resource.
                <ParentRoute path=path!("/o") view=OwnerShell>
                    <Route path=path!("")                   view=OwnerDashboard/>
                    <Route path=path!("/properties/:id")    view=OwnerPropertyDetail/>
                    <Route path=path!("/statements")        view=OwnerStatements/>
                    <Route path=path!("/distributions")     view=OwnerDistributions/>
                    <Route path=path!("/maintenance")       view=OwnerMaintenanceApproval/>
                </ParentRoute>

                // ── Public wizards (no auth required) ─────────────────────────
                <Route path=path!("/apply/:property_id") view=RenterApplication/>
                <Route path=path!("/leads/:token")       view=LeadPortal/>
                <Route path=path!("/inquiry/thanks")     view=InquiryConfirm/>
                <Route path=path!("/jobs/:token")        view=VendorJobLink/>
                // /pmc/onboard — admin-initiated wizard, not reachable from PMC sidebar nav.
                // Invoked via email link sent by an Atlas platform administrator.
                // See docs/folio/page_queue.md § P3 for rationale.
                <Route path=path!("/pmc/onboard")        view=PmcOnboard/>

                <Route path=path!("/listings/ltr")       view=LtrListings/>
                <Route path=path!("/listings/str")       view=StrListings/>
                <Route path=path!("/ni/signup")          view=NiSignup/>
                // Cohost Network marketplace — public discovery page
                <Route path=path!("/cohost-market")      view=CohostMarketplace/>
                // ── Shared authenticated routes (all roles) ────────────────────
                <Route path=path!("/settings")           view=Settings/>

                // ── Agent namespace /a/** ──────────────────────────────────────
                // Only valid when folio_mode = "brokerage" on the instance.
                // Backend API guards enforce the folio_mode constraint.
                <ParentRoute path=path!("/a") view=AgentShell>
                    <Route path=path!("")            view=AgentDashboard/>
                    <Route path=path!("/clients")   view=AgentClients/>
                    <Route path=path!("/listings")  view=AgentListings/>
                    <Route path=path!("/deals")     view=AgentDeals/>
                    <Route path=path!("/schedule")  view=AgentSchedule/>
                </ParentRoute>

                // ── Broker namespace /b/** ─────────────────────────────────────
                // Licensed broker — manages the office, agents, and compliance.
                <ParentRoute path=path!("/b") view=BrokerShell>
                    <Route path=path!("")             view=BrokerDashboard/>
                    <Route path=path!("/agents")     view=BrokerAgents/>
                    <Route path=path!("/listings")   view=BrokerListings/>
                    <Route path=path!("/compliance") view=BrokerCompliance/>
                    <Route path=path!("/revenue")    view=BrokerRevenue/>
                </ParentRoute>

                // ── Property Owner Lite namespace /po/** ────────────────────────
                // Free-tier self-registered owner: value tracking + vendor browse.
                // Auth guard: PropertyOwnerLite role (via role_shell_view).
                <ParentRoute path=path!("/po") view=PropertyOwnerShell>
                    <Route path=path!("")               view=PropertyOwnerDashboard/>
                    <Route path=path!("/value")         view=PropertyValuePage/>
                    <Route path=path!("/find-vendor")   view=FindVendorPage/>
                </ParentRoute>

                // ── Public review route /review/:invite_id ───────────────────
                // Zero-auth — vendor sends this link to past clients.
                // Inline OTP gate lives inside ReviewSubmitPage.
                <Route path=path!("/review/:invite_id") view=ReviewSubmitPage/>

                // ── Public renter help /help ────────────────────────────
                // Zero-auth — entry from Google, vendor links, QR codes.
                // ?vendor_id= pre-selects vendor, ?trade= pre-filters category.
                <Route path=path!("/help") view=RenterHelpPage/>
            </Routes>
        </Router>
    }
}

// ── Per-role shell components ─────────────────────────────────────────────────
//
// Each shell:
//   1. Reads the shared session resource
//   2. Redirects to /login if unauthenticated
//   3. Redirects to the correct namespace if wrong role
//   4. Renders the layout (which includes <Outlet/>) if authorized
//
// No children prop needed — the child routes render into <Outlet/>
// inside the layout, which is how Leptos 0.8 ParentRoute works.

#[component]
fn LandlordShell() -> impl IntoView {
    role_shell_view(FolioRole::Landlord, || {
        view! { <LandlordLayout/> }.into_any()
    })
}

#[component]
fn TenantShell() -> impl IntoView {
    role_shell_view(FolioRole::Tenant, || view! { <TenantLayout/> }.into_any())
}

#[component]
fn VendorShell() -> impl IntoView {
    role_shell_view(FolioRole::Vendor, || view! { <VendorLayout/> }.into_any())
}

#[component]
fn PmcShell() -> impl IntoView {
    role_shell_view(FolioRole::PropertyManager, || {
        view! { <PmcLayout/> }.into_any()
    })
}

#[component]
fn OwnerShell() -> impl IntoView {
    role_shell_view(FolioRole::Owner, || view! { <OwnerLayout/> }.into_any())
}

#[component]
fn AgentShell() -> impl IntoView {
    role_shell_view(FolioRole::Agent, || view! { <AgentLayout/> }.into_any())
}

#[component]
fn BrokerShell() -> impl IntoView {
    role_shell_view(FolioRole::Broker, || view! { <BrokerLayout/> }.into_any())
}

#[component]
fn PropertyOwnerShell() -> impl IntoView {
    role_shell_view(FolioRole::PropertyOwnerLite, || {
        view! { <PropertyOwnerLayout/> }.into_any()
    })
}

/// Shared guard logic for all role shells.
///
/// Creates its own `get_session` Resource locally rather than reading
/// from context. This is intentional — see App doc comment for the
/// session strategy.
fn role_shell_view(
    required: FolioRole,
    layout: impl Fn() -> AnyView + Send + Sync + 'static,
) -> impl IntoView {
    use crate::auth::get_session;
    let session = Resource::new(|| (), |_| get_session());

    view! {
        <Suspense fallback=|| view! { <FullPageLoader/> }>
            {move || session.get().map(|result| match result {
                Err(_) => view! { <Redirect path="/login"/> }.into_any(),
                Ok(ref info) if info.folio_role != required =>
                    view! { <Redirect path=info.folio_role.home_path()/> }.into_any(),
                Ok(_) => layout(),
            })}
        </Suspense>
    }
}

// ── HomeDispatch — dispatches / based on session state ──────────────────────
//
// Authenticated  → role portal (same as old RoleRedirect)
// Unauthenticated → MarketLandingPage (marketing homepage)
//
// Login is reached via the nav "Sign in" link on the marketing page,
// not by an automatic redirect. This ensures first-time visitors see
// the product before being asked to authenticate.

#[component]
fn HomeDispatch() -> impl IntoView {
    // Creates its own session resource — not shared from App context.
    // See App doc comment for the session strategy.
    use crate::auth::get_session;
    let session = Resource::new(|| (), |_| get_session());

    view! {
        <Suspense fallback=|| view! { <FullPageLoader/> }>
            {move || session.get().map(|r| match r {
                Ok(info) => view! { <Redirect path=info.folio_role.home_path()/> }.into_any(),
                Err(_)   => view! { <MarketLandingPage/> }.into_any(),
            })}
        </Suspense>
    }
}

// ── Full-page loader ─────────────────────────────────────────────────────────

#[component]
fn FullPageLoader() -> impl IntoView {
    view! { <div class="loading-screen"><span class="loader-dot"/></div> }
}
