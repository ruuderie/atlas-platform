// apps/folio/src/pages/onboarding/agent_wizard.rs
//
// AgentWizard — /onboard/agent
//
// 4 steps mirroring wiz_agent_onboard/code.html:
//   1. Profile & License
//   2. Specialties & Markets
//   3. Brokerage Assignment
//   4. Tools & Notifications

use crate::components::wizard_shell::{
    resolve_invite_code, ResolvedInviteCode, WizardShell, WizardStepDesc,
};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;
use leptos::prelude::*;

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc {
        id: "profile",
        label: "Profile & License",
        skippable: false,
    },
    WizardStepDesc {
        id: "specialties",
        label: "Specialties & Markets",
        skippable: false,
    },
    WizardStepDesc {
        id: "brokerage",
        label: "Brokerage Assignment",
        skippable: false,
    },
    WizardStepDesc {
        id: "tools",
        label: "Tools & Notifications",
        skippable: false,
    },
];

const TX_TYPES: &[(&str, &str)] = &[
    ("res_buy", "Residential Buy"),
    ("res_sell", "Residential Sell"),
    ("commercial", "Commercial"),
    ("rentals", "Rentals / Leasing"),
    ("luxury", "Luxury"),
    ("investment", "Investment / Wholesaling"),
    ("pm", "Property Management"),
    ("new_const", "New Construction"),
];

const LANGUAGES: &[&str] = &["English", "Spanish", "Portuguese", "French", "Mandarin"];

#[component]
pub fn AgentWizard() -> impl IntoView {
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
            "Go to Agent Dashboard"
        } else {
            "Continue"
        }
    });

    // Profile & license
    let first = RwSignal::new(String::new());
    let last = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let license = RwSignal::new(String::new());
    let state = RwSignal::new("FL".to_string());
    let expiry = RwSignal::new(String::new());

    // Specialties
    let tx_sel: RwSignal<std::collections::HashSet<&'static str>> =
        RwSignal::new(std::collections::HashSet::new());
    let lang_sel: RwSignal<std::collections::HashSet<&'static str>> =
        RwSignal::new(std::collections::HashSet::from(["English"]));
    let market1 = RwSignal::new(String::new());
    let market2 = RwSignal::new(String::new());
    let price_range = RwSignal::new("any".to_string());

    // Notifications
    let notify_leads = RwSignal::new(true);
    let notify_deals = RwSignal::new(true);
    let notify_listing = RwSignal::new(true);
    let notify_comm = RwSignal::new(false);

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
            <p class="wiz-ctx-p">"Join your broker's Folio workspace to manage client relationships, listings, and commission splits."</p>
            <ul class="wiz-ctx-list">
                <li><span class="ms msf">"check_circle"</span>"Commission tracking & splits"</li>
                <li><span class="ms msf">"check_circle"</span>"Shared listing network access"</li>
                <li><span class="ms msf">"check_circle"</span>"CRM & lead management"</li>
            </ul>
        }
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx persona_pill="Agent"
            persona_icon="real_estate_agent" accent_color="#059669" panel_bg="#061910"
            ctx_headline="Set up your Agent profile" ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev is_last_step=is_last next_label=next_label>

            // ── Step 1: Profile & License ───────────────────────────────────
            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(5,150,105,.08); color:#047857;">
                        <span class="ms" style="font-size:13px;">"badge"</span>"Step 1 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Profile & License"</h1>
                    <p class="wiz-s-sub">"Your license info is required by your broker and stored securely for compliance."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Agent Info"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Marcus"
                                    prop:value=move || first.get()
                                    on:input=move |e| first.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Williams"
                                    prop:value=move || last.get()
                                    on:input=move |e| last.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Phone (Client-facing)"</label>
                                <input class="wiz-inp" type="tel" placeholder="+1 (555) 000-0000"
                                    prop:value=move || phone.get()
                                    on:input=move |e| phone.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Business Email"</label>
                                <input class="wiz-inp" type="email" placeholder="marcus@apexrealty.com"
                                    prop:value=move || email.get()
                                    on:input=move |e| email.set(event_target_value(&e))/></div>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"License Details"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"License Number"</label>
                                <input class="wiz-inp" type="text" placeholder="SL3000000"
                                    prop:value=move || license.get()
                                    on:input=move |e| license.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"License State"</label>
                                <select class="wiz-inp"
                                    prop:value=move || state.get()
                                    on:change=move |e| state.set(event_target_value(&e))>
                                    <option value="FL">"FL"</option>
                                    <option value="NY">"NY"</option>
                                    <option value="CA">"CA"</option>
                                    <option value="TX">"TX"</option>
                                </select></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"License Expiry"</label>
                            <input class="wiz-inp" type="date"
                                prop:value=move || expiry.get()
                                on:input=move |e| expiry.set(event_target_value(&e))/></div>
                    </div>
                </div>
            </Show>

            // ── Step 2: Specialties & Markets ───────────────────────────────
            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(5,150,105,.08); color:#047857;">
                        <span class="ms" style="font-size:13px;">"sell"</span>"Step 2 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Specialties & Markets"</h1>
                    <p class="wiz-s-sub">"Help clients and your broker know where you specialize."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Transaction Types"</div>
                        <div class="wiz-og wiz-og3">
                            {TX_TYPES.iter().map(|(id, label)| {
                                let cid = *id;
                                let lbl = *label;
                                view! {
                                    <button type="button"
                                        class=move || if tx_sel.get().contains(cid) { "wiz-oc sel" } else { "wiz-oc" }
                                        on:click=move |_| tx_sel.update(|s| {
                                            if s.contains(cid) { s.remove(cid); } else { s.insert(cid); }
                                        })>
                                        <div class="wiz-oc-label">{lbl}</div>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Primary Markets"</div>
                        <div class="wiz-f"><label class="wiz-label">"City / Metro Area 1"</label>
                            <input class="wiz-inp" type="text" placeholder="Miami-Dade"
                                prop:value=move || market1.get()
                                on:input=move |e| market1.set(event_target_value(&e))/></div>
                        <div class="wiz-f"><label class="wiz-label">"City / Metro Area 2 (optional)"</label>
                            <input class="wiz-inp" type="text" placeholder="Broward County"
                                prop:value=move || market2.get()
                                on:input=move |e| market2.set(event_target_value(&e))/></div>
                        <div class="wiz-f"><label class="wiz-label">"Price Range Focus"</label>
                            <select class="wiz-inp"
                                prop:value=move || price_range.get()
                                on:change=move |e| price_range.set(event_target_value(&e))>
                                <option value="any">"Any"</option>
                                <option value="entry">"Entry / Starter"</option>
                                <option value="mid">"Mid-market"</option>
                                <option value="luxury">"Luxury / High-end"</option>
                            </select></div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Languages"</div>
                        <div class="wiz-og wiz-og3">
                            {LANGUAGES.iter().map(|lang| {
                                let l = *lang;
                                view! {
                                    <button type="button"
                                        class=move || if lang_sel.get().contains(l) { "wiz-oc sel" } else { "wiz-oc" }
                                        on:click=move |_| lang_sel.update(|s| {
                                            if s.contains(l) { s.remove(l); } else { s.insert(l); }
                                        })>
                                        <div class="wiz-oc-label">{l}</div>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 3: Brokerage Assignment ────────────────────────────────
            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(5,150,105,.08); color:#047857;">
                        <span class="ms" style="font-size:13px;">"corporate_fare"</span>"Step 3 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Brokerage Assignment"</h1>
                    <p class="wiz-s-sub">"Confirm your affiliation with your sponsoring broker and upload required documents."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Commission Split"</div>
                        <div style="padding:14px; background:#f4f6fb; border-radius:8px; font-size:14px;">
                            <div style="display:flex; justify-content:space-between; margin-bottom:8px;">
                                <span style="color:#64748b;">"Default split"</span>
                                <strong>"70% Agent / 30% Broker"</strong>
                            </div>
                            <p style="font-size:12px; color:#94a3b8; margin:0;">
                                "Your broker can adjust your tier after onboarding."
                            </p>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Required Documents"</div>
                        <div class="wiz-na-row">
                            <span class="ms msf" style="font-size:24px; color:#059669;">"badge"</span>
                            <div>
                                <div style="font-size:14px; font-weight:600;">"License Certificate"</div>
                                <div style="font-size:12px; color:#64748b;">"Upload later from your profile if needed"</div>
                            </div>
                        </div>
                        <div class="wiz-na-row">
                            <span class="ms msf" style="font-size:24px; color:#059669;">"verified_user"</span>
                            <div>
                                <div style="font-size:14px; font-weight:600;">"E&O Insurance Certificate"</div>
                                <div style="font-size:12px; color:#64748b;">"Required before first listing"</div>
                            </div>
                        </div>
                        <div class="wiz-na-row">
                            <span class="ms msf" style="font-size:24px; color:#059669;">"handshake"</span>
                            <div>
                                <div style="font-size:14px; font-weight:600;">"Broker Agreement"</div>
                                <div style="font-size:12px; color:#64748b;">"Signed by you and your broker"</div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 4: Tools & Notifications ───────────────────────────────
            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(5,150,105,.08); color:#047857;">
                        <span class="ms" style="font-size:13px;">"notifications"</span>"Step 4 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Tools & Notifications"</h1>
                    <p class="wiz-s-sub">"Configure how you want to work and receive client and deal alerts."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Notifications"</div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"New lead alerts"</div>
                                <div class="wiz-tr-desc">"When a client inquiry is assigned to you"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_leads.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_leads.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Deal status updates"</div>
                                <div class="wiz-tr-desc">"Offers, counteroffers, and closings"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_deals.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_deals.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Listing changes"</div>
                                <div class="wiz-tr-desc">"Price updates and status changes"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_listing.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_listing.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Commission remittance"</div>
                                <div class="wiz-tr-desc">"When your split is paid out"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_comm.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_comm.update(|v| *v = !*v)
                            ></button>
                        </div>
                    </div>
                </div>
            </Show>
        </WizardShell>
    }
}
