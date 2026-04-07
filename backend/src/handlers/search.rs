use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{DatabaseConnection, DbBackend, Statement, FromQueryResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use crate::entities::user;
use axum::routing::get;
use axum::Router;

pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/v1/search", get(global_search))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q: String,
    tenant_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, sea_orm::FromQueryResult)]
pub struct SearchResult {
    id: Uuid,
    entity_type: String,
    entity_id: Uuid,
    tenant_id: Option<Uuid>,
    metadata: Value,
}

pub async fn global_search(
    State(db): State<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(query): Query<SearchQuery>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    if query.q.trim().is_empty() {
        return Ok(Json(vec![]));
    }

    let mut effective_tenant_id = query.tenant_id;
    if !current_user.is_admin {
        if effective_tenant_id.is_none() {
            return Err((StatusCode::FORBIDDEN, "Tenant ID required for non-admins".to_string()));
        }
    }

    let mut sql = String::from("SELECT id, entity_type, entity_id, tenant_id, metadata FROM global_search_index WHERE searchable_text @@ plainto_tsquery('english', $1)");
    let mut values = vec![query.q.into()];
    
    if let Some(tid) = effective_tenant_id {
        sql.push_str(" AND (tenant_id = $2 OR tenant_id IS NULL)");
        values.push(tid.into());
    }

    sql.push_str(" ORDER BY created_at DESC LIMIT 50");

    let stmt = Statement::from_sql_and_values(DbBackend::Postgres, sql, values);
    
    let query_results = SearchResult::find_by_statement(stmt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!("Search query failed: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Search failed".to_string())
        })?;

    Ok(Json(query_results))
}
