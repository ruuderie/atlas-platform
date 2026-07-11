// apps/folio/src/pages/onboarding/landlord_wizard.rs
//
// LandlordWizard — /onboard/landlord
//
// 5 steps mirroring wiz_landlord_onboard/code.html:
//   1. Your Profile
//   2. Portfolio Setup
//   3. First Property
//   4. Workspace Settings
//   5. Ready to Launch
//
// Invite code support: if ?code= is in the URL query string, the wizard
// pre-populates the context panel with the resolved landlord / entity info.
//
// Email OTP is handled by WizardShell as a pre-auth gate (not a wizard step).

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::wizard_shell::{
    resolve_invite_code, ResolvedInviteCode, WizardShell, WizardStepDesc,
};
use crate::pages::onboarding::invite_codes_client::accept_invite_code;

// ── Server function — submit landlord onboarding ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LandlordDraft {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub jurisdiction_code: Option<String>,
    pub license_number: Option<String>,
    pub completed_steps: Vec<String>,
}

#[server(GetLandlordDraft, "/api")]
pub async fn get_landlord_draft() -> Result<LandlordDraft, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<LandlordDraft>(
        "/api/folio/onboarding/draft",
        &token,
        None,
    )
    .await
    .or_else(|_| Ok(LandlordDraft::default()))
}

#[server(SaveLandlordProfile, "/api")]
pub async fn save_landlord_profile(
    first_name: String,
    last_name: String,
    phone: String,
    jurisdiction_code: String,
    license_number: String,
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
        "/api/folio/onboarding/submit",
        &token,
        None,
        &payload,
    )
    .await
    .map(|_| ())
    .map_err(server_fn::error::ServerFnError::new)
}

#[server(SaveLandlordProperty, "/api")]
pub async fn save_landlord_property(
    property_name: String,
    property_address: String,
    property_city: String,
    property_state: String,
    property_type: String,
    unit_count: String,
    str_eligible: bool,
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
        "/api/folio/onboarding/submit",
        &token,
        None,
        &payload,
    )
    .await
    .map(|_| ())
    .map_err(server_fn::error::ServerFnError::new)
}

fn country_to_jurisdiction(country: &str) -> String {
    match country {
        "CA" => "CA-ON".to_string(),
        "BR" => "BR-SP".to_string(),
        "GB" => "GB-ENG".to_string(),
        _ => "US-FL".to_string(),
    }
}

// ── Steps (labels match wiz_landlord_onboard) ─────────────────────────────────

const STEPS: &[WizardStepDesc] = &[
    WizardStepDesc {
        id: "profile",
        label: "Your Profile",
        skippable: false,
    },
    WizardStepDesc {
        id: "portfolio",
        label: "Portfolio Setup",
        skippable: false,
    },
    WizardStepDesc {
        id: "property",
        label: "First Property",
        skippable: false,
    },
    WizardStepDesc {
        id: "workspace",
        label: "Workspace Settings",
        skippable: true,
    },
    WizardStepDesc {
        id: "launch",
        label: "Ready to Launch",
        skippable: false,
    },
];

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn LandlordWizard() -> impl IntoView {
    let query = leptos_router::hooks::use_query_map();
    let code_key = move || query.with(|q| q.get("code").map(|s| s.to_string()).unwrap_or_default());

    let invite_sig: RwSignal<Option<ResolvedInviteCode>> = RwSignal::new(None);
    let code_resource = Resource::new(code_key, |code| resolve_invite_code(code));

    Effect::new(move |_| {
        if let Some(Ok(resolved)) = code_resource.get() {
            invite_sig.set(resolved);
        }
    });

    let current_idx = RwSignal::new(0usize);
    let total = STEPS.len();

    let is_last = Signal::derive(move || current_idx.get() == total - 1);
    let next_label = Signal::derive(move || {
        if is_last.get() {
            "Launch Folio"
        } else {
            "Continue"
        }
    });

    // Profile
    let first_name = RwSignal::new(String::new());
    let last_name = RwSignal::new(String::new());
    let display_name = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let account_type = RwSignal::new("individual".to_string()); // individual | company

    // Portfolio
    let business_name = RwSignal::new(String::new());
    let country = RwSignal::new("US".to_string());
    let currency = RwSignal::new("USD".to_string());
    let portfolio_size = RwSignal::new("1-5".to_string());
    let type_ltr = RwSignal::new(true);
    let type_str = RwSignal::new(false);
    let type_commercial = RwSignal::new(false);

    // Property
    let prop_name = RwSignal::new(String::new());
    let prop_address = RwSignal::new(String::new());
    let prop_city = RwSignal::new(String::new());
    let prop_state = RwSignal::new("FL".to_string());
    let prop_postal = RwSignal::new(String::new());
    let prop_type = RwSignal::new("apartment".to_string());
    let unit_count = RwSignal::new("1".to_string());
    let beds = RwSignal::new("2".to_string());
    let monthly_rent = RwSignal::new(String::new());
    let str_eligible = RwSignal::new(false);

    // Workspace
    let notify_maint = RwSignal::new(true);
    let notify_rent = RwSignal::new(true);
    let notify_lease = RwSignal::new(true);
    let notify_str = RwSignal::new(false);
    let enable_str = RwSignal::new(false);
    let list_network = RwSignal::new(false);

    let saving: RwSignal<bool> = RwSignal::new(false);
    let save_error: RwSignal<Option<String>> = RwSignal::new(None);

    let draft = Resource::new(|| (), |_| get_landlord_draft());
    Effect::new(move |_| {
        if let Some(Ok(d)) = draft.get() {
            if let Some(v) = d.first_name {
                first_name.set(v.clone());
                display_name.update(|dn| {
                    if dn.is_empty() {
                        *dn = v;
                    }
                });
            }
            if let Some(v) = d.last_name {
                last_name.set(v);
            }
            if let Some(v) = d.phone {
                phone.set(v);
            }
        }
    });

    let on_next = Callback::new(move |_| {
        let idx = current_idx.get();
        if idx == 0 {
            let f = first_name.get();
            let l = last_name.get();
            let ph = phone.get();
            let j = country_to_jurisdiction(&country.get());
            saving.set(true);
            save_error.set(None);
            leptos::task::spawn_local(async move {
                match save_landlord_profile(f, l, ph, j, String::new()).await {
                    Ok(_) => {
                        saving.set(false);
                        current_idx.set(idx + 1);
                    }
                    Err(e) => {
                        saving.set(false);
                        save_error.set(Some(e.to_string()));
                    }
                }
            });
            return;
        }
        if idx == 1 {
            // Portfolio is local + jurisdiction refresh from country
            let f = first_name.get();
            let l = last_name.get();
            let ph = phone.get();
            let j = country_to_jurisdiction(&country.get());
            saving.set(true);
            save_error.set(None);
            leptos::task::spawn_local(async move {
                match save_landlord_profile(f, l, ph, j, String::new()).await {
                    Ok(_) => {
                        saving.set(false);
                        current_idx.set(idx + 1);
                    }
                    Err(e) => {
                        saving.set(false);
                        save_error.set(Some(e.to_string()));
                    }
                }
            });
            return;
        }
        if idx == 2 {
            let n = if prop_name.get().trim().is_empty() {
                format!("{} property", prop_city.get())
            } else {
                prop_name.get()
            };
            let a = prop_address.get();
            let c = prop_city.get();
            let s = prop_state.get();
            let t = prop_type.get();
            let u = unit_count.get();
            let st = str_eligible.get() || type_str.get() || enable_str.get();
            saving.set(true);
            save_error.set(None);
            leptos::task::spawn_local(async move {
                match save_landlord_property(n, a, c, s, t, u, st).await {
                    Ok(_) => {
                        saving.set(false);
                        current_idx.set(idx + 1);
                    }
                    Err(e) => {
                        saving.set(false);
                        save_error.set(Some(e.to_string()));
                    }
                }
            });
            return;
        }
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
        let idx = current_idx.get();
        if idx > 0 {
            current_idx.set(idx - 1);
        }
    });

    let ctx_body = ViewFn::from(|| {
        view! {
            <p class="wiz-ctx-p">
                "Your profile is how tenants, vendors, and your team will recognise you across the platform."
            </p>
            <ul class="wiz-ctx-list">
                <li><span class="ms msf">"check_circle"</span>"Shown on lease documents and communications"</li>
                <li><span class="ms msf">"check_circle"</span>"Displayed to tenants in their portal"</li>
                <li><span class="ms msf">"check_circle"</span>"Used for legal signature attribution"</li>
            </ul>
        }
    });

    view! {
        <WizardShell
            steps=STEPS.to_vec()
            current_idx=current_idx
            persona_pill="Landlord"
            persona_icon="apartment"
            accent_color="#6366f1"
            panel_bg="#0f1117"
            ctx_headline="Let's get to know you"
            ctx_body=ctx_body
            invite_code=invite_sig
            on_next=on_next
            on_prev=on_prev
            is_last_step=is_last
            next_label=next_label
        >
            <Show when=move || save_error.get().is_some()>
                <div class="wiz-error-banner">
                    <span class="ms">"warning"</span>
                    {move || save_error.get().unwrap_or_default()}
                </div>
            </Show>

            // ── Step 1: Your Profile ────────────────────────────────────────
            <Show when=move || current_idx.get() == 0>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#6366f1;">
                        <span class="ms" style="font-size:13px;">"person"</span>
                        "Step 1 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Your Profile"</h1>
                    <p class="wiz-s-sub">"How should people see you in the platform? This appears on leases, comms, and your team's workspace."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Name & Contact"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"First Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Ruud"
                                    prop:value=move || first_name.get()
                                    on:input=move |e| first_name.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Last Name"</label>
                                <input class="wiz-inp" type="text" placeholder="Erie"
                                    prop:value=move || last_name.get()
                                    on:input=move |e| last_name.set(event_target_value(&e))/>
                            </div>
                        </div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Display Name"</label>
                            <input class="wiz-inp" type="text" placeholder="e.g. Ruud Erie or Meridian Property Group"
                                prop:value=move || display_name.get()
                                on:input=move |e| display_name.set(event_target_value(&e))/>
                        </div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Phone"</label>
                            <input class="wiz-inp" type="tel" placeholder="+1 (555) 000-0000"
                                prop:value=move || phone.get()
                                on:input=move |e| phone.set(event_target_value(&e))/>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Account Type"</div>
                        <div class="wiz-og wiz-og2">
                            <button type="button"
                                class=move || if account_type.get() == "individual" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| account_type.set("individual".into())>
                                <span class="ms msf">"person"</span>
                                <div class="wiz-oc-label">"Individual"</div>
                                <div class="wiz-oc-desc">"I manage properties personally"</div>
                            </button>
                            <button type="button"
                                class=move || if account_type.get() == "company" { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| account_type.set("company".into())>
                                <span class="ms msf">"apartment"</span>
                                <div class="wiz-oc-label">"Company / LLC"</div>
                                <div class="wiz-oc-desc">"I operate under a business entity"</div>
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 2: Portfolio Setup ─────────────────────────────────────
            <Show when=move || current_idx.get() == 1>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#6366f1;">
                        <span class="ms" style="font-size:13px;">"apartment"</span>
                        "Step 2 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Your Portfolio"</h1>
                    <p class="wiz-s-sub">"This configures which tools, currencies, and compliance rules apply to your workspace."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Business Details"</div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Business / Brand Name"</label>
                            <input class="wiz-inp" type="text" placeholder="e.g. Meridian Property Group"
                                prop:value=move || business_name.get()
                                on:input=move |e| business_name.set(event_target_value(&e))/>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"Primary Country"</label>
                                <select class="wiz-inp"
                                    prop:value=move || country.get()
                                    on:change=move |e| country.set(event_target_value(&e))>
                                    <option value="US">"United States"</option>
                                    <option value="CA">"Canada"</option>
                                    <option value="GB">"United Kingdom"</option>
                                    <option value="BR">"Brazil"</option>
                                </select>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Default Currency"</label>
                                <select class="wiz-inp"
                                    prop:value=move || currency.get()
                                    on:change=move |e| currency.set(event_target_value(&e))>
                                    <option value="USD">"USD – US Dollar"</option>
                                    <option value="CAD">"CAD – Dollar"</option>
                                    <option value="EUR">"EUR – Euro"</option>
                                    <option value="GBP">"GBP – Pound"</option>
                                    <option value="BRL">"BRL – Real"</option>
                                </select>
                            </div>
                        </div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Portfolio Size"</label>
                            <select class="wiz-inp"
                                prop:value=move || portfolio_size.get()
                                on:change=move |e| portfolio_size.set(event_target_value(&e))>
                                <option value="1-5">"1–5 units"</option>
                                <option value="6-25">"6–25 units"</option>
                                <option value="26-100">"26–100 units"</option>
                                <option value="100+">"100+ units"</option>
                            </select>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Portfolio Type"</div>
                        <div class="wiz-og wiz-og3">
                            <button type="button"
                                class=move || if type_ltr.get() { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| type_ltr.update(|v| *v = !*v)>
                                <span class="ms msf">"key"</span>
                                <div class="wiz-oc-label">"Long-Term"</div>
                                <div class="wiz-oc-desc">"6+ month leases"</div>
                            </button>
                            <button type="button"
                                class=move || if type_str.get() { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| type_str.update(|v| *v = !*v)>
                                <span class="ms msf">"hotel"</span>
                                <div class="wiz-oc-label">"Short-Term"</div>
                                <div class="wiz-oc-desc">"Nightly / weekly"</div>
                            </button>
                            <button type="button"
                                class=move || if type_commercial.get() { "wiz-oc sel" } else { "wiz-oc" }
                                on:click=move |_| type_commercial.update(|v| *v = !*v)>
                                <span class="ms msf">"storefront"</span>
                                <div class="wiz-oc-label">"Commercial"</div>
                                <div class="wiz-oc-desc">"Office / retail"</div>
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 3: First Property ──────────────────────────────────────
            <Show when=move || current_idx.get() == 2>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#6366f1;">
                        <span class="ms" style="font-size:13px;">"home"</span>
                        "Step 3 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Add Your First Property"</h1>
                    <p class="wiz-s-sub">"You can add more later. Starting with one property brings your workspace to life."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Property Address"</div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Street Address"</label>
                            <input class="wiz-inp" type="text" placeholder="123 Main Street"
                                prop:value=move || prop_address.get()
                                on:input=move |e| prop_address.set(event_target_value(&e))/>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"City"</label>
                                <input class="wiz-inp" type="text" placeholder="San Francisco"
                                    prop:value=move || prop_city.get()
                                    on:input=move |e| prop_city.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"State / Province"</label>
                                <input class="wiz-inp" type="text" placeholder="CA"
                                    prop:value=move || prop_state.get()
                                    on:input=move |e| prop_state.set(event_target_value(&e))/>
                            </div>
                        </div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"Postal Code"</label>
                                <input class="wiz-inp" type="text" placeholder="94102"
                                    prop:value=move || prop_postal.get()
                                    on:input=move |e| prop_postal.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Property Name (optional)"</label>
                                <input class="wiz-inp" type="text" placeholder="The Meridian"
                                    prop:value=move || prop_name.get()
                                    on:input=move |e| prop_name.set(event_target_value(&e))/>
                            </div>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Property Type"</div>
                        <div class="wiz-og wiz-og3">
                            {[
                                ("apartment", "apartment", "Apartment"),
                                ("house", "house", "House"),
                                ("multi_unit", "domain", "Multi-Unit"),
                                ("villa", "villa", "Villa / STR"),
                                ("commercial", "storefront", "Commercial"),
                                ("industrial", "warehouse", "Industrial"),
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
                        <div class="wiz-ct">"Unit Info"</div>
                        <div class="wiz-inp-row">
                            <div class="wiz-f">
                                <label class="wiz-label">"Number of Units"</label>
                                <input class="wiz-inp" type="number" min="1"
                                    prop:value=move || unit_count.get()
                                    on:input=move |e| unit_count.set(event_target_value(&e))/>
                            </div>
                            <div class="wiz-f">
                                <label class="wiz-label">"Beds per Unit"</label>
                                <select class="wiz-inp"
                                    prop:value=move || beds.get()
                                    on:change=move |e| beds.set(event_target_value(&e))>
                                    <option value="studio">"Studio"</option>
                                    <option value="1">"1"</option>
                                    <option value="2">"2"</option>
                                    <option value="3">"3"</option>
                                    <option value="4+">"4+"</option>
                                </select>
                            </div>
                        </div>
                        <div class="wiz-f">
                            <label class="wiz-label">"Monthly Rent"</label>
                            <input class="wiz-inp" type="text" placeholder="$2,500"
                                prop:value=move || monthly_rent.get()
                                on:input=move |e| monthly_rent.set(event_target_value(&e))/>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"STR Eligible"</div>
                                <div class="wiz-tr-desc">"Allow short-term bookings on this property"</div>
                            </div>
                            <button type="button"
                                class=move || if str_eligible.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| str_eligible.update(|v| *v = !*v)
                            ></button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 4: Workspace Settings ──────────────────────────────────
            <Show when=move || current_idx.get() == 3>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(99,102,241,.08); color:#6366f1;">
                        <span class="ms" style="font-size:13px;">"settings"</span>
                        "Step 4 of 5"
                    </div>
                    <h1 class="wiz-s-title">"Workspace Settings"</h1>
                    <p class="wiz-s-sub">"Configure notifications, invite your team, and choose platform features. Adjustable any time."</p>

                    <div class="wiz-card">
                        <div class="wiz-ct">"Notifications"</div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Maintenance requests"</div>
                                <div class="wiz-tr-desc">"Notify when tenants submit new requests"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_maint.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_maint.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Rent payment alerts"</div>
                                <div class="wiz-tr-desc">"Payments received, late, or failed"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_rent.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_rent.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Lease expiry reminders"</div>
                                <div class="wiz-tr-desc">"60-day and 30-day advance notices"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_lease.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_lease.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"STR booking inquiries"</div>
                                <div class="wiz-tr-desc">"New requests and guest messages"</div>
                            </div>
                            <button type="button"
                                class=move || if notify_str.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| notify_str.update(|v| *v = !*v)
                            ></button>
                        </div>
                    </div>

                    {
                        use crate::components::network_invite_panel::{AngleCard, NetworkInvitePanel};
                        view! {
                            <NetworkInvitePanel
                                actor_role="landlord"
                                preferred_slug="landlord_invite_peers"
                                angles=vec![
                                    AngleCard {
                                        icon: "apartment",
                                        title: "Fellow landlords & owners",
                                        body: "Share Folio with owners in your circle so you can coordinate vendors and compare notes.",
                                    },
                                    AngleCard {
                                        icon: "handyman",
                                        title: "Trusted contractors",
                                        body: "Invite your plumber, HVAC tech, or cleaner. Dispatch and invoice live on Folio next time.",
                                    },
                                ]
                                section_title="Grow your network".to_string()
                                footnote="Folio works better with people you already trust. Skip if you like. Invite anytime from your dashboard.".to_string()
                                show_history=false
                            />
                        }
                    }

                    <div class="wiz-card">
                        <div class="wiz-ct">"STR & Network"</div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"Enable STR on eligible properties"</div>
                                <div class="wiz-tr-desc">"STR tools appear on properties you mark eligible"</div>
                            </div>
                            <button type="button"
                                class=move || if enable_str.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| enable_str.update(|v| *v = !*v)
                            ></button>
                        </div>
                        <div class="wiz-tr">
                            <div>
                                <div class="wiz-tr-label">"List on Cohost Network"</div>
                                <div class="wiz-tr-desc">"STR listings visible to partner network instances"</div>
                            </div>
                            <button type="button"
                                class=move || if list_network.get() { "wiz-toggle on" } else { "wiz-toggle" }
                                on:click=move |_| list_network.update(|v| *v = !*v)
                            ></button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Step 5: Ready to Launch ─────────────────────────────────────
            <Show when=move || current_idx.get() == 4>
                <div class="wiz-anim">
                    <div class="wiz-s-badge" style="background:rgba(16,185,129,.1); color:#059669;">
                        <span class="ms msf" style="font-size:13px;">"check_circle"</span>
                        "All done!"
                    </div>
                    <h1 class="wiz-s-title">"You're ready to launch"</h1>
                    <p class="wiz-s-sub">"Your Folio workspace is configured. Here's what you can do first."</p>

                    <div class="wiz-card" style="background:linear-gradient(135deg,#0f1117 0%,#1a1b2e 100%);color:#fff;border:none;">
                        <div style="text-align:center;padding:12px 0 4px;">
                            <div style="width:68px;height:68px;background:rgba(16,185,129,.12);border:2px solid rgba(16,185,129,.35);border-radius:50%;display:flex;align-items:center;justify-content:center;margin:0 auto 18px;">
                                <span class="ms msf" style="font-size:32px;color:#10b981;">"verified"</span>
                            </div>
                            <div style="font-size:22px;font-weight:800;margin-bottom:6px;">"Workspace ready"</div>
                            <div style="font-size:13px;color:rgba(255,255,255,.55);">
                                {move || {
                                    let brand = business_name.get();
                                    let brand = if brand.trim().is_empty() { "Your portfolio".to_string() } else { brand };
                                    format!("{brand} · 1 property")
                                }}
                            </div>
                        </div>
                    </div>

                    <div class="wiz-card">
                        <div class="wiz-ct">"What to do next"</div>
                        <div class="wiz-na-row">
                            <span class="ms msf" style="font-size:28px;color:#6366f1;">"description"</span>
                            <div>
                                <div style="font-size:14px;font-weight:600;">"Create a lease"</div>
                                <div style="font-size:12px;color:#64748b;">"Add a tenant to your first property"</div>
                            </div>
                        </div>
                        <div class="wiz-na-row">
                            <span class="ms msf" style="font-size:28px;color:#10b981;">"add_home"</span>
                            <div>
                                <div style="font-size:14px;font-weight:600;">"Add more properties"</div>
                                <div style="font-size:12px;color:#64748b;">"Import from CSV or add individually"</div>
                            </div>
                        </div>
                        <div class="wiz-na-row">
                            <span class="ms msf" style="font-size:28px;color:#f59e0b;">"dashboard"</span>
                            <div>
                                <div style="font-size:14px;font-weight:600;">"Explore your dashboard"</div>
                                <div style="font-size:12px;color:#64748b;">"See your full portfolio at a glance"</div>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>

        </WizardShell>
    }
}
