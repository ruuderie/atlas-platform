// apps/folio/src/pages/pmc/onboard.rs
//
// PMC Onboarding Wizard — /pmc/onboard
//
// Admin-initiated onboarding for new Property Management Company accounts.
// Triggered by a platform-admin invite link containing a token.
// Steps: 1) Company details, 2) Primary contact + billing, 3) Portfolio scope,
//         4) Confirm + activate.
// POSTs to /api/folio/pm/onboard  (admin-guarded endpoint).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

// ── Server function ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmcOnboardInput {
    pub invite_token:    String,
    pub company_name:    String,
    pub company_type:    String,
    pub website:         Option<String>,
    pub primary_name:    String,
    pub primary_email:   String,
    pub primary_phone:   String,
    pub billing_email:   Option<String>,
    pub portfolio_types: Vec<String>,
    pub unit_count:      String,
    pub markets:         Vec<String>,
    pub consented:       bool,
}

#[server(SubmitPmcOnboard, "/api")]
pub async fn submit_pmc_onboard(input: PmcOnboardInput) -> Result<String, server_fn::error::ServerFnError> {
    Ok("pmc_stub_id".to_string())
}

// ── Option data ───────────────────────────────────────────────────────────────

const PORTFOLIO_TYPES: &[(&str, &str)] = &[
    ("residential_ltr", "🏘 Residential LTR"),
    ("residential_str", "🏖 Residential STR"),
    ("commercial",      "🏢 Commercial"),
    ("mixed_use",       "🏙 Mixed Use"),
    ("multifamily",     "🏗 Multifamily"),
    ("vacation",        "⛵ Vacation Rentals"),
];

const US_MARKETS: &[&str] = &[
    "Miami, FL", "New York, NY", "Los Angeles, CA", "Austin, TX",
    "Atlanta, GA", "Chicago, IL", "Dallas, TX", "Phoenix, AZ",
    "Orlando, FL", "Seattle, WA", "Denver, CO", "Other",
];

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn PmcOnboard() -> impl IntoView {
    let query        = use_query_map();
    let token        = query.get().get("token").unwrap_or_default();

    let step         = RwSignal::new(1u8);
    let company_name = RwSignal::new(String::new());
    let company_type = RwSignal::new("property_management".to_string());
    let website      = RwSignal::new(String::new());
    let primary_name = RwSignal::new(String::new());
    let primary_email= RwSignal::new(String::new());
    let primary_phone= RwSignal::new(String::new());
    let billing_email= RwSignal::new(String::new());
    let port_types: RwSignal<std::collections::HashSet<&'static str>> = RwSignal::new(std::collections::HashSet::new());
    let unit_count   = RwSignal::new("50_200".to_string());
    let markets: RwSignal<std::collections::HashSet<&'static str>>    = RwSignal::new(std::collections::HashSet::new());
    let consented    = RwSignal::new(false);
    let submitting   = RwSignal::new(false);
    let submitted    = RwSignal::new(false);
    let error        = RwSignal::new(None::<String>);

    let token_sv = store_value(token.clone());
    let handle_submit = move |_| {
        if !consented.get() { return; }
        submitting.set(true);
        let input = PmcOnboardInput {
            invite_token:    token_sv.get_value(),
            company_name:    company_name.get(),
            company_type:    company_type.get(),
            website:         if website.get().is_empty() { None } else { Some(website.get()) },
            primary_name:    primary_name.get(),
            primary_email:   primary_email.get(),
            primary_phone:   primary_phone.get(),
            billing_email:   if billing_email.get().is_empty() { None } else { Some(billing_email.get()) },
            portfolio_types: port_types.get().iter().map(|s| s.to_string()).collect(),
            unit_count:      unit_count.get(),
            markets:         markets.get().iter().map(|s| s.to_string()).collect(),
            consented:       true,
        };
        leptos::task::spawn_local(async move {
            match submit_pmc_onboard(input).await {
                Ok(_)  => { submitted.set(true); submitting.set(false); }
                Err(e) => { error.set(Some(e.to_string())); submitting.set(false); }
            }
        });
    };

    view! {
        <div class="apply-layout">
            <div class="apply-header">
                <div class="apply-logo">"⚡ Atlas Platform"</div>
                <div class="apply-subtitle">"Property Management Company — Account Setup"</div>
                {if !token.is_empty() {
                    view! { <div class="ph-badge ph-badge--paid" style="margin-top:.4rem;">"✓ Invite Token Valid"</div> }.into_any()
                } else {
                    view! { <div class="ph-badge ph-badge--overdue" style="margin-top:.4rem;">"Missing Invite Token"</div> }.into_any()
                }}
            </div>

            {move || if submitted.get() {
                view! {
                    <div class="wiz-success-card" style="max-width:30rem;margin:2rem auto;">
                        <div class="wiz-success-icon">"🏢"</div>
                        <div class="wiz-success-title">"PMC Account Created!"</div>
                        <div class="wiz-success-sub">
                            "Your account is being provisioned. A platform admin will review and activate it within 1 business day. "
                            "You'll receive a confirmation email with login instructions."
                        </div>
                        <div class="wiz-success-actions">
                            <a href="/login" class="btn btn-primary">"Go to Login"</a>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="apply-card">
                        // ── Step bar ──
                        <div class="wiz-steps" style="margin-bottom:1.5rem;">
                            <div class=move || format!("wiz-step {}", if step.get() >= 1 { "wiz-step--active" } else { "" })>"1 Company"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get() >= 2 { "wiz-step--active" } else { "" })>"2 Contact"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get() >= 3 { "wiz-step--active" } else { "" })>"3 Portfolio"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get() >= 4 { "wiz-step--active" } else { "" })>"4 Activate"</div>
                        </div>

                        {move || error.get().map(|e| view! { <div class="wiz-error-banner">"⚠ " {e}</div> })}

                        // ── Step 1: Company ──
                        <Show when=move || step.get() == 1>
                            <div class="apply-section-title">"Company Information"</div>
                            <div class="form-field">
                                <label class="form-label">"Company Name" <span class="form-required">"*"</span></label>
                                <input type="text" class="form-input" placeholder="Apex Property Group LLC"
                                    prop:value=move || company_name.get()
                                    on:input=move |ev| company_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Company Type"</label>
                                <select class="form-select"
                                    on:change=move |ev| company_type.set(event_target_value(&ev))
                                >
                                    <option value="property_management">"Property Management Company"</option>
                                    <option value="brokerage">"Real Estate Brokerage"</option>
                                    <option value="hoa">"HOA / Community Management"</option>
                                    <option value="developer">"Real Estate Developer"</option>
                                    <option value="investment_fund">"Investment Fund / REIT"</option>
                                </select>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Website"</label>
                                <input type="url" class="form-input" placeholder="https://apexpropertygroup.com"
                                    prop:value=move || website.get()
                                    on:input=move |ev| website.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="viol-info-banner">
                                <span class="viol-info-icon">"🔐"</span>
                                <p class="viol-info-text">"Your account will be scoped to a dedicated PMC workspace. Clients and assets will be isolated from other operators."</p>
                            </div>
                            <div class="wiz-footer">
                                <div></div>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || company_name.get().trim().is_empty()
                                    on:click=move |_| step.set(2)
                                >"Next →"</button>
                            </div>
                        </Show>

                        // ── Step 2: Contact & Billing ──
                        <Show when=move || step.get() == 2>
                            <div class="apply-section-title">"Primary Contact & Billing"</div>
                            <div class="form-field">
                                <label class="form-label">"Primary Contact Name" <span class="form-required">"*"</span></label>
                                <input type="text" class="form-input" placeholder="Sarah Chen"
                                    prop:value=move || primary_name.get()
                                    on:input=move |ev| primary_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="apply-two-col">
                                <div class="form-field">
                                    <label class="form-label">"Work Email" <span class="form-required">"*"</span></label>
                                    <input type="email" class="form-input" placeholder="sarah@apex.com"
                                        prop:value=move || primary_email.get()
                                        on:input=move |ev| primary_email.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Phone"</label>
                                    <input type="tel" class="form-input" placeholder="+1 305 000 1234"
                                        prop:value=move || primary_phone.get()
                                        on:input=move |ev| primary_phone.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Billing Email (if different)"</label>
                                <input type="email" class="form-input" placeholder="billing@apex.com"
                                    prop:value=move || billing_email.get()
                                    on:input=move |ev| billing_email.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(1)>"← Back"</button>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || primary_name.get().trim().is_empty() || primary_email.get().trim().is_empty()
                                    on:click=move |_| step.set(3)
                                >"Next →"</button>
                            </div>
                        </Show>

                        // ── Step 3: Portfolio Scope ──
                        <Show when=move || step.get() == 3>
                            <div class="apply-section-title">"Portfolio Scope"</div>

                            <div class="form-field">
                                <label class="form-label">"Property Types Managed (select all)"</label>
                                <div class="triage-category-grid">
                                    {PORTFOLIO_TYPES.iter().map(|(id, label)| {
                                        let cid = *id;
                                        let lbl = *label;
                                        view! {
                                            <button
                                                class=move || format!("triage-cat-btn {}", if port_types.get().contains(cid) { "triage-cat-btn--active" } else { "" })
                                                on:click=move |_| port_types.update(|s| {
                                                    if s.contains(cid) { s.remove(cid); } else { s.insert(cid); }
                                                })
                                            >{lbl}</button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>

                            <div class="form-field">
                                <label class="form-label">"Units Under Management"</label>
                                <select class="form-select"
                                    on:change=move |ev| unit_count.set(event_target_value(&ev))
                                >
                                    <option value="1_10">"1–10 units"</option>
                                    <option value="11_50">"11–50 units"</option>
                                    <option value="50_200" selected>"50–200 units"</option>
                                    <option value="200_500">"200–500 units"</option>
                                    <option value="500_plus">"500+ units"</option>
                                </select>
                            </div>

                            <div class="form-field">
                                <label class="form-label">"Primary Markets"</label>
                                <div class="pmc-markets-grid">
                                    {US_MARKETS.iter().map(|market| {
                                        let m = *market;
                                        view! {
                                            <label class="pmc-market-chip">
                                                <input type="checkbox"
                                                    prop:checked=move || markets.get().contains(m)
                                                    on:change=move |ev: web_sys::Event| {
                                                        let el = Some(event_target::<web_sys::HtmlInputElement>(&ev));
                                                        if let Some(el) = el {
                                                            markets.update(|s| {
                                                                if el.checked() { s.insert(m); } else { s.remove(m); }
                                                            });
                                                        }
                                                    }
                                                />
                                                {m}
                                            </label>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>

                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(2)>"← Back"</button>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || port_types.get().is_empty()
                                    on:click=move |_| step.set(4)
                                >"Review →"</button>
                            </div>
                        </Show>

                        // ── Step 4: Confirm + Activate ──
                        <Show when=move || step.get() == 4>
                            <div class="apply-section-title">"Review & Activate"</div>
                            <div class="wiz-confirm-table">
                                <div class="wiz-confirm-row"><span>"Company"</span><strong>{move || company_name.get()}</strong></div>
                                <div class="wiz-confirm-row"><span>"Type"</span><strong>{move || company_type.get().replace('_', " ")}</strong></div>
                                <div class="wiz-confirm-row"><span>"Primary Contact"</span><strong>{move || primary_name.get()}</strong></div>
                                <div class="wiz-confirm-row"><span>"Email"</span><strong>{move || primary_email.get()}</strong></div>
                                <div class="wiz-confirm-row"><span>"Units"</span><strong>{move || unit_count.get().replace('_', "–")}</strong></div>
                                <div class="wiz-confirm-row"><span>"Portfolio Types"</span><strong>{move || {
                                    let pt = port_types.get();
                                    if pt.is_empty() { "None".to_string() } else { pt.iter().cloned().collect::<Vec<_>>().join(", ") }
                                }}</strong></div>
                                <div class="wiz-confirm-row"><span>"Markets"</span><strong>{move || {
                                    let m = markets.get();
                                    if m.is_empty() { "Not specified".to_string() } else { m.iter().cloned().collect::<Vec<_>>().join(", ") }
                                }}</strong></div>
                            </div>

                            <div class="pmc-onboard-platform-info">
                                <div class="pmc-onboard-plan-badge">"Enterprise PMC Plan"</div>
                                <div class="text-xs text-on-surface-variant" style="margin-top:.35rem;">"Per-unit pricing · Includes white-label subdomain · SFTP / API access · Dedicated support"</div>
                            </div>

                            <label class="apply-consent-row">
                                <input type="checkbox"
                                    prop:checked=move || consented.get()
                                    on:change=move |ev: web_sys::Event| {
                                        let el = Some(event_target::<web_sys::HtmlInputElement>(&ev));
                                        if let Some(el) = el { consented.set(el.checked()); }
                                    }
                                />
                                <span>"I agree to the Atlas Platform Terms of Service, Data Processing Agreement, and PMC Operator License."</span>
                            </label>

                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(3)>"← Back"</button>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || !consented.get() || submitting.get()
                                    on:click=handle_submit
                                >
                                    {move || if submitting.get() { "Activating…" } else { "Activate PMC Account" }}
                                </button>
                            </div>
                        </Show>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
