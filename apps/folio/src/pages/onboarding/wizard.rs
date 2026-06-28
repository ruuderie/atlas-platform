// apps/folio/src/pages/onboarding/wizard.rs
//
// First-Run Onboarding Wizard — /onboarding
//
// Shown to any authenticated user whose tenant onboarding is not yet complete.
// Role-aware: owner/admin sees all steps; end-users (tenant, vendor) see only
// the Welcome + passkey reminder steps.
//
// Steps (role-gated):
//   1. Welcome               — all roles
//   2. Your Profile          — all roles (name confirmation, avatar)
//   3. Jurisdiction          — owner/admin only
//   4. First Property        — owner/admin only
//   5. Payment Rails         — owner/admin only (stubbed — "coming soon")
//   6. Invite Your Team      — owner/admin only
//   7. You're Live!          — all roles (celebration)

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── API types (mirrors backend OnboardingStatusResponse) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnboardingStatus {
    pub is_ready: bool,
    pub dismissed_at: Option<String>,
}

// ── Step definitions ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
enum WizardStep {
    Welcome,
    Profile,
    Jurisdiction,
    FirstProperty,
    PaymentRails,
    InviteTeam,
    GoLive,
}

impl WizardStep {
    fn all_steps(is_owner: bool) -> Vec<WizardStep> {
        if is_owner {
            vec![
                WizardStep::Welcome,
                WizardStep::Profile,
                WizardStep::Jurisdiction,
                WizardStep::FirstProperty,
                WizardStep::PaymentRails,
                WizardStep::InviteTeam,
                WizardStep::GoLive,
            ]
        } else {
            vec![WizardStep::Welcome, WizardStep::Profile, WizardStep::GoLive]
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            WizardStep::Welcome       => "👋",
            WizardStep::Profile       => "👤",
            WizardStep::Jurisdiction  => "🌎",
            WizardStep::FirstProperty => "🏠",
            WizardStep::PaymentRails  => "💳",
            WizardStep::InviteTeam    => "👥",
            WizardStep::GoLive        => "🚀",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            WizardStep::Welcome       => "Welcome",
            WizardStep::Profile       => "Your Profile",
            WizardStep::Jurisdiction  => "Jurisdiction",
            WizardStep::FirstProperty => "First Property",
            WizardStep::PaymentRails  => "Payments",
            WizardStep::InviteTeam    => "Invite Team",
            WizardStep::GoLive        => "Go Live",
        }
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn OnboardingWizard() -> impl IntoView {
    // TODO: read from auth context — for now default to owner=true
    let is_owner = true;
    let steps    = StoredValue::new(WizardStep::all_steps(is_owner));
    let total    = steps.with_value(|s| s.len());

    let current_idx     = RwSignal::new(0usize);
    let completed_steps = RwSignal::new(std::collections::HashSet::<usize>::new());

    // Profile step state
    let profile_first = RwSignal::new(String::new());
    let profile_last  = RwSignal::new(String::new());

    // Jurisdiction step state
    let jurisdiction = RwSignal::new("US".to_string());

    // First property step state
    let prop_name    = RwSignal::new(String::new());
    let prop_address = RwSignal::new(String::new());
    let prop_city    = RwSignal::new(String::new());
    let prop_saving  = RwSignal::new(false);

    // Invite team step state
    let invite_emails = RwSignal::new(String::new());
    let invite_role   = RwSignal::new("tenant".to_string());
    let invite_sent   = RwSignal::new(false);

    let go_next = move || {
        let idx = current_idx.get();
        completed_steps.update(|s| { s.insert(idx); });
        if idx + 1 < total {
            current_idx.set(idx + 1);
        }
    };

    let go_prev = move || {
        let idx = current_idx.get();
        if idx > 0 { current_idx.set(idx - 1); }
    };

    let current_step = move || steps.with_value(|s| s.get(current_idx.get()).copied().unwrap_or(WizardStep::GoLive));
    let progress_pct = move || ((current_idx.get() as f32 / (total - 1).max(1) as f32) * 100.0) as u32;

    view! {
        <div
            id="onboarding-wizard"
            style="min-height:100vh;display:flex;background:linear-gradient(135deg,#0d1117 0%,#1a1033 100%);font-family:'Inter',sans-serif;"
        >
            // ── Left rail — step list ─────────────────────────────────────────
            <aside style="width:260px;flex-shrink:0;padding:40px 24px;border-right:1px solid rgba(255,255,255,.06);\
                           display:flex;flex-direction:column;">
                // Logo
                <div style="margin-bottom:40px;">
                    <span style="font-size:11px;font-weight:700;letter-spacing:.15em;text-transform:uppercase;color:rgba(165,180,252,.6);">
                        "Atlas · Folio"
                    </span>
                </div>

                // Steps
                <nav style="flex:1;">
                    {steps.with_value(|step_list| step_list.iter().enumerate().map(|(i, step)| {
                        let step_icon  = step.icon();
                        let step_label = step.label();
                        let is_current  = move || current_idx.get() == i;
                        let is_done     = move || completed_steps.get().contains(&i);

                        view! {
                            <div
                                style=move || format!(
                                    "display:flex;align-items:center;gap:12px;padding:10px 12px;\
                                     border-radius:10px;margin-bottom:4px;cursor:default;transition:background .15s;{}",
                                    if is_current() { "background:rgba(99,102,241,.15);" }
                                    else { "" }
                                )
                            >
                                <div
                                    style=move || format!(
                                        "width:32px;height:32px;border-radius:50%;display:flex;align-items:center;\
                                         justify-content:center;font-size:14px;flex-shrink:0;{}",
                                        if is_done() {
                                            "background:#22c55e;color:#fff;"
                                        } else if is_current() {
                                            "background:#6366f1;color:#fff;box-shadow:0 0 12px rgba(99,102,241,.5);"
                                        } else {
                                            "background:rgba(255,255,255,.06);color:#64748b;"
                                        }
                                    )
                                >
                                    {move || if is_done() { "✓".to_string() } else { step_icon.to_string() }}
                                </div>
                                <span
                                    style=move || format!("font-size:13px;font-weight:{};{}",
                                        if is_current() { "600" } else { "400" },
                                        if is_done() { "color:#22c55e;" }
                                        else if is_current() { "color:#e2e8f0;" }
                                        else { "color:#475569;" }
                                    )
                                >
                                    {step_label}
                                </span>
                            </div>
                        }
                    }).collect_view())}
                </nav>

                // Progress
                <div style="margin-top:auto;padding-top:24px;">
                    <div style="display:flex;justify-content:space-between;font-size:11px;color:#475569;margin-bottom:8px;">
                        <span>"Progress"</span>
                        <span>{move || format!("{}/{}", current_idx.get() + 1, total)}</span>
                    </div>
                    <div style="height:4px;background:rgba(255,255,255,.06);border-radius:4px;overflow:hidden;">
                        <div
                            style=move || format!(
                                "height:100%;border-radius:4px;background:linear-gradient(90deg,#6366f1,#818cf8);transition:width .3s ease;width:{}%;",
                                progress_pct()
                            )
                        ></div>
                    </div>
                </div>
            </aside>

            // ── Main content area ─────────────────────────────────────────────
            <main style="flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center;padding:40px;">
                <div style="width:100%;max-width:580px;">

                    // ── Step: Welcome ─────────────────────────────────────────
                    <Show when=move || current_step() == WizardStep::Welcome>
                        <div style="text-align:center;">
                            <div style="font-size:64px;margin-bottom:24px;line-height:1;">"👋"</div>
                            <h1 style="font-size:32px;font-weight:800;color:#f1f5f9;margin:0 0 16px;letter-spacing:-.5px;">
                                "Welcome to Folio"
                            </h1>
                            <p style="font-size:16px;color:#94a3b8;line-height:1.7;max-width:440px;margin:0 auto 40px;">
                                "Let's get your account set up in just a few steps. This should take about 3 minutes."
                            </p>
                            {if is_owner {
                                view! {
                                    <div style="display:flex;gap:16px;justify-content:center;flex-wrap:wrap;margin-bottom:48px;">
                                        {[
                                            ("🏠", "Add your first property"),
                                            ("💳", "Configure payments"),
                                            ("👥", "Invite your team"),
                                        ].map(|(icon, label)| view! {
                                            <div style="display:flex;align-items:center;gap:8px;\
                                                        background:rgba(255,255,255,.04);border:1px solid rgba(255,255,255,.08);\
                                                        border-radius:12px;padding:10px 16px;font-size:13px;color:#94a3b8;">
                                                <span>{icon}</span>
                                                <span>{label}</span>
                                            </div>
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div style="margin-bottom:48px;"></div> }.into_any()
                            }}
                            <button
                                id="ob-welcome-start"
                                on:click=move |_| go_next()
                                style="padding:16px 40px;border-radius:12px;\
                                       background:linear-gradient(135deg,#4f46e5,#7c3aed);\
                                       color:#fff;font-size:16px;font-weight:600;\
                                       border:none;cursor:pointer;\
                                       box-shadow:0 8px 24px rgba(99,102,241,.35);"
                            >
                                "Let's get started →"
                            </button>
                        </div>
                    </Show>

                    // ── Step: Profile ─────────────────────────────────────────
                    <Show when=move || current_step() == WizardStep::Profile>
                        <div>
                            <div style="font-size:48px;margin-bottom:16px;">"👤"</div>
                            <h2 style="font-size:26px;font-weight:700;color:#f1f5f9;margin:0 0 8px;">"Your Profile"</h2>
                            <p style="font-size:14px;color:#94a3b8;margin:0 0 32px;line-height:1.6;">
                                "How should people see you in the platform?"
                            </p>
                            <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-bottom:24px;">
                                <div>
                                    <label style="display:block;font-size:12px;font-weight:600;color:#94a3b8;margin-bottom:8px;text-transform:uppercase;letter-spacing:.05em;">
                                        "First Name"
                                    </label>
                                    <input
                                        id="ob-first-name"
                                        type="text"
                                        placeholder="Sarah"
                                        prop:value=profile_first
                                        on:input=move |ev| profile_first.set(event_target_value(&ev))
                                        style="width:100%;box-sizing:border-box;background:rgba(255,255,255,.05);\
                                               border:1px solid rgba(255,255,255,.1);border-radius:10px;\
                                               padding:12px 16px;color:#f1f5f9;font-size:15px;outline:none;"
                                    />
                                </div>
                                <div>
                                    <label style="display:block;font-size:12px;font-weight:600;color:#94a3b8;margin-bottom:8px;text-transform:uppercase;letter-spacing:.05em;">
                                        "Last Name"
                                    </label>
                                    <input
                                        id="ob-last-name"
                                        type="text"
                                        placeholder="Chen"
                                        prop:value=profile_last
                                        on:input=move |ev| profile_last.set(event_target_value(&ev))
                                        style="width:100%;box-sizing:border-box;background:rgba(255,255,255,.05);\
                                               border:1px solid rgba(255,255,255,.1);border-radius:10px;\
                                               padding:12px 16px;color:#f1f5f9;font-size:15px;outline:none;"
                                    />
                                </div>
                            </div>
                        </div>
                    </Show>

                    // ── Step: Jurisdiction ────────────────────────────────────
                    <Show when=move || current_step() == WizardStep::Jurisdiction>
                        <div>
                            <div style="font-size:48px;margin-bottom:16px;">"🌎"</div>
                            <h2 style="font-size:26px;font-weight:700;color:#f1f5f9;margin:0 0 8px;">"Where do you operate?"</h2>
                            <p style="font-size:14px;color:#94a3b8;margin:0 0 32px;line-height:1.6;">
                                "Your jurisdiction determines tax rules, compliance requirements, and which payment rails are available."
                            </p>
                            <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;margin-bottom:24px;">
                                {[
                                    ("US",   "🇺🇸", "United States",      "USD · ACH · Stripe"),
                                    ("BR",   "🇧🇷", "Brazil",              "BRL · PIX · Stripe"),
                                    ("USVI", "🇻🇮", "U.S. Virgin Islands", "USD · ACH"),
                                    ("DR",   "🇩🇴", "Dominican Republic",  "DOP · Wire"),
                                    ("HT",   "🇭🇹", "Haiti",               "HTG · Wire"),
                                    ("OTHER","🌐",  "Other",               "Contact support"),
                                ].map(|(code, flag, name, rails)| {
                                    let code_str = code.to_string();
                                    let code_str2 = code_str.clone();
                                    view! {
                                        <div
                                            id=format!("ob-jurisdiction-{}", code)
                                            on:click=move |_| jurisdiction.set(code_str.clone())
                                            style=move || format!(
                                                "padding:16px;border-radius:12px;cursor:pointer;\
                                                 border:2px solid {};background:{};transition:all .15s;",
                                                if jurisdiction.get() == code_str2 { "#6366f1" } else { "rgba(255,255,255,.06)" },
                                                if jurisdiction.get() == code_str2 { "rgba(99,102,241,.12)" } else { "rgba(255,255,255,.02)" }
                                            )
                                        >
                                            <div style="font-size:24px;margin-bottom:6px;">{flag}</div>
                                            <div style="font-size:14px;font-weight:600;color:#e2e8f0;">{name}</div>
                                            <div style="font-size:11px;color:#64748b;margin-top:2px;">{rails}</div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </div>
                    </Show>

                    // ── Step: First Property ──────────────────────────────────
                    <Show when=move || current_step() == WizardStep::FirstProperty>
                        <div>
                            <div style="font-size:48px;margin-bottom:16px;">"🏠"</div>
                            <h2 style="font-size:26px;font-weight:700;color:#f1f5f9;margin:0 0 8px;">"Add your first property"</h2>
                            <p style="font-size:14px;color:#94a3b8;margin:0 0 32px;line-height:1.6;">
                                "Start with just the basics — you can add full details, photos, and units later."
                            </p>
                            <div style="display:flex;flex-direction:column;gap:16px;margin-bottom:24px;">
                                <div>
                                    <label style="display:block;font-size:12px;font-weight:600;color:#94a3b8;margin-bottom:8px;text-transform:uppercase;letter-spacing:.05em;">
                                        "Property Name"
                                    </label>
                                    <input
                                        id="ob-property-name"
                                        type="text"
                                        placeholder="e.g. Ocean View Apartments"
                                        prop:value=prop_name
                                        on:input=move |ev| prop_name.set(event_target_value(&ev))
                                        style="width:100%;box-sizing:border-box;background:rgba(255,255,255,.05);\
                                               border:1px solid rgba(255,255,255,.1);border-radius:10px;\
                                               padding:12px 16px;color:#f1f5f9;font-size:15px;outline:none;"
                                    />
                                </div>
                                <div>
                                    <label style="display:block;font-size:12px;font-weight:600;color:#94a3b8;margin-bottom:8px;text-transform:uppercase;letter-spacing:.05em;">
                                        "Street Address"
                                    </label>
                                    <input
                                        id="ob-property-address"
                                        type="text"
                                        placeholder="123 Main Street"
                                        prop:value=prop_address
                                        on:input=move |ev| prop_address.set(event_target_value(&ev))
                                        style="width:100%;box-sizing:border-box;background:rgba(255,255,255,.05);\
                                               border:1px solid rgba(255,255,255,.1);border-radius:10px;\
                                               padding:12px 16px;color:#f1f5f9;font-size:15px;outline:none;"
                                    />
                                </div>
                                <div>
                                    <label style="display:block;font-size:12px;font-weight:600;color:#94a3b8;margin-bottom:8px;text-transform:uppercase;letter-spacing:.05em;">
                                        "City"
                                    </label>
                                    <input
                                        id="ob-property-city"
                                        type="text"
                                        placeholder="Miami, FL"
                                        prop:value=prop_city
                                        on:input=move |ev| prop_city.set(event_target_value(&ev))
                                        style="width:100%;box-sizing:border-box;background:rgba(255,255,255,.05);\
                                               border:1px solid rgba(255,255,255,.1);border-radius:10px;\
                                               padding:12px 16px;color:#f1f5f9;font-size:15px;outline:none;"
                                    />
                                </div>
                            </div>
                        </div>
                    </Show>

                    // ── Step: Payment Rails (stubbed) ─────────────────────────
                    <Show when=move || current_step() == WizardStep::PaymentRails>
                        <div>
                            <div style="font-size:48px;margin-bottom:16px;">"💳"</div>
                            <h2 style="font-size:26px;font-weight:700;color:#f1f5f9;margin:0 0 8px;">"Connect a payment method"</h2>
                            <p style="font-size:14px;color:#94a3b8;margin:0 0 32px;line-height:1.6;">
                                "Enable tenants to pay rent online. You can set this up now or after launch."
                            </p>
                            <div style="background:rgba(255,255,255,.03);border:1px solid rgba(255,255,255,.08);\
                                        border-radius:16px;padding:32px;text-align:center;">
                                <div style="font-size:40px;margin-bottom:16px;">"⚙️"</div>
                                <p style="font-size:15px;font-weight:600;color:#e2e8f0;margin:0 0 8px;">
                                    "Payment setup coming soon"
                                </p>
                                <p style="font-size:13px;color:#64748b;line-height:1.6;margin:0 0 20px;">
                                    "Stripe, PIX, Zelle, and wire transfer configuration will be available here. "
                                    "In the meantime, contact support to get your payment rails activated."
                                </p>
                                <a
                                    href="mailto:support@atlas.oply.co"
                                    style="display:inline-block;padding:10px 24px;border-radius:8px;\
                                           background:rgba(99,102,241,.2);border:1px solid rgba(99,102,241,.4);\
                                           color:#a5b4fc;font-size:13px;font-weight:600;text-decoration:none;"
                                >
                                    "Contact Support →"
                                </a>
                            </div>
                        </div>
                    </Show>

                    // ── Step: Invite Team ─────────────────────────────────────
                    <Show when=move || current_step() == WizardStep::InviteTeam>
                        <div>
                            <div style="font-size:48px;margin-bottom:16px;">"👥"</div>
                            <h2 style="font-size:26px;font-weight:700;color:#f1f5f9;margin:0 0 8px;">"Invite your team"</h2>
                            <p style="font-size:14px;color:#94a3b8;margin:0 0 32px;line-height:1.6;">
                                "Add property managers, staff, or your first tenant. They'll receive a magic link to set up their passkey."
                            </p>

                            <Show when=move || !invite_sent.get()>
                                <div style="display:flex;flex-direction:column;gap:16px;margin-bottom:24px;">
                                    <div>
                                        <label style="display:block;font-size:12px;font-weight:600;color:#94a3b8;margin-bottom:8px;text-transform:uppercase;letter-spacing:.05em;">
                                            "Email Addresses " <span style="color:#475569;text-transform:none;font-weight:400;">"(comma-separated)"</span>
                                        </label>
                                        <textarea
                                            id="ob-invite-emails"
                                            rows="3"
                                            placeholder="jane@company.com, john@company.com"
                                            prop:value=invite_emails
                                            on:input=move |ev| invite_emails.set(event_target_value(&ev))
                                            style="width:100%;box-sizing:border-box;background:rgba(255,255,255,.05);\
                                                   border:1px solid rgba(255,255,255,.1);border-radius:10px;\
                                                   padding:12px 16px;color:#f1f5f9;font-size:15px;outline:none;resize:vertical;"
                                        ></textarea>
                                    </div>
                                    <div>
                                        <label style="display:block;font-size:12px;font-weight:600;color:#94a3b8;margin-bottom:8px;text-transform:uppercase;letter-spacing:.05em;">
                                            "They'll join as"
                                        </label>
                                        <div style="display:flex;gap:8px;flex-wrap:wrap;">
                                            {[
                                                ("tenant",  "🏡", "Tenant"),
                                                ("landlord","🔑", "Landlord"),
                                                ("vendor",  "🔧", "Vendor"),
                                                ("pmc",     "📋", "Property Manager"),
                                            ].map(|(role, icon, label)| {
                                                let r = role.to_string();
                                                let r2 = r.clone();
                                                view! {
                                                    <button
                                                        id=format!("ob-invite-role-{}", role)
                                                        on:click=move |_| invite_role.set(r.clone())
                                                        style=move || format!(
                                                            "padding:8px 16px;border-radius:8px;font-size:13px;\
                                                             cursor:pointer;border:1px solid {};background:{};color:{};",
                                                            if invite_role.get() == r2 { "#6366f1" } else { "rgba(255,255,255,.1)" },
                                                            if invite_role.get() == r2 { "rgba(99,102,241,.2)" } else { "transparent" },
                                                            if invite_role.get() == r2 { "#a5b4fc" } else { "#94a3b8" }
                                                        )
                                                    >
                                                        {icon} " " {label}
                                                    </button>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                </div>

                                <button
                                    id="ob-send-invites-btn"
                                    on:click=move |_| {
                                        if invite_emails.get().trim().is_empty() {
                                            // Skip — go next without inviting
                                            go_next();
                                            return;
                                        }
                                        // TODO: call backend invite API for each email
                                        invite_sent.set(true);
                                    }
                                    style="padding:14px 32px;border-radius:10px;\
                                           background:linear-gradient(135deg,#4f46e5,#7c3aed);\
                                           color:#fff;font-size:15px;font-weight:600;\
                                           border:none;cursor:pointer;margin-right:12px;"
                                >
                                    "Send Invitations"
                                </button>
                            </Show>

                            <Show when=move || invite_sent.get()>
                                <div style="background:rgba(34,197,94,.08);border:1px solid rgba(34,197,94,.3);\
                                            border-radius:12px;padding:20px;text-align:center;margin-bottom:24px;">
                                    <div style="font-size:28px;margin-bottom:8px;">"✅"</div>
                                    <p style="font-size:15px;font-weight:600;color:#86efac;margin:0 0 4px;">"Invitations sent!"</p>
                                    <p style="font-size:13px;color:#4ade80;">"They'll receive a magic link to join your workspace."</p>
                                </div>
                            </Show>
                        </div>
                    </Show>

                    // ── Step: Go Live 🎉 ──────────────────────────────────────
                    <Show when=move || current_step() == WizardStep::GoLive>
                        <div style="text-align:center;">
                            <div style="font-size:72px;margin-bottom:24px;animation:bounce 1s ease infinite alternate;">"🚀"</div>
                            <h1 style="font-size:32px;font-weight:800;color:#f1f5f9;margin:0 0 16px;letter-spacing:-.5px;">
                                "You're all set!"
                            </h1>
                            <p style="font-size:16px;color:#94a3b8;line-height:1.7;max-width:420px;margin:0 auto 40px;">
                                "Your Folio workspace is ready. Head to your dashboard to start managing properties."
                            </p>
                            <a
                                id="ob-go-to-dashboard"
                                href="/dashboard"
                                style="display:inline-block;padding:16px 48px;border-radius:12px;\
                                       background:linear-gradient(135deg,#4f46e5,#7c3aed);\
                                       color:#fff;font-size:17px;font-weight:700;\
                                       text-decoration:none;\
                                       box-shadow:0 12px 32px rgba(99,102,241,.4);\
                                       letter-spacing:.2px;"
                            >
                                "Go to Dashboard →"
                            </a>
                        </div>
                    </Show>

                    // ── Navigation footer ─────────────────────────────────────
                    <Show when=move || current_step() != WizardStep::Welcome && current_step() != WizardStep::GoLive>
                        <div style="display:flex;align-items:center;justify-content:space-between;margin-top:40px;\
                                    padding-top:24px;border-top:1px solid rgba(255,255,255,.06);">
                            <button
                                on:click=move |_| go_prev()
                                style="background:rgba(255,255,255,.06);border:1px solid rgba(255,255,255,.1);\
                                       color:#94a3b8;padding:12px 24px;border-radius:10px;\
                                       font-size:14px;cursor:pointer;"
                            >
                                "← Back"
                            </button>
                            <div style="display:flex;gap:12px;">
                                <Show when=move || current_step() == WizardStep::PaymentRails || current_step() == WizardStep::InviteTeam>
                                    <button
                                        id=move || format!("ob-skip-{:?}", current_idx.get())
                                        on:click=move |_| go_next()
                                        style="background:none;border:none;color:#475569;font-size:14px;\
                                               cursor:pointer;text-decoration:underline;padding:12px;"
                                    >
                                        "Skip for now"
                                    </button>
                                </Show>
                                <button
                                    id=move || format!("ob-continue-{}", current_idx.get())
                                    on:click=move |_| go_next()
                                    style="background:linear-gradient(135deg,#4f46e5,#7c3aed);\
                                           border:none;color:#fff;padding:12px 32px;border-radius:10px;\
                                           font-size:14px;font-weight:600;cursor:pointer;\
                                           box-shadow:0 4px 16px rgba(99,102,241,.3);"
                                >
                                    {move || if current_step() == WizardStep::InviteTeam && invite_sent.get() {
                                        "Finish Setup →"
                                    } else {
                                        "Continue →"
                                    }}
                                </button>
                            </div>
                        </div>
                    </Show>

                </div>
            </main>

            <style>
                "@keyframes bounce { from { transform: translateY(0); } to { transform: translateY(-8px); } }"
            </style>
        </div>
    }
}
