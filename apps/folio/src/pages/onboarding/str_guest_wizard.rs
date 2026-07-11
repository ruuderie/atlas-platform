// apps/folio/src/pages/onboarding/str_guest_wizard.rs
//
// StrGuestWizard — /onboard/str-guest
//
// 4 steps mirroring wiz_str_guest_onboard/code.html:
//   1. Select Your Stay
//   2. Create Account (identity confirm / passkey upsell — OTP is WizardShell pre-auth)
//   3. Guest Profile
//   4. House Rules & Check-In

use leptos::prelude::*;
use crate::components::wizard_shell::{
    ResolvedInviteCode, WizardShell, WizardStepDesc, resolve_invite_code,
};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc { id: "stay",     label: "Select Your Stay",          skippable: false },
    WizardStepDesc { id: "account",  label: "Create Account",            skippable: false },
    WizardStepDesc { id: "profile",  label: "Guest Profile",             skippable: false },
    WizardStepDesc { id: "rules",    label: "House Rules & Check-In",    skippable: false },
];

#[component]
pub fn StrGuestWizard() -> impl IntoView {
    let query    = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());
    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));
    Effect::new(move |_| { if let Some(Ok(r)) = code_resource.get() { invite_sig.set(r); } });

    let current_idx = RwSignal::new(0usize);
    let total       = STEPS.len();
    let is_last     = Signal::derive(move || current_idx.get() == total - 1);
    let next_label  = Signal::derive(move || if is_last.get() { "Confirm Booking" } else { "Continue" });

    // Stay
    let checkin  = RwSignal::new(String::new());
    let checkout = RwSignal::new(String::new());
    let guests   = RwSignal::new("2".to_string());
    let special  = RwSignal::new(String::new());

    // Account (post-OTP confirm / passkey upsell)
    let auth_method = RwSignal::new("passkey".to_string());
    let confirm_email = RwSignal::new(String::new());

    // Profile
    let first    = RwSignal::new(String::new());
    let last     = RwSignal::new(String::new());
    let phone    = RwSignal::new(String::new());
    let country  = RwSignal::new("US".to_string());
    let emerg_name = RwSignal::new(String::new());
    let emerg_phone = RwSignal::new(String::new());
    let emerg_rel = RwSignal::new(String::new());
    let trip_purpose = RwSignal::new(String::new());

    // Rules
    let rules_ok = RwSignal::new(false);

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx + 1 >= total {
            let invite_id = invite_sig.get().map(|c| c.code.clone()).unwrap_or_default();
            leptos::task::spawn_local(async move {
                let nav = leptos_router::hooks::use_navigate();
                match accept_invite_code(invite_id, "/t/reservations".to_string()).await {
                    Ok(resp) => nav(&resp.redirect, Default::default()),
                    Err(_)   => nav("/t/reservations", Default::default()),
                }
            });
        } else { current_idx.set(idx + 1); }
    });
    let on_prev = Callback::new(move |_| { let i = current_idx.get(); if i > 0 { current_idx.set(i - 1); } });

    let ctx_body = ViewFn::from(|| view! {
        <p class="wiz-ctx-p">"Book your stay directly through Folio and skip OTA platform fees."</p>
        <ul class="wiz-ctx-list">
            <li><span class="ms msf">"check_circle"</span>"No platform fees"</li>
            <li><span class="ms msf">"check_circle"</span>"Direct host communication"</li>
            <li><span class="ms msf">"check_circle"</span>"Secure check-in details in-app"</li>
        </ul>
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx persona_pill="STR Guest"
            persona_icon="beach_access" accent_color="#f59e0b" panel_bg="#1c1007"
            ctx_headline="Book your stay" ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev is_last_step=is_last next_label=next_label>

            // ── Step 1: Select Your Stay ────────────────────────────────────
            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(245,158,11,.08); color:#b45309;">
                        <span class="ms" style="font-size:13px;">"calendar_month"</span>"Step 1 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Select Your Stay"</h1>
                    <p class="wiz-s-sub">"Choose your check-in and check-out dates, then confirm the number of guests."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Dates"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Check-In"</label>
                                <input class="wiz-inp" type="date"
                                    prop:value=move || checkin.get()
                                    on:input=move |e| checkin.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Check-Out"</label>
                                <input class="wiz-inp" type="date"
                                    prop:value=move || checkout.get()
                                    on:input=move |e| checkout.set(event_target_value(&e))/></div>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Guests"</div>
                        <div class="wiz-f"><label class="wiz-label">"Number of Guests"</label>
                            <select class="wiz-inp"
                                prop:value=move || guests.get()
                                on:change=move |e| guests.set(event_target_value(&e))>
                                <option value="1">"1"</option>
                                <option value="2">"2"</option>
                                <option value="3">"3"</option>
                                <option value="4">"4"</option>
                                <option value="5+">"5+"</option>
                            </select></div>
                        <div class="wiz-f"><label class="wiz-label">"Special Requests (optional)"</label>
                            <textarea class="wiz-inp" rows="2" placeholder="Early check-in, crib, etc."
                                prop:value=move || special.get()
                                on:input=move |e| special.set(event_target_value(&e))></textarea></div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Stay Summary"</div>
                        <div style="display:flex; justify-content:space-between; padding:10px 0; border-bottom:1px solid #e2e8f0; font-size:14px;">
                            <span style="color:#64748b;">"Check-In"</span>
                            <strong>{move || { let v = checkin.get(); if v.is_empty() { "—".into() } else { v } }}</strong>
                        </div>
                        <div style="display:flex; justify-content:space-between; padding:10px 0; border-bottom:1px solid #e2e8f0; font-size:14px;">
                            <span style="color:#64748b;">"Check-Out"</span>
                            <strong>{move || { let v = checkout.get(); if v.is_empty() { "—".into() } else { v } }}</strong>
                        </div>
                        <div style="display:flex; justify-content:space-between; padding:10px 0; font-size:14px;">
                            <span style="color:#64748b;">"Guests"</span>
                            <strong>{move || guests.get()}</strong>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 2: Create Account (confirm identity / passkey upsell) ───
            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(245,158,11,.08); color:#b45309;">
                        <span class="ms" style="font-size:13px;">"person_add"</span>"Step 2 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Create Your Account"</h1>
                    <p class="wiz-s-sub">"A free Folio account keeps your booking secure and lets you message your host, track check-in details, and manage future stays."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Sign-In Preference"</div>
                        <p style="font-size:13px; color:#64748b; margin-bottom:14px;">
                            "You're already verified via email. Optionally add a passkey for faster return visits."
                        </p>
                        <div class="wiz-og wiz-og2">
                            <button type="button"
                                class=move || if auth_method.get() == "passkey" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| auth_method.set("passkey".into())>
                                <span class="ms msf">"fingerprint"</span>
                                <div class="wiz-oc-label">"Use Passkey (Recommended)"</div>
                                <div class="wiz-oc-desc">"Face ID, Touch ID, or device PIN — no password"</div>
                            </button>
                            <button type="button"
                                class=move || if auth_method.get() == "magic" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| auth_method.set("magic".into())>
                                <span class="ms msf">"mail"</span>
                                <div class="wiz-oc-label">"Use Email + Magic Link"</div>
                                <div class="wiz-oc-desc">"Continue with email links only"</div>
                            </button>
                        </div>
                        <div class="wiz-f" style="margin-top:14px;">
                            <label class="wiz-label">"Your Email"</label>
                            <input class="wiz-inp" type="email" placeholder="jamie@email.com"
                                prop:value=move || confirm_email.get()
                                on:input=move |e| confirm_email.set(event_target_value(&e))/>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 3: Guest Profile ───────────────────────────────────────
            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(245,158,11,.08); color:#b45309;">
                        <span class="ms" style="font-size:13px;">"badge"</span>"Step 3 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Your Guest Profile"</h1>
                    <p class="wiz-s-sub">"Your host needs this info to prepare for your arrival. It's stored securely and never shared publicly."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Contact"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Jamie"
                                    prop:value=move || first.get()
                                    on:input=move |e| first.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Rivera"
                                    prop:value=move || last.get()
                                    on:input=move |e| last.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Phone Number"</label>
                                <input class="wiz-inp" type="tel" placeholder="+1 (555) 000-0000"
                                    prop:value=move || phone.get()
                                    on:input=move |e| phone.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Home Country"</label>
                                <select class="wiz-inp"
                                    prop:value=move || country.get()
                                    on:change=move |e| country.set(event_target_value(&e))>
                                    <option value="US">"United States"</option>
                                    <option value="CA">"Canada"</option>
                                    <option value="GB">"United Kingdom"</option>
                                    <option value="BR">"Brazil"</option>
                                    <option value="OTHER">"Other"</option>
                                </select></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"Trip Purpose (optional)"</label>
                            <input class="wiz-inp" type="text" placeholder="Vacation, business, family visit…"
                                prop:value=move || trip_purpose.get()
                                on:input=move |e| trip_purpose.set(event_target_value(&e))/></div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Emergency Contact"</div>
                        <div class="wiz-f"><label class="wiz-label">"Contact Name"</label>
                            <input class="wiz-inp" type="text" placeholder="Full name"
                                prop:value=move || emerg_name.get()
                                on:input=move |e| emerg_name.set(event_target_value(&e))/></div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Contact Phone"</label>
                                <input class="wiz-inp" type="tel" placeholder="+1 (555) 000-0000"
                                    prop:value=move || emerg_phone.get()
                                    on:input=move |e| emerg_phone.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Relationship"</label>
                                <input class="wiz-inp" type="text" placeholder="Spouse, parent, friend…"
                                    prop:value=move || emerg_rel.get()
                                    on:input=move |e| emerg_rel.set(event_target_value(&e))/></div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 4: House Rules & Check-In ──────────────────────────────
            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(245,158,11,.08); color:#b45309;">
                        <span class="ms" style="font-size:13px;">"rule"</span>"Step 4 of 4"
                    </div>
                    <h1 class="wiz-s-title">"House Rules & Check-In"</h1>
                    <p class="wiz-s-sub">"Please read and acknowledge the house rules, then review your check-in details."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Check-In Details"</div>
                        <div style="display:flex; justify-content:space-between; padding:10px 0; border-bottom:1px solid #e2e8f0; font-size:14px;">
                            <span style="color:#64748b;">"Check-In Time"</span>
                            <strong>"After 3:00 PM"</strong>
                        </div>
                        <div style="display:flex; justify-content:space-between; padding:10px 0; border-bottom:1px solid #e2e8f0; font-size:14px;">
                            <span style="color:#64748b;">"Check-Out Time"</span>
                            <strong>"Before 11:00 AM"</strong>
                        </div>
                        <div style="display:flex; justify-content:space-between; padding:10px 0; border-bottom:1px solid #e2e8f0; font-size:14px;">
                            <span style="color:#64748b;">"Key Access"</span>
                            <strong>"Smart lock code (sent 24h before)"</strong>
                        </div>
                        <div style="display:flex; justify-content:space-between; padding:10px 0; font-size:14px;">
                            <span style="color:#64748b;">"Parking"</span>
                            <strong>"Street / driveway as posted"</strong>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"House Rules"</div>
                        <ul style="list-style:none; display:flex; flex-direction:column; gap:10px; margin:0 0 16px; padding:0; font-size:14px; color:#475569;">
                            <li style="display:flex; gap:8px;"><span class="ms msf" style="color:#f59e0b;">"smoke_free"</span>"No smoking"</li>
                            <li style="display:flex; gap:8px;"><span class="ms msf" style="color:#f59e0b;">"pets"</span>"No pets"</li>
                            <li style="display:flex; gap:8px;"><span class="ms msf" style="color:#f59e0b;">"bedtime"</span>"Quiet hours 10pm–8am"</li>
                            <li style="display:flex; gap:8px;"><span class="ms msf" style="color:#f59e0b;">"group"</span>"Max 4 guests"</li>
                            <li style="display:flex; gap:8px;"><span class="ms msf" style="color:#f59e0b;">"cleaning_services"</span>"Leave it tidy"</li>
                        </ul>
                        <label style="display:flex; align-items:flex-start; gap:12px; cursor:pointer; font-size:13px; color:#64748b; line-height:1.6;">
                            <input type="checkbox" style="margin-top:3px; flex-shrink:0; accent-color:#f59e0b;"
                                prop:checked=move || rules_ok.get()
                                on:change=move |ev: web_sys::Event| {
                                    let el = event_target::<web_sys::HtmlInputElement>(&ev);
                                    rules_ok.set(el.checked());
                                }/>
                            "I agree to the house rules and guest policy"
                        </label>
                    </div>
                </div>
            </Show>
        </WizardShell>
    }
}
