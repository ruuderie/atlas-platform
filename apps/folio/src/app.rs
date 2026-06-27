use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Redirect, Route, Router, Routes, ParentRoute};
use leptos_router::path;

use crate::auth::{FolioRole, SessionInfo, check_session};
use crate::pages::not_found::NotFound;
use crate::pages::login::Login;
use crate::pages::verify::Verify;
use crate::pages::marketing::market_landing_page::MarketLandingPage;

// Landlord pages
use crate::pages::landlord::{
    dashboard::LandlordDashboard,
    portfolio::Portfolio,
    assets::Assets,
    asset_detail::AssetDetail,
    leases::Leases,
    lease_detail::LeaseDetail,
    leads::Leads,
    campaigns::Campaigns,
    billing::Billing,
    str_compliance::StrCompliance,
    catalog::Catalog,
    vendors::Vendors,
    reservations::LandlordReservations,
};

// Tenant pages
use crate::pages::tenant::{
    dashboard::TenantDashboard,
    my_lease::MyLease,
    payments::TenantPayments,
    maintenance::MaintenanceRequests,
    reservations::TenantReservations,
};

// Vendor pages
use crate::pages::vendor::{
    dashboard::VendorDashboard,
    work_orders::WorkOrders,
    invoices::VendorInvoices,
};

// PMC pages
use crate::pages::pmc::{
    dashboard::PmcDashboard,
    client_book::ClientBook,
};

// Owner pages
use crate::pages::owner::dashboard::OwnerDashboard;

// Agent pages
use crate::pages::agent::dashboard::{
    AgentDashboard, AgentClients, AgentListings, AgentDeals, AgentSchedule,
};

// Broker pages
use crate::pages::broker::dashboard::{
    BrokerDashboard, BrokerAgents, BrokerListings, BrokerCompliance, BrokerRevenue,
};

// Layouts — each already renders <Outlet/> for its child routes
use crate::components::layouts::{
    landlord_layout::LandlordLayout,
    tenant_layout::TenantLayout,
    vendor_layout::VendorLayout,
    pmc_layout::PmcLayout,
    owner_layout::OwnerLayout,
    brokerage_layouts::{AgentLayout, BrokerLayout},
};

/// Root application. Provides session context once, then routes to the
/// appropriate namespace via RoleRedirect.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Single session resource — fetched once, provided via context to all children.
    let session = Resource::new(|| (), |_| check_session());
    provide_context(session);

    view! {
        <Router>
            <Routes fallback=|| view! { <NotFound/> }>
                // ── Public ────────────────────────────────────────────────────
                <Route path=path!("/login")  view=Login/>
                <Route path=path!("/verify") view=Verify/>

                // ── Marketing landing pages (zero-auth SSR) ───────────────────
                // /lp              → product master page (folio.app)
                // /lp/:variant_slug → market variant    (miami.folio.app → /lp/miami-fl)
                // Placed before role dispatch so CDN requests match without auth.
                <Route path=path!("/lp")               view=MarketLandingPage/>
                <Route path=path!("/lp/:variant_slug") view=MarketLandingPage/>

                // ── Role dispatch: / → namespace ──────────────────────────────
                <Route path=path!("/") view=RoleRedirect/>

                // ── Landlord namespace /l/** ───────────────────────────────────
                // LandlordShell: checks role, redirects if wrong, then renders
                // LandlordLayout which contains <Outlet/> for child routes.
                <ParentRoute path=path!("/l") view=LandlordShell>
                    <Route path=path!("")             view=LandlordDashboard/>
                    <Route path=path!("/portfolio")   view=Portfolio/>
                    <Route path=path!("/assets")      view=Assets/>
                    <Route path=path!("/assets/:id")  view=AssetDetail/>
                    <Route path=path!("/leases")      view=Leases/>
                    <Route path=path!("/leases/:id")  view=LeaseDetail/>
                    <Route path=path!("/leads")       view=Leads/>
                    <Route path=path!("/campaigns")   view=Campaigns/>
                    <Route path=path!("/billing")     view=Billing/>
                    <Route path=path!("/str")         view=StrCompliance/>
                    <Route path=path!("/catalog")     view=Catalog/>
                    <Route path=path!("/vendors")     view=Vendors/>
                    <Route path=path!("/reservations") view=LandlordReservations/>
                </ParentRoute>

                // ── Tenant namespace /t/** ─────────────────────────────────────
                <ParentRoute path=path!("/t") view=TenantShell>
                    <Route path=path!("")              view=TenantDashboard/>
                    <Route path=path!("/my-lease")     view=MyLease/>
                    <Route path=path!("/payments")     view=TenantPayments/>
                    <Route path=path!("/maintenance")  view=MaintenanceRequests/>
                    <Route path=path!("/reservations") view=TenantReservations/>
                </ParentRoute>

                // ── Vendor namespace /v/** ─────────────────────────────────────
                <ParentRoute path=path!("/v") view=VendorShell>
                    <Route path=path!("")              view=VendorDashboard/>
                    <Route path=path!("/work-orders")  view=WorkOrders/>
                    <Route path=path!("/invoices")     view=VendorInvoices/>
                </ParentRoute>

                // ── PMC namespace /pmc/** ──────────────────────────────────────
                // Only accessible when folio_mode = "pmc" on the instance.
                // PmcShell checks role = PropertyManager; backend guards check folio_mode.
                <ParentRoute path=path!("/pmc") view=PmcShell>
                    <Route path=path!("")          view=PmcDashboard/>
                    <Route path=path!("/clients")  view=ClientBook/>
                </ParentRoute>

                // ── Owner namespace /o/** ──────────────────────────────────────
                // Read-only portal for beneficial property owners.
                // Owner cannot create, edit, or delete any resource.
                <ParentRoute path=path!("/o") view=OwnerShell>
                    <Route path=path!("")           view=OwnerDashboard/>
                </ParentRoute>
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
    role_shell_view(FolioRole::Landlord, || view! { <LandlordLayout/> }.into_any())
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
    role_shell_view(FolioRole::PropertyManager, || view! { <PmcLayout/> }.into_any())
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

/// Shared guard logic for all role shells.
fn role_shell_view(required: FolioRole, layout: impl Fn() -> AnyView + Send + Sync + 'static) -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, server_fn::error::ServerFnError>>>()
        .expect("Session context missing");

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

// ── RoleRedirect — dispatches / to the correct namespace ─────────────────────

#[component]
fn RoleRedirect() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, server_fn::error::ServerFnError>>>()
        .expect("Session context missing");

    view! {
        <Suspense fallback=|| view! { <FullPageLoader/> }>
            {move || session.get().map(|r| match r {
                Ok(info) => view! { <Redirect path=info.folio_role.home_path()/> }.into_any(),
                Err(_)   => view! { <Redirect path="/login"/> }.into_any(),
            })}
        </Suspense>
    }
}

// ── Full-page loader ─────────────────────────────────────────────────────────

#[component]
fn FullPageLoader() -> impl IntoView {
    view! { <div class="loading-screen"><span class="loader-dot"/></div> }
}
