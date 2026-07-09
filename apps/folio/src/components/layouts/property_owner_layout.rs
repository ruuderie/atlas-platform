use leptos::prelude::*;
use leptos_router::components::Outlet;
use crate::auth::SessionInfo;
use crate::components::onboarding_banner::{OnboardingBanner, SetupStatus};

/// Persistent shell for all /po/** property-owner-lite routes.
///
/// Sidebar navigation:
///   - My Property (dashboard)
///   - Property Value (/po/value)
///   - Find a Vendor (/po/find-vendor)
///   - Settings
///
/// Upgrade banner is shown in the dashboard page itself, not in the shell,
/// so it only appears on the overview — not on every page.
#[component]
pub fn PropertyOwnerLayout() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, server_fn::error::ServerFnError>>>()
        .expect("Session context missing");

    view! {
        <div class="folio-layout folio-layout--property-owner">
            <nav class="folio-nav folio-nav--property-owner">
                <div class="nav-brand">
                    <span class="nav-logo">"Folio"</span>
                    <span class="nav-role-badge nav-role-badge--po">"Property Owner"</span>
                </div>
                <ul class="nav-links">
                    <NavLink href="/po"              label="My Property"   icon="home"/>
                    <NavLink href="/po/value"         label="Property Value" icon="show_chart"/>
                    <NavLink href="/po/find-vendor"   label="Find a Vendor"  icon="handyman"/>
                    <NavLink href="/settings"         label="Settings"       icon="settings"/>
                </ul>

                // Upgrade CTA in sidebar
                <div class="nav-upgrade-cta">
                    <p class="nav-upgrade-cta__text">"Upgrade to Landlord"</p>
                    <p class="nav-upgrade-cta__sub">"Add tenants, leases & rent collection"</p>
                    <a href="/po/upgrade" class="nav-upgrade-cta__btn" id="po-nav-upgrade-btn">
                        "Upgrade →"
                    </a>
                </div>

                <div class="nav-footer">
                    <LogoutButton/>
                </div>
            </nav>
            <main class="folio-main">
                <Suspense fallback=|| view! { <div/> }>
                    {move || session.get().map(|r| {
                        let status = match r {
                            Ok(ref info) if info.onboarding_complete && info.has_passkey => SetupStatus::Complete,
                            Ok(ref info) if info.wizard_dismissed => SetupStatus::Complete,
                            Ok(ref info) if !info.has_passkey => SetupStatus::PasskeyOnly,
                            Ok(ref info) => SetupStatus::WizardInProgress { completed: info.wizard_steps_completed, total: info.wizard_steps_total },
                            Err(_) => SetupStatus::Complete,
                        };
                        view! { <OnboardingBanner status=status /> }
                    })}
                </Suspense>
                <Outlet/>
            </main>
        </div>
    }
}

#[component]
fn NavLink(href: &'static str, label: &'static str, icon: &'static str) -> impl IntoView {
    view! {
        <li><a href=href class="nav-link">
            <span class="ms msf nav-icon">{icon}</span>
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
