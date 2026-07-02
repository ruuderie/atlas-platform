use leptos::prelude::*;
use uuid::Uuid;
use crate::api::admin::{
    get_users, toggle_admin, UserModel, impersonate_user,
    get_invites, create_invite, revoke_invite, resend_invite, InviteModel, CreateInviteInput,
    get_audit_logs, get_tenant_stats,
};
use crate::app::GlobalToast;

#[component]
pub fn PlatformAdmins() -> impl IntoView {
    let active_network = use_context::<ReadSignal<Option<Uuid>>>().expect("active network context");
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");

    // UI state
    let active_tab = RwSignal::new("users".to_string());
    let refetch_trigger = RwSignal::new(0);
    
    // Filters
    let search_query = RwSignal::new(String::new());
    let role_filter = RwSignal::new("All Roles".to_string());
    let tenant_filter = RwSignal::new("All Tenants".to_string());
    let status_filter = RwSignal::new("All Statuses".to_string());

    // Modals
    let show_invite_modal  = RwSignal::new(false);
    let invite_step        = RwSignal::new(1u8);      // 1 = Who, 2 = Access
    let show_manage_modal  = RwSignal::new(None::<UserModel>);

    // Invite form — Step 1: Who
    let invite_display_name   = RwSignal::new(String::new());
    let invite_email          = RwSignal::new(String::new());
    let invite_personal_msg   = RwSignal::new(String::new());

    // Invite form — Step 2: Access
    let invite_platform_role  = RwSignal::new("Admin".to_string());
    let invite_app_role       = RwSignal::new(String::new());
    let invite_tenant         = RwSignal::new(String::new());
    let invite_app_instance   = RwSignal::new(String::new()); // Uuid as string
    let invite_target_url     = RwSignal::new(String::new());
    let invite_expires_days   = RwSignal::new(7i64);
    let invite_sending        = RwSignal::new(false);
    let invite_sent_to        = RwSignal::new(None::<String>); // email of last sent

    // Live invitations resource
    let invites_refetch = RwSignal::new(0);
    let invites_res = LocalResource::new(move || {
        let _ = invites_refetch.get();
        async move {
            get_invites().await.unwrap_or_default()
        }
    });

    // Resource hooks for actual users
    let users_res = LocalResource::new(move || {
        let n = active_network.get();
        let _ = refetch_trigger.get();
        async move {
            get_users(n).await.unwrap_or_default()
        }
    });

    // Audit log resource (loaded lazily when audit tab is active)
    let audit_logs_res = LocalResource::new(|| async move {
        get_audit_logs().await.unwrap_or_default()
    });

    // Tenant list for the invite modal dropdown
    let tenants_for_invite = LocalResource::new(|| async move {
        get_tenant_stats().await.unwrap_or_default()
    });

    // Impersonate
    let handle_impersonate = move |id: Uuid| {
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match impersonate_user(id).await {
                Ok(_) => {
                    t_toast.show_toast("Success", "Impersonation session initiated. Redirecting...", "success");
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
                    t_toast.show_toast("Success", &format!("Updated admin status for {}", updated.email), "success");
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
        let display_name   = invite_display_name.get();
        let personal_msg   = invite_personal_msg.get();
        let platform_role  = invite_platform_role.get();
        let app_role       = invite_app_role.get();
        let tenant         = invite_tenant.get();
        let app_instance   = invite_app_instance.get();
        let target_url     = invite_target_url.get();
        let expires_days   = invite_expires_days.get();
        let t_toast        = toast.clone();

        let input = CreateInviteInput {
            email:           email.clone(),
            display_name:    if display_name.trim().is_empty() { None } else { Some(display_name.trim().to_string()) },
            role:            platform_role,
            app_role:        if app_role.trim().is_empty() { None } else { Some(app_role.trim().to_string()) },
            tenant,
            app_instance_id: uuid::Uuid::parse_str(&app_instance).ok(),
            target_app_url:  if target_url.trim().is_empty() { None } else { Some(target_url.trim().to_string()) },
            personal_message:if personal_msg.trim().is_empty() { None } else { Some(personal_msg.trim().to_string()) },
            expires_days:    Some(expires_days),
        };

        leptos::task::spawn_local(async move {
            match create_invite(input).await {
                Ok(_) => {
                    t_toast.show_toast("Invitation sent", &format!("Magic link dispatched to {}", email), "success");
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
                    t_toast.show_toast("Error", &format!("Failed to send invitation: {}", e), "error");
                }
            }
            invite_sending.set(false);
        });
    };


    // Revoke Invite helper
    let handle_revoke_invite = move |id: Uuid, email: String| {
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match revoke_invite(id).await {
                Ok(_) => {
                    t_toast.show_toast("Warning", &format!("Invitation to {} revoked.", email), "warn");
                    invites_refetch.update(|v| *v += 1);
                }
                Err(e) => {
                    t_toast.show_toast("Error", &format!("Failed to revoke invitation: {}", e), "error");
                }
            }
        });
    };

    // Resend Invite helper
    let handle_resend_invite = move |id: Uuid, email: String| {
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match resend_invite(id).await {
                Ok(_) => {
                    t_toast.show_toast("Success", &format!("Invitation resent to {}", email), "success");
                    invites_refetch.update(|v| *v += 1);
                }
                Err(e) => {
                    t_toast.show_toast("Error", &format!("Failed to resend invitation: {}", e), "error");
                }
            }
        });
    };

    view! {
        <div class="main-canvas">
        // Page Header
        <div class="page-header">
            <div>
                <div class="page-title">"Users & Access Control"</div>
                <div class="page-subtitle">"Platform users · Roles · Invitations · Audit log"</div>
            </div>
            <div class="page-actions">
            <button class="btn btn-primary btn-sm" on:click=move |_| {
                invite_email.set(String::new());
                show_invite_modal.set(true);
            }>"+ Invite User"</button>
            </div>
        </div>

        // KPI Row
        <div class="kpi-row">
            <div class="kpi-card">
                <span class="kpi-label">"Total Users"</span>
                <span class="kpi-value">
                    {move || users_res.get().map(|v| v.len().to_string()).unwrap_or_else(|| "—".to_string())}
                </span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Admins"</span>
                <span class="kpi-value" style="color:var(--cobalt)">
                    {move || users_res.get().map(|v| v.iter().filter(|u| u.is_admin).count().to_string()).unwrap_or_else(|| "—".to_string())}
                </span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Pending Invites"</span>
                <span class="kpi-value" style="color:var(--amber)">
                    {move || invites_res.get().map(|list| list.len().to_string()).unwrap_or_else(|| "0".to_string())}
                </span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Audit Events"</span>
                <span class="kpi-value">
                    {move || audit_logs_res.get().map(|v| v.len().to_string()).unwrap_or_else(|| "—".to_string())}
                </span>
            </div>
        </div>

        // Tab Bar
        <div class="tab-bar">
            <button 
                class=move || if active_tab.get() == "users" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("users".to_string())
            >
                "All Users"
            </button>
            <button 
                class=move || if active_tab.get() == "invites" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("invites".to_string())
            >
                "Pending Invites "
                <span style="font-size:10px;color:var(--amber)">
                    {move || format!("  {}", invites_res.get().map(|list| list.len()).unwrap_or(0))}
                </span>
            </button>
            <button 
                class=move || if active_tab.get() == "roles" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("roles".to_string())
            >
                "Roles & Permissions"
            </button>
            <button 
                class=move || if active_tab.get() == "audit" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("audit".to_string())
            >
                "Audit Log"
            </button>
        </div>

        // Body Content
        <div class="body">
            // ── Tab: All Users ──
            <Show when=move || active_tab.get() == "users">
                <div class="card">
                    <div class="filter-bar">
                        <input 
                            class="f-input" 
                            placeholder="Search name or email…"
                            prop:value=search_query
                            on:input=move |ev| search_query.set(event_target_value(&ev))
                        />
                        <select 
                            class="f-select"
                            on:change=move |ev| role_filter.set(event_target_value(&ev))
                        >
                            <option value="All Roles">"All Roles"</option>
                            <option value="Super-Admin">"Super-Admin"</option>
                            <option value="Admin">"Admin"</option>
                            <option value="Editor">"Editor"</option>
                            <option value="Viewer">"Viewer"</option>
                        </select>
                        <select 
                            class="f-select"
                            on:change=move |ev| tenant_filter.set(event_target_value(&ev))
                        >
                            <option value="All Tenants">"All Tenants"</option>
                            <option value="Platform-wide">"Platform-wide"</option>
                            {move || tenants_for_invite.get().unwrap_or_default().into_iter().map(|t| {
                                let n = t.name.clone();
                                view! { <option value=n.clone()>{n.clone()}</option> }
                            }).collect_view()}
                        </select>
                        <select 
                            class="f-select"
                            on:change=move |ev| status_filter.set(event_target_value(&ev))
                        >
                            <option value="All Statuses">"All Statuses"</option>
                            <option value="Active">"Active"</option>
                            <option value="Inactive">"Inactive"</option>
                            <option value="Suspended">"Suspended"</option>
                        </select>
                    </div>

                    <table class="w-full text-sm">
                        <thead>
                            <tr class="border-b border-outline-variant/20">
                                <th class="px-4 py-2.5 text-left text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60 w-10">""</th>
                                <th class="px-4 py-2.5 text-left text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Name"</th>
                                <th class="px-4 py-2.5 text-left text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Email"</th>
                                <th class="px-4 py-2.5 text-left text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Role"</th>
                                <th class="px-4 py-2.5 text-left text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Scope"</th>
                                <th class="px-4 py-2.5 text-left text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Status"</th>
                                <th class="px-4 py-2.5 text-left text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Last Login"</th>
                                <th class="px-4 py-2.5 text-left text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60 col-hide-mobile">"MFA"</th>
                                <th class="px-4 py-2.5 w-20">""</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-outline-variant/10">
                            // Real DB users only
                            {move || {
                                let users = users_res.get().unwrap_or_default();
                                if users.is_empty() {
                                    view! {
                                        <tr>
                                            <td colspan="9" class="px-4 py-12 text-center text-xs text-on-surface-variant/50">
                                                <span class="material-symbols-outlined block text-3xl mb-2 opacity-30">"group"</span>
                                                "No users found. Invite someone to get started."
                                            </td>
                                        </tr>
                                    }.into_any()
                                } else {
                                    users.into_iter().map(|u| {
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
                                        let (role_bg, role_text) = if is_super {
                                            ("bg-primary/10 border-primary/30", "text-primary")
                                        } else {
                                            ("bg-violet-500/10 border-violet-500/30", "text-violet-400")
                                        };
                                        let scope_str = if u.is_admin { "Platform-wide" } else { "Tenant" };
                                        let (status_bg, status_dot, status_label) = if u.is_active {
                                            ("bg-emerald-500/10 border-emerald-500/20", "bg-emerald-400", "Active")
                                        } else {
                                            ("bg-outline-variant/10 border-outline-variant/20", "bg-on-surface-variant/30", "Inactive")
                                        };
                                        view! {
                                            <tr class="hover:bg-surface-bright/5 transition-colors group">
                                                <td class="px-4 py-3">
                                                    <div class="w-7 h-7 rounded-full bg-primary/10 border border-primary/20 flex items-center justify-center text-[10px] font-bold text-primary shrink-0">
                                                        {initials}
                                                    </div>
                                                </td>
                                                <td class="px-4 py-3 font-semibold text-on-surface text-xs">{u.username.clone()}</td>
                                                <td class="px-4 py-3 text-xs text-on-surface-variant/70 font-mono">{u.email.clone()}</td>
                                                <td class="px-4 py-3">
                                                    <span class=format!("inline-flex items-center px-2 py-0.5 rounded-full text-[9px] font-bold uppercase border {} {}", role_bg, role_text)>
                                                        {role_str}
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3 text-xs text-on-surface-variant/70">{scope_str}</td>
                                                <td class="px-4 py-3">
                                                    <span class=format!("inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase border {}", status_bg)>
                                                        <span class=format!("w-1.5 h-1.5 rounded-full {}", status_dot)></span>
                                                        <span class="text-on-surface-variant">{status_label}</span>
                                                    </span>
                                                </td>
                                                <td class="px-4 py-3 text-xs text-on-surface-variant/50 col-hide-mobile">"—"</td>
                                                <td class="px-4 py-3 text-xs text-on-surface-variant/50 col-hide-mobile">"—"</td>
                                                <td class="px-4 py-3 text-right opacity-0 group-hover:opacity-100 transition-opacity">
                                                    <button
                                                        class="px-2.5 py-1 text-[10px] font-semibold bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 hover:border-primary/30 rounded-lg transition-all text-on-surface-variant"
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
                                    }).collect_view().into_any()
                                }
                            }}
                        </tbody>
                    </table>
                </div>
            </Show>

            // ── Tab: Pending Invites ──
            <Show when=move || active_tab.get() == "invites">
                <div class="card">
                    <div class="card-hdr">
                        <span class="card-title">"Pending Invitations"</span>
                        <button class="btn btn-primary btn-sm" on:click=move |_| show_invite_modal.set(true)>"+ Invite User"</button>
                    </div>
                    <table>
                        <thead>
                            <tr>
                                <th>"Email"</th>
                                <th>"Role"</th>
                                <th>"Tenant Scope"</th>
                                <th>"Invited By"</th>
                                <th>"Sent"</th>
                                <th>"Expires"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || invites_res.get().map(|invites| view! {
                                <For 
                                    each=move || invites.clone()
                                    key=|i: &InviteModel| i.id.clone()
                                    children=move |invite| {
                                        let invite_id = invite.id;
                                        let invite_id2 = invite.id;
                                        let email_clone = invite.email.clone();
                                        let email_clone2 = invite.email.clone();
                                        view! {
                                            <tr>
                                                <td class="cobalt">{invite.email}</td>
                                                <td><span class="pill" style="color:var(--violet);border-color:var(--violet)">{invite.role}</span></td>
                                                <td class="muted">{invite.tenant}</td>
                                                <td class="muted">{invite.invited_by}</td>
                                                <td class="muted">{invite.sent}</td>
                                                <td class="muted amber">{invite.expires}</td>
                                                <td style="display:flex;gap:4px;">
                                                    <button 
                                                        class="btn btn-ghost btn-sm"
                                                        on:click=move |_| handle_resend_invite(invite_id, email_clone.clone())
                                                    >
                                                        "Resend"
                                                    </button>
                                                    <button 
                                                        class="btn btn-ghost btn-sm"
                                                        on:click=move |_| handle_revoke_invite(invite_id2, email_clone2.clone())
                                                    >
                                                        "Revoke"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            })}
                        </tbody>
                    </table>
                </div>
            </Show>

            // ── Tab: Roles & Permissions ──
            <Show when=move || active_tab.get() == "roles">
                <div class="two-col">
                    <div class="card">
                        <div class="card-hdr"><span class="card-title">"Platform Roles"</span></div>
                        <div class="role-row">
                            <span class="role-name" style="color:var(--cobalt)">"Super-Admin"</span>
                            <span class="role-desc">"Full platform access: all tenants, billing, commission plans, user management, impersonation."</span>
                            <span class="role-count">"3"</span>
                        </div>
                        <div class="role-row">
                            <span class="role-name" style="color:var(--violet)">"Admin"</span>
                            <span class="role-desc">"Manage their tenant's users, app config, billing. Cannot access other tenants or platform settings."</span>
                            <span class="role-count">"8"</span>
                        </div>
                        <div class="role-row">
                            <span class="role-name" style="color:var(--amber)">"Editor"</span>
                            <span class="role-desc">"Read + write access to CRM entities, reservations, assets. Cannot manage users or billing."</span>
                            <span class="role-count">"12"</span>
                        </div>
                        <div class="role-row">
                            <span class="role-name" style="color:var(--text-muted)">"Viewer"</span>
                            <span class="role-desc">"Read-only access to all records in their tenant scope."</span>
                            <span class="role-count">"8"</span>
                        </div>
                    </div>
                    <div class="card">
                        <div class="card-hdr"><span class="card-title">"Permission Matrix"</span></div>
                        <div class="stat-row"><span class="s-label">"Manage platform users"</span><span class="s-value" style="color:var(--cobalt)">"Super-Admin only"</span></div>
                        <div class="stat-row"><span class="s-label">"Impersonate tenants"</span><span class="s-value" style="color:var(--cobalt)">"Super-Admin only"</span></div>
                        <div class="stat-row"><span class="s-label">"View billing & ledger"</span><span class="s-value">"Super-Admin, Admin"</span></div>
                        <div class="stat-row"><span class="s-label">"Manage commission plans"</span><span class="s-value" style="color:var(--cobalt)">"Super-Admin only"</span></div>
                        <div class="stat-row"><span class="s-label">"Create/edit CRM records"</span><span class="s-value">"Admin, Editor"</span></div>
                        <div class="stat-row"><span class="s-label">"Configure app instance"</span><span class="s-value">"Admin+"</span></div>
                        <div class="stat-row"><span class="s-label">"View records"</span><span class="s-value">"All roles"</span></div>
                        <div class="stat-row"><span class="s-label">"Export data"</span><span class="s-value">"Admin+"</span></div>
                        <div class="stat-row"><span class="s-label">"Manage API keys"</span><span class="s-value" style="color:var(--cobalt)">"Super-Admin only"</span></div>
                    </div>
                </div>
            </Show>

            // ── Tab: Audit Log ──
            <Show when=move || active_tab.get() == "audit">
                <div class="card">
                    <div class="card-hdr">
                        <span class="card-title">"Audit Log · All Actions"</span>
                        <button class="btn btn-ghost btn-sm" on:click=move |_| toast.show_toast("Info", "Audit CSV export triggered.", "info")>"Export CSV"</button>
                    </div>
                    <Suspense fallback=move || view! { <div style="padding:24px;text-align:center;color:var(--text-muted);font-size:12px;">"Loading audit log..."</div> }>
                    <table>
                        <thead>
                            <tr>
                                <th>"Time"</th>
                                <th>"Actor ID"</th>
                                <th>"Action"</th>
                                <th>"Entity Type"</th>
                                <th>"Entity ID"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let logs = audit_logs_res.get().unwrap_or_default();
                                if logs.is_empty() {
                                    view! {
                                        <tr>
                                            <td colspan="5" style="text-align:center;padding:32px;color:var(--text-muted);font-size:12px;">
                                                "No audit events recorded yet."
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
        </div>

        // ── Modal: Invite User (2-step) ───────────────────────────────────────
        <Show when=move || show_invite_modal.get()>
            <div class="fixed inset-0 z-[100] bg-black/85 backdrop-blur-md flex items-center justify-center p-4">
                <div class="bg-[#0f1117] w-full max-w-xl rounded-2xl border border-white/10 shadow-2xl overflow-hidden relative">

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
                                <div class="w-8 h-8 rounded-full bg-indigo-600 flex items-center justify-center text-xs font-bold text-white">
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
                        <div class="px-8 py-10 text-center">
                            <div class="text-5xl mb-4">"🎉"</div>
                            <h4 class="text-lg font-bold text-white mb-2">"Invitation sent!"</h4>
                            <p class="text-sm text-slate-400 mb-1">
                                "A magic link has been dispatched to"
                            </p>
                            <p class="text-sm font-semibold text-indigo-400 mb-6">
                                {move || invite_sent_to.get().unwrap_or_default()}
                            </p>
                            <p class="text-xs text-slate-500 mb-6">
                                "The link will prompt them to set up a passkey (Touch ID / Face ID) to secure their account."
                            </p>
                            <div class="flex justify-center gap-3">
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


        // Modal: Manage User Dialog
        <Show when=move || show_manage_modal.get().is_some()>
            {let user = show_manage_modal.get().unwrap();
             let u_id = user.id.clone();
             let _u_email = user.email.clone();
             let is_admin = user.is_admin;
             let is_real = user.id != Uuid::nil();
             view! {
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-[#111520] w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_manage_modal.set(None)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Manage User Accounts"</h3>
                        <p class="text-xs text-secondary mt-1">{user.username} " · " {user.email}</p>
                        
                        <div class="space-y-3 mt-4">
                            <Show when=move || is_real>
                                <button 
                                    on:click=move |_| handle_toggle_admin(u_id.clone())
                                    class="btn btn-ghost w-full justify-center"
                                >
                                    {if is_admin { "Revoke Super-Admin privilege" } else { "Grant Super-Admin privilege" }}
                                </button>
                                <button 
                                    on:click=move |_| handle_impersonate(u_id.clone())
                                    class="btn btn-primary w-full justify-center"
                                >
                                    "Impersonate User Session"
                                </button>
                            </Show>
                            <Show when=move || !is_real>
                                <div class="p-3 bg-white/5 border border-white/10 rounded-xl text-center">
                                    <p class="text-xs text-secondary">"Impersonation and privilege overrides are disabled for mock users."</p>
                                </div>
                            </Show>
                            <div class="flex justify-end gap-3 pt-4 border-t border-white/5">
                                <button on:click=move |_| show_manage_modal.set(None) class="btn btn-ghost">"Close"</button>
                            </div>
                        </div>
                    </div>
                </div>
            }}
        </Show>
        </div> // end main-canvas
    }
}
