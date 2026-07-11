use crate::auth::SessionInfo;
use crate::components::onboarding_banner::{OnboardingBanner, SetupStatus};
use leptos::prelude::*;
use leptos_router::components::Outlet;

/// Persistent shell for all /a/** agent routes.
/// Requires `folio_mode = "brokerage"` on the instance.
/// Shows the onboarding banner until the wizard + passkey are both complete.
#[component]
pub fn AgentLayout() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, server_fn::error::ServerFnError>>>()
        .expect("Session context missing");

    view! {
        <div class="folio-layout folio-layout--agent">
            <nav class="folio-nav folio-nav--agent">
                <div class="nav-brand">
                    <span class="nav-logo">"Folio"</span>
                    <span class="nav-role-badge nav-role-badge--agent">"Agent Portal"</span>
                </div>
                <ul class="nav-links">
                    <NavLink href="/a"           label="Dashboard"  icon="⊞"/>
                    <NavLink href="/a/clients"   label="My Clients" icon="👤"/>
                    <NavLink href="/a/listings"  label="Listings"   icon="🏠"/>
                    <NavLink href="/a/deals"     label="Deals"      icon="🤝"/>
                    <NavLink href="/a/schedule"  label="Schedule"   icon="📅"/>
                </ul>
                <div class="nav-footer">
                    <LogoutButton/>
                </div>
            </nav>
            <main class="folio-main">
                // Onboarding banner — shown until passkey + wizard complete
                <Suspense fallback=|| view! { <div/> }>
                    {move || session.get().map(|r| {
                        let status = match r {
                            Ok(ref info) if info.onboarding_complete && info.has_passkey => {
                                SetupStatus::Complete
                            }
                            Ok(ref info) if info.wizard_dismissed => SetupStatus::Complete,
                            Ok(ref info) if !info.has_passkey => SetupStatus::PasskeyOnly,
                            Ok(ref info) => SetupStatus::WizardInProgress {
                                completed: info.wizard_steps_completed,
                                total: info.wizard_steps_total,
                            },
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

/// Persistent shell for all /b/** broker routes.
/// Requires `folio_mode = "brokerage"` on the instance.
/// Shows the onboarding banner until the wizard + passkey are both complete.
#[component]
pub fn BrokerLayout() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, server_fn::error::ServerFnError>>>()
        .expect("Session context missing");

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
                // Onboarding banner — shown until passkey + wizard complete
                <Suspense fallback=|| view! { <div/> }>
                    {move || session.get().map(|r| {
                        let status = match r {
                            Ok(ref info) if info.onboarding_complete && info.has_passkey => {
                                SetupStatus::Complete
                            }
                            Ok(ref info) if info.wizard_dismissed => SetupStatus::Complete,
                            Ok(ref info) if !info.has_passkey => SetupStatus::PasskeyOnly,
                            Ok(ref info) => SetupStatus::WizardInProgress {
                                completed: info.wizard_steps_completed,
                                total: info.wizard_steps_total,
                            },
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
