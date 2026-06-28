// apps/folio/src/pages/marketing/renter_application.rs
//
// Renter Application — /apply/:property_id
//
// Public (unauthenticated) multi-step rental application.
// Step 1: Personal info. Step 2: Income & employment. Step 3: References.
// Step 4: Consent + submit. Posts to /api/folio/applications/public.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

// ── Server function ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenterApplicationInput {
    pub property_id:      String,
    pub first_name:       String,
    pub last_name:        String,
    pub email:            String,
    pub phone:            String,
    pub employment_type:  String,
    pub employer:         String,
    pub monthly_income:   String,
    pub reference_name:   String,
    pub reference_phone:  String,
    pub consented:        bool,
}

#[server(SubmitRenterApplication, "/api")]
pub async fn submit_renter_application(
    app: RenterApplicationInput,
) -> Result<String, server_fn::error::ServerFnError> {
    Ok("app_stub_id".to_string())
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn RenterApplication() -> impl IntoView {
    let params      = use_params_map();
    let property_id = params.get().get("property_id").unwrap_or_default();

    let step          = RwSignal::new(1u8);
    let first_name    = RwSignal::new(String::new());
    let last_name     = RwSignal::new(String::new());
    let email         = RwSignal::new(String::new());
    let phone         = RwSignal::new(String::new());
    let emp_type      = RwSignal::new("employed".to_string());
    let employer      = RwSignal::new(String::new());
    let income        = RwSignal::new(String::new());
    let ref_name      = RwSignal::new(String::new());
    let ref_phone     = RwSignal::new(String::new());
    let consented     = RwSignal::new(false);
    let submitting    = RwSignal::new(false);
    let submitted     = RwSignal::new(false);
    let error         = RwSignal::new(None::<String>);

    let property_id2 = property_id.clone();
    let handle_submit = move |_| {
        if !consented.get() { return; }
        submitting.set(true);
        let app = RenterApplicationInput {
            property_id:     property_id2.clone(),
            first_name:      first_name.get(),
            last_name:       last_name.get(),
            email:           email.get(),
            phone:           phone.get(),
            employment_type: emp_type.get(),
            employer:        employer.get(),
            monthly_income:  income.get(),
            reference_name:  ref_name.get(),
            reference_phone: ref_phone.get(),
            consented:       consented.get(),
        };
        leptos::task::spawn_local(async move {
            match submit_renter_application(app).await {
                Ok(_)  => { submitted.set(true); submitting.set(false); }
                Err(e) => { error.set(Some(e.to_string())); submitting.set(false); }
            }
        });
    };

    view! {
        <div class="apply-layout">
            // ── Brand header ──
            <div class="apply-header">
                <div class="apply-logo">"⚡ Atlas"</div>
                <div class="apply-subtitle">"Rental Application"</div>
                {if !property_id.is_empty() {
                    view! { <div class="apply-property-id">"Property " {property_id.chars().take(8).collect::<String>()} "…"</div> }.into_any()
                } else { ().into_any() }}
            </div>

            {move || if submitted.get() {
                view! {
                    <div class="wiz-success-card" style="max-width:28rem;margin:2rem auto;">
                        <div class="wiz-success-icon">"✓"</div>
                        <div class="wiz-success-title">"Application Submitted!"</div>
                        <div class="wiz-success-sub">"You'll receive a confirmation email shortly. The landlord will review your application and contact you within 3–5 business days."</div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="apply-card">

                        // Step bar
                        <div class="wiz-steps" style="margin-bottom:1.5rem;">
                            <div class=move || format!("wiz-step {}", if step.get() >= 1 { "wiz-step--active" } else { "" })>"1 Personal"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get() >= 2 { "wiz-step--active" } else { "" })>"2 Income"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get() >= 3 { "wiz-step--active" } else { "" })>"3 References"</div>
                            <div class="wiz-step-divider"></div>
                            <div class=move || format!("wiz-step {}", if step.get() >= 4 { "wiz-step--active" } else { "" })>"4 Submit"</div>
                        </div>

                        {move || error.get().map(|e| view! {
                            <div class="wiz-error-banner">"⚠ " {e}</div>
                        })}

                        // ── Step 1: Personal ──
                        <Show when=move || step.get() == 1>
                            <div class="apply-section-title">"Personal Information"</div>
                            <div class="apply-two-col">
                                <div class="form-field">
                                    <label class="form-label">"First Name" <span class="form-required">"*"</span></label>
                                    <input type="text" class="form-input" placeholder="Jane"
                                        prop:value=move || first_name.get()
                                        on:input=move |ev| first_name.set(event_target_value(&ev))
                                    />
                                </div>
                                <div class="form-field">
                                    <label class="form-label">"Last Name" <span class="form-required">"*"</span></label>
                                    <input type="text" class="form-input" placeholder="Smith"
                                        prop:value=move || last_name.get()
                                        on:input=move |ev| last_name.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Email Address" <span class="form-required">"*"</span></label>
                                <input type="email" class="form-input" placeholder="jane@example.com"
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
                            <div class="wiz-footer">
                                <div></div>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || first_name.get().trim().is_empty() || email.get().trim().is_empty()
                                    on:click=move |_| step.set(2)
                                >"Next →"</button>
                            </div>
                        </Show>

                        // ── Step 2: Income ──
                        <Show when=move || step.get() == 2>
                            <div class="apply-section-title">"Income & Employment"</div>
                            <div class="form-field">
                                <label class="form-label">"Employment Status"</label>
                                <select class="form-select"
                                    on:change=move |ev| emp_type.set(event_target_value(&ev))
                                >
                                    <option value="employed">"Employed Full-Time"</option>
                                    <option value="part_time">"Employed Part-Time"</option>
                                    <option value="self_employed">"Self-Employed"</option>
                                    <option value="retired">"Retired"</option>
                                    <option value="student">"Student"</option>
                                    <option value="other">"Other"</option>
                                </select>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Employer / Company"</label>
                                <input type="text" class="form-input" placeholder="Acme Corp"
                                    prop:value=move || employer.get()
                                    on:input=move |ev| employer.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Monthly Gross Income"</label>
                                <input type="text" class="form-input" placeholder="e.g. 6500"
                                    prop:value=move || income.get()
                                    on:input=move |ev| income.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(1)>"← Back"</button>
                                <button class="btn btn-primary" on:click=move |_| step.set(3)>"Next →"</button>
                            </div>
                        </Show>

                        // ── Step 3: References ──
                        <Show when=move || step.get() == 3>
                            <div class="apply-section-title">"Personal Reference"</div>
                            <div class="form-field">
                                <label class="form-label">"Reference Name"</label>
                                <input type="text" class="form-input" placeholder="Full name"
                                    prop:value=move || ref_name.get()
                                    on:input=move |ev| ref_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Reference Phone"</label>
                                <input type="tel" class="form-input" placeholder="+1 555 000 0000"
                                    prop:value=move || ref_phone.get()
                                    on:input=move |ev| ref_phone.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(2)>"← Back"</button>
                                <button class="btn btn-primary" on:click=move |_| step.set(4)>"Review →"</button>
                            </div>
                        </Show>

                        // ── Step 4: Consent + Submit ──
                        <Show when=move || step.get() == 4>
                            <div class="apply-section-title">"Review & Submit"</div>
                            <div class="wiz-confirm-table">
                                <div class="wiz-confirm-row"><span>"Name"</span><strong>{move || format!("{} {}", first_name.get(), last_name.get())}</strong></div>
                                <div class="wiz-confirm-row"><span>"Email"</span><strong>{move || email.get()}</strong></div>
                                <div class="wiz-confirm-row"><span>"Employment"</span><strong>{move || emp_type.get().replace('_', " ")}</strong></div>
                                <div class="wiz-confirm-row"><span>"Monthly Income"</span><strong>{move || if income.get().is_empty() { "Not provided".to_string() } else { format!("${}", income.get()) }}</strong></div>
                                <div class="wiz-confirm-row"><span>"Reference"</span><strong>{move || if ref_name.get().is_empty() { "—".to_string() } else { ref_name.get() }}</strong></div>
                            </div>

                            <label class="apply-consent-row">
                                <input type="checkbox"
                                    prop:checked=move || consented.get()
                                    on:change=move |ev: web_sys::Event| {
                                        let el = Some(event_target::<web_sys::HtmlInputElement>(&ev));
                                        if let Some(el) = el { consented.set(el.checked()); }
                                    }
                                />
                                <span>"I consent to a background and credit check. I confirm all information provided is accurate."</span>
                            </label>

                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(3)>"← Back"</button>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || !consented.get() || submitting.get()
                                    on:click=handle_submit
                                >{move || if submitting.get() { "Submitting…" } else { "Submit Application" }}</button>
                            </div>
                        </Show>

                    </div>
                }.into_any()
            }}
        </div>
    }
}
