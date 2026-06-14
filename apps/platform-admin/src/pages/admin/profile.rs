use leptos::prelude::*;
use crate::api::models::UserInfo;
use crate::api::profile::update_email;
use crate::app::GlobalToast;
use shared_ui::components::auth::passkey_manager::ManagePasskeys;

#[component]
pub fn Settings() -> impl IntoView {
    let user = use_context::<ReadSignal<Option<UserInfo>>>().expect("user context");
    let set_user = use_context::<WriteSignal<Option<UserInfo>>>().expect("set user context");
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");

    let active_tab = RwSignal::new("profile".to_string());

    // Profile inputs
    let first_name_input = RwSignal::new(String::new());
    let last_name_input = RwSignal::new(String::new());
    let email_input = RwSignal::new(String::new());
    let phone_input = RwSignal::new(String::new());
    let timezone_input = RwSignal::new("America/New_York (UTC-4)".to_string());
    let language_input = RwSignal::new("English (US)".to_string());

    // Modal control signals
    let (show_mfa_modal, set_show_mfa_modal) = signal(false);
    let (show_new_key_modal, set_show_new_key_modal) = signal(false);
    let (show_add_webhook_modal, set_show_add_webhook_modal) = signal(false);

    // Notification switch toggles
    let lead_converted_notif = RwSignal::new(true);
    let invoice_overdue_notif = RwSignal::new(true);
    let anomaly_detected_notif = RwSignal::new(true);
    let tenant_sub_notif = RwSignal::new(false);
    let queue_item_notif = RwSignal::new(true);
    let new_ip_notif = RwSignal::new(true);

    Effect::new(move |_| {
        if let Some(ref u) = user.get() {
            first_name_input.set(u.first_name.clone());
            last_name_input.set(u.last_name.clone());
            email_input.set(u.email.clone());
        }
    });

    let handle_save_profile = move |_| {
        let email = email_input.get();
        let first = first_name_input.get();
        let last = last_name_input.get();
        let t = toast.clone();
        let u = user.get();
        let su = set_user.clone();

        leptos::task::spawn_local(async move {
            if email.is_empty() {
                t.show_toast("Error", "Email is required", "error");
                return;
            }
            if let Some(mut user_info) = u {
                user_info.first_name = first;
                user_info.last_name = last;
                
                if email != user_info.email {
                    match update_email(email.clone()).await {
                        Ok(_) => {
                            user_info.email = email;
                            su.set(Some(user_info));
                            t.show_toast("Success", "Profile and email updated successfully.", "success");
                        }
                        Err(e) => {
                            t.show_toast("Error", &format!("Failed to update email: {}", e), "error");
                        }
                    }
                } else {
                    su.set(Some(user_info));
                    t.show_toast("Success", "Profile updated successfully.", "success");
                }
            }
        });
    };



    let record_name = move || {
        user.get().map(|u| format!("{} {}", u.first_name, u.last_name)).unwrap_or_else(|| "Jamie Delaney".to_string())
    };

    let initials = move || {
        let name = record_name();
        name.split_whitespace()
            .map(|w| w.chars().next().unwrap_or('?'))
            .collect::<String>()
            .chars()
            .take(2)
            .collect::<String>()
            .to_uppercase()
    };

    view! {
        <div class="main-area" style="padding: 0; gap: 0; display: flex; flex-direction: column; height: 100%;">
            // ── Page Header ──
            <div class="page-hdr">
                <div>
                    <div class="page-title">"Profile & Preferences"</div>
                    <div class="page-sub">"Your account · Security · Notifications · API Keys"</div>
                </div>
                <button class="btn btn-ghost btn-sm" on:click=move |_| {
                    toast.show_toast("Auth", "Logging out...", "info");
                    leptos::task::spawn_local(async move {
                        let _ = crate::api::auth::logout().await;
                        set_user.set(None);
                        let _ = web_sys::window().unwrap().location().assign("/login");
                    });
                }>"Sign Out"</button>
            </div>

            // ── Tab Bar ──
            <div class="tab-bar">
                <button class=move || format!("tab {}", if active_tab.get() == "profile" { "active" } else { "" }) on:click=move |_| active_tab.set("profile".to_string())>"Profile"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "security" { "active" } else { "" }) on:click=move |_| active_tab.set("security".to_string())>"Security & MFA"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "notifications" { "active" } else { "" }) on:click=move |_| active_tab.set("notifications".to_string())>"Notifications"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "apikeys" { "active" } else { "" }) on:click=move |_| active_tab.set("apikeys".to_string())>"API Keys"</button>
                <button class=move || format!("tab {}", if active_tab.get() == "sessions" { "active" } else { "" }) on:click=move |_| active_tab.set("sessions".to_string())>"Active Sessions"</button>
            </div>

            <div class="body" style="flex: 1; overflow-y: auto; padding: 24px; max-width: 740px;">
                <Show when=move || user.get().is_some() fallback=move || view! {
                    <div class="card p-6 text-error">
                        "You must be logged in to view settings."
                    </div>
                }>
                    {move || match active_tab.get().as_str() {
                        "profile" => view! {
                            <div>
                                <div class="card">
                                    <div class="profile-hero">
                                        <div class="avatar-lg">{initials}</div>
                                        <div>
                                            <div class="profile-name">{record_name}</div>
                                            <div class="profile-role">
                                                {move || user.get().map(|u| format!("{} · Atlas Platform · Member since Jan 2025", if u.is_admin { "Super-Admin" } else { "Admin" })).unwrap_or_default()}
                                            </div>
                                            <button class="btn btn-ghost btn-sm" style="margin-top:8px" on:click=move |_| {
                                                toast.show_toast("Photo", "Upload photo trigger.", "info");
                                            }>"Change Photo"</button>
                                        </div>
                                    </div>
                                </div>
                                <div class="card">
                                    <div class="card-hdr">
                                        <span class="card-title">"Personal Information"</span>
                                        <button class="btn btn-ghost btn-sm" on:click=handle_save_profile>"Save Changes"</button>
                                    </div>
                                    <div class="card-body">
                                        <div class="two-col">
                                            <div class="form-row">
                                                <label class="form-label">"First Name"</label>
                                                <input class="form-input" prop:value=first_name_input on:input=move |e| first_name_input.set(event_target_value(&e))/>
                                            </div>
                                            <div class="form-row">
                                                <label class="form-label">"Last Name"</label>
                                                <input class="form-input" prop:value=last_name_input on:input=move |e| last_name_input.set(event_target_value(&e))/>
                                            </div>
                                        </div>
                                        <div class="form-row">
                                            <label class="form-label">"Email"</label>
                                            <input class="form-input" type="email" prop:value=email_input on:input=move |e| email_input.set(event_target_value(&e))/>
                                        </div>
                                        <div class="form-row">
                                            <label class="form-label">"Phone (optional)"</label>
                                            <input class="form-input" type="tel" placeholder="+1 305 555 0100" prop:value=phone_input on:input=move |e| phone_input.set(event_target_value(&e))/>
                                        </div>
                                        <div class="form-row">
                                            <label class="form-label">"Timezone"</label>
                                            <select class="form-select" prop:value=timezone_input on:change=move |e| timezone_input.set(event_target_value(&e))>
                                                <option value="America/New_York (UTC-4)">"America/New_York (UTC-4)"</option>
                                                <option value="America/Los_Angeles (UTC-7)">"America/Los_Angeles (UTC-7)"</option>
                                                <option value="America/Sao_Paulo (UTC-3)">"America/Sao_Paulo (UTC-3)"</option>
                                                <option value="UTC">"UTC"</option>
                                            </select>
                                        </div>
                                        <div class="form-row">
                                            <label class="form-label">"Language"</label>
                                            <select class="form-select" prop:value=language_input on:change=move |e| language_input.set(event_target_value(&e))>
                                                <option value="English (US)">"English (US)"</option>
                                                <option value="Português (BR)">"Português (BR)"</option>
                                                <option value="Español">"Español"</option>
                                            </select>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),

                        "security" => view! {
                            <div>
                                <div class="card">
                                    <div class="card-hdr">
                                        <span class="card-title">"Passkeys & Biometrics"</span>
                                    </div>
                                    <div class="card-body">
                                        <ManagePasskeys 
                                            api_base_url=Signal::derive(move || crate::api::client::api_url("/api/passkeys")) 
                                            auth_token="CSR_COOKIE_FLOW".to_string()
                                            auto_register=false
                                        />
                                    </div>
                                </div>
                                <div class="card">
                                    <div class="card-hdr">
                                        <span class="card-title">"Two-Factor Authentication · TOTP"</span>
                                        <button class="btn btn-ghost btn-sm" on:click=move |_| set_show_mfa_modal.set(true)>"Manage"</button>
                                    </div>
                                    <div class="stat-row">
                                        <span class="s-label">"Status"</span>
                                        <span class="s-value green">"✓ Active · TOTP enrolled"</span>
                                    </div>
                                    <div class="stat-row">
                                        <span class="s-label">"Enrolled"</span>
                                        <span class="s-value">"Jan 12, 2025"</span>
                                    </div>
                                    <div class="stat-row">
                                        <span class="s-label">"Backup codes"</span>
                                        <span class="s-value amber">"3 of 10 remaining"</span>
                                    </div>
                                </div>
                                <div class="card">
                                    <div class="card-hdr">
                                        <span class="card-title">"Danger Zone"</span>
                                    </div>
                                    <div class="stat-row">
                                        <div>
                                            <div class="s-label" style="color:var(--red)">"Revoke all sessions"</div>
                                            <div style="font-size:10.5px;color:var(--text-muted);margin-top:2px">"Signs you out from all devices immediately"</div>
                                        </div>
                                        <button class="btn btn-danger btn-sm" on:click=move |_| {
                                            toast.show_toast("Sessions", "All sessions revoked.", "info");
                                            let _ = web_sys::window().unwrap().location().assign("/login");
                                        }>"Revoke All Sessions"</button>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),

                        "notifications" => view! {
                            <div>
                                <div class="card">
                                    <div class="card-hdr">
                                        <span class="card-title">"Email Notifications"</span>
                                    </div>
                                    <div class="notif-row">
                                        <div>
                                            <div class="notif-label">"Lead converted"</div>
                                            <div class="notif-sub">"When a lead is atomically converted to Account/Contact/Opportunity"</div>
                                        </div>
                                        <div 
                                            class=move || format!("toggle {}", if lead_converted_notif.get() { "" } else { "off" }) 
                                            on:click=move |_| lead_converted_notif.update(|v| *v = !*v)
                                        ></div>
                                    </div>
                                    <div class="notif-row">
                                        <div>
                                            <div class="notif-label">"Invoice overdue"</div>
                                            <div class="notif-sub">"When a tenant invoice passes its due date"</div>
                                        </div>
                                        <div 
                                            class=move || format!("toggle {}", if invoice_overdue_notif.get() { "" } else { "off" }) 
                                            on:click=move |_| invoice_overdue_notif.update(|v| *v = !*v)
                                        ></div>
                                    </div>
                                    <div class="notif-row">
                                        <div>
                                            <div class="notif-label">"G-27 anomaly detected"</div>
                                            <div class="notif-sub">"When a scorecard dimension has |z| > 2.0"</div>
                                        </div>
                                        <div 
                                            class=move || format!("toggle {}", if anomaly_detected_notif.get() { "" } else { "off" }) 
                                            on:click=move |_| anomaly_detected_notif.update(|v| *v = !*v)
                                        ></div>
                                    </div>
                                    <div class="notif-row">
                                        <div>
                                            <div class="notif-label">"New tenant subscription"</div>
                                            <div class="notif-sub">"When a tenant starts or upgrades a billing plan"</div>
                                        </div>
                                        <div 
                                            class=move || format!("toggle {}", if tenant_sub_notif.get() { "" } else { "off" }) 
                                            on:click=move |_| tenant_sub_notif.update(|v| *v = !*v)
                                        ></div>
                                    </div>
                                    <div class="notif-row">
                                        <div>
                                            <div class="notif-label">"Verification queue item"</div>
                                            <div class="notif-sub">"When a new G-06 verification request is submitted"</div>
                                        </div>
                                        <div 
                                            class=move || format!("toggle {}", if queue_item_notif.get() { "" } else { "off" }) 
                                            on:click=move |_| queue_item_notif.update(|v| *v = !*v)
                                        ></div>
                                    </div>
                                    <div class="notif-row">
                                        <div>
                                            <div class="notif-label">"User login from new IP"</div>
                                            <div class="notif-sub">"Security: when any admin logs in from an unrecognized IP"</div>
                                        </div>
                                        <div 
                                            class=move || format!("toggle {}", if new_ip_notif.get() { "" } else { "off" }) 
                                            on:click=move |_| new_ip_notif.update(|v| *v = !*v)
                                        ></div>
                                    </div>
                                </div>
                                <div class="card">
                                    <div class="card-hdr">
                                        <span class="card-title">"Notification Email"</span>
                                        <button class="btn btn-ghost btn-sm" on:click=move |_| toast.show_toast("Success", "Notification email saved.", "success")>"Save"</button>
                                    </div>
                                    <div class="card-body">
                                        <div class="form-row">
                                            <label class="form-label">"Send to"</label>
                                            <input class="form-input" type="email" value="jamie@atlasplatform.io"/>
                                        </div>
                                        <div class="form-row">
                                            <label class="form-label">"Digest frequency"</label>
                                            <select class="form-select">
                                                <option>"Immediately"</option>
                                                <option>"Hourly digest"</option>
                                                <option>"Daily digest"</option>
                                            </select>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),

                        "apikeys" => view! {
                            <div>
                                <div class="card">
                                    <div class="card-hdr">
                                        <span class="card-title">"API Keys · Super-Admin Scope"</span>
                                        <button class="btn btn-primary btn-sm" on:click=move |_| set_show_new_key_modal.set(true)>"+ New Key"</button>
                                    </div>
                                    <div class="api-key-row">
                                        <span class="api-key-label">"Production Key"</span>
                                        <span class="api-key-val">"atls_sk_live_4aJx9Q2…••••••••"</span>
                                        <span class="api-key-status" style="color:var(--green);border-color:var(--green);background:var(--green-dim)">"Active"</span>
                                        <button class="btn btn-ghost btn-sm" on:click=move |_| toast.show_toast("API Key", "Revoke Production Key clicked.", "info")>"Revoke"</button>
                                    </div>
                                    <div class="api-key-row">
                                        <span class="api-key-label">"Staging Key"</span>
                                        <span class="api-key-val">"atls_sk_test_9bKm3P8…••••••••"</span>
                                        <span class="api-key-status" style="color:var(--green);border-color:var(--green);background:var(--green-dim)">"Active"</span>
                                        <button class="btn btn-ghost btn-sm" on:click=move |_| toast.show_toast("API Key", "Revoke Staging Key clicked.", "info")>"Revoke"</button>
                                    </div>
                                    <div class="api-key-row">
                                        <span class="api-key-label">"Legacy Reporting"</span>
                                        <span class="api-key-val">"atls_sk_live_2cFg1L4…••••••••"</span>
                                        <span class="api-key-status" style="color:var(--text-muted);border-color:var(--border-default)">"Expired"</span>
                                        <button class="btn btn-ghost btn-sm" style="color:var(--text-muted)" disabled=true>"Revoke"</button>
                                    </div>
                                </div>
                                <div class="card">
                                    <div class="card-hdr">
                                        <span class="card-title">"Webhook Endpoints"</span>
                                        <button class="btn btn-ghost btn-sm" on:click=move |_| set_show_add_webhook_modal.set(true)>"+ Add"</button>
                                    </div>
                                    <div class="api-key-row">
                                        <span class="api-key-label">"Zapier → Slack"</span>
                                        <span class="api-key-val">"https://hooks.zapier.com/hooks/catch/14…"</span>
                                        <span class="api-key-status" style="color:var(--green);border-color:var(--green);background:var(--green-dim)">"Active"</span>
                                        <button class="btn btn-ghost btn-sm" on:click=move |_| toast.show_toast("Webhook", "Remove Webhook clicked.", "info")>"Remove"</button>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),

                        "sessions" => view! {
                            <div>
                                <div class="card">
                                    <div class="card-hdr">
                                        <span class="card-title">"Active Sessions"</span>
                                        <button class="btn btn-danger btn-sm" on:click=move |_| toast.show_toast("Sessions", "Revoked other sessions.", "success")>"Revoke All Others"</button>
                                    </div>
                                    <div class="api-key-row">
                                        <div style="flex:1">
                                            <div style="font-size:12px;font-weight:500">
                                                "Chrome · macOS "
                                                <span style="font-size:9px;color:var(--cobalt);border:1px solid var(--cobalt);border-radius:3px;padding:1px 4px;margin-left:4px">"CURRENT"</span>
                                            </div>
                                            <div style="font-size:10.5px;color:var(--text-muted);margin-top:2px">"IP: 10.0.1.2 · Miami, FL · Started 2h ago"</div>
                                        </div>
                                        <span class="api-key-status" style="color:var(--green);border-color:var(--green);background:var(--green-dim)">"Active"</span>
                                    </div>
                                    <div class="api-key-row">
                                        <div style="flex:1">
                                            <div style="font-size:12px;font-weight:500">"Safari · iPhone"</div>
                                            <div style="font-size:10.5px;color:var(--text-muted);margin-top:2px">"IP: 10.0.2.8 · Miami, FL · Started 18h ago"</div>
                                        </div>
                                        <span class="api-key-status" style="color:var(--green);border-color:var(--green);background:var(--green-dim)">"Active"</span>
                                        <button class="btn btn-ghost btn-sm" on:click=move |_| toast.show_toast("Sessions", "Revoked iPhone session.", "success")>"Revoke"</button>
                                    </div>
                                </div>
                            </div>
                        }.into_any(),

                        _ => view! {}.into_any()
                    }}
                </Show>
            </div>

            // ── Modal Dialogs ──



            // MFA Status Modal
            <Show when=move || show_mfa_modal.get()>
                <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[1000]">
                    <div class="bg-[#111520] border border-outline-variant/40 rounded-xl p-6 w-full max-w-md shadow-2xl">
                        <div class="flex justify-between items-center mb-4">
                            <h3 class="text-base font-bold text-on-surface">"MFA Status"</h3>
                            <button class="text-on-surface-variant hover:text-on-surface font-bold" on:click=move |_| set_show_mfa_modal.set(false)>"✕"</button>
                        </div>
                        <p style="font-size:12.5px;color:var(--green)">"✓ TOTP authenticator is active. Scan the QR code in your authenticator app to re-enroll a new device."</p>
                        <div class="flex justify-end gap-3 mt-6 border-t border-outline-variant/10 pt-4">
                            <button class="btn btn-primary btn-sm" on:click=move |_| set_show_mfa_modal.set(false)>"Close"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // Create API Key Modal
            <Show when=move || show_new_key_modal.get()>
                <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[1000]">
                    <div class="bg-[#111520] border border-outline-variant/40 rounded-xl p-6 w-full max-w-md shadow-2xl">
                        <div class="flex justify-between items-center mb-6">
                            <h3 class="text-base font-bold text-on-surface">"Create API Key"</h3>
                            <button class="text-on-surface-variant hover:text-on-surface font-bold" on:click=move |_| set_show_new_key_modal.set(false)>"✕"</button>
                        </div>
                        <div class="space-y-4">
                            <div class="form-row">
                                <label class="form-label">"Key Name"</label>
                                <input class="form-input" placeholder="e.g. CI/CD Pipeline Key"/>
                            </div>
                            <div class="form-row">
                                <label class="form-label">"Scope"</label>
                                <select class="form-select">
                                    <option>"read-only"</option>
                                    <option>"read-write"</option>
                                    <option>"admin"</option>
                                </select>
                            </div>
                            <div class="form-row">
                                <label class="form-label">"Expiry"</label>
                                <select class="form-select">
                                    <option>"30 days"</option>
                                    <option>"90 days"</option>
                                    <option>"1 year"</option>
                                    <option>"Never"</option>
                                </select>
                            </div>
                        </div>
                        <div class="flex justify-end gap-3 mt-6 border-t border-outline-variant/10 pt-4">
                            <button class="btn btn-ghost btn-sm" on:click=move |_| set_show_new_key_modal.set(false)>"Cancel"</button>
                            <button class="btn btn-primary btn-sm" on:click=move |_| {
                                toast.show_toast("API Key", "API key created. Copy it now — it won't be shown again.", "success");
                                set_show_new_key_modal.set(false);
                            }>"Generate Key"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // Add Webhook Modal
            <Show when=move || show_add_webhook_modal.get()>
                <div class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-[1000]">
                    <div class="bg-[#111520] border border-outline-variant/40 rounded-xl p-6 w-full max-w-md shadow-2xl">
                        <div class="flex justify-between items-center mb-6">
                            <h3 class="text-base font-bold text-on-surface">"Add Webhook"</h3>
                            <button class="text-on-surface-variant hover:text-on-surface font-bold" on:click=move |_| set_show_add_webhook_modal.set(false)>"✕"</button>
                        </div>
                        <div class="space-y-4">
                            <div class="form-row">
                                <label class="form-label">"Endpoint URL"</label>
                                <input class="form-input" type="url" placeholder="https://your-server.com/webhook"/>
                            </div>
                            <div class="form-row">
                                <label class="form-label">"Events (select multiple)"</label>
                                <select class="form-select" multiple=true style="height:100px">
                                    <option>"lead.created"</option>
                                    <option>"lead.converted"</option>
                                    <option>"ledger.entry.created"</option>
                                    <option>"tenant.subscription.updated"</option>
                                    <option>"g27.anomaly.detected"</option>
                                </select>
                            </div>
                        </div>
                        <div class="flex justify-end gap-3 mt-6 border-t border-outline-variant/10 pt-4">
                            <button class="btn btn-ghost btn-sm" on:click=move |_| set_show_add_webhook_modal.set(false)>"Cancel"</button>
                            <button class="btn btn-primary btn-sm" on:click=move |_| {
                                toast.show_toast("Webhook", "Webhook added.", "success");
                                set_show_add_webhook_modal.set(false);
                            }>"Add Webhook"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
