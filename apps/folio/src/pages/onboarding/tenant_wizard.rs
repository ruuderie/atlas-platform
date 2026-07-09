// apps/folio/src/pages/onboarding/tenant_wizard.rs
//
// TenantApplicantWizard — /onboard/tenant
//
// 5 steps: Profile → Income & Employment → References → Documents & Consent → Review
// Invite code: pre-fills the unit card on the left panel.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::wizard_shell::{
    ResolvedInviteCode, WizardShell, WizardStepDesc, resolve_invite_code,
};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantApplicationInput {
    pub first_name:   String,
    pub last_name:    String,
    pub email:        String,
    pub phone:        String,
    pub dob:          String,
    pub monthly_income: String,
    pub employer:     String,
    pub invite_code:  Option<String>,
}

#[server(SubmitTenantApplication, "/api")]
pub async fn submit_tenant_application(
    input: TenantApplicationInput,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let payload = serde_json::to_value(&input)
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/folio/applications/submit", &token, None, &payload,
    ).await.map(|_| ()).map_err(server_fn::error::ServerFnError::new)
}

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc { id: "profile",    label: "Your Profile",         skippable: false },
    WizardStepDesc { id: "income",     label: "Income & Employment",   skippable: false },
    WizardStepDesc { id: "references", label: "References",            skippable: false },
    WizardStepDesc { id: "documents",  label: "Documents & Consent",   skippable: false },
    WizardStepDesc { id: "review",     label: "Review & Submit",       skippable: false },
];

#[component]
pub fn TenantApplicantWizard() -> impl IntoView {
    let query    = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());
    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));
    Effect::new(move |_| {
        if let Some(Ok(resolved)) = code_resource.get() { invite_sig.set(resolved); }
    });

    let current_idx = RwSignal::new(0usize);
    let total       = STEPS.len();
    let is_last     = Signal::derive(move || current_idx.get() == total - 1);
    let next_label  = Signal::derive(move || {
        if is_last.get() { "Submit Application" } else { "Continue" }
    });

    // Form signals
    let first_name   = RwSignal::new(String::new());
    let last_name    = RwSignal::new(String::new());
    let email        = RwSignal::new(String::new());
    let phone        = RwSignal::new(String::new());
    let employer     = RwSignal::new(String::new());
    let monthly_inc  = RwSignal::new(String::new());
    let saving:     RwSignal<bool>          = RwSignal::new(false);
    let save_error: RwSignal<Option<String>> = RwSignal::new(None);

    let invite_code_val = move || invite_sig.get().map(|c| c.code.clone());

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx == total - 1 {
            let input = TenantApplicationInput {
                first_name: first_name.get(), last_name: last_name.get(),
                email: email.get(), phone: phone.get(), dob: String::new(),
                monthly_income: monthly_inc.get(), employer: employer.get(),
                invite_code: invite_code_val(),
            };
            // Capture invite id before async block
            let invite_id = invite_sig.get().map(|c| c.code.clone()).unwrap_or_default();
            saving.set(true); save_error.set(None);
            leptos::task::spawn_local(async move {
                match submit_tenant_application(input).await {
                    Ok(_) => {
                        saving.set(false);
                        // Chain: accept invite (provisions G-32 tenant role, no-ops if no code)
                        let redirect = match accept_invite_code(invite_id, "/t/application".to_string()).await {
                            Ok(resp) => resp.redirect,
                            Err(_)   => "/t/application".to_string(),
                        };
                        let nav = leptos_router::hooks::use_navigate();
                        nav(&redirect, Default::default());
                    }
                    Err(e) => { saving.set(false); save_error.set(Some(e.to_string())); }
                }
            });
        } else {
            current_idx.set(idx + 1);
        }
    });
    let on_prev = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx > 0 { current_idx.set(idx - 1); }
    });

    let ctx_body = ViewFn::from(|| view! {
        <p class="wiz-ctx-p">
            "Complete your rental application. Your information is encrypted and shared only with the property manager."
        </p>
        <ul class="wiz-ctx-list">
            <li><span class="ms msf">"check_circle"</span>"Soft credit check — no score impact"</li>
            <li><span class="ms msf">"check_circle"</span>"Decision in 2–3 business days"</li>
            <li><span class="ms msf">"check_circle"</span>"E-sign your lease directly on Folio"</li>
        </ul>
    });

    view! {
        <WizardShell
            steps=STEPS.to_vec()
            current_idx=current_idx
            persona_pill="Tenant Applicant"
            persona_icon="person"
            accent_color="#6366f1"
            panel_bg="#18181b"
            ctx_headline="Apply for your next home"
            ctx_body=ctx_body
            invite_code=invite_sig
            on_next=on_next on_prev=on_prev
            is_last_step=is_last next_label=next_label
        >
            <Show when=move || save_error.get().is_some()>
                <div class="wiz-error-banner"><span class="ms">"warning"</span>
                    {move || save_error.get().unwrap_or_default()}
                </div>
            </Show>

            // Step 1: Profile
            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#4f46e5;">
                        <span class="ms" style="font-size:13px;">"person"</span>"Step 1 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Your Profile"</h1>
                    <p class="wiz-s-sub">"Basic personal information for the rental application."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Personal Info"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Jamie"
                                    prop:value=move || first_name.get()
                                    on:input=move |e| first_name.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Rivera"
                                    prop:value=move || last_name.get()
                                    on:input=move |e| last_name.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Email"</label>
                                <input class="wiz-inp" type="email" placeholder="jamie@email.com"
                                    prop:value=move || email.get()
                                    on:input=move |e| email.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Phone"</label>
                                <input class="wiz-inp" type="tel" placeholder="+1 (555) 000-0000"
                                    prop:value=move || phone.get()
                                    on:input=move |e| phone.set(event_target_value(&e))/></div>
                        </div>
                    </div>
                </div>
            </Show>

            // Step 2: Income
            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#4f46e5;">
                        <span class="ms" style="font-size:13px;">"payments"</span>"Step 2 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Income &amp; Employment"</h1>
                    <p class="wiz-s-sub">"Most landlords require monthly income of 3× the rent."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Primary Employment"</div>
                        <div class="wiz-f"><label class="wiz-label">"Employer Name"</label>
                            <input class="wiz-inp" type="text" placeholder="Company name"
                                prop:value=move || employer.get()
                                on:input=move |e| employer.set(event_target_value(&e))/></div>
                        <div class="wiz-f"><label class="wiz-label">"Monthly Gross Income"</label>
                            <input class="wiz-inp" type="text" placeholder="e.g. $8,200"
                                prop:value=move || monthly_inc.get()
                                on:input=move |e| monthly_inc.set(event_target_value(&e))/></div>
                    </div>
                </div>
            </Show>

            // Steps 3–5: placeholder cards (full forms would follow same pattern)
            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#4f46e5;">
                        <span class="ms" style="font-size:13px;">"people"</span>"Step 3 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Rental References"</h1>
                    <p class="wiz-s-sub">"Provide at least 2 references — ideally a prior landlord plus a personal reference."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Landlord Reference"</div>
                        <div class="wiz-f"><label class="wiz-label">"Landlord Name"</label>
                            <input class="wiz-inp" type="text" placeholder="Previous landlord name"/></div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Phone"</label>
                                <input class="wiz-inp" type="tel" placeholder="+1 (555) 000-0000"/></div>
                            <div class="wiz-f"><label class="wiz-label">"Email"</label>
                                <input class="wiz-inp" type="email" placeholder="landlord@email.com"/></div>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#4f46e5;">
                        <span class="ms" style="font-size:13px;">"upload_file"</span>"Step 4 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Documents &amp; Consent"</h1>
                    <p class="wiz-s-sub">"Upload supporting documents and authorize screening."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Authorization"</div>
                        <label style="display:flex; align-items:center; gap:10px; font-size:14px; cursor:pointer; margin-bottom:10px;">
                            <input type="checkbox" style="accent-color:#6366f1; width:18px; height:18px;"/>
                            "I authorize a soft credit check"
                        </label>
                        <label style="display:flex; align-items:center; gap:10px; font-size:14px; cursor:pointer;">
                            <input type="checkbox" style="accent-color:#6366f1; width:18px; height:18px;"/>
                            "I confirm all information provided is accurate"
                        </label>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 4>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#4f46e5;">
                        <span class="ms" style="font-size:13px;">"fact_check"</span>"Step 5 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Review &amp; Submit"</h1>
                    <p class="wiz-s-sub">"Review your application before submitting."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Summary"</div>
                        <div style="font-size:14px; color:#64748b; display:flex; justify-content:space-between; padding:10px 0; border-bottom:1px solid #e2e8f0;">
                            <span>"Applicant"</span>
                            <strong style="color:#0f172a;">{move || format!("{} {}", first_name.get(), last_name.get())}</strong>
                        </div>
                        <div style="font-size:14px; color:#64748b; display:flex; justify-content:space-between; padding:10px 0;">
                            <span>"Employer"</span>
                            <strong style="color:#0f172a;">{move || employer.get()}</strong>
                        </div>
                    </div>
                </div>
            </Show>
        </WizardShell>
    }
}
