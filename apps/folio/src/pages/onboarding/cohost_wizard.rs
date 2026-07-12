// apps/folio/src/pages/onboarding/cohost_wizard.rs
//
// CohostWizard — /onboard/cohost
//
// 4 steps mirroring wiz_cohost_onboard/code.html:
//   1. Your Profile
//   2. Experience & Specialties
//   3. Assigned Properties
//   4. Availability & Notifications

use crate::components::wizard_shell::{
    resolve_invite_code, ResolvedInviteCode, WizardShell, WizardStepDesc,
};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;
use leptos::prelude::*;

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc {
        id: "profile",
        label: "Your Profile",
        skippable: false,
    },
    WizardStepDesc {
        id: "experience",
        label: "Experience & Specialties",
        skippable: false,
    },
    WizardStepDesc {
        id: "properties",
        label: "Assigned Properties",
        skippable: false,
    },
    WizardStepDesc {
        id: "availability",
        label: "Availability & Notifications",
        skippable: false,
    },
];

const SPECIALTIES: &[&str] = &[
    "Guest Communication",
    "Check-In / Check-Out",
    "Cleaning Coordination",
    "Maintenance Oversight",
    "Pricing & Revenue",
    "Listing Optimization",
    "Photography",
    "STR Compliance",
    "Multi-City Management",
];

const PROP_TYPES: &[&str] = &[
    "Apartments",
    "Villas / Vacation Homes",
    "Condos",
    "Boutique Hotels",
    "Luxury Properties",
    "Budget Rentals",
];

const DAYS: &[(&str, &str)] = &[
    ("mon", "Mon"),
    ("tue", "Tue"),
    ("wed", "Wed"),
    ("thu", "Thu"),
    ("fri", "Fri"),
    ("sat", "Sat"),
    ("sun", "Sun"),
];

#[component]
pub fn CohostWizard() -> impl IntoView {
    let query = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());
    crate::pages::landlord::referrals::use_referral_attribution("cohost");
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
            "Go to Cohost Dashboard"
        } else {
            "Continue"
        }
    });

    let first = RwSignal::new(String::new());
    let last = RwSignal::new(String::new());
    let display = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let bio = RwSignal::new(String::new());
    let languages = RwSignal::new("English".to_string());
    let timezone = RwSignal::new("America/New_York".to_string());
    let years = RwSignal::new("1-2".to_string());
    let props_managed = RwSignal::new("1-3".to_string());

    let specs: RwSignal<std::collections::HashSet<&'static str>> =
        RwSignal::new(std::collections::HashSet::new());
    let prop_types: RwSignal<std::collections::HashSet<&'static str>> =
        RwSignal::new(std::collections::HashSet::new());
    let days_sel: RwSignal<std::collections::HashSet<&'static str>> =
        RwSignal::new(std::collections::HashSet::from([
            "mon", "tue", "wed", "thu", "fri",
        ]));

    let start_time = RwSignal::new("09:00".to_string());
    let end_time = RwSignal::new("18:00".to_string());
    let response_target = RwSignal::new("1h".to_string());

    let notify_guest = RwSignal::new(true);
    let notify_booking = RwSignal::new(true);
    let notify_maint = RwSignal::new(true);
    let notify_review = RwSignal::new(false);

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx + 1 >= total {
            let invite_id = invite_sig.get().map(|c| c.code.clone()).unwrap_or_default();
            leptos::task::spawn_local(async move {
                let nav = leptos_router::hooks::use_navigate();
                match accept_invite_code(invite_id, "/s".to_string()).await {
                    Ok(resp) => nav(&resp.redirect, Default::default()),
                    Err(_) => nav("/s", Default::default()),
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
            <p class="wiz-ctx-p">"Join as a co-host and earn a share of STR revenue from properties you help manage."</p>
            <ul class="wiz-ctx-list">
                <li><span class="ms msf">"check_circle"</span>"Co-manage STR properties"</li>
                <li><span class="ms msf">"check_circle"</span>"Revenue share automatically calculated"</li>
                <li><span class="ms msf">"check_circle"</span>"Guest messaging and calendar access"</li>
            </ul>
        }
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx persona_pill="Co-host"
            persona_icon="supervisor_account" accent_color="#0891b2" panel_bg="#0c1820"
            ctx_headline="Set up your Cohost profile" ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev is_last_step=is_last next_label=next_label>

            // ── Step 1: Your Profile ────────────────────────────────────────
            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(8,145,178,.08); color:#0e7490;">
                        <span class="ms" style="font-size:13px;">"person"</span>"Step 1 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Your Profile"</h1>
                    <p class="wiz-s-sub">"This is your public Cohost profile — guests and landlords will see this."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Photo & Name"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Alex"
                                    prop:value=move || first.get()
                                    on:input=move |e| first.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Torres"
                                    prop:value=move || last.get()
                                    on:input=move |e| last.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"Display Name"</label>
                            <input class="wiz-inp" type="text" placeholder="Alex T."
                                prop:value=move || display.get()
                                on:input=move |e| display.set(event_target_value(&e))/></div>
                        <div class="wiz-f"><label class="wiz-label">"Phone"</label>
                            <input class="wiz-inp" type="tel" placeholder="+1 (555) 000-0000"
                                prop:value=move || phone.get()
                                on:input=move |e| phone.set(event_target_value(&e))/></div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"About You"</div>
                        <div class="wiz-f"><label class="wiz-label">"Short Bio (shown on guest portal)"</label>
                            <textarea class="wiz-inp" rows="3"
                                placeholder="Tell guests and landlords about your hosting style..."
                                prop:value=move || bio.get()
                                on:input=move |e| bio.set(event_target_value(&e))></textarea></div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Languages"</label>
                                <input class="wiz-inp" type="text" placeholder="English, Spanish"
                                    prop:value=move || languages.get()
                                    on:input=move |e| languages.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Time Zone"</label>
                                <select class="wiz-inp"
                                    prop:value=move || timezone.get()
                                    on:change=move |e| timezone.set(event_target_value(&e))>
                                    <option value="America/New_York">"Eastern (US)"</option>
                                    <option value="America/Chicago">"Central (US)"</option>
                                    <option value="America/Denver">"Mountain (US)"</option>
                                    <option value="America/Los_Angeles">"Pacific (US)"</option>
                                    <option value="America/Sao_Paulo">"São Paulo"</option>
                                    <option value="Europe/London">"London"</option>
                                </select></div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 2: Experience & Specialties ────────────────────────────
            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(8,145,178,.08); color:#0e7490;">
                        <span class="ms" style="font-size:13px;">"star"</span>"Step 2 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Experience & Specialties"</h1>
                    <p class="wiz-s-sub">"Tell us about your STR hosting experience and what you're best at."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"STR Experience"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Years of STR Experience"</label>
                                <select class="wiz-inp"
                                    prop:value=move || years.get()
                                    on:change=move |e| years.set(event_target_value(&e))>
                                    <option value="<1">"Less than 1 year"</option>
                                    <option value="1-2">"1–2 years"</option>
                                    <option value="3-5">"3–5 years"</option>
                                    <option value="5+">"5+ years"</option>
                                </select></div>
                            <div class="wiz-f"><label class="wiz-label">"Properties Managed (total)"</label>
                                <select class="wiz-inp"
                                    prop:value=move || props_managed.get()
                                    on:change=move |e| props_managed.set(event_target_value(&e))>
                                    <option value="1-3">"1–3"</option>
                                    <option value="4-10">"4–10"</option>
                                    <option value="11-25">"11–25"</option>
                                    <option value="25+">"25+"</option>
                                </select></div>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Specialties"</div>
                        <div class="wiz-og wiz-og3">
                            {SPECIALTIES.iter().map(|s| {
                                let label = *s;
                                view! {
                                    <button type="button"
                                        class=move || if specs.get().contains(label) { "wiz-oc sel" } else { "wiz-oc" }
                                        on:click=move |_| specs.update(|set| {
                                            if set.contains(label) { set.remove(label); } else { set.insert(label); }
                                        })>
                                        <div class="wiz-oc-label">{label}</div>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Property Types"</div>
                        <div class="wiz-og wiz-og3">
                            {PROP_TYPES.iter().map(|s| {
                                let label = *s;
                                view! {
                                    <button type="button"
                                        class=move || if prop_types.get().contains(label) { "wiz-oc sel" } else { "wiz-oc" }
                                        on:click=move |_| prop_types.update(|set| {
                                            if set.contains(label) { set.remove(label); } else { set.insert(label); }
                                        })>
                                        <div class="wiz-oc-label">{label}</div>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 3: Assigned Properties ─────────────────────────────────
            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(8,145,178,.08); color:#0e7490;">
                        <span class="ms" style="font-size:13px;">"home"</span>"Step 3 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Assigned Properties"</h1>
                    <p class="wiz-s-sub">"These are the properties you'll be co-managing. Review them and confirm your understanding of each."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Your Properties"</div>
                        <Show
                            when=move || {
                                invite_sig.get().and_then(|c| c.context.asset_count).unwrap_or(0) > 0
                            }
                            fallback=|| view! {
                                <p style="font-size:13px; color:#64748b;">
                                    "Properties linked to your invite will appear here. Your landlord can assign more after you join."
                                </p>
                            }
                        >
                            <div class="wiz-na-row">
                                <span class="ms msf" style="font-size:28px; color:#0891b2;">"villa"</span>
                                <div>
                                    <div style="font-size:14px; font-weight:600;">
                                        {move || {
                                            let n = invite_sig.get().and_then(|c| c.context.asset_count).unwrap_or(0);
                                            format!(
                                                "{} propert{}",
                                                n,
                                                if n == 1 { "y" } else { "ies" }
                                            )
                                        }}
                                    </div>
                                    <div style="font-size:12px; color:#64748b;">"Assigned via your invite"</div>
                                </div>
                            </div>
                        </Show>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Your Responsibilities"</div>
                        <ul class="wiz-ctx-list" style="color:#475569;">
                            <li><span class="ms msf" style="color:#0891b2;">"check_circle"</span>"Respond to guest messages within your target window"</li>
                            <li><span class="ms msf" style="color:#0891b2;">"check_circle"</span>"Coordinate cleaning and turnover between stays"</li>
                            <li><span class="ms msf" style="color:#0891b2;">"check_circle"</span>"Escalate maintenance issues to the landlord"</li>
                        </ul>
                    </div>
                </div>
            </Show>

            // ── Step 4: Availability & Notifications ────────────────────────
            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(8,145,178,.08); color:#0e7490;">
                        <span class="ms" style="font-size:13px;">"schedule"</span>"Step 4 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Availability & Notifications"</h1>
                    <p class="wiz-s-sub">"Set your working days and how you want to receive alerts from guests and the landlord."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Available Days"</div>
                        <div class="wiz-og" style="grid-template-columns:repeat(7,1fr);">
                            {DAYS.iter().map(|(id, label)| {
                                let cid = *id;
                                let lbl = *label;
                                view! {
                                    <button type="button"
                                        class=move || if days_sel.get().contains(cid) { "wiz-oc sel" } else { "wiz-oc" }
                                        on:click=move |_| days_sel.update(|s| {
                                            if s.contains(cid) { s.remove(cid); } else { s.insert(cid); }
                                        })>
                                        <div class="wiz-oc-label">{lbl}</div>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                        <div class="wiz-inp-row" style="margin-top:14px;">
                            <div class="wiz-f"><label class="wiz-label">"Start Time"</label>
                                <input class="wiz-inp" type="time"
                                    prop:value=move || start_time.get()
                                    on:input=move |e| start_time.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"End Time"</label>
                                <input class="wiz-inp" type="time"
                                    prop:value=move || end_time.get()
                                    on:input=move |e| end_time.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"Response Time Target"</label>
                            <select class="wiz-inp"
                                prop:value=move || response_target.get()
                                on:change=move |e| response_target.set(event_target_value(&e))>
                                <option value="15m">"Within 15 minutes"</option>
                                <option value="1h">"Within 1 hour"</option>
                                <option value="4h">"Within 4 hours"</option>
                                <option value="24h">"Within 24 hours"</option>
                            </select></div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Alert Preferences"</div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Guest messages"</div>
                                <div class="wiz-tr-desc">"New inquiries and in-stay requests"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_guest.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_guest.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Booking alerts"</div>
                                <div class="wiz-tr-desc">"New reservations and cancellations"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_booking.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_booking.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Maintenance escalations"</div>
                                <div class="wiz-tr-desc">"Issues that need landlord attention"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_maint.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_maint.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Guest reviews"</div>
                                <div class="wiz-tr-desc">"When a review is posted"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_review.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_review.update(|v| *v = !*v)
                            ></button>
                        </div>
                    </div>
                </div>
            </Show>
        </WizardShell>
    }
}
