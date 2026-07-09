use leptos::prelude::*;
use crate::components::wizard_shell::{ResolvedInviteCode, WizardShell, WizardStepDesc, resolve_invite_code};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc { id: "profile",      label: "Your Profile",       skippable: false },
    WizardStepDesc { id: "experience",   label: "STR Experience",     skippable: false },
    WizardStepDesc { id: "terms",        label: "Agreement & Terms",  skippable: false },
    WizardStepDesc { id: "confirm",      label: "Confirm",            skippable: false },
];

#[component]
pub fn CohostWizard() -> impl IntoView {
    let query    = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());
    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));
    Effect::new(move |_| { if let Some(Ok(r)) = code_resource.get() { invite_sig.set(r); } });

    let current_idx = RwSignal::new(0usize);
    let total       = STEPS.len();
    let is_last     = Signal::derive(move || current_idx.get() == total - 1);
    let next_label  = Signal::derive(move || if is_last.get() { "Accept Co-host Role" } else { "Continue" });

    let first  = RwSignal::new(String::new());
    let last   = RwSignal::new(String::new());
    let bio    = RwSignal::new(String::new());

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx + 1 >= total {
            // Accept invite code (provisions G-32 role) then navigate.
            // No-ops cleanly if no invite code was present.
            let invite_id = invite_sig.get().map(|c| c.code.clone()).unwrap_or_default();
            leptos::task::spawn_local(async move {
                let nav = leptos_router::hooks::use_navigate();
                match accept_invite_code(invite_id, "/s".to_string()).await {
                    Ok(resp) => nav(&resp.redirect, Default::default()),
                    Err(_)   => nav("/s", Default::default()),
                }
            });
        } else { current_idx.set(idx + 1); }
    });
    let on_prev = Callback::new(move |_| { let i = current_idx.get(); if i > 0 { current_idx.set(i - 1); } });

    let ctx_body = ViewFn::from(|| view! {
        <p class="wiz-ctx-p">"Join as a co-host and earn a share of STR revenue from properties you help manage."</p>
        <ul class="wiz-ctx-list">
            <li><span class="ms msf">"check_circle"</span>"Co-manage STR properties"</li>
            <li><span class="ms msf">"check_circle"</span>"Revenue share automatically calculated"</li>
            <li><span class="ms msf">"check_circle"</span>"Guest messaging and calendar access"</li>
        </ul>
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx persona_pill="Co-host"
            persona_icon="supervisor_account" accent_color="#0891b2" panel_bg="#0c1820"
            ctx_headline="Join as a co-host" ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev is_last_step=is_last next_label=next_label>

            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(8,145,178,.08); color:#0e7490;">
                        <span class="ms" style="font-size:13px;">"person"</span>"Step 1 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Your Profile"</h1>
                    <p class="wiz-s-sub">"Your co-host profile is visible to property owners who work with you."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Personal Info"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Alex"
                                    prop:value=move || first.get() on:input=move |e| first.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Torres"
                                    prop:value=move || last.get() on:input=move |e| last.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"Bio"</label>
                            <textarea class="wiz-inp" rows="3" placeholder="Tell property owners about your experience..."
                                prop:value=move || bio.get() on:input=move |e| bio.set(event_target_value(&e))></textarea></div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(8,145,178,.08); color:#0e7490;">
                        <span class="ms" style="font-size:13px;">"star"</span>"Step 2 of 4"
                    </div>
                    <h1 class="wiz-s-title">"STR Experience"</h1>
                    <p class="wiz-s-sub">"Tell us about your short-term rental management background."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Experience"</div>
                        <div class="wiz-f"><label class="wiz-label">"Years of STR Experience"</label>
                            <select class="wiz-inp"><option>"Less than 1 year"</option><option>"1–2 years"</option>
                                <option>"3–5 years"</option><option>"5+ years"</option></select></div>
                        <div class="wiz-f"><label class="wiz-label">"Properties Currently Managed"</label>
                            <select class="wiz-inp"><option>"1–3"</option><option>"4–10"</option>
                                <option>"11–25"</option><option>"25+"</option></select></div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(8,145,178,.08); color:#0e7490;">
                        <span class="ms" style="font-size:13px;">"handshake"</span>"Step 3 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Agreement &amp; Terms"</h1>
                    <p class="wiz-s-sub">"Review the co-host agreement before accepting your invitation."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Co-host Agreement"</div>
                        <label style="display:flex; align-items:center; gap:10px; font-size:14px; cursor:pointer; margin-bottom:10px;">
                            <input type="checkbox" style="accent-color:#0891b2; width:18px; height:18px;"/>
                            "I agree to the co-host terms of service"
                        </label>
                        <label style="display:flex; align-items:center; gap:10px; font-size:14px; cursor:pointer;">
                            <input type="checkbox" style="accent-color:#0891b2; width:18px; height:18px;"/>
                            "I understand the revenue split terms"
                        </label>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim" style="text-align:center; padding:40px 0;">
                    <div style="width:72px; height:72px; background:rgba(8,145,178,.1); border:2px solid rgba(8,145,178,.3); border-radius:50%; display:flex; align-items:center; justify-content:center; margin:0 auto 20px;">
                        <span class="ms msf" style="font-size:34px; color:#0891b2;">"supervisor_account"</span>
                    </div>
                    <h1 class="wiz-s-title" style="text-align:center;">"Welcome to the Team"</h1>
                    <p style="font-size:14px; color:#64748b; line-height:1.7; max-width:400px; margin:0 auto 24px;">
                        "You're all set. Accept to complete your co-host onboarding and access your property calendar."
                    </p>
                    <div class="wiz-card" style="text-align:left;">
                        <div class="wiz-ct">"Your Co-host Dashboard"</div>
                        <p style="font-size:14px; color:#64748b;">"Calendars, guest messaging, pricing rules, and revenue reports — all in one place."</p>
                    </div>
                </div>
            </Show>
        </WizardShell>
    }
}
