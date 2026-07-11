// apps/folio/src/pages/tenant/profile.rs
//
// Tenant Profile — /t/profile
//
// Displays the tenant's account identity and role information from /api/folio/me.
// Shows notification channel preferences alongside profile data.
// Edit is available for name fields (when a PATCH /api/folio/me endpoint exists).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeProfile {
    pub user_id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub email: String,
    pub display_name: Option<String>,
    pub folio_role: String,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchTenantProfile, "/api")]
pub async fn fetch_tenant_profile() -> Result<MeProfile, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<MeProfile>("/api/folio/me", &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

#[cfg(feature = "ssr")]
fn session_token(
    headers: &axum::http::HeaderMap,
) -> Result<String, server_fn::error::ServerFnError> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            })
        })
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn role_label(role: &str) -> &'static str {
    match role {
        "Tenant" => "Tenant",
        "Landlord" => "Landlord / Operator",
        "PMC" => "Property Manager",
        "Vendor" => "Vendor",
        "Agent" => "Agent / Broker",
        "Owner" => "Property Owner",
        _ => "Folio User",
    }
}

fn initials_from(name: &Option<String>, email: &str) -> String {
    if let Some(n) = name {
        n.split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    } else {
        email
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string())
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantProfile() -> impl IntoView {
    let profile_res = Resource::new(|| (), |_| fetch_tenant_profile());

    // Edit mode state (ready for when PATCH /api/folio/me is added)
    let edit_mode = RwSignal::new(false);
    let edit_name = RwSignal::new(String::new());

    view! {
        <div class="main-area">

            // ── Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"My Profile"</h1>
                    <p class="page-subtitle">"Your account identity, role, and notification preferences"</p>
                </div>
            </div>

            <Suspense fallback=|| view! {
                <div class="profile-skeleton">"Loading profile…"</div>
            }>
                {move || profile_res.get().map(|res| {
                    match res {
                        Ok(profile) => {
                            let ini       = initials_from(&profile.display_name, &profile.email);
                            let name      = profile.display_name.clone().unwrap_or_else(|| profile.email.clone());
                            let role_str  = role_label(&profile.folio_role).to_string();
                            let email_str = profile.email.clone();
                            let uid_str   = profile.user_id.to_string();
                            let tid_str   = profile.tenant_id.map(|t| t.to_string()).unwrap_or_else(|| "—".to_string());

                            // Seed edit field
                            if edit_name.get().is_empty() {
                                edit_name.set(profile.display_name.clone().unwrap_or_default());
                            }

                            view! {
                                <div class="profile-layout">

                                    // ── Avatar card ──
                                    <div class="profile-avatar-card">
                                        <div class="profile-avatar">{ini}</div>
                                        <div class="profile-name">{name.clone()}</div>
                                        <div class="profile-role-chip">{role_str.clone()}</div>
                                        <div class="profile-email">{email_str.clone()}</div>

                                        <Show when=move || !edit_mode.get()>
                                            <button
                                                class="btn btn-ghost btn-sm mt-4"
                                                on:click=move |_| edit_mode.set(true)
                                            >
                                                "✏ Edit Name"
                                            </button>
                                        </Show>
                                    </div>

                                    // ── Detail cards ──
                                    <div class="profile-details">

                                        // Account info
                                        <div class="profile-section">
                                            <div class="profile-section-title">"Account Information"</div>
                                            <dl class="profile-dl">
                                                <dt>"Display Name"</dt>
                                                <dd>
                                                    <Show
                                                        when=move || edit_mode.get()
                                                        fallback=move || view! { {name.clone()} }
                                                    >
                                                        <div class="profile-edit-row">
                                                            <input
                                                                type="text"
                                                                class="form-input"
                                                                placeholder="Your name"
                                                                prop:value=edit_name
                                                                on:input=move |ev| edit_name.set(event_target_value(&ev))
                                                            />
                                                            <button
                                                                class="btn btn-primary btn-sm"
                                                                on:click=move |_| {
                                                                    // Saving will be wired when PATCH /api/folio/me is added
                                                                    edit_mode.set(false);
                                                                }
                                                            >"Save"</button>
                                                            <button
                                                                class="btn btn-ghost btn-sm"
                                                                on:click=move |_| edit_mode.set(false)
                                                            >"Cancel"</button>
                                                        </div>
                                                    </Show>
                                                </dd>
                                                <dt>"Email"</dt><dd>{email_str.clone()}</dd>
                                                <dt>"Role"</dt><dd>{role_str}</dd>
                                                <dt>"User ID"</dt>
                                                <dd class="font-mono text-xs opacity-60">{uid_str}</dd>
                                                <dt>"Tenant Account"</dt>
                                                <dd class="font-mono text-xs opacity-60">{tid_str}</dd>
                                            </dl>
                                        </div>

                                        // Security section
                                        <div class="profile-section">
                                            <div class="profile-section-title">"Security"</div>
                                            <div class="profile-security-row">
                                                <div class="profile-security-item">
                                                    <span class="profile-security-icon">"🔑"</span>
                                                    <div>
                                                        <div class="profile-security-label">"Passkey / WebAuthn"</div>
                                                        <div class="profile-security-sub">"Manage biometric login credentials"</div>
                                                    </div>
                                                    <button class="btn btn-ghost btn-sm">"Manage"</button>
                                                </div>
                                                <div class="profile-security-item">
                                                    <span class="profile-security-icon">"🛡"</span>
                                                    <div>
                                                        <div class="profile-security-label">"Two-Factor Authentication"</div>
                                                        <div class="profile-security-sub">"Add an extra layer of protection"</div>
                                                    </div>
                                                    <button class="btn btn-ghost btn-sm">"Set up"</button>
                                                </div>
                                            </div>
                                        </div>

                                        // Notification channels (read link to /t/notifications)
                                        <div class="profile-section">
                                            <div class="profile-section-title">"Notification Preferences"</div>
                                            <p class="text-xs text-on-surface-variant mb-3">
                                                "Configure how you receive alerts for rent reminders, maintenance updates, and messages."
                                            </p>
                                            <a href="/t/notifications" class="btn btn-ghost btn-sm inline-flex items-center gap-1">
                                                "⚙ Manage notification channels →"
                                            </a>
                                        </div>

                                        // Danger zone
                                        <div class="profile-section profile-section--danger">
                                            <div class="profile-section-title text-red-400">"Account Actions"</div>
                                            <div class="profile-danger-row">
                                                <div>
                                                    <div class="text-sm font-semibold">"Sign Out"</div>
                                                    <div class="text-xs text-on-surface-variant">"End your current session"</div>
                                                </div>
                                                <a href="/logout" class="btn btn-sm" style="background:rgba(239,68,68,0.12);border:1px solid rgba(239,68,68,0.3);color:#f87171;">"Sign Out"</a>
                                            </div>
                                        </div>

                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(e) => view! {
                            <div class="doc-empty text-red-400">"Could not load profile: " {e.to_string()}</div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
