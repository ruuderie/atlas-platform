use crate::auth::SessionInfo;
use crate::components::nav::{SidebarNav, TENANT_NAV};
use crate::components::onboarding_banner::{OnboardingBanner, SetupStatus};
use crate::components::scorecard_nudge_host::ScorecardNudgeHost;
use leptos::prelude::*;
use leptos_router::components::Outlet;

/// Persistent shell for all /t/** tenant routes.
/// Nav items driven by `TENANT_NAV` in `components/nav.rs`.
#[component]
pub fn TenantLayout() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, server_fn::error::ServerFnError>>>()
        .expect("Session context missing");

    view! {
        <div class="folio-layout folio-layout--tenant">
            <Suspense fallback=|| view! { <div class="sidebar-loading"/> }>
                {move || session.get().map(|r| {
                    let (name, initials) = match r {
                        Ok(ref info) => (
                            info.display_name.clone(),
                            info.display_name.as_deref().map(user_initials),
                        ),
                        Err(_) => (None, None),
                    };
                    view! {
                        <SidebarNav config=&TENANT_NAV user_name=name.unwrap_or_default() initials=initials.unwrap_or_default()/>
                    }
                })}
            </Suspense>
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
                <ScorecardNudgeHost/>
            </main>
        </div>
    }
}

fn user_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}
