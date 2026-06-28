// apps/folio/src/pages/onboarding/wizard.rs
//
// First-Run Onboarding Wizard — /onboarding
//
// Shown to any authenticated user whose tenant onboarding is not yet complete.
// Role-aware: owner/admin sees all steps; end-users see Welcome + Profile + GoLive.
//
// Visual identity: matches Atlas/Folio stitch design system —
//   surface #f7f9fb, near-black #191c1e primary, Inter font,
//   white rounded-2xl cards, top pill-step progress bar, fixed bottom nav.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnboardingStatus {
    pub is_ready: bool,
    pub dismissed_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingSubmitResponse {
    pub portfolio_id: Option<String>,
    pub asset_id: Option<String>,
    pub applied: Vec<String>,
}

// ── Server function ───────────────────────────────────────────────────────────

/// POST /api/folio/onboarding/submit via Leptos server function.
/// SSR-only: runs on the Axum server thread, forwards the session cookie.
#[server(SubmitOnboarding, "/api")]
pub async fn submit_onboarding(
    first_name: String,
    last_name: String,
    jurisdiction_code: String,
    property_name: String,
    property_address: String,
    property_city: String,
    property_type: String,
) -> Result<OnboardingSubmitResponse, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let payload = serde_json::json!({
        "first_name":        first_name,
        "last_name":         last_name,
        "jurisdiction_code": jurisdiction_code,
        "property_name":     property_name,
        "property_address":  property_address,
        "property_city":     property_city,
        "property_type":     property_type,
    });

    crate::atlas_client::authenticated_post::<_, OnboardingSubmitResponse>(
        "/api/folio/onboarding/submit",
        &token,
        None,
        &payload,
    )
    .await
    .map_err(server_fn::error::ServerFnError::new)
}

#[cfg(feature = "ssr")]
fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ").map(str::to_string))
        .or_else(|| {
            headers
                .get(axum::http::header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .and_then(|c| {
                    c.split(';').find_map(|part| {
                        part.trim()
                            .strip_prefix("atlas_session=")
                            .map(str::to_string)
                    })
                })
        })
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

    /// True for steps that should show a "Skip for now" link.
    fn is_skippable(&self) -> bool {
        matches!(self, WizardStep::PaymentRails | WizardStep::InviteTeam)
    }

    /// True for steps that need a backend call before advancing.
    fn needs_save(&self) -> bool {
        matches!(
            self,
            WizardStep::Profile | WizardStep::Jurisdiction | WizardStep::FirstProperty
        )
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn OnboardingWizard() -> impl IntoView {
    // TODO: resolve from SessionInfo context — defaulting to owner for now
    let is_owner = true;
    let steps = StoredValue::new(WizardStep::all_steps(is_owner));
    let total = steps.with_value(|s| s.len());
    // Steps shown in the pill bar = all except Welcome (step 0) and GoLive (last)
    let pill_count = total.saturating_sub(2); // interior steps only

    let current_idx = RwSignal::new(0usize);
    let completed_steps = RwSignal::new(std::collections::HashSet::<usize>::new());

    // ── Form state ────────────────────────────────────────────────────────────
    let profile_first = RwSignal::new(String::new());
    let profile_last  = RwSignal::new(String::new());

    let jurisdiction  = RwSignal::new("US".to_string());

    let prop_name     = RwSignal::new(String::new());
    let prop_address  = RwSignal::new(String::new());
    let prop_city     = RwSignal::new(String::new());
    let prop_type     = RwSignal::new("single_family".to_string());

    let invite_emails = RwSignal::new(String::new());
    let invite_role   = RwSignal::new("tenant".to_string());
    let invite_sent   = RwSignal::new(false);

    // ── API state ─────────────────────────────────────────────────────────────
    let saving: RwSignal<bool>         = RwSignal::new(false);
    let save_error: RwSignal<Option<String>> = RwSignal::new(None);

    // ── Navigation ────────────────────────────────────────────────────────────
    let go_next = move || {
        let idx = current_idx.get();
        completed_steps.update(|s| { s.insert(idx); });
        if idx + 1 < total {
            current_idx.set(idx + 1);
        }
    };

    let go_next_with_save = {
        let pf = profile_first;
        let pl = profile_last;
        let jc = jurisdiction;
        let pn = prop_name;
        let pa = prop_address;
        let pc = prop_city;
        let pt = prop_type;

        move || {
            let step = steps.with_value(|s| {
                s.get(current_idx.get()).copied().unwrap_or(WizardStep::GoLive)
            });

            if !step.needs_save() {
                go_next();
                return;
            }

            saving.set(true);
            save_error.set(None);

            let first = pf.get();
            let last  = pl.get();
            let jcode = jc.get();
            let pname = pn.get();
            let paddr = pa.get();
            let pcity = pc.get();
            let ptype = pt.get();

            leptos::task::spawn_local(async move {
                match submit_onboarding(first, last, jcode, pname, paddr, pcity, ptype).await {
                    Ok(_)  => { saving.set(false); go_next(); }
                    Err(e) => { saving.set(false); save_error.set(Some(e.to_string())); }
                }
            });
        }
    };

    let go_prev = move || {
        let idx = current_idx.get();
        if idx > 0 { current_idx.set(idx - 1); }
    };

    let current_step = move || {
        steps.with_value(|s| s.get(current_idx.get()).copied().unwrap_or(WizardStep::GoLive))
    };

    // Pill index: 0-based among interior steps (skip Welcome=idx0, GoLive=last)
    // pill_idx = current_idx - 1  (0 = Profile, 1 = Jurisdiction, ...)
    let pill_active = move || current_idx.get().saturating_sub(1);
    let pill_progress_pct = move || {
        let idx = current_idx.get();
        ((idx as f32 / (total - 1).max(1) as f32) * 100.0) as u32
    };

    view! {
        // ── Global styles ───────────────────────────────────────────────────
        <style>
            "@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap');
            @import url('https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200&display=swap');
            #ob-wizard * { box-sizing: border-box; }
            #ob-wizard { font-family: 'Inter', sans-serif; }
            .ob-fi {
                background: #f2f4f6;
                border: none;
                border-radius: 8px;
                padding: 11px 16px;
                font-size: 14px;
                width: 100%;
                outline: none;
                transition: box-shadow 0.15s;
                font-family: 'Inter', sans-serif;
                color: #191c1e;
            }
            .ob-fi:focus { box-shadow: 0 0 0 2px #191c1e; }
            .ob-fi::placeholder { color: #45464d; opacity: 0.45; }
            .ob-label {
                display: block;
                font-size: 10px;
                font-weight: 700;
                text-transform: uppercase;
                letter-spacing: 0.08em;
                color: #45464d;
                margin-bottom: 6px;
            }
            .ob-type-opt {
                border: 1.5px solid #e6e8ea;
                border-radius: 12px;
                padding: 12px 16px;
                cursor: pointer;
                font-size: 13px;
                font-weight: 600;
                text-align: center;
                transition: border-color 0.15s, background 0.15s;
                color: #45464d;
                background: #fff;
            }
            .ob-type-opt:hover { border-color: #191c1e; color: #191c1e; }
            .ob-type-opt.selected { border-color: #191c1e; background: #f2f4f6; color: #191c1e; }
            .ob-pill { display:flex; align-items:center; justify-content:center; width:24px; height:24px; border-radius:50%; font-size:10px; font-weight:700; transition:all 0.2s; flex-shrink:0; }
            .ob-pill.done    { background:#069669; color:#fff; }
            .ob-pill.current { background:#191c1e; color:#fff; box-shadow:0 0 0 3px rgba(25,28,30,0.12); }
            .ob-pill.pending { background:#e0e3e5; color:#45464d; }
            .ob-card { background:#fff; border-radius:16px; padding:24px; box-shadow:0 2px 8px rgba(25,28,30,0.06); }
            .ob-step-label { font-size:9px; font-weight:700; text-transform:uppercase; letter-spacing:0.08em; color:#45464d; margin-bottom:4px; }
            @keyframes ob-slide { from { opacity:0; transform:translateX(8px); } to { opacity:1; transform:translateX(0); } }
            .ob-step-anim { animation: ob-slide 0.2s ease; }
            .ms { font-family:'Material Symbols Outlined'; font-variation-settings:'FILL' 0,'wght' 400,'GRAD' 0,'opsz' 24; line-height:1; }"
        </style>

        <div
            id="ob-wizard"
            style="min-height:100vh; background:#f2f4f6; color:#191c1e; padding-bottom:80px;"
        >
            // ── Header ─────────────────────────────────────────────────────
            <header style="background:#fff; border-bottom:1px solid #e6e8ea; position:sticky; top:0; z-index:40;">
                <div style="max-width:680px; margin:0 auto; padding:0 24px; height:56px; display:flex; align-items:center; justify-content:space-between; gap:16px;">
                    <div style="display:flex; align-items:center; gap:8px;">
                        <div style="width:28px; height:28px; background:#191c1e; border-radius:8px; display:flex; align-items:center; justify-content:center;">
                            <span style="color:#fff; font-size:14px;">"\u{1F3E0}"</span>
                        </div>
                        <span style="font-weight:700; font-size:14px; color:#191c1e; letter-spacing:-0.3px;">"Folio"</span>
                        <span style="color:#c6c6cd;">"/"</span>
                        <span style="font-size:12px; font-weight:600; color:#45464d;">"Workspace Setup"</span>
                    </div>
                    <Show when=move || current_step() != WizardStep::GoLive>
                        <a
                            href="/dashboard"
                            style="display:flex; align-items:center; gap:4px; font-size:12px; color:#45464d; text-decoration:none; transition:color 0.15s;"
                        >
                            <span class="ms" style="font-size:16px;">"close"</span>
                            "Exit"
                        </a>
                    </Show>
                </div>
            </header>

            // ── Pill progress bar (hidden on Welcome and GoLive) ────────────
            <Show when=move || {
                let s = current_step();
                s != WizardStep::Welcome && s != WizardStep::GoLive
            }>
                <div style="background:#fff; border-bottom:1px solid #e6e8ea;">
                    <div style="max-width:680px; margin:0 auto; padding:12px 24px;">
                        // Pill row
                        <div style="display:flex; align-items:center; justify-content:center; gap:0; margin-bottom:10px;">
                            {steps.with_value(|step_list| {
                                step_list.iter().enumerate()
                                    .filter(|(_, s)| **s != WizardStep::Welcome && **s != WizardStep::GoLive)
                                    .enumerate()
                                    .map(|(pill_i, (real_i, step))| {
                                        let label = step.label();
                                        let is_done    = move || completed_steps.get().contains(&real_i);
                                        let is_current = move || current_idx.get() == real_i;
                                        let pill_class = move || {
                                            if is_done() { "ob-pill done" }
                                            else if is_current() { "ob-pill current" }
                                            else { "ob-pill pending" }
                                        };

                                        view! {
                                            <div style="display:flex; flex-direction:column; align-items:center; gap:4px;">
                                                <div style="display:flex; align-items:center;">
                                                    {if pill_i > 0 {
                                                        view! {
                                                            <div style="width:40px; height:1px; background:#c6c6cd; flex-shrink:0;"></div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }}
                                                    <div class=pill_class>
                                                        {move || if is_done() { "\u{2713}".to_string() }
                                                         else { (pill_i + 1).to_string() }}
                                                    </div>
                                                    {if pill_i < pill_count.saturating_sub(1) {
                                                        view! {
                                                            <div style="width:40px; height:1px; background:#c6c6cd; flex-shrink:0;"></div>
                                                        }.into_any()
                                                    } else {
                                                        view! { <span></span> }.into_any()
                                                    }}
                                                </div>
                                                <span
                                                    style=move || format!(
                                                        "font-size:9px; font-weight:{}; color:{}; white-space:nowrap;",
                                                        if is_current() { "700" } else { "600" },
                                                        if is_done() { "#069669" }
                                                        else if is_current() { "#191c1e" }
                                                        else { "#45464d" }
                                                    )
                                                >{label}</span>
                                            </div>
                                        }
                                    })
                                    .collect_view()
                            })}
                        </div>
                        // Thin progress bar
                        <div style="height:3px; background:#e0e3e5; border-radius:4px; overflow:hidden;">
                            <div style=move || format!(
                                "height:100%; background:#191c1e; border-radius:4px; transition:width 0.35s ease; width:{}%;",
                                pill_progress_pct()
                            )></div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Main content ────────────────────────────────────────────────
            <main style="max-width:680px; margin:0 auto; padding:40px 24px 120px;">

                // Error banner
                <Show when=move || save_error.get().is_some()>
                    <div style="background:#ffdad6; border:1px solid rgba(186,26,26,0.3); border-radius:10px; padding:12px 16px; margin-bottom:24px; font-size:13px; color:#93000a; display:flex; align-items:center; gap:8px;">
                        <span style="font-size:16px;">"&#x26A0;"</span>
                        <span>{move || save_error.get().unwrap_or_default()}</span>
                    </div>
                </Show>

                // ── STEP: Welcome ───────────────────────────────────────────
                <Show when=move || current_step() == WizardStep::Welcome>
                    <div class="ob-step-anim" style="text-align:center; padding:40px 0;">
                        <div style="font-size:64px; margin-bottom:24px; line-height:1;">"&#x1F44B;"</div>
                        <h1 style="font-size:28px; font-weight:800; color:#191c1e; margin:0 0 12px; letter-spacing:-0.5px;">"Welcome to Folio"</h1>
                        <p style="font-size:15px; color:#45464d; margin:0 auto 40px; max-width:420px; line-height:1.7;">
                            "Let\u{2019}s get your workspace ready. It takes about 3 minutes and you can always come back to finish later."
                        </p>
                        <button
                            id="ob-welcome-start"
                            on:click=move |_| go_next()
                            style="display:inline-flex; align-items:center; gap:8px; padding:14px 40px; border-radius:12px; border:none; background:linear-gradient(135deg,#000 0%,#131b2e 100%); color:#fff; font-size:15px; font-weight:700; cursor:pointer; box-shadow:0 4px 16px rgba(25,28,30,0.2);"
                        >
                            "Get started"
                            <span class="ms" style="font-size:20px;">"arrow_forward"</span>
                        </button>
                    </div>
                </Show>

                // ── STEP: Profile ───────────────────────────────────────────
                <Show when=move || current_step() == WizardStep::Profile>
                    <div class="ob-step-anim">
                        <p class="ob-step-label">"Step "{move || current_idx.get()}" of "{total - 2}</p>
                        <h1 style="font-size:20px; font-weight:700; color:#191c1e; margin:0 0 4px;">"Your Profile"</h1>
                        <p style="font-size:14px; color:#45464d; margin:0 0 24px;">"How should people see you in the platform?"</p>

                        <div class="ob-card" style="display:flex; flex-direction:column; gap:20px;">
                            <div style="display:grid; grid-template-columns:1fr 1fr; gap:16px;">
                                <div>
                                    <label class="ob-label" for="ob-first-name">"First Name"</label>
                                    <input
                                        id="ob-first-name"
                                        class="ob-fi"
                                        type="text"
                                        placeholder="Sarah"
                                        prop:value=profile_first
                                        on:input=move |ev| profile_first.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label class="ob-label" for="ob-last-name">"Last Name"</label>
                                    <input
                                        id="ob-last-name"
                                        class="ob-fi"
                                        type="text"
                                        placeholder="Chen"
                                        prop:value=profile_last
                                        on:input=move |ev| profile_last.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                        </div>
                    </div>
                </Show>

                // ── STEP: Jurisdiction ──────────────────────────────────────
                <Show when=move || current_step() == WizardStep::Jurisdiction>
                    <div class="ob-step-anim">
                        <p class="ob-step-label">"Step "{move || current_idx.get()}" of "{total - 2}</p>
                        <h1 style="font-size:20px; font-weight:700; color:#191c1e; margin:0 0 4px;">"Operating Jurisdiction"</h1>
                        <p style="font-size:14px; color:#45464d; margin:0 0 24px;">"Where are your properties located? This sets tax, compliance, and payment rails."</p>

                        <div class="ob-card" style="display:flex; flex-direction:column; gap:12px;">
                            {[
                                ("US",   "\u{1F1FA}\u{1F1F8}", "United States",        "USD \u{00B7} English"),
                                ("BR",   "\u{1F1E7}\u{1F1F7}", "Brazil",               "BRL \u{00B7} Portugu\u{00EA}s"),
                                ("DR",   "\u{1F1E9}\u{1F1F4}", "Dominican Republic",   "DOP \u{00B7} Espa\u{00F1}ol"),
                                ("HT",   "\u{1F1ED}\u{1F1F9}", "Haiti",                "HTG \u{00B7} Krey\u{00F2}l"),
                                ("USVI", "\u{1F1FB}\u{1F1EE}", "U.S. Virgin Islands",  "USD \u{00B7} English"),
                            ].map(|(code, flag, name, detail)| {
                                let c_click = code.to_string();
                                let c2      = c_click.clone();
                                let c3      = c_click.clone();
                                let c4      = c_click.clone();
                                view! {
                                    <div
                                        class=move || if jurisdiction.get() == c2 { "ob-type-opt selected" } else { "ob-type-opt" }
                                        on:click={let cc = c_click.clone(); move |_| jurisdiction.set(cc.clone())}
                                        style="display:flex; align-items:center; gap:12px; text-align:left; padding:14px 16px;"
                                    >
                                        <span style="font-size:24px;">{flag}</span>
                                        <div>
                                            <div style="font-size:13px; font-weight:600; color:#191c1e;">{name}</div>
                                            <div style="font-size:11px; color:#45464d; margin-top:1px;">{detail}</div>
                                        </div>
                                        <span
                                            style=move || format!(
                                                "margin-left:auto; width:18px; height:18px; border-radius:50%; border:2px solid {}; background:{}; flex-shrink:0;",
                                                if jurisdiction.get() == c3 { "#191c1e" } else { "#c6c6cd" },
                                                if jurisdiction.get() == c4 { "#191c1e" } else { "transparent" },
                                            )
                                        ></span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </Show>

                // ── STEP: First Property ────────────────────────────────────
                <Show when=move || current_step() == WizardStep::FirstProperty>
                    <div class="ob-step-anim">
                        <p class="ob-step-label">"Step "{move || current_idx.get()}" of "{total - 2}</p>
                        <h1 style="font-size:20px; font-weight:700; color:#191c1e; margin:0 0 4px;">"Add Your First Property"</h1>
                        <p style="font-size:14px; color:#45464d; margin:0 0 24px;">"Register your first property to start managing leases, maintenance, and payments."</p>

                        <div class="ob-card" style="display:flex; flex-direction:column; gap:20px;">
                            <div>
                                <label class="ob-label" for="ob-prop-name">"Property Name"</label>
                                <input
                                    id="ob-prop-name"
                                    class="ob-fi"
                                    type="text"
                                    placeholder="Ocean View Residences"
                                    prop:value=prop_name
                                    on:input=move |ev| prop_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="ob-label" for="ob-prop-address">"Street Address"</label>
                                <input
                                    id="ob-prop-address"
                                    class="ob-fi"
                                    type="text"
                                    placeholder="123 Main St"
                                    prop:value=prop_address
                                    on:input=move |ev| prop_address.set(event_target_value(&ev))
                                />
                            </div>
                            <div style="display:grid; grid-template-columns:1fr 1fr; gap:16px;">
                                <div>
                                    <label class="ob-label" for="ob-prop-city">"City"</label>
                                    <input
                                        id="ob-prop-city"
                                        class="ob-fi"
                                        type="text"
                                        placeholder="Miami"
                                        prop:value=prop_city
                                        on:input=move |ev| prop_city.set(event_target_value(&ev))
                                    />
                                </div>
                                <div>
                                    <label class="ob-label" for="ob-prop-type">"Property Type"</label>
                                    <select
                                        id="ob-prop-type"
                                        class="ob-fi"
                                        on:change=move |ev| prop_type.set(event_target_value(&ev))
                                    >
                                        <option value="single_family">"Single Family"</option>
                                        <option value="multi_family">"Multi-Family"</option>
                                        <option value="condo">"Condominium"</option>
                                        <option value="townhouse">"Townhouse"</option>
                                        <option value="str">"Short-Term Rental"</option>
                                        <option value="commercial">"Commercial"</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                    </div>
                </Show>

                // ── STEP: Payment Rails ─────────────────────────────────────
                <Show when=move || current_step() == WizardStep::PaymentRails>
                    <div class="ob-step-anim">
                        <p class="ob-step-label">"Step "{move || current_idx.get()}" of "{total - 2}</p>
                        <h1 style="font-size:20px; font-weight:700; color:#191c1e; margin:0 0 4px;">"Payment Collection"</h1>
                        <p style="font-size:14px; color:#45464d; margin:0 0 24px;">"Configure how tenants pay rent. You can set this up later."</p>

                        <div class="ob-card" style="display:flex; flex-direction:column; gap:16px;">
                            {[
                                ("stripe",  "\u{1F4B3}", "Stripe (Credit/Debit/ACH)",  "US, USVI"),
                                ("pix",     "\u{1F4F1}", "PIX",                         "Brazil"),
                                ("bitcoin", "\u{20BF}",  "Bitcoin",                     "All jurisdictions"),
                                ("zelle",   "\u{1F4B8}", "Zelle",                       "US only"),
                            ].map(|(id, icon, name, note)| {
                                view! {
                                    <div style="display:flex; align-items:center; gap:12px; padding:14px 16px; border:1.5px solid #e6e8ea; border-radius:12px; opacity:0.6;">
                                        <span style="font-size:20px;">{icon}</span>
                                        <div style="flex:1;">
                                            <div style="font-size:13px; font-weight:600; color:#191c1e;">{name}</div>
                                            <div style="font-size:11px; color:#45464d; margin-top:1px;">{note}</div>
                                        </div>
                                        <span style="font-size:10px; font-weight:700; text-transform:uppercase; letter-spacing:0.05em; padding:2px 8px; border-radius:4px; background:#e0e3e5; color:#45464d;">"Coming Soon"</span>
                                    </div>
                                }
                            }).collect_view()}

                            <div style="padding:14px 16px; background:#f2f4f6; border-radius:12px; font-size:13px; color:#45464d; display:flex; align-items:center; gap:8px;">
                                <span class="ms" style="font-size:18px; color:#45464d;">"info"</span>
                                "Payment configuration will be available soon. Skip for now — your data is saved."
                            </div>
                        </div>
                    </div>
                </Show>

                // ── STEP: Invite Team ───────────────────────────────────────
                <Show when=move || current_step() == WizardStep::InviteTeam>
                    <div class="ob-step-anim">
                        <p class="ob-step-label">"Step "{move || current_idx.get()}" of "{total - 2}</p>
                        <h1 style="font-size:20px; font-weight:700; color:#191c1e; margin:0 0 4px;">"Invite Your Team"</h1>
                        <p style="font-size:14px; color:#45464d; margin:0 0 24px;">"Add property managers, staff, or your first tenant. They\u{2019}ll receive a magic link \u{2014} no password required."</p>

                        <Show when=move || !invite_sent.get()>
                            <div class="ob-card" style="display:flex; flex-direction:column; gap:20px;">
                                <div>
                                    <label class="ob-label" for="ob-invite-emails">
                                        "Email Addresses "
                                        <span style="font-weight:400; text-transform:none; letter-spacing:normal; opacity:0.6;">"(comma-separated)"</span>
                                    </label>
                                    <textarea
                                        id="ob-invite-emails"
                                        class="ob-fi"
                                        rows="3"
                                        placeholder="jane@company.com, john@company.com"
                                        prop:value=invite_emails
                                        on:input=move |ev| invite_emails.set(event_target_value(&ev))
                                        style="resize:vertical;"
                                    ></textarea>
                                </div>
                                <div>
                                    <label class="ob-label">"They\u{2019}ll join as"</label>
                                    <div style="display:grid; grid-template-columns:repeat(4, 1fr); gap:8px;">
                                        {[
                                            ("tenant",  "\u{1F3E1}", "Tenant"),
                                            ("landlord","\u{1F511}", "Landlord"),
                                            ("vendor",  "\u{1F527}", "Vendor"),
                                            ("pmc",     "\u{1F4CB}", "PM"),
                                        ].map(|(role, icon, label)| {
                                            let r_click = role.to_string();
                                            let r2      = r_click.clone();
                                            view! {
                                                <button
                                                    id=format!("ob-role-{}", role)
                                                    class=move || if invite_role.get() == r2 { "ob-type-opt selected" } else { "ob-type-opt" }
                                                    on:click={let rc = r_click.clone(); move |_| invite_role.set(rc.clone())}
                                                    style="padding:10px 8px; display:flex; flex-direction:column; align-items:center; gap:4px;"
                                                >
                                                    <span style="font-size:18px;">{icon}</span>
                                                    <span style="font-size:11px;">{label}</span>
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                                <button
                                    id="ob-send-invites"
                                    on:click=move |_| {
                                        // TODO: call backend invite API
                                        invite_sent.set(true);
                                    }
                                    style="width:100%; padding:14px; border-radius:12px; border:none; background:linear-gradient(135deg,#000 0%,#131b2e 100%); color:#fff; font-size:14px; font-weight:700; cursor:pointer; display:flex; align-items:center; justify-content:center; gap:8px;"
                                >
                                    <span class="ms" style="font-size:20px; font-variation-settings:'FILL' 1;">"send"</span>
                                    "Send Invitations"
                                </button>
                            </div>
                        </Show>

                        <Show when=move || invite_sent.get()>
                            <div style="background:#f0fdf4; border:1px solid #bbf7d0; border-radius:16px; padding:24px; text-align:center;">
                                <div style="width:56px; height:56px; background:rgba(6,150,105,0.1); border-radius:50%; display:flex; align-items:center; justify-content:center; margin:0 auto 12px;">
                                    <span class="ms" style="font-size:32px; color:#069669; font-variation-settings:'FILL' 1;">"check_circle"</span>
                                </div>
                                <p style="font-size:15px; font-weight:700; color:#191c1e; margin:0 0 4px;">"Invitations sent!"</p>
                                <p style="font-size:13px; color:#45464d;">"They\u{2019}ll receive a magic link to join your workspace."</p>
                            </div>
                        </Show>
                    </div>
                </Show>

                // ── STEP: Go Live ───────────────────────────────────────────
                <Show when=move || current_step() == WizardStep::GoLive>
                    <div class="ob-step-anim" style="text-align:center; padding:40px 0;">
                        <div style="width:96px; height:96px; background:rgba(6,150,105,0.1); border-radius:50%; display:flex; align-items:center; justify-content:center; margin:0 auto 24px;">
                            <span class="ms" style="font-size:52px; color:#069669; font-variation-settings:'FILL' 1;">"verified_user"</span>
                        </div>
                        <h1 style="font-size:28px; font-weight:800; color:#191c1e; margin:0 0 12px; letter-spacing:-0.5px;">"You\u{2019}re all set!"</h1>
                        <p style="font-size:15px; color:#45464d; margin:0 auto 40px; max-width:400px; line-height:1.7;">
                            "Your Folio workspace is ready. Head to your dashboard to start managing properties."
                        </p>
                        <a
                            id="ob-go-to-dashboard"
                            href="/dashboard"
                            style="display:inline-flex; align-items:center; gap:8px; padding:16px 48px; border-radius:12px; background:linear-gradient(135deg,#000 0%,#131b2e 100%); color:#fff; font-size:16px; font-weight:700; text-decoration:none; box-shadow:0 4px 16px rgba(25,28,30,0.2);"
                        >
                            "Go to Dashboard"
                            <span class="ms" style="font-size:20px;">"arrow_forward"</span>
                        </a>
                    </div>
                </Show>

            </main>

            // ── Fixed bottom nav ────────────────────────────────────────────
            <Show when=move || {
                let s = current_step();
                s != WizardStep::Welcome && s != WizardStep::GoLive
            }>
                <div style="position:fixed; bottom:0; left:0; right:0; background:#fff; border-top:1px solid #e6e8ea; z-index:40;">
                    <div style="max-width:680px; margin:0 auto; padding:12px 24px; display:flex; align-items:center; justify-content:space-between;">
                        <button
                            id="ob-btn-back"
                            on:click=move |_| go_prev()
                            style="display:flex; align-items:center; gap:6px; padding:10px 20px; border-radius:10px; border:1.5px solid #c6c6cd; background:#fff; font-size:14px; font-weight:600; color:#191c1e; cursor:pointer;"
                        >
                            <span class="ms" style="font-size:18px;">"arrow_back"</span>
                            "Back"
                        </button>

                        <div style="display:flex; align-items:center; gap:12px;">
                            <Show when=move || current_step().is_skippable()>
                                <button
                                    id=move || format!("ob-skip-{}", current_idx.get())
                                    on:click=move |_| go_next()
                                    style="font-size:13px; font-weight:600; color:#45464d; background:none; border:none; cursor:pointer; text-decoration:underline; padding:10px;"
                                >
                                    "Skip for now"
                                </button>
                            </Show>

                            <button
                                id=move || format!("ob-continue-{}", current_idx.get())
                                disabled=move || saving.get()
                                on:click=move |_| go_next_with_save()
                                style=move || format!(
                                    "display:flex; align-items:center; gap:6px; padding:10px 24px; border-radius:10px; border:none; background:linear-gradient(135deg,#000 0%,#131b2e 100%); color:#fff; font-size:14px; font-weight:700; cursor:pointer;{}",
                                    if saving.get() { "opacity:0.6; cursor:wait;" } else { "" }
                                )
                            >
                                {move || if saving.get() {
                                    "Saving\u{2026}"
                                } else if current_step() == WizardStep::InviteTeam && invite_sent.get() {
                                    "Finish Setup"
                                } else {
                                    "Continue"
                                }}
                                <span class="ms" style="font-size:18px;">"arrow_forward"</span>
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
