use axum::{
    extract::{Extension, Path, Json, Query},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::{
    DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
    ColumnTrait, ActiveModelTrait, Order,
};
use crate::entities::atlas_account;
use crate::types::account::{AccountType, AccountStatus};
use uuid::Uuid;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

/// Public (non-admin) account routes used by network-instance and other internal apps.
/// Admin-facing routes live in `admin/routes.rs` → `admin_routes_raw()`.
pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/accounts",      get(get_accounts).post(create_account))
        .route("/api/accounts/{id}", get(get_account).put(update_account).delete(delete_account))
        .route("/api/accounts/{id}/users",               get(get_account_users).post(add_user_to_account))
        .route("/api/accounts/{id}/ledger",              get(get_account_ledger))
        .route("/api/accounts/{account_id}/users/{user_id}",       delete(remove_user_from_account))
        .route("/api/accounts/{account_id}/users/{user_id}/role",  put(update_user_role_in_account))
}


// ── DTOs ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize, Clone)]
pub struct CreateAccountDto {
    pub name: String,
    #[serde(default)]
    pub account_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Deserialize, Clone)]
pub struct UpdateAccountDto {
    pub name: Option<String>,
    pub account_type: Option<String>,
    pub status: Option<String>,
    pub dba_name: Option<String>,
    pub website: Option<String>,
    pub domain: Option<String>,
    pub company_phone: Option<String>,
    pub company_email: Option<String>,
    pub industry: Option<String>,
    pub company_type: Option<String>,
    pub num_employees: Option<i32>,
    pub annual_revenue: Option<Decimal>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub street_address: Option<String>,
    pub postal_code: Option<String>,
    pub year_established: Option<i16>,
    pub data_source: Option<String>,
}

/// Query params for `GET /api/admin/accounts`
#[derive(Deserialize, Default)]
pub struct AccountListParams {
    pub search:       Option<String>,
    /// Filter by status discriminant: "active" | "prospect" | "suspended" | "archived"
    pub status:       Option<String>,
    /// Filter by type discriminant: "organization" | "individual"
    pub account_type: Option<String>,
    pub page:         Option<u64>,
    pub per_page:     Option<u64>,
}

impl AccountListParams {
    pub fn offset(&self) -> u64 {
        (self.page.unwrap_or(1).max(1) - 1) * self.limit()
    }
    pub fn limit(&self) -> u64 { self.per_page.unwrap_or(50).min(200).max(1) }
}

/// Rich account response — sourced from `atlas_accounts`.
///
/// `account_type` and `status` use the canonical enums from `crate::types::account`
/// so JSON serialises as `"organization"` / `"individual"` / `"active"` / etc.
/// The frontend mirrors these with identically-serialised enums in `api/models.rs`.
#[derive(Serialize, Clone)]
pub struct AccountResponse {
    pub id:               String,
    pub name:             String,
    /// Typed discriminant — serialises as snake_case string.
    pub account_type:     AccountType,
    /// Typed lifecycle status — serialises as snake_case string.
    pub status:           AccountStatus,

    // Identity
    pub dba_name:         Option<String>,
    pub website:          Option<String>,
    pub domain:           Option<String>,

    // Contact channels
    pub company_phone:    Option<String>,
    pub company_email:    Option<String>,

    // Firmographics
    pub industry:         Option<String>,
    pub company_type:     Option<String>,
    pub num_employees:    Option<i32>,
    pub annual_revenue:   Option<Decimal>,
    pub year_established: Option<i16>,

    // Address
    pub street_address:   Option<String>,
    pub city:             Option<String>,
    pub state:            Option<String>,
    pub postal_code:      Option<String>,
    pub country:          Option<String>,

    // Import
    pub data_source:      Option<String>,

    // Timestamps
    pub created_at:       String,
    pub updated_at:       String,
}

impl From<atlas_account::Model> for AccountResponse {
    fn from(m: atlas_account::Model) -> Self {
        // Parse discriminants — fall back to safe defaults if DB has unexpected value
        let account_type = AccountType::try_from(m.account_type.as_str())
            .unwrap_or(AccountType::Organization);
        let status = AccountStatus::try_from(m.status.as_str())
            .unwrap_or(AccountStatus::Active);

        Self {
            id:               m.id.to_string(),
            name:             m.name,
            account_type,
            status,
            dba_name:         m.dba_name,
            website:          m.website,
            domain:           m.domain,
            company_phone:    m.company_phone,
            company_email:    m.company_email,
            industry:         m.industry,
            company_type:     m.company_type,
            num_employees:    m.num_employees,
            annual_revenue:   m.annual_revenue,
            year_established: m.year_established,
            street_address:   m.street_address,
            city:             m.city,
            state:            m.state,
            postal_code:      m.postal_code,
            country:          m.country,
            data_source:      m.data_source,
            created_at:       m.created_at.to_rfc3339(),
            updated_at:       m.updated_at.to_rfc3339(),
        }
    }
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn get_accounts(
    Extension(db): Extension<DatabaseConnection>,
    Query(params): Query<AccountListParams>,
) -> impl IntoResponse {
    tracing::info!(
        "Fetching atlas_accounts (search={:?} status={:?} type={:?} page={:?})",
        params.search, params.status, params.account_type, params.page
    );

    let mut query = atlas_account::Entity::find()
        .filter(atlas_account::Column::IsDuplicate.eq(false))
        .order_by(atlas_account::Column::Name, Order::Asc);

    // Apply status filter using the typed enum
    if let Some(ref status_str) = params.status {
        if let Ok(status) = AccountStatus::try_from(status_str.as_str()) {
            query = query.filter(atlas_account::Column::Status.eq(status.to_string()));
        }
    }

    // Apply account_type filter using the typed enum
    if let Some(ref type_str) = params.account_type {
        if let Ok(acc_type) = AccountType::try_from(type_str.as_str()) {
            query = query.filter(atlas_account::Column::AccountType.eq(acc_type.to_string()));
        }
    }

    let all: Vec<atlas_account::Model> = match query.all(&db).await {
        Ok(v)  => v,
        Err(e) => {
            tracing::error!("Error fetching atlas_accounts: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(Vec::<AccountResponse>::new()));
        }
    };

    // In-memory search on name/domain/industry (avoids ILIKE without pg_trgm)
    let filtered: Vec<atlas_account::Model> = if let Some(ref term) = params.search {
        if term.is_empty() {
            all
        } else {
            let t = term.to_lowercase();
            all.into_iter().filter(|a| {
                a.name.to_lowercase().contains(&t)
                    || a.domain.as_deref().map(|d| d.to_lowercase().contains(&t)).unwrap_or(false)
                    || a.industry.as_deref().map(|i| i.to_lowercase().contains(&t)).unwrap_or(false)
                    || a.company_email.as_deref().map(|e| e.to_lowercase().contains(&t)).unwrap_or(false)
            }).collect()
        }
    } else {
        all
    };

    let offset = params.offset() as usize;
    let limit  = params.limit()  as usize;
    let page: Vec<AccountResponse> = filtered
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(AccountResponse::from)
        .collect();

    (StatusCode::OK, JsonResponse(page))
}

pub async fn get_account(
    Extension(db): Extension<DatabaseConnection>,
    Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!("Fetching atlas_account: {}", account_id);

    match atlas_account::Entity::find_by_id(account_id).one(&db).await {
        Ok(Some(account)) => (StatusCode::OK, JsonResponse(Some(AccountResponse::from(account)))),
        Ok(None)          => (StatusCode::NOT_FOUND, JsonResponse(None)),
        Err(err) => {
            tracing::error!("Error fetching atlas_account: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(None))
        }
    }
}

pub async fn create_account(
    Extension(db): Extension<DatabaseConnection>,
    Json(payload): Json<CreateAccountDto>,
) -> impl IntoResponse {
    tracing::info!("Creating atlas_account: {}", payload.name);

    let new_account = atlas_account::ActiveModel {
        id:           Set(Uuid::new_v4()),
        tenant_id:    Set(Uuid::nil()),
        name:         Set(payload.name.trim().to_string()),
        // Parse incoming type/status strings through enums for validation
        account_type: Set(payload.account_type
            .as_deref()
            .and_then(|s| AccountType::try_from(s).ok())
            .unwrap_or(AccountType::Organization)
            .to_string()),
        status:       Set(payload.status
            .as_deref()
            .and_then(|s| AccountStatus::try_from(s).ok())
            .unwrap_or(AccountStatus::Active)
            .to_string()),
        is_duplicate: Set(false),
        created_at:   Set(Utc::now()),
        updated_at:   Set(Utc::now()),
        ..Default::default()
    };

    match new_account.insert(&db).await {
        Ok(account) => (StatusCode::CREATED, JsonResponse(Some(AccountResponse::from(account)))),
        Err(err) => {
            tracing::error!("Error creating atlas_account: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(None::<AccountResponse>))
        }
    }
}

pub async fn update_account(
    Extension(db): Extension<DatabaseConnection>,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<UpdateAccountDto>,
) -> impl IntoResponse {
    tracing::info!("Updating atlas_account: {}", account_id);

    let existing = match atlas_account::Entity::find_by_id(account_id).one(&db).await {
        Ok(Some(a)) => a,
        Ok(None)    => return (StatusCode::NOT_FOUND, JsonResponse(None::<AccountResponse>)),
        Err(err) => {
            tracing::error!("Error fetching atlas_account for update: {:?}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(None));
        }
    };

    let mut active: atlas_account::ActiveModel = existing.into();

    if let Some(v) = payload.name             { active.name             = Set(v); }
    // Validate account_type and status strings via enums before writing
    if let Some(ref v) = payload.account_type {
        active.account_type = Set(AccountType::try_from(v.as_str())
            .map(|t| t.to_string())
            .unwrap_or_else(|_| "organization".to_string()));
    }
    if let Some(ref v) = payload.status {
        active.status = Set(AccountStatus::try_from(v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|_| "active".to_string()));
    }
    if let Some(v) = payload.dba_name         { active.dba_name         = Set(Some(v)); }
    if let Some(v) = payload.website          { active.website          = Set(Some(v)); }
    if let Some(v) = payload.domain           { active.domain           = Set(Some(v)); }
    if let Some(v) = payload.company_phone    { active.company_phone    = Set(Some(v)); }
    if let Some(v) = payload.company_email    { active.company_email    = Set(Some(v)); }
    if let Some(v) = payload.industry         { active.industry         = Set(Some(v)); }
    if let Some(v) = payload.company_type     { active.company_type     = Set(Some(v)); }
    if let Some(v) = payload.num_employees    { active.num_employees    = Set(Some(v)); }
    if let Some(v) = payload.annual_revenue   { active.annual_revenue   = Set(Some(v)); }
    if let Some(v) = payload.city             { active.city             = Set(Some(v)); }
    if let Some(v) = payload.state            { active.state            = Set(Some(v)); }
    if let Some(v) = payload.country          { active.country          = Set(Some(v)); }
    if let Some(v) = payload.street_address   { active.street_address   = Set(Some(v)); }
    if let Some(v) = payload.postal_code      { active.postal_code      = Set(Some(v)); }
    if let Some(v) = payload.year_established { active.year_established = Set(Some(v)); }
    if let Some(v) = payload.data_source      { active.data_source      = Set(Some(v)); }
    active.updated_at = Set(Utc::now());

    match active.update(&db).await {
        Ok(updated) => (StatusCode::OK, JsonResponse(Some(AccountResponse::from(updated)))),
        Err(err) => {
            tracing::error!("Error updating atlas_account: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, JsonResponse(None))
        }
    }
}

pub async fn delete_account(
    Extension(db): Extension<DatabaseConnection>,
    Path(account_id): Path<Uuid>,
) -> impl IntoResponse {
    tracing::info!("Deleting atlas_account: {}", account_id);

    match atlas_account::Entity::delete_by_id(account_id).exec(&db).await {
        Ok(_)    => StatusCode::NO_CONTENT,
        Err(err) => {
            tracing::error!("Error deleting atlas_account: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// Legacy stubs kept for routes that reference the old user_account join pattern
pub async fn add_user_to_account() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
pub async fn get_account_users() -> StatusCode   { StatusCode::NOT_IMPLEMENTED }
pub async fn get_account_ledger() -> StatusCode  { StatusCode::NOT_IMPLEMENTED }
pub async fn remove_user_from_account() -> StatusCode { StatusCode::NOT_IMPLEMENTED }
pub async fn update_user_role_in_account() -> StatusCode { StatusCode::NOT_IMPLEMENTED }