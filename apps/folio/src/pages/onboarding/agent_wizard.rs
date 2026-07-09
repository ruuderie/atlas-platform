use leptos::prelude::*;
use crate::components::wizard_shell::{ResolvedInviteCode, WizardShell, WizardStepDesc, resolve_invite_code};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc { id: "profile",  label: "Agent Profile",    skippable: false },
    WizardStepDesc { id: "license",  label: "License & MLS",    skippable: false },
    WizardStepDesc { id: "brokerage",label: "Brokerage Link",   skippable: false },
    WizardStepDesc { id: "confirm",  label: "Confirm",          skippable: false },
];

#[component]
pub fn AgentWizard() -> impl IntoView {
    let query    = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());
    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));
    Effect::new(move |_| { if let Some(Ok(r)) = code_resource.get() { invite_sig.set(r); } });

    let current_idx = RwSignal::new(0usize);
    let total       = STEPS.len();
    let is_last     = Signal::derive(move || current_idx.get() == total - 1);
    let next_label  = Signal::derive(move || if is_last.get() { "Join Brokerage" } else { "Continue" });

    let first   = RwSignal::new(String::new());
    let last    = RwSignal::new(String::new());
    let license = RwSignal::new(String::new());
    let state   = RwSignal::new("FL".to_string());

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx + 1 >= total {
            let invite_id = invite_sig.get().map(|c| c.code.clone()).unwrap_or_default();
            leptos::task::spawn_local(async move {
                let nav = leptos_router::hooks::use_navigate();
                match accept_invite_code(invite_id, "/l".to_string()).await {
                    Ok(resp) => nav(&resp.redirect, Default::default()),
                    Err(_)   => nav("/l", Default::default()),
                }
            });
        } else { current_idx.set(idx + 1); }
    });
    let on_prev = Callback::new(move |_| { let i = current_idx.get(); if i > 0 { current_idx.set(i - 1); } });

    let ctx_body = ViewFn::from(|| view! {
        <p class="wiz-ctx-p">"Join your broker's Folio workspace to manage client relationships, listings, and commission splits."</p>
        <ul class="wiz-ctx-list">
            <li><span class="ms msf">"check_circle"</span>"Commission tracking &amp; splits"</li>
            <li><span class="ms msf">"check_circle"</span>"Shared listing network access"</li>
            <li><span class="ms msf">"check_circle"</span>"CRM &amp; lead management"</li>
        </ul>
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx persona_pill="Agent"
            persona_icon="real_estate_agent" accent_color="#059669" panel_bg="#061910"
            ctx_headline="Join your brokerage workspace" ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev is_last_step=is_last next_label=next_label>

            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(5,150,105,.08); color:#047857;">
                        <span class="ms" style="font-size:13px;">"person"</span>"Step 1 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Agent Profile"</h1>
                    <p class="wiz-s-sub">"Your profile is visible to clients and within your brokerage."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Personal Info"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Marcus"
                                    prop:value=move || first.get() on:input=move |e| first.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Williams"
                                    prop:value=move || last.get() on:input=move |e| last.set(event_target_value(&e))/></div>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(5,150,105,.08); color:#047857;">
                        <span class="ms" style="font-size:13px;">"badge"</span>"Step 2 of 4"
                    </div>
                    <h1 class="wiz-s-title">"License &amp; MLS"</h1>
                    <p class="wiz-s-sub">"Your license is verified against state records before you can list properties."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"License Details"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"License Number"</label>
                                <input class="wiz-inp" type="text" placeholder="SL3000000"
                                    prop:value=move || license.get() on:input=move |e| license.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"State"</label>
                                <select class="wiz-inp" prop:value=move || state.get() on:change=move |e| state.set(event_target_value(&e))>
                                    <option>"FL"</option><option>"NY"</option><option>"CA"</option><option>"TX"</option>
                                </select></div>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(5,150,105,.08); color:#047857;">
                        <span class="ms" style="font-size:13px;">"corporate_fare"</span>"Step 3 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Brokerage Link"</h1>
                    <p class="wiz-s-sub">"Confirm your brokerage association and commission tier."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Brokerage Details"</div>
                        <p style="font-size:14px; color:#64748b;">"Your broker has pre-configured your commission split. Review and accept below."</p>
                        <div style="margin-top:14px; padding:14px; background:#f4f6fb; border-radius:8px; font-size:14px;">
                            <div style="display:flex; justify-content:space-between;">
                                <span style="color:#64748b;">"Commission Split"</span>
                                <strong>"70% Agent / 30% Broker"</strong>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim" style="text-align:center; padding:40px 0;">
                    <div style="width:72px; height:72px; background:rgba(5,150,105,.1); border:2px solid rgba(5,150,105,.3); border-radius:50%; display:flex; align-items:center; justify-content:center; margin:0 auto 20px;">
                        <span class="ms msf" style="font-size:34px; color:#059669;">"real_estate_agent"</span>
                    </div>
                    <h1 class="wiz-s-title" style="text-align:center;">"Welcome to the Brokerage"</h1>
                    <p style="font-size:14px; color:#64748b; line-height:1.7; max-width:400px; margin:0 auto;">
                        "You're confirmed. Your broker will be notified and your workspace will be ready within minutes."
                    </p>
                </div>
            </Show>
        </WizardShell>
    }
}
