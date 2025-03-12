use axum::{
    response::{IntoResponse, Json},
    http::StatusCode,
    Extension,
};
use serde_json::json;
use sea_orm::{DatabaseBackend, Statement, DatabaseConnection, ConnectionTrait};

pub async fn health_check(
    Extension(db): Extension<DatabaseConnection>
) -> impl IntoResponse {
    // Execute a minimal database health check query
    let health_result = db.execute(
        Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT 1 AS health_check".to_string()
        )
    ).await;

    match health_result {
        Ok(_) => {
            // Successful execution indicates connectivity
            (
                StatusCode::OK,
                Json(json!({
                    "status": "healthy",
                    "database": "connected",
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            )
        }
        Err(db_error) => {
            // Log the error (in a real app, you'd use a proper logger)
            eprintln!("Database health check failed: {:?}", db_error);
            
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "unhealthy",
                    "database": "disconnected",
                    "error": db_error.to_string(),
                    "version": env!("CARGO_PKG_VERSION"),
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            )
        }
    }
}