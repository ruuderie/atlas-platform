use leptos::prelude::*;
use leptos_router::components::Outlet;

/// Persistent shell for all /pmc/** property management company routes.
#[component]
pub fn PmcLayout() -> impl IntoView {
    view! {
        <div class="folio-layout folio-layout--pmc">
            <nav class="folio-nav folio-nav--pmc">
                <div class="nav-brand">
                    <span class="nav-logo">"Folio"</span>
                    <span class="nav-role-badge nav-role-badge--pmc">"PMC Dashboard"</span>
                </div>
                <ul class="nav-links">
                    <NavLink href="/pmc"          label="Overview"    icon="⊞"/>
                    <NavLink href="/pmc/clients"  label="Client Book" icon="🏢"/>
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
