use leptos::prelude::*;
use crate::components::wizard_shell::{ResolvedInviteCode, WizardShell, WizardStepDesc, resolve_invite_code};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc { id: "booking",    label: "Booking Details",  skippable: false },
    WizardStepDesc { id: "profile",    label: "Guest Profile",    skippable: false },
    WizardStepDesc { id: "payment",    label: "Payment",          skippable: false },
    WizardStepDesc { id: "confirm",    label: "Confirm",          skippable: false },
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

    let checkin  = RwSignal::new(String::new());
    let checkout = RwSignal::new(String::new());
    let guests   = RwSignal::new("2".to_string());
    let first    = RwSignal::new(String::new());
    let last     = RwSignal::new(String::new());
    let email    = RwSignal::new(String::new());

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
            <li><span class="ms msf">"check_circle"</span>"Secure payment processing"</li>
        </ul>
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx persona_pill="STR Guest"
            persona_icon="beach_access" accent_color="#f59e0b" panel_bg="#1c1007"
            ctx_headline="Book your stay" ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev is_last_step=is_last next_label=next_label>

            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(245,158,11,.08); color:#b45309;">
                        <span class="ms" style="font-size:13px;">"calendar_today"</span>"Step 1 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Booking Details"</h1>
                    <p class="wiz-s-sub">"Choose your dates and number of guests."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Dates &amp; Guests"</div>
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
                        <div class="wiz-f"><label class="wiz-label">"Guests"</label>
                            <select class="wiz-inp" prop:value=move || guests.get() on:change=move |e| guests.set(event_target_value(&e))>
                                <option>"1"</option><option selected>"2"</option><option>"3"</option><option>"4+"</option>
                            </select></div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(245,158,11,.08); color:#b45309;">
                        <span class="ms" style="font-size:13px;">"person"</span>"Step 2 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Guest Profile"</h1>
                    <p class="wiz-s-sub">"Your details for the booking confirmation and host communication."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Guest Info"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Jamie"
                                    prop:value=move || first.get() on:input=move |e| first.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Rivera"
                                    prop:value=move || last.get() on:input=move |e| last.set(event_target_value(&e))/></div>
                        </div>
                        <div class="wiz-f"><label class="wiz-label">"Email"</label>
                            <input class="wiz-inp" type="email" placeholder="jamie@email.com"
                                prop:value=move || email.get() on:input=move |e| email.set(event_target_value(&e))/></div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(245,158,11,.08); color:#b45309;">
                        <span class="ms" style="font-size:13px;">"credit_card"</span>"Step 3 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Payment"</h1>
                    <p class="wiz-s-sub">"Secure payment processing. Your card is not charged until the host accepts."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Payment Method"</div>
                        <div class="wiz-f"><label class="wiz-label">"Card Number"</label>
                            <input class="wiz-inp" type="text" placeholder="•••• •••• •••• ••••"/></div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Expiry"</label>
                                <input class="wiz-inp" type="text" placeholder="MM / YY"/></div>
                            <div class="wiz-f"><label class="wiz-label">"CVV"</label>
                                <input class="wiz-inp" type="text" placeholder="•••"/></div>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(245,158,11,.08); color:#b45309;">
                        <span class="ms" style="font-size:13px;">"task_alt"</span>"Step 4 of 4"
                    </div>
                    <h1 class="wiz-s-title">"Confirm Your Booking"</h1>
                    <p class="wiz-s-sub">"Review your booking summary before confirming."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Booking Summary"</div>
                        <div style="display:flex; justify-content:space-between; padding:10px 0; border-bottom:1px solid #e2e8f0; font-size:14px;">
                            <span style="color:#64748b;">"Check-In"</span>
                            <strong>{move || checkin.get()}</strong>
                        </div>
                        <div style="display:flex; justify-content:space-between; padding:10px 0; border-bottom:1px solid #e2e8f0; font-size:14px;">
                            <span style="color:#64748b;">"Check-Out"</span>
                            <strong>{move || checkout.get()}</strong>
                        </div>
                        <div style="display:flex; justify-content:space-between; padding:10px 0; font-size:14px;">
                            <span style="color:#64748b;">"Guests"</span>
                            <strong>{move || guests.get()}</strong>
                        </div>
                    </div>
                </div>
            </Show>

        </WizardShell>
    }
}
