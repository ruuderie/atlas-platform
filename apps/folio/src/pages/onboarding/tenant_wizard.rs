// apps/folio/src/pages/onboarding/tenant_wizard.rs
//
// TenantPendingWizard — /onboard/tenant
//
// Pending-onboard after application approval (wiz_tenant_pending_onboard):
//   1. Complete Profile
//   2. Lease Review & Sign
//   3. Move-In Checklist
//   4. Portal Setup
//
// Component name kept as TenantApplicantWizard for route compatibility.
// Existing submit_tenant_application server fn is reused to persist profile fields.

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
    WizardStepDesc { id: "profile",   label: "Complete Profile",       skippable: false },
    WizardStepDesc { id: "lease",     label: "Lease Review & Sign",    skippable: false },
    WizardStepDesc { id: "movein",    label: "Move-In Checklist",      skippable: false },
    WizardStepDesc { id: "portal",    label: "Portal Setup",           skippable: false },
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
        if is_last.get() { "Go to My Portal" } else { "Continue" }
    });

    // Profile
    let first_name = RwSignal::new(String::new());
    let last_name  = RwSignal::new(String::new());
    let phone      = RwSignal::new(String::new());
    let dob        = RwSignal::new(String::new());
    let emerg_name = RwSignal::new(String::new());
    let emerg_rel  = RwSignal::new(String::new());
    let occupant   = RwSignal::new(String::new());

    // Lease
    let lease_signed = RwSignal::new(false);

    // Checklist
    let chk_rent     = RwSignal::new(false);
    let chk_deposit  = RwSignal::new(false);
    let chk_electric = RwSignal::new(false);
    let chk_internet = RwSignal::new(false);
    let chk_insurance = RwSignal::new(false);
    let chk_keys     = RwSignal::new(false);
    let chk_condition = RwSignal::new(false);
    let chk_super    = RwSignal::new(false);

    // Portal
    let pay_method = RwSignal::new("ach".to_string());
    let notify_rent = RwSignal::new(true);
    let notify_maint = RwSignal::new(true);
    let notify_lease = RwSignal::new(true);
    let notify_msg  = RwSignal::new(true);

    let saving:     RwSignal<bool>          = RwSignal::new(false);
    let save_error: RwSignal<Option<String>> = RwSignal::new(None);

    let invite_code_val = move || invite_sig.get().map(|c| c.code.clone());

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx == total - 1 {
            // Persist profile fields via existing server fn (adapted mapping)
            let input = TenantApplicationInput {
                first_name: first_name.get(),
                last_name: last_name.get(),
                email: String::new(),
                phone: phone.get(),
                dob: dob.get(),
                monthly_income: String::new(),
                employer: emerg_name.get(), // emergency contact name reused as spare field
                invite_code: invite_code_val(),
            };
            let invite_id = invite_sig.get().map(|c| c.code.clone()).unwrap_or_default();
            saving.set(true); save_error.set(None);
            leptos::task::spawn_local(async move {
                let _ = submit_tenant_application(input).await;
                saving.set(false);
                let redirect = match accept_invite_code(invite_id, "/t".to_string()).await {
                    Ok(resp) => resp.redirect,
                    Err(_)   => "/t".to_string(),
                };
                let nav = leptos_router::hooks::use_navigate();
                nav(&redirect, Default::default());
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
            "Complete these 4 steps to get your tenant portal ready and confirm your move-in."
        </p>
        <ul class="wiz-ctx-list">
            <li><span class="ms msf">"check_circle"</span>"E-sign your lease"</li>
            <li><span class="ms msf">"check_circle"</span>"Track move-in checklist items"</li>
            <li><span class="ms msf">"check_circle"</span>"Set rent payment and alerts"</li>
        </ul>
    });

    view! {
        <WizardShell
            steps=STEPS.to_vec()
            current_idx=current_idx
            persona_pill="Tenant"
            persona_icon="verified"
            accent_color="#6366f1"
            panel_bg="#18181b"
            ctx_headline="Welcome to your new home"
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

            // ── Step 1: Complete Profile ────────────────────────────────────
            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#4f46e5;">
                        <span class="ms" style="font-size:13px;">"person"</span>"Step 1 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Complete Your Profile"</h1>
                    <p class="wiz-s-sub">"Your landlord needs this info to prepare your unit and keep you safe during your tenancy."</p>

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
                            <div class="wiz-f"><label class="wiz-label">"Phone"</label>
                                <input class="wiz-inp" type="tel" placeholder="+1 (555) 000-0000"
                                    prop:value=move || phone.get()
                                    on:input=move |e| phone.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Date of Birth"</label>
                                <input class="wiz-inp" type="date"
                                    prop:value=move || dob.get()
                                    on:input=move |e| dob.set(event_target_value(&e))/></div>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Emergency Contact"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Full Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Contact name"
                                    prop:value=move || emerg_name.get()
                                    on:input=move |e| emerg_name.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Relationship"</label>
                                <input class="wiz-inp" type="text" placeholder="Parent, spouse, friend…"
                                    prop:value=move || emerg_rel.get()
                                    on:input=move |e| emerg_rel.set(event_target_value(&e))/></div>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Household Members"</div>
                        <div class="wiz-f"><label class="wiz-label">"Additional Occupant 1"</label>
                            <input class="wiz-inp" type="text" placeholder="Name (optional)"
                                prop:value=move || occupant.get()
                                on:input=move |e| occupant.set(event_target_value(&e))/></div>
                    </div>
                </div>
            </Show>

            // ── Step 2: Lease Review & Sign ─────────────────────────────────
            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#4f46e5;">
                        <span class="ms" style="font-size:13px;">"description"</span>"Step 2 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Lease Review & Signature"</h1>
                    <p class="wiz-s-sub">"Read your lease agreement carefully before signing. Your signed copy will be stored securely."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Lease Agreement"</div>
                        <div style="background:#f4f5f9; border:1px solid #e2e8f0; border-radius:8px; padding:20px; min-height:160px; font-size:13px; color:#64748b; line-height:1.7; margin-bottom:16px;">
                            "Your landlord will attach the full lease PDF here. Review term, rent, deposits, and house rules before signing."
                        </div>
                        <button type="button"
                            class=move || if lease_signed.get() { "wiz-oc sel" } else { "wiz-oc" }
                            style="width:100%; text-align:left;"
                            on:click=move |_| lease_signed.update(|v| *v = !*v)>
                            <span class="ms msf">"draw"</span>
                            <div class="wiz-oc-label">
                                {move || if lease_signed.get() { "Signed" } else { "Tap to e-sign" }}
                            </div>
                            <div class="wiz-oc-desc">"Electronic signature is legally binding"</div>
                        </button>
                    </div>
                </div>
            </Show>

            // ── Step 3: Move-In Checklist ───────────────────────────────────
            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#4f46e5;">
                        <span class="ms" style="font-size:13px;">"checklist"</span>"Step 3 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Move-In Checklist"</h1>
                    <p class="wiz-s-sub">"Complete these before your move-in date to ensure everything goes smoothly."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Before Move-In"</div>
                        {[
                            (chk_rent, "Pay first month's rent"),
                            (chk_deposit, "Pay security deposit"),
                            (chk_electric, "Set up electricity account"),
                            (chk_internet, "Set up internet service"),
                            (chk_insurance, "Obtain renter's insurance"),
                        ].into_iter().map(|(sig, label)| {
                            view! {
                                <div class="wiz-tr">
                                    <div class="wiz-tr-label">{label}</div>
                                    <button type="button"
                                        class=move || if sig.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                        on:click=move |_| sig.update(|v| *v = !*v)
                                    ></button>
                                </div>
                            }
                        }).collect_view()}
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Move-In Day"</div>
                        {[
                            (chk_keys, "Pick up keys from management office"),
                            (chk_condition, "Complete move-in condition report"),
                            (chk_super, "Meet building superintendent"),
                        ].into_iter().map(|(sig, label)| {
                            view! {
                                <div class="wiz-tr">
                                    <div class="wiz-tr-label">{label}</div>
                                    <button type="button"
                                        class=move || if sig.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                        on:click=move |_| sig.update(|v| *v = !*v)
                                    ></button>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </Show>

            // ── Step 4: Portal Setup ────────────────────────────────────────
            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#4f46e5;">
                        <span class="ms" style="font-size:13px;">"settings"</span>"Step 4 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Set Up Your Portal"</h1>
                    <p class="wiz-s-sub">"Configure how you want to pay rent and receive notifications."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Rent Payment Method"</div>
                        <div class="wiz-og wiz-og3">
                            <button type="button"
                                class=move || if pay_method.get() == "ach" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| pay_method.set("ach".into())>
                                <span class="ms msf">"account_balance"</span>
                                <div class="wiz-oc-label">"Bank Transfer (ACH)"</div>
                                <div class="wiz-oc-desc">"Usually free"</div>
                            </button>
                            <button type="button"
                                class=move || if pay_method.get() == "card" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| pay_method.set("card".into())>
                                <span class="ms msf">"credit_card"</span>
                                <div class="wiz-oc-label">"Credit / Debit Card"</div>
                                <div class="wiz-oc-desc">"Convenience fee may apply"</div>
                            </button>
                            <button type="button"
                                class=move || if pay_method.get() == "crypto" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| pay_method.set("crypto".into())>
                                <span class="ms msf">"currency_bitcoin"</span>
                                <div class="wiz-oc-label">"Cryptocurrency"</div>
                                <div class="wiz-oc-desc">"USDC preferred"</div>
                            </button>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Notifications"</div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Rent reminders"</div>
                                <div class="wiz-tr-desc">"Due date and late notices"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_rent.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_rent.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Maintenance updates"</div>
                                <div class="wiz-tr-desc">"Status changes on your requests"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_maint.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_maint.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Lease notices"</div>
                                <div class="wiz-tr-desc">"Renewal and policy updates"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_lease.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_lease.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Landlord messages"</div>
                                <div class="wiz-tr-desc">"Direct messages from your PM"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_msg.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_msg.update(|v| *v = !*v)
                            ></button>
                        </div>
                    </div>
                </div>
            </Show>
        </WizardShell>
    }
}
