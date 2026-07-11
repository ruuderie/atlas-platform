// apps/folio/src/pages/onboarding/property_owner_wizard.rs
//
// PropertyOwnerWizard — /onboard/property-owner
//
// Free-tier Property Owner Lite onboarding (wiz_property_owner_onboard):
//   1. Your details
//   2. Your property
//   3. Your vendors
//   4. Grow your network (optional)
//
// Navigates to /po on completion.

use crate::components::wizard_shell::{
    resolve_invite_code, ResolvedInviteCode, WizardShell, WizardStepDesc,
};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;
use leptos::prelude::*;

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc {
        id: "details",
        label: "Your details",
        skippable: false,
    },
    WizardStepDesc {
        id: "property",
        label: "Your property",
        skippable: false,
    },
    WizardStepDesc {
        id: "vendors",
        label: "Your vendors",
        skippable: true,
    },
    WizardStepDesc {
        id: "network",
        label: "Grow your network",
        skippable: true,
    },
];

#[component]
pub fn PropertyOwnerWizard() -> impl IntoView {
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
            "Go to My Dashboard"
        } else {
            "Continue"
        }
    });

    // Details
    let first = RwSignal::new(String::new());
    let last = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());

    // Property
    let address = RwSignal::new(String::new());
    let city = RwSignal::new(String::new());
    let state = RwSignal::new(String::new());
    let prop_type = RwSignal::new("single_family".to_string());
    let value = RwSignal::new(String::new());
    let value_method = RwSignal::new("manual".to_string());

    // Vendors
    let review_emails = RwSignal::new(true);
    let vendor_search = RwSignal::new(String::new());

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx + 1 >= total {
            let invite_id = invite_sig.get().map(|c| c.code.clone()).unwrap_or_default();
            leptos::task::spawn_local(async move {
                let nav = leptos_router::hooks::use_navigate();
                match accept_invite_code(invite_id, "/po".to_string()).await {
                    Ok(resp) => nav(&resp.redirect, Default::default()),
                    Err(_) => nav("/po", Default::default()),
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
            <p class="wiz-ctx-p">"No subscription required. Monitor your home's value, connect with vendors, and leave verified reviews — all in one place."</p>
            <ul class="wiz-ctx-list">
                <li><span class="ms msf">"check_circle"</span>"Track your property value over time"</li>
                <li><span class="ms msf">"check_circle"</span>"See all vendors you've worked with"</li>
                <li><span class="ms msf">"check_circle"</span>"Leave verified reviews"</li>
            </ul>
        }
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx persona_pill="Property Owner"
            persona_icon="home" accent_color="#0284c7" panel_bg="#0d1421"
            ctx_headline="Track your property. For free." ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev is_last_step=is_last next_label=next_label>

            // ── Step 1: Your details ────────────────────────────────────────
            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"person"</span>"Step 1 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Let's set up your account"</h1>
                    <p class="wiz-s-sub">"Your email is already verified. Just tell us your name and we'll get your dashboard ready in seconds."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Your Information"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Sarah"
                                    prop:value=move || first.get()
                                    on:input=move |e| first.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Wilson"
                                    prop:value=move || last.get()
                                    on:input=move |e| last.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"Email Address"</label>
                            <input class="wiz-inp" type="email" placeholder="sarah.wilson@gmail.com"
                                prop:value=move || email.get()
                                on:input=move |e| email.set(event_target_value(&e))/></div>
                        <div class="wiz-f"><label class="wiz-label">"Phone (optional)"</label>
                            <input class="wiz-inp" type="tel" placeholder="+1 (555) 000-0000"
                                prop:value=move || phone.get()
                                on:input=move |e| phone.set(event_target_value(&e))/></div>
                    </div>
                </div>
            </Show>

            // ── Step 2: Your property ───────────────────────────────────────
            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"home"</span>"Step 2 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Tell us about your property"</h1>
                    <p class="wiz-s-sub">"Add your address to track property value and find local vendors."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Address"</div>
                        <div class="wiz-f"><label class="wiz-label">"Property Address"</label>
                            <input class="wiz-inp" type="text" placeholder="123 Oak Street"
                                prop:value=move || address.get()
                                on:input=move |e| address.set(event_target_value(&e))/></div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"City"</label>
                                <input class="wiz-inp" type="text" placeholder="Austin"
                                    prop:value=move || city.get()
                                    on:input=move |e| city.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"State"</label>
                                <select class="wiz-inp"
                                    prop:value=move || state.get()
                                    on:change=move |e| state.set(event_target_value(&e))>
                                    <option value="">"Select…"</option>
                                    <option value="TX">"Texas"</option>
                                    <option value="CA">"California"</option>
                                    <option value="FL">"Florida"</option>
                                    <option value="NY">"New York"</option>
                                </select></div>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Property Type"</div>
                        <div class="wiz-og wiz-og3">
                            {[
                                ("single_family", "home", "Single Family"),
                                ("condo", "apartment", "Condo / Apt"),
                                ("multi", "domain", "Multi-Family"),
                                ("townhouse", "holiday_village", "Townhouse"),
                                ("commercial", "storefront", "Commercial"),
                                ("other", "more_horiz", "Other"),
                            ].into_iter().map(|(val, icon, label)| {
                                let val = val.to_string();
                                let val2 = val.clone();
                                view! {
                                    <button type="button"
                                        class=move || if prop_type.get() == val { "wiz-oc sel" } else { "wiz-oc" }
                                        on:click=move |_| prop_type.set(val2.clone())>
                                        <span class="ms msf">{icon}</span>
                                        <div class="wiz-oc-label">{label}</div>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Property Value (optional)"</div>
                        <div class="wiz-f"><label class="wiz-label">"Estimated Current Value"</label>
                            <input class="wiz-inp" type="text" placeholder="$450,000"
                                prop:value=move || value.get()
                                on:input=move |e| value.set(event_target_value(&e))/></div>
                        <div class="wiz-f"><label class="wiz-label">"Value Tracking Method"</label>
                            <select class="wiz-inp"
                                prop:value=move || value_method.get()
                                on:change=move |e| value_method.set(event_target_value(&e))>
                                <option value="manual">"Manual updates"</option>
                                <option value="zillow">"Auto (Zillow AVM)"</option>
                                <option value="county">"County records"</option>
                            </select></div>
                    </div>
                </div>
            </Show>

            // ── Step 3: Your vendors ────────────────────────────────────────
            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"handyman"</span>"Step 3 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Your vendors"</h1>
                    <p class="wiz-s-sub">"We found one vendor linked to your invite. Add others you work with to track service history and reviews."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Linked from your invite"</div>
                        <div class="wiz-na-row">
                            <div style="width:38px; height:38px; border-radius:50%; background:linear-gradient(135deg,#0284c7,#0d9488); display:flex; align-items:center; justify-content:center; font-size:13px; font-weight:800; color:#fff; flex-shrink:0;">
                                "HS"
                            </div>
                            <div>
                                <div style="font-size:14px; font-weight:600;">"HomeShine Services"</div>
                                <div style="font-size:12px; color:#64748b;">"Roofing & Exterior"</div>
                            </div>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Add More Vendors"</div>
                        <div class="wiz-f"><label class="wiz-label">"Search vendor by name or license number"</label>
                            <input class="wiz-inp" type="text" placeholder="Start typing…"
                                prop:value=move || vendor_search.get()
                                on:input=move |e| vendor_search.set(event_target_value(&e))/></div>
                        <p style="font-size:12px; color:#94a3b8; margin-top:4px;">
                            "Skip this — add vendors anytime from your dashboard."
                        </p>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Enable review request emails"</div>
                                <div class="wiz-tr-desc">"When a vendor completes a job, they can send you a review invite"</div>
                            </div>
                            <button type="button"
                                class=move || if review_emails.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| review_emails.update(|v| *v = !*v)
                            ></button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 4: Grow your network ───────────────────────────────────
            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"group_add"</span>"Step 4 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Grow your network"</h1>
                    <p class="wiz-s-sub">"Invite other owners and landlords you know, and vendors you trust. Optional anytime."</p>
                    {
                        use crate::components::network_invite_panel::{AngleCard, NetworkInvitePanel};
                        view! {
                            <NetworkInvitePanel
                                actor_role="property_owner"
                                preferred_slug="property_owner_invite_peers"
                                angles=vec![
                                    AngleCard {
                                        icon: "apartment",
                                        title: "Other owners & landlords",
                                        body: "Share Folio with owners in your circle so they can track value and vendors the same way you do.",
                                        benefit_icon: None,
                                        benefit_label: None,
                                    },
                                    AngleCard {
                                        icon: "handyman",
                                        title: "Vendors you recommend",
                                        body: "Invite a contractor you trust. The next job stays on Folio with shared history and reviews.",
                                        benefit_icon: None,
                                        benefit_label: None,
                                    },
                                ]
                                section_title="Who to invite".to_string()
                                footnote="They get a personal invite link. Skip if you are not ready. Invite anytime from your dashboard.".to_string()
                                show_history=false
                            />
                        }
                    }
                </div>
            </Show>
        </WizardShell>
    }
}
