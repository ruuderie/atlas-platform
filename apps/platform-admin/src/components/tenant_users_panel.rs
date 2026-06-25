//! TenantUsersPanel — per-instance user list with invite + role management.
//!
//! Fetches all users scoped to a tenant (via ?tenant_id= query param),
//! shows a table with email, username, admin status, and active state.
//! Allows:
//!   - Toggling platform admin status (toggle_admin)
//!   - Inviting a new user by email + folio role
//!   - Viewing pending invitations for this tenant

use leptos::prelude::*;
use uuid::Uuid;
use crate::api::admin::{
    get_users, toggle_admin, get_invites, create_invite, revoke_invite,
};

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantUsersPanel(
    tenant_id: Uuid,
) -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // ── Resources ──────────────────────────────────────────────────────────────
    let users_res = LocalResource::new(move || {
        async move { get_users(Some(tenant_id)).await.unwrap_or_default() }
    });

    let invites_res = LocalResource::new(move || {
        async move {
            get_invites().await.unwrap_or_default()
                .into_iter()
                .filter(|i| i.tenant == tenant_id.to_string())
                .collect::<Vec<_>>()
        }
    });

    // ── Invite form signals ────────────────────────────────────────────────────
    let show_invite_modal = RwSignal::new(false);
    let invite_email = RwSignal::new(String::new());
    let invite_role  = RwSignal::new("landlord".to_string());
    let invite_saving = RwSignal::new(false);

    let handle_invite = move |_| {
        let t = toast.clone();
        let email = invite_email.get();
        let role  = invite_role.get();
        let tenant = tenant_id.to_string();

        if email.is_empty() { return; }
        invite_saving.set(true);
        leptos::task::spawn_local(async move {
            match create_invite(email, role, tenant).await {
                Ok(_) => {
                    t.show_toast("Invited", "Invitation sent successfully.", "success");
                    show_invite_modal.set(false);
                    invite_email.set(String::new());
                }
                Err(e) => t.show_toast("Error", &format!("Invite failed: {}", e), "error"),
            }
            invite_saving.set(false);
        });
    };

    view! {
        <div class="w-full flex flex-col gap-5">

            // ── Active Users ──────────────────────────────────────────────────
            <div class="w-full bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40 flex items-center justify-between">
                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant flex items-center gap-2">
                        <svg viewBox="0 0 14 14" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5">
                            <circle cx="7" cy="5" r="3"/>
                            <path d="M1 13c0-3.3 2.7-6 6-6s6 2.7 6 6"/>
                        </svg>
                        "Users"
                        <span class="text-[10px] text-on-surface-variant/60 font-normal normal-case tracking-normal">
                            {move || format!("({} users)", users_res.get().unwrap_or_default().len())}
                        </span>
                    </h3>
                    <button
                        class="px-3 py-1.5 rounded-lg text-[11px] font-semibold bg-primary text-on-primary hover:opacity-90 transition-all"
                        on:click=move |_| show_invite_modal.set(true)
                    >"+ Invite User"</button>
                </div>

                <Suspense fallback=move || view! { <div class="p-4 muted">"Loading users…"</div> }>
                <table>
                    <thead>
                        <tr>
                            <th>"Email"</th>
                            <th>"Username"</th>
                            <th class="center">"Active"</th>
                            <th class="center">"Platform Admin"</th>
                            <th class="right">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || users_res.get().unwrap_or_default().into_iter().map(|user| {
                            let user_id = user.id;
                            let t = toast.clone();
                            let email_str = user.email.clone();
                            let username_str = user.username.clone();
                            let is_active = user.is_active;
                            let is_admin = user.is_admin;

                            view! {
                                <tr>
                                    <td>
                                        <div style="font-size:12px;font-weight:500;color:var(--text-primary);">{email_str}</div>
                                    </td>
                                    <td><span class="font-mono text-muted" style="font-size:11px;">{username_str}</span></td>
                                    <td class="center">
                                        <span style=if is_active { "color:var(--green);font-weight:600;font-size:11px;" } else { "color:var(--text-muted);font-size:11px;" }>
                                            {if is_active { "● Active" } else { "○ Inactive" }}
                                        </span>
                                    </td>
                                    <td class="center">
                                        <span style=if is_admin { "color:var(--amber);font-weight:600;font-size:11px;" } else { "color:var(--text-muted);font-size:11px;" }>
                                            {if is_admin { "✓ Admin" } else { "—" }}
                                        </span>
                                    </td>
                                    <td class="right">
                                        <button
                                            class="btn btn-ghost"
                                            style="font-size:11px;padding:3px 8px;"
                                            on:click=move |_| {
                                                let t = t.clone();
                                                leptos::task::spawn_local(async move {
                                                    match toggle_admin(user_id).await {
                                                        Ok(_) => t.show_toast("Updated", "Admin status toggled.", "success"),
                                                        Err(e) => t.show_toast("Error", &format!("Toggle failed: {}", e), "error"),
                                                    }
                                                });
                                            }
                                        >
                                            {if is_admin { "Revoke Admin" } else { "Make Admin" }}
                                        </button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
                </Suspense>
            </div>

            // ── Pending Invitations ─────────────────────────────────────────────
            <div class="w-full bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                <div class="px-5 py-3.5 border-b border-outline-variant/20 bg-surface-container-high/40 flex items-center justify-between">
                    <h3 class="text-xs font-bold uppercase tracking-wider text-on-surface-variant flex items-center gap-2">
                        <svg viewBox="0 0 14 14" width="12" height="12" fill="none" stroke="currentColor" stroke-width="1.5">
                            <path d="M1 4l6 4 6-4"/>
                            <rect x="1" y="3" width="12" height="8" rx="1"/>
                        </svg>
                        "Pending Invitations"
                        <span class="text-[10px] text-on-surface-variant/60 font-normal normal-case tracking-normal">
                            {move || format!("({})", invites_res.get().unwrap_or_default().len())}
                        </span>
                    </h3>
                </div>

                <Suspense fallback=move || view! { <div class="p-4 muted">"Loading invites…"</div> }>
                {move || {
                    let invites = invites_res.get().unwrap_or_default();
                    if invites.is_empty() {
                        view! {
                            <div style="padding:20px;text-align:center;color:var(--text-muted);font-size:12px;">
                                "No pending invitations."
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <table>
                                <thead>
                                    <tr>
                                        <th>"Email"</th>
                                        <th>"Role"</th>
                                        <th>"Sent"</th>
                                        <th>"Expires"</th>
                                        <th class="right">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {invites.into_iter().map(|inv| {
                                        let inv_id = inv.id;
                                        let t = toast.clone();
                                        let email_str = inv.email.clone();
                                        let role_str  = inv.role.clone();
                                        let sent_str  = inv.sent.get(..10).map(|s| s.to_string()).unwrap_or_else(|| inv.sent.clone());
                                        let exp_str   = inv.expires.get(..10).map(|s| s.to_string()).unwrap_or_else(|| inv.expires.clone());
                                        view! {
                                            <tr>
                                                <td style="font-size:12px;">{email_str}</td>
                                                <td><span class="plan-badge">{role_str}</span></td>
                                                <td class="right muted">{sent_str}</td>
                                                <td class="right muted">{exp_str}</td>
                                                <td class="right">
                                                    <button
                                                        class="btn btn-ghost"
                                                        style="font-size:11px;padding:3px 8px;color:var(--red);"
                                                        on:click=move |_| {
                                                            let t = t.clone();
                                                            leptos::task::spawn_local(async move {
                                                                match revoke_invite(inv_id).await {
                                                                    Ok(_) => t.show_toast("Revoked", "Invitation revoked.", "success"),
                                                                    Err(e) => t.show_toast("Error", &format!("Revoke failed: {}", e), "error"),
                                                                }
                                                            });
                                                        }
                                                    >"Revoke"</button>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_any()
                    }
                }}
                </Suspense>
            </div>

            // ── Invite Modal ───────────────────────────────────────────────────
            {move || if show_invite_modal.get() {
                view! {
                    <div class="modal-backdrop" on:click=move |_| show_invite_modal.set(false)>
                        <div class="modal" on:click=|ev| ev.stop_propagation()>
                            <div class="modal-header">
                                <div class="modal-title">"Invite User"</div>
                                <button class="modal-close" on:click=move |_| show_invite_modal.set(false)>"✕"</button>
                            </div>
                            <div class="modal-body" style="display:flex;flex-direction:column;gap:14px;">

                                <div class="form-group">
                                    <label class="form-label">"Email Address"</label>
                                    <input
                                        class="form-input"
                                        type="email"
                                        placeholder="user@example.com"
                                        prop:value=move || invite_email.get()
                                        on:input=move |ev| invite_email.set(event_target_value(&ev))
                                    />
                                </div>

                                <div class="form-group">
                                    <label class="form-label">"Folio Role"</label>
                                    <select
                                        class="form-select"
                                        prop:value=move || invite_role.get()
                                        on:change=move |ev| invite_role.set(event_target_value(&ev))
                                    >
                                        <option value="landlord">"Landlord"</option>
                                        <option value="tenant">"Tenant"</option>
                                        <option value="vendor">"Vendor"</option>
                                        <option value="property_manager">"Property Manager (PMC)"</option>
                                        <option value="owner">"Owner (Read-only)"</option>
                                        <option value="agent">"Agent (Brokerage)"</option>
                                        <option value="broker">"Broker (Brokerage)"</option>
                                    </select>
                                </div>

                            </div>
                            <div class="modal-footer">
                                <button class="btn btn-ghost" on:click=move |_| show_invite_modal.set(false)>"Cancel"</button>
                                <button
                                    class="btn btn-primary"
                                    on:click=handle_invite
                                    disabled=move || invite_saving.get() || invite_email.get().is_empty()
                                >
                                    {move || if invite_saving.get() { "Sending…" } else { "Send Invitation" }}
                                </button>
                            </div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! { <></> }.into_any()
            }}

        </div>
    }
}
