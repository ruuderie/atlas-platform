use crate::entities::request_log;
use crate::models::request_log::{RequestStatus, RequestType};
use axum::http::{Method, StatusCode, Uri};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use uuid::Uuid;

pub async fn log_request(
    method: Method,
    uri: Uri,
    status: StatusCode,
    user_id: Option<Uuid>,
    user_agent: &str,
    ip_address: &str,
    request_type: RequestType,
    request_status: RequestStatus,
    failure_reason: Option<String>,
    db: &DatabaseConnection,
) -> Result<(), StatusCode> {
    let log_entry = request_log::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user_id),
        ip_address: Set(ip_address.to_string()),
        user_agent: Set(Some(user_agent.to_string())),
        path: Set(uri.path().to_string()),
        method: Set(method.to_string()),
        status_code: Set(status.as_u16() as i32),
        request_type: Set(request_type),
        request_status: Set(request_status),
        failure_reason: Set(failure_reason),
        created_at: Set(Utc::now()),
    };

    log_entry.insert(db).await.map_err(|e| {
        tracing::error!("Failed to insert request log: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}
