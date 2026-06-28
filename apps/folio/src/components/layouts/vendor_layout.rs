use leptos::prelude::*;
use leptos_router::components::Outlet;
use crate::auth::SessionInfo;
use crate::components::onboarding_banner::{OnboardingBanner, SetupStatus};

/// Persistent shell for all /v/** vendor routes.
#[component]
pub fn VendorLayout() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, server_fn::error::ServerFnError>>>()
        .expect("Session context missing");

    view! {
        <div class="folio-layout folio-layout--vendor">
            <nav class="folio-nav folio-nav--vendor">
                <div class="nav-brand">
                    <span class="nav-logo">"Folio"</span>
                    <span class="nav-role-badge nav-role-badge--vendor">"Vendor Portal"</span>
                </div>
                <ul class="nav-links">
                    <NavLink href="/v"             label="Dashboard"   icon="\u{229E}"/>
                    <NavLink href="/v/work-orders" label="Work Orders" icon="\u{1F4DD}"/>
                    <NavLink href="/v/invoices"    label="Invoices"    icon="\u{1F9FE}"/>
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
