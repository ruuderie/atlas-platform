use leptos::prelude::*;
use uuid::Uuid;
use crate::api::admin::{get_users, toggle_admin, UserModel, impersonate_user};
use crate::app::GlobalToast;

#[derive(Clone, Debug, PartialEq)]
pub struct MockInvite {
    pub email: String,
    pub role: String,
    pub tenant: String,
    pub invited_by: String,
    pub sent: String,
    pub expires: String,
}

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
    let invite_tenant = RwSignal::new("Nexus PM Group".to_string());

    // Mock invites
    let mock_invites = RwSignal::new(vec![
        MockInvite {
            email: "jill@newclient.com".to_string(),
            role: "Admin".to_string(),
            tenant: "Folio PM (new tenant)".to_string(),
            invited_by: "Jamie Delaney".to_string(),
            sent: "Jun 8".to_string(),
            expires: "Jun 15".to_string(),
        },
        MockInvite {
            email: "pedro@rioverde.br".to_string(),
            role: "Editor".to_string(),
            tenant: "Rio Verde PMC".to_string(),
            invited_by: "Maria Fernandes".to_string(),
            sent: "Jun 7".to_string(),
            expires: "Jun 14".to_string(),
        },
    ]);

    // Resource hooks for actual users
    let users_res = LocalResource::new(move || {
        let n = active_network.get();
        let _ = refetch_trigger.get();
        async move {
            get_users(n).await.unwrap_or_default()
        }
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

        mock_invites.update(|list| {
            list.push(MockInvite {
                email: email.clone(),
                role,
                tenant,
                invited_by: "Jamie Delaney".to_string(),
                sent: "Jun 14".to_string(),
                expires: "Jun 21".to_string(),
            });
        });

        invite_email.set(String::new());
        show_invite_modal.set(false);
        toast.show_toast("Success", &format!("Invitation sent to {}", email), "success");
    };

    // Revoke Invite helper
    let handle_revoke_invite = move |email: String| {
        mock_invites.update(|list| {
            list.retain(|i| i.email != email);
        });
        toast.show_toast("Warning", &format!("Invitation to {} revoked.", email), "warn");
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
                    {move || {
                        let db_count = users_res.get().map(|v| v.len()).unwrap_or(0);
                        (db_count + 5).to_string()
                    }}
                </div>
                <div class="kpi-sub">"Across all tenants"</div>
            </div>
            <div class="kpi">
                <div class="kpi-label">"Super-Admins"</div>
                <div class="kpi-value" style="color:var(--cobalt)">
                    {move || {
                        let db_count = users_res.get().map(|v| v.iter().filter(|u| u.is_admin).count()).unwrap_or(0);
                        (db_count + 2).to_string()
                    }}
                </div>
                <div class="kpi-sub">"Platform-wide access"</div>
            </div>
            <div class="kpi">
                <div class="kpi-label">"Tenant Admins"</div>
                <div class="kpi-value">"8"</div>
                <div class="kpi-sub">"Manage their tenant"</div>
            </div>
            <div class="kpi">
                <div class="kpi-label">"Pending Invites"</div>
                <div class="kpi-value" style="color:var(--amber)">
                    {move || mock_invites.get().len().to_string()}
                </div>
                <div class="kpi-sub">"Awaiting acceptance"</div>
            </div>
            <div class="kpi">
                <div class="kpi-label">"Last 30 Days Logins"</div>
                <div class="kpi-value mono">"218"</div>
                <div class="kpi-sub">"Across all users"</div>
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
                    {move || format!("  {}", mock_invites.get().len())}
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
                            <option value="Nexus PM Group">"Nexus PM Group"</option>
                            <option value="Biscayne STR Co.">"Biscayne STR Co."</option>
                            <option value="Harbor Media">"Harbor Media"</option>
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
                            // Jamie Delaney (Self)
                            <tr on:click=move |_| {
                                toast.show_toast("Info", "You cannot modify your own profile settings.", "info");
                            }>
                                <td><div class="user-avatar" style="background:var(--cobalt-dim);color:var(--cobalt)">"JD"</div></td>
                                <td style="font-weight:500">
                                    "Jamie Delaney"
                                    <span style="font-size:9px;color:var(--cobalt);border:1px solid var(--cobalt);border-radius:3px;padding:1px 4px;margin-left:4px">"YOU"</span>
                                </td>
                                <td class="muted">"jamie@atlasplatform.io"</td>
                                <td><span class="pill" style="color:var(--cobalt);border-color:var(--cobalt)">"Super-Admin"</span></td>
                                <td class="muted">"Platform-wide"</td>
                                <td class="green">"Active"</td>
                                <td class="muted">"Now"</td>
                                <td class="green">"✓ TOTP"</td>
                                <td>
                                    <button 
                                        class="btn btn-ghost btn-sm" 
                                        on:click=move |e| {
                                            e.stop_propagation();
                                            toast.show_toast("Info", "You cannot modify your own profile settings.", "info");
                                        }
                                    >
                                        "Manage"
                                    </button>
                                </td>
                            </tr>

                            // Maria Fernandes
                            <tr>
                                <td><div class="user-avatar" style="background:var(--green-dim);color:var(--green)">"MF"</div></td>
                                <td style="font-weight:500">"Maria Fernandes"</td>
                                <td class="muted">"maria@atlasplatform.io"</td>
                                <td><span class="pill" style="color:var(--cobalt);border-color:var(--cobalt)">"Super-Admin"</span></td>
                                <td class="muted">"Platform-wide"</td>
                                <td class="green">"Active"</td>
                                <td class="muted">"2h ago"</td>
                                <td class="green">"✓ TOTP"</td>
                                <td>
                                    <button 
                                        class="btn btn-ghost btn-sm"
                                        on:click=move |e| {
                                            e.stop_propagation();
                                            let mut dummy = UserModel {
                                                id: Uuid::nil(),
                                                username: "Maria Fernandes".to_string(),
                                                email: "maria@atlasplatform.io".to_string(),
                                                is_active: true,
                                                is_admin: true,
                                            };
                                            show_manage_modal.set(Some(dummy));
                                        }
                                    >
                                        "Manage"
                                    </button>
                                </td>
                            </tr>

                            // Render Database Users dynamically
                            {move || users_res.get().map(|users| view! {
                                <For 
                                    each=move || users.clone()
                                    key=|u: &UserModel| u.id.clone()
                                    children=move |u| {
                                        let u_clone = u.clone();
                                        let u_clone2 = u.clone();
                                        let initials = u.username.chars().take(2).collect::<String>().to_uppercase();
                                        let initial_display = if initials.is_empty() { "U".to_string() } else { initials };
                                        let role_str = if u.is_admin { "Super-Admin" } else { "Admin" };
                                        let scope_str = if u.is_admin { "Platform-wide" } else { "Tenant Scope" };
                                        
                                        view! {
                                            <tr>
                                                <td><div class="user-avatar" style="background:var(--bg-elevated);color:var(--text-secondary)">{initial_display}</div></td>
                                                <td style="font-weight:500">{u.username.clone()}</td>
                                                <td class="muted">{u.email.clone()}</td>
                                                <td><span class="pill" style="color:var(--violet);border-color:var(--violet)">{role_str}</span></td>
                                                <td class="muted">{scope_str}</td>
                                                <td class=if u.is_active { "green" } else { "muted" }>{if u.is_active { "Active" } else { "Inactive" }}</td>
                                                <td class="muted">"Yesterday"</td>
                                                <td class="green">"✓ TOTP"</td>
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
                                    }
                                />
                            })}

                            // Carlos Mendes
                            <tr>
                                <td><div class="user-avatar" style="background:var(--violet-dim);color:var(--violet)">"CM"</div></td>
                                <td style="font-weight:500">"Carlos Mendes"</td>
                                <td class="muted">"carlos@nexus.com"</td>
                                <td><span class="pill" style="color:var(--violet);border-color:var(--violet)">"Admin"</span></td>
                                <td class="muted">"Nexus PM Group"</td>
                                <td class="green">"Active"</td>
                                <td class="muted">"Yesterday"</td>
                                <td class="green">"✓ TOTP"</td>
                                <td>
                                    <button 
                                        class="btn btn-ghost btn-sm"
                                        on:click=move |e| {
                                            e.stop_propagation();
                                            let mut dummy = UserModel {
                                                id: Uuid::nil(),
                                                username: "Carlos Mendes".to_string(),
                                                email: "carlos@nexus.com".to_string(),
                                                is_active: true,
                                                is_admin: false,
                                            };
                                            show_manage_modal.set(Some(dummy));
                                        }
                                    >
                                        "Manage"
                                    </button>
                                </td>
                            </tr>

                            // Rafael Pinto
                            <tr>
                                <td><div class="user-avatar" style="background:var(--amber-dim);color:var(--amber)">"RP"</div></td>
                                <td style="font-weight:500">"Rafael Pinto"</td>
                                <td class="muted">"rafael@biscayne.com"</td>
                                <td><span class="pill" style="color:var(--amber);border-color:var(--amber)">"Editor"</span></td>
                                <td class="muted">"Biscayne STR Co."</td>
                                <td class="green">"Active"</td>
                                <td class="muted">"3 days ago"</td>
                                <td class="amber">"⚠ None"</td>
                                <td>
                                    <button 
                                        class="btn btn-ghost btn-sm"
                                        on:click=move |e| {
                                            e.stop_propagation();
                                            let mut dummy = UserModel {
                                                id: Uuid::nil(),
                                                username: "Rafael Pinto".to_string(),
                                                email: "rafael@biscayne.com".to_string(),
                                                is_active: true,
                                                is_admin: false,
                                            };
                                            show_manage_modal.set(Some(dummy));
                                        }
                                    >
                                        "Manage"
                                    </button>
                                </td>
                            </tr>

                            // Ana Silva
                            <tr>
                                <td><div class="user-avatar" style="background:var(--bg-elevated);color:var(--text-muted)">"AS"</div></td>
                                <td style="font-weight:500;color:var(--text-muted)">"Ana Silva"</td>
                                <td class="muted">"ana@harbor.io"</td>
                                <td><span class="pill" style="color:var(--text-muted);border-color:var(--border-default)">"Viewer"</span></td>
                                <td class="muted">"Harbor Media"</td>
                                <td class="muted">"Inactive"</td>
                                <td class="muted">"14 days ago"</td>
                                <td class="muted">"—"</td>
                                <td>
                                    <button 
                                        class="btn btn-ghost btn-sm"
                                        on:click=move |e| {
                                            e.stop_propagation();
                                            let mut dummy = UserModel {
                                                id: Uuid::nil(),
                                                username: "Ana Silva".to_string(),
                                                email: "ana@harbor.io".to_string(),
                                                is_active: false,
                                                is_admin: false,
                                            };
                                            show_manage_modal.set(Some(dummy));
                                        }
                                    >
                                        "Manage"
                                    </button>
                                </td>
                            </tr>

                            // Bob Keller
                            <tr>
                                <td><div class="user-avatar" style="background:var(--red-dim);color:var(--red)">"BK"</div></td>
                                <td style="font-weight:500;color:var(--red)">"Bob Keller"</td>
                                <td class="muted">"bob@southbeach.io"</td>
                                <td><span class="pill" style="color:var(--text-muted);border-color:var(--border-default)">"Viewer"</span></td>
                                <td class="muted">"South Beach Nets"</td>
                                <td class="red">"Suspended"</td>
                                <td class="muted">"—"</td>
                                <td class="muted">"—"</td>
                                <td>
                                    <button 
                                        class="btn btn-ghost btn-sm"
                                        on:click=move |e| {
                                            e.stop_propagation();
                                            let mut dummy = UserModel {
                                                id: Uuid::nil(),
                                                username: "Bob Keller".to_string(),
                                                email: "bob@southbeach.io".to_string(),
                                                is_active: false,
                                                is_admin: false,
                                            };
                                            show_manage_modal.set(Some(dummy));
                                        }
                                    >
                                        "Manage"
                                    </button>
                                </td>
                            </tr>
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
                            <For 
                                each=move || mock_invites.get()
                                key=|i| i.email.clone()
                                children=move |invite| {
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
                                                    on:click=move |_| {
                                                        toast.show_toast("Success", &format!("Invitation resent to {}", email_clone), "success");
                                                    }
                                                >
                                                    "Resend"
                                                </button>
                                                <button 
                                                    class="btn btn-ghost btn-sm"
                                                    on:click=move |_| handle_revoke_invite(email_clone2.clone())
                                                >
                                                    "Revoke"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
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
                        <button class="btn btn-ghost btn-sm" on:click=move |_| toast.show_toast("Success", "Audit CSV Export triggered.", "success")>"Export CSV"</button>
                    </div>
                    <table>
                        <thead>
                            <tr>
                                <th>"Time"</th>
                                <th>"User"</th>
                                <th>"Action"</th>
                                <th>"Entity"</th>
                                <th>"IP Address"</th>
                                <th>"Impersonated?"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td class="muted">"Jun 10 · 01:07"</td>
                                <td>"Jamie Delaney"</td>
                                <td class="cobalt">"login"</td>
                                <td class="muted">"session"</td>
                                <td class="muted">"10.0.1.2"</td>
                                <td class="muted">"—"</td>
                            </tr>
                            <tr>
                                <td class="muted">"Jun 10 · 00:58"</td>
                                <td>"Maria Fernandes"</td>
                                <td>"lead.convert"</td>
                                <td class="muted">"atlas_lead · le_8f2c…"</td>
                                <td class="muted">"10.0.1.8"</td>
                                <td class="muted">"—"</td>
                            </tr>
                            <tr>
                                <td class="muted">"Jun 9 · 23:41"</td>
                                <td>"Jamie Delaney"</td>
                                <td style="color:var(--amber)">"impersonate.start"</td>
                                <td class="muted">"Nexus PM Group"</td>
                                <td class="muted">"10.0.1.2"</td>
                                <td class="amber">"YES"</td>
                            </tr>
                            <tr>
                                <td class="muted">"Jun 9 · 23:40"</td>
                                <td>"Jamie Delaney"</td>
                                <td>"tenant.suspend"</td>
                                <td class="muted">"South Beach Nets"</td>
                                <td class="muted">"10.0.1.2"</td>
                                <td class="muted">"—"</td>
                            </tr>
                            <tr>
                                <td class="muted">"Jun 9 · 22:15"</td>
                                <td>"Carlos Mendes"</td>
                                <td>"ledger.entry.create"</td>
                                <td class="muted">"le_h9k2…"</td>
                                <td class="muted">"10.0.2.4"</td>
                                <td class="muted">"—"</td>
                            </tr>
                            <tr>
                                <td class="muted">"Jun 9 · 20:00"</td>
                                <td>"Rafael Pinto"</td>
                                <td>"contact.update"</td>
                                <td class="muted">"atlas_contact · ct_4a2…"</td>
                                <td class="muted">"10.0.3.7"</td>
                                <td class="muted">"—"</td>
                            </tr>
                        </tbody>
                    </table>
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
                                    <option value="Nexus PM Group">"Nexus PM Group"</option>
                                    <option value="Biscayne STR Co.">"Biscayne STR Co."</option>
                                    <option value="Harbor Media">"Harbor Media"</option>
                                    <option value="Platform-wide">"Platform-wide (Super)"</option>
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
