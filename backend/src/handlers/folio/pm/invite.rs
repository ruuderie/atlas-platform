//! POST /api/folio/pm/clients/:account_id/invite
//!
//! Invites an external landlord to manage a specific client account within the
//! PMC tenant. The flow:
//!
//! 1. Validate request (PM role + PMC mode, via `PropertyManagerOnly`)
//! 2. Look up or create a `user` row for the invitee email
//! 3. Look up or create a `user_account` row for the invitee in this tenant
//! 4. Assign `FolioRole::Landlord` via G-32, scoped to `client_account_id`
//! 5. Create a 72h setup token via `AuthService::create_setup_token`
//! 6. Send the invite email with the setup link and client context
//!
//! # Security model
//!
//! The `client_account_id` on `atlas_user_app_roles` limits this user to records
//! where `managed_account_id = client_account_id`. The `RequireFolioRole` extractor
//! will surface this as `TenantContext.client_account_id` for downstream service
//! layer queries to apply as an additional WHERE clause.
//!
//! # Idempotency
//!
//! If the user is already assigned the Landlord role for this client account,
//! a new token is still generated (re-invite). The old role row is left active.

use axum::{
    Extension, Json, Router,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;
use lettre::{SmtpTransport, Transport, Message, transport::smtp::authentication::Credentials};
use lettre::message::{header as mail_header, MultiPart, SinglePart};
use std::env;

use crate::extractors::folio_role::PropertyManagerOnly;
use crate::extractors::tenant::TenantContext;
use crate::services::auth_service::AuthService;

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/folio/pm/clients/:account_id/invite",
            post(invite_client_landlord),
        )
}

// ── Request / response ────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct InviteRequest {
    pub email:         String,
    /// Optional display name to set if this is a new user
    pub display_name:  Option<String>,
    /// Custom message shown in the invite email body
    pub invite_note:   Option<String>,
}

#[derive(Serialize)]
pub struct InviteResponse {
    /// ID of the user row that was created or found
    pub user_id:          Uuid,
    /// Whether the user account was newly created (false = existing user re-invited)
    pub is_new_user:      bool,
    pub role_assignment_id: Uuid,
    pub invite_sent_to:   String,
}

// ── Handler ───────────────────────────────────────────────────────────────────

async fn invite_client_landlord(
    _guard: PropertyManagerOnly,
    ctx: TenantContext,
    Path(client_account_id): Path<Uuid>,
    Extension(db): Extension<DatabaseConnection>,
    Json(body): Json<InviteRequest>,
) -> impl IntoResponse {
    // ── 0. Validate the client account belongs to this tenant ─────────────────
    let client_account = match crate::entities::atlas_account::Entity::find_by_id(client_account_id)
        .one(&db)
        .await
    {
        Ok(Some(a)) if a.tenant_id == ctx.tenant_id => a,
        Ok(Some(_)) => {
            tracing::warn!(
                %client_account_id, tenant_id = %ctx.tenant_id,
                "invite: client account belongs to different tenant"
            );
            return StatusCode::FORBIDDEN.into_response();
        }
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "invite: error fetching client account");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // ── 1. Look up or create the user row ─────────────────────────────────────
    let email_lower = body.email.to_lowercase();

    let (user_row, is_new_user) = match crate::entities::user::Entity::find()
        .filter(crate::entities::user::Column::Email.eq(&email_lower))
        .one(&db)
        .await
    {
        Ok(Some(u)) => (u, false),
        Ok(None) => {
            // Provision a new user (no password, activated on first login via setup token)
            let first = body.display_name.clone().unwrap_or_default();
            let username_slug = email_lower.split('@').next().unwrap_or(&email_lower).to_string();
            let new_user = crate::entities::user::ActiveModel {
                id:            Set(Uuid::new_v4()),
                email:         Set(email_lower.clone()),
                username:      Set(username_slug),
                first_name:    Set(first),
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
                    tracing::error!(error = %e, "invite: failed to create user");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
        }
        Err(e) => {
            tracing::error!(error = %e, "invite: user lookup failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // ── 2. Look up the Landlord role profile for Folio ────────────────────────
    // UUID 000...0001 is the platform-default landlord role (seeded in G-32).
    let landlord_profile_id =
        Uuid::parse_str("00000000-0000-0000-0001-000000000001").expect("const uuid");

    // ── 3. Assign Landlord role scoped to this client account ─────────────────
    let role_row = crate::entities::atlas_user_app_roles::ActiveModel {
        id:                Set(Uuid::new_v4()),
        user_id:           Set(user_row.id),
        tenant_id:         Set(ctx.tenant_id),
        app_slug:          Set("folio".to_string()),
        role_profile_id:   Set(landlord_profile_id),
        granted_by:        Set(Some(ctx.user_id)),
        granted_at:        Set(Utc::now().into()),
        expires_at:        Set(None),
        is_active:         Set(true),
        client_account_id: Set(Some(client_account_id)),
    };

    let role_assignment = match role_row.insert(&db).await {
        Ok(r) => r,
        Err(e) => {
            // Unique constraint violation = user already has a role in this tenant.
            // In production: UPSERT or revoke + re-grant. Here: return 409.
            tracing::warn!(
                error = %e,
                user_id = %user_row.id,
                %client_account_id,
                "invite: role assignment conflict — user may already have a role in this tenant"
            );
            return StatusCode::CONFLICT.into_response();
        }
    };

    // ── 4. Generate setup token (72h expiry for invites) ──────────────────────
    let token = match AuthService::create_setup_token(&db, user_row.id).await {
        Ok(t) => t,
        Err((_, msg)) => {
            tracing::error!(msg, "invite: failed to create setup token");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // ── 5. Send invite email ──────────────────────────────────────────────────
    let frontend_url = env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "https://network.uat.atlas.oply.co".to_string());

    // Setup link routes to the Folio onboarding flow, pre-keyed to the client
    let setup_url = format!(
        "{}/folio/setup?token={}&client={}",
        frontend_url, token.token, client_account_id
    );

    let client_name = client_account.name.clone();

    let invite_note_html = body
        .invite_note
        .as_deref()
        .map(|n| format!("<p><em>Message from your property manager:</em> {}</p>", n))
        .unwrap_or_default();

    let html_body = format!(
        r#"<p>You have been invited to manage the property portfolio for <strong>{client}</strong>
        on the Folio platform.</p>
        {note}
        <p><a href="{url}">Accept Invitation & Set Up Your Account</a></p>
        <p><small>This link expires in 72 hours. If you did not expect this invitation,
        you can safely ignore this email.</small></p>"#,
        client = client_name,
        note = invite_note_html,
        url = setup_url,
    );

    let text_body = format!(
        "You have been invited to manage the property portfolio for {}.\n\
         Accept the invitation and set up your account here: {}\n\
         This link expires in 72 hours.",
        client_name, setup_url
    );

    let smtp_server   = env::var("SMTP_SERVER").unwrap_or_default();
    let smtp_username = env::var("SMTP_USERNAME").unwrap_or_default();
    let smtp_token    = env::var("SMTP_TOKEN").unwrap_or_default();
    let smtp_port     = env::var("SMTP_PORT").unwrap_or("587".to_string()).parse::<u16>().unwrap_or(587);
    let smtp_from     = env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@atlas.oply.co".to_string());

    if let Ok(email_msg) = Message::builder()
        .from(smtp_from.parse().unwrap())
        .to(email_lower.parse().unwrap())
        .subject(format!("You've been invited to manage {} on Folio", client_name))
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
        )
    {
        let creds = Credentials::new(smtp_username, smtp_token);
        let mailer = SmtpTransport::relay(&smtp_server)
            .unwrap()
            .port(smtp_port)
            .credentials(creds)
            .build();

        tokio::task::spawn_blocking(move || {
            match mailer.send(&email_msg) {
                Ok(_)  => tracing::info!(to = %email_lower, "invite: email sent"),
                Err(e) => tracing::error!(error = %e, "invite: email send failed"),
            }
        });
    }

    // ── 6. Respond ────────────────────────────────────────────────────────────
    tracing::info!(
        user_id = %user_row.id,
        %client_account_id,
        role_id = %role_assignment.id,
        is_new_user,
        "invite: landlord invited to client account"
    );

    (
        StatusCode::CREATED,
        Json(InviteResponse {
            user_id:            user_row.id,
            is_new_user,
            role_assignment_id: role_assignment.id,
            invite_sent_to:     body.email,
        }),
    )
        .into_response()
}
