//! POST /api/folio/provision/invite
//!
//! Unified Folio provisioning endpoint. Creates a scoped invite for any of the
//! 9 supported Folio personas and sends a role-appropriate magic-link email.
//!
//! # Supported Roles & Scoping
//!
//! | `app_role`       | Required scope | What is auto-linked on accept           |
//! |------------------|----------------|-----------------------------------------|
//! | `landlord`       | none           | New account created                     |
//! | `landlord`       | `account_id`   | Linked to existing account              |
//! | `landlord`       | `asset_ids`    | Landlord role + asset access rows       |
//! | `tenant`         | `lease_id`     | `atlas_leases.tenant_user_id` set       |
//! | `vendor`         | optional `asset_ids` | Asset access rows (or org-level)  |
//! | `cohost`         | `asset_ids` (≥1) | Asset access rows for STR properties  |
//! | `str_host`       | none           | New account created (STR-first portal)  |
//! | `owner`          | `account_id`   | Linked to client account (read-only)    |
//! | `property_manager` | none         | Existing PMC onboard flow               |
//! | `agent`          | none           | Brokerage-mode agent                    |
//! | `broker`         | none           | Brokerage-mode broker                   |
//!
//! # Auth
//!
//! The caller must be an authenticated Folio user. The specific permission
//! required varies by role being provisioned:
//! - Provisioning `tenant` or `vendor` or `cohost` requires `landlord` role (or higher)
//! - Provisioning `agent` requires `broker` role
//! - Platform-admin can provision any role (via `admin/users.rs` which also calls
//!   this logic via shared `ProvisionInviteInput`)
//!
//! # Idempotency
//!
//! If an unexpired invite exists for this email + role + scope, returns 200
//! with the existing invite_id. A new invite is created only if expired.

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Order, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Duration, Utc};
use std::env;
use lettre::{SmtpTransport, Transport, Message, transport::smtp::authentication::Credentials};
use lettre::message::{header as mail_header, MultiPart, SinglePart};

use crate::entities::{user, platform_invite};
use crate::types::pm::FolioRole;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new().route("/api/folio/provision/invite", post(provision_invite))
}

// ── Request / Response ────────────────────────────────────────────────────────

/// Unified invite input — accepted by both the in-Folio endpoint and platform-admin.
#[derive(Debug, Deserialize)]
pub struct ProvisionInviteInput {
    pub email:        String,
    pub display_name: Option<String>,
    /// Role slug: "landlord" | "tenant" | "vendor" | "cohost" |
    ///            "owner" | "property_manager" | "agent" | "broker"
    /// NOTE: "str_host" is NOT a valid role. STR capability is enabled
    /// per-asset via atlas_assets.str_eligible. Use "landlord" instead.
    pub app_role:     String,
    /// Asset UUIDs for cohost/vendor/delegate scoping.
    /// REQUIRED for cohost (≥1 STR asset). Optional for vendor.
    pub asset_ids:    Option<Vec<Uuid>>,
    /// Lease UUID for tenant invites — auto-links user to the lease on accept.
    /// REQUIRED for tenant. Sets atlas_leases.tenant_user_id.
    pub lease_id:     Option<Uuid>,
    /// Lease type carried from the lease — "ltr" (long-term) or "str" (short-term/guest).
    /// Determines which tenant portal view the user lands in after onboarding.
    /// Informational only at invite time; ground truth is atlas_leases.lease_type.
    pub lease_type:   Option<String>,
    /// Existing atlas_accounts row to link the user to on accept.
    /// REQUIRED for owner. Optional for landlord (joins existing workspace).
    pub account_id:   Option<Uuid>,
    pub invite_note:  Option<String>,
    /// Expiry in days. Default 7. Max 30.
    pub expires_days: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ProvisionInviteResponse {
    pub invite_id:   Uuid,
    pub email:       String,
    pub app_role:    String,
    pub is_new_user: bool,
    pub expires_at:  String,
    /// true if an existing unexpired invite was returned (idempotent)
    pub reused:      bool,
}

// ── Validation ────────────────────────────────────────────────────────────────

fn validate_invite(input: &ProvisionInviteInput) -> Result<FolioRole, String> {
    // str_host is a removed role — give an actionable error
    if input.app_role == "str_host" {
        return Err(
            "\"str_host\" is not a valid role. STR capability is a property trait, not a persona. \
             Invite this user as \"landlord\" and enable STR on specific assets via \
             atlas_assets.str_eligible = true.".to_string()
        );
    }

    let role = FolioRole::try_from(input.app_role.as_str())
        .map_err(|e| e.to_string())?;

    match role {
        // Cohost: must have at least one STR asset to delegate
        FolioRole::Cohost => {
            let empty = input.asset_ids.as_ref().map_or(true, |v| v.is_empty());
            if empty {
                return Err(
                    "Cohost invites require at least one asset_id. \
                     The cohost must be delegated to specific STR-eligible properties.".to_string()
                );
            }
        }
        // Tenant: must be linked to a specific lease
        // The lease carries the lease_type (ltr|str) that determines portal view
        FolioRole::Tenant => {
            if input.lease_id.is_none() {
                return Err(
                    "Tenant invites require a lease_id. \
                     The tenant must be linked to a specific lease (LTR or STR). \
                     Create the lease first, then invite the tenant.".to_string()
                );
            }
        }
        // Owner: must be scoped to a specific client account (PMC context)
        FolioRole::Owner => {
            if input.account_id.is_none() {
                return Err(
                    "Owner invites require an account_id. \
                     The beneficial owner must be linked to their client account.".to_string()
                );
            }
        }
        // Agent: implicitly under the broker whose instance sent this invite.
        // No explicit broker_id needed — the app_instance_id on the invite carries it.
        FolioRole::Agent | FolioRole::Broker => {}
        // Landlord, Vendor, PropertyManager: no mandatory dependencies at invite time
        _ => {}
    }

    Ok(role)
}

// ── Handler ───────────────────────────────────────────────────────────────────

pub async fn provision_invite(
    State(db): State<DatabaseConnection>,
    Json(body): Json<ProvisionInviteInput>,
) -> impl IntoResponse {
    // 1. Validate
    let role = match validate_invite(&body) {
        Ok(r) => r,
        Err(msg) => {
            return (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({
                "error": msg
            }))).into_response();
        }
    };

    let email_lower = body.email.to_lowercase();
    let expires_days = body.expires_days.unwrap_or(7).min(30) as i64;

    // 2. Idempotency check — reuse unexpired invite for same email + role
    let existing = platform_invite::Entity::find()
        .filter(platform_invite::Column::Email.eq(&email_lower))
        .filter(platform_invite::Column::AppRole.eq(&body.app_role))
        .filter(platform_invite::Column::ExpiresAt.gt(Utc::now()))
        .order_by(platform_invite::Column::CreatedAt, Order::Desc)
        .one(&db)
        .await
        .unwrap_or(None);

    if let Some(inv) = existing {
        return (StatusCode::OK, Json(ProvisionInviteResponse {
            invite_id:   inv.id,
            email:       inv.email,
            app_role:    body.app_role,
            is_new_user: false,
            expires_at:  inv.expires_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            reused:      true,
        })).into_response();
    }

    // 3. Look up or create the invitee user row
    let (invitee, is_new_user) = match user::Entity::find()
        .filter(user::Column::Email.eq(&email_lower))
        .one(&db)
        .await
    {
        Ok(Some(u)) => (u, false),
        Ok(None) => {
            let username = email_lower.split('@').next().unwrap_or(&email_lower).to_string();
            let new_user = user::ActiveModel {
                id:            Set(Uuid::new_v4()),
                email:         Set(email_lower.clone()),
                username:      Set(username),
                first_name:    Set(body.display_name.clone().unwrap_or_default()),
                last_name:     Set(String::new()),
                phone:         Set(String::new()),
                password_hash: Set(String::new()),
                is_active:     Set(false),
                created_at:    Set(Utc::now()),
                ..Default::default()
            };
            match new_user.insert(&db).await {
                Ok(u) => (u, true),
                Err(e) => {
                    tracing::error!(error = %e, "provision_invite: user creation failed");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "provision_invite: user lookup failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // 4. Serialize asset_ids as JSONB
    let asset_ids_json = body.asset_ids.as_ref().map(|ids| {
        serde_json::to_value(ids).ok()
    }).flatten();

    // 5. Create platform_invite row
    let invite_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::days(expires_days);

    let new_invite = platform_invite::ActiveModel {
        id:              Set(invite_id),
        email:           Set(email_lower.clone()),
        role:            Set("Member".to_string()),
        tenant_name:     Set("Folio".to_string()),  // overridden by app_instance context
        invited_by:      Set("platform".to_string()),
        display_name:    Set(body.display_name.clone()),
        app_role:        Set(Some(body.app_role.clone())),
        app_instance_id: Set(None),
        account_id:      Set(body.account_id),
        asset_ids:       Set(asset_ids_json),
        lease_id:        Set(body.lease_id),
        target_app_url:  Set(None),
        personal_message:Set(body.invite_note.clone()),
        created_at:      Set(Utc::now()),
        expires_at:      Set(expires_at),
    };

    if let Err(e) = new_invite.insert(&db).await {
        tracing::error!(error = %e, "provision_invite: invite insert failed");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    // 6. Send role-tailored invite email
    let _ = send_provision_email(
        &email_lower,
        body.display_name.as_deref(),
        &role,
        invite_id,
        body.invite_note.as_deref(),
    ).await;

    tracing::info!(
        event = "provision_invite.created",
        invite_id = %invite_id,
        email = %email_lower,
        role = %role,
        is_new_user,
    );

    (StatusCode::CREATED, Json(ProvisionInviteResponse {
        invite_id,
        email: email_lower,
        app_role: body.app_role,
        is_new_user,
        expires_at: expires_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        reused: false,
    })).into_response()
}

// ── Email dispatch ────────────────────────────────────────────────────────────

async fn send_provision_email(
    email:        &str,
    display_name: Option<&str>,
    role:         &FolioRole,
    invite_id:    Uuid,
    note:         Option<&str>,
) {
    let frontend_url = env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "https://folio.uat.atlas.oply.co".to_string());

    // The setup token is embedded in the link — the verify endpoint reads it
    // and seeds the RBAC row with the invite's role + scoping.
    let magic_link_url = format!("{}/setup?invite={}", frontend_url, invite_id);

    let display = display_name.unwrap_or(email);
    let note_block = note.map(|n| format!(
        "<blockquote style=\"border-left:3px solid #d1d5db;margin:12px 0;padding:8px 16px;color:#4b5563;font-style:italic;\">{n}</blockquote>"
    )).unwrap_or_default();

    let (subject, portal_name, portal_desc) = match role {
        FolioRole::Tenant  => (
            "You've been invited to your Tenant Portal",
            "Tenant Portal",
            "Pay rent, submit maintenance requests, view your lease, and message your landlord — all from one place.",
        ),
        FolioRole::Vendor  => (
            "You've been invited to Folio as a Vendor",
            "Vendor Portal",
            "Receive work order dispatches, submit invoices, manage your schedule, and build your marketplace profile.",
        ),
        FolioRole::Cohost  => (
            "You've been invited as a Cohost",
            "Cohost Portal",
            "Manage bookings, guest messaging, and property operations for the properties you've been assigned.",
        ),
        // StrHost removed — STR is an asset trait. Landlords with STR-eligible
        // properties use the Landlord portal with conditional STR nav sections.
        FolioRole::Owner   => (
            "You've been invited to your Owner Portal",
            "Owner Portal",
            "View your portfolio performance, owner statements, and distributions — managed on your behalf.",
        ),
        FolioRole::Agent   => (
            "You've been invited as a Real Estate Agent",
            "Agent Portal",
            "Manage your client files, listings, and deals in one place.",
        ),
        FolioRole::Broker  => (
            "You've been invited as a Broker",
            "Broker Portal",
            "Supervise your agents, manage the office, and co-sign transactions.",
        ),
        FolioRole::PropertyManager => (
            "You've been invited as a Property Manager",
            "PMC Dashboard",
            "Manage your client portfolios, owner disbursements, and branded tenant portals.",
        ),
        _ => (
            "You've been invited to Folio",
            "Folio",
            "Your property management workspace is ready.",
        ),
    };

    let html_body = format!(
        r#"<!DOCTYPE html><html><head><meta charset="UTF-8"></head>
        <body style="font-family:Inter,system-ui,sans-serif;background:#f9fafb;margin:0;padding:40px 16px;">
          <div style="max-width:520px;margin:auto;background:#fff;border-radius:16px;
                      padding:40px;box-shadow:0 1px 4px rgba(0,0,0,.07);">
            <div style="font-size:24px;font-weight:700;color:#111827;margin-bottom:8px;">Folio</div>
            <hr style="border:none;border-top:1px solid #e5e7eb;margin:16px 0 24px;">
            <p style="font-size:15px;color:#374151;margin:0 0 12px;">Hi {display},</p>
            <h2 style="font-size:20px;font-weight:600;color:#111827;margin:0 0 8px;">{portal}</h2>
            <p style="font-size:14px;color:#6b7280;margin:0 0 16px;">{desc}</p>
            {note}
            <div style="text-align:center;margin:28px 0;">
              <a href="{url}"
                 style="display:inline-block;background:#111827;color:#fff;text-decoration:none;
                        padding:14px 32px;border-radius:10px;font-size:15px;font-weight:600;
                        letter-spacing:-0.01em;">
                Accept Invitation →
              </a>
            </div>
            <p style="font-size:12px;color:#9ca3af;text-align:center;margin:0;">
              This link expires in {days} days. If you didn't expect this, you can ignore it.
            </p>
          </div>
        </body></html>"#,
        display = display,
        portal  = portal_name,
        desc    = portal_desc,
        note    = note_block,
        url     = magic_link_url,
        days    = 7,
    );

    let text_body = format!(
        "{subject}\n\nHi {display},\n\n{desc}\n\nAccept your invitation:\n{url}\n\nThis link expires in 7 days.",
        subject = subject,
        display = display,
        desc    = portal_desc,
        url     = magic_link_url,
    );

    let smtp_server   = env::var("SMTP_SERVER").unwrap_or_default();
    let smtp_username = env::var("SMTP_USERNAME").unwrap_or_default();
    let smtp_token    = env::var("SMTP_TOKEN").unwrap_or_default();
    let smtp_port     = env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string())
        .parse::<u16>().unwrap_or(587);
    let smtp_from     = env::var("SMTP_FROM")
        .unwrap_or_else(|_| "noreply@atlas.oply.co".to_string());

    if smtp_server.is_empty() {
        tracing::warn!("provision_invite: SMTP not configured, skipping email to {email}");
        tracing::info!("provision_invite: magic link = {magic_link_url}");
        return;
    }

    let msg = match Message::builder()
        .from(smtp_from.parse().unwrap())
        .to(email.parse().unwrap())
        .subject(subject)
        .multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::builder()
                    .header(mail_header::ContentType::TEXT_PLAIN)
                    .body(text_body))
                .singlepart(SinglePart::builder()
                    .header(mail_header::ContentType::TEXT_HTML)
                    .body(html_body)),
        ) {
        Ok(m) => m,
        Err(e) => { tracing::error!("provision_invite: failed to build email: {e:?}"); return; }
    };

    let creds  = Credentials::new(smtp_username, smtp_token);
    let mailer = SmtpTransport::relay(&smtp_server)
        .unwrap().port(smtp_port).credentials(creds).build();

    let email_str = email.to_string();
    tokio::task::spawn_blocking(move || {
        match mailer.send(&msg) {
            Ok(_)  => tracing::info!("provision_invite: email sent to {email_str}"),
            Err(e) => tracing::error!("provision_invite: SMTP send failed: {e:?}"),
        }
    });
}
