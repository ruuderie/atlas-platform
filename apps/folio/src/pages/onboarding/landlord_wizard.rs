// apps/folio/src/pages/onboarding/landlord_wizard.rs
//
// LandlordWizard — /onboarding (replaces the original OnboardingWizard)
//
// 5 steps mirroring wiz_landlord_onboard/code.html:
//   1. Profile
//   2. Jurisdiction & License
//   3. First Property
//   4. Payment Rails
//   5. Go Live
//
// Invite code support: if ?code= is in the URL query string, the wizard
// pre-populates the context panel with the resolved landlord / entity info.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::wizard_shell::{
    ResolvedInviteCode, WizardShell, WizardStepDesc, resolve_invite_code,
};

// ── Server function — submit landlord onboarding ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LandlordDraft {
    pub first_name:        Option<String>,
    pub last_name:         Option<String>,
    pub phone:             Option<String>,
    pub jurisdiction_code: Option<String>,
    pub license_number:    Option<String>,
    pub completed_steps:   Vec<String>,
}

#[server(GetLandlordDraft, "/api")]
pub async fn get_landlord_draft() -> Result<LandlordDraft, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<LandlordDraft>(
        "/api/folio/onboarding/draft", &token, None,
    ).await.or_else(|_| Ok(LandlordDraft::default()))
}

#[server(SaveLandlordProfile, "/api")]
pub async fn save_landlord_profile(
    first_name:        String,
    last_name:         String,
    phone:             String,
    jurisdiction_code: String,
    license_number:    String,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let payload = serde_json::json!({
        "first_name": first_name, "last_name": last_name,
        "phone": phone, "jurisdiction_code": jurisdiction_code,
        "license_number": license_number, "step": "profile",
    });
    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/folio/onboarding/submit", &token, None, &payload,
    ).await.map(|_| ()).map_err(server_fn::error::ServerFnError::new)
}

#[server(SaveLandlordProperty, "/api")]
pub async fn save_landlord_property(
    property_name:    String,
    property_address: String,
    property_city:    String,
    property_state:   String,
    property_type:    String,
    unit_count:       String,
    str_eligible:     bool,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let payload = serde_json::json!({
        "property_name": property_name, "property_address": property_address,
        "property_city": property_city, "property_state": property_state,
        "property_type": property_type, "unit_count": unit_count,
        "str_eligible": str_eligible, "step": "first_property",
    });
    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/folio/onboarding/submit", &token, None, &payload,
    ).await.map(|_| ()).map_err(server_fn::error::ServerFnError::new)
}

// ── Steps ─────────────────────────────────────────────────────────────────────

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc { id: "profile",      label: "Your Profile",    skippable: false },
    WizardStepDesc { id: "jurisdiction", label: "Jurisdiction",     skippable: false },
    WizardStepDesc { id: "property",     label: "First Property",   skippable: false },
    WizardStepDesc { id: "payments",     label: "Payment Rails",    skippable: true  },
    WizardStepDesc { id: "go_live",      label: "Go Live",          skippable: false },
];

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn LandlordWizard() -> impl IntoView {
    // ── Invite code resolution ─────────────────────────────────────────────
    // Read ?code= from query string
    let query    = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());

    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));

    Effect::new(move |_| {
        if let Some(Ok(resolved)) = code_resource.get() {
            invite_sig.set(resolved);
        }
    });

    // ── Navigation ─────────────────────────────────────────────────────────
    let current_idx = RwSignal::new(0usize);
    let total       = STEPS.len();

    let is_last = Signal::derive(move || current_idx.get() == total - 1);
    let next_label = Signal::derive(move || {
        if is_last.get() { "Launch Folio" } else { "Continue" }
    });

    // ── Form signals ───────────────────────────────────────────────────────
    let first_name     = RwSignal::new(String::new());
    let last_name      = RwSignal::new(String::new());
    let phone          = RwSignal::new(String::new());
    let jurisdiction   = RwSignal::new("US-FL".to_string());
    let license_number = RwSignal::new(String::new());

    let prop_name      = RwSignal::new(String::new());
    let prop_address   = RwSignal::new(String::new());
    let prop_city      = RwSignal::new(String::new());
    let prop_state     = RwSignal::new("FL".to_string());
    let prop_type      = RwSignal::new("single_family".to_string());
    let unit_count     = RwSignal::new("1".to_string());
    let str_eligible   = RwSignal::new(false);

    let saving:     RwSignal<bool>          = RwSignal::new(false);
    let save_error: RwSignal<Option<String>> = RwSignal::new(None);

    // ── Draft fetch ────────────────────────────────────────────────────────
    let draft = Resource::new(|| (), |_| get_landlord_draft());
    Effect::new(move |_| {
        if let Some(Ok(d)) = draft.get() {
            if let Some(v) = d.first_name        { first_name.set(v); }
            if let Some(v) = d.last_name         { last_name.set(v);  }
            if let Some(v) = d.jurisdiction_code { jurisdiction.set(v); }
        }
    });

    // ── Navigation callbacks ───────────────────────────────────────────────
    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx == 0 || idx == 1 {
            // Save profile + jurisdiction together on step 1 advance
            if idx == 0 {
                let f  = first_name.get(); let l  = last_name.get();
                let ph = phone.get();       let j  = jurisdiction.get();
                let li = license_number.get();
                saving.set(true); save_error.set(None);
                leptos::task::spawn_local(async move {
                    match save_landlord_profile(f, l, ph, j, li).await {
                        Ok(_)  => { saving.set(false); current_idx.set(idx + 1); }
                        Err(e) => { saving.set(false); save_error.set(Some(e.to_string())); }
                    }
                });
                return;
            }
        }
        if idx == 2 {
            let n  = prop_name.get();    let a  = prop_address.get();
            let c  = prop_city.get();    let s  = prop_state.get();
            let t  = prop_type.get();    let u  = unit_count.get();
            let st = str_eligible.get();
            saving.set(true); save_error.set(None);
            leptos::task::spawn_local(async move {
                match save_landlord_property(n, a, c, s, t, u, st).await {
                    Ok(_)  => { saving.set(false); current_idx.set(idx + 1); }
                    Err(e) => { saving.set(false); save_error.set(Some(e.to_string())); }
                }
            });
            return;
        }
        if idx + 1 >= total {
            // Final step — redirect to dashboard
            let navigate = leptos_router::hooks::use_navigate();
            navigate("/l", Default::default());
        } else {
            current_idx.set(idx + 1);
        }
    });

    let on_prev = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx > 0 { current_idx.set(idx - 1); }
    });

    // ── Context panel body (generic copy when no invite code) ──────────────
    let ctx_body = ViewFn::from(|| view! {
        <p class="wiz-ctx-p">
            "Set up your Folio landlord workspace. You'll be able to manage properties, "
            "leases, maintenance, and tenant communications from a single dashboard."
        </p>
        <ul class="wiz-ctx-list">
            <li><span class="ms msf">"check_circle"</span>"Full portfolio management"</li>
            <li><span class="ms msf">"check_circle"</span>"Lease management &amp; e-sign"</li>
            <li><span class="ms msf">"check_circle"</span>"Tenant onboarding &amp; screening"</li>
            <li><span class="ms msf">"check_circle"</span>"Maintenance tracking"</li>
            <li><span class="ms msf">"check_circle"</span>"Payments &amp; ledger"</li>
        </ul>
    });

    view! {
        <WizardShell
            steps=STEPS.to_vec()
            current_idx=current_idx
            persona_pill="Landlord"
            persona_icon="apartment"
            accent_color="#0284c7"
            panel_bg="#0e1c36"
            ctx_headline="Set up your landlord workspace"
            ctx_body=ctx_body
            invite_code=invite_sig
            on_next=on_next
            on_prev=on_prev
            is_last_step=is_last
            next_label=next_label
        >
            // Error banner
            <Show when=move || save_error.get().is_some()>
                <div class="wiz-error-banner">
                    <span class="ms">"warning"</span>
                    {move || save_error.get().unwrap_or_default()}
                </div>
            </Show>

            // ── Step 1: Profile ─────────────────────────────────────────────
            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"person"</span>
                        "Step 1 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Your Profile"</h1>
                    <p class="wiz-s-sub">"Tell us about yourself. This appears on lease documents and tenant communications."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Personal Info"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Jamie"
                                    prop:value=move || first_name.get()
                                    on:input=move |e| first_name.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Rivera"
                                    prop:value=move || last_name.get()
                                    on:input=move |e| last_name.set(event_target_value(&e))/>
                            </div>
                        </div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Phone"</label>
                            <input class="wiz-inp" type="tel" placeholder="+1 (305) 000-0000"
                                prop:value=move || phone.get()
                                on:input=move |e| phone.set(event_target_value(&e))/>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 2: Jurisdiction ────────────────────────────────────────
            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"gavel"</span>
                        "Step 2 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Jurisdiction &amp; License"</h1>
                    <p class="wiz-s-sub">"We use your jurisdiction to surface the right lease templates and compliance requirements."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Primary Jurisdiction"</div>
                        <div class="wiz-f">
                            <label class="wiz-label">"State / Province"</label>
                            <select class="wiz-inp"
                                prop:value=move || jurisdiction.get()
                                on:change=move |e| jurisdiction.set(event_target_value(&e))>
                                <option value="US-FL">"Florida"</option>
                                <option value="US-NY">"New York"</option>
                                <option value="US-CA">"California"</option>
                                <option value="US-TX">"Texas"</option>
                                <option value="US-GA">"Georgia"</option>
                                <option value="US-NC">"North Carolina"</option>
                            </select>
                        </div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Real Estate License # (optional)"</label>
                            <input class="wiz-inp" type="text" placeholder="BK3000000"
                                prop:value=move || license_number.get()
                                on:input=move |e| license_number.set(event_target_value(&e))/>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 3: First Property ──────────────────────────────────────
            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"apartment"</span>
                        "Step 3 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Add Your First Property"</h1>
                    <p class="wiz-s-sub">"You can add more properties and units after setup. Start with one to get your workspace configured."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Property Details"</div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Property Name"</label>
                            <input class="wiz-inp" type="text" placeholder="The Meridian at Brickell"
                                prop:value=move || prop_name.get()
                                on:input=move |e| prop_name.set(event_target_value(&e))/>
                        </div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Street Address"</label>
                            <input class="wiz-inp" type="text" placeholder="123 Oak Street"
                                prop:value=move || prop_address.get()
                                on:input=move |e| prop_address.set(event_target_value(&e))/>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"City"</label>
                                <input class="wiz-inp" type="text" placeholder="Miami"
                                    prop:value=move || prop_city.get()
                                    on:input=move |e| prop_city.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"State"</label>
                                <select class="wiz-inp"
                                    prop:value=move || prop_state.get()
                                    on:change=move |e| prop_state.set(event_target_value(&e))>
                                    <option>"FL"</option><option>"NY"</option>
                                    <option>"CA"</option><option>"TX"</option>
                                </select>
                            </div>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"Property Type"</label>
                                <select class="wiz-inp"
                                    prop:value=move || prop_type.get()
                                    on:change=move |e| prop_type.set(event_target_value(&e))>
                                    <option value="single_family">"Single Family"</option>
                                    <option value="condo">"Condo"</option>
                                    <option value="multi_family">"Multi-Family"</option>
                                    <option value="commercial">"Commercial"</option>
                                </select>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Units"</label>
                                <select class="wiz-inp"
                                    prop:value=move || unit_count.get()
                                    on:change=move |e| unit_count.set(event_target_value(&e))>
                                    <option>"1"</option><option>"2"</option>
                                    <option>"3"</option><option>"4–10"</option>
                                    <option>"11–50"</option><option>"50+"</option>
                                </select>
                            </div>
                        </div>
                    </div>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Short-Term Rental"</div>
                        <div style="display:flex; align-items:center; justify-content:space-between; padding:4px 0;">
                            <div>
                                <div style="font-size:14px; font-weight:500; color:#0f172a;">"STR Eligible"</div>
                                <div style="font-size:12px; color:#64748b; margin-top:2px;">"Allow short-term bookings on this property"</div>
                            </div>
                            <button
                                class=move || if str_eligible.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| str_eligible.update(|v| *v = !*v)
                            ></button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 4: Payment Rails ───────────────────────────────────────
            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"payments"</span>
                        "Step 4 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Payment Rails"</h1>
                    <p class="wiz-s-sub">"Connect a payout method so rent payments flow directly to your account."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Payout Method"</div>
                        <div style="display:flex; flex-direction:column; gap:10px;">
                            <div class="wiz-pay-option">
                                <span class="ms msf" style="font-size:22px; color:#0284c7;">"account_balance"</span>
                                <div>
                                    <div style="font-size:14px; font-weight:600;">"Bank Account (ACH)"</div>
                                    <div style="font-size:12px; color:#64748b;">"2–3 business days · No fees"</div>
                                </div>
                                <span class="ms msf" style="margin-left:auto; color:#10b981;">"check_circle"</span>
                            </div>
                        </div>
                        <p style="font-size:12px; color:#94a3b8; margin-top:14px;">"You can add or change payout methods later in Settings → Billing."</p>
                    </div>
                </div>
            </Show>

            // ── Step 5: Go Live ─────────────────────────────────────────────
            <Show when=move || current_idx.get() == 4>
                <div class="wiz-anim">
                    <div style="text-align:center; padding:40px 0;">
                        <div style="width:80px; height:80px; background:linear-gradient(135deg,#0e1c36,#1a3356); border-radius:50%; display:flex; align-items:center; justify-content:center; margin:0 auto 24px;">
                            <span class="ms msf" style="font-size:36px; color:#38bdf8;">"rocket_launch"</span>
                        </div>
                        <h1 class="wiz-s-title" style="text-align:center;">"Your Workspace Is Ready"</h1>
                        <p style="font-size:14px; color:#64748b; line-height:1.7; max-width:400px; margin:0 auto 36px;">
                            "Folio is configured and ready. Head to your dashboard to add more properties, invite tenants, and start managing your portfolio."
                        </p>
                        <div class="wiz-card" style="text-align:left;">
                            <div class="wiz-ct">"Quick Actions After Launch"</div>
                            <ul style="list-style:none; padding:0; margin:0; display:flex; flex-direction:column; gap:10px;">
                                <li style="display:flex; align-items:center; gap:10px; font-size:14px;">
                                    <span class="ms msf" style="color:#0284c7;">"apartment"</span>
                                    "Add more properties and units"
                                </li>
                                <li style="display:flex; align-items:center; gap:10px; font-size:14px;">
                                    <span class="ms msf" style="color:#10b981;">"person_add"</span>
                                    "Invite tenants with a single link"
                                </li>
                                <li style="display:flex; align-items:center; gap:10px; font-size:14px;">
                                    <span class="ms msf" style="color:#f59e0b;">"qr_code_2"</span>
                                    "Generate invite codes for units"
                                </li>
                            </ul>
                        </div>
                    </div>
                </div>
            </Show>

        </WizardShell>
    }
}
