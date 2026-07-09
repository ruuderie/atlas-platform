use leptos::prelude::*;
use crate::components::wizard_shell::{ResolvedInviteCode, WizardShell, WizardStepDesc, resolve_invite_code};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc { id: "company",    label: "Company Profile",   skippable: false },
    WizardStepDesc { id: "license",    label: "Broker License",    skippable: false },
    WizardStepDesc { id: "team",       label: "Agent Roster",      skippable: true  },
    WizardStepDesc { id: "commission", label: "Commission Plans",  skippable: false },
    WizardStepDesc { id: "confirm",    label: "Go Live",           skippable: false },
];

#[component]
pub fn BrokerWizard() -> impl IntoView {
    let query    = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());
    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));
    Effect::new(move |_| { if let Some(Ok(r)) = code_resource.get() { invite_sig.set(r); } });

    let current_idx = RwSignal::new(0usize);
    let total       = STEPS.len();
    let is_last     = Signal::derive(move || current_idx.get() == total - 1);
    let next_label  = Signal::derive(move || if is_last.get() { "Launch Brokerage Workspace" } else { "Continue" });

    let company  = RwSignal::new(String::new());
    let broker_license = RwSignal::new(String::new());
    let state    = RwSignal::new("FL".to_string());
    let split    = RwSignal::new("70".to_string());

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
        <p class="wiz-ctx-p">"Configure your brokerage workspace to manage agents, listings, and commission structures from one platform."</p>
        <ul class="wiz-ctx-list">
            <li><span class="ms msf">"check_circle"</span>"Agent roster &amp; invite management"</li>
            <li><span class="ms msf">"check_circle"</span>"Customizable commission plans"</li>
            <li><span class="ms msf">"check_circle"</span>"MLS &amp; listing network integration"</li>
            <li><span class="ms msf">"check_circle"</span>"Deal pipeline &amp; analytics"</li>
        </ul>
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx persona_pill="Broker"
            persona_icon="gavel" accent_color="#7c3aed" panel_bg="#0d0b1a"
            ctx_headline="Set up your brokerage" ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev is_last_step=is_last next_label=next_label>

            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"corporate_fare"</span>"Step 1 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Company Profile"</h1>
                    <p class="wiz-s-sub">"Your brokerage profile appears on listings, agent portals, and MLS submissions."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Company Info"</div>
                        <div class="wiz-f"><label class="wiz-label">"Brokerage Name"</label>
                            <input class="wiz-inp" type="text" placeholder="Apex Realty Group LLC"
                                prop:value=move || company.get() on:input=move |e| company.set(event_target_value(&e))/></div>
                        <div class="wiz-f"><label class="wiz-label">"Office Address"</label>
                            <input class="wiz-inp" type="text" placeholder="1000 Brickell Ave, Miami, FL 33131"/></div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"badge"</span>"Step 2 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Broker License"</h1>
                    <p class="wiz-s-sub">"Your broker license is required to activate agent roster and listing features."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"License Details"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Broker License #"</label>
                                <input class="wiz-inp" type="text" placeholder="BK3000000"
                                    prop:value=move || broker_license.get() on:input=move |e| broker_license.set(event_target_value(&e))/></div>
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
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"group_add"</span>"Step 3 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Agent Roster"</h1>
                    <p class="wiz-s-sub">"Invite your agents. Each gets their own workspace linked to your brokerage."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Invite Agents"</div>
                        <div class="wiz-f"><label class="wiz-label">"Agent Email"</label>
                            <input class="wiz-inp" type="email" placeholder="agent@email.com"/></div>
                        <p style="font-size:12px; color:#94a3b8; margin-top:4px;">"Or generate an invite code for bulk agent onboarding on the next step."</p>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"percent"</span>"Step 4 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Commission Plans"</h1>
                    <p class="wiz-s-sub">"Set your default agent commission split. You can create multiple tiers later."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Default Split"</div>
                        <div class="wiz-f"><label class="wiz-label">"Agent Share (%)"</label>
                            <select class="wiz-inp" prop:value=move || split.get() on:change=move |e| split.set(event_target_value(&e))>
                                <option value="60">"60% Agent / 40% Broker"</option>
                                <option value="70" selected>"70% Agent / 30% Broker"</option>
                                <option value="80">"80% Agent / 20% Broker"</option>
                                <option value="90">"90% Agent / 10% Broker"</option>
                            </select></div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 4>
                <div class="wiz-anim" style="text-align:center; padding:40px 0;">
                    <div style="width:72px; height:72px; background:rgba(124,58,237,.1); border:2px solid rgba(124,58,237,.3); border-radius:50%; display:flex; align-items:center; justify-content:center; margin:0 auto 20px;">
                        <span class="ms msf" style="font-size:34px; color:#7c3aed;">"gavel"</span>
                    </div>
                    <h1 class="wiz-s-title" style="text-align:center;">"Brokerage Ready"</h1>
                    <p style="font-size:14px; color:#64748b; line-height:1.7; max-width:400px; margin:0 auto;">
                        {move || format!("{} — your brokerage workspace is configured and ready to launch.", company.get())}
                    </p>
                </div>
            </Show>
        </WizardShell>
    }
}
