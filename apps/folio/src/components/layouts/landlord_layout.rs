use crate::auth::SessionInfo;
use crate::components::global_search::GlobalSearch;
use crate::components::nav::{SidebarNav, HIRED_PM_LANDLORD_NAV, LANDLORD_NAV};
use crate::components::onboarding_banner::{OnboardingBanner, SetupStatus};
use crate::components::scorecard_nudge_host::ScorecardNudgeHost;
use leptos::prelude::*;
use leptos_router::components::Outlet;

/// Persistent shell for all /l/** landlord routes.
/// Nav items are driven by `LANDLORD_NAV` / `HIRED_PM_LANDLORD_NAV` in `components/nav.rs`.
#[component]
pub fn LandlordLayout() -> impl IntoView {
    let session = use_context::<Resource<Result<SessionInfo, server_fn::error::ServerFnError>>>()
        .expect("Session context missing");

    view! {
        <div class="folio-layout folio-layout--landlord">
            <Suspense fallback=|| view! { <div class="sidebar-loading"/> }>
                {move || session.get().map(|r| {
                    let (name, initials, hired) = match r {
                        Ok(ref info) => (
                            info.display_name.clone(),
                            info.display_name.as_deref().map(user_initials),
                            info.is_hired_pm,
                        ),
                        Err(_) => (None, None, false),
                    };
                    let nav = if hired {
                        &HIRED_PM_LANDLORD_NAV
                    } else {
                        &LANDLORD_NAV
                    };
                    view! {
                        <SidebarNav
                            config=nav
                            user_name=name.unwrap_or_default()
                            initials=initials.unwrap_or_default()
                        />
                    }
                })}
            </Suspense>
            <main class="folio-main">
                <div class="folio-main__toolbar">
                    <GlobalSearch/>
                </div>
                <Suspense fallback=|| view! { <div/> }>
                    {move || session.get().map(|r| {
                        match r {
                            Ok(ref info) if info.is_hired_pm => {
                                let label = match &info.employer_display_name {
                                    Some(n) if !n.is_empty() => format!("Managing for {n}"),
                                    _ => "Managing for your landlord".into(),
                                };
                                view! {
                                    <div
                                        class="folio-hired-banner"
                                        role="status"
                                        style="margin:0 1rem 0.75rem;padding:0.5rem 0.75rem;border-radius:8px;background:#e0f2fe;color:#0c4a6e;font-size:0.875rem;font-weight:600;"
                                    >
                                        {label}
                                    </div>
                                }.into_any()
                            }
                            _ => view! { <div/> }.into_any(),
                        }
                    })}
                </Suspense>
                // Onboarding banner — shown until passkey + wizard complete
                <Suspense fallback=|| view! { <div/> }>
                    {move || session.get().map(|r| {
                        let status = match r {
                            Ok(ref info) if info.is_hired_pm => SetupStatus::Complete,
                            Ok(ref info) if info.onboarding_complete && info.has_passkey => {
                                SetupStatus::Complete
                            }
                            Ok(ref info) if info.wizard_dismissed => SetupStatus::Complete,
                            Ok(ref info) if !info.has_passkey => SetupStatus::PasskeyOnly,
                            Ok(ref info) => SetupStatus::WizardInProgress {
                                completed: info.wizard_steps_completed,
                                total: info.wizard_steps_total,
                            },
                            Err(_) => SetupStatus::Complete, // don't block on auth errors
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
