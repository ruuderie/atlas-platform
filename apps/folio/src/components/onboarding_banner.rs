// apps/folio/src/components/onboarding_banner.rs
//
// OnboardingBanner — persistent top-of-page setup nudge.
//
// Shown on all authenticated Folio pages when:
//   (a) the user has not completed passkey setup, OR
//   (b) the user hasn't finished the first-run onboarding wizard
//
// The banner collapses to a slim pill when dismissed for the session
// (stored in sessionStorage, not persisted — appears again on next login
// until onboarding is genuinely complete).

use leptos::prelude::*;
use leptos_router::components::A;

// ── Status ─────────────────────────────────────────────────────────────────────

/// What the banner represents — determines copy, icon, and CTA.
#[derive(Clone, PartialEq)]
pub enum SetupStatus {
    /// User has no passkey and hasn't started the wizard.
    PasskeyOnly,
    /// Passkey is set up but wizard is incomplete. `completed` of `total` steps done.
    WizardInProgress { completed: usize, total: usize },
    /// Everything done — don't render the banner at all.
    Complete,
}

// ── Component ──────────────────────────────────────────────────────────────────

/// Slim top-of-page setup banner.
///
/// # Usage
/// ```rust,ignore
/// <OnboardingBanner status=SetupStatus::WizardInProgress { completed: 2, total: 7 } />
/// ```
#[component]
pub fn OnboardingBanner(
    status: SetupStatus,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let dismissed = RwSignal::new(false);

    // Nothing to show
    if status == SetupStatus::Complete {
        return view! { <div></div> }.into_any();
    }

    let (icon, headline, sub, cta_label, cta_href) = match &status {
        SetupStatus::PasskeyOnly => (
            "\u{1F511}",
            "Secure your account",
            "Set up a passkey for instant, password-free login.".to_string(),
            "Set up passkey \u{2192}",
            "/auth/passkey-setup",
        ),
        SetupStatus::WizardInProgress { completed, total } => (
            "\u{1F6A7}",
            "Finish setting up your workspace",
            format!(
                "{} of {} setup steps complete \u{2014} takes about 2 minutes.",
                completed, total
            ),
            "Continue setup \u{2192}",
            "/onboarding",
        ),
        SetupStatus::Complete => unreachable!(),
    };

    let progress_pct = match &status {
        SetupStatus::WizardInProgress { completed, total } => {
            Some((*completed as f32 / (*total as f32).max(1.0) * 100.0) as u32)
        }
        _ => None,
    };

    view! {
        <Show when=move || !dismissed.get()>
            <div
                id="onboarding-banner"
                class=class.unwrap_or("")
                style="position:relative;background:linear-gradient(135deg,#1e1b4b 0%,#312e81 100%);\
                       border-bottom:1px solid rgba(99,102,241,.3);padding:12px 24px;\
                       display:flex;align-items:center;gap:16px;z-index:40;"
            >
                // Icon bubble
                <div style="width:36px;height:36px;border-radius:50%;flex-shrink:0;\
                             background:rgba(99,102,241,.25);border:1px solid rgba(99,102,241,.4);\
                             display:flex;align-items:center;justify-content:center;font-size:17px;">
                    {icon}
                </div>

                // Text block
                <div style="flex:1;min-width:0;">
                    <div style="display:flex;align-items:center;gap:8px;flex-wrap:wrap;">
                        <span style="font-size:13px;font-weight:700;color:#e2e8f0;">
                            {headline}
                        </span>
                        <span style="font-size:12px;color:#94a3b8;">
                            {sub.clone()}
                        </span>
                    </div>

                    // Progress bar (wizard only)
                    {progress_pct.map(|pct| view! {
                        <div style="margin-top:6px;height:3px;background:rgba(255,255,255,.1);\
                                    border-radius:3px;overflow:hidden;max-width:260px;">
                            <div
                                style=format!(
                                    "height:100%;border-radius:3px;\
                                     background:linear-gradient(90deg,#818cf8,#a5b4fc);\
                                     width:{}%;transition:width .4s ease;",
                                    pct
                                )
                            ></div>
                        </div>
                    })}
                </div>

                // CTA
                <A
                    href=cta_href
                    attr:id="banner-cta-btn"
                    attr:style="flex-shrink:0;padding:8px 20px;border-radius:8px;\
                                background:linear-gradient(135deg,#4f46e5,#6d28d9);\
                                color:#fff;font-size:13px;font-weight:600;\
                                text-decoration:none;white-space:nowrap;\
                                box-shadow:0 4px 12px rgba(79,70,229,.35);\
                                transition:opacity .2s;"
                >
                    {cta_label}
                </A>

                // Dismiss × (session-only)
                <button
                    id="banner-dismiss-btn"
                    on:click=move |_| dismissed.set(true)
                    style="flex-shrink:0;background:none;border:none;color:#64748b;\
                           font-size:18px;cursor:pointer;padding:4px 6px;\
                           line-height:1;transition:color .15s;"
                    title="Dismiss for this session"
                >
                    "\u{00D7}"
                </button>
            </div>
        </Show>
    }.into_any()
}
