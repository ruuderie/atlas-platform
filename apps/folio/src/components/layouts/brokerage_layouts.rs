use leptos::prelude::*;
use leptos_router::components::Outlet;

/// Persistent shell for all /a/** agent routes.
/// Requires `folio_mode = "brokerage"` on the instance.
#[component]
pub fn AgentLayout() -> impl IntoView {
    view! {
        <div class="folio-layout folio-layout--agent">
            <nav class="folio-nav folio-nav--agent">
                <div class="nav-brand">
                    <span class="nav-logo">"Folio"</span>
                    <span class="nav-role-badge nav-role-badge--agent">"Agent Portal"</span>
                </div>
                <ul class="nav-links">
                    <NavLink href="/a"              label="Dashboard"   icon="⊞"/>
                    <NavLink href="/a/clients"      label="My Clients"  icon="👤"/>
                    <NavLink href="/a/listings"     label="Listings"    icon="🏠"/>
                    <NavLink href="/a/deals"        label="Deals"       icon="🤝"/>
                    <NavLink href="/a/schedule"     label="Schedule"    icon="📅"/>
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

/// Persistent shell for all /b/** broker routes.
/// Requires `folio_mode = "brokerage"` on the instance.
#[component]
pub fn BrokerLayout() -> impl IntoView {
    view! {
        <div class="folio-layout folio-layout--broker">
            <nav class="folio-nav folio-nav--broker">
                <div class="nav-brand">
                    <span class="nav-logo">"Folio"</span>
                    <span class="nav-role-badge nav-role-badge--broker">"Broker Office"</span>
                </div>
                <ul class="nav-links">
                    <NavLink href="/b"            label="Office Overview" icon="⊞"/>
                    <NavLink href="/b/agents"     label="Agent Roster"   icon="👥"/>
                    <NavLink href="/b/listings"   label="All Listings"   icon="🏠"/>
                    <NavLink href="/b/compliance" label="Compliance"     icon="📋"/>
                    <NavLink href="/b/revenue"    label="Revenue"        icon="💰"/>
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
        <li><a href=href class="nav-link">
            <span class="nav-icon">{icon}</span>
            <span class="nav-label">{label}</span>
        </a></li>
    }
}

#[component]
fn LogoutButton() -> impl IntoView {
    view! {
        <button class="nav-logout" on:click=move |_| {
            leptos::task::spawn_local(async {
                let _ = crate::auth::revoke_session().await;
                let _ = web_sys::window().map(|w| { let _ = w.location().set_href("/login"); });
            });
        }>"Sign out"</button>
    }
}
