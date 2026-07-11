// apps/folio/src/pages/onboarding/pmc_wizard.rs
//
// PmcWizard — /onboard/pmc?code=XXXX
//
// Dual-mode wizard for property manager onboarding:
//
//   Mode A — Standalone PMC:   No employer_user_id in invite code.
//             Steps: Company Profile → PM Mode & Billing → Client Portfolios
//                    → Modules → Invite Codes → Done (5 steps)
//
//   Mode B — Landlord-hired PM: employer_user_id present in resolved code.
//             Steps: Your Profile → Review & Accept → Done (2 steps)
//
// Mode is auto-detected from the resolved invite code context.
// Falls back to Standalone if no code is present (e.g. admin-initiated).

use crate::components::wizard_shell::{
    resolve_invite_code, ResolvedInviteCode, WizardShell, WizardStepDesc,
};
use leptos::prelude::*;

// ── Step lists ────────────────────────────────────────────────────────────────

const STANDALONE_STEPS: &[WizardStepDesc] = &[
    WizardStepDesc {
        id: "company",
        label: "Company Profile",
        skippable: false,
    },
    WizardStepDesc {
        id: "mode",
        label: "PM Mode & Billing",
        skippable: false,
    },
    WizardStepDesc {
        id: "clients",
        label: "Client Portfolios",
        skippable: true,
    },
    WizardStepDesc {
        id: "modules",
        label: "Modules & Features",
        skippable: false,
    },
    WizardStepDesc {
        id: "codes",
        label: "Invite Code Setup",
        skippable: false,
    },
];

const HIRED_STEPS: &[WizardStepDesc] = &[
    WizardStepDesc {
        id: "profile",
        label: "Your Profile",
        skippable: false,
    },
    WizardStepDesc {
        id: "accept",
        label: "Review & Accept",
        skippable: false,
    },
];

// ── Server function — accept invite ──────────────────────────────────────────

#[server(AcceptPmcInvite, "/api")]
pub async fn accept_pmc_invite(invite_code: String) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    if invite_code.is_empty() {
        return Ok(()); // no-op for standalone mode (no code to accept)
    }

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    // Resolve invite code UUID from the short code string
    let code_id_result: Result<serde_json::Value, _> = crate::atlas_client::authenticated_get(
        &format!("/api/folio/invite/resolve/{}", invite_code),
        &token,
        None,
    )
    .await;

    let code_id = match code_id_result {
        Ok(v) => v["id"].as_str().unwrap_or_default().to_string(),
        Err(e) => return Err(server_fn::error::ServerFnError::new(e.to_string())),
    };

    if code_id.is_empty() {
        return Err(server_fn::error::ServerFnError::new(
            "Could not resolve invite code id",
        ));
    }

    crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        &format!("/api/folio/invite-codes/{}/accept", code_id),
        &token,
        None,
        &serde_json::json!({}),
    )
    .await
    .map(|_| ())
    .map_err(server_fn::error::ServerFnError::new)
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn PmcWizard() -> impl IntoView {
    let query = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());

    // Resolve the invite code to determine mode and pre-fill employer context
    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));
    Effect::new(move |_| {
        if let Some(Ok(r)) = code_resource.get() {
            invite_sig.set(r);
        }
    });

    // Derive mode from invite code: if code has employer context → hired
    // The employer is surfaced via InviteCodeContext.landlord (re-used as the employer entity).
    let is_hired = Signal::derive(move || {
        invite_sig
            .get()
            .as_ref()
            .map(|c| c.context.landlord.is_some())
            .unwrap_or(false)
    });

    let current_idx = RwSignal::new(0usize);

    // Dynamic step list based on mode
    let steps = Signal::derive(move || {
        if is_hired.get() {
            HIRED_STEPS.to_vec()
        } else {
            STANDALONE_STEPS.to_vec()
        }
    });

    let total = Signal::derive(move || steps.get().len());
    let is_last = Signal::derive(move || current_idx.get() == total.get().saturating_sub(1));

    let next_label = Signal::derive(move || {
        if is_last.get() {
            if is_hired.get() {
                "Accept & Join"
            } else {
                "Launch PM Workspace"
            }
        } else {
            "Continue"
        }
    });

    // ── Form state — Standalone ───────────────────────────────────────────────
    let company_name = RwSignal::new(String::new());
    let dba_name = RwSignal::new(String::new());
    let license_num = RwSignal::new(String::new());
    let license_state = RwSignal::new("FL".to_string());
    let contact_first = RwSignal::new(String::new());
    let contact_last = RwSignal::new(String::new());
    let contact_email = RwSignal::new(String::new());
    let contact_phone = RwSignal::new(String::new());
    let office_address = RwSignal::new(String::new());
    let portfolio_mode = RwSignal::new("full_pmc".to_string());
    let fee_model = RwSignal::new("percent".to_string());
    let default_rate = RwSignal::new("8".to_string());
    let leasing_fee = RwSignal::new(String::new());
    let approval_limit = RwSignal::new("1000".to_string());

    // Modules
    let mod_statements = RwSignal::new(true);
    let mod_tenant = RwSignal::new(true);
    let mod_vendor = RwSignal::new(true);
    let mod_maint = RwSignal::new(true);
    let mod_str = RwSignal::new(false);
    let mod_ota = RwSignal::new(false);
    let mod_crm = RwSignal::new(false);

    // ── Form state — Hired ────────────────────────────────────────────────────
    let pm_first = RwSignal::new(String::new());
    let pm_last = RwSignal::new(String::new());
    let pm_email = RwSignal::new(String::new());
    let pm_phone = RwSignal::new(String::new());
    let pm_title = RwSignal::new("On-site Manager".to_string());
    let terms_ok = RwSignal::new(false);

    // ── Submit state ──────────────────────────────────────────────────────────
    let submitting = RwSignal::new(false);
    let submit_err: RwSignal<Option<String>> = RwSignal::new(None);
    let code_snapshot = StoredValue::new(code_key());

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        let tot = total.get();
        if idx + 1 >= tot {
            // Final step — call accept endpoint, then navigate
            let code = code_snapshot.get_value();
            submitting.set(true);
            submit_err.set(None);
            leptos::task::spawn_local(async move {
                match accept_pmc_invite(code).await {
                    Ok(_) => {
                        let nav = leptos_router::hooks::use_navigate();
                        nav("/l", Default::default());
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
        if i > 0 {
            current_idx.set(i - 1);
        }
    });

    // ── Left panel body — dynamic based on mode ───────────────────────────────
    let ctx_body = ViewFn::from(move || {
        let hired = is_hired.get();
        let employer_name = invite_sig
            .get()
            .as_ref()
            .and_then(|c| c.context.landlord.as_ref())
            .map(|l| l.name.clone())
            .unwrap_or_else(|| "Your Landlord".to_string());
        let asset_count = invite_sig
            .get()
            .as_ref()
            .and_then(|c| c.context.asset_count)
            .unwrap_or(0);

        if hired {
            view! {
                // Hiring landlord context card
                <div style="background:rgba(255,255,255,.06); border:1px solid rgba(255,255,255,.1); border-radius:10px; padding:14px 16px; margin-bottom:20px; display:flex; align-items:center; gap:12px;">
                    <div style="width:38px; height:38px; border-radius:50%; background:linear-gradient(135deg,#0284c7,#0ea5e9); display:flex; align-items:center; justify-content:center; font-size:13px; font-weight:800; color:#fff; flex-shrink:0;">
                        {employer_name.chars().next().map(|c| c.to_string()).unwrap_or_default()}
                    </div>
                    <div>
                        <div style="font-size:10px; font-weight:700; text-transform:uppercase; letter-spacing:.08em; color:rgba(255,255,255,.35); margin-bottom:2px;">"Invited by"</div>
                        <div style="font-size:14px; font-weight:700;">{employer_name.clone()}</div>
                        <div style="font-size:12px; color:rgba(255,255,255,.5); margin-top:1px;">
                            {if asset_count > 0 {
                                format!("Full Portfolio · {} units", asset_count)
                            } else {
                                "Full Portfolio Access".to_string()
                            }}
                        </div>
                    </div>
                </div>
                <ul style="list-style:none; display:flex; flex-direction:column; gap:9px; margin-bottom:22px;">
                    <li style="display:flex; align-items:flex-start; gap:9px; font-size:13px; color:rgba(255,255,255,.6);">
                        <span class="ms msf" style="font-size:17px; color:#34d399; margin-top:1px;">"check_circle"</span>
                        "Access to all landlord properties & tenants"
                    </li>
                    <li style="display:flex; align-items:flex-start; gap:9px; font-size:13px; color:rgba(255,255,255,.6);">
                        <span class="ms msf" style="font-size:17px; color:#34d399; margin-top:1px;">"check_circle"</span>
                        "Maintenance dispatch and vendor coordination"
                    </li>
                    <li style="display:flex; align-items:flex-start; gap:9px; font-size:13px; color:rgba(255,255,255,.6);">
                        <span class="ms msf" style="font-size:17px; color:#34d399; margin-top:1px;">"check_circle"</span>
                        {format!("{} remains the account admin", &employer_name)}
                    </li>
                    <li style="display:flex; align-items:flex-start; gap:9px; font-size:13px; color:rgba(255,255,255,.6);">
                        <span class="ms msf" style="font-size:17px; color:#34d399; margin-top:1px;">"check_circle"</span>
                        "Scope can be adjusted by the landlord at any time"
                    </li>
                </ul>
            }.into_any()
        } else {
            view! {
                <p class="wiz-ctx-p">"Folio PM Edition gives you a full client management layer — manage multiple landlord portfolios from a single seat."</p>
                <ul class="wiz-ctx-list">
                    <li><span class="ms msf">"check_circle"</span>"Manage multiple landlord client portfolios"</li>
                    <li><span class="ms msf">"check_circle"</span>"Generate invite codes per client and unit"</li>
                    <li><span class="ms msf">"check_circle"</span>"Owner statement generation & distribution"</li>
                    <li><span class="ms msf">"check_circle"</span>"Vendor dispatch and maintenance tracking"</li>
                    <li><span class="ms msf">"check_circle"</span>"PMC-branded tenant and owner portals"</li>
                </ul>
            }.into_any()
        }
    });

    view! {
        <WizardShell
            steps=Signal::derive(move || steps.get()).get_untracked()
            current_idx=current_idx
            persona_pill=if is_hired.get_untracked() { "Property Manager" } else { "PMC" }
            persona_icon="corporate_fare"
            accent_color="#0284c7"
            panel_bg="#0d1421"
            ctx_headline=if is_hired.get_untracked() { "Accept your PM role" } else { "Set up your PM workspace" }
            ctx_body=ctx_body
            invite_code=invite_sig
            on_next=on_next
            on_prev=on_prev
            is_last_step=is_last
            next_label=next_label
        >
            // ── ERROR BANNER ──────────────────────────────────────────────────
            <Show when=move || submit_err.get().is_some()>
                <div style="background:#ffdad6; border:1px solid rgba(186,26,26,.3); border-radius:10px; padding:12px 16px; margin-bottom:24px; font-size:13px; color:#93000a; display:flex; align-items:center; gap:8px;">
                    <span class="ms" style="font-size:16px;">"warning"</span>
                    <span>{move || submit_err.get().unwrap_or_default()}</span>
                </div>
            </Show>

            // ════ STANDALONE STEPS ════

            // Step 1: Company Profile
            <Show when=move || !is_hired.get() && current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"corporate_fare"</span>"Step 1 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Company Profile"</h1>
                    <p class="wiz-s-sub">"Your PMC profile appears on owner statements, tenant portals, and all Folio communications."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Company Identity"</div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Company Name"</label>
                            <input class="wiz-inp" type="text" placeholder="Meridian Property Group"
                                prop:value=move || company_name.get()
                                on:input=move |e| company_name.set(event_target_value(&e))/>
                        </div>
                        <div class="wiz-f">
                            <label class="wiz-label">"DBA / Trading Name (if different)"</label>
                            <input class="wiz-inp" type="text" placeholder="Meridian PM"
                                prop:value=move || dba_name.get()
                                on:input=move |e| dba_name.set(event_target_value(&e))/>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"License Number"</label>
                                <input class="wiz-inp" type="text" placeholder="CAM-00000000"
                                    prop:value=move || license_num.get()
                                    on:input=move |e| license_num.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"License State"</label>
                                <select class="wiz-inp"
                                    prop:value=move || license_state.get()
                                    on:change=move |e| license_state.set(event_target_value(&e))>
                                    <option value="FL">"FL"</option>
                                    <option value="NY">"NY"</option>
                                    <option value="CA">"CA"</option>
                                    <option value="TX">"TX"</option>
                                </select>
                            </div>
                        </div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Office Address"</label>
                            <input class="wiz-inp" type="text" placeholder="100 Brickell Ave, Miami, FL"
                                prop:value=move || office_address.get()
                                on:input=move |e| office_address.set(event_target_value(&e))/>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Primary Contact"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Sarah"
                                    prop:value=move || contact_first.get()
                                    on:input=move |e| contact_first.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Chen"
                                    prop:value=move || contact_last.get()
                                    on:input=move |e| contact_last.set(event_target_value(&e))/>
                            </div>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"Email"</label>
                                <input class="wiz-inp" type="email" placeholder="sarah@meridianpm.com"
                                    prop:value=move || contact_email.get()
                                    on:input=move |e| contact_email.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Phone"</label>
                                <input class="wiz-inp" type="tel" placeholder="+1 (305) 000-0000"
                                    prop:value=move || contact_phone.get()
                                    on:input=move |e| contact_phone.set(event_target_value(&e))/>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // Step 2: PM Mode & Billing
            <Show when=move || !is_hired.get() && current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"settings"</span>"Step 2 of 5"
                    </div>
                    <h1 class="wiz-s-title">"PM Mode & Billing"</h1>
                    <p class="wiz-s-sub">"Choose how your workspace operates. These defaults apply to all clients and can be overridden per-account."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Portfolio Mode"</div>
                        <div class="wiz-og wiz-og2">
                            <button type="button"
                                class=move || if portfolio_mode.get() == "full_pmc" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| portfolio_mode.set("full_pmc".into())>
                                <span class="ms msf">"corporate_fare"</span>
                                <div class="wiz-oc-label">"Full PMC"</div>
                                <div class="wiz-oc-desc">"Multiple landlord clients under one seat"</div>
                            </button>
                            <button type="button"
                                class=move || if portfolio_mode.get() == "single" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| portfolio_mode.set("single".into())>
                                <span class="ms msf">"home"</span>
                                <div class="wiz-oc-label">"Single Portfolio"</div>
                                <div class="wiz-oc-desc">"Manage one landlord's properties"</div>
                            </button>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Fee Structure"</div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Management Fee Model"</label>
                            <select class="wiz-inp"
                                prop:value=move || fee_model.get()
                                on:change=move |e| fee_model.set(event_target_value(&e))>
                                <option value="percent">"% of monthly collected rent"</option>
                                <option value="flat">"Flat monthly fee per unit"</option>
                                <option value="hybrid">"Hybrid (flat + performance)"</option>
                            </select>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f"><label class="wiz-label">"Default Rate"</label>
                                <input class="wiz-inp" type="text" placeholder="8%"
                                    prop:value=move || default_rate.get()
                                    on:input=move |e| default_rate.set(event_target_value(&e))/></div>
                            <div class="wiz-f"><label class="wiz-label">"Leasing Fee (one-time, per placement)"</label>
                                <input class="wiz-inp" type="text" placeholder="50% of first month"
                                    prop:value=move || leasing_fee.get()
                                    on:input=move |e| leasing_fee.set(event_target_value(&e))/></div>
                        </div>
                    </div>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Maintenance Authorization"</div>
                        <p style="font-size:13px; color:#64748b; margin-bottom:14px;">"Repairs above this threshold require owner approval before you can dispatch."</p>
                        <div class="wiz-f">
                            <label class="wiz-label">"Default Approval Threshold"</label>
                            <select class="wiz-inp"
                                prop:value=move || approval_limit.get()
                                on:change=move |e| approval_limit.set(event_target_value(&e))>
                                <option value="250">"$250"</option>
                                <option value="500">"$500"</option>
                                <option value="1000">"$1,000"</option>
                                <option value="2500">"$2,500"</option>
                            </select>
                        </div>
                    </div>
                </div>
            </Show>

            // Step 3: Client Portfolios
            <Show when=move || !is_hired.get() && current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"people"</span>"Step 3 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Invite your clients"</h1>
                    <p class="wiz-s-sub">"Bring landlord clients onto Folio so they get an Owner portal with statements and reporting. Your PMC stays the hub."</p>
                    {
                        use crate::components::network_invite_panel::{AngleCard, NetworkInvitePanel};
                        view! {
                            <NetworkInvitePanel
                                actor_role="property_manager"
                                preferred_slug="pmc_invite_clients"
                                angles=vec![
                                    AngleCard {
                                        icon: "apartment",
                                        title: "Existing owner clients",
                                        body: "Invite landlords you already manage. They see statements here while you keep operations centralized.",
                                    },
                                    AngleCard {
                                        icon: "campaign",
                                        title: "Prospects & referrals",
                                        body: "When pitching a new owner, send a Folio invite instead of a PDF.",
                                    },
                                ]
                                section_title="Invite a new client".to_string()
                                send_label="Send Owner Invite".to_string()
                                show_note=true
                                allow_multi=false
                                show_history=false
                            />
                        }
                    }
                </div>
            </Show>

            // Step 4: Modules
            <Show when=move || !is_hired.get() && current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"extension"</span>"Step 4 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Modules & Features"</h1>
                    <p class="wiz-s-sub">"Enable the modules your PMC needs. All can be toggled per-client later from the client settings panel."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Core PM Modules"</div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Owner Statements"</div>
                                <div class="wiz-tr-desc">"Monthly owner reporting and distributions"</div>
                            </div>
                            <button type="button"
                                class=move || if mod_statements.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| mod_statements.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Tenant Portal"</div>
                                <div class="wiz-tr-desc">"Rent pay, maintenance, lease docs"</div>
                            </div>
                            <button type="button"
                                class=move || if mod_tenant.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| mod_tenant.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Vendor Dispatch"</div>
                                <div class="wiz-tr-desc">"Work order matching and invoices"</div>
                            </div>
                            <button type="button"
                                class=move || if mod_vendor.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| mod_vendor.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Maintenance Tracker"</div>
                                <div class="wiz-tr-desc">"Request triage and SLA tracking"</div>
                            </div>
                            <button type="button"
                                class=move || if mod_maint.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| mod_maint.update(|v| *v = !*v)
                            ></button>
                        </div>
                    </div>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Optional Add-ons"</div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"STR Booking Engine"</div>
                                <div class="wiz-tr-desc">"Direct bookings for short-term units"</div>
                            </div>
                            <button type="button"
                                class=move || if mod_str.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| mod_str.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"OTA Syndication"</div>
                                <div class="wiz-tr-desc">"Airbnb / VRBO channel sync"</div>
                            </div>
                            <button type="button"
                                class=move || if mod_ota.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| mod_ota.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Lease Application CRM"</div>
                                <div class="wiz-tr-desc">"Applicant pipeline and screening"</div>
                            </div>
                            <button type="button"
                                class=move || if mod_crm.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| mod_crm.update(|v| *v = !*v)
                            ></button>
                        </div>
                    </div>
                </div>
            </Show>

            // Step 5: Done / Invite Codes
            <Show when=move || !is_hired.get() && current_idx.get() == 4>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(2,132,199,.08); color:#0369a1;">
                        <span class="ms" style="font-size:13px;">"qr_code_2"</span>"Step 5 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Invite Code Setup"</h1>
                    <p class="wiz-s-sub">"Generate codes so tenants, vendors, and co-managers can self-onboard pre-linked to your properties."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Auto-generated Codes"</div>
                        <p style="font-size:13px; color:#64748b; margin-bottom:14px;">"We've pre-generated codes for your current units. You can generate more from the Team page after setup."</p>
                        <div style="background:#f4f5f9; border:1px solid #e2e8f0; border-radius:8px; padding:13px; display:flex; align-items:center; gap:12px; margin-bottom:8px;">
                            <span class="ms msf" style="font-size:18px; color:#0284c7;">"door_front"</span>
                            <div style="flex:1;">
                                <div style="font-size:13px; font-weight:600;">"Portfolio — Vendor Signup"</div>
                                <div style="font-size:11px; color:#64748b; margin-top:2px;">"Role: Vendor · Multi-use · No expiry"</div>
                            </div>
                            <span style="font-size:11px; font-weight:700; font-family:monospace; letter-spacing:.08em; padding:5px 10px; border-radius:6px; background:rgba(2,132,199,.08); color:#0369a1;">
                                {move || format!("MPG-{}", company_name.get().chars().take(3).collect::<String>().to_uppercase())}
                            </span>
                        </div>
                    </div>
                    // Completion summary
                    <div style="background:linear-gradient(135deg,#0d1421,#0f2744); color:#fff; border-radius:12px; padding:32px; text-align:center; position:relative; overflow:hidden;">
                        <div style="position:relative; z-index:1;">
                            <div style="width:68px; height:68px; background:rgba(2,132,199,.12); border:2px solid rgba(2,132,199,.35); border-radius:50%; display:flex; align-items:center; justify-content:center; margin:0 auto 18px;">
                                <span class="ms msf" style="font-size:30px; color:#38bdf8;">"corporate_fare"</span>
                            </div>
                            <div style="font-size:22px; font-weight:800; margin-bottom:6px;">"PMC Workspace Ready"</div>
                            <div style="font-size:13px; color:rgba(255,255,255,.55);">
                                {move || format!("{} · Full PMC Mode", company_name.get())}
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

            // ════ HIRED STEPS ════

            // Hired Step 1: Your Profile
            <Show when=move || is_hired.get() && current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(16,185,129,.08); color:#047857;">
                        <span class="ms" style="font-size:13px;">"person"</span>"Step 1 of 2"
                    </div>
                    <h1 class="wiz-s-title">"Your Profile"</h1>
                    <p class="wiz-s-sub">"Tell your landlord a little about yourself. This will appear on their property management dashboard."</p>
                    <div class="wiz-card">
                        <div class="wiz-ct">"Personal Details"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Marcus"
                                    prop:value=move || pm_first.get()
                                    on:input=move |e| pm_first.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Kim"
                                    prop:value=move || pm_last.get()
                                    on:input=move |e| pm_last.set(event_target_value(&e))/>
                            </div>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"Email"</label>
                                <input class="wiz-inp" type="email" placeholder="marcus@email.com"
                                    prop:value=move || pm_email.get()
                                    on:input=move |e| pm_email.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Phone"</label>
                                <input class="wiz-inp" type="tel" placeholder="+1 (312) 000-0000"
                                    prop:value=move || pm_phone.get()
                                    on:input=move |e| pm_phone.set(event_target_value(&e))/>
                            </div>
                        </div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Your Title"</label>
                            <input class="wiz-inp" type="text" placeholder="e.g. On-site Manager, Property Manager"
                                prop:value=move || pm_title.get()
                                on:input=move |e| pm_title.set(event_target_value(&e))/>
                        </div>
                    </div>
                </div>
            </Show>

            // Hired Step 2: Review & Accept
            <Show when=move || is_hired.get() && current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(16,185,129,.08); color:#047857;">
                        <span class="ms" style="font-size:13px;">"handshake"</span>"Step 2 of 2"
                    </div>
                    <h1 class="wiz-s-title">"Review & Accept"</h1>
                    <p class="wiz-s-sub">"Review your role before you confirm. Your landlord will be notified when you accept."</p>

                    <div style="background:linear-gradient(135deg,rgba(16,185,129,.08),rgba(2,132,199,.05)); border:1.5px solid rgba(16,185,129,.2); border-radius:12px; padding:24px; text-align:center; margin-bottom:14px;">
                        <div style="width:56px; height:56px; border-radius:50%; background:rgba(16,185,129,.12); border:2px solid rgba(16,185,129,.25); display:flex; align-items:center; justify-content:center; margin:0 auto 14px;">
                            <span class="ms msf" style="font-size:28px; color:#059669;">"verified"</span>
                        </div>
                        <div style="font-size:18px; font-weight:800; margin-bottom:6px;">"Property Manager Invite"</div>
                        <div style="font-size:13px; color:#64748b; line-height:1.6; max-width:360px; margin:0 auto 14px;">
                            {move || {
                                let title = pm_title.get();
                                let employer = invite_sig.get()
                                    .as_ref()
                                    .and_then(|c| c.context.landlord.as_ref())
                                    .map(|l| l.name.clone())
                                    .unwrap_or_else(|| "Your Landlord".to_string());
                                format!("You're joining {}'s Folio workspace as {}. They remain the account admin and can adjust or revoke access at any time.", employer, title)
                            }}
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Terms"</div>
                        <label style="display:flex; align-items:flex-start; gap:12px; cursor:pointer; font-size:13px; color:#64748b; line-height:1.6;">
                            <input type="checkbox" style="margin-top:3px; flex-shrink:0;"
                                prop:checked=move || terms_ok.get()
                                on:change=move |ev: web_sys::Event| {
                                    let el = event_target::<web_sys::HtmlInputElement>(&ev);
                                    terms_ok.set(el.checked());
                                }/>
                            "I understand that the landlord is the account admin and that my access is scoped to managing their portfolio within Folio. I agree to the platform terms of service."
                        </label>
                    </div>
                </div>
            </Show>
        </WizardShell>
    }
}
