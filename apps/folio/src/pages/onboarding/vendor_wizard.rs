// apps/folio/src/pages/onboarding/vendor_wizard.rs
//
// VendorWizard — /onboard/vendor?code=XXXX
//
// Split-panel onboarding wizard for contractors/vendors.
// Replaces the legacy VendorOnboard (vendor/onboard.rs) which uses the old
// single-column apply-card layout.
//
// Steps:
//   1. Business & Contact Info
//   2. Trades & Coverage Area
//   3. Credentials & Insurance
//   4. Pricing & Availability
//   5. Ready to Go (done card)
//
// The invite code is resolved on mount and shown in the WizardShell context panel
// (landlord/PMC name, property context).

use leptos::prelude::*;
use crate::components::wizard_shell::{ResolvedInviteCode, WizardShell, WizardStepDesc, resolve_invite_code};

// ── Step definitions ──────────────────────────────────────────────────────────

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc { id: "business",     label: "Business & Contact",   skippable: false },
    WizardStepDesc { id: "trades",       label: "Trades & Coverage",    skippable: false },
    WizardStepDesc { id: "credentials",  label: "Credentials",          skippable: true  },
    WizardStepDesc { id: "pricing",      label: "Pricing",              skippable: false },
    WizardStepDesc { id: "done",         label: "Ready to Go",          skippable: false },
];

// ── Trade constants ───────────────────────────────────────────────────────────

const TRADES: &[(&str, &str, &str)] = &[
    ("electrical",   "electrical_services", "Electrical"),
    ("plumbing",     "water_damage",        "Plumbing"),
    ("hvac",         "ac_unit",             "HVAC"),
    ("painting",     "format_paint",        "Painting"),
    ("roofing",      "roofing",             "Roofing"),
    ("carpentry",    "door_front",          "Carpentry"),
    ("cleaning",     "cleaning_services",   "Cleaning"),
    ("landscaping",  "landscape",           "Landscaping"),
    ("flooring",     "tile",                "Flooring"),
    ("pest",         "pest_control",        "Pest Control"),
    ("security",     "security",            "Security"),
    ("general",      "build",               "General"),
];

// ── Server function — submit vendor profile ────────────────────────────────────

#[server(SubmitVendorProfile, "/api")]
pub async fn submit_vendor_profile(
    invite_code: String,
    business_name: String,
    contact_first: String,
    contact_last: String,
    email: String,
    phone: String,
    trades: String,
    coverage: String,
    license_number: String,
    hourly_rate: String,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let payload = serde_json::json!({
        "invite_code":   invite_code,
        "business_name": business_name,
        "contact_first": contact_first,
        "contact_last":  contact_last,
        "email":         email,
        "phone":         phone,
        "trades":        trades,
        "coverage":      coverage,
        "license_number": if license_number.is_empty() { None::<String> } else { Some(license_number) },
        "hourly_rate":   hourly_rate,
    });

    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/folio/vendor/profile",
        &token,
        None,
        &payload,
    )
    .await
    .map(|_| ())
    .map_err(server_fn::error::ServerFnError::new)
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn VendorWizard() -> impl IntoView {
    let query    = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());

    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));
    Effect::new(move |_| {
        if let Some(Ok(r)) = code_resource.get() {
            invite_sig.set(r);
        }
    });

    let current_idx = RwSignal::new(0usize);
    let total       = STEPS.len();
    let is_last     = Signal::derive(move || current_idx.get() == total - 1);
    let next_label  = Signal::derive(move || {
        if is_last.get() { "Go to Vendor Dashboard" } else { "Continue" }
    });

    // ── Form state ────────────────────────────────────────────────────────────
    let biz_name      = RwSignal::new(String::new());
    let contact_first = RwSignal::new(String::new());
    let contact_last  = RwSignal::new(String::new());
    let email         = RwSignal::new(String::new());
    let phone         = RwSignal::new(String::new());
    let coverage      = RwSignal::new(String::new());
    let license_num   = RwSignal::new(String::new());
    let hourly_rate   = RwSignal::new(String::new());

    // Trade selection as a set of string keys
    let trades_sel: RwSignal<std::collections::HashSet<&'static str>> =
        RwSignal::new(std::collections::HashSet::new());

    // ── Submit state ──────────────────────────────────────────────────────────
    let submitting = RwSignal::new(false);
    let submit_err: RwSignal<Option<String>> = RwSignal::new(None);
    let code_snapshot = StoredValue::new(code_key());

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx + 1 >= total {
            // Final step — submit and navigate to vendor dashboard
            submitting.set(true);
            submit_err.set(None);
            let code = code_snapshot.get_value();
            let trades_str = trades_sel.get().iter().cloned().collect::<Vec<_>>().join(",");
            let bn = biz_name.get();
            let cf = contact_first.get();
            let cl = contact_last.get();
            let em = email.get();
            let ph = phone.get();
            let cv = coverage.get();
            let ln = license_num.get();
            let hr = hourly_rate.get();
            leptos::task::spawn_local(async move {
                match submit_vendor_profile(code, bn, cf, cl, em, ph, trades_str, cv, ln, hr).await {
                    Ok(_) => {
                        let nav = leptos_router::hooks::use_navigate();
                        nav("/v", Default::default());
                    }
                    Err(e) => {
                        submitting.set(false);
                        submit_err.set(Some(e.to_string()));
                    }
                }
            });
        } else {
            current_idx.set(idx + 1);
        }
    });

    let on_prev = Callback::new(move |_| {
        let i = current_idx.get();
        if i > 0 { current_idx.set(i - 1); }
    });

    // Left panel body — shows who invited the vendor if a code is present
    let ctx_body = ViewFn::from(move || {
        let landlord_name = invite_sig.get()
            .as_ref()
            .and_then(|c| c.context.landlord.as_ref())
            .map(|l| l.name.clone());

        view! {
            {if let Some(name) = landlord_name {
                view! {
                    <div style="background:rgba(255,255,255,.06); border:1px solid rgba(255,255,255,.1); border-radius:10px; padding:14px 16px; margin-bottom:20px; display:flex; align-items:center; gap:12px;">
                        <div style="width:38px; height:38px; border-radius:50%; background:linear-gradient(135deg,#0284c7,#0ea5e9); display:flex; align-items:center; justify-content:center; font-size:13px; font-weight:800; color:#fff; flex-shrink:0;">
                            {name.chars().next().map(|c| c.to_string()).unwrap_or_default()}
                        </div>
                        <div>
                            <div style="font-size:10px; font-weight:700; text-transform:uppercase; letter-spacing:.08em; color:rgba(255,255,255,.35); margin-bottom:2px;">"Invited by"</div>
                            <div style="font-size:14px; font-weight:700;">{name.clone()}</div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            <ul class="wiz-ctx-list">
                <li><span class="ms msf">"check_circle"</span>"Receive work order dispatches directly"</li>
                <li><span class="ms msf">"check_circle"</span>"Submit invoices and track payments"</li>
                <li><span class="ms msf">"check_circle"</span>"Build your marketplace profile for new clients"</li>
                <li><span class="ms msf">"check_circle"</span>"Set your rates and availability"</li>
            </ul>
        }
    });

    view! {
        <WizardShell steps=STEPS.to_vec() current_idx=current_idx
            persona_pill="Vendor" persona_icon="handyman"
            accent_color="#0284c7" panel_bg="#0d1421"
            ctx_headline="Build your contractor profile"
            ctx_body=ctx_body invite_code=invite_sig
            on_next=on_next on_prev=on_prev
            is_last_step=is_last next_label=next_label>

            // Error banner
            <Show when=move || submit_err.get().is_some()>
                <div style="background:#ffdad6; border:1px solid rgba(186,26,26,.3); border-radius:10px; padding:12px 16px; margin-bottom:24px; font-size:13px; color:#93000a; display:flex; align-items:center; gap:8px;">
                    <span class="ms" style="font-size:16px;">"warning"</span>
                    <span>{move || submit_err.get().unwrap_or_default()}</span>
                </div>
            </Show>

            // ── Step 1: Business & Contact ────────────────────────────────────
            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"business_center"</span>"Step 1 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Business & Contact Info"</h1>
                    <p class="wiz-s-sub">"Tell us about your contracting business. This appears on work orders and invoices."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Business Details"</div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Business Name"</label>
                            <input class="wiz-inp" type="text" placeholder="ProFix Services LLC"
                                prop:value=move || biz_name.get()
                                on:input=move |e| biz_name.set(event_target_value(&e))/>
                        </div>
                    </div>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Primary Contact"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Marco"
                                    prop:value=move || contact_first.get()
                                    on:input=move |e| contact_first.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Santos"
                                    prop:value=move || contact_last.get()
                                    on:input=move |e| contact_last.set(event_target_value(&e))/>
                            </div>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"Phone"</label>
                                <input class="wiz-inp" type="tel" placeholder="+1 (555) 000-0000"
                                    prop:value=move || phone.get()
                                    on:input=move |e| phone.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Dispatch Email"</label>
                                <input class="wiz-inp" type="email" placeholder="dispatch@profixservices.com"
                                    prop:value=move || email.get()
                                    on:input=move |e| email.set(event_target_value(&e))/>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 2: Trades & Coverage ─────────────────────────────────────
            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"construction"</span>"Step 2 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Trades & Coverage Area"</h1>
                    <p class="wiz-s-sub">"Select your trade specialties and the area you serve. This determines which work orders you receive."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Trade Specialties"</div>
                        <div style="display:grid; grid-template-columns:repeat(3,1fr); gap:10px; margin-bottom:4px;">
                            {TRADES.iter().map(|(id, icon, label)| {
                                let cid = *id;
                                let ico = *icon;
                                let lbl = *label;
                                view! {
                                    <button
                                        style=move || {
                                            let sel = trades_sel.get().contains(cid);
                                            if sel {
                                                "border:1.5px solid #0284c7; border-radius:8px; padding:10px 8px; cursor:pointer; background:rgba(2,132,199,.06); display:flex; flex-direction:column; align-items:center; gap:4px; font-family:'Inter',sans-serif; transition:.15s;"
                                            } else {
                                                "border:1.5px solid #e2e8f0; border-radius:8px; padding:10px 8px; cursor:pointer; background:#fff; display:flex; flex-direction:column; align-items:center; gap:4px; font-family:'Inter',sans-serif; transition:.15s;"
                                            }
                                        }
                                        on:click=move |_| trades_sel.update(|s| {
                                            if s.contains(cid) { s.remove(cid); } else { s.insert(cid); }
                                        })
                                    >
                                        <span class="ms msf" style=move || {
                                            if trades_sel.get().contains(cid) { "font-size:22px; color:#0284c7;" }
                                            else { "font-size:22px; color:#94a3b8;" }
                                        }>{ico}</span>
                                        <span style="font-size:11px; font-weight:600; color:#0d1421;">{lbl}</span>
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Coverage Radius"</div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Based From (ZIP or city)"</label>
                            <input class="wiz-inp" type="text" placeholder="Miami, FL 33101"
                                prop:value=move || coverage.get()
                                on:input=move |e| coverage.set(event_target_value(&e))/>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 3: Credentials ───────────────────────────────────────────
            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"verified"</span>"Step 3 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Credentials & Insurance"</h1>
                    <p class="wiz-s-sub">"Verified contractors get higher placement in work order matching. Upload what you have — you can add more later."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Business License"</div>
                        <div class="wiz-f">
                            <label class="wiz-label">"License Number " <span style="font-size:10px; color:#94a3b8; text-transform:none; letter-spacing:0; font-weight:400;">"(optional)"</span></label>
                            <input class="wiz-inp" type="text" placeholder="CGC-0000000"
                                prop:value=move || license_num.get()
                                on:input=move |e| license_num.set(event_target_value(&e))/>
                        </div>
                    </div>
                    <div class="wiz-card">
                        <div class="wiz-ct">"General Liability Insurance"</div>
                        <p style="font-size:13px; color:#64748b;">"Certificate of insurance upload will be available in a future update. Your application will be reviewed and you'll be asked to provide it during vetting."</p>
                    </div>
                </div>
            </Show>

            // ── Step 4: Pricing & Availability ────────────────────────────────
            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"paid"</span>"Step 4 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Pricing & Availability"</h1>
                    <p class="wiz-s-sub">"Set your preferred billing model and working hours. These can be overridden per job."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Rate & Response"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"Hourly Rate (base)"</label>
                                <input class="wiz-inp" type="text" placeholder="e.g. $85/hr"
                                    prop:value=move || hourly_rate.get()
                                    on:input=move |e| hourly_rate.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Typical Response"</label>
                                <select class="wiz-inp">
                                    <option>"Within 2 hours"</option>
                                    <option selected>"Same day"</option>
                                    <option>"Next day"</option>
                                    <option>"Within 48 hours"</option>
                                </select>
                            </div>
                        </div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Emergency / After-Hours Availability"</label>
                            <select class="wiz-inp">
                                <option>"Yes – 24/7 available"</option>
                                <option selected>"Yes – weekends and evenings"</option>
                                <option>"No – business hours only"</option>
                            </select>
                        </div>
                    </div>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Payment Preference"</div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Preferred Payment Method"</label>
                            <select class="wiz-inp">
                                <option>"ACH Bank Transfer"</option>
                                <option selected>"Platform Credit (via Folio)"</option>
                                <option>"Check"</option>
                            </select>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 5: Done ──────────────────────────────────────────────────
            <Show when=move || current_idx.get() == 4>
                <div class="wiz-anim" style="text-align:center; padding:40px 0;">
                    <div style="background:linear-gradient(135deg,#0d1421,#0f2744); color:#fff; border-radius:12px; padding:36px; text-align:center; position:relative; overflow:hidden; margin-bottom:14px;">
                        <div style="position:relative; z-index:1;">
                            <div style="width:68px; height:68px; background:rgba(2,132,199,.12); border:2px solid rgba(2,132,199,.35); border-radius:50%; display:flex; align-items:center; justify-content:center; margin:0 auto 18px;">
                                <span class="ms msf" style="font-size:30px; color:#38bdf8;">"handyman"</span>
                            </div>
                            <div style="font-size:22px; font-weight:800; margin-bottom:6px;">"Profile Live"</div>
                            <div style="font-size:13px; color:rgba(255,255,255,.55);">
                                {move || format!("{} · {} trade{}", biz_name.get(), trades_sel.get().len(), if trades_sel.get().len() == 1 { "" } else { "s" })}
                            </div>
                        </div>
                    </div>
                    <div class="wiz-card" style="text-align:left;">
                        <div class="wiz-ct">"What happens next"</div>
                        <div style="display:flex; align-items:flex-start; gap:14px; padding:14px 0; border-bottom:1px solid #e2e8f0;">
                            <div style="width:36px; height:36px; background:rgba(2,132,199,.1); border-radius:8px; display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                                <span class="ms msf" style="font-size:18px; color:#0284c7;">"notifications"</span>
                            </div>
                            <div>
                                <div style="font-size:14px; font-weight:600;">"Work orders arrive by email and in-app"</div>
                                <div style="font-size:12px; color:#64748b; margin-top:2px;">"Accept, decline, or request more info on each dispatch"</div>
                            </div>
                        </div>
                        <div style="display:flex; align-items:flex-start; gap:14px; padding:14px 0; border-bottom:1px solid #e2e8f0;">
                            <div style="width:36px; height:36px; background:rgba(2,132,199,.1); border-radius:8px; display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                                <span class="ms msf" style="font-size:18px; color:#0284c7;">"receipt_long"</span>
                            </div>
                            <div>
                                <div style="font-size:14px; font-weight:600;">"Submit invoices directly from the job"</div>
                                <div style="font-size:12px; color:#64748b; margin-top:2px;">"Attach photos, parts receipts, and labor hours"</div>
                            </div>
                        </div>
                        <div style="display:flex; align-items:flex-start; gap:14px; padding:14px 0;">
                            <div style="width:36px; height:36px; background:rgba(2,132,199,.1); border-radius:8px; display:flex; align-items:center; justify-content:center; flex-shrink:0;">
                                <span class="ms msf" style="font-size:18px; color:#0284c7;">"storefront"</span>
                            </div>
                            <div>
                                <div style="font-size:14px; font-weight:600;">"Your marketplace profile is searchable"</div>
                                <div style="font-size:12px; color:#64748b; margin-top:2px;">"Other landlords on Folio can discover and invite you"</div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </WizardShell>
    }
}
