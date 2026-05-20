use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use serde_json::{json, Value};
use uuid::Uuid;
use chrono::Utc;
use url::Url;
use validator::Validate;

use std::sync::Arc;
use crate::entities::{tenant, account, app_instance, app_domain, user, user_account};
use crate::handlers::passkeys::WebauthnState;
use crate::middleware::DynamicCorsRegistry;
use crate::models::provision::{ProvisionTenantPayload, ProvisionTenantResponse, validate_domain};
use crate::services::auth_service::AuthService;
use crate::services::ingress_provisioner::IngressProvisioner;
use crate::webauthn_registry::effective_tld_plus_one;
use crate::atlas_apps::core_platform::CorePlatformApp;
use crate::traits::atlas_app::AtlasApp;

// ── Error helpers ─────────────────────────────────────────────────────────────

fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::BAD_REQUEST, Json(json!({ "message": msg.into() })))
}

fn internal(msg: impl std::fmt::Display) -> (StatusCode, Json<Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "message": msg.to_string() })))
}

fn conflict(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::CONFLICT, Json(json!({ "message": msg.into() })))
}

fn forbidden(msg: impl Into<String>) -> (StatusCode, Json<Value>) {
    (StatusCode::FORBIDDEN, Json(json!({ "message": msg.into() })))
}

async fn verify_dns_txt_record(domain: &str, tenant_slug: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let url = format!("https://cloudflare-dns.com/dns-query?name={}&type=TXT", domain);
    
    let res = client.get(&url)
        .header("Accept", "application/dns-json")
        .send()
        .await
        .map_err(|e| format!("Failed to send DoH request: {}", e))?;
        
    if !res.status().is_success() {
        return Err(format!("Cloudflare DoH returned status {}", res.status()));
    }
    
    #[derive(serde::Deserialize)]
    struct DohAnswer {
        data: String,
    }
    
    #[derive(serde::Deserialize)]
    struct DohResponse {
        #[serde(rename = "Answer")]
        answer: Option<Vec<DohAnswer>>,
    }
    
    let dns_res: DohResponse = res.json()
        .await
        .map_err(|e| format!("Failed to parse DoH JSON: {}", e))?;
        
    let expected_txt = format!("\"atlas-verification={}\"", tenant_slug);
    let expected_txt_unquoted = format!("atlas-verification={}", tenant_slug);
    
    if let Some(answers) = dns_res.answer {
        for ans in answers {
            let trimmed = ans.data.trim();
            if trimmed == expected_txt || trimmed == expected_txt_unquoted {
                return Ok(());
            }
        }
    }
    
    Err(format!(
        "TXT record 'atlas-verification={}' not found for '{}'",
        tenant_slug, domain
    ))
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `POST /api/admin/tenants/provision`
///
/// Atomically provisions a fully wired tenant in a single transaction:
///   1. Validates input (domain format, slug uniqueness, domain uniqueness)
///   2. INSERT tenant
///   3. INSERT account (billing entity)
///   4. INSERT app_instance(s) — one per requested app type
///   5. INSERT app_domain — binds the FQDN to the anchor instance
///   6. CorePlatformApp::provision() — seeds default home page + header menu
///   7. UPSERT user (find-or-create by email)
///   8. GRANT Owner role on the new account
///   9. COMMIT
///  10. Seed WebAuthn registry for the new domain (live, no pod restart needed)
///  11. Generate one-time passkey setup token + send email to tenant admin
///
/// Requires: PlatformSuperAdmin role.
/// Idempotency: rejects duplicate `domain` with 409. Duplicate `tenant_name`
/// also returns 409. Calling again with a different domain is fine.
pub async fn provision_tenant(
    State(db): State<DatabaseConnection>,
    Extension(webauthn_state): Extension<WebauthnState>,
    Extension(cors_registry): Extension<Arc<DynamicCorsRegistry>>,
    Extension(ingress_provisioner): Extension<Arc<IngressProvisioner>>,
    Extension(user): Extension<user::Model>,
    Json(payload): Json<ProvisionTenantPayload>,
) -> Result<(StatusCode, Json<ProvisionTenantResponse>), (StatusCode, Json<Value>)> {

    // ── 0. Auth guard ──────────────────────────────────────────────────────────
    // The auth_middleware already enforces PlatformSuperAdmin for all /api/admin
    // routes. Here we additionally fetch the user_account row to confirm the role
    // is active, and to get the caller's account context for audit logging.
    let is_super_admin = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(user.id))
        .filter(user_account::Column::Role.eq(user_account::UserRole::PlatformSuperAdmin))
        .filter(user_account::Column::IsActive.eq(true))
        .one(&db)
        .await
        .map_err(|e| internal(e))?
        .is_some();

    if !is_super_admin {
        return Err(forbidden("This endpoint requires PlatformSuperAdmin role"));
    }

    // ── 1. Validate payload fields ─────────────────────────────────────────────
    payload.validate()
        .map_err(|e| bad_request(format!("Validation error: {e}")))?;

    validate_domain(&payload.domain)
        .map_err(|e| bad_request(e))?;

    // Determine which apps to provision (default: anchor)
    let apps: Vec<String> = payload.apps
        .clone()
        .unwrap_or_else(|| vec!["anchor".to_string()]);

    if !apps.contains(&"anchor".to_string()) {
        return Err(bad_request("At least one app must be 'anchor'"));
    }

    // ── 2. Pre-flight uniqueness checks (outside transaction for clear errors) ──
    let existing_tenant = tenant::Entity::find()
        .filter(tenant::Column::Name.eq(&payload.tenant_name))
        .one(&db)
        .await
        .map_err(|e| internal(e))?;
    if existing_tenant.is_some() {
        return Err(conflict(format!(
            "A tenant with slug '{}' already exists", payload.tenant_name
        )));
    }

    let existing_domain = app_domain::Entity::find()
        .filter(app_domain::Column::DomainName.eq(&payload.domain))
        .one(&db)
        .await
        .map_err(|e| internal(e))?;
    if existing_domain.is_some() {
        return Err(conflict(format!(
            "Domain '{}' is already registered to another app instance", payload.domain
        )));
    }

    // ── 2b. DNS TXT record challenge verification ──────────────────────────────
    let env = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "production".to_string());
    let bypass = payload.bypass_dns_verification.unwrap_or(false);
    
    if env == "development" || bypass || payload.domain == "localhost" {
        tracing::info!(
            event = "provision.dns_verification.bypass",
            domain = %payload.domain,
            reason = if env == "development" {
                "development_environment"
            } else if payload.domain == "localhost" {
                "localhost_domain"
            } else {
                "super_admin_bypass"
            }
        );
    } else {
        verify_dns_txt_record(&payload.domain, &payload.tenant_name)
            .await
            .map_err(|e| bad_request(format!("DNS verification failed: {}", e)))?;
    }

    // ── 3. Begin atomic transaction ────────────────────────────────────────────
    let txn = db.begin().await.map_err(|e| internal(e))?;

    // 3a. INSERT tenant
    let tenant_id = Uuid::new_v4();
    let now = Utc::now();
    let new_tenant = tenant::ActiveModel {
        id: Set(tenant_id),
        name: Set(payload.tenant_name.clone()),
        description: Set(payload.display_name.clone()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };
    new_tenant.insert(&txn).await.map_err(|e| internal(e))?;
    tracing::info!(event = "provision.tenant.created", tenant_id = %tenant_id, slug = %payload.tenant_name);

    // 3b. INSERT account
    let account_id = Uuid::new_v4();
    let new_account = account::ActiveModel {
        id: Set(account_id),
        tenant_id: Set(tenant_id),
        name: Set(format!("{} Account", payload.display_name)),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
        stripe_customer_id: Set(None),
        stripe_payment_method_id: Set(None),
    };
    new_account.insert(&txn).await.map_err(|e| internal(e))?;

    // 3c. INSERT app_instances (one per requested app type)
    let mut anchor_instance_id: Option<Uuid> = None;
    for app_type in &apps {
        let instance_id = Uuid::new_v4();
        let new_instance = app_instance::ActiveModel {
            id: Set(instance_id),
            tenant_id: Set(tenant_id),
            app_type: Set(app_type.clone()),
            database_url: Set(None),
            data_seed_name: Set(None),
            settings: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        new_instance.insert(&txn).await.map_err(|e| internal(e))?;
        if app_type == "anchor" {
            anchor_instance_id = Some(instance_id);
        }
        tracing::info!(event = "provision.app_instance.created", tenant_id = %tenant_id, app_type = %app_type, instance_id = %instance_id);
    }

    let anchor_instance_id = anchor_instance_id
        .ok_or_else(|| internal("anchor instance_id unexpectedly missing after insert"))?;

    // 3d. INSERT app_domain — binds the FQDN to the anchor instance
    let new_domain = app_domain::ActiveModel {
        id: Set(Uuid::new_v4()),
        app_instance_id: Set(anchor_instance_id),
        domain_name: Set(payload.domain.clone()),
        created_at: Set(now),
    };
    new_domain.insert(&txn).await.map_err(|e| internal(e))?;
    tracing::info!(event = "provision.app_domain.created", tenant_id = %tenant_id, domain = %payload.domain);

    // 3e. (CMS scaffolding + module seeding called post-commit below — idempotent)

    // 3f. UPSERT user by email (find-or-create)
    let existing_user = user::Entity::find()
        .filter(user::Column::Email.eq(&payload.admin_email))
        .one(&txn)
        .await
        .map_err(|e| internal(e))?;

    let user_id = if let Some(u) = existing_user {
        tracing::info!(event = "provision.user.found_existing", user_id = %u.id, email = %payload.admin_email);
        u.id
    } else {
        let new_user_id = Uuid::new_v4();
        let new_user = user::ActiveModel {
            id: Set(new_user_id),
            email: Set(payload.admin_email.clone()),
            username: Set(payload.admin_email.clone()),
            first_name: Set(payload.admin_first_name.clone()),
            last_name: Set(payload.admin_last_name.clone()),
            phone: Set(String::new()),
            password_hash: Set(String::new()), // passwordless — passkey setup link sent via email
            is_active: Set(true),
            last_login: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };
        new_user.insert(&txn).await.map_err(|e| internal(e))?;
        tracing::info!(event = "provision.user.created", user_id = %new_user_id, email = %payload.admin_email);
        new_user_id
    };

    // 3g. GRANT Owner role on the new account
    let new_user_account = user_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        account_id: Set(account_id),
        role: Set(user_account::UserRole::Owner),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    };
    new_user_account.insert(&txn).await.map_err(|e| internal(e))?;

    // ── 4. COMMIT ──────────────────────────────────────────────────────────────
    txn.commit().await.map_err(|e| internal(e))?;
    tracing::info!(event = "provision.committed", tenant_id = %tenant_id, domain = %payload.domain);

    // ── 5. Post-commit idempotent provisioning ────────────────────────────────
    // Called post-commit on DatabaseConnection so AtlasApp::provision() trait
    // signatures are satisfied. Each step uses ON CONFLICT DO NOTHING.

    // 5a. CMS scaffolding (default home page + header menu)
    CorePlatformApp.provision(&db, tenant_id)
        .await
        .map_err(|e| internal(format!("CMS provisioning failed: {e}")))?;

    // 5b. Admin module seeding for anchor app
    {
        use crate::atlas_apps::anchor::AnchorApp;
        if let Err(e) = AnchorApp.provision(&db, tenant_id).await {
            tracing::warn!(event = "provision.module_seed.failed", tenant_id = %tenant_id, error = %e);
        }
    }

    // 5c. Seed WebAuthn registry (live, no pod restart needed)
    //     Done after commit so only successfully provisioned tenants are cached.
    let origin_url = format!("https://{}", payload.domain);
    if let Ok(url) = Url::parse(&origin_url) {
        if let Some(host) = url.host_str() {
            let rp_id = effective_tld_plus_one(host);
            let _ = webauthn_state.registry.get_or_create(&origin_url).await;
            tracing::info!(event = "provision.webauthn.seeded", domain = %payload.domain, rp_id = %rp_id);
        }
    }

    // 5d. Register new domain with the dynamic CORS registry on-the-fly
    cors_registry.add_host(&payload.domain);

    // 5e. Trigger ingress and TLS automation via K8s Sidecar
    if let Err(e) = ingress_provisioner.provision_domain(&payload.tenant_name, &payload.domain).await {
        tracing::error!(
            event = "provision.ingress.failed",
            tenant_slug = %payload.tenant_name,
            domain = %payload.domain,
            error = %e,
            message = "Ingress provisioning failed, but tenant database setup succeeded."
        );
    }

    // ── 6. Generate one-time passkey setup link + send email ───────────────────
    let setup_token = AuthService::create_setup_token(&db, user_id)
        .await
        .map_err(|(status, msg)| (status, Json(json!({ "message": msg }))))?;

    let setup_url = format!("https://{}/setup-passkey?token={}", payload.domain, setup_token.token);

    // Fire-and-forget setup email — do not block the response on email delivery
    {
        let db_clone = db.clone();
        let to_email = payload.admin_email.clone();
        let display_name = payload.display_name.clone();
        let url = setup_url.clone();
        let tenant_id_for_email = tenant_id;
        tokio::spawn(async move {
            let email_payload = crate::handlers::communications::SendEmailPayload {
                tenant_id: tenant_id_for_email,
                to_email,
                subject: format!("Welcome to {} — Set up your passkey", display_name),
                body_html: format!(
                    "<h2>Your workspace is ready</h2>\
                     <p>You've been granted admin access to <strong>{display_name}</strong>.</p>\
                     <p>Click the link below to set up your passkey and access your dashboard:</p>\
                     <p><a href=\"{url}\">{url}</a></p>\
                     <p><em>This link expires in 24 hours.</em></p>",
                ),
            };
            let _ = crate::handlers::communications::send_email_handler(
                axum::extract::State(db_clone),
                axum::Json(email_payload),
            ).await;
        });
    }

    tracing::info!(
        event = "provision.complete",
        tenant_id = %tenant_id,
        domain = %payload.domain,
        admin_email = %payload.admin_email,
    );

    Ok((StatusCode::CREATED, Json(ProvisionTenantResponse {
        tenant_id,
        account_id,
        domain: payload.domain,
        setup_url,
    })))
}
