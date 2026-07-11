use crate::auth::SessionInfo;
use crate::components::onboarding_banner::{OnboardingBanner, SetupStatus};
use leptos::prelude::*;
use leptos_router::components::Outlet;

/// Persistent shell for all /pmc/** property management company routes.
#[component]
pub fn PmcLayout() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, server_fn::error::ServerFnError>>>()
        .expect("Session context missing");

    view! {
        <div class="folio-layout folio-layout--pmc">
            <nav class="folio-nav folio-nav--pmc">
                <div class="nav-brand">
                    <span class="nav-logo">"Folio"</span>
                    <span class="nav-role-badge nav-role-badge--pmc">"PMC Dashboard"</span>
                </div>
                <ul class="nav-links">
                    <NavLink href="/pmc"         label="Overview"    icon="\u{229E}"/>
                    <NavLink href="/pmc/clients" label="Client Book" icon="\u{1F3E2}"/>
                </ul>
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
