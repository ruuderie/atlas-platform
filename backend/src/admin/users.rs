//! Admin — Platform User Invitations handler
//!
//! Manages invitations for new platform users, including app-scoped
//! Folio invites that automatically send a branded magic-link email.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/admin/users/invites            → list_invites
//! POST /api/admin/users/invite             → create_invite  (enhanced)
//! DELETE /api/admin/users/invites/{id}     → revoke_invite
//! POST /api/admin/users/invites/{id}/resend → resend_invite
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use chrono::{Duration, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{platform_invite, user};

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/users/invites", get(list_invites))
        .route("/api/admin/users/invite", post(create_invite))
        .route("/api/admin/users/invites/{id}", delete(revoke_invite))
        .route("/api/admin/users/invites/{id}/resend", post(resend_invite))
}

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InviteResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub role: String,
    pub app_role: Option<String>,
    pub tenant: String,
    pub app_instance_id: Option<Uuid>,
    pub invited_by: String,
    pub sent: String,
    pub expires: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateInviteInput {
    pub email: String,
    /// Full name to pre-fill on the user record when the invite is accepted
    pub display_name: Option<String>,
    /// Platform privilege level: "Admin" | "Editor" | "Viewer"
    pub role: String,
    /// Folio-specific persona: "landlord" | "pmc" | "tenant" | "str_host" | "vendor"
    pub app_role: Option<String>,
    /// Tenant name / label shown in the invite table
    pub tenant: String,
    /// Scopes the invite to a specific app instance
    pub app_instance_id: Option<Uuid>,
    /// When set: links the invited user to this existing atlas_accounts row instead of
    /// creating a new account. Useful for adding a user to an existing workspace.
    pub account_id: Option<Uuid>,
    /// When set: scopes the invite to specific atlas_assets rows (cohost/vendor/delegate).
    /// NULL = no asset restriction (org-level access for the granted role).
    #[serde(default)]
    pub asset_ids: Option<Vec<Uuid>>,
    /// When set (tenant invites): auto-links the accepted user to this lease.
    pub lease_id: Option<Uuid>,
    /// Overrides FRONTEND_URL for the magic link — use the instance's custom domain
    pub target_app_url: Option<String>,
    /// Optional personal note from the operator, shown in the email
    pub personal_message: Option<String>,
    /// Expiry days — default 7, max 30
    pub expires_days: Option<i64>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert an app_role slug into a display label for the invite email.
/// The invite system doesn't enumerate roles — each app owns its role vocabulary.
/// This just title-cases common slug patterns (e.g. "str_host" → "Str Host") and
/// falls back to "User" for empty values. Apps can override the label at their
/// own onboarding step.
fn app_role_label(role: &str) -> String {
    if role.is_empty() {
        return "User".to_string();
    }
    role.split('_')
        .map(|word| {
            let mut c = word.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Compose and dispatch the branded magic-link invitation email.
async fn dispatch_invite_email(
    db: &DatabaseConnection,
    invite_id: Uuid,
    email: &str,
    display_name: Option<&str>,
    invited_by: &str,
    app_role: Option<&str>,
    tenant_name: &str,
    personal_message: Option<&str>,
    target_app_url: Option<&str>,
) {
    use crate::services::auth_service::AuthService;
    use lettre::{
        Message, SmtpTransport, Transport,
        message::{MultiPart, SinglePart, header as mail_header},
        transport::smtp::authentication::Credentials,
    };
    use std::env;

    // ── 1. Create magic link token ─────────────────────────────────────────
    let token = match AuthService::create_magic_link(db, email).await {
        Ok(t) => t.token,
        Err((_, msg)) => {
            tracing::warn!("Could not create magic link for {}: {}", email, msg);
            return;
        }
    };

    // ── 2. Resolve landing URL ─────────────────────────────────────────────
    let mut resolved_url = target_app_url
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string());

    if resolved_url.is_none() {
        use crate::entities::{app_domain, platform_invite};
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
        if let Ok(Some(invite)) = platform_invite::Entity::find_by_id(invite_id).one(db).await {
            if let Some(instance_id) = invite.app_instance_id {
                if let Ok(Some(domain_record)) = app_domain::Entity::find()
                    .filter(app_domain::Column::AppInstanceId.eq(instance_id))
                    .one(db)
                    .await
                {
                    resolved_url = Some(format!("https://{}", domain_record.domain_name));
                }
            }
        }
    }

    let frontend_url = resolved_url.unwrap_or_else(|| {
        env::var("FRONTEND_URL").unwrap_or_else(|_| "https://network.dev.atlas.oply.co".to_string())
    });
    let magic_link_url = format!("{}/magic-login?token={}", frontend_url, token);

    // ── 3. Build email content ─────────────────────────────────────────────
    let recipient_name = display_name.unwrap_or("there");
    let role_label = app_role
        .map(app_role_label)
        .unwrap_or_else(|| "User".to_string());
    let tenant_display = if tenant_name.is_empty() {
        "Atlas Platform"
    } else {
        tenant_name
    };
    let message_block = personal_message
        .filter(|m| !m.is_empty())
        .map(|m| format!(
            r#"<div style="background:#f8f9fc;border-left:3px solid #6366f1;padding:12px 16px;border-radius:4px;margin:20px 0;font-style:italic;color:#4b5563;">&ldquo;{}&rdquo;</div>"#,
            html_escape(m)
        ))
        .unwrap_or_default();

    let html_body = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>You're invited to {tenant_display}</title>
</head>
<body style="margin:0;padding:0;background:#f0f2f5;font-family:'Segoe UI',Arial,sans-serif;">
  <div style="max-width:560px;margin:40px auto;background:#ffffff;border-radius:12px;overflow:hidden;box-shadow:0 2px 12px rgba(0,0,0,.08);">

    <!-- Header -->
    <div style="background:linear-gradient(135deg,#1e293b 0%,#312e81 100%);padding:36px 40px;text-align:center;">
      <div style="font-size:28px;margin-bottom:8px;">⚡</div>
      <div style="color:#ffffff;font-size:22px;font-weight:700;letter-spacing:-0.3px;">Atlas Platform</div>
      <div style="color:#a5b4fc;font-size:13px;margin-top:4px;">{tenant_display}</div>
    </div>

    <!-- Body -->
    <div style="padding:40px;">
      <p style="font-size:18px;font-weight:600;color:#111827;margin:0 0 8px;">Hi {recipient_name},</p>
      <p style="font-size:15px;color:#6b7280;margin:0 0 20px;line-height:1.6;">
        <strong style="color:#374151;">{invited_by}</strong> has invited you to join <strong style="color:#374151;">{tenant_display}</strong> as a <strong style="color:#374151;">{role_label}</strong>.
      </p>

      {message_block}

      <!-- CTA -->
      <div style="text-align:center;margin:32px 0;">
        <a href="{magic_link_url}"
           style="display:inline-block;background:linear-gradient(135deg,#4f46e5,#7c3aed);color:#ffffff;text-decoration:none;padding:14px 36px;border-radius:8px;font-size:16px;font-weight:600;letter-spacing:0.2px;box-shadow:0 4px 14px rgba(99,102,241,.35);">
          Accept Invitation →
        </a>
      </div>

      <p style="font-size:13px;color:#9ca3af;text-align:center;margin:0 0 8px;">
        Or copy this link into your browser:
      </p>
      <p style="font-size:11px;color:#9ca3af;text-align:center;word-break:break-all;margin:0;">
        {magic_link_url}
      </p>
    </div>

    <!-- Footer -->
    <div style="background:#f9fafb;border-top:1px solid #e5e7eb;padding:20px 40px;text-align:center;">
      <p style="font-size:12px;color:#9ca3af;margin:0 0 4px;">
        This invitation expires in 7 days. If you didn't expect it, you can safely ignore this email.
      </p>
      <p style="font-size:12px;color:#9ca3af;margin:0;">
        You'll be asked to create a passkey (Touch ID / Face ID) to secure your account.
      </p>
    </div>
  </div>
</body>
</html>"#,
        tenant_display = tenant_display,
        recipient_name = recipient_name,
        invited_by = invited_by,
        role_label = role_label,
        message_block = message_block,
        magic_link_url = magic_link_url,
    );

    let text_body = format!(
        "Hi {recipient_name},\n\n\
        {invited_by} has invited you to join {tenant_display} as a {role_label}.\n\n\
        Accept your invitation:\n{magic_link_url}\n\n\
        This link expires in 7 days. You'll be asked to create a passkey to secure your account.\n\n\
        If you didn't expect this, ignore this email.\n",
        recipient_name = recipient_name,
        invited_by = invited_by,
        tenant_display = tenant_display,
        role_label = role_label,
        magic_link_url = magic_link_url,
    );

    // ── 4. Dispatch via SMTP ───────────────────────────────────────────────
    let smtp_server = env::var("SMTP_SERVER").unwrap_or_default();
    let smtp_username = env::var("SMTP_USERNAME").unwrap_or_default();
    let smtp_token = env::var("SMTP_TOKEN").unwrap_or_default();
    let smtp_port = env::var("SMTP_PORT")
        .unwrap_or("587".to_string())
        .parse::<u16>()
        .unwrap_or(587);
    let smtp_from = env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@atlas.oply.co".to_string());

    if smtp_server.is_empty() || smtp_username.is_empty() {
        tracing::warn!("SMTP not configured — skipping invite email to {}", email);
        return;
    }

    let from_addr = format!("Atlas Platform <{}>", smtp_from);
    let message = match Message::builder()
        .from(from_addr.parse().unwrap())
        .to(email.parse().unwrap())
        .subject(format!("You've been invited to {}", tenant_display))
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(mail_header::ContentType::TEXT_PLAIN)
                        .body(text_body),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(mail_header::ContentType::TEXT_HTML)
                        .body(html_body),
                ),
        ) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("Failed to build invite email: {:?}", e);
            return;
        }
    };

    let creds = Credentials::new(smtp_username, smtp_token);
    let mailer = SmtpTransport::starttls_relay(&smtp_server)
        .unwrap()
        .port(smtp_port)
        .credentials(creds)
        .build();

    match mailer.send(&message) {
        Ok(_) => tracing::info!("Invite email sent to {}", email),
        Err(e) => tracing::error!("Failed to send invite email to {}: {:?}", email, e),
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_invites(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let list = platform_invite::Entity::find()
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list platform invites: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response: Vec<InviteResponse> = list
        .into_iter()
        .map(|m| InviteResponse {
            id: m.id,
            email: m.email,
            display_name: m.display_name,
            role: m.role,
            app_role: m.app_role,
            tenant: m.tenant_name,
            app_instance_id: m.app_instance_id,
            invited_by: m.invited_by,
            sent: m.created_at.format("%b %d").to_string(),
            expires: m.expires_at.format("%b %d").to_string(),
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_invite(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateInviteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let invited_by_str = {
        let name = format!("{} {}", current_user.first_name, current_user.last_name);
        let name = name.trim();
        if name.is_empty() {
            current_user.email.clone()
        } else {
            name.to_string()
        }
    };

    let expires_days = input.expires_days.unwrap_or(7).clamp(1, 30);
    let id = Uuid::new_v4();
    let created_at = Utc::now();
    let expires_at = created_at + Duration::days(expires_days);

    let new_invite = platform_invite::ActiveModel {
        id: Set(id),
        email: Set(input.email.clone()),
        role: Set(input.role.clone()),
        tenant_name: Set(input.tenant.clone()),
        invited_by: Set(invited_by_str.clone()),
        display_name: Set(input.display_name.clone()),
        app_role: Set(input.app_role.clone()),
        app_instance_id: Set(input.app_instance_id),
        account_id: Set(input.account_id),
        asset_ids: Set(input
            .asset_ids
            .as_ref()
            .and_then(|ids| serde_json::to_value(ids).ok())),
        lease_id: Set(input.lease_id),
        target_app_url: Set(input.target_app_url.clone()),
        personal_message: Set(input.personal_message.clone()),
        created_at: Set(created_at),
        expires_at: Set(expires_at),
    };

    new_invite.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to create platform invite: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Fire-and-forget: dispatch the branded magic link email asynchronously
    {
        let db2 = db.clone();
        let email = input.email.clone();
        let dname = input.display_name.clone();
        let iby = invited_by_str.clone();
        let frole = input.app_role.clone();
        let tenant = input.tenant.clone();
        let pmsg = input.personal_message.clone();
        let turl = input.target_app_url.clone();
        tokio::spawn(async move {
            dispatch_invite_email(
                &db2,
                id,
                &email,
                dname.as_deref(),
                &iby,
                frole.as_deref(),
                &tenant,
                pmsg.as_deref(),
                turl.as_deref(),
            )
            .await;
        });
    }

    let response = InviteResponse {
        id,
        email: input.email,
        display_name: input.display_name,
        role: input.role,
        app_role: input.app_role,
        tenant: input.tenant,
        app_instance_id: input.app_instance_id,
        invited_by: invited_by_str,
        sent: created_at.format("%b %d").to_string(),
        expires: expires_at.format("%b %d").to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn revoke_invite(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    platform_invite::Entity::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to revoke platform invite: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

pub async fn resend_invite(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let invite = platform_invite::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active: platform_invite::ActiveModel = invite.clone().into();
    let now = Utc::now();
    active.created_at = Set(now);
    active.expires_at = Set(now + Duration::days(7));
    active.update(&db).await.map_err(|e| {
        tracing::error!("Failed to resend platform invite: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Re-dispatch email
    {
        let db2 = db.clone();
        let email = invite.email.clone();
        let dname = invite.display_name.clone();
        let iby = invite.invited_by.clone();
        let frole = invite.app_role.clone();
        let tenant = invite.tenant_name.clone();
        let pmsg = invite.personal_message.clone();
        let turl = invite.target_app_url.clone();
        tokio::spawn(async move {
            dispatch_invite_email(
                &db2,
                id,
                &email,
                dname.as_deref(),
                &iby,
                frole.as_deref(),
                &tenant,
                pmsg.as_deref(),
                turl.as_deref(),
            )
            .await;
        });
    }

    Ok(StatusCode::OK)
}
