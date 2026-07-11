// apps/folio/src/pages/onboarding/broker_wizard.rs
//
// BrokerWizard — /onboard/broker
//
// 5 steps mirroring wiz_broker_onboard/code.html:
//   1. Broker Profile & License
//   2. Brokerage Details
//   3. Compliance Documents
//   4. Agent Roster
//   5. Commission Plans

use crate::components::wizard_shell::{
    resolve_invite_code, ResolvedInviteCode, WizardShell, WizardStepDesc,
};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;
use leptos::prelude::*;

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc {
        id: "profile",
        label: "Broker Profile & License",
        skippable: false,
    },
    WizardStepDesc {
        id: "brokerage",
        label: "Brokerage Details",
        skippable: false,
    },
    WizardStepDesc {
        id: "compliance",
        label: "Compliance Docs",
        skippable: true,
    },
    WizardStepDesc {
        id: "roster",
        label: "Agent Roster",
        skippable: true,
    },
    WizardStepDesc {
        id: "commission",
        label: "Commission Plans",
        skippable: false,
    },
];

#[component]
pub fn BrokerWizard() -> impl IntoView {
    let query = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());
    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));
    Effect::new(move |_| {
        if let Some(Ok(r)) = code_resource.get() {
            invite_sig.set(r);
        }
    });

    let current_idx = RwSignal::new(0usize);
    let total = STEPS.len();
    let is_last = Signal::derive(move || current_idx.get() == total - 1);
    let next_label = Signal::derive(move || {
        if is_last.get() {
            "Launch Brokerage"
        } else {
            "Continue"
        }
    });

    // Broker profile
    let first = RwSignal::new(String::new());
    let last = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let broker_license = RwSignal::new(String::new());
    let state = RwSignal::new("FL".to_string());
    let issued = RwSignal::new(String::new());
    let expiry = RwSignal::new(String::new());
    let addl_states = RwSignal::new(String::new());

    // Brokerage details
    let legal_name = RwSignal::new(String::new());
    let dba = RwSignal::new(String::new());
    let ein = RwSignal::new(String::new());
    let website = RwSignal::new(String::new());
    let office_street = RwSignal::new(String::new());
    let office_city = RwSignal::new(String::new());
    let office_zip = RwSignal::new(String::new());
    let office_phone = RwSignal::new(String::new());
    let mls_name = RwSignal::new(String::new());
    let mls_id = RwSignal::new(String::new());

    // Roster
    let agent_email = RwSignal::new(String::new());

    // Commission
    let new_agent_split = RwSignal::new("60".to_string());
    let associate_split = RwSignal::new("70".to_string());
    let senior_split = RwSignal::new("80".to_string());
    let top_producer_split = RwSignal::new("90".to_string());
    let txn_fee = RwSignal::new("295".to_string());

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx + 1 >= total {
            let invite_id = invite_sig.get().map(|c| c.code.clone()).unwrap_or_default();
            leptos::task::spawn_local(async move {
                let nav = leptos_router::hooks::use_navigate();
                match accept_invite_code(invite_id, "/l".to_string()).await {
                    Ok(resp) => nav(&resp.redirect, Default::default()),
                    Err(_) => nav("/l", Default::default()),
                }
            });
        } else {
            current_idx.set(idx + 1);
        }
    });
    let on_prev = Callback::new(move |_| {
        let i = current_idx.get();
        if i > 0 {
            current_idx.set(i - 1);
        }
    });

    let ctx_body = ViewFn::from(|| {
        view! {
            <p class="wiz-ctx-p">"Configure your brokerage workspace to manage agents, listings, and commission structures from one platform."</p>
            <ul class="wiz-ctx-list">
                <li><span class="ms msf">"check_circle"</span>"Agent roster & invite management"</li>
                <li><span class="ms msf">"check_circle"</span>"Customizable commission plans"</li>
                <li><span class="ms msf">"check_circle"</span>"MLS & listing network integration"</li>
                <li><span class="ms msf">"check_circle"</span>"Deal pipeline & analytics"</li>
            </ul>
        }
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx persona_pill="Broker"
            persona_icon="gavel" accent_color="#7c3aed" panel_bg="#0d0b1a"
            ctx_headline="Set up your Brokerage" ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev is_last_step=is_last next_label=next_label>

            // ── Step 1: Broker Profile & License ────────────────────────────
            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"gavel"</span>"Step 1 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Broker Profile & License"</h1>
                    <p class="wiz-s-sub">"Your personal broker profile and state license information."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Broker Info"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Diana"
                                    prop:value=move || first.get()
                                    on:input=move |e| first.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Park"
                                    prop:value=move || last.get()
                                    on:input=move |e| last.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Email"</label>
                                <input class="wiz-inp" type="email" placeholder="diana@apexrealty.com"
                                    prop:value=move || email.get()
                                    on:input=move |e| email.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Phone"</label>
                                <input class="wiz-inp" type="tel" placeholder="+1 (305) 000-0000"
                                    prop:value=move || phone.get()
                                    on:input=move |e| phone.set(event_target_value(&e))/></div>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Broker License"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Broker License #"</label>
                                <input class="wiz-inp" type="text" placeholder="BK3000000"
                                    prop:value=move || broker_license.get()
                                    on:input=move |e| broker_license.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Primary State"</label>
                                <select class="wiz-inp"
                                    prop:value=move || state.get()
                                    on:change=move |e| state.set(event_target_value(&e))>
                                    <option value="FL">"FL"</option>
                                    <option value="NY">"NY"</option>
                                    <option value="CA">"CA"</option>
                                    <option value="TX">"TX"</option>
                                </select></div>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"License Issued"</label>
                                <input class="wiz-inp" type="date"
                                    prop:value=move || issued.get()
                                    on:input=move |e| issued.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"License Expiry"</label>
                                <input class="wiz-inp" type="date"
                                    prop:value=move || expiry.get()
                                    on:input=move |e| expiry.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"Additional Licensed States"</label>
                            <input class="wiz-inp" type="text" placeholder="e.g. NY, GA"
                                prop:value=move || addl_states.get()
                                on:input=move |e| addl_states.set(event_target_value(&e))/></div>
                    </div>
                </div>
            </Show>

            // ── Step 2: Brokerage Details ───────────────────────────────────
            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"corporate_fare"</span>"Step 2 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Brokerage Details"</h1>
                    <p class="wiz-s-sub">"Information about your brokerage entity — this appears on all official documents and agent materials."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Brokerage Identity"</div>
                        <div class="wiz-f"><label class="wiz-label">"Brokerage Legal Name"</label>
                            <input class="wiz-inp" type="text" placeholder="Apex Realty Group LLC"
                                prop:value=move || legal_name.get()
                                on:input=move |e| legal_name.set(event_target_value(&e))/></div>
                        <div class="wiz-f"><label class="wiz-label">"DBA / Trading Name (if different)"</label>
                            <input class="wiz-inp" type="text" placeholder="Apex Realty"
                                prop:value=move || dba.get()
                                on:input=move |e| dba.set(event_target_value(&e))/></div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Federal Tax ID (EIN)"</label>
                                <input class="wiz-inp" type="text" placeholder="XX-XXXXXXX"
                                    prop:value=move || ein.get()
                                    on:input=move |e| ein.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Brokerage Website"</label>
                                <input class="wiz-inp" type="url" placeholder="https://apexrealty.com"
                                    prop:value=move || website.get()
                                    on:input=move |e| website.set(event_target_value(&e))/></div>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Primary Office"</div>
                        <div class="wiz-f"><label class="wiz-label">"Street Address"</label>
                            <input class="wiz-inp" type="text" placeholder="1000 Brickell Ave"
                                prop:value=move || office_street.get()
                                on:input=move |e| office_street.set(event_target_value(&e))/></div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"City"</label>
                                <input class="wiz-inp" type="text" placeholder="Miami"
                                    prop:value=move || office_city.get()
                                    on:input=move |e| office_city.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"State / ZIP"</label>
                                <input class="wiz-inp" type="text" placeholder="FL 33131"
                                    prop:value=move || office_zip.get()
                                    on:input=move |e| office_zip.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"Office Phone"</label>
                            <input class="wiz-inp" type="tel" placeholder="+1 (305) 555-0100"
                                prop:value=move || office_phone.get()
                                on:input=move |e| office_phone.set(event_target_value(&e))/></div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"MLS Membership"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"MLS Name"</label>
                                <input class="wiz-inp" type="text" placeholder="MIAMI MLS"
                                    prop:value=move || mls_name.get()
                                    on:input=move |e| mls_name.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"MLS Broker ID"</label>
                                <input class="wiz-inp" type="text" placeholder="BR-000000"
                                    prop:value=move || mls_id.get()
                                    on:input=move |e| mls_id.set(event_target_value(&e))/></div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 3: Compliance Documents ───────────────────────────────
            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"folder_managed"</span>"Step 3 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Compliance Documents"</h1>
                    <p class="wiz-s-sub">"Upload required compliance documents to activate your brokerage workspace."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Required Uploads"</div>
                        {[
                            ("badge", "Broker License Certificate"),
                            ("verified_user", "E&O Insurance Certificate"),
                            ("description", "Articles of Organization / Incorporation"),
                            ("menu_book", "Office Policy Manual"),
                        ].into_iter().map(|(icon, label)| view! {
                            <div class="wiz-na-row">
                                <span class="ms msf" style="font-size:24px; color:#7c3aed;">{icon}</span>
                                <div style="flex:1;">
                                    <div style="font-size:14px; font-weight:600;">{label}</div>
                                    <div style="font-size:12px; color:#64748b;">"Upload from your dashboard after launch if needed"</div>
                                </div>
                                <span style="font-size:11px; font-weight:700; color:#94a3b8; text-transform:uppercase;">"Optional now"</span>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            </Show>

            // ── Step 4: Agent Roster ────────────────────────────────────────
            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"group_add"</span>"Step 4 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Agent Roster"</h1>
                    <p class="wiz-s-sub">"Invite your existing agents to Folio. You can also add more after setup."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Invite More Agents"</div>
                        <div class="wiz-f"><label class="wiz-label">"Agent Email"</label>
                            <input class="wiz-inp" type="email" placeholder="agent@email.com"
                                prop:value=move || agent_email.get()
                                on:input=move |e| agent_email.set(event_target_value(&e))/></div>
                        <p style="font-size:12px; color:#94a3b8; margin-top:4px;">
                            "Invites go out after you launch. Agents complete the Agent onboarding wizard."
                        </p>
                    </div>
                </div>
            </Show>

            // ── Step 5: Commission Plans ────────────────────────────────────
            <Show when=move || current_idx.get() == 4>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(124,58,237,.08); color:#6d28d9;">
                        <span class="ms" style="font-size:13px;">"percent"</span>"Step 5 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Commission Plans"</h1>
                    <p class="wiz-s-sub">"Set up default commission split tiers for your agents. Individual overrides can be applied per agent."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Default Commission Structure"</div>
                        <div class="wiz-f"><label class="wiz-label">"New Agent (0–12 months)"</label>
                            <select class="wiz-inp"
                                prop:value=move || new_agent_split.get()
                                on:change=move |e| new_agent_split.set(event_target_value(&e))>
                                <option value="50">"50% Agent / 50% Broker"</option>
                                <option value="60">"60% Agent / 40% Broker"</option>
                                <option value="70">"70% Agent / 30% Broker"</option>
                            </select></div>
                        <div class="wiz-f"><label class="wiz-label">"Associate Agent (1–3 years)"</label>
                            <select class="wiz-inp"
                                prop:value=move || associate_split.get()
                                on:change=move |e| associate_split.set(event_target_value(&e))>
                                <option value="60">"60% Agent / 40% Broker"</option>
                                <option value="70">"70% Agent / 30% Broker"</option>
                                <option value="80">"80% Agent / 20% Broker"</option>
                            </select></div>
                        <div class="wiz-f"><label class="wiz-label">"Senior Agent (3+ years)"</label>
                            <select class="wiz-inp"
                                prop:value=move || senior_split.get()
                                on:change=move |e| senior_split.set(event_target_value(&e))>
                                <option value="70">"70% Agent / 30% Broker"</option>
                                <option value="80">"80% Agent / 20% Broker"</option>
                                <option value="90">"90% Agent / 10% Broker"</option>
                            </select></div>
                        <div class="wiz-f"><label class="wiz-label">"Top Producer (100+ units/yr)"</label>
                            <select class="wiz-inp"
                                prop:value=move || top_producer_split.get()
                                on:change=move |e| top_producer_split.set(event_target_value(&e))>
                                <option value="80">"80% Agent / 20% Broker"</option>
                                <option value="90">"90% Agent / 10% Broker"</option>
                                <option value="95">"95% Agent / 5% Broker"</option>
                            </select></div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Transaction Fee"</div>
                        <div class="wiz-f"><label class="wiz-label">"Per-Transaction Fee"</label>
                            <input class="wiz-inp" type="text" placeholder="$295"
                                prop:value=move || txn_fee.get()
                                on:input=move |e| txn_fee.set(event_target_value(&e))/></div>
                        <div class="wiz-f"><label class="wiz-label">"Applies To"</label>
                            <select class="wiz-inp">
                                <option>"All closed transactions"</option>
                                <option>"Buy-side only"</option>
                                <option>"Sell-side only"</option>
                            </select></div>
                    </div>
                </div>
            </Show>
        </WizardShell>
    }
}
