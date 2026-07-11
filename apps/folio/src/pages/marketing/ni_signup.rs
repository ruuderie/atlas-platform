// apps/folio/src/pages/marketing/ni_signup.rs
//
// Network Instance Signup — /ni/signup
//
// Self-serve operator onboarding for new Network Instance (white-label
// marketplace) accounts. Distinct from PMC Onboard — this creates a
// subdomain-isolated tenant site (e.g., miami.yourbrand.com), not a PMC
// account within an existing Folio instance.
//
// Steps:
//   1. Brand & domain — instance name, slug, subdomain preview
//   2. Use case — LTR / STR / both, market focus
//   3. Account — admin name, email, password
//   4. Plan & confirm — plan tier, billing email, launch date
//
// POST → /api/pub/network-instances   (public endpoint, no auth)
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NiSignupInput {
    pub instance_name: String,
    pub subdomain: String,
    pub use_cases: Vec<String>,
    pub primary_market: String,
    pub admin_name: String,
    pub admin_email: String,
    pub admin_password: String,
    pub plan: String,
    pub billing_email: Option<String>,
    pub launch_target: Option<String>,
}

#[server(SubmitNiSignup, "/api")]
pub async fn submit_ni_signup(
    input: NiSignupInput,
) -> Result<String, server_fn::error::ServerFnError> {
    // Returns the new instance slug on success
    Ok(input.subdomain.clone())
}

// ── Plan data ─────────────────────────────────────────────────────────────────

const PLANS: &[(&str, &str, &str, &str)] = &[
    (
        "starter",
        "Starter",
        "$99/mo",
        "Up to 25 listings · 1 subdomain · Basic support",
    ),
    (
        "growth",
        "Growth",
        "$299/mo",
        "Up to 200 listings · Custom domain · Priority support",
    ),
    (
        "enterprise",
        "Enterprise",
        "Custom",
        "Unlimited · Multi-domain · Dedicated CSM · SLA",
    ),
];

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn NiSignup() -> impl IntoView {
    let step = RwSignal::new(1u8);
    let input = RwSignal::new(NiSignupInput {
        plan: "growth".to_string(),
        ..Default::default()
    });

    let use_case_ltr = RwSignal::new(true);
    let use_case_str = RwSignal::new(false);

    let confirm_pass = RwSignal::new(String::new());
    let submitting = RwSignal::new(false);
    let submitted = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);

    // Live subdomain preview
    let slug_preview = move || {
        let s = input.get().subdomain;
        if s.is_empty() {
            "yourslug.atlas.app".to_string()
        } else {
            format!("{s}.atlas.app")
        }
    };

    let handle_submit = move |_| {
        let mut inp = input.get();
        let mut use_cases = vec![];
        if use_case_ltr.get() {
            use_cases.push("ltr".to_string());
        }
        if use_case_str.get() {
            use_cases.push("str".to_string());
        }
        inp.use_cases = use_cases;
        if inp.admin_password != confirm_pass.get() {
            error.set(Some("Passwords do not match.".to_string()));
            return;
        }
        submitting.set(true);
        leptos::task::spawn_local(async move {
            match submit_ni_signup(inp).await {
                Ok(slug) => {
                    submitted.set(Some(slug));
                    submitting.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    submitting.set(false);
                }
            }
        });
    };

    view! {
        <div class="apply-layout">
            <div class="apply-header">
                <div class="apply-logo">"⚡ Atlas Platform"</div>
                <div class="apply-subtitle">"Launch your own white-label property marketplace"</div>
            </div>

            {move || if let Some(slug) = submitted.get() {
                view! {
                    <div class="wiz-success-card" style="max-width:32rem;margin:2rem auto;">
                        <div class="wiz-success-icon">"🚀"</div>
                        <div class="wiz-success-title">"Network Instance Created!"</div>
                        <div class="wiz-success-sub">
                            "Your instance is being provisioned at "
                            <strong>{format!("{slug}.atlas.app")}</strong>
                            ". Expect a setup email within 5 minutes."
                        </div>
                        <div class="ni-signup-next">
                            <div class="ni-signup-next-step">
                                <span class="inquiry-confirm-step-num">"1"</span>
                                "Verify your admin email"
                            </div>
                            <div class="ni-signup-next-step">
                                <span class="inquiry-confirm-step-num">"2"</span>
                                "Set up your brand (logo, colors, domain)"
                            </div>
                            <div class="ni-signup-next-step">
                                <span class="inquiry-confirm-step-num">"3"</span>
                                "Import or add your first listings"
                            </div>
                        </div>
                        <div class="wiz-success-actions">
                            <a href="/login" class="btn btn-primary">"Admin Login →"</a>
                            <a href="/lp"    class="btn btn-ghost">"Learn More"</a>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="apply-card">
                        // ── Step indicator ──
                        <div class="wiz-steps" style="margin-bottom:1.5rem;">
                            <div class=move || format!("wiz-step {}", if step.get()>=1 {"wiz-step--active"} else {""})>"1 Brand"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get()>=2 {"wiz-step--active"} else {""})>"2 Use Case"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get()>=3 {"wiz-step--active"} else {""})>"3 Account"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get()>=4 {"wiz-step--active"} else {""})>"4 Plan"</div>
                        </div>

                        {move || error.get().map(|e| view! { <div class="wiz-error-banner">"⚠ " {e}</div> })}

                        // ── Step 1: Brand & Domain ──
                        <Show when=move || step.get() == 1>
                            <div class="apply-section-title">"Brand & Domain"</div>

                            <div class="form-field">
                                <label class="form-label">"Marketplace Name" <span class="form-required">"*"</span></label>
                                <input type="text" class="form-input" placeholder="Miami Rentals"
                                    prop:value=move || input.get().instance_name.clone()
                                    on:input=move |ev| input.update(|i| i.instance_name = event_target_value(&ev))
                                />
                                <div class="form-hint">"Your brand name — shown to tenants and guests."</div>
                            </div>

                            <div class="form-field">
                                <label class="form-label">"Subdomain Slug" <span class="form-required">"*"</span></label>
                                <input type="text" class="form-input" placeholder="miami-rentals"
                                    prop:value=move || input.get().subdomain.clone()
                                    on:input=move |ev| {
                                        let val = event_target_value(&ev)
                                            .to_lowercase()
                                            .replace(' ', "-")
                                            .chars()
                                            .filter(|c| c.is_alphanumeric() || *c == '-')
                                            .collect::<String>();
                                        input.update(|i| i.subdomain = val);
                                    }
                                />
                                <div class="ni-domain-preview">"🌐 " {move || slug_preview()}</div>
                            </div>

                            <div class="viol-info-banner">
                                <span class="viol-info-icon">"🔒"</span>
                                <p class="viol-info-text">"Custom domains (e.g. miami.yourbrand.com) are available on Growth and Enterprise plans. Configure after signup."</p>
                            </div>

                            <div class="wiz-footer">
                                <div></div>
                                <button class="btn btn-primary"
                                    disabled=move || input.get().instance_name.trim().is_empty() || input.get().subdomain.trim().len() < 3
                                    on:click=move |_| step.set(2)
                                >"Next →"</button>
                            </div>
                        </Show>

                        // ── Step 2: Use Case ──
                        <Show when=move || step.get() == 2>
                            <div class="apply-section-title">"Use Case"</div>

                            <div class="form-field">
                                <label class="form-label">"What will your marketplace list?"</label>
                                <div class="ni-usecase-grid">
                                    <label class=move || format!("ni-usecase-card {}", if use_case_ltr.get() {"ni-usecase-card--active"} else {""})>
                                        <input type="checkbox" style="display:none;"
                                            prop:checked=move || use_case_ltr.get()
                                            on:change=move |ev: web_sys::Event| {
                                                let el = Some(event_target::<web_sys::HtmlInputElement>(&ev));
                                                if let Some(el) = el { use_case_ltr.set(el.checked()); }
                                            }
                                        />
                                        <div class="ni-usecase-icon">"🏘"</div>
                                        <div class="ni-usecase-label">"Long-Term Rentals"</div>
                                        <div class="ni-usecase-desc">"Monthly leases, tenant applications, maintenance"</div>
                                    </label>
                                    <label class=move || format!("ni-usecase-card {}", if use_case_str.get() {"ni-usecase-card--active"} else {""})>
                                        <input type="checkbox" style="display:none;"
                                            prop:checked=move || use_case_str.get()
                                            on:change=move |ev: web_sys::Event| {
                                                let el = Some(event_target::<web_sys::HtmlInputElement>(&ev));
                                                if let Some(el) = el { use_case_str.set(el.checked()); }
                                            }
                                        />
                                        <div class="ni-usecase-icon">"🏖"</div>
                                        <div class="ni-usecase-label">"Short-Term Rentals"</div>
                                        <div class="ni-usecase-desc">"Nightly bookings, channel sync, guest messaging"</div>
                                    </label>
                                </div>
                            </div>

                            <div class="form-field">
                                <label class="form-label">"Primary Market"</label>
                                <input type="text" class="form-input" placeholder="Miami, FL"
                                    prop:value=move || input.get().primary_market.clone()
                                    on:input=move |ev| input.update(|i| i.primary_market = event_target_value(&ev))
                                />
                            </div>

                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(1)>"← Back"</button>
                                <button class="btn btn-primary"
                                    disabled=move || !use_case_ltr.get() && !use_case_str.get()
                                    on:click=move |_| step.set(3)
                                >"Next →"</button>
                            </div>
                        </Show>

                        // ── Step 3: Account ──
                        <Show when=move || step.get() == 3>
                            <div class="apply-section-title">"Admin Account"</div>

                            <div class="form-field">
                                <label class="form-label">"Full Name" <span class="form-required">"*"</span></label>
                                <input type="text" class="form-input" placeholder="Alex Rivera"
                                    prop:value=move || input.get().admin_name.clone()
                                    on:input=move |ev| input.update(|i| i.admin_name = event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Email" <span class="form-required">"*"</span></label>
                                <input type="email" class="form-input" placeholder="alex@miamientals.com"
                                    prop:value=move || input.get().admin_email.clone()
                                    on:input=move |ev| input.update(|i| i.admin_email = event_target_value(&ev))
                                />
                            </div>
                            <div class="apply-two-col">
                                <div class="form-field">
                                    <label class="form-label">"Password" <span class="form-required">"*"</span></label>
                                    <input type="password" class="form-input" placeholder="At least 12 characters"
                                        on:input=move |ev| input.update(|i| i.admin_password = event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Confirm Password"</label>
                                    <input type="password" class="form-input"
                                        on:input=move |ev| confirm_pass.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>

                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(2)>"← Back"</button>
                                <button class="btn btn-primary"
                                    disabled=move || {
                                        let i = input.get();
                                        i.admin_name.trim().is_empty() || i.admin_email.trim().is_empty() || i.admin_password.len() < 8
                                    }
                                    on:click=move |_| step.set(4)
                                >"Next →"</button>
                            </div>
                        </Show>

                        // ── Step 4: Plan & Confirm ──
                        <Show when=move || step.get() == 4>
                            <div class="apply-section-title">"Choose a Plan"</div>

                            <div class="ni-plan-grid">
                                {PLANS.iter().map(|(id, name, price, desc)| {
                                    let pid = *id;
                                    let pname = *name;
                                    let pprice = *price;
                                    let pdesc = *desc;
                                    view! {
                                        <div
                                            class=move || format!("ni-plan-card {}", if input.get().plan == pid {"ni-plan-card--active"} else {""})
                                            on:click=move |_| input.update(|i| i.plan = pid.to_string())
                                        >
                                            <div class="ni-plan-name">{pname}</div>
                                            <div class="ni-plan-price">{pprice}</div>
                                            <div class="ni-plan-desc">{pdesc}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>

                            <div class="form-field" style="margin-top:.75rem;">
                                <label class="form-label">"Billing Email (if different)"</label>
                                <input type="email" class="form-input" placeholder="billing@miamientals.com"
                                    on:input=move |ev| {
                                        let v = event_target_value(&ev);
                                        input.update(|i| i.billing_email = if v.is_empty() { None } else { Some(v) });
                                    }
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Target Launch Date"</label>
                                <input type="date" class="form-input"
                                    on:input=move |ev| {
                                        let v = event_target_value(&ev);
                                        input.update(|i| i.launch_target = if v.is_empty() { None } else { Some(v) });
                                    }
                                />
                            </div>

                            <div class="wiz-confirm-table" style="margin-top:.75rem;">
                                <div class="wiz-confirm-row"><span>"Instance"</span><strong>{move || input.get().instance_name.clone()}</strong></div>
                                <div class="wiz-confirm-row"><span>"URL"</span><strong>{move || slug_preview()}</strong></div>
                                <div class="wiz-confirm-row"><span>"Plan"</span><strong>{move || input.get().plan.clone()}</strong></div>
                                <div class="wiz-confirm-row"><span>"Admin"</span><strong>{move || input.get().admin_email.clone()}</strong></div>
                            </div>

                            <div class="wiz-footer" style="margin-top:1rem;">
                                <button class="btn btn-ghost" on:click=move |_| step.set(3)>"← Back"</button>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || submitting.get()
                                    on:click=handle_submit
                                >{move || if submitting.get() { "Creating…" } else { "🚀 Launch Instance" }}</button>
                            </div>
                        </Show>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
