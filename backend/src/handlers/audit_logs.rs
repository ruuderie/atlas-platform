use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::Deserialize;
use uuid::Uuid;

use crate::entities::{audit_log, user};

#[derive(Deserialize)]
pub struct AuditLogQuery {
    pub tenant_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub entity_id: Option<Uuid>,
}

pub async fn get_audit_logs(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    // TODO: if tenant is bound to session, extract it. For now assuming Super Admin can see all, Tenant Admin must have a bound tenant
    Query(params): Query<AuditLogQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut query = audit_log::Entity::find()
        .order_by_desc(audit_log::Column::CreatedAt);

    // is_admin was removed from the user entity (RBAC migration). Check via user_account role.
    let is_admin = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(current_user.id))
        .filter(crate::entities::user_account::Column::Role.eq(crate::entities::user_account::UserRole::PlatformSuperAdmin))
        .one(&db)
        .await
        .unwrap_or(None)
        .is_some();

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
    } else {
        // Super admin can optionally filter by tenant
        if let Some(tenant) = params.tenant_id {
            query = query.filter(audit_log::Column::TenantId.eq(tenant));
        }
    }

    if let Some(actor) = params.actor_id {
        query = query.filter(audit_log::Column::ActorId.eq(actor));
    }

    if let Some(entity) = params.entity_id {
        query = query.filter(audit_log::Column::EntityId.eq(entity));
    }

    let logs = query.all(&db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok((StatusCode::OK, Json(logs)))
}

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/admin/audit-logs", get(get_audit_logs))
}
