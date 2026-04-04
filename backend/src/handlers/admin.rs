use crate::entities::{
    user,user_account, tenant, listing, ad_purchase, 
    profile, account, session,request_log,
    template, category
};
use axum::{
    Router,
    extract::{Extension, Json, Path, State, Query},
    routing::get,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sea_orm::{DatabaseConnection, EntityTrait,QuerySelect, QueryFilter,Order, ColumnTrait,QueryOrder, Set, ActiveModelTrait, PaginatorTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::listing::ListingStatus;
use crate::models::user::UserAdminView;
use crate::models::ad_purchase::AdStatus;

use std::collections::HashMap;
use crate::handlers::{listings,sessions};
use crate::models::session::{SessionResponse, UserInfo};

#[derive(Deserialize)]
pub struct UpdateUserInput {
    username: Option<String>,
    email: Option<String>,
}
#[derive(Serialize)]
pub struct DirectoryStats {
    tenant_id: Uuid,
    name: String,
    profile_count: u64,
    listing_count: u64,
    ad_purchase_count: u64,
}

#[derive(Serialize)]
pub struct AdPurchaseStats {
    total_purchases: i64,
    active_purchases: i64,
    total_revenue: f64,
}

#[derive(Serialize)]
pub struct UserStatistics {
    total_users: i64,
    active_users: i64,
    total_admins: i64,
}

#[derive(Serialize)]
pub struct AccountStatistics {
    total_accounts: i64,
    active_accounts: i64,
}

#[derive(Serialize)]
pub struct ListingStats {
    total_listings: i64,
    active_listings: i64,
}

#[derive(Serialize)]
pub struct ActivityReport {
    recent_listings: Vec<listing::Model>,
    recent_ad_purchases: Vec<ad_purchase::Model>,
    recent_profiles: Vec<profile::Model>,
    recent_users: Vec<user::Model>,
    recent_accounts: Vec<account::Model>,
}





pub async fn get_directory_listings(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    // extension database connection and query params
    let extension_db = Extension(db);
    let query_params = Query(HashMap::new());

    let listings = listings::get_listings(extension_db, query_params).await?;
    Ok(listings)
}

pub async fn get_listing(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Path((listing_id)): Path<(Uuid)>,
) -> Result<impl IntoResponse, StatusCode> {
    let extension_db = Extension(db);
    let path = Path((listing_id));

    let listing = listings::get_listing_by_id(extension_db, path).await?;
    Ok(listing)
}

pub async fn list_users(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Query(query): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Listing users via admin route");

    let mut user_ids_to_filter: Option<Vec<Uuid>> = None;

    if let Some(dir_id_str) = query.get("tenant_id") {
        if let Ok(dir_id) = Uuid::parse_str(dir_id_str) {
                let profiles = profile::Entity::find()
                    .filter(profile::Column::TenantId.eq(dir_id))
                    .all(&db)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                
                let account_ids: Vec<Uuid> = profiles.into_iter().map(|p| p.account_id).collect();
                
                let user_accounts = user_account::Entity::find()
                    .filter(user_account::Column::AccountId.is_in(account_ids))
                    .all(&db)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    
                let user_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.user_id).collect();
                user_ids_to_filter = Some(user_ids);
            }
        }

    let mut find_query = user::Entity::find();
    if let Some(ids) = user_ids_to_filter {
        // If there are no users in that directory, early return an empty list 
        // because `is_in([])` might fetch all or fail depending on the SQL builder, 
        // though `sea-orm` usually handles it safely. We can just be explicit:
        if ids.is_empty() {
            return Ok(Json(Vec::<user::Model>::new()));
        }
        find_query = find_query.filter(user::Column::Id.is_in(ids));
    }

    let users = find_query
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users))
}

pub async fn get_user(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Getting user via admin route");
    let user = user::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    
    let mut user_admin_view = UserAdminView{
        user: user.clone(),
        user_accounts: Vec::new(),
        profiles: Vec::new(),
        directories: Vec::new(),
        login_history: Vec::new(),
    };
    
    let user_accounts = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(user_id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    user_admin_view.user_accounts = user_accounts.clone();
    user_admin_view.user = user.clone();
    let user_accounts_ids: Vec<Uuid> = user_accounts.iter().map(|user_account| user_account.account_id).collect();
    let profiles = profile::Entity::find()
        .filter(profile::Column::AccountId.is_in(user_accounts_ids.clone()))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    user_admin_view.profiles = profiles.clone();
    let directories = tenant::Entity::find()
        .filter(tenant::Column::Id.is_in(profiles.into_iter().map(|profile| profile.tenant_id)))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    //filter by request type of LOGIN
    let login_history = request_log::Entity::find()
        .filter(request_log::Column::UserId.eq(user.id))
        .filter(request_log::Column::RequestType.eq("LOGIN".to_string()))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    user_admin_view.directories = directories;
    user_admin_view.login_history = login_history;

    Ok(Json(user_admin_view))
}

pub async fn update_user(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Path(user_id): Path<Uuid>,
    Json(input): Json<UpdateUserInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut user: user::ActiveModel = user::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    if let Some(username) = input.username {
        user.username = Set(username);
    }
    if let Some(email) = input.email {
        user.email = Set(email);
    }

    let updated_user = user.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated_user))
}

pub async fn delete_user(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {

    user::Entity::delete_by_id(user_id)
        .exec(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn toggle_admin(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {

    let mut user: user::ActiveModel = user::Entity::find_by_id(user_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    user.is_admin = Set(!user.is_admin.unwrap());

    let updated_user = user.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated_user))
}

pub async fn get_all_directory_stats(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {

    let directories = tenant::Entity::find().all(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut stats = Vec::new();
    for dir in directories {
        let profile_count = profile::Entity::find()
            .filter(profile::Column::TenantId.eq(dir.id))
            .count(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let listing_count = listing::Entity::find()
            .filter(listing::Column::TenantId.eq(dir.id))
            .count(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let ad_purchase_count = ad_purchase::Entity::find()
            .inner_join(profile::Entity)
            .filter(profile::Column::TenantId.eq(dir.id))
            .count(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        stats.push(DirectoryStats {
            tenant_id: dir.id,
            name: dir.name,
            profile_count,
            listing_count,
            ad_purchase_count,
        });
    }

    Ok(Json(stats))
}

pub async fn get_directory_stats(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Path(tenant_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {

    let directory = tenant::Entity::find_by_id(tenant_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let profile_count = profile::Entity::find()
        .filter(profile::Column::TenantId.eq(tenant_id))
        .count(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let listing_count = listing::Entity::find()
        .filter(listing::Column::TenantId.eq(tenant_id))
        .count(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;


    // filter by profile.directory id
    let ad_purchase_count = ad_purchase::Entity::find()
        .inner_join(profile::Entity)
        .filter(profile::Column::TenantId.eq(tenant_id))
        .count(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats = DirectoryStats {
        tenant_id: directory.id,
        name: directory.name,
        profile_count,
        listing_count,
        ad_purchase_count,
    };

    Ok(Json(stats))
}

pub async fn list_pending_listings(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {

    let pending_listings = listing::Entity::find()
        .filter(listing::Column::Status.eq(ListingStatus::Pending))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(pending_listings))
}

pub async fn approve_listing(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Path(listing_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {

    let mut listing: listing::ActiveModel = listing::Entity::find_by_id(listing_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    listing.status = Set(ListingStatus::Approved);

    let updated_listing = listing.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated_listing))
}

pub async fn reject_listing(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Path(listing_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut listing: listing::ActiveModel = listing::Entity::find_by_id(listing_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    listing.status = Set(ListingStatus::Rejected);

    let updated_listing = listing.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated_listing))
}

pub async fn get_ad_purchase_stats(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {

    let total_purchases = ad_purchase::Entity::find().count(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let active_purchases = ad_purchase::Entity::find()
        .filter(ad_purchase::Column::Status.eq(AdStatus::Active))
        .count(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_revenue = ad_purchase::Entity::find()
        .filter(ad_purchase::Column::Status.eq(AdStatus::Active))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .iter()
        .fold(0.0, |acc, purchase| acc + purchase.price);

    let stats = AdPurchaseStats {
        total_purchases: total_purchases.try_into().unwrap(),
        active_purchases: active_purchases.try_into().unwrap(),
        total_revenue: total_revenue.try_into().unwrap(),
    };

    Ok(Json(stats))
}

pub async fn list_active_ad_purchases(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {

    let active_purchases = ad_purchase::Entity::find()
        .filter(ad_purchase::Column::Status.eq(AdStatus::Active))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(active_purchases))
}

pub async fn get_ad_purchase(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Path(purchase_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {

    let purchase = ad_purchase::Entity::find_by_id(purchase_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(purchase))
}

pub async fn list_ad_purchases(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    tracing::info!("Attempting to list ad purchases for user: {:?}", current_user.id);

    match ad_purchase::Entity::find().all(&db).await {
        Ok(purchases) => {
            tracing::info!("Successfully fetched {} ad purchases", purchases.len());
            Ok(Json(purchases))
        },
        Err(e) => {
            tracing::error!("Failed to fetch ad purchases: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
pub async fn cancel_ad_purchase(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
    Path(purchase_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {

    let mut purchase: ad_purchase::ActiveModel = ad_purchase::Entity::find_by_id(purchase_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?
        .into();

    purchase.status = Set(AdStatus::Cancelled.to_string());

    let updated_purchase = purchase.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(updated_purchase))
}

pub async fn get_user_statistics(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {

    let total_users = user::Entity::find().count(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let active_users = user::Entity::find()
        .filter(user::Column::IsActive.eq(true))
        .count(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_admins = user::Entity::find()
        .filter(user::Column::IsAdmin.eq(true))
        .count(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats = UserStatistics {
        total_users: total_users.try_into().unwrap(),
        active_users: active_users.try_into().unwrap(),
        total_admins: total_admins.try_into().unwrap(),
    };

    Ok(Json(stats))
}

pub async fn get_account_statistics(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {

    let total_accounts = account::Entity::find().count(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let active_accounts = account::Entity::find()
        .filter(account::Column::IsActive.eq(true))
        .count(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats = AccountStatistics {
        total_accounts: total_accounts.try_into().unwrap(),
        active_accounts: active_accounts.try_into().unwrap(),
    };

    Ok(Json(stats))
}

pub async fn get_listing_statistics(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {

    let total_listings = listing::Entity::find().count(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let active_listings = listing::Entity::find()
        .filter(listing::Column::Status.eq(ListingStatus::Approved))
        .count(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stats = ListingStats {
        total_listings: total_listings.try_into().unwrap(),
        active_listings: active_listings.try_into().unwrap(),
    };

    Ok(Json(stats))
}

pub async fn get_ad_purchase_statistics(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {

    let total_purchases = ad_purchase::Entity::find().count(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let active_purchases = ad_purchase::Entity::find()
        .filter(ad_purchase::Column::Status.eq(AdStatus::Active))
        .count(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_revenue = ad_purchase::Entity::find()
        .filter(ad_purchase::Column::Status.eq(AdStatus::Active))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .iter()
        .fold(0.0, |acc, purchase| acc + purchase.price);

    let stats = AdPurchaseStats {
        total_purchases: total_purchases.try_into().unwrap(),
        active_purchases: active_purchases.try_into().unwrap(),
        total_revenue: total_revenue.try_into().unwrap(),
    };

    Ok(Json(stats))
}

pub async fn get_activity_report(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let report;
    // Fetch recent activity data
    let recent_activities = {
        let recent_listings = listing::Entity::find()
            .order_by(listing::Column::CreatedAt, Order::Desc)
            .limit(5)
            .all(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;  

        let recent_ad_purchases = ad_purchase::Entity::find()
            .order_by(ad_purchase::Column::CreatedAt, Order::Desc)
            .limit(5)
            .all(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let recent_profiles = profile::Entity::find()
            .order_by(profile::Column::CreatedAt, Order::Desc)
            .limit(5)
            .all(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let recent_users = user::Entity::find()
            .order_by(user::Column::CreatedAt, Order::Desc)
            .limit(5)
            .all(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let recent_accounts = account::Entity::find()
            .order_by(account::Column::CreatedAt, Order::Desc)
            .limit(5)
            .all(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        report = ActivityReport {
            recent_listings,
            recent_ad_purchases,
            recent_profiles,
            recent_users,
            recent_accounts,
        };
    };
    

    Ok(Json(report))
}

pub async fn get_revenue_report(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(current_session): Extension<session::Model>,
) -> Result<impl IntoResponse, StatusCode> {

    // Fetch revenue data total ad purchases by month
    let revenue_data = {
        let ad_purchases = ad_purchase::Entity::find()
            .all(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut revenue_data: HashMap<String, f64> = HashMap::new();
        
        // ... populate revenue_data ...

        revenue_data
    };

    Ok(Json(revenue_data))
}

pub async fn impersonate_user(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Extension(_current_session): Extension<session::Model>,
    Path(target_user_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    
    // 1. Strictly verify the caller is a global platform admin
    if !current_user.is_admin {
        tracing::warn!("Non-admin user {} attempted to impersonate {}", current_user.id, target_user_id);
        return Err(StatusCode::FORBIDDEN);
    }

    // 2. Load the target user
    let target_user = user::Entity::find_by_id(target_user_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // 3. Generate an impersonation JWT containing the impersonator_id claim
    let token = crate::auth::generate_impersonation_jwt(&target_user, &current_user.id)
        .map_err(|e| {
            tracing::error!("Failed to generate impersonation token: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // 4. Return SessionResponse
    let response = SessionResponse {
        token,
        refresh_token: "".to_string(), // Refresh tokens are generally not needed for impersonation
        user: Some(UserInfo {
            id: target_user.id,
            email: target_user.email,
            first_name: target_user.first_name,
            last_name: target_user.last_name,
            is_admin: target_user.is_admin,
        }),
    };

    Ok(Json(response))
}