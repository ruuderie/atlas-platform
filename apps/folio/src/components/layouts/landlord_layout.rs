use leptos::prelude::*;
use leptos_router::components::Outlet;

/// Persistent shell for all /l/** landlord routes.
/// Renders left sidebar nav + main content area.
#[component]
pub fn LandlordLayout() -> impl IntoView {
    view! {
        <div class="folio-layout folio-layout--landlord">
            <nav class="folio-nav">
                <div class="nav-brand">
                    <span class="nav-logo">"Folio"</span>
                    <span class="nav-role-badge">"Property Manager"</span>
                </div>
                <ul class="nav-links">
                    <NavLink href="/l"              label="Overview"     icon="⊞"/>
                    <NavLink href="/l/portfolio"    label="Portfolio"    icon="🏠"/>
                    <NavLink href="/l/assets"       label="Assets"       icon="🗂"/>
                    <NavLink href="/l/leases"       label="Leases"       icon="📋"/>
                    <NavLink href="/l/leads"        label="Leads"        icon="👤"/>
                    <NavLink href="/l/reservations" label="Reservations" icon="📅"/>
                    <NavLink href="/l/campaigns"    label="Campaigns"    icon="📣"/>
                    <NavLink href="/l/vendors"      label="Vendors"      icon="🔧"/>
                    <NavLink href="/l/billing"      label="Billing"      icon="💰"/>
                    <NavLink href="/l/str"          label="STR Permits"  icon="🏷"/>
                    <NavLink href="/l/catalog"      label="Catalog"      icon="📦"/>
                </ul>
                <div class="nav-footer">
                    <LogoutButton/>
                </div>
            </nav>
            <main class="folio-main">
                <Outlet/>
            </main>
        </div>
    }
}

#[component]
fn NavLink(href: &'static str, label: &'static str, icon: &'static str) -> impl IntoView {
    view! {
        <li>
            <a href=href class="nav-link">
                <span class="nav-icon">{icon}</span>
                <span class="nav-label">{label}</span>
            </a>
        </li>
    }
}

#[component]
fn LogoutButton() -> impl IntoView {
    view! {
        <button class="nav-logout" on:click=move |_| {
            leptos::spawn_local(async {
                let _ = crate::auth::revoke_session().await;
                let _ = web_sys::window().map(|w| { let _ = w.location().set_href("/login"); });
            });
        }>"Sign out"</button>
    }
}
