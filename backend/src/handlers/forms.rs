use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json, Extension,
};
use serde::{Deserialize, Serialize};
use sea_orm::{DatabaseConnection, ConnectionTrait};
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;
use crate::config::site_config::SiteConfig;

#[derive(Deserialize)]
pub struct FormSubmissionReq {
    pub form_id: String,
    pub payload_json: serde_json::Value,
}

#[derive(Serialize)]
pub struct FormSubmissionResp {
    pub success: bool,
    pub submission_id: String,
}

pub async fn submit_form(
    State(db): State<DatabaseConnection>,
    Extension(site): Extension<SiteConfig>,
    Json(payload): Json<FormSubmissionReq>,
) -> impl IntoResponse {
    let tenant_id = site.tenant_id;

    let form_u = match uuid::Uuid::parse_str(&payload.form_id) {
        Ok(u) => u,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid form ID").into_response(),
    };

    let sub_id = uuid::Uuid::new_v4();
    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "INSERT INTO form_submissions (id, form_id, tenant_id, payload_json) VALUES ($1, $2, $3, $4)",
        vec![sub_id.into(), form_u.into(), tenant_id.into(), payload.payload_json.clone().into()]
    );
    let res = db.execute(stmt).await;

    match res {
        Ok(_) => {
            let db_clone = db.clone();
            let payload_clone = payload.payload_json.clone();
            tokio::spawn(async move {
                if let Err(e) = crate::services::webhook::dispatch_event(
                    &db_clone,
                    tenant_id,
                    "webform.submitted",
                    payload_clone,
                ).await {
                    tracing::error!("Webhook dispatch failed for form submission {}: {:?}", sub_id, e);
                }
            });

            (StatusCode::OK, Json(FormSubmissionResp { success: true, submission_id: sub_id.to_string() })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to save form submission: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct PresignedUrlReq {
    pub filename: String,
    pub content_type: String,
}

#[derive(Serialize)]
pub struct PresignedUrlResp {
    pub upload_url: String,
    pub file_key: String,
}

pub async fn get_presigned_url(
    Extension(site): Extension<SiteConfig>,
    Json(payload): Json<PresignedUrlReq>,
) -> impl IntoResponse {
    let tenant_id = site.tenant_id;

    // Explicitly targets the secure platform vault preventing public access leaks
    let bucket_name = "atlas-tenant-vault".to_string();
    
    let access_key = std::env::var("R2_ACCESS_KEY_ID").unwrap_or_default();
    let secret = std::env::var("R2_SECRET_ACCESS_KEY").unwrap_or_default();
    let endpoint = std::env::var("R2_ENDPOINT").unwrap_or_default();

    if access_key.is_empty() || endpoint.is_empty() {
        return (StatusCode::NOT_IMPLEMENTED, "R2 currently unconfigured for this tenant environment").into_response();
    }

    let credentials = aws_sdk_s3::config::Credentials::new(
        access_key, secret, None, None, "cloudflare"
    );
    let s3_config = aws_sdk_s3::config::Builder::new()
        .credentials_provider(credentials)
        .region(aws_sdk_s3::config::Region::new("auto"))
        .endpoint_url(endpoint.clone())
        .build();

    let client = Client::from_conf(s3_config);
    let expires_in = std::time::Duration::from_secs(3600);
    let presigning_config = match PresigningConfig::expires_in(expires_in) {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Config err").into_response(),
    };

    let unique_id = uuid::Uuid::new_v4();
    let file_key = format!("tenant_{}/lead_uploads/{}_{}", tenant_id, unique_id, payload.filename);

    let presigned_req = client
        .put_object()
        .bucket(&bucket_name)
        .key(&file_key)
        .content_type(&payload.content_type)
        .presigned(presigning_config)
        .await;

    match presigned_req {
        Ok(req) => {
            (StatusCode::OK, Json(PresignedUrlResp { 
                upload_url: req.uri().to_string(), 
                file_key 
            })).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to generate presigned URL: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "S3 Presign Error").into_response()
        }
    }
}

pub fn public_routes() -> axum::Router<DatabaseConnection> {
    axum::Router::new()
        .route("/api/forms/submit", axum::routing::post(submit_form))
        .route("/api/forms/upload-url", axum::routing::post(get_presigned_url))
}
