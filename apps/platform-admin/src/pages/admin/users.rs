use crate::api::admin::{
    CreateInviteInput, InviteModel, PasskeyAdminModel, UserModel, create_invite, get_all_passkeys,
    get_audit_logs, get_invites, get_tenant_stats, get_users, impersonate_user, resend_invite,
    revoke_invite, revoke_passkey_admin, toggle_admin,
};
use crate::api::audit_logs::audit_logs_export_url;
use crate::api::rbac::{AssignRoleInput, assign_role, list_role_profiles, revoke_role};
use crate::app::GlobalToast;
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlatformInviteRole {
    SuperAdmin,
    Admin,
    Editor,
    Viewer,
}

impl PlatformInviteRole {
    #[allow(dead_code)]
    fn as_str(self) -> &'static str {
        match self {
            Self::SuperAdmin => "Super-Admin",
            Self::Admin => "Admin",
            Self::Editor => "Editor",
            Self::Viewer => "Viewer",
        }
    }

    fn parse(s: &str) -> Option<Self> {
        match s {
            "Super-Admin" => Some(Self::SuperAdmin),
            "Admin" => Some(Self::Admin),
            "Editor" => Some(Self::Editor),
            "Viewer" => Some(Self::Viewer),
            _ => None,
        }
    }
}

#[component]
pub fn PlatformAdmins() -> impl IntoView {
    let active_network = use_context::<ReadSignal<Option<Uuid>>>().expect("active network context");
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");

    // UI state
    let active_tab = RwSignal::new("users".to_string());
    let refetch_trigger = RwSignal::new(0);

    // Filters (operators tab — stitch filter-bar; no unused tenant dropdown)
    let search_query = RwSignal::new(String::new());
    let role_filter = RwSignal::new("All roles".to_string());
    let status_filter = RwSignal::new("All statuses".to_string());

    // Modals
    let show_invite_modal = RwSignal::new(false);
    let invite_step = RwSignal::new(1u8); // 1 = Who, 2 = Access
    let show_manage_modal = RwSignal::new(None::<UserModel>);
    let confirm_revoke_invite = RwSignal::new(None::<(Uuid, String)>);
    let confirm_revoke_passkey = RwSignal::new(None::<Uuid>);

    // Invite form — Step 1: Who
    let invite_display_name = RwSignal::new(String::new());
    let invite_email = RwSignal::new(String::new());
    let invite_personal_msg = RwSignal::new(String::new());

    // Invite form — Step 2: Access
    let invite_platform_role = RwSignal::new("Admin".to_string());
    let invite_app_role = RwSignal::new(String::new());
    let invite_tenant = RwSignal::new(String::new());
    let invite_app_instance = RwSignal::new(String::new()); // Uuid as string
    let invite_target_url = RwSignal::new(String::new());
    let invite_expires_days = RwSignal::new(7i64);
    let invite_sending = RwSignal::new(false);
    let invite_sent_to = RwSignal::new(None::<String>); // email of last sent

    // RBAC (G-32)
    let rbac_app_slug = RwSignal::new("folio".to_string());
    let rbac_user_id = RwSignal::new(String::new());
    let rbac_role_slug = RwSignal::new(String::new());
    let rbac_refetch = RwSignal::new(0u32);

    // Passkeys ops
    let passkeys_refetch = RwSignal::new(0u32);
    let passkey_search = RwSignal::new(String::new());

    // Live invitations resource
    let invites_refetch = RwSignal::new(0);
    let invites_res = LocalResource::new(move || {
        let _ = invites_refetch.get();
        async move { get_invites().await.unwrap_or_default() }
    });

    // Resource hooks for actual users
    let users_res = LocalResource::new(move || {
        let n = active_network.get();
        let _ = refetch_trigger.get();
        async move { get_users(n).await.unwrap_or_default() }
    });

    // Audit log resource
    let audit_logs_res =
        LocalResource::new(|| async move { get_audit_logs().await.unwrap_or_default() });

    let passkeys_res = LocalResource::new(move || {
        let _ = passkeys_refetch.get();
        async move { get_all_passkeys(None).await.unwrap_or_default() }
    });

    let role_profiles_res = LocalResource::new(move || {
        let slug = rbac_app_slug.get();
        let _ = rbac_refetch.get();
        async move { list_role_profiles(&slug).await.unwrap_or_default() }
    });

    // Tenant list for the invite modal dropdown
    let tenants_for_invite =
        LocalResource::new(|| async move { get_tenant_stats().await.unwrap_or_default() });

    // Deep-link: /team#passkeys
    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            if let Ok(hash) = window.location().hash() {
                let h = hash.trim_start_matches('#').to_lowercase();
                if h == "passkeys" || h == "security" {
                    active_tab.set("passkeys".to_string());
                }
            }
        }
    });

    let filtered_users = Signal::derive(move || {
        let users = users_res.get().unwrap_or_default();
        let q = search_query.get().to_lowercase();
        let role = role_filter.get();
        let status = status_filter.get();
        users
            .into_iter()
            .filter(|u| {
                if !q.is_empty()
                    && !u.username.to_lowercase().contains(&q)
                    && !u.email.to_lowercase().contains(&q)
                {
                    return false;
                }
                let role_str = if u.is_admin { "Super-Admin" } else { "Admin" };
                if role != "All roles" && role != role_str {
                    return false;
                }
                match status.as_str() {
                    "Active" if !u.is_active => return false,
                    "Suspended" if u.is_active => return false,
                    _ => {}
                }
                true
            })
            .collect::<Vec<_>>()
    });

    // Impersonate
    let handle_impersonate = move |id: Uuid| {
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match impersonate_user(id).await {
                Ok(_) => {
                    t_toast.show_toast(
                        "Success",
                        "Impersonation session initiated. Redirecting...",
                        "success",
                    );
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    let _ = web_sys::window().unwrap().location().assign("/");
                }
                Err(e) => {
                    t_toast.show_toast("Error", &format!("Impersonation failed: {}", e), "error");
                }
            }
        });
    };

    // Toggle admin helper
    let handle_toggle_admin = move |id: Uuid| {
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match toggle_admin(id).await {
                Ok(updated) => {
                    t_toast.show_toast(
                        "Success",
                        &format!("Updated admin status for {}", updated.email),
                        "success",
                    );
                    refetch_trigger.update(|v| *v += 1);
                }
                Err(e) => {
                    t_toast.show_toast("Error", &format!("Failed: {}", e), "error");
                }
            }
            show_manage_modal.set(None);
        });
    };

    // Submit Invite User
    let submit_invite_user = move |_| {
        let email = invite_email.get();
        if email.trim().is_empty() {
            toast.show_toast("Error", "Email is required.", "error");
            return;
        }
        invite_sending.set(true);
        let display_name = invite_display_name.get();
        let personal_msg = invite_personal_msg.get();
        let platform_role = invite_platform_role.get();
        if PlatformInviteRole::parse(&platform_role).is_none() {
            toast.show_toast("Error", "Invalid platform role.", "error");
            return;
        }
        let app_role = invite_app_role.get();
        let tenant = invite_tenant.get();
        let app_instance = invite_app_instance.get();
        let target_url = invite_target_url.get();
        let expires_days = invite_expires_days.get();
        let t_toast = toast.clone();

        let input = CreateInviteInput {
            email: email.clone(),
            display_name: if display_name.trim().is_empty() {
                None
            } else {
                Some(display_name.trim().to_string())
            },
            role: platform_role,
            app_role: if app_role.trim().is_empty() {
                None
            } else {
                Some(app_role.trim().to_string())
            },
            tenant,
            app_instance_id: uuid::Uuid::parse_str(&app_instance).ok(),
            target_app_url: if target_url.trim().is_empty() {
                None
            } else {
                Some(target_url.trim().to_string())
            },
            personal_message: if personal_msg.trim().is_empty() {
                None
            } else {
                Some(personal_msg.trim().to_string())
            },
            expires_days: Some(expires_days),
        };

        leptos::task::spawn_local(async move {
            match create_invite(input).await {
                Ok(_) => {
                    t_toast.show_toast(
                        "Invitation sent",
                        &format!("Magic link dispatched to {}", email),
                        "success",
                    );
                    invite_sent_to.set(Some(email.clone()));
                    invites_refetch.update(|v| *v += 1);
                    // reset form
                    invite_display_name.set(String::new());
                    invite_email.set(String::new());
                    invite_personal_msg.set(String::new());
                    invite_app_role.set(String::new());
                    invite_app_instance.set(String::new());
                    invite_target_url.set(String::new());
                    invite_step.set(1);
                }
                Err(e) => {
                    t_toast.show_toast(
                        "Error",
                        &format!("Failed to send invitation: {}", e),
                        "error",
                    );
                }
            }
            invite_sending.set(false);
        });
    };

    // Revoke Invite helper (called after confirm)
    let handle_revoke_invite = move |id: Uuid, email: String| {
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match revoke_invite(id).await {
                Ok(_) => {
                    t_toast.show_toast(
                        "Warning",
                        &format!("Invitation to {} revoked.", email),
                        "warn",
                    );
                    invites_refetch.update(|v| *v += 1);
                }
                Err(e) => {
                    t_toast.show_toast(
                        "Error",
                        &format!("Failed to revoke invitation: {}", e),
                        "error",
                    );
                }
            }
            confirm_revoke_invite.set(None);
        });
    };

    // Resend Invite helper
    let handle_resend_invite = move |id: Uuid, email: String| {
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match resend_invite(id).await {
                Ok(_) => {
                    t_toast.show_toast(
                        "Success",
                        &format!("Invitation resent to {}", email),
                        "success",
                    );
                    invites_refetch.update(|v| *v += 1);
                }
                Err(e) => {
                    t_toast.show_toast(
                        "Error",
                        &format!("Failed to resend invitation: {}", e),
                        "error",
                    );
                }
            }
        });
    };

    view! {
        <div class="main-canvas">
        // Page Header
        <div class="page-header">
            <div>
                <div class="page-title">"Team"</div>
                <div class="page-subtitle">"Platform operators · invites · roles · access audit"</div>
            </div>
            <div class="page-actions">
            <button class="btn btn-primary btn-sm" on:click=move |_| {
                invite_email.set(String::new());
                show_invite_modal.set(true);
            }>"+ Invite teammate"</button>
            </div>
        </div>

        <div class="card" style="padding:10px 14px;border-left:3px solid var(--cobalt);margin-bottom:4px">
            <p class="muted" style="font-size:11.5px;margin:0">
                <strong style="color:var(--text-primary)">"Live data path. "</strong>
                "KPIs and tables bind to "
                <code style="font-size:10.5px">"GET /api/admin/users"</code>
                ", invites, and audit logs. Empty API → empty table + invite CTA — no invented operators."
            </p>
        </div>

        // KPI Row
        <div class="kpi-row">
            <div class="kpi-card">
                <span class="kpi-label">"Operators"</span>
                <span class="kpi-value">
                    {move || users_res.get().map(|v| v.len().to_string()).unwrap_or_else(|| "—".to_string())}
                </span>
                <span class="kpi-sub">"From user list"</span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Super-Admins"</span>
                <span class="kpi-value" style="color:var(--cobalt)">
                    {move || users_res.get().map(|v| v.iter().filter(|u| u.is_admin).count().to_string()).unwrap_or_else(|| "—".to_string())}
                </span>
                <span class="kpi-sub">"Platform-wide"</span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Pending invites"</span>
                <span class="kpi-value" style="color:var(--amber)">
                    {move || invites_res.get().map(|list| list.len().to_string()).unwrap_or_else(|| "0".to_string())}
                </span>
                <span class="kpi-sub">"Awaiting accept"</span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Passkeys enrolled"</span>
                <span class="kpi-value">
                    {move || passkeys_res.get().map(|v| v.len().to_string()).unwrap_or_else(|| "—".to_string())}
                </span>
                <span class="kpi-sub">"WebAuthn"</span>
            </div>
        </div>

        // Tab Bar
        <div class="tab-bar">
            <button
                class=move || if active_tab.get() == "users" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("users".to_string())
            >
                "All operators"
            </button>
            <button
                class=move || if active_tab.get() == "invites" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("invites".to_string())
            >
                "Pending invites"
                <Show when=move || invites_res.get().map(|list| !list.is_empty()).unwrap_or(false)>
                    <span style="font-size:10px;color:var(--amber);margin-left:4px">
                        {move || invites_res.get().map(|list| list.len()).unwrap_or(0)}
                    </span>
                </Show>
            </button>
            <button
                class=move || if active_tab.get() == "roles" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("roles".to_string())
            >
                "Roles & permissions"
            </button>
            <button
                class=move || if active_tab.get() == "audit" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("audit".to_string())
            >
                "Access audit"
            </button>
            <button
                class=move || if active_tab.get() == "passkeys" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("passkeys".to_string())
            >
                "Passkeys"
            </button>
        </div>

        // Body Content
        <div>
            // ── Tab: All operators ──
            <Show when=move || active_tab.get() == "users">
                <div class="card">
                    <div class="filter-bar">
                        <input
                            class="f-input"
                            placeholder="Filter name or email…"
                            prop:value=search_query
                            on:input=move |ev| search_query.set(event_target_value(&ev))
                        />
                        <select
                            class="f-select"
                            on:change=move |ev| role_filter.set(event_target_value(&ev))
                        >
                            <option value="All roles">"All roles"</option>
                            <option value="Super-Admin">"Super-Admin"</option>
                            <option value="Admin">"Admin"</option>
                            <option value="Editor">"Editor"</option>
                            <option value="Viewer">"Viewer"</option>
                        </select>
                        <select
                            class="f-select"
                            on:change=move |ev| status_filter.set(event_target_value(&ev))
                        >
                            <option value="All statuses">"All statuses"</option>
                            <option value="Active">"Active"</option>
                            <option value="Suspended">"Suspended"</option>
                        </select>
                    </div>

                    <Show
                        when=move || !filtered_users.get().is_empty()
                        fallback=move || {
                            let has_users = users_res.get().map(|v| !v.is_empty()).unwrap_or(false);
                            view! {
                                <div class="n-empty">
                                    <div class="n-empty-title">
                                        {if has_users { "No matching operators" } else { "No operators yet" }}
                                    </div>
                                    <div class="n-empty-sub">
                                        {if has_users {
                                            "Try clearing filters, or invite someone new."
                                        } else {
                                            "Invite the first teammate. They receive a magic link and land on Account Settings to enroll a passkey."
                                        }}
                                    </div>
                                    <button class="btn btn-primary btn-sm" on:click=move |_| {
                                        invite_email.set(String::new());
                                        show_invite_modal.set(true);
                                    }>"Invite teammate"</button>
                                </div>
                            }
                        }
                    >
                    <table>
                        <thead>
                            <tr>
                                <th style="width:36px"></th>
                                <th>"Name"</th>
                                <th>"Email"</th>
                                <th>"Role"</th>
                                <th>"Status"</th>
                                <th>"Last login"</th>
                                <th>"Auth"</th>
                                <th style="width:80px"></th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                filtered_users.get().into_iter().map(|u| {
                                        let u_clone = u.clone();
                                        let initials = {
                                            let mut chars = u.username.chars();
                                            let a = chars.next().unwrap_or('U');
                                            let b = chars.next();
                                            if let Some(b) = b {
                                                format!("{}{}", a, b).to_uppercase()
                                            } else {
                                                a.to_uppercase().to_string()
                                            }
                                        };
                                        let is_super = u.is_admin;
                                        let role_str = if is_super { "Super-Admin" } else { "Admin" };
                                        let role_sty = if is_super {
                                            "color:var(--cobalt);border-color:var(--cobalt)"
                                        } else {
                                            "color:var(--violet);border-color:var(--violet)"
                                        };
                                        let (status_sty, status_label) = if u.is_active {
                                            ("color:var(--green)", "Active")
                                        } else {
                                            ("color:var(--text-muted)", "Suspended")
                                        };
                                        // Passkey count from registry for this email
                                        let auth_label = passkeys_res.get().map(|pks| {
                                            if pks.iter().any(|pk| pk.user_email.eq_ignore_ascii_case(&u.email)) {
                                                "Passkey"
                                            } else {
                                                "—"
                                            }
                                        }).unwrap_or("—");
                                        view! {
                                            <tr>
                                                <td>
                                                    <div class="user-avatar">{initials}</div>
                                                </td>
                                                <td style="font-weight:500">{u.username.clone()}</td>
                                                <td class="muted">{u.email.clone()}</td>
                                                <td>
                                                    <span class="pill" style=role_sty>
                                                        {role_str}
                                                    </span>
                                                </td>
                                                <td style=status_sty>{status_label}</td>
                                                <td class="muted">"—"</td>
                                                <td class="muted">{auth_label}</td>
                                                <td>
                                                    <button
                                                        class="btn btn-ghost btn-sm"
                                                        on:click=move |e| {
                                                            e.stop_propagation();
                                                            show_manage_modal.set(Some(u_clone.clone()));
                                                        }
                                                    >
                                                        "Manage"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                }).collect_view()
                            }}
                        </tbody>
                    </table>
                    </Show>
                </div>
            </Show>

            // ── Tab: Pending Invites ──
            <Show when=move || active_tab.get() == "invites">
                <div class="card">
                    <Show
                        when=move || !invites_res.get().unwrap_or_default().is_empty()
                        fallback=move || view! {
                            <div class="n-empty">
                                <div class="n-empty-title">"No pending invites"</div>
                                <div class="n-empty-sub">"Sent invites appear here until accepted or revoked."</div>
                                <button class="btn btn-primary btn-sm" on:click=move |_| show_invite_modal.set(true)>"Invite teammate"</button>
                            </div>
                        }
                    >
                    <table>
                        <thead>
                            <tr>
                                <th>"Email"</th>
                                <th>"Role"</th>
                                <th>"Invited by"</th>
                                <th>"Sent"</th>
                                <th>"Expires"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let invites = invites_res.get().unwrap_or_default();
                                view! {
                                    <For
                                        each=move || invites.clone()
                                        key=|i: &InviteModel| i.id.clone()
                                        children=move |invite| {
                                            let invite_id = invite.id;
                                            let invite_id2 = invite.id;
                                            let email_clone = invite.email.clone();
                                            let email_clone2 = invite.email.clone();
                                            let role_color = match invite.role.as_str() {
                                                "Super-Admin" => "color:var(--cobalt);border-color:var(--cobalt)",
                                                "Admin" => "color:var(--violet);border-color:var(--violet)",
                                                "Editor" => "color:var(--amber);border-color:var(--amber)",
                                                _ => "color:var(--text-muted);border-color:var(--border-default)",
                                            };
                                            view! {
                                                <tr>
                                                    <td style="color:var(--cobalt)">{invite.email}</td>
                                                    <td><span class="pill" style=role_color>{invite.role}</span></td>
                                                    <td class="muted">{invite.invited_by}</td>
                                                    <td class="muted">{invite.sent}</td>
                                                    <td class="muted" style="color:var(--amber)">{invite.expires}</td>
                                                    <td style="display:flex;gap:4px;">
                                                        <button
                                                            class="btn btn-ghost btn-sm"
                                                            on:click=move |_| handle_resend_invite(invite_id, email_clone.clone())
                                                        >
                                                            "Resend"
                                                        </button>
                                                        <button
                                                            class="btn btn-ghost btn-sm"
                                                            on:click=move |_| confirm_revoke_invite.set(Some((invite_id2, email_clone2.clone())))
                                                        >
                                                            "Revoke"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    />
                                }.into_any()
                            }}
                        </tbody>
                    </table>
                    </Show>
                </div>
            </Show>

            // ── Tab: Roles & Permissions ──
            <Show when=move || active_tab.get() == "roles">
                <div class="two-col">
                    <div class="card">
                        <div class="card-hdr">
                            <span class="card-title">"Platform roles"</span>
                        </div>
                        <div class="role-row">
                            <span class="role-name" style="color:var(--cobalt)">"Super-Admin"</span>
                            <span class="role-desc">"Full console: tenants, billing, impersonation, flags, team."</span>
                        </div>
                        <div class="role-row">
                            <span class="role-name" style="color:var(--violet)">"Admin"</span>
                            <span class="role-desc">"Operate assigned tenants; cannot manage platform operators."</span>
                        </div>
                        <div class="role-row">
                            <span class="role-name" style="color:var(--amber)">"Editor"</span>
                            <span class="role-desc">"CRM / content write; no billing or user admin."</span>
                        </div>
                        <div class="role-row">
                            <span class="role-name" style="color:var(--text-muted)">"Viewer"</span>
                            <span class="role-desc">"Read-only within assigned scope."</span>
                        </div>
                    </div>
                    <div class="card">
                        <div class="card-hdr">
                            <span class="card-title">"Security invariants"</span>
                        </div>
                        <div class="team-stat-row">
                            <span class="role-desc">"Impersonation"</span>
                            <span style="font-size:12px;color:var(--cobalt);font-weight:500">"Super-Admin only · audited"</span>
                        </div>
                        <div class="team-stat-row">
                            <span class="role-desc">"Toggle Super-Admin"</span>
                            <span style="font-size:12px;color:var(--cobalt);font-weight:500">"Cannot demote self"</span>
                        </div>
                        <div class="team-stat-row">
                            <span class="role-desc">"Invite delivery"</span>
                            <span style="font-size:12px;font-weight:500">"OutboxWorker · non-fatal"</span>
                        </div>
                        <div class="team-stat-row">
                            <span class="role-desc">"Passkey registry"</span>
                            <span style="font-size:12px;font-weight:500">
                                <a href="/team#passkeys" style="color:var(--cobalt);text-decoration:none" on:click=move |_| active_tab.set("passkeys".to_string())>
                                    "Multi-tenant WebAuthn →"
                                </a>
                            </span>
                        </div>
                    </div>
                </div>
                <div class="card" style="margin-top:4px">
                    <div class="card-hdr">
                        <span class="card-title">"App role profiles (G-32)"</span>
                        <div style="display:flex;gap:8px;align-items:center">
                            <input class="f-input" style="max-width:120px;flex:none" placeholder="app_slug"
                                prop:value=move || rbac_app_slug.get()
                                on:input=move |ev| rbac_app_slug.set(event_target_value(&ev))/>
                            <button class="btn btn-ghost btn-sm" on:click=move |_| rbac_refetch.update(|v| *v += 1)>"Load"</button>
                        </div>
                    </div>
                    <div class="two-col" style="margin:0;border:none">
                        <div>
                            <Suspense fallback=move || view! { <div class="p-4 muted text-sm">"Loading roles…"</div> }>
                            {move || {
                                let profiles = role_profiles_res.get().unwrap_or_default();
                                if profiles.is_empty() {
                                    view! {
                                        <div class="n-empty" style="padding:24px">
                                            <div class="n-empty-sub" style="margin-bottom:0">
                                                "No role profiles for this app_slug (or insufficient rbac:read). Try folio."
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    profiles.into_iter().map(|p| {
                                        let slug = p.role_slug.clone();
                                        view! {
                                            <div class="role-row" style="cursor:pointer" on:click=move |_| rbac_role_slug.set(slug.clone())>
                                                <span class="role-name" style="color:var(--cobalt)">{p.display_name.clone()}</span>
                                                <span class="role-desc">{p.description.clone().unwrap_or_else(|| p.role_slug.clone())}</span>
                                                <span class="role-count mono">{p.role_slug.clone()}</span>
                                            </div>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                            </Suspense>
                        </div>
                        <div style="padding:14px;border-left:1px solid var(--border-default);display:flex;flex-direction:column;gap:10px">
                            <div style="font-size:11.5px;font-weight:600">"Assign / revoke"</div>
                            <div class="form-row" style="margin:0">
                                <label class="form-label">"User ID"</label>
                                <input class="form-input" placeholder="uuid"
                                    prop:value=move || rbac_user_id.get()
                                    on:input=move |ev| rbac_user_id.set(event_target_value(&ev))/>
                            </div>
                            <div class="form-row" style="margin:0">
                                <label class="form-label">"Role slug"</label>
                                <input class="form-input" placeholder="tenant | owner | …"
                                    prop:value=move || rbac_role_slug.get()
                                    on:input=move |ev| rbac_role_slug.set(event_target_value(&ev))/>
                            </div>
                            <div style="display:flex;gap:8px;flex-wrap:wrap">
                                <button class="btn btn-primary btn-sm" on:click=move |_| {
                                    let Ok(uid) = Uuid::parse_str(&rbac_user_id.get()) else {
                                        toast.show_toast("Error", "Invalid user UUID.", "error");
                                        return;
                                    };
                                    let app = rbac_app_slug.get();
                                    let role = rbac_role_slug.get();
                                    if role.trim().is_empty() {
                                        toast.show_toast("Error", "Role slug required.", "error");
                                        return;
                                    }
                                    let t = toast.clone();
                                    leptos::task::spawn_local(async move {
                                        match assign_role(uid, AssignRoleInput { app_slug: app, role_slug: role }).await {
                                            Ok(_) => t.show_toast("Assigned", "Role assigned.", "success"),
                                            Err(e) => t.show_toast("Error", &e, "error"),
                                        }
                                    });
                                }>"Assign role"</button>
                                <button class="btn btn-ghost btn-sm" style="color:var(--red)" on:click=move |_| {
                                    let Ok(uid) = Uuid::parse_str(&rbac_user_id.get()) else {
                                        toast.show_toast("Error", "Invalid user UUID.", "error");
                                        return;
                                    };
                                    let app = rbac_app_slug.get();
                                    let t = toast.clone();
                                    leptos::task::spawn_local(async move {
                                        match revoke_role(uid, &app).await {
                                            Ok(_) => t.show_toast("Revoked", "Role revoked.", "warn"),
                                            Err(e) => t.show_toast("Error", &e, "error"),
                                        }
                                    });
                                }>"Revoke for app"</button>
                            </div>
                            <p class="muted" style="font-size:11px;margin:0">
                                "Platform Super-Admin is toggled on All operators. App roles use /api/rbac."
                            </p>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Tab: Audit Log ──
            <Show when=move || active_tab.get() == "audit">
                <div class="card">
                    <div class="card-hdr">
                        <span class="card-title">"Recent access events"</span>
                        <div style="display:flex;gap:8px">
                            <a class="btn btn-ghost btn-sm" href="/logs">"Open full audit ledger →"</a>
                            <a
                                class="btn btn-ghost btn-sm"
                                href=move || audit_logs_export_url(None, None, "", "")
                                download="audit-logs.csv"
                            >"Export CSV"</a>
                        </div>
                    </div>
                    <Suspense fallback=move || view! { <div style="padding:24px;text-align:center;color:var(--text-muted);font-size:12px;">"Loading audit log..."</div> }>
                    <table>
                        <thead>
                            <tr>
                                <th>"Time"</th>
                                <th>"Actor"</th>
                                <th>"Action"</th>
                                <th>"Target"</th>
                                <th>"Entity"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let logs = audit_logs_res.get().unwrap_or_default();
                                if logs.is_empty() {
                                    view! {
                                        <tr>
                                            <td colspan="5" class="muted" style="text-align:center;padding:24px;">
                                                "No access events yet."
                                            </td>
                                        </tr>
                                    }.into_any()
                                } else {
                                    logs.into_iter().map(|log| {
                                        let ts = log.created_at.chars().take(16).collect::<String>();
                                        let actor = log.actor_id.map(|id| id.to_string().chars().take(8).collect::<String>() + "…").unwrap_or_else(|| "system".to_string());
                                        let entity = log.entity_id.to_string().chars().take(8).collect::<String>() + "…";
                                        view! {
                                            <tr>
                                                <td class="muted mono" style="font-size:11px;">{ts}</td>
                                                <td class="muted mono" style="font-size:11px;">{actor}</td>
                                                <td class="cobalt" style="font-size:11px;">{log.action_type}</td>
                                                <td class="muted" style="font-size:11px;">{log.entity_type}</td>
                                                <td class="muted mono" style="font-size:11px;">{entity}</td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </tbody>
                    </table>
                    </Suspense>
                </div>
            </Show>

            // ── Tab: Passkeys (platform ops registry) ──
            <Show when=move || active_tab.get() == "passkeys">
                <div class="card">
                    <div class="card-hdr">
                        <span class="card-title">"Platform passkey registry"</span>
                        <button class="btn btn-ghost btn-sm" on:click=move |_| passkeys_refetch.update(|v| *v += 1)>"Refresh"</button>
                    </div>
                    <div class="filter-bar">
                        <input class="f-input" placeholder="Search email or device…"
                            prop:value=move || passkey_search.get()
                            on:input=move |ev| passkey_search.set(event_target_value(&ev))/>
                    </div>
                    <table>
                        <thead>
                            <tr>
                                <th>"User"</th>
                                <th>"Device"</th>
                                <th>"Sign count"</th>
                                <th>"Last used"</th>
                                <th>"Registered"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let q = passkey_search.get().to_lowercase();
                                let pks: Vec<PasskeyAdminModel> = passkeys_res.get().unwrap_or_default()
                                    .into_iter()
                                    .filter(|pk| q.is_empty()
                                        || pk.user_email.to_lowercase().contains(&q)
                                        || pk.name.to_lowercase().contains(&q))
                                    .collect();
                                if pks.is_empty() {
                                    view! {
                                        <tr><td colspan="6">
                                            <div class="n-empty" style="padding:28px">
                                                <div class="n-empty-title">"No passkeys registered"</div>
                                                <div class="n-empty-sub" style="margin-bottom:0">
                                                    "Operators enroll passkeys from Account Settings → Security."
                                                </div>
                                            </div>
                                        </td></tr>
                                    }.into_any()
                                } else {
                                    pks.into_iter().map(|pk| {
                                        let pk_id = pk.id;
                                        let last = pk.last_used_at.clone().unwrap_or_else(|| "—".into());
                                        let created = pk.created_at.chars().take(10).collect::<String>();
                                        view! {
                                            <tr>
                                                <td>
                                                    <div style="font-weight:500">{pk.user_email.clone()}</div>
                                                    <div class="mono muted" style="font-size:10px">{pk.user_id.to_string().chars().take(8).collect::<String>()}"…"</div>
                                                </td>
                                                <td class="muted">{pk.name.clone()}</td>
                                                <td class="mono">{pk.sign_count}</td>
                                                <td class="muted">{last}</td>
                                                <td class="muted">{created}</td>
                                                <td>
                                                    <button class="btn btn-ghost btn-sm" style="color:var(--red)"
                                                        on:click=move |_| confirm_revoke_passkey.set(Some(pk_id))
                                                    >"Revoke"</button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view().into_any()
                                }
                            }}
                        </tbody>
                    </table>
                    <div style="padding:12px 14px;font-size:12px;color:var(--text-secondary)">
                        "Revoking a passkey is immediate. Self-service enrollment: "
                        <a href="/settings#security" style="color:var(--cobalt)">"Account Settings → Security"</a>
                        "."
                    </div>
                </div>
            </Show>
        </div>

        // ── Modal: Invite User (2-step) ───────────────────────────────────────
        <Show when=move || show_invite_modal.get()>
            <div class="fixed inset-0 z-[100] bg-black/85 backdrop-blur-md flex items-center justify-center p-4">
                <div class="bg-surface w-full max-w-xl rounded-2xl border border-white/10 shadow-2xl overflow-hidden relative">

                    // ── Close button ─────────────────────────────────────────
                    <button
                        class="absolute top-4 right-4 z-10 w-8 h-8 flex items-center justify-center rounded-full text-slate-400 hover:text-white hover:bg-white/10 transition-colors"
                        on:click=move |_| {
                            show_invite_modal.set(false);
                            invite_step.set(1);
                            invite_sent_to.set(None);
                        }
                    >"✕"</button>

                    // ── Header ────────────────────────────────────────────────
                    <div class="px-8 pt-8 pb-4 border-b border-white/5">
                        <div class="flex items-center gap-3 mb-1">
                            <span style="font-size:22px;">"✉️"</span>
                            <h3 class="text-lg font-bold text-white">
                                {move || if invite_sent_to.get().is_some() { "Invitation Sent!" } else { "Invite a User" }}
                            </h3>
                        </div>
                        // Step indicator (hidden on success screen)
                        <Show when=move || invite_sent_to.get().is_none()>
                            <div class="flex items-center gap-2 mt-3">
                                {(1u8..=2).map(|n| {
                                    let is_active   = move || invite_step.get() == n;
                                    let is_complete = move || invite_step.get() > n;
                                    view! {
                                        <div class="flex items-center gap-2">
                                            <div
                                                class="w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold transition-all"
                                                style=move || if is_complete() {
                                                    "background:#22c55e;color:#fff;"
                                                } else if is_active() {
                                                    "background:#6366f1;color:#fff;"
                                                } else {
                                                    "background:#1e2433;color:#64748b;border:1px solid #2d3548;"
                                                }
                                            >
                                                {move || if is_complete() { "✓".to_string() } else { n.to_string() }}
                                            </div>
                                            <span
                                                class="text-xs font-medium"
                                                style=move || if is_active() { "color:#a5b4fc;" } else { "color:#64748b;" }
                                            >
                                                {if n == 1 { "Who" } else { "Access" }}
                                            </span>
                                            {(n < 2).then(|| view! {
                                                <div class="w-12 h-px bg-white/10 mx-1"></div>
                                            })}
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        </Show>
                    </div>

                    // ── Step 1: Who ───────────────────────────────────────────
                    <Show when=move || invite_step.get() == 1 && invite_sent_to.get().is_none()>
                        <div class="px-8 py-6 space-y-5">
                            <div class="n-form-row">
                                <label class="n-form-label">"Full Name"</label>
                                <input
                                    type="text"
                                    id="invite-display-name"
                                    class="n-form-input"
                                    placeholder="e.g. Sarah Chen"
                                    prop:value=invite_display_name
                                    on:input=move |ev| invite_display_name.set(event_target_value(&ev))
                                />
                                <p class="text-xs text-slate-500 mt-1">"Pre-fills their profile. They can update it after signing in."</p>
                            </div>
                            <div class="n-form-row">
                                <label class="n-form-label">"Email Address " <span style="color:#f87171;">"*"</span></label>
                                <input
                                    type="email"
                                    id="invite-email"
                                    class="n-form-input"
                                    placeholder="e.g. sarah@company.com"
                                    prop:value=invite_email
                                    on:input=move |ev| invite_email.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="n-form-row">
                                <label class="n-form-label">"Personal Note " <span class="text-slate-500 font-normal">"(optional)"</span></label>
                                <textarea
                                    id="invite-personal-msg"
                                    class="n-form-input"
                                    rows="3"
                                    placeholder="Add a short message that will appear in the invitation email…"
                                    style="resize:vertical;"
                                    prop:value=invite_personal_msg
                                    on:input=move |ev| invite_personal_msg.set(event_target_value(&ev))
                                ></textarea>
                            </div>
                        </div>
                        <div class="px-8 pb-6 flex justify-end">
                            <button
                                class="btn btn-primary"
                                on:click=move |_| {
                                    if invite_email.get().trim().is_empty() {
                                        toast.show_toast("Error", "Email is required to continue.", "error");
                                        return;
                                    }
                                    invite_step.set(2);
                                }
                            >"Next: Access →"</button>
                        </div>
                    </Show>

                    // ── Step 2: Access ────────────────────────────────────────
                    <Show when=move || invite_step.get() == 2 && invite_sent_to.get().is_none()>
                        <div class="px-8 py-6 space-y-5">
                            // Quick identity recap
                            <div class="flex items-center gap-3 p-3 rounded-xl bg-white/5 border border-white/5">
                                <div class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold text-white">
                                    {move || invite_display_name.get().chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or("?".to_string())}
                                </div>
                                <div>
                                    <p class="text-sm font-medium text-white">{move || {
                                        let n = invite_display_name.get();
                                        if n.trim().is_empty() { invite_email.get() } else { n }
                                    }}</p>
                                    <p class="text-xs text-slate-400">{move || invite_email.get()}</p>
                                </div>
                            </div>

                            <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;">
                                <div class="n-form-row">
                                    <label class="n-form-label">"Platform Privilege"</label>
                                    <select
                                        id="invite-platform-role"
                                        class="n-form-select"
                                        on:change=move |ev| invite_platform_role.set(event_target_value(&ev))
                                    >
                                        <option value="Admin">"Admin"</option>
                                        <option value="Super-Admin">"Super-Admin"</option>
                                        <option value="Editor">"Editor"</option>
                                        <option value="Viewer">"Viewer"</option>
                                    </select>
                                </div>
                                <div class="n-form-row">
                                    <label class="n-form-label">"App Role " <span class="text-slate-500 font-normal">"(optional)"</span></label>
                                    <input
                                        type="text"
                                        id="invite-app-role"
                                        class="n-form-input"
                                        placeholder="e.g. landlord, tenant, pmc…"
                                        prop:value=invite_app_role
                                        on:input=move |ev| invite_app_role.set(event_target_value(&ev))
                                    />
                                    <p class="text-xs text-slate-500 mt-1">"Interpreted by the target app."</p>
                                </div>
                            </div>

                            <div class="n-form-row">
                                <label class="n-form-label">"Tenant"</label>
                                <select
                                    id="invite-tenant"
                                    class="n-form-select"
                                    on:change=move |ev| invite_tenant.set(event_target_value(&ev))
                                >
                                    <option value="">"— Platform-wide —"</option>
                                    {move || tenants_for_invite.get().unwrap_or_default().into_iter().map(|t| {
                                        let n = t.name.clone();
                                        let id = t.tenant_id.clone();
                                        view! { <option value=id>{n}</option> }
                                    }).collect_view()}
                                </select>
                            </div>

                            <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;">
                                <div class="n-form-row">
                                    <label class="n-form-label">"Link Destination " <span class="text-slate-500 font-normal">"(URL)"</span></label>
                                    <input
                                        type="url"
                                        id="invite-target-url"
                                        class="n-form-input"
                                        placeholder="https://folio.yourclient.com"
                                        prop:value=invite_target_url
                                        on:input=move |ev| invite_target_url.set(event_target_value(&ev))
                                    />
                                    <p class="text-xs text-slate-500 mt-1">"Leave blank to use the default app URL."</p>
                                </div>
                                <div class="n-form-row">
                                    <label class="n-form-label">"Link Expires"</label>
                                    <select
                                        id="invite-expires"
                                        class="n-form-select"
                                        on:change=move |ev| {
                                            let v: i64 = event_target_value(&ev).parse().unwrap_or(7);
                                            invite_expires_days.set(v);
                                        }
                                    >
                                        <option value="1">"24 hours"</option>
                                        <option value="7" selected>"7 days"</option>
                                        <option value="14">"14 days"</option>
                                        <option value="30">"30 days"</option>
                                    </select>
                                </div>
                            </div>
                        </div>
                        <div class="px-8 pb-6 flex items-center justify-between">
                            <button
                                class="btn btn-ghost flex items-center gap-1"
                                on:click=move |_| invite_step.set(1)
                            >"← Back"</button>
                            <button
                                class="btn btn-primary flex items-center gap-2"
                                disabled=move || invite_sending.get()
                                on:click=submit_invite_user
                            >
                                {move || if invite_sending.get() {
                                    view! { <span>"Sending…"</span> }.into_any()
                                } else {
                                    view! { <span>"Send Magic Link ✉️"</span> }.into_any()
                                }}
                            </button>
                        </div>
                    </Show>

                    // ── Success screen ────────────────────────────────────────
                    <Show when=move || invite_sent_to.get().is_some()>
                        <div class="px-8 py-10 flex flex-col items-center text-center gap-4">
                            // Icon circle
                            <div class="w-16 h-16 rounded-full bg-emerald-500/10 border border-emerald-500/20 flex items-center justify-center mb-1">
                                <span class="material-symbols-outlined text-3xl text-emerald-400">"mark_email_read"</span>
                            </div>
                            <div>
                                <h4 class="text-lg font-bold text-on-surface mb-1">"Invitation sent!"</h4>
                                <p class="text-xs text-on-surface-variant/60 mb-1">"A magic link has been dispatched to"</p>
                                <p class="text-sm font-semibold text-primary font-mono">
                                    {move || invite_sent_to.get().unwrap_or_default()}
                                </p>
                            </div>

                            // What happens next
                            <div class="w-full bg-surface-container/60 border border-outline-variant/20 rounded-xl p-4 text-left mt-1">
                                <p class="text-[10px] font-bold uppercase tracking-widest text-on-surface-variant/50 mb-3">"What happens next"</p>
                                <div class="space-y-2.5">
                                    <div class="flex items-start gap-2.5">
                                        <span class="w-4 h-4 rounded-full bg-primary/10 border border-primary/30 flex items-center justify-center text-[9px] font-bold text-primary shrink-0 mt-0.5">"1"</span>
                                        <p class="text-xs text-on-surface-variant/80">"The invitee receives an email with a secure magic link (expires in 7 days)."</p>
                                    </div>
                                    <div class="flex items-start gap-2.5">
                                        <span class="w-4 h-4 rounded-full bg-primary/10 border border-primary/30 flex items-center justify-center text-[9px] font-bold text-primary shrink-0 mt-0.5">"2"</span>
                                        <p class="text-xs text-on-surface-variant/80">"Clicking the link prompts them to set up a passkey (Touch ID / Face ID) for their account."</p>
                                    </div>
                                    <div class="flex items-start gap-2.5">
                                        <span class="w-4 h-4 rounded-full bg-primary/10 border border-primary/30 flex items-center justify-center text-[9px] font-bold text-primary shrink-0 mt-0.5">"3"</span>
                                        <p class="text-xs text-on-surface-variant/80">"Once registered, they appear in the Users table with their assigned role and scope."</p>
                                    </div>
                                </div>
                            </div>

                            // Actions
                            <div class="flex gap-3 mt-1 w-full justify-center">
                                <button
                                    class="btn btn-ghost"
                                    on:click=move |_| {
                                        invite_sent_to.set(None);
                                        invite_step.set(1);
                                    }
                                >"Invite Another →"</button>
                                <button
                                    class="btn btn-primary"
                                    on:click=move |_| {
                                        show_invite_modal.set(false);
                                        invite_sent_to.set(None);
                                        invite_step.set(1);
                                    }
                                >"Done"</button>
                            </div>
                        </div>
                    </Show>

                </div>
            </div>
        </Show>

        // Confirm revoke invite
        <Show when=move || confirm_revoke_invite.get().is_some()>
            {let (id, email) = confirm_revoke_invite.get().unwrap_or_else(|| (Uuid::nil(), String::new()));
             view! {
                <div style="position:fixed;inset:0;z-index:100;background:rgba(0,0,0,0.7);display:flex;align-items:center;justify-content:center;padding:16px">
                    <div style="background:var(--bg-surface);max-width:400px;width:100%;padding:24px;border-radius:12px;border:1px solid var(--border-default)">
                        <h3 style="font-size:16px;font-weight:600;margin-bottom:8px">"Revoke invite"</h3>
                        <p class="muted" style="font-size:12px;margin-bottom:16px">"Revoke invite for " <strong>{email.clone()}</strong> "?"</p>
                        <div style="display:flex;justify-content:flex-end;gap:8px">
                            <button class="btn btn-ghost" on:click=move |_| confirm_revoke_invite.set(None)>"Cancel"</button>
                            <button class="btn btn-primary" style="background:var(--red);border-color:var(--red)"
                                on:click=move |_| handle_revoke_invite(id, email.clone())>"Revoke"</button>
                        </div>
                    </div>
                </div>
             }}
        </Show>

        // Confirm revoke passkey
        <Show when=move || confirm_revoke_passkey.get().is_some()>
            {let pk_id = confirm_revoke_passkey.get().unwrap_or_else(Uuid::nil);
             view! {
                <div style="position:fixed;inset:0;z-index:100;background:rgba(0,0,0,0.7);display:flex;align-items:center;justify-content:center;padding:16px">
                    <div style="background:var(--bg-surface);max-width:400px;width:100%;padding:24px;border-radius:12px;border:1px solid var(--border-default)">
                        <h3 style="font-size:16px;font-weight:600;margin-bottom:8px">"Revoke passkey"</h3>
                        <p class="muted" style="font-size:12px;margin-bottom:16px">"This is immediate and irreversible for WebAuthn login."</p>
                        <div style="display:flex;justify-content:flex-end;gap:8px">
                            <button class="btn btn-ghost" on:click=move |_| confirm_revoke_passkey.set(None)>"Cancel"</button>
                            <button class="btn btn-primary" style="background:var(--red);border-color:var(--red)"
                                on:click=move |_| {
                                    let t = toast.clone();
                                    leptos::task::spawn_local(async move {
                                        match revoke_passkey_admin(pk_id).await {
                                            Ok(()) => {
                                                t.show_toast("Revoked", "Passkey revoked.", "success");
                                                passkeys_refetch.update(|v| *v += 1);
                                            }
                                            Err(e) => t.show_toast("Error", &e, "error"),
                                        }
                                        confirm_revoke_passkey.set(None);
                                    });
                                }
                            >"Revoke"</button>
                        </div>
                    </div>
                </div>
             }}
        </Show>

        // Modal: Manage User
        <Show when=move || show_manage_modal.get().is_some()>
            {let user = show_manage_modal.get().unwrap();
             let u_id  = user.id.clone();
             let is_admin = user.is_admin;
             view! {
                <div style="position:fixed;inset:0;z-index:100;background:rgba(0,0,0,0.7);backdrop-filter:blur(4px);display:flex;align-items:center;justify-content:center;padding:16px">
                    <div style="background:var(--bg-surface);width:100%;max-width:400px;padding:24px;border-radius:12px;border:1px solid var(--border-default);box-shadow:0 24px 64px rgba(0,0,0,0.6);position:relative;color:var(--text-primary)">
                        <button
                            style="position:absolute;top:14px;right:14px;background:none;border:none;color:var(--text-muted);cursor:pointer;font-size:16px;line-height:1"
                            on:click=move |_| show_manage_modal.set(None)
                        >"✕"</button>
                        <div style="margin-bottom:16px">
                            <div style="font-size:14px;font-weight:700;margin-bottom:4px">"Manage User"</div>
                            <div style="font-size:11px;color:var(--text-muted)">{user.username} " · " {user.email}</div>
                        </div>
                        <p class="muted" style="font-size:11px;margin-bottom:12px">"Toggle Super-Admin and impersonation are audit-logged."</p>
                        <div style="display:flex;flex-direction:column;gap:8px">
                            <button
                                on:click=move |_| handle_toggle_admin(u_id.clone())
                                class="btn btn-ghost"
                                style="width:100%;justify-content:center"
                            >
                                {if is_admin { "Revoke Super-Admin privilege" } else { "Grant Super-Admin privilege" }}
                            </button>
                            <button
                                on:click=move |_| handle_impersonate(u_id.clone())
                                class="btn btn-primary"
                                style="width:100%;justify-content:center"
                            >
                                "Impersonate User Session"
                            </button>
                            <button
                                class="btn btn-ghost btn-sm"
                                on:click=move |_| {
                                    rbac_user_id.set(u_id.to_string());
                                    show_manage_modal.set(None);
                                    active_tab.set("roles".to_string());
                                }
                            >"Assign app role (G-32) →"</button>
                        </div>
                        <div style="display:flex;justify-content:flex-end;padding-top:16px;margin-top:16px;border-top:1px solid var(--border-default)">
                            <button on:click=move |_| show_manage_modal.set(None) class="btn btn-ghost">"Close"</button>
                        </div>
                    </div>
                </div>
            }}
        </Show>
        </div> // end main-canvas
    }
}
