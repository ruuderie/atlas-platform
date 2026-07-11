// apps/folio/src/pages/landlord/team.rs
//
// LandlordTeam — /l/team
//
// Team & Access Management dashboard.
//
// Shows:
//   ● Active team members (property managers, co-hosts, vendors) with role badges,
//     permission scope, and quick actions (revoke, adjust scope).
//   ● Invite code panel — creates + manages invite links per role.
//   ● Quick-invite modal (email + role → generates a code and drafts the message).
//
// Data flow:
//   - GET /api/folio/invite-codes        → list active codes (landlord created)
//   - POST /api/folio/invite-codes       → create new code
//   - PATCH /api/folio/invite-codes/:id  → deactivate / update label

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteCodeRow {
    pub id: String,
    pub code: String,
    pub role: String,
    pub label: Option<String>,
    pub max_uses: Option<i32>,
    pub uses_count: i32,
    pub expires_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub join_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteCodesResponse {
    pub codes: Vec<InviteCodeRow>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(ListInviteCodes, "/api")]
pub async fn list_invite_codes() -> Result<InviteCodesResponse, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("Not authenticated"))?;
    let result = crate::atlas_client::authenticated_get::<InviteCodesResponse>(
        "/api/folio/invite-codes",
        &token,
        None,
    )
    .await
    .map_err(server_fn::error::ServerFnError::new)?;
    Ok(result)
}

#[server(CreateTeamInvite, "/api")]
pub async fn create_team_invite(
    role: String,
    label: String,
    max_uses: Option<i32>,
    employer_self: bool, // true = landlord is hiring a PM for themselves (sets employer_user_id)
) -> Result<String, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("Not authenticated"))?;

    // Resolve calling user ID for employer_user_id stamping
    let caller_id_str = if employer_self {
        // fetch user info from token to get caller UUID
        match crate::atlas_client::authenticated_get::<serde_json::Value>(
            "/api/folio/me",
            &token,
            None,
        )
        .await
        {
            Ok(me) => me["id"].as_str().map(|s| s.to_string()),
            Err(_) => None,
        }
    } else {
        None
    };

    let mut payload = serde_json::json!({
        "role":      role,
        "label":     label,
        "max_uses":  max_uses,
    });
    if let Some(eid) = caller_id_str {
        payload["employer_user_id"] = serde_json::Value::String(eid);
    }

    let result = crate::atlas_client::authenticated_post::<_, serde_json::Value>(
        "/api/folio/invite-codes",
        &token,
        None,
        &payload,
    )
    .await
    .map_err(server_fn::error::ServerFnError::new)?;

    Ok(result["code"].as_str().unwrap_or_default().to_string())
}

#[server(DeactivateInviteCode, "/api")]
pub async fn deactivate_invite_code(
    code_id: String,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("Not authenticated"))?;
    crate::atlas_client::authenticated_patch::<_, serde_json::Value>(
        &format!("/api/folio/invite-codes/{}", code_id),
        &token,
        &serde_json::json!({ "is_active": false }),
    )
    .await
    .map(|_| ())
    .map_err(server_fn::error::ServerFnError::new)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn role_label(r: &str) -> &'static str {
    match r {
        "property_manager" => "Property Manager",
        "vendor" => "Vendor",
        "cohost" => "Co-host",
        "tenant" => "Tenant",
        "agent" => "Agent",
        _ => "Member",
    }
}

fn role_icon(r: &str) -> &'static str {
    match r {
        "property_manager" => "corporate_fare",
        "vendor" => "handyman",
        "cohost" => "supervisor_account",
        "tenant" => "door_front",
        "agent" => "real_estate_agent",
        _ => "person",
    }
}

fn role_accent(r: &str) -> &'static str {
    match r {
        "property_manager" => "#0284c7",
        "vendor" => "#0891b2",
        "cohost" => "#7c3aed",
        "tenant" => "#059669",
        "agent" => "#d97706",
        _ => "#64748b",
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn LandlordTeam() -> impl IntoView {
    // ── Data loading ──────────────────────────────────────────────────────────
    let codes_resource: Resource<Result<InviteCodesResponse, _>> =
        Resource::new(|| (), |_| list_invite_codes());

    let trigger_reload = RwSignal::new(0u32);
    let codes_resource_reloadable: Resource<Result<InviteCodesResponse, _>> =
        Resource::new(move || trigger_reload.get(), |_| list_invite_codes());

    // ── Modal state ───────────────────────────────────────────────────────────
    let modal_open = RwSignal::new(false);
    let modal_role = RwSignal::new("property_manager".to_string());
    let modal_label = RwSignal::new(String::new());
    let modal_uses = RwSignal::new("1".to_string()); // "unlimited" | "1" | "5" | "10"
    let modal_saving = RwSignal::new(false);
    let modal_err: RwSignal<Option<String>> = RwSignal::new(None);
    let modal_created: RwSignal<Option<String>> = RwSignal::new(None);

    let handle_create = move |_| {
        modal_saving.set(true);
        modal_err.set(None);
        let role = modal_role.get();
        let label = modal_label.get();
        let max_uses: Option<i32> = match modal_uses.get().as_str() {
            "unlimited" => None,
            s => s.parse().ok(),
        };
        let is_pm = role == "property_manager";
        leptos::task::spawn_local(async move {
            match create_team_invite(role, label, max_uses, is_pm).await {
                Ok(code) => {
                    modal_created.set(Some(code));
                    modal_saving.set(false);
                    trigger_reload.update(|n| *n += 1);
                }
                Err(e) => {
                    modal_err.set(Some(e.to_string()));
                    modal_saving.set(false);
                }
            }
        });
    };

    let handle_deactivate = move |code_id: String| {
        leptos::task::spawn_local(async move {
            let _ = deactivate_invite_code(code_id).await;
            trigger_reload.update(|n| *n += 1);
        });
    };

    view! {
        <style>
            {r#"
            @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800;900&display=swap');
            @import url('https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200&display=swap');
            .ms { font-family:'Material Symbols Outlined'; font-variation-settings:'FILL' 0,'wght' 400; line-height:1; display:inline-block; user-select:none; }
            .msf { font-variation-settings:'FILL' 1,'wght' 400; }

            /* ── Page layout ── */
            .team-page { max-width:1100px; width:100%; margin:0 auto; padding:0 0 60px; }

            /* ── Hero header ── */
            .team-hero { display:flex; align-items:flex-start; justify-content:space-between; gap:24px; padding:32px 0 28px; }
            .team-hero-left h1 { font-size:26px; font-weight:800; letter-spacing:-.02em; margin:0 0 6px; }
            .team-hero-left p  { font-size:14px; color:#64748b; margin:0; line-height:1.5; }
            .team-hero-btn { display:flex; align-items:center; gap:8px; padding:11px 20px; background:#0284c7; color:#fff; font-size:14px; font-weight:700; border:none; border-radius:10px; cursor:pointer; transition:.15s; font-family:'Inter',sans-serif; white-space:nowrap; box-shadow:0 4px 14px rgba(2,132,199,.25); }
            .team-hero-btn:hover { background:#0369a1; }

            /* ── Divider ── */
            .team-divider { height:1px; background:#e2e8f0; margin:0 0 28px; }

            /* ── Stats row ── */
            .team-stats { display:grid; grid-template-columns:repeat(4,1fr); gap:14px; margin-bottom:32px; }
            .team-stat { background:#fff; border:1px solid #e2e8f0; border-radius:12px; padding:18px 20px; }
            .team-stat-val { font-size:26px; font-weight:800; letter-spacing:-.02em; margin-bottom:2px; }
            .team-stat-lbl { font-size:12px; color:#64748b; font-weight:600; text-transform:uppercase; letter-spacing:.04em; }

            /* ── Section headers ── */
            .team-sh { display:flex; align-items:center; justify-content:space-between; margin-bottom:14px; }
            .team-sh-title { font-size:16px; font-weight:700; letter-spacing:-.01em; }
            .team-sh-meta { font-size:12px; color:#94a3b8; }

            /* ── Member cards ── */
            .team-members { display:flex; flex-direction:column; gap:10px; margin-bottom:32px; }
            .team-member { background:#fff; border:1px solid #e2e8f0; border-radius:12px; padding:16px 20px; display:flex; align-items:center; gap:16px; transition:.15s; }
            .team-member:hover { border-color:#cbd5e1; }
            .team-member-avatar { width:44px; height:44px; border-radius:50%; display:flex; align-items:center; justify-content:center; font-size:12px; font-weight:800; color:#fff; flex-shrink:0; }
            .team-member-info { flex:1; min-width:0; }
            .team-member-name { font-size:14px; font-weight:700; }
            .team-member-meta { font-size:12px; color:#94a3b8; margin-top:2px; display:flex; align-items:center; gap:6px; }
            .team-member-badge { display:inline-flex; align-items:center; gap:4px; font-size:11px; font-weight:700; padding:3px 8px; border-radius:6px; }
            .team-member-actions { display:flex; align-items:center; gap:8px; }
            .team-member-btn { display:flex; align-items:center; gap:4px; font-size:12px; font-weight:600; padding:6px 12px; border-radius:7px; border:1px solid #e2e8f0; background:#fff; cursor:pointer; font-family:'Inter',sans-serif; transition:.15s; color:#64748b; }
            .team-member-btn:hover { background:#f8fafc; border-color:#cbd5e1; color:#0f172a; }
            .team-member-btn-danger:hover { background:#fff1f2; border-color:#fda4af; color:#be123c; }

            /* ── Invite codes table ── */
            .team-codes { background:#fff; border:1px solid #e2e8f0; border-radius:14px; overflow:hidden; margin-bottom:32px; }
            .team-codes-header { display:grid; grid-template-columns:1fr 130px 120px 90px 80px 110px; gap:0; border-bottom:1px solid #e2e8f0; padding:10px 20px; font-size:11px; font-weight:700; text-transform:uppercase; letter-spacing:.05em; color:#94a3b8; }
            .team-code-row { display:grid; grid-template-columns:1fr 130px 120px 90px 80px 110px; gap:0; padding:14px 20px; border-bottom:1px solid #f1f5f9; align-items:center; transition:.12s; }
            .team-code-row:last-child { border-bottom:none; }
            .team-code-row:hover { background:#f8fafc; }
            .team-code-code { font-family:monospace; font-size:13px; font-weight:700; letter-spacing:.06em; color:#0284c7; background:rgba(2,132,199,.06); padding:3px 8px; border-radius:5px; display:inline-block; }
            .team-code-copy { background:none; border:none; cursor:pointer; color:#94a3b8; padding:2px; display:inline-flex; align-items:center; transition:.1s; }
            .team-code-copy:hover { color:#0284c7; }
            .team-code-status { display:inline-flex; align-items:center; gap:5px; font-size:11px; font-weight:700; padding:3px 9px; border-radius:6px; }
            .team-code-status-active { background:rgba(5,150,105,.08); color:#047857; }
            .team-code-status-inactive { background:#f1f5f9; color:#94a3b8; }
            .team-code-deact { background:none; border:none; cursor:pointer; color:#94a3b8; padding:4px; border-radius:5px; transition:.1s; font-size:12px; font-family:'Inter',sans-serif; display:flex; align-items:center; gap:3px; }
            .team-code-deact:hover { color:#be123c; background:#fff1f2; }
            .team-empty { text-align:center; padding:40px; color:#94a3b8; font-size:13px; }

            /* ── Modal overlay ── */
            .team-modal-bg { position:fixed; inset:0; background:rgba(0,0,0,.45); z-index:1000; display:flex; align-items:center; justify-content:center; padding:20px; backdrop-filter:blur(4px); }
            .team-modal { background:#fff; border-radius:16px; padding:28px; width:100%; max-width:480px; box-shadow:0 24px 48px rgba(0,0,0,.14); }
            .team-modal-h { font-size:18px; font-weight:800; margin-bottom:6px; }
            .team-modal-sub { font-size:13px; color:#64748b; margin-bottom:22px; }
            .tm-f { margin-bottom:14px; }
            .tm-label { display:block; font-size:12px; font-weight:700; text-transform:uppercase; letter-spacing:.05em; color:#64748b; margin-bottom:6px; }
            .tm-inp { width:100%; padding:10px 14px; border:1.5px solid #e2e8f0; border-radius:8px; font-size:14px; font-family:'Inter',sans-serif; outline:none; transition:.15s; box-sizing:border-box; }
            .tm-inp:focus { border-color:#0284c7; box-shadow:0 0 0 3px rgba(2,132,199,.12); }
            .tm-select { width:100%; padding:10px 14px; border:1.5px solid #e2e8f0; border-radius:8px; font-size:14px; font-family:'Inter',sans-serif; background:#fff; outline:none; cursor:pointer; transition:.15s; }
            .tm-select:focus { border-color:#0284c7; }
            .tm-footer { display:flex; gap:10px; margin-top:22px; }
            .tm-btn { flex:1; padding:11px; border-radius:9px; border:none; font-size:14px; font-weight:700; cursor:pointer; font-family:'Inter',sans-serif; transition:.15s; }
            .tm-btn-primary { background:#0284c7; color:#fff; box-shadow:0 4px 12px rgba(2,132,199,.25); }
            .tm-btn-primary:hover:not(:disabled) { background:#0369a1; }
            .tm-btn-primary:disabled { opacity:.6; cursor:not-allowed; }
            .tm-btn-ghost { background:#f1f5f9; color:#0f172a; border:1px solid #e2e8f0; }
            .tm-btn-ghost:hover { background:#e2e8f0; }
            .tm-success { text-align:center; padding:20px 0; }
            .tm-success-code { font-family:monospace; font-size:20px; font-weight:800; letter-spacing:.08em; color:#0284c7; background:rgba(2,132,199,.06); padding:8px 18px; border-radius:8px; display:inline-block; margin:12px 0; }
            .tm-err { background:#fff1f2; border:1px solid #fda4af; border-radius:8px; padding:10px 14px; font-size:13px; color:#be123c; margin-bottom:14px; }
            "#}
        </style>

        <div class="team-page">
            // ── Hero header ───────────────────────────────────────────────────
            <div class="team-hero">
                <div class="team-hero-left">
                    <h1>"Team & Access"</h1>
                    <p>"Manage who has access to your portfolio and how much control they have. Generate invite codes for new members."</p>
                </div>
                <button class="team-hero-btn" id="btn-invite-new"
                    on:click=move |_| {
                        modal_created.set(None);
                        modal_err.set(None);
                        modal_label.set(String::new());
                        modal_role.set("property_manager".to_string());
                        modal_uses.set("1".to_string());
                        modal_open.set(true);
                    }>
                    <span class="ms msf" style="font-size:18px;">"add_link"</span>
                    "Invite Member"
                </button>
            </div>
            <div class="team-divider"></div>

            // ── G-36 Network invites ──────────────────────────────────────────
            <div style="margin-bottom:28px;">
                <div class="team-sh">
                    <div class="team-sh-title">"Grow your network"</div>
                    <div class="team-sh-meta">"Connect"</div>
                </div>
                <p style="font-size:14px;color:#64748b;margin:0 0 14px;line-height:1.5;max-width:560px;">
                    "Invite fellow landlords and trusted contractors. Track who joined and which outcomes completed."
                </p>
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
                                    body: "Invite trades you already use. Dispatch and invoice live on Folio next time.",
                                },
                            ]
                            show_stats=true
                            show_history=true
                        />
                    }
                }
            </div>

            // ── Stats ─────────────────────────────────────────────────────────
            <div class="team-stats">
                <Suspense fallback=|| view! {
                    <div class="team-stat"><div class="team-stat-val">"-"</div><div class="team-stat-lbl">"Property Managers"</div></div>
                    <div class="team-stat"><div class="team-stat-val">"-"</div><div class="team-stat-lbl">"Vendors"</div></div>
                    <div class="team-stat"><div class="team-stat-val">"-"</div><div class="team-stat-lbl">"Active Codes"</div></div>
                    <div class="team-stat"><div class="team-stat-val">"-"</div><div class="team-stat-lbl">"Total Uses"</div></div>
                }>
                    {move || {
                        let codes = codes_resource_reloadable.get()
                            .and_then(|r| r.ok())
                            .map(|r| r.codes)
                            .unwrap_or_default();
                        let pm_count   = codes.iter().filter(|c| c.role == "property_manager" && c.is_active).count();
                        let vend_count = codes.iter().filter(|c| c.role == "vendor" && c.is_active).count();
                        let active_count = codes.iter().filter(|c| c.is_active).count();
                        let total_uses: i32 = codes.iter().map(|c| c.uses_count).sum();
                        view! {
                            <div class="team-stat">
                                <div class="team-stat-val">{pm_count}</div>
                                <div class="team-stat-lbl">"Property Managers"</div>
                            </div>
                            <div class="team-stat">
                                <div class="team-stat-val">{vend_count}</div>
                                <div class="team-stat-lbl">"Vendor Invites"</div>
                            </div>
                            <div class="team-stat">
                                <div class="team-stat-val">{active_count}</div>
                                <div class="team-stat-lbl">"Active Codes"</div>
                            </div>
                            <div class="team-stat">
                                <div class="team-stat-val">{total_uses}</div>
                                <div class="team-stat-lbl">"Total Code Uses"</div>
                            </div>
                        }
                    }}
                </Suspense>
            </div>

            // ── Invite Codes table ────────────────────────────────────────────
            <div class="team-sh">
                <div class="team-sh-title">"Active Invite Links"</div>
                <div class="team-sh-meta">"Share these links to let members self-onboard"</div>
            </div>

            <div class="team-codes">
                <div class="team-codes-header">
                    <span>"Code"</span>
                    <span>"Role"</span>
                    <span>"Label"</span>
                    <span>"Uses"</span>
                    <span>"Status"</span>
                    <span></span>
                </div>

                <Suspense fallback=|| view! {
                    <div class="team-empty"><span class="ms" style="font-size:28px; display:block; margin-bottom:8px;">"sync"</span>"Loading invite codes…"</div>
                }>
                    {move || {
                        let codes = codes_resource_reloadable.get()
                            .and_then(|r| r.ok())
                            .map(|r| r.codes)
                            .unwrap_or_default();

                        if codes.is_empty() {
                            view! {
                                <div class="team-empty">
                                    <span class="ms msf" style="font-size:32px; display:block; margin-bottom:8px; color:#cbd5e1;">"qr_code_2"</span>
                                    "No invite codes yet. Click 'Invite Member' to create your first one."
                                </div>
                            }.into_any()
                        } else {
                            let hdl = handle_deactivate.clone();
                            codes.into_iter().map(|c| {
                                let code_str = c.code.clone();
                                let cid = c.id.clone();
                                let hdl2 = hdl.clone();
                                let uses_text = match c.max_uses {
                                    Some(max) => format!("{}/{}", c.uses_count, max),
                                    None => format!("{}/∞", c.uses_count),
                                };
                                let is_pm = c.role == "property_manager";
                                view! {
                                    <div class="team-code-row">
                                        // Code
                                        <div style="display:flex; align-items:center; gap:8px;">
                                            <span class="team-code-code">{code_str.clone()}</span>
                                            <button class="team-code-copy" title="Copy invite link"
                                                on:click=move |_| {
                                                    let url = format!("{}/join/{}", "https://folio.app", &code_str);
                                                    if let Some(w) = web_sys::window() {
                                                        let _ = w.navigator().clipboard().write_text(&url);
                                                    }
                                                }>
                                                <span class="ms" style="font-size:14px;">"content_copy"</span>
                                            </button>
                                        </div>
                                        // Role badge
                                        <div>
                                            <span class="team-member-badge"
                                                style=move || format!("background:rgba(2,132,199,.08); color:{};", role_accent(&c.role))>
                                                <span class="ms msf" style="font-size:13px;">{role_icon(&c.role)}</span>
                                                {role_label(&c.role)}
                                            </span>
                                            {if is_pm {
                                                view! { <span style="display:block; font-size:10px; color:#94a3b8; margin-top:2px;">"employer linked"</span> }.into_any()
                                            } else { view! { <span></span> }.into_any() }}
                                        </div>
                                        // Label
                                        <div style="font-size:12px; color:#64748b; overflow:hidden; text-overflow:ellipsis; white-space:nowrap;">
                                            {c.label.clone().unwrap_or_else(|| "—".to_string())}
                                        </div>
                                        // Uses
                                        <div style="font-size:13px; font-weight:600; color:#0f172a;">{uses_text}</div>
                                        // Status
                                        <div>
                                            {if c.is_active {
                                                view! { <span class="team-code-status team-code-status-active"><span class="ms msf" style="font-size:11px;">"circle"</span>"Active"</span> }.into_any()
                                            } else {
                                                view! { <span class="team-code-status team-code-status-inactive">"Inactive"</span> }.into_any()
                                            }}
                                        </div>
                                        // Actions
                                        <div style="display:flex; justify-content:flex-end;">
                                            {if c.is_active {
                                                view! {
                                                    <button class="team-code-deact"
                                                        on:click=move |_| {
                                                            let id = cid.clone();
                                                            hdl2(id);
                                                        }>
                                                        <span class="ms" style="font-size:14px;">"block"</span>"Revoke"
                                                    </button>
                                                }.into_any()
                                            } else {
                                                view! { <span style="font-size:11px; color:#94a3b8;">"Revoked"</span> }.into_any()
                                            }}
                                        </div>
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </Suspense>
            </div>
        </div>

        // ── Modal: Create Invite Code ─────────────────────────────────────────
        <Show when=move || modal_open.get()>
            <div class="team-modal-bg" on:click=move |ev| {
                // Close when clicking the backdrop directly (not a child element).
                // current_target is always the div; if target == current_target the
                // user clicked the overlay, not any content inside it.
                let is_backdrop = ev.target()
                    .zip(ev.current_target())
                    .map(|(t, ct)| t == ct)
                    .unwrap_or(false);
                if is_backdrop { modal_open.set(false); }
            }>
                <div class="team-modal">
                    {move || if let Some(code) = modal_created.get() {
                        view! {
                            <div class="tm-success">
                                <span class="ms msf" style="font-size:40px; color:#0284c7; display:block; margin-bottom:12px;">"check_circle"</span>
                                <div style="font-size:18px; font-weight:800; margin-bottom:4px;">"Invite Code Created"</div>
                                <div style="font-size:13px; color:#64748b; margin-bottom:8px;">"Share this code or link with your new team member:"</div>
                                <div class="tm-success-code">{code.clone()}</div>
                                <div style="font-size:12px; color:#94a3b8; margin-bottom:18px;">
                                    {format!("folio.app/join/{}", &code)}
                                </div>
                                <button class="tm-btn tm-btn-primary" on:click=move |_| modal_open.set(false)>"Done"</button>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div>
                                <div class="team-modal-h">"New Invite Link"</div>
                                <div class="team-modal-sub">"The invited person will self-onboard through the role wizard."</div>

                                {move || modal_err.get().map(|e| view! {
                                    <div class="tm-err"><span class="ms" style="font-size:14px; margin-right:6px;">"warning"</span>{e}</div>
                                })}

                                <div class="tm-f">
                                    <label class="tm-label">"Role"</label>
                                    <select class="tm-select"
                                        prop:value=move || modal_role.get()
                                        on:change=move |e| modal_role.set(event_target_value(&e))>
                                        <option value="property_manager">"Property Manager"</option>
                                        <option value="vendor">"Vendor / Contractor"</option>
                                        <option value="cohost">"Co-host"</option>
                                        <option value="tenant">"Tenant Applicant"</option>
                                        <option value="agent">"Agent"</option>
                                    </select>
                                </div>

                                <div class="tm-f">
                                    <label class="tm-label">"Label " <span style="font-size:10px; color:#94a3b8; text-transform:none; letter-spacing:0; font-weight:400;">"(optional)"</span></label>
                                    <input class="tm-inp" type="text" placeholder="e.g. On-site PM for Oak St"
                                        prop:value=move || modal_label.get()
                                        on:input=move |e| modal_label.set(event_target_value(&e))/>
                                </div>

                                <div class="tm-f">
                                    <label class="tm-label">"Max Uses"</label>
                                    <select class="tm-select"
                                        prop:value=move || modal_uses.get()
                                        on:change=move |e| modal_uses.set(event_target_value(&e))>
                                        <option value="1">"Single use (1)"</option>
                                        <option value="5">"5 uses"</option>
                                        <option value="10">"10 uses"</option>
                                        <option value="unlimited">"Unlimited"</option>
                                    </select>
                                </div>

                                // PM-specific note about employer linking
                                <Show when=move || modal_role.get() == "property_manager">
                                    <div style="background:rgba(2,132,199,.06); border:1px solid rgba(2,132,199,.15); border-radius:8px; padding:12px 14px; font-size:12px; color:#0369a1; margin-bottom:14px; line-height:1.6; display:flex; align-items:flex-start; gap:8px;">
                                        <span class="ms msf" style="font-size:15px; flex-shrink:0; margin-top:1px;">"info"</span>
                                        "This PM will be linked to your account as their employer. You remain the admin. A property management agreement (G-11 contract) will be created when they accept."
                                    </div>
                                </Show>

                                <div class="tm-footer">
                                    <button class="tm-btn tm-btn-ghost" on:click=move |_| modal_open.set(false)>"Cancel"</button>
                                    <button class="tm-btn tm-btn-primary"
                                        disabled=move || modal_saving.get()
                                        on:click=handle_create>
                                        {move || if modal_saving.get() { "Creating…" } else { "Create Link" }}
                                    </button>
                                </div>
                            </div>
                        }.into_any()
                    }}
                </div>
            </div>
        </Show>
    }
}
