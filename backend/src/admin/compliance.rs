//! Admin — Compliance and Geo handler
//!
//! Manages municipal permits (G-16), contracts (G-11), and PostGIS spatial geo-zones (G-01).

use axum::{
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_contract, atlas_regulatory_registration, geo_service_area, tenant, user};

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/compliance/permits", get(list_permits).post(create_permit))
        .route("/api/admin/compliance/permits/{id}/verify", post(verify_permit))
        .route("/api/admin/compliance/geo-zones", get(list_geo_zones).post(create_geo_zone))
        .route("/api/admin/compliance/contracts", get(list_contracts).post(create_contract))
}

// ── Models ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermitResponse {
    pub id: Uuid,
    pub name: String,
    pub holder: String,
    pub license: String,
    pub permit_type: String,
    pub status: String,
    pub status_class: String,
    pub last_checked: String,
    pub date_renewed: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePermitInput {
    pub name: String,
    pub license: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeoZoneResponse {
    pub key: String,
    pub name: String,
    pub region: String,
    pub listings: String,
    pub status: String,
    pub status_class: String,
    pub coverage: String,
    pub points: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateGeoZoneInput {
    pub name: String,
    pub region: String,
    pub points: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContractResponse {
    pub id: String,
    pub name: String,
    pub signee: String,
    pub contract_type: String,
    pub status: String,
    pub status_class: String,
    pub date_executed: String,
    pub expiry_date: String,
    pub vault_file: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateContractInput {
    pub contract_type: String,
    pub signee_tenant_id: Option<Uuid>,
    pub start_date: String,   // "YYYY-MM-DD"
    pub end_date: Option<String>,
    pub vault_file: Option<String>,
}

// ── Helper functions for coordinate conversion ───────────────────────────────

fn svg_points_to_wkt(points_str: &str) -> Option<String> {
    let points: Vec<&str> = points_str.split_whitespace().collect();
    if points.len() < 3 {
        return None;
    }
    let mut wkt_points = Vec::new();
    for pt in &points {
        let coords: Vec<&str> = pt.split(',').collect();
        if coords.len() != 2 {
            return None;
        }
        wkt_points.push(format!("{} {}", coords[0], coords[1]));
    }
    // Close the polygon (WKT requires first and last point to be identical)
    let first_pt: Vec<&str> = points[0].split(',').collect();
    wkt_points.push(format!("{} {}", first_pt[0], first_pt[1]));

    Some(format!("MULTIPOLYGON((({})))", wkt_points.join(", ")))
}

fn wkt_to_svg_points(wkt: &str) -> String {
    let clean = wkt
        .replace("MULTIPOLYGON(((", "")
        .replace(")))", "")
        .replace("POLYGON((", "")
        .replace("))", "");
    let parts: Vec<&str> = clean.split(',').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return String::new();
    }
    
    let len = parts.len();
    let iter_len = if len > 1 && parts[0] == parts[len - 1] {
        len - 1
    } else {
        len
    };

    parts[0..iter_len]
        .iter()
        .map(|pt| {
            let coords: Vec<&str> = pt.split_whitespace().collect();
            if coords.len() == 2 {
                format!("{},{}", coords[0], coords[1])
            } else {
                pt.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn list_permits(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let list = atlas_regulatory_registration::Entity::find()
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch tenant names in one query for holder resolution
    let tenant_ids: Vec<Uuid> = list.iter().map(|r| r.tenant_id).collect();
    let tenants = tenant::Entity::find()
        .filter(tenant::Column::Id.is_in(tenant_ids))
        .all(&db)
        .await
        .unwrap_or_default();
    let tenant_map: std::collections::HashMap<Uuid, String> =
        tenants.into_iter().map(|t| (t.id, t.name)).collect();

    let response: Vec<PermitResponse> = list
        .into_iter()
        .map(|m| {
            let status = m.status.clone();
            let status_class = match status.as_str() {
                s if s.contains("Verified") || s.contains("Active") => "tag tag-ok",
                s if s.contains("Review") || s.contains("Pending") => "tag tag-warn",
                _ => "tag tag-err",
            };
            let holder = tenant_map
                .get(&m.tenant_id)
                .cloned()
                .unwrap_or_else(|| "Unknown Tenant".to_string());
            PermitResponse {
                id: m.id,
                name: m.jurisdiction_code.clone(),
                holder,
                license: m.registration_number.clone(),
                permit_type: m.registration_type.clone(),
                status,
                status_class: status_class.to_string(),
                last_checked: "Just now".to_string(),
                date_renewed: m.created_at.format("%b %d, %Y").to_string(),
            }
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_permit(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreatePermitInput>,
) -> Result<impl IntoResponse, StatusCode> {
    // Resolve tenant_id
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(current_user.id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tenant_id = if let Some(ua) = user_accounts.first() {
        // Find tenant by account_id
        let profile = crate::entities::profile::Entity::find()
            .filter(crate::entities::profile::Column::AccountId.eq(ua.account_id))
            .one(&db)
            .await
            .unwrap_or(None);
        profile.map(|p| p.tenant_id).unwrap_or_else(Uuid::new_v4)
    } else {
        Uuid::new_v4()
    };

    let id = Uuid::new_v4();
    let new_permit = atlas_regulatory_registration::ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        registration_type: Set("Short-term Rental".to_string()),
        jurisdiction_code: Set(input.name.clone()),
        registration_number: Set(input.license.clone()),
        status: Set("✓ Active".to_string()),
        created_at: Set(Utc::now()),
        ..Default::default()
    };

    new_permit.insert(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let status_class = "tag tag-ok".to_string();
    let res = PermitResponse {
        id,
        name: input.name,
        holder: "Biscayne STR Co.".to_string(),
        license: input.license,
        permit_type: "Short-term Rental".to_string(),
        status: "✓ Active".to_string(),
        status_class,
        last_checked: "Just now".to_string(),
        date_renewed: Utc::now().format("%b %d, %Y").to_string(),
    };

    Ok((StatusCode::CREATED, Json(res)))
}

pub async fn verify_permit(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let permit = atlas_regulatory_registration::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut active: atlas_regulatory_registration::ActiveModel = permit.into();
    active.status = Set("✓ Verified".to_string());
    active.update(&db).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}

pub async fn list_geo_zones(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let list = geo_service_area::Entity::find()
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response: Vec<GeoZoneResponse> = list
        .into_iter()
        .map(|m| {
            let points = m.geom.as_deref().map(wkt_to_svg_points).unwrap_or_default();
            GeoZoneResponse {
                key: m.id.to_string(),
                name: m.label.clone().unwrap_or_else(|| "Unnamed Zone".to_string()),
                region: m.owner_entity_type.clone(),
                listings: "12 listings".to_string(), // Stub count
                status: "SRID 4326 (Valid)".to_string(),
                status_class: "tag tag-ok".to_string(),
                coverage: "4.2 sq km".to_string(), // Stub area
                points,
            }
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_geo_zone(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateGeoZoneInput>,
) -> Result<impl IntoResponse, StatusCode> {
    // Resolve tenant_id
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(current_user.id))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let tenant_id = if let Some(ua) = user_accounts.first() {
        let profile = crate::entities::profile::Entity::find()
            .filter(crate::entities::profile::Column::AccountId.eq(ua.account_id))
            .one(&db)
            .await
            .unwrap_or(None);
        profile.map(|p| p.tenant_id).unwrap_or_else(Uuid::new_v4)
    } else {
        Uuid::new_v4()
    };

    let geom_wkt = svg_points_to_wkt(&input.points).ok_or(StatusCode::BAD_REQUEST)?;

    let id = Uuid::new_v4();
    let new_zone = geo_service_area::ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        owner_entity_type: Set(input.region.clone()),
        owner_entity_id: Set(Uuid::new_v4()), // Stub association ID
        label: Set(Some(input.name.clone())),
        geom: Set(Some(geom_wkt)),
        created_at: Set(Utc::now()),
        ..Default::default()
    };

    new_zone.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to insert geo zone: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let res = GeoZoneResponse {
        key: id.to_string(),
        name: input.name,
        region: input.region,
        listings: "12 listings".to_string(),
        status: "SRID 4326 (Valid)".to_string(),
        status_class: "tag tag-ok".to_string(),
        coverage: "2.4 sq km".to_string(),
        points: input.points,
    };

    Ok((StatusCode::CREATED, Json(res)))
}

// ── Contracts (G-11) ─────────────────────────────────────────────────────────

pub async fn list_contracts(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let contracts = atlas_contract::Entity::find()
        .order_by_desc(atlas_contract::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Resolve tenant names for signee column
    let tenant_ids: Vec<Uuid> = contracts.iter().map(|c| c.tenant_id).collect();
    let tenants = tenant::Entity::find()
        .filter(tenant::Column::Id.is_in(tenant_ids))
        .all(&db)
        .await
        .unwrap_or_default();
    let tenant_map: std::collections::HashMap<Uuid, String> =
        tenants.into_iter().map(|t| (t.id, t.name)).collect();

    let response: Vec<ContractResponse> = contracts
        .into_iter()
        .map(|c| {
            let status_display = match c.status.as_str() {
                "active" => "Executed".to_string(),
                "draft" => "In Review".to_string(),
                "pending_signature" => "Pending Signature".to_string(),
                "expired" => "Expired".to_string(),
                "terminated" => "Terminated".to_string(),
                other => other.to_string(),
            };
            let status_class = match c.status.as_str() {
                "active" => "tag tag-ok",
                "draft" | "pending_signature" => "tag tag-warn",
                "expired" | "terminated" => "tag tag-err",
                _ => "tag tag-warn",
            };
            let date_executed = c
                .signed_at
                .map(|dt| dt.format("%b %d, %Y").to_string())
                .unwrap_or_else(|| "Draft".to_string());
            let expiry_date = c
                .end_date
                .map(|d| d.format("%b %d, %Y").to_string())
                .unwrap_or_else(|| "—".to_string());
            let signee = tenant_map
                .get(&c.tenant_id)
                .cloned()
                .unwrap_or_else(|| "Unknown Tenant".to_string());
            let short_id = c.id.to_string().chars().take(8).collect::<String>();
            let name = format!(
                "ct_{}_{}",
                signee
                    .to_lowercase()
                    .replace(' ', "_")
                    .chars()
                    .take(12)
                    .collect::<String>(),
                short_id
            );
            let vault_file = c
                .terms_metadata
                .as_ref()
                .and_then(|m| m.get("vault_file"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            ContractResponse {
                id: c.id.to_string(),
                name,
                signee,
                contract_type: c.contract_type.clone(),
                status: status_display,
                status_class: status_class.to_string(),
                date_executed,
                expiry_date,
                vault_file,
            }
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_contract(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateContractInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(current_user.id))
        .all(&db)
        .await
        .unwrap_or_default();
    let tenant_id = input.signee_tenant_id.unwrap_or_else(|| {
        user_accounts
            .first()
            .map(|ua| ua.account_id)
            .unwrap_or_else(Uuid::new_v4)
    });

    let start_date = chrono::NaiveDate::parse_from_str(&input.start_date, "%Y-%m-%d")
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let end_date = input
        .end_date
        .as_deref()
        .map(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d"))
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut metadata = serde_json::json!({});
    if let Some(ref vf) = input.vault_file {
        metadata["vault_file"] = serde_json::json!(vf);
    }

    let id = Uuid::new_v4();
    let new_contract = atlas_contract::ActiveModel {
        id: Set(id),
        tenant_id: Set(tenant_id),
        contract_type: Set(input.contract_type.clone()),
        counterparty_user_id: Set(None),
        asset_id: Set(None),
        start_date: Set(start_date),
        end_date: Set(end_date),
        auto_renew: Set(false),
        recurring_amount_cents: Set(None),
        currency: Set("USD".to_string()),
        billing_interval: Set("monthly".to_string()),
        status: Set("active".to_string()),
        signed_at: Set(Some(Utc::now())),
        terminated_at: Set(None),
        termination_reason: Set(None),
        terms_metadata: Set(Some(metadata)),
        created_at: Set(Utc::now()),
        managed_account_id: Set(None),
    };

    new_contract.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to insert contract: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let short_id = id.to_string().chars().take(8).collect::<String>();
    let res = ContractResponse {
        id: id.to_string(),
        name: format!("ct_new_{}", short_id),
        signee: "New Tenant".to_string(),
        contract_type: input.contract_type,
        status: "Executed".to_string(),
        status_class: "tag tag-ok".to_string(),
        date_executed: Utc::now().format("%b %d, %Y").to_string(),
        expiry_date: end_date
            .map(|d| d.format("%b %d, %Y").to_string())
            .unwrap_or_else(|| "—".to_string()),
        vault_file: input.vault_file,
    };

    Ok((StatusCode::CREATED, Json(res)))
}
