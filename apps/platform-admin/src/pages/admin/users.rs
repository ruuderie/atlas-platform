use leptos::prelude::*;
use uuid::Uuid;
use crate::api::admin::{
    get_users, toggle_admin, UserModel, impersonate_user,
    get_invites, create_invite, revoke_invite, resend_invite, InviteModel,
    get_audit_logs, AuditLogModel, get_tenant_stats,
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
    let show_invite_modal = RwSignal::new(false);
    let show_manage_modal = RwSignal::new(None::<UserModel>);
    let invite_email = RwSignal::new(String::new());
    let invite_role = RwSignal::new("Admin".to_string());
    let invite_tenant = RwSignal::new(String::new());

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

        let role = invite_role.get();
        let tenant = invite_tenant.get();
        let t_toast = toast.clone();

        leptos::task::spawn_local(async move {
            match create_invite(email.clone(), role, tenant).await {
                Ok(_) => {
                    t_toast.show_toast("Success", &format!("Invitation sent to {}", email), "success");
                    invites_refetch.update(|v| *v += 1);
                }
                Err(e) => {
                    t_toast.show_toast("Error", &format!("Failed to send invitation: {}", e), "error");
                }
            }
        });

        invite_email.set(String::new());
        show_invite_modal.set(false);
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
        // Page Header
        <div class="page-hdr">
            <div>
                <div class="page-title">"Users & Access Control"</div>
                <div class="page-sub">"Platform users · Roles · Invitations · Audit log"</div>
            </div>
            <button class="btn btn-primary btn-sm" on:click=move |_| {
                invite_email.set(String::new());
                show_invite_modal.set(true);
            }>"+ Invite User"</button>
        </div>

        // KPI Strip
        <div class="kpi-strip">
            <div class="kpi">
                <div class="kpi-label">"Total Users"</div>
                <div class="kpi-value">
                    {move || users_res.get().map(|v| v.len().to_string()).unwrap_or_else(|| "—".to_string())}
                </div>
                <div class="kpi-sub">"Registered in DB"</div>
            </div>
            <div class="kpi">
                <div class="kpi-label">"Admins"</div>
                <div class="kpi-value" style="color:var(--cobalt)">
                    {move || users_res.get().map(|v| v.iter().filter(|u| u.is_admin).count().to_string()).unwrap_or_else(|| "—".to_string())}
                </div>
                <div class="kpi-sub">"Platform-wide access"</div>
            </div>
            <div class="kpi">
                <div class="kpi-label">"Pending Invites"</div>
                <div class="kpi-value" style="color:var(--amber)">
                    {move || invites_res.get().map(|list| list.len().to_string()).unwrap_or_else(|| "0".to_string())}
                </div>
                <div class="kpi-sub">"Awaiting acceptance"</div>
            </div>
            <div class="kpi">
                <div class="kpi-label">"Audit Events"</div>
                <div class="kpi-value mono">
                    {move || audit_logs_res.get().map(|v| v.len().to_string()).unwrap_or_else(|| "—".to_string())}
                </div>
                <div class="kpi-sub">"All recorded actions"</div>
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

                    <table>
                        <thead>
                            <tr>
                                <th>""</th>
                                <th>"Name"</th>
                                <th>"Email"</th>
                                <th>"Role"</th>
                                <th>"Tenant Scope"</th>
                                <th>"Status"</th>
                                <th>"Last Login"</th>
                                <th>"MFA"</th>
                                <th>"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            // Real DB users only
                            {move || {
                                let users = users_res.get().unwrap_or_default();
                                if users.is_empty() {
                                    view! {
                                        <tr>
                                            <td colspan="9" style="text-align:center;padding:32px;color:var(--text-muted);font-size:12px;">
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
                                        let role_str = if u.is_admin { "Super-Admin" } else { "Admin" };
                                        let role_color = if u.is_admin { "color:var(--cobalt);border-color:var(--cobalt)" } else { "color:var(--violet);border-color:var(--violet)" };
                                        let scope_str = if u.is_admin { "Platform-wide" } else { "Tenant" };
                                        view! {
                                            <tr>
                                                <td><div class="user-avatar" style="background:var(--bg-elevated);color:var(--text-secondary)">{initials}</div></td>
                                                <td style="font-weight:500">{u.username.clone()}</td>
                                                <td class="muted">{u.email.clone()}</td>
                                                <td><span class="pill" style=role_color>{role_str}</span></td>
                                                <td class="muted">{scope_str}</td>
                                                <td class=if u.is_active { "green" } else { "muted" }>{if u.is_active { "Active" } else { "Inactive" }}</td>
                                                <td class="muted">"—"</td>
                                                <td class="muted">"—"</td>
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

        // Modal: Invite User
        <Show when=move || show_invite_modal.get()>
            <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                <div class="bg-[#111520] w-full max-w-lg p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                    <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_invite_modal.set(false)>"✕"</button>
                    <h3 class="text-xl font-semibold mb-2">"Invite Platform User"</h3>
                    <div class="space-y-4 mt-4">
                        <div class="n-form-row">
                            <label class="n-form-label">"Email Address"</label>
                            <input 
                                type="email" 
                                class="n-form-input"
                                placeholder="e.g. user@organization.com"
                                prop:value=invite_email
                                on:input=move |ev| invite_email.set(event_target_value(&ev))
                            />
                        </div>
                        <div style="display:grid; grid-template-columns:1fr 1fr; gap:12px">
                            <div class="n-form-row">
                                <label class="n-form-label">"Privilege Level (Role)"</label>
                                <select 
                                    class="n-form-select"
                                    on:change=move |ev| invite_role.set(event_target_value(&ev))
                                >
                                    <option value="Admin">"Admin"</option>
                                    <option value="Super-Admin">"Super-Admin"</option>
                                    <option value="Editor">"Editor"</option>
                                    <option value="Viewer">"Viewer"</option>
                                </select>
                            </div>
                            <div class="n-form-row">
                                <label class="n-form-label">"Tenant Scope"</label>
                                <select
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
                        </div>
                        <div class="flex justify-end gap-3 pt-4 border-t border-white/5">
                            <button on:click=move |_| show_invite_modal.set(false) class="btn btn-ghost">"Cancel"</button>
                            <button on:click=submit_invite_user class="btn btn-primary">"Send Invitation"</button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>

        // Modal: Manage User Dialog
        <Show when=move || show_manage_modal.get().is_some()>
            {let user = show_manage_modal.get().unwrap();
             let u_id = user.id.clone();
             let u_email = user.email.clone();
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
    }
}
