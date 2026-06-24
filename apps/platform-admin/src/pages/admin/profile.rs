use leptos::prelude::*;
use uuid::Uuid;
use crate::api::models::UserInfo;
use crate::api::profile::update_email;
use crate::api::developer::{
    list_api_tokens, create_api_token, revoke_api_token,
    list_webhook_endpoints, create_webhook_endpoint, delete_webhook_endpoint,
    CreateApiTokenRequest, CreateWebhookRequest,
};
use crate::api::admin::{list_my_sessions, revoke_session_by_id, revoke_all_other_sessions};
use crate::api::files::{get_admin_presign, put_to_presigned_url, create_file_record, set_user_avatar};
use crate::api::models::CreateFileInput;
use crate::app::GlobalToast;
use shared_ui::components::auth::passkey_manager::ManagePasskeys;

#[component]
pub fn Settings() -> impl IntoView {
    let user = use_context::<ReadSignal<Option<UserInfo>>>().expect("user context");
    let set_user = use_context::<WriteSignal<Option<UserInfo>>>().expect("set user context");
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");

    let active_tab = RwSignal::new("profile".to_string());

    // ── Photo upload state ────────────────────────────────────────────────────
    let avatar_url = RwSignal::new(Option::<String>::None);
    let is_uploading_photo = RwSignal::new(false);
    let photo_input_ref: NodeRef<leptos::html::Input> = NodeRef::new();

    let handle_photo_change = move |ev: leptos::ev::Event| {
        use wasm_bindgen::JsCast;
        let input: web_sys::HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
        let files = input.files().unwrap();
        if files.length() == 0 { return; }
        let file = files.item(0).unwrap();
        let name = file.name();
        let mime = file.type_();
        let user_id = user.get().map(|u| u.id);
        let toast2 = toast.clone();
        is_uploading_photo.set(true);
        leptos::task::spawn_local(async move {
            // 1. Get presigned URL
            let presign = match get_admin_presign(&name, &mime, "avatars").await {
                Ok(p) => p,
                Err(e) => {
                    toast2.show_toast("Error", &format!("Presign failed: {}", e), "error");
                    is_uploading_photo.set(false);
                    return;
                }
            };
            // 2. Read bytes
            use wasm_bindgen_futures::JsFuture;
            use wasm_bindgen::JsCast;
            let ab = JsFuture::from(file.array_buffer()).await.unwrap();
            let bytes = js_sys::Uint8Array::new(&ab).to_vec();
            // 3. PUT to R2
            if let Err(e) = put_to_presigned_url(&presign.upload_url, bytes, &mime).await {
                toast2.show_toast("Error", &format!("Upload failed: {}", e), "error");
                is_uploading_photo.set(false);
                return;
            }
            // 4. Create file record
            let size = file.size() as i64;
            let file_record = match create_file_record(CreateFileInput {
                name: name.clone(),
                size,
                mime_type: mime.clone(),
                hash_sha256: String::new(),
                storage_type: "S3".to_string(),
                storage_path: presign.file_key.clone(),
                is_anonymous: false,
                user_id: user_id.clone().map(|id| id.to_string()),
            }).await {
                Ok(r) => r,
                Err(e) => {
                    toast2.show_toast("Error", &format!("File record failed: {}", e), "error");
                    is_uploading_photo.set(false);
                    return;
                }
            };
            // 5. Associate with User entity
            if let Some(uid) = user_id {
                let _ = set_user_avatar(&uid, &file_record.id).await;
            }
            // 6. Show avatar immediately
            avatar_url.set(Some(presign.public_url));
            toast2.show_toast("Profile", "Photo updated!", "success");
            is_uploading_photo.set(false);
        });
    };

    // Profile inputs
    let first_name_input = RwSignal::new(String::new());
    let last_name_input = RwSignal::new(String::new());
    let email_input = RwSignal::new(String::new());
    let phone_input = RwSignal::new(String::new());
    let timezone_input = RwSignal::new("America/New_York (UTC-4)".to_string());
    let language_input = RwSignal::new("English (US)".to_string());

    // Developer API state — keyed on active_network tenant UUID
    let active_network = use_context::<ReadSignal<Option<Uuid>>>().expect("active network context");

    let api_keys_res = LocalResource::new(move || async move {
        match active_network.get() {
            Some(tid) => list_api_tokens(tid).await.unwrap_or_default(),
            None => vec![],
        }
    });
    let webhooks_res = LocalResource::new(move || async move {
        match active_network.get() {
            Some(tid) => list_webhook_endpoints(tid).await.unwrap_or_default(),
            None => vec![],
        }
    });

    // Session list — keyed on sessions_version so revoke actions trigger a refetch
    let (sessions_version, set_sessions_version) = signal(0u32);
    let sessions_res = LocalResource::new(move || async move {
        sessions_version.get();
        list_my_sessions().await.unwrap_or_default()
    });

    // New API key form state
    let new_key_name   = RwSignal::new(String::new());
    let new_key_scope  = RwSignal::new("read-write".to_string());
    let is_gen_key     = RwSignal::new(false);
    let created_secret = RwSignal::new(Option::<String>::None);

    // New webhook form state
    let new_hook_url    = RwSignal::new(String::new());
    let is_add_hook     = RwSignal::new(false);

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
                <button class="tab" class:active=move || active_tab.get() == "profile" on:click=move |_| active_tab.set("profile".to_string())>"Profile"</button>
                <button class="tab" class:active=move || active_tab.get() == "security" on:click=move |_| active_tab.set("security".to_string())>"Security & MFA"</button>
                <button class="tab" class:active=move || active_tab.get() == "notifications" on:click=move |_| active_tab.set("notifications".to_string())>"Notifications"</button>
                <button class="tab" class:active=move || active_tab.get() == "apikeys" on:click=move |_| active_tab.set("apikeys".to_string())>"API Keys"</button>
                <button class="tab" class:active=move || active_tab.get() == "sessions" on:click=move |_| active_tab.set("sessions".to_string())>"Active Sessions"</button>
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
                                            {move || {
                                                if let Some(url) = avatar_url.get() {
                                                    view! {
                                                        <img src=url class="avatar-photo" style="width:72px;height:72px;border-radius:50%;object-fit:cover;margin-bottom:8px" alt="Profile photo" />
                                                    }.into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }
                                            }}
                                            <button
                                                class=move || if is_uploading_photo.get() { "btn btn-ghost btn-sm opacity-60" } else { "btn btn-ghost btn-sm" }
                                                style="margin-top:8px"
                                                disabled=move || is_uploading_photo.get()
                                                on:click=move |_| {
                                                    if let Some(input) = photo_input_ref.get() {
                                                        let _ = input.click();
                                                    }
                                                }
                                            >{move || if is_uploading_photo.get() { "Uploading…" } else { "Change Photo" }}</button>
                                            <input
                                                node_ref=photo_input_ref
                                                type="file"
                                                accept="image/png,image/jpeg,image/webp"
                                                style="display:none"
                                                on:change=handle_photo_change
                                            />
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
                                        <button class="btn btn-ghost btn-sm opacity-40 cursor-not-allowed" title="Notification preferences endpoint pending" disabled>"Save"</button>
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
                                        <span class="card-title">"API Keys · Tenant Scope"</span>
                                        {move || match active_network.get() {
                                            Some(_) => view! {
                                                <button class="btn btn-primary btn-sm" on:click=move |_| {
                                                    new_key_name.set(String::new());
                                                    created_secret.set(None);
                                                    set_show_new_key_modal.set(true);
                                                }>"+ New Key"</button>
                                            }.into_any(),
                                            None => view! {
                                                <span class="text-xs text-on-surface-variant">"Select a tenant to manage keys"</span>
                                            }.into_any()
                                        }}
                                    </div>
                                    <Suspense fallback=move || view! { <div class="p-4 text-sm text-on-surface-variant">"Loading keys…"</div> }>
                                    {move || {
                                        let keys = api_keys_res.get().unwrap_or_default();
                                        if keys.is_empty() {
                                            view! { <div class="p-4 text-sm text-on-surface-variant">"No API keys yet. Generate one above."</div> }.into_any()
                                        } else {
                                            view! {
                                                <div>
                                                <For
                                                    each=move || keys.clone()
                                                    key=|k| k.id
                                                    children=move |key| {
                                                        let kid = key.id;
                                                        let tid_for_revoke = active_network.get();
                                                        let t = toast.clone();
                                                        view! {
                                                            <div class="api-key-row">
                                                                <span class="api-key-label">{key.name.clone()}</span>
                                                                <span class="api-key-val font-mono text-xs">{key.prefix.clone()}"••••••"</span>
                                                                <span class="api-key-status" style=if key.is_active { "color:var(--green);border-color:var(--green);background:var(--green-dim)" } else { "color:var(--text-muted)" }>
                                                                    {if key.is_active { "Active" } else { "Revoked" }}
                                                                </span>
                                                                {if key.is_active {
                                                                    view! {
                                                                        <button class="btn btn-ghost btn-sm"
                                                                            on:click=move |_| {
                                                                                if let Some(tid) = tid_for_revoke {
                                                                                    let t2 = t.clone();
                                                                                    leptos::task::spawn_local(async move {
                                                                                        match revoke_api_token(tid, kid).await {
                                                                                            Ok(_) => {
                                                                                                t2.show_toast("Revoked", "API key revoked.", "success");
                                                                                                api_keys_res.refetch();
                                                                                            }
                                                                                            Err(e) => t2.show_toast("Error", &format!("{e}"), "error"),
                                                                                        }
                                                                                    });
                                                                                }
                                                                            }
                                                                        >"Revoke"</button>
                                                                    }.into_any()
                                                                } else {
                                                                    view! { <span class="text-xs text-on-surface-variant/40">"—"</span> }.into_any()
                                                                }}
                                                            </div>
                                                        }
                                                    }
                                                />
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                    </Suspense>
                                </div>
                                <div class="card">
                                    <div class="card-hdr">
                                        <span class="card-title">"Webhook Endpoints"</span>
                                        {move || if active_network.get().is_some() {
                                            view! {
                                                <button class="btn btn-ghost btn-sm" on:click=move |_| {
                                                    new_hook_url.set(String::new());
                                                    set_show_add_webhook_modal.set(true);
                                                }>"+ Add"</button>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                    <Suspense fallback=move || view! { <div class="p-4 text-sm text-on-surface-variant">"Loading webhooks…"</div> }>
                                    {move || {
                                        let hooks = webhooks_res.get().unwrap_or_default();
                                        if hooks.is_empty() {
                                            view! { <div class="p-4 text-sm text-on-surface-variant">"No webhooks configured."</div> }.into_any()
                                        } else {
                                            view! {
                                                <div>
                                                <For
                                                    each=move || hooks.clone()
                                                    key=|h| h.id
                                                    children=move |hook| {
                                                        let hid = hook.id;
                                                        let tid_for_del = active_network.get();
                                                        let t = toast.clone();
                                                        view! {
                                                            <div class="api-key-row">
                                                                <span class="api-key-label">{hook.target_url.clone()}</span>
                                                                <span class="api-key-status" style=if hook.is_active { "color:var(--green);border-color:var(--green);background:var(--green-dim)" } else { "color:var(--text-muted)" }>
                                                                    {if hook.is_active { "Active" } else { "Disabled" }}
                                                                </span>
                                                                <button class="btn btn-ghost btn-sm"
                                                                    on:click=move |_| {
                                                                        if let Some(tid) = tid_for_del {
                                                                            let t2 = t.clone();
                                                                            leptos::task::spawn_local(async move {
                                                                                match delete_webhook_endpoint(tid, hid).await {
                                                                                    Ok(_) => {
                                                                                        t2.show_toast("Deleted", "Webhook removed.", "success");
                                                                                        webhooks_res.refetch();
                                                                                    }
                                                                                    Err(e) => t2.show_toast("Error", &format!("{e}"), "error"),
                                                                                }
                                                                            });
                                                                        }
                                                                    }
                                                                >"Remove"</button>
                                                            </div>
                                                        }
                                                    }
                                                />
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                    </Suspense>
                                </div>
                            </div>
                        }.into_any(),

                        "sessions" => view! {
                            <div>
                                <div class="card">
                                    <div class="card-hdr">
                                        <span class="card-title">"Active Sessions"</span>
                                        <button
                                            class="btn btn-danger btn-sm"
                                            on:click=move |_| {
                                                let toast = toast.clone();
                                                leptos::task::spawn_local(async move {
                                                    match revoke_all_other_sessions().await {
                                                        Ok(v) => {
                                                            let count = v.get("revoked").and_then(|n| n.as_u64()).unwrap_or(0);
                                                            toast.show_toast("Sessions", &format!("Revoked {} other session(s).", count), "success");
                                                            set_sessions_version.update(|v| *v += 1);
                                                        }
                                                        Err(e) => toast.show_toast("Error", &e, "error"),
                                                    }
                                                });
                                            }
                                        >"Revoke All Others"</button>
                                    </div>
                                    {move || {
                                        let sessions = sessions_res.get().unwrap_or_default();
                                        if sessions.is_empty() {
                                            return view! { <div class="api-key-row" style="color:var(--text-muted);font-size:11px">"No active sessions found."</div> }.into_any();
                                        }
                                        view! {
                                            <For
                                                each=move || sessions.clone()
                                                key=|s| s.id
                                                children=move |sess| {
                                                    let sid = sess.id;
                                                    let toast2 = toast.clone();
                                                    let last = sess.last_accessed_at.get(..16).unwrap_or(&sess.last_accessed_at).to_string();
                                                    let started = sess.created_at.get(..16).unwrap_or(&sess.created_at).to_string();
                                                    view! {
                                                        <div class="api-key-row">
                                                            <div style="flex:1">
                                                                <div style="font-size:12px;font-weight:500">
                                                                    {format!("Session {}", &sid.to_string()[..8])}
                                                                    {if sess.is_current {
                                                                        view! { <span style="font-size:9px;color:var(--cobalt);border:1px solid var(--cobalt);border-radius:3px;padding:1px 4px;margin-left:4px">"CURRENT"</span> }.into_any()
                                                                    } else {
                                                                        view! {}.into_any()
                                                                    }}
                                                                </div>
                                                                <div style="font-size:10.5px;color:var(--text-muted);margin-top:2px">
                                                                    {format!("Started: {} · Last active: {}", started, last)}
                                                                </div>
                                                            </div>
                                                            <span class="api-key-status" style="color:var(--green);border-color:var(--green);background:var(--green-dim)">"Active"</span>
                                                            {if !sess.is_current {
                                                                view! {
                                                                    <button
                                                                        class="btn btn-ghost btn-sm"
                                                                        on:click=move |_| {
                                                                            let toast3 = toast2.clone();
                                                                            leptos::task::spawn_local(async move {
                                                                                match revoke_session_by_id(sid).await {
                                                                                    Ok(_) => {
                                                                                        toast3.show_toast("Sessions", "Session revoked.", "success");
                                                                                        set_sessions_version.update(|v| *v += 1);
                                                                                    }
                                                                                    Err(e) => toast3.show_toast("Error", &e, "error"),
                                                                                }
                                                                            });
                                                                        }
                                                                    >"Revoke"</button>
                                                                }.into_any()
                                                            } else {
                                                                view! {}.into_any()
                                                            }}
                                                        </div>
                                                    }
                                                }
                                            />
                                        }.into_any()
                                    }}
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
                        {move || {
                            if let Some(secret) = created_secret.get() {
                                view! {
                                    <div class="space-y-4">
                                        <p class="text-sm text-on-surface-variant">"Copy this key now — it won't be shown again."</p>
                                        <div class="bg-surface-container-highest rounded-lg p-3 font-mono text-xs break-all text-on-surface">{secret}</div>
                                    </div>
                                    <div class="flex justify-end mt-6">
                                        <button class="btn btn-primary btn-sm" on:click=move |_| set_show_new_key_modal.set(false)>"Done"</button>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="space-y-4">
                                        <div class="form-row">
                                            <label class="form-label">"Key Name"</label>
                                            <input class="form-input" placeholder="e.g. CI/CD Pipeline Key"
                                                prop:value=move || new_key_name.get()
                                                on:input=move |e| new_key_name.set(event_target_value(&e))
                                            />
                                        </div>
                                        <div class="form-row">
                                            <label class="form-label">"Scope"</label>
                                            <select class="form-select" on:change=move |e| new_key_scope.set(event_target_value(&e))>
                                                <option value="read-only">"read-only"</option>
                                                <option value="read-write" selected=true>"read-write"</option>
                                                <option value="admin">"admin"</option>
                                            </select>
                                        </div>
                                    </div>
                                    <div class="flex justify-end gap-3 mt-6 border-t border-outline-variant/10 pt-4">
                                        <button class="btn btn-ghost btn-sm" on:click=move |_| set_show_new_key_modal.set(false)>"Cancel"</button>
                                        <button
                                            class="btn btn-primary btn-sm"
                                            disabled=move || is_gen_key.get()
                                            on:click=move |_| {
                                                let name = new_key_name.get().trim().to_string();
                                                if name.is_empty() { return; }
                                                let Some(tid) = active_network.get() else { return; };
                                                if is_gen_key.get() { return; }
                                                is_gen_key.set(true);
                                                let t = toast.clone();
                                                leptos::task::spawn_local(async move {
                                                    let req = CreateApiTokenRequest {
                                                        name,
                                                        scopes: vec![new_key_scope.get()],
                                                    };
                                                    match create_api_token(tid, req).await {
                                                        Ok(resp) => {
                                                            created_secret.set(Some(resp.secret));
                                                            api_keys_res.refetch();
                                                        }
                                                        Err(e) => t.show_toast("Error", &format!("{e}"), "error"),
                                                    }
                                                    is_gen_key.set(false);
                                                });
                                            }
                                        >
                                            {move || if is_gen_key.get() { "Generating…" } else { "Generate Key" }}
                                        </button>
                                    </div>
                                }.into_any()
                            }
                        }}
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
                                <input class="form-input" type="url" placeholder="https://your-server.com/webhook"
                                    prop:value=move || new_hook_url.get()
                                    on:input=move |e| new_hook_url.set(event_target_value(&e))
                                />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3 mt-6 border-t border-outline-variant/10 pt-4">
                            <button class="btn btn-ghost btn-sm" on:click=move |_| set_show_add_webhook_modal.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary btn-sm"
                                disabled=move || is_add_hook.get()
                                on:click=move |_| {
                                    let url = new_hook_url.get().trim().to_string();
                                    if url.is_empty() { return; }
                                    let Some(tid) = active_network.get() else { return; };
                                    if is_add_hook.get() { return; }
                                    is_add_hook.set(true);
                                    let t = toast.clone();
                                    leptos::task::spawn_local(async move {
                                        let req = CreateWebhookRequest {
                                            target_url: url,
                                            events: vec!["*".to_string()],
                                            secret: None,
                                        };
                                        match create_webhook_endpoint(tid, req).await {
                                            Ok(_) => {
                                                t.show_toast("Added", "Webhook endpoint registered.", "success");
                                                set_show_add_webhook_modal.set(false);
                                                webhooks_res.refetch();
                                            }
                                            Err(e) => t.show_toast("Error", &format!("{e}"), "error"),
                                        }
                                        is_add_hook.set(false);
                                    });
                                }
                            >
                                {move || if is_add_hook.get() { "Adding…" } else { "Add Webhook" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
