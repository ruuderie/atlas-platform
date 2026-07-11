use axum::{
    Json, Router,
    extract::{Extension, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::entities::{audit_log, user};

#[derive(Deserialize)]
pub struct AuditLogQuery {
    pub tenant_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub entity_id: Option<Uuid>,
    /// Inclusive start date (ISO `YYYY-MM-DD`).
    pub date_from: Option<String>,
    /// Inclusive end date (ISO `YYYY-MM-DD`).
    pub date_to: Option<String>,
    /// Max rows to return (default 500).
    pub limit: Option<u64>,
}

fn parse_date_start(s: &str) -> Result<DateTime<Utc>, (StatusCode, String)> {
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid date_from '{s}'; expected YYYY-MM-DD"),
        )
    })?;
    Ok(Utc.from_utc_datetime(&d.and_hms_opt(0, 0, 0).unwrap()))
}

fn parse_date_end(s: &str) -> Result<DateTime<Utc>, (StatusCode, String)> {
    let d = NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid date_to '{s}'; expected YYYY-MM-DD"),
        )
    })?;
    // Inclusive end-of-day
    Ok(Utc.from_utc_datetime(&d.and_hms_opt(23, 59, 59).unwrap()))
}

fn apply_audit_filters(
    mut query: sea_orm::Select<audit_log::Entity>,
    params: &AuditLogQuery,
    is_admin: bool,
) -> Result<sea_orm::Select<audit_log::Entity>, (StatusCode, String)> {
    // Strict Tenant Scoping:
    // Only super admins can omit tenant filtering
    if !is_admin {
        if let Some(tenant) = params.tenant_id {
            query = query.filter(audit_log::Column::TenantId.eq(tenant));
        } else {
            return Err((
                StatusCode::FORBIDDEN,
                "Tenant ID is required for non-admin users".to_string(),
            ));
        }
    } else if let Some(tenant) = params.tenant_id {
        query = query.filter(audit_log::Column::TenantId.eq(tenant));
    }

    if let Some(actor) = params.actor_id {
        query = query.filter(audit_log::Column::ActorId.eq(actor));
    }

    if let Some(entity) = params.entity_id {
        query = query.filter(audit_log::Column::EntityId.eq(entity));
    }

    if let Some(ref from) = params.date_from {
        if !from.is_empty() {
            let start = parse_date_start(from)?;
            query = query.filter(audit_log::Column::CreatedAt.gte(start));
        }
    }

    if let Some(ref to) = params.date_to {
        if !to.is_empty() {
            let end = parse_date_end(to)?;
            query = query.filter(audit_log::Column::CreatedAt.lte(end));
        }
    }

    Ok(query)
}

async fn is_platform_super_admin(db: &DatabaseConnection, user_id: Uuid) -> bool {
    crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user_id))
        .filter(
            crate::entities::user_account::Column::Role
                .eq(crate::entities::user_account::UserRole::PlatformSuperAdmin),
        )
        .one(db)
        .await
        .unwrap_or(None)
        .is_some()
}

pub async fn get_audit_logs(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let is_admin = is_platform_super_admin(&db, current_user.id).await;

    let limit = params.limit.unwrap_or(500).clamp(1, 5000);

    let query = audit_log::Entity::find().order_by_desc(audit_log::Column::CreatedAt);
    let query = apply_audit_filters(query, &params, is_admin)?;
    let query = query.limit(limit);

    let logs = query.all(&db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok((StatusCode::OK, Json(logs)))
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// `GET /api/admin/audit-logs/export` — CSV download with the same filters as list.
pub async fn export_audit_logs_csv(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(params): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let is_admin = is_platform_super_admin(&db, current_user.id).await;

    let limit = params.limit.unwrap_or(500).clamp(1, 10_000);

    let query = audit_log::Entity::find().order_by_desc(audit_log::Column::CreatedAt);
    let query = apply_audit_filters(query, &params, is_admin)?;
    let query = query.limit(limit);

    let logs = query.all(&db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let mut csv = String::from(
        "id,tenant_id,actor_id,action_type,entity_type,entity_id,ip_address,created_at,old_state,new_state\n",
    );
    for log in &logs {
        let old = log
            .old_state
            .as_ref()
            .and_then(|v| serde_json::to_string(v).ok())
            .unwrap_or_default();
        let new = log
            .new_state
            .as_ref()
            .and_then(|v| serde_json::to_string(v).ok())
            .unwrap_or_default();
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{}\n",
            log.id,
            log.tenant_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            log.actor_id.map(|id| id.to_string()).unwrap_or_default(),
            csv_escape(&log.action_type),
            csv_escape(&log.entity_type),
            log.entity_id,
            csv_escape(log.ip_address.as_deref().unwrap_or("")),
            log.created_at.to_rfc3339(),
            csv_escape(&old),
            csv_escape(&new),
        ));
    }

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "text/csv".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"audit-log.csv\"".parse().unwrap(),
    );

    Ok((StatusCode::OK, headers, csv))
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/audit-logs", get(get_audit_logs))
        .route("/api/admin/audit-logs/export", get(export_audit_logs_csv))
}
