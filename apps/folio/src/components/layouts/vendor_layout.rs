use leptos::prelude::*;
use leptos_router::components::Outlet;

/// Persistent shell for all /v/** vendor routes.
#[component]
pub fn VendorLayout() -> impl IntoView {
    view! {
        <div class="folio-layout folio-layout--vendor">
            <nav class="folio-nav folio-nav--vendor">
                <div class="nav-brand">
                    <span class="nav-logo">"Folio"</span>
                    <span class="nav-role-badge nav-role-badge--vendor">"Vendor Portal"</span>
                </div>
                <ul class="nav-links">
                    <NavLink href="/v"              label="Dashboard"    icon="⊞"/>
                    <NavLink href="/v/work-orders"  label="Work Orders"  icon="📝"/>
                    <NavLink href="/v/invoices"     label="Invoices"     icon="🧾"/>
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
            leptos::spawn_local(async {
                let _ = crate::auth::revoke_session().await;
                let _ = web_sys::window().map(|w| { let _ = w.location().set_href("/login"); });
            });
        }>"Sign out"</button>
    }
}
