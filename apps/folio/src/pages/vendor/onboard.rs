// apps/folio/src/pages/vendor/onboard.rs
//
// Vendor Onboarding — /v/onboard
//
// Token-gated onboarding wizard for service providers / contractors.
// Steps: 1) Business details, 2) Trades + coverage, 3) Certifications, 4) Submit.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

// ── Server function ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorOnboardInput {
    pub invite_token:  String,
    pub business_name: String,
    pub contact_name:  String,
    pub email:         String,
    pub phone:         String,
    pub trades:        Vec<String>,
    pub coverage_area: String,
    pub license_number:Option<String>,
    pub insured:       bool,
}

#[server(SubmitVendorOnboard, "/api")]
pub async fn submit_vendor_onboard(input: VendorOnboardInput) -> Result<String, server_fn::error::ServerFnError> {
    Ok("vendor_stub".to_string())
}

// ── Trade options ─────────────────────────────────────────────────────────────

const TRADES: &[(&str, &str)] = &[
    ("plumbing",      "🚰 Plumbing"),
    ("electrical",    "⚡ Electrical"),
    ("hvac",          "❄️ HVAC"),
    ("roofing",       "🏠 Roofing"),
    ("painting",      "🎨 Painting"),
    ("landscaping",   "🌿 Landscaping"),
    ("cleaning",      "🧹 Cleaning"),
    ("locksmith",     "🔑 Locksmith"),
    ("pest_control",  "🐛 Pest Control"),
    ("general",       "🔧 General"),
];

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn VendorOnboard() -> impl IntoView {
    let query = use_query_map();
    let token = query.get().get("token").unwrap_or_default();

    let step          = RwSignal::new(1u8);
    let biz_name      = RwSignal::new(String::new());
    let contact_name  = RwSignal::new(String::new());
    let email         = RwSignal::new(String::new());
    let phone         = RwSignal::new(String::new());
    let trades_sel: RwSignal<std::collections::HashSet<&'static str>> = RwSignal::new(std::collections::HashSet::new());
    let coverage      = RwSignal::new(String::new());
    let license       = RwSignal::new(String::new());
    let insured       = RwSignal::new(false);
    let submitting    = RwSignal::new(false);
    let submitted     = RwSignal::new(false);
    let error         = RwSignal::new(None::<String>);

    let token2 = token.clone();
    let handle_submit = move |_| {
        submitting.set(true);
        let trades_vec: Vec<String> = trades_sel.get().iter().map(|s| s.to_string()).collect();
        let input = VendorOnboardInput {
            invite_token:   token2.clone(),
            business_name:  biz_name.get(),
            contact_name:   contact_name.get(),
            email:          email.get(),
            phone:          phone.get(),
            trades:         trades_vec,
            coverage_area:  coverage.get(),
            license_number: if license.get().is_empty() { None } else { Some(license.get()) },
            insured:        insured.get(),
        };
        leptos::task::spawn_local(async move {
            match submit_vendor_onboard(input).await {
                Ok(_)  => { submitted.set(true); submitting.set(false); }
                Err(e) => { error.set(Some(e.to_string())); submitting.set(false); }
            }
        });
    };

    view! {
        <div class="apply-layout">
            <div class="apply-header">
                <div class="apply-logo">"⚡ Atlas"</div>
                <div class="apply-subtitle">"Vendor Onboarding"</div>
                {if !token.is_empty() {
                    view! { <div class="ph-badge ph-badge--paid">"✓ Invite Valid"</div> }.into_any()
                } else { view! { <div class="ph-badge ph-badge--overdue">"No Invite Token"</div> }.into_any() }}
            </div>

            {move || if submitted.get() {
                view! {
                    <div class="wiz-success-card" style="max-width:28rem;margin:2rem auto;">
                        <div class="wiz-success-icon">"🔧"</div>
                        <div class="wiz-success-title">"Welcome to the Atlas Network!"</div>
                        <div class="wiz-success-sub">"Your vendor profile is under review. You'll receive an email once approved. You can then log in to accept work orders."</div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="apply-card">
                        <div class="wiz-steps" style="margin-bottom:1.5rem;">
                            <div class=move || format!("wiz-step {}", if step.get() >= 1 { "wiz-step--active" } else { "" })>"1 Business"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get() >= 2 { "wiz-step--active" } else { "" })>"2 Trades"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get() >= 3 { "wiz-step--active" } else { "" })>"3 Credentials"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get() >= 4 { "wiz-step--active" } else { "" })>"4 Submit"</div>
                        </div>

                        {move || error.get().map(|e| view! { <div class="wiz-error-banner">"⚠ " {e}</div> })}

                        // Step 1
                        <Show when=move || step.get() == 1>
                            <div class="apply-section-title">"Business Details"</div>
                            <div class="form-field">
                                <label class="form-label">"Business / Company Name" <span class="form-required">"*"</span></label>
                                <input type="text" class="form-input" placeholder="Apex Plumbing LLC"
                                    prop:value=move || biz_name.get()
                                    on:input=move |ev| biz_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Contact Name" <span class="form-required">"*"</span></label>
                                <input type="text" class="form-input" placeholder="John Smith"
                                    prop:value=move || contact_name.get()
                                    on:input=move |ev| contact_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="apply-two-col">
                                <div class="form-field">
                                    <label class="form-label">"Email" <span class="form-required">"*"</span></label>
                                    <input type="email" class="form-input" placeholder="john@apex.com"
                                        prop:value=move || email.get()
                                        on:input=move |ev| email.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Phone"</label>
                                    <input type="tel" class="form-input" placeholder="+1 555 000 1234"
                                        prop:value=move || phone.get()
                                        on:input=move |ev| phone.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                            <div class="wiz-footer">
                                <div></div>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || biz_name.get().trim().is_empty() || email.get().trim().is_empty()
                                    on:click=move |_| step.set(2)
                                >"Next →"</button>
                            </div>
                        </Show>

                        // Step 2: Trades
                        <Show when=move || step.get() == 2>
                            <div class="apply-section-title">"Trades & Service Area"</div>
                            <div class="triage-category-grid">
                                {TRADES.iter().map(|(id, label)| {
                                    let cid = *id;
                                    let lbl = *label;
                                    view! {
                                        <button
                                            class=move || format!("triage-cat-btn {}", if trades_sel.get().contains(cid) { "triage-cat-btn--active" } else { "" })
                                            on:click=move |_| trades_sel.update(|s| {
                                                if s.contains(cid) { s.remove(cid); } else { s.insert(cid); }
                                            })
                                        >{lbl}</button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            <div class="form-field" style="margin-top:1rem;">
                                <label class="form-label">"Service Coverage Area"</label>
                                <input type="text" class="form-input" placeholder="e.g. Miami-Dade County, FL"
                                    prop:value=move || coverage.get()
                                    on:input=move |ev| coverage.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(1)>"← Back"</button>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || trades_sel.get().is_empty()
                                    on:click=move |_| step.set(3)
                                >"Next →"</button>
                            </div>
                        </Show>

                        // Step 3: Credentials
                        <Show when=move || step.get() == 3>
                            <div class="apply-section-title">"Licenses & Insurance"</div>
                            <div class="form-field">
                                <label class="form-label">"Contractor License Number"</label>
                                <input type="text" class="form-input" placeholder="Optional"
                                    prop:value=move || license.get()
                                    on:input=move |ev| license.set(event_target_value(&ev))
                                />
                            </div>
                            <label class="apply-consent-row">
                                <input type="checkbox"
                                    prop:checked=move || insured.get()
                                    on:change=move |ev: web_sys::Event| {
                                        let el = event_target::<web_sys::HtmlInputElement>(&ev).ok();
                                        if let Some(el) = el { insured.set(el.checked()); }
                                    }
                                />
                                <span>"I carry general liability insurance (min $1M coverage)"</span>
                            </label>
                            <div class="viol-info-banner" style="margin-top:1rem;">
                                <span class="viol-info-icon">"📄"</span>
                                <p class="viol-info-text">"Certificate of insurance upload will be available in Phase 7. Please have it ready — you may be asked to provide it during vetting."</p>
                            </div>
                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(2)>"← Back"</button>
                                <button class="btn btn-primary" on:click=move |_| step.set(4)>"Review →"</button>
                            </div>
                        </Show>

                        // Step 4: Confirm
                        <Show when=move || step.get() == 4>
                            <div class="apply-section-title">"Confirm & Submit"</div>
                            <div class="wiz-confirm-table">
                                <div class="wiz-confirm-row"><span>"Business"</span><strong>{move || biz_name.get()}</strong></div>
                                <div class="wiz-confirm-row"><span>"Contact"</span><strong>{move || contact_name.get()}</strong></div>
                                <div class="wiz-confirm-row"><span>"Email"</span><strong>{move || email.get()}</strong></div>
                                <div class="wiz-confirm-row"><span>"Trades"</span><strong>{move || {
                                    let sel = trades_sel.get();
                                    if sel.is_empty() { "None".to_string() } else { sel.iter().cloned().collect::<Vec<_>>().join(", ") }
                                }}</strong></div>
                                <div class="wiz-confirm-row"><span>"Area"</span><strong>{move || if coverage.get().is_empty() { "—".to_string() } else { coverage.get() }}</strong></div>
                                <div class="wiz-confirm-row"><span>"Insured"</span><strong>{move || if insured.get() { "Yes" } else { "No" }}</strong></div>
                            </div>
                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(3)>"← Back"</button>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || submitting.get()
                                    on:click=handle_submit
                                >{move || if submitting.get() { "Submitting…" } else { "Submit Profile" }}</button>
                            </div>
                        </Show>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
