use leptos::prelude::*;
use crate::components::wizard_shell::{ResolvedInviteCode, WizardShell, WizardStepDesc, resolve_invite_code};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc { id: "profile",      label: "Owner Profile",     skippable: false },
    WizardStepDesc { id: "preferences",  label: "Preferences",       skippable: false },
    WizardStepDesc { id: "confirm",      label: "Activate Portal",   skippable: false },
];

#[component]
pub fn OwnerWizard() -> impl IntoView {
    let query    = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());
    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));
    Effect::new(move |_| { if let Some(Ok(r)) = code_resource.get() { invite_sig.set(r); } });

    let current_idx = RwSignal::new(0usize);
    let total       = STEPS.len();
    let is_last     = Signal::derive(move || current_idx.get() == total - 1);
    let next_label  = Signal::derive(move || if is_last.get() { "Activate Owner Portal" } else { "Continue" });

    let first  = RwSignal::new(String::new());
    let last   = RwSignal::new(String::new());
    let email  = RwSignal::new(String::new());

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx + 1 >= total {
            let invite_id = invite_sig.get().map(|c| c.code.clone()).unwrap_or_default();
            leptos::task::spawn_local(async move {
                let nav = leptos_router::hooks::use_navigate();
                match accept_invite_code(invite_id, "/o".to_string()).await {
                    Ok(resp) => nav(&resp.redirect, Default::default()),
                    Err(_)   => nav("/o", Default::default()),
                }
            });
        } else { current_idx.set(idx + 1); }
    });
    let on_prev = Callback::new(move |_| { let i = current_idx.get(); if i > 0 { current_idx.set(i - 1); } });

    let ctx_body = ViewFn::from(|| view! {
        <p class="wiz-ctx-p">"Your owner portal gives you read-only visibility into your portfolio managed by your property manager."</p>
        <ul class="wiz-ctx-list">
            <li><span class="ms msf">"check_circle"</span>"Monthly owner statements"</li>
            <li><span class="ms msf">"check_circle"</span>"Real-time occupancy &amp; income"</li>
            <li><span class="ms msf">"check_circle"</span>"Maintenance approval requests"</li>
        </ul>
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx persona_pill="Owner"
            persona_icon="account_balance" accent_color="#7c3aed" panel_bg="#140f20"
            ctx_headline="Activate your owner portal" ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev is_last_step=is_last next_label=next_label>

            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"person"</span>"Step 1 of 3"
                    </div>
                    <h1 class="wiz-s-title">"Owner Profile"</h1>
                    <p class="wiz-s-sub">"Confirm your identity so your property manager can link your portfolio."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Personal Info"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Robert"
                                    prop:value=move || first.get() on:input=move |e| first.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Chen"
                                    prop:value=move || last.get() on:input=move |e| last.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"Email"</label>
                            <input class="wiz-inp" type="email" placeholder="robert@rchholdings.com"
                                prop:value=move || email.get() on:input=move |e| email.set(event_target_value(&e))/></div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"tune"</span>"Step 2 of 3"
                    </div>
                    <h1 class="wiz-s-title">"Statement Preferences"</h1>
                    <p class="wiz-s-sub">"How would you like to receive your monthly owner statements?"</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Delivery"</div>
                        <div class="wiz-f"><label class="wiz-label">"Delivery Method"</label>
                            <select class="wiz-inp"><option>"Email PDF"</option><option>"Portal only"</option><option>"Both"</option></select></div>
                        <div class="wiz-f"><label class="wiz-label">"Preferred Send Date"</label>
                            <select class="wiz-inp"><option>"1st of month"</option><option>"5th of month"</option><option>"10th of month"</option></select></div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim" style="text-align:center; padding:40px 0;">
                    <div style="width:72px; height:72px; background:rgba(124,58,237,.1); border:2px solid rgba(124,58,237,.3); border-radius:50%; display:flex; align-items:center; justify-content:center; margin:0 auto 20px;">
                        <span class="ms msf" style="font-size:34px; color:#7c3aed;">"account_balance"</span>
                    </div>
                    <h1 class="wiz-s-title" style="text-align:center;">"Portal Ready"</h1>
                    <p style="font-size:14px; color:#64748b; line-height:1.7; max-width:400px; margin:0 auto;">
                        "Your owner portal is ready. You'll receive monthly statements and can approve maintenance requests over your threshold."
                    </p>
                </div>
            </Show>
        </WizardShell>
    }
}
