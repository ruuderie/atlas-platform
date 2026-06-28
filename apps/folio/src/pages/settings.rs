use wasm_bindgen::JsCast;
// apps/folio/src/pages/settings.rs
//
// Settings — /settings
//
// Universal settings page — accessible from the sidebar footer of every role.
// Four tabs:
//   1. Profile    — display name, avatar initials, timezone, language
//   2. Security   — change password, active sessions, 2FA status
//   3. Notifications — per-channel toggles (email, sms, in-app) per event type
//   4. Appearance — theme (system / light / dark), sidebar density
//
// API:
//   GET  /api/folio/settings/profile          → UserProfileSettings
//   PATCH /api/folio/settings/profile
//   GET  /api/folio/settings/notifications    → NotificationPrefs
//   PATCH /api/folio/settings/notifications
//   POST /api/folio/auth/change-password
//   GET  /api/folio/auth/sessions             → Vec<SessionInfo>
//   DELETE /api/folio/auth/sessions/:id       (revoke)
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserProfileSettings {
    pub display_name: String,
    pub email:        String,
    pub phone:        Option<String>,
    pub timezone:     String,
    pub language:     String,
    pub initials:     Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationPrefs {
    // Maintenance events
    pub maint_email:   bool,
    pub maint_sms:     bool,
    pub maint_in_app:  bool,
    // Lease events
    pub lease_email:   bool,
    pub lease_sms:     bool,
    pub lease_in_app:  bool,
    // Payment events
    pub payment_email: bool,
    pub payment_sms:   bool,
    pub payment_in_app:bool,
    // Message events
    pub msg_email:     bool,
    pub msg_sms:       bool,
    pub msg_in_app:    bool,
    // System alerts
    pub system_email:  bool,
    pub system_in_app: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id:          String,
    pub device:      String,
    pub ip:          Option<String>,
    pub location:    Option<String>,
    pub created_at:  String,
    pub is_current:  bool,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchUserProfile, "/api")]
pub async fn fetch_user_profile() -> Result<UserProfileSettings, server_fn::error::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;
        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let token = headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            }))
            .ok_or_else(|| server_fn::error::ServerFnError::new("No session"))?;
        crate::atlas_client::authenticated_get::<UserProfileSettings>(
            "/api/folio/settings/profile", &token, None,
        ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(SaveUserProfile, "/api")]
pub async fn save_user_profile(profile: UserProfileSettings) -> Result<(), server_fn::error::ServerFnError> {
    let _ = profile;
    Ok(())
}

#[server(FetchNotifPrefs, "/api")]
pub async fn fetch_notif_prefs() -> Result<NotificationPrefs, server_fn::error::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;
        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let token = headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            }))
            .ok_or_else(|| server_fn::error::ServerFnError::new("No session"))?;
        crate::atlas_client::authenticated_get::<NotificationPrefs>(
            "/api/folio/settings/notifications", &token, None,
        ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(SaveNotifPrefs, "/api")]
pub async fn save_notif_prefs(prefs: NotificationPrefs) -> Result<(), server_fn::error::ServerFnError> {
    let _ = prefs;
    Ok(())
}

#[server(FetchSessions, "/api")]
pub async fn fetch_sessions() -> Result<Vec<SessionInfo>, server_fn::error::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;
        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let token = headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            }))
            .ok_or_else(|| server_fn::error::ServerFnError::new("No session"))?;
        crate::atlas_client::authenticated_get::<Vec<SessionInfo>>(
            "/api/folio/auth/sessions", &token, None,
        ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(ChangePassword, "/api")]
pub async fn change_password(current: String, new_pass: String) -> Result<(), server_fn::error::ServerFnError> {
    let _ = (current, new_pass);
    Ok(())
}

// ── Profile Tab ───────────────────────────────────────────────────────────────

#[component]
fn ProfileTab() -> impl IntoView {
    let profile_res = Resource::new(|| (), |_| fetch_user_profile());
    let profile     = RwSignal::new(UserProfileSettings::default());
    let saved       = RwSignal::new(false);
    let saving      = RwSignal::new(false);

    let TIMEZONES: &[&str] = &[
        "America/New_York","America/Chicago","America/Denver","America/Los_Angeles",
        "America/Anchorage","Pacific/Honolulu","America/Puerto_Rico",
        "Europe/London","Europe/Paris","Europe/Berlin","Europe/Amsterdam",
        "Asia/Tokyo","Asia/Singapore","Australia/Sydney","Pacific/Auckland",
    ];

    view! {
        <Suspense fallback=|| view! { <div class="doc-empty">"Loading profile…"</div> }>
            {move || profile_res.get().map(|res| {
                if let Ok(p) = res { profile.set(p); }
                view! {
                    <div class="settings-section">
                        // Avatar
                        <div class="settings-avatar-row">
                            <div class="settings-avatar">
                                {move || profile.get().initials.clone().unwrap_or_else(|| {
                                    profile.get().display_name.split_whitespace()
                                        .filter_map(|w| w.chars().next())
                                        .take(2).collect::<String>().to_uppercase()
                                })}
                            </div>
                            <div>
                                <div class="settings-avatar-name">{move || profile.get().display_name.clone()}</div>
                                <div class="settings-avatar-email text-xs text-on-surface-variant">{move || profile.get().email.clone()}</div>
                            </div>
                        </div>

                        <div class="apply-two-col">
                            <div class="form-field">
                                <label class="form-label">"Display Name"</label>
                                <input type="text" class="form-input"
                                    prop:value=move || profile.get().display_name.clone()
                                    on:input=move |ev| profile.update(|p| p.display_name = event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Phone"</label>
                                <input type="tel" class="form-input" placeholder="+1 (555) 000-0000"
                                    prop:value=move || profile.get().phone.clone().unwrap_or_default()
                                    on:input=move |ev| {
                                        let v = event_target_value(&ev);
                                        profile.update(|p| p.phone = if v.is_empty() { None } else { Some(v) });
                                    }
                                />
                            </div>
                        </div>
                        <div class="apply-two-col">
                            <div class="form-field">
                                <label class="form-label">"Timezone"</label>
                                <select class="form-select"
                                    on:change=move |ev| profile.update(|p| p.timezone = event_target_value(&ev))
                                >
                                    {TIMEZONES.iter().map(|tz| {
                                        let tz_val = tz.to_string();
                                        view! {
                                            <option value={tz_val.clone()}
                                                selected=move || profile.get().timezone == *tz
                                            >{ tz_val.clone() }</option>
                                        }
                                    }).collect::<Vec<_>>()}
                                </select>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Language"</label>
                                <select class="form-select"
                                    on:change=move |ev| profile.update(|p| p.language = event_target_value(&ev))
                                >
                                    <option value="en" selected=move || profile.get().language == "en">"English"</option>
                                    <option value="es" selected=move || profile.get().language == "es">"Español"</option>
                                    <option value="fr" selected=move || profile.get().language == "fr">"Français"</option>
                                    <option value="pt" selected=move || profile.get().language == "pt">"Português"</option>
                                    <option value="de" selected=move || profile.get().language == "de">"Deutsch"</option>
                                </select>
                            </div>
                        </div>

                        {move || if saved.get() {
                            view! { <div class="alert-saved-toast">"✓ Profile saved"</div> }.into_any()
                        } else { ().into_any() }}

                        <div style="margin-top:1rem;">
                            <button
                                class="btn btn-primary"
                                disabled=move || saving.get()
                                on:click=move |_| {
                                    saving.set(true);
                                    saved.set(false);
                                    let p = profile.get();
                                    leptos::task::spawn_local(async move {
                                        let _ = save_user_profile(p).await;
                                        saved.set(true);
                                        saving.set(false);
                                    });
                                }
                            >{move || if saving.get() { "Saving…" } else { "Save Profile" }}</button>
                        </div>
                    </div>
                }
            })}
        </Suspense>
    }
}

// ── Security Tab ──────────────────────────────────────────────────────────────

#[component]
fn SecurityTab() -> impl IntoView {
    let current_pass = RwSignal::new(String::new());
    let new_pass     = RwSignal::new(String::new());
    let confirm_pass = RwSignal::new(String::new());
    let pw_saving    = RwSignal::new(false);
    let pw_saved     = RwSignal::new(false);
    let pw_error     = RwSignal::new(None::<String>);

    let sessions_res = Resource::new(|| (), |_| fetch_sessions());

    view! {
        <div class="settings-section">
            // ── Change password ──
            <div class="owner-section">
                <div class="owner-section-title">"Change Password"</div>
                <div style="max-width:24rem;display:flex;flex-direction:column;gap:.75rem;">
                    <div class="form-field">
                        <label class="form-label">"Current Password"</label>
                        <input type="password" class="form-input"
                            on:input=move |ev| current_pass.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"New Password"</label>
                        <input type="password" class="form-input" placeholder="At least 12 characters"
                            on:input=move |ev| new_pass.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-field">
                        <label class="form-label">"Confirm New Password"</label>
                        <input type="password" class="form-input"
                            on:input=move |ev| confirm_pass.set(event_target_value(&ev))
                        />
                    </div>

                    {move || pw_error.get().map(|e| view! { <div class="wiz-error-banner">"⚠ " {e}</div> })}
                    {move || if pw_saved.get() { view! { <div class="alert-saved-toast">"✓ Password updated"</div> }.into_any() } else { ().into_any() }}

                    <button
                        class="btn btn-primary"
                        disabled=move || pw_saving.get() || new_pass.get().len() < 8
                        on:click=move |_| {
                            if new_pass.get() != confirm_pass.get() {
                                pw_error.set(Some("Passwords do not match.".to_string()));
                                return;
                            }
                            pw_error.set(None);
                            pw_saving.set(true);
                            let cur = current_pass.get();
                            let new = new_pass.get();
                            leptos::task::spawn_local(async move {
                                match change_password(cur, new).await {
                                    Ok(_)  => { pw_saved.set(true); pw_saving.set(false); }
                                    Err(e) => { pw_error.set(Some(e.to_string())); pw_saving.set(false); }
                                }
                            });
                        }
                    >{move || if pw_saving.get() { "Updating…" } else { "Update Password" }}</button>
                </div>
            </div>

            // ── Sessions ──
            <div class="owner-section" style="margin-top:1.5rem;">
                <div class="owner-section-title">"Active Sessions"</div>
                <Suspense fallback=|| view! { <div class="doc-empty">"Loading sessions…"</div> }>
                    {move || sessions_res.get().map(|res| {
                        match res {
                            Ok(sessions) => view! {
                                <div class="settings-session-list">
                                    {sessions.iter().map(|s| {
                                        let device   = s.device.clone();
                                        let location = s.location.clone().unwrap_or_else(|| "—".to_string());
                                        let ip       = s.ip.clone().unwrap_or_else(|| "—".to_string());
                                        let date     = s.created_at.chars().take(10).collect::<String>();
                                        let is_cur   = s.is_current;
                                        view! {
                                            <div class="settings-session-row">
                                                <div class="settings-session-icon">{if device.contains("Mobile") { "📱" } else { "💻" }}</div>
                                                <div class="settings-session-info">
                                                    <div class="settings-session-device">{device}
                                                        {if is_cur { view! { <span class="ph-badge ph-badge--paid" style="font-size:.65rem;margin-left:.4rem;">"Current"</span> }.into_any() } else { ().into_any() }}
                                                    </div>
                                                    <div class="text-xs text-on-surface-variant">{location} " · " {ip} " · " {date}</div>
                                                </div>
                                                {if !is_cur { view! {
                                                    <button class="btn btn-ghost btn-sm" style="color:#f87171;">"Revoke"</button>
                                                }.into_any() } else { ().into_any() }}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any(),
                            Err(_) => view! { <div class="doc-empty">"Could not load sessions."</div> }.into_any(),
                        }
                    })}
                </Suspense>
            </div>

            // ── 2FA stub ──
            <div class="owner-section" style="margin-top:1.5rem;">
                <div class="owner-section-title">"Two-Factor Authentication"</div>
                <div class="viol-info-banner">
                    <span class="viol-info-icon">"🔐"</span>
                    <p class="viol-info-text">"Two-factor authentication via TOTP authenticator app is available. Contact your administrator to enable."</p>
                </div>
            </div>
        </div>
    }
}

// ── Notifications Tab ─────────────────────────────────────────────────────────

#[component]
fn NotificationsTab() -> impl IntoView {
    let prefs_res = Resource::new(|| (), |_| fetch_notif_prefs());
    let prefs     = RwSignal::new(NotificationPrefs::default());
    let saved     = RwSignal::new(false);

    // (key, label, description)
    let event_groups: &[(&str, &str, &str)] = &[
        ("maint",   "Maintenance",        "Work order updates, completion notices, emergency alerts"),
        ("lease",   "Lease Events",       "Renewal reminders, move-in/out notifications, expirations"),
        ("payment", "Payments",           "Rent receipts, late payment notices, refunds"),
        ("msg",     "Messages",           "New inbox messages from tenants, vendors, or your team"),
    ];

    view! {
        <Suspense fallback=|| view! { <div class="doc-empty">"Loading preferences…"</div> }>
            {move || prefs_res.get().map(|res| {
                if let Ok(p) = res { prefs.set(p); }
                view! {
                    <div class="settings-section">
                        <div class="settings-notif-table">
                            // Header
                            <div class="settings-notif-header">
                                <div>"Event"</div>
                                <div>"Email"</div>
                                <div>"SMS"</div>
                                <div>"In-App"</div>
                            </div>

                            {event_groups.iter().map(|(key, label, desc)| {
                                let key = *key;
                                let label = *label;
                                let desc  = *desc;

                                let email_on = move || match key {
                                    "maint"   => prefs.get().maint_email,
                                    "lease"   => prefs.get().lease_email,
                                    "payment" => prefs.get().payment_email,
                                    "msg"     => prefs.get().msg_email,
                                    _         => false,
                                };
                                let sms_on = move || match key {
                                    "maint"   => prefs.get().maint_sms,
                                    "lease"   => prefs.get().lease_sms,
                                    "payment" => prefs.get().payment_sms,
                                    "msg"     => prefs.get().msg_sms,
                                    _         => false,
                                };
                                let inapp_on = move || match key {
                                    "maint"   => prefs.get().maint_in_app,
                                    "lease"   => prefs.get().lease_in_app,
                                    "payment" => prefs.get().payment_in_app,
                                    "msg"     => prefs.get().msg_in_app,
                                    _         => false,
                                };

                                view! {
                                    <div class="settings-notif-row">
                                        <div>
                                            <div class="settings-notif-label">{label}</div>
                                            <div class="settings-notif-desc text-xs text-on-surface-variant">{desc}</div>
                                        </div>
                                        // Email toggle
                                        <label class="syndic-toggle-wrap">
                                            <input type="checkbox" class="syndic-toggle-input"
                                                prop:checked=move || email_on()
                                                on:change=move |ev: web_sys::Event| {
                                                    let el = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                                                    if let Some(el) = el { let c = el.checked(); prefs.update(|p| match key {
                                                        "maint"   => p.maint_email   = c,
                                                        "lease"   => p.lease_email   = c,
                                                        "payment" => p.payment_email = c,
                                                        "msg"     => p.msg_email     = c,
                                                        _         => {},
                                                    }); }
                                                }
                                            />
                                            <span class="syndic-toggle-track"></span>
                                        </label>
                                        // SMS toggle
                                        <label class="syndic-toggle-wrap">
                                            <input type="checkbox" class="syndic-toggle-input"
                                                prop:checked=move || sms_on()
                                                on:change=move |ev: web_sys::Event| {
                                                    let el = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                                                    if let Some(el) = el { let c = el.checked(); prefs.update(|p| match key {
                                                        "maint"   => p.maint_sms   = c,
                                                        "lease"   => p.lease_sms   = c,
                                                        "payment" => p.payment_sms = c,
                                                        "msg"     => p.msg_sms     = c,
                                                        _         => {},
                                                    }); }
                                                }
                                            />
                                            <span class="syndic-toggle-track"></span>
                                        </label>
                                        // In-app toggle
                                        <label class="syndic-toggle-wrap">
                                            <input type="checkbox" class="syndic-toggle-input"
                                                prop:checked=move || inapp_on()
                                                on:change=move |ev: web_sys::Event| {
                                                    let el = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                                                    if let Some(el) = el { let c = el.checked(); prefs.update(|p| match key {
                                                        "maint"   => p.maint_in_app   = c,
                                                        "lease"   => p.lease_in_app   = c,
                                                        "payment" => p.payment_in_app = c,
                                                        "msg"     => p.msg_in_app     = c,
                                                        _         => {},
                                                    }); }
                                                }
                                            />
                                            <span class="syndic-toggle-track"></span>
                                        </label>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>

                        {move || if saved.get() {
                            view! { <div class="alert-saved-toast">"✓ Notification preferences saved"</div> }.into_any()
                        } else { ().into_any() }}

                        <div style="margin-top:1rem;">
                            <button class="btn btn-primary" on:click=move |_| {
                                let p = prefs.get();
                                leptos::task::spawn_local(async move {
                                    let _ = save_notif_prefs(p).await;
                                    saved.set(true);
                                });
                            }>"Save Preferences"</button>
                        </div>
                    </div>
                }
            })}
        </Suspense>
    }
}

// ── Appearance Tab ────────────────────────────────────────────────────────────

#[component]
fn AppearanceTab() -> impl IntoView {
    let theme   = RwSignal::new("system".to_string());
    let density = RwSignal::new("default".to_string());
    let saved   = RwSignal::new(false);

    view! {
        <div class="settings-section">
            <div class="form-field">
                <label class="form-label">"Theme"</label>
                <div class="settings-theme-grid">
                    {[("system","🖥 System"), ("dark","🌙 Dark"), ("light","☀ Light")].iter().map(|(id, label)| {
                        let id = *id;
                        let label = *label;
                        view! {
                            <div
                                class=move || format!("settings-theme-card {}", if theme.get() == id { "settings-theme-card--active" } else { "" })
                                on:click=move |_| theme.set(id.to_string())
                            >{label}</div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>

            <div class="form-field" style="margin-top:1rem;">
                <label class="form-label">"Sidebar Density"</label>
                <div style="display:flex;gap:.5rem;flex-wrap:wrap;">
                    {[("default","Default"), ("compact","Compact"), ("comfortable","Comfortable")].iter().map(|(id, label)| {
                        let id = *id;
                        let label = *label;
                        view! {
                            <button
                                class=move || format!("btn btn-sm {}", if density.get() == id { "btn-primary" } else { "btn-ghost" })
                                on:click=move |_| density.set(id.to_string())
                            >{label}</button>
                        }
                    }).collect::<Vec<_>>()}
                </div>
                <div class="form-hint" style="margin-top:.3rem;">"Compact reduces vertical padding in nav items. Comfortable increases spacing."</div>
            </div>

            <div class="viol-info-banner" style="margin-top:.75rem;">
                <span class="viol-info-icon">"💡"</span>
                <p class="viol-info-text">"Theme and density preferences are stored locally in your browser. They do not sync across devices."</p>
            </div>

            {move || if saved.get() {
                view! { <div class="alert-saved-toast">"✓ Appearance saved"</div> }.into_any()
            } else { ().into_any() }}

            <div style="margin-top:1rem;">
                <button class="btn btn-primary" on:click=move |_| saved.set(true)>"Save Appearance"</button>
            </div>
        </div>
    }
}

// ── Root Component ────────────────────────────────────────────────────────────

#[component]
pub fn Settings() -> impl IntoView {
    let tab = RwSignal::new(0u8);

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Settings"</h1>
                    <p class="page-subtitle">"Account, security and preferences"</p>
                </div>
            </div>

            // ── Tabs ──
            <div class="owner-tabs" style="margin-bottom:1.25rem;">
                <button class=move || format!("owner-tab {}", if tab.get()==0 {"owner-tab--active"} else {""}) on:click=move |_| tab.set(0)>"👤 Profile"</button>
                <button class=move || format!("owner-tab {}", if tab.get()==1 {"owner-tab--active"} else {""}) on:click=move |_| tab.set(1)>"🔒 Security"</button>
                <button class=move || format!("owner-tab {}", if tab.get()==2 {"owner-tab--active"} else {""}) on:click=move |_| tab.set(2)>"🔔 Notifications"</button>
                <button class=move || format!("owner-tab {}", if tab.get()==3 {"owner-tab--active"} else {""}) on:click=move |_| tab.set(3)>"🎨 Appearance"</button>
            </div>

            {move || match tab.get() {
                0 => view! { <ProfileTab /> }.into_any(),
                1 => view! { <SecurityTab /> }.into_any(),
                2 => view! { <NotificationsTab /> }.into_any(),
                _ => view! { <AppearanceTab /> }.into_any(),
            }}
        </div>
    }
}
