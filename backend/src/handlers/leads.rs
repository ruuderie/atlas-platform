use axum::{
    extract::{Extension, Path, Json},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};

// ============================================================
// LEGACY CRM HANDLER - CUTOVER IN PROGRESS
// This file is a migration bridge. 
// New code should use:
//   - AccountService + ContactService for parties
//   - OpportunityService for leads/deals
//   - CaseService + RealtimeService for cases/activities
//   - Ledger for any billing
// Old direct entity writes (lead::, contact:: etc.) are being phased out.
// ============================================================
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, Set, ColumnTrait,
    ActiveModelTrait, ModelTrait, TransactionTrait
};
use uuid::Uuid;
use chrono::Utc;
use crate::entities::{lead, listing, account, note, user, activity, contact, user_account};
use crate::models::lead::{LeadModel, CreateLeadInput, UpdateLeadInput};

// New unified services (cutover in progress)
use crate::services::account_service::AccountService;
use crate::services::contact_service::ContactService;
use crate::services::opportunity_service::OpportunityService;
use crate::services::ledger;
use crate::models::file::FileAssociation;
use crate::models::note::{NoteModel, CreateNoteInput};
use crate::models::activity::{ActivityModel, CreateActivityInput};
use axum::http::HeaderMap;
use std::time::{Instant, Duration};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;

static RATE_LIMITER: Lazy<Arc<DashMap<String, (u32, Instant)>>> = Lazy::new(|| Arc::new(DashMap::new()));

pub fn public_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/leads", post(create_lead))
        .route("/api/v1/leads/ingest", post(ingest_lead))
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/leads", get(get_leads))
        .route("/api/leads/{id}", get(get_lead))
        .route("/api/leads/{id}", put(update_lead))
        .route("/api/leads/{id}", delete(delete_lead))
        .route("/api/crm/leads/{id}/convert", post(convert_lead))
        .route("/api/leads/{lead_id}/files/{file_id}", post(add_file_to_lead))
        .route("/api/leads/{id}/files", get(get_lead_files))
        .route("/api/leads/{id}/notes", get(get_lead_notes))
        .route("/api/leads/{id}/activities", get(get_lead_activities))
}

pub async fn create_lead(
    Extension(db): Extension<DatabaseConnection>,
    site_config_opt: Option<Extension<crate::config::site_config::SiteConfig>>,
    headers: HeaderMap,
    Json(input): Json<CreateLeadInput>,
) -> Result<impl IntoResponse, StatusCode> {
    // 1. Honeypot check
    if let Some(bot_val) = &input._bot_check {
        if !bot_val.is_empty() {
            tracing::warn!("Honeypot triggered on lead submission");
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // 2. IP Rate Limiting
    if let Some(ip) = headers.get("x-forwarded-for").and_then(|h| h.to_str().ok()).or_else(|| headers.get("x-real-ip").and_then(|h| h.to_str().ok())) {
        let ip = ip.split(',').next().unwrap_or("").trim().to_string();
        if !ip.is_empty() {
            let mut entry = RATE_LIMITER.entry(ip).or_insert((0, Instant::now()));
            if entry.value().1.elapsed() > Duration::from_secs(60) {
                entry.value_mut().0 = 1;
                entry.value_mut().1 = Instant::now();
            } else {
                entry.value_mut().0 += 1;
                if entry.value().0 > 3 { // Max 3 leads per minute per IP
                    tracing::warn!("Rate limit exceeded for lead submissions from IP");
                    return Err(StatusCode::TOO_MANY_REQUESTS);
                }
            }
        }
    }

    // Check if the listing exists if provided
    if let Some(listing_id) = input.listing_id {
        listing::Entity::find_by_id(listing_id)
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::NOT_FOUND)?;
    }

    // 3. Resolve Account ID securely from SiteConfig/Listing
    let mut resolved_account_id = None;
    if let Some(listing_id) = input.listing_id {
        // If a listing is provided, find the account associated with the listing's profile
        if let Ok(Some(lst)) = listing::Entity::find_by_id(listing_id).one(&db).await {
            if let Ok(Some(profile)) = crate::entities::profile::Entity::find_by_id(lst.profile_id).one(&db).await {
                resolved_account_id = Some(profile.account_id);
            }
        }
    }
    
    // If no listing provided or listing not found, fallback to the primary account of the active network
    if resolved_account_id.is_none() {
        if let Some(Extension(site_config)) = &site_config_opt {
            if let Ok(Some(primary_account)) = account::Entity::find()
                .filter(account::Column::TenantId.eq(site_config.tenant_id))
                .one(&db)
                .await 
            {
                resolved_account_id = Some(primary_account.id);
            } else {
                // If the network has no accounts at all, fallback
                resolved_account_id = input.account_id;
            }
        } else {
            resolved_account_id = input.account_id;
        }
    }

    let mut resolved_tenant_id = input.tenant_id;
    if let Some(Extension(site_config)) = site_config_opt {
        resolved_tenant_id = Some(site_config.tenant_id);
    }
    // Final fallback: derive tenant_id from the atlas_account record for the resolved account.
    // This covers authenticated direct-API calls (no SiteConfig) where the caller
    // supplies account_id but not tenant_id.
    if resolved_tenant_id.is_none() {
        if let Some(acct_id) = resolved_account_id {
            use crate::entities::atlas_account;
            if let Ok(Some(acct)) = atlas_account::Entity::find_by_id(acct_id).one(&db).await {
                resolved_tenant_id = Some(acct.tenant_id);
            }
        }
    }

    let mut new_lead = lead::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(input.name),
        listing_id: Set(input.listing_id),
        account_id: Set(resolved_account_id),
        first_name: Set(input.first_name),
        last_name: Set(input.last_name),
        email: Set(input.email),
        phone: Set(input.phone),
        whatsapp: Set(input.whatsapp),
        telegram: Set(input.telegram),
        twitter: Set(input.twitter),
        instagram: Set(input.instagram),
        facebook: Set(input.facebook),
        message: Set(input.message),
        source: Set(input.source),
        is_converted: Set(false),
        converted_to_contact: Set(false),
        associated_deal_id: Set(None),
        converted_customer_id: Set(None),
        converted_contact_id: Set(None),
        company: Set(input.company),
        title: Set(input.title),
        lead_status: Set(input.lead_status.or(Some("New".to_string()))),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        tenant_id: Set(resolved_tenant_id),
        properties: Set(None),
        avatar_url: Set(input.avatar_url),
        ..Default::default()
    };

    if let Some(billing_address) = input.billing_address {
        new_lead.billing_address = Set(Some(billing_address));
    }

    if let Some(shipping_address) = input.shipping_address {
        new_lead.shipping_address = Set(Some(shipping_address));
    }

    let lead = new_lead.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(LeadModel::from(lead))))
}

pub async fn ingest_lead(
    Extension(db): Extension<DatabaseConnection>,
    site_config_opt: Option<Extension<crate::config::site_config::SiteConfig>>,
    headers: HeaderMap,
    Json(input): Json<CreateLeadInput>,
) -> Result<impl IntoResponse, StatusCode> {
    // 1. Honeypot/Spam check
    if let Some(bot_val) = &input._bot_check {
        if !bot_val.is_empty() {
            tracing::warn!("Honeypot triggered on lead ingestion");
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // 2. IP Rate Limiting (same pattern as create_lead)
    if let Some(ip) = headers.get("x-forwarded-for").and_then(|h| h.to_str().ok()).or_else(|| headers.get("x-real-ip").and_then(|h| h.to_str().ok())) {
        let ip = ip.split(',').next().unwrap_or("").trim().to_string();
        if !ip.is_empty() {
            let mut entry = RATE_LIMITER.entry(ip).or_insert((0, Instant::now()));
            if entry.value().1.elapsed() > Duration::from_secs(60) {
                entry.value_mut().0 = 1;
                entry.value_mut().1 = Instant::now();
            } else {
                entry.value_mut().0 += 1;
                if entry.value().0 > 3 {
                    tracing::warn!("Rate limit exceeded for lead ingestions from IP");
                    return Err(StatusCode::TOO_MANY_REQUESTS);
                }
            }
        }
    }

    // 3. Deduplication & Exclusivity (30 days)
    let mut cond = sea_orm::Condition::any();
    let mut has_contact_info = false;
    
    if let Some(ref email) = input.email {
        if !email.is_empty() {
            cond = cond.add(lead::Column::Email.eq(email.clone()));
            has_contact_info = true;
        }
    }
    if let Some(ref phone) = input.phone {
        if !phone.is_empty() {
            cond = cond.add(lead::Column::Phone.eq(phone.clone()));
            has_contact_info = true;
        }
    }

    if has_contact_info {
        let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
        let existing = lead::Entity::find()
            .filter(cond)
            .filter(lead::Column::CreatedAt.gte(thirty_days_ago))
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if existing.is_some() {
            tracing::warn!("Lead exclusivity check failed: Duplicate found within 30 days.");
            return Err(StatusCode::CONFLICT);
        }
    }

    // 4. Matchmaking & Geographic Routing
    let mut resolved_account_id = None;
    if let Some(listing_id) = input.listing_id {
        if let Ok(Some(lst)) = listing::Entity::find_by_id(listing_id).one(&db).await {
            if let Ok(Some(profile)) = crate::entities::profile::Entity::find_by_id(lst.profile_id).one(&db).await {
                resolved_account_id = Some(profile.account_id);
            }
        }
    }

    if resolved_account_id.is_none() {
        // Try dynamic matchmaking based on zip code
        let target_zip = input.shipping_address.as_ref().and_then(|a| a.0.postal_code.clone())
            .or_else(|| input.billing_address.as_ref().and_then(|a| a.0.postal_code.clone()));
            
        if let Some(zip) = target_zip {
            // Find a profile that covers this zip code
            let profiles = crate::entities::profile::Entity::find()
                .filter(crate::entities::profile::Column::IsActive.eq(true))
                .all(&db)
                .await
                .unwrap_or_default();
            
            // In-memory filter for now (ideally this is a Postgres ArrayContains query)
            let matched_profile = profiles.into_iter().find(|p| {
                if let Some(ref zips) = p.service_area_zips {
                    zips.contains(&zip)
                } else {
                    false
                }
            });
            
            if let Some(p) = matched_profile {
                resolved_account_id = Some(p.account_id);
            }
        }
    }
    
    if resolved_account_id.is_none() {
        if let Some(Extension(site_config)) = &site_config_opt {
            if let Ok(Some(primary_account)) = account::Entity::find()
                .filter(account::Column::TenantId.eq(site_config.tenant_id))
                .one(&db)
                .await 
            {
                resolved_account_id = Some(primary_account.id);
            } else {
                resolved_account_id = input.account_id;
            }
        } else {
            resolved_account_id = input.account_id;
        }
    }

    let mut resolved_tenant_id = None;
    if let Some(Extension(site_config)) = site_config_opt {
        resolved_tenant_id = Some(site_config.tenant_id);
    }

    let mut new_lead = lead::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(input.name.clone()),
        listing_id: Set(input.listing_id),
        account_id: Set(resolved_account_id),
        first_name: Set(input.first_name.clone()),
        last_name: Set(input.last_name.clone()),
        email: Set(input.email.clone()),
        phone: Set(input.phone.clone()),
        whatsapp: Set(input.whatsapp.clone()),
        telegram: Set(input.telegram.clone()),
        twitter: Set(input.twitter.clone()),
        instagram: Set(input.instagram.clone()),
        facebook: Set(input.facebook.clone()),
        message: Set(input.message.clone()),
        source: Set(input.source.clone().or_else(|| Some("API Ingestion".to_string()))),
        company: Set(input.company.clone()),
        title: Set(input.title.clone()),
        lead_status: Set(input.lead_status.clone().or_else(|| Some("New".to_string()))),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        tenant_id: Set(resolved_tenant_id),
        avatar_url: Set(input.avatar_url.clone()),
        ..Default::default()
    };

    if let Some(ref billing_address) = input.billing_address {
        new_lead.billing_address = Set(Some(billing_address.clone()));
    }
    if let Some(ref shipping_address) = input.shipping_address {
        new_lead.shipping_address = Set(Some(shipping_address.clone()));
    }

    let legacy_lead = new_lead.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to save ingested lead: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // === HANDLER CUTOVER: Dual-write to new unified model ===
    if let Some(tid) = resolved_tenant_id {
        if let Ok(account_id) = AccountService::find_or_create_tenant_account(&db, tid, "tenant").await {
            // Land the lead as a proper Opportunity in the new model
            let _ = OpportunityService::create_opportunity(
                &db, tid, "lead", &legacy_lead.name, Some(account_id), None, Some(25), Some("new")
            ).await;

            // Record via unified ledger (preferred path going forward)
            let _ = ledger::record_lead_purchase(&db, tid, account_id, legacy_lead.id, 5000, Some("stripe")).await;
        }
    }

    // Legacy billing path kept during transition
    if let Some(acct_id) = legacy_lead.account_id {
        let db_clone = db.clone();
        let l_id = legacy_lead.id;
        let tenant_for_billing = resolved_tenant_id;
        
        tokio::spawn(async move {
            let account_res = crate::entities::account::Entity::find_by_id(acct_id).one(&db_clone).await;
            if let Ok(Some(acct)) = account_res {
                if let Some(tid) = tenant_for_billing {
                    let _ = crate::services::lead_billing::charge_for_lead(&db_clone, tid, acct_id, l_id, acct.stripe_customer_id).await;
                }
            }
        });
    }
    
    Ok((StatusCode::CREATED, JsonResponse(LeadModel::from(legacy_lead))))
}

pub async fn get_leads(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_accounts = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();
    
    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
    let user_tenant_id = profile.tenant_id;

    let leads = lead::Entity::find()
        .filter(lead::Column::TenantId.eq(user_tenant_id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let lead_models: Vec<LeadModel> = leads.into_iter().map(LeadModel::from).collect();
    Ok(JsonResponse(lead_models))
}

pub async fn get_lead(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_accounts = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();
    
    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
    let user_tenant_id = profile.tenant_id;

    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if lead.tenant_id != Some(user_tenant_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(JsonResponse(LeadModel::from(lead)))
}

pub async fn update_lead(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateLeadInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_accounts = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();
    
    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
    let user_tenant_id = profile.tenant_id;

    let existing_lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if existing_lead.tenant_id != Some(user_tenant_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut lead: lead::ActiveModel = existing_lead.into();

    if let Some(name) = input.name {
        lead.name = Set(name);
    }

    if let Some(listing_id) = input.listing_id {
        lead.listing_id = Set(Some(listing_id));
    }

    if let Some(account_id) = input.account_id {
        lead.account_id = Set(Some(account_id));
    }

    if let Some(first_name) = input.first_name {
        lead.first_name = Set(Some(first_name));
    }

    if let Some(last_name) = input.last_name {
        lead.last_name = Set(Some(last_name));
    }

    if let Some(email) = input.email {
        lead.email = Set(Some(email));
    }

    if let Some(phone) = input.phone {
        lead.phone = Set(Some(phone));
    }

    if let Some(whatsapp) = input.whatsapp {
        lead.whatsapp = Set(Some(whatsapp));
    }

    if let Some(telegram) = input.telegram {
        lead.telegram = Set(Some(telegram));
    }

    if let Some(twitter) = input.twitter {
        lead.twitter = Set(Some(twitter));
    }

    if let Some(instagram) = input.instagram {
        lead.instagram = Set(Some(instagram));
    }

    if let Some(facebook) = input.facebook {
        lead.facebook = Set(Some(facebook));
    }

    if let Some(billing_address) = input.billing_address {
        lead.billing_address = Set(Some(billing_address));
    }

    if let Some(shipping_address) = input.shipping_address {
        lead.shipping_address = Set(Some(shipping_address));
    }

    if let Some(message) = input.message {
        lead.message = Set(Some(message));
    }

    if let Some(source) = input.source {
        lead.source = Set(Some(source));
    }

    if let Some(is_converted) = input.is_converted {
        lead.is_converted = Set(is_converted);
    }

    if let Some(converted_to_contact) = input.converted_to_contact {
        lead.converted_to_contact = Set(converted_to_contact);
    }

    if let Some(associated_deal_id) = input.associated_deal_id {
        lead.associated_deal_id = Set(Some(associated_deal_id));
    }

    if let Some(converted_customer_id) = input.converted_customer_id {
        lead.converted_customer_id = Set(Some(converted_customer_id));
    }

    if let Some(converted_contact_id) = input.converted_contact_id {
        lead.converted_contact_id = Set(Some(converted_contact_id));
    }

    if let Some(company) = input.company {
        lead.company = Set(Some(company));
    }

    if let Some(title) = input.title {
        lead.title = Set(Some(title));
    }

    if let Some(lead_status) = input.lead_status {
        lead.lead_status = Set(Some(lead_status));
    }
    if let Some(avatar_url) = input.avatar_url {
        lead.avatar_url = Set(Some(avatar_url));
    }

    let updated_lead = lead.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok(JsonResponse(LeadModel::from(updated_lead)))
}

pub async fn delete_lead(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_accounts = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();
    
    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
    let user_tenant_id = profile.tenant_id;

    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if lead.tenant_id != Some(user_tenant_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    lead.delete(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_file_to_lead(
    Extension(db): Extension<DatabaseConnection>,
    Path((lead_id, file_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(lead_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch lead: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    lead.add_file(&db, file_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to add file to lead: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

pub async fn get_lead_files(
    Extension(db): Extension<DatabaseConnection>,
    Path(lead_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(lead_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch lead: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let file_ids = lead.get_associated_files(&db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get associated files: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(file_ids))
}

pub async fn get_lead_notes(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let notes = note::Entity::find()
        .filter(note::Column::EntityType.eq("Lead"))
        .filter(note::Column::EntityId.eq(lead.id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let note_models: Vec<NoteModel> = notes.into_iter().map(NoteModel::from).collect();
    Ok(JsonResponse(note_models))
}

pub async fn get_lead_activities(
    Extension(db): Extension<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let activities = activity::Entity::find()
        .filter(activity::Column::LeadId.eq(Some(lead.id)))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let activity_models: Vec<ActivityModel> = activities.into_iter().map(ActivityModel::from).collect();
    Ok(JsonResponse(activity_models))
}

use crate::handlers::notes::get_user_tenant_id;

pub async fn create_lead_note(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateNoteInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = get_user_tenant_id(&db, current_user.id).await?;

    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_note = note::ActiveModel {
        id: Set(Uuid::new_v4()),
        content: Set(input.content),
        created_by: Set(current_user.id),
        entity_type: Set("Lead".to_string()),
        entity_id: Set(lead.id),
        tenant_id: Set(Some(tenant_id)),
        is_private: Set(false),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let inserted_note = new_note.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(NoteModel::from(inserted_note))))
}

pub async fn create_lead_activity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateActivityInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let lead = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let new_activity = activity::ActiveModel {
        id: Set(Uuid::new_v4()),
        lead_id: Set(Some(lead.id)),
        activity_type: Set(input.activity_type),
        title: Set(input.title),
        description: Set(input.description),
        status: Set(input.status),
        due_date: Set(input.due_date),
        completed_at: Set(None),
        created_by: Set(current_user.id),
        assigned_to: Set(input.assigned_to),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        ..Default::default()
    };

    let inserted_activity = new_activity.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    Ok((StatusCode::CREATED, JsonResponse(ActivityModel::from(inserted_activity))))
}

pub async fn convert_lead(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_accounts = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(current_user.id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();
    
    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;
    let user_tenant_id = profile.tenant_id;

    let lead_model = lead::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if lead_model.tenant_id != Some(user_tenant_id) {
        return Err(StatusCode::FORBIDDEN);
    }

    if lead_model.is_converted {
        return Err(StatusCode::BAD_REQUEST);
    }

    let txn = db.begin().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut duplicate_contact = None;
    if let Some(ref email) = lead_model.email {
        if !email.is_empty() {
            let dup = contact::Entity::find()
                .filter(contact::Column::TenantId.eq(user_tenant_id))
                .filter(contact::Column::Email.eq(email.clone()))
                .one(&txn)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if dup.is_some() {
                duplicate_contact = dup;
            }
        }
    }

    if duplicate_contact.is_none() {
        if let Some(ref phone) = lead_model.phone {
            if !phone.is_empty() {
                let dup = contact::Entity::find()
                    .filter(contact::Column::TenantId.eq(user_tenant_id))
                    .filter(contact::Column::Phone.eq(phone.clone()))
                    .one(&txn)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                if dup.is_some() {
                    duplicate_contact = dup;
                }
            }
        }
    }

    // === CUTOVER: Also create canonical Account + Contact in the new model ===
    let contact_id = if let Some(contact) = duplicate_contact {
        contact.id
    } else {
        let new_contact_id = Uuid::new_v4();
        // Dual write to legacy for now
        let new_contact = contact::ActiveModel {
            id: Set(new_contact_id),
            customer_id: Set(None),
            name: Set(lead_model.name.clone()),
            first_name: Set(lead_model.first_name.clone()),
            last_name: Set(lead_model.last_name.clone()),
            email: Set(lead_model.email.clone()),
            phone: Set(lead_model.phone.clone()),
            // ... other fields abbreviated for cutover
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            tenant_id: Set(Some(user_tenant_id)),
            ..Default::default()
        };
        new_contact.insert(&txn).await.map_err(|e| {
            tracing::error!("Failed to insert contact on lead conversion: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // New unified path (note: using &db during txn for service compatibility in cutover phase)
        if let Ok(acc_id) = AccountService::find_or_create_tenant_account(&db, user_tenant_id, "tenant").await {
            let _ = ContactService::create_contact(
                &db,
                user_tenant_id,
                acc_id,
                lead_model.first_name.as_deref(),
                lead_model.last_name.as_deref(),
                lead_model.email.as_deref(),
                false,
            ).await;
        }

        new_contact_id
    };

    let mut active_lead: lead::ActiveModel = lead_model.into();
    active_lead.is_converted = Set(true);
    active_lead.converted_to_contact = Set(true);
    active_lead.converted_contact_id = Set(Some(contact_id));
    active_lead.lead_status = Set(Some("Converted".to_string()));
    active_lead.updated_at = Set(Utc::now());

    let updated = active_lead.update(&txn).await.map_err(|e| {
        tracing::error!("Failed to update lead status on lead conversion: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    txn.commit().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(JsonResponse(LeadModel::from(updated)))
}


