//! Admin-scoped R2 presigned upload URL.
//!
//! `POST /api/admin/upload-presign`
//!
//! Returns a presigned PUT URL targeting `atlas-tenant-vault` bucket,
//! scoped to `platform-admin/{folder}/{user_id}/{uuid}.{ext}`.
//! The client PUTs file bytes directly to R2; the backend is never in
//! the upload path.

use crate::entities::user;
use aws_sdk_s3::Client;
use aws_sdk_s3::presigning::PresigningConfig;
use axum::{
    Router,
    extract::{Extension, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct AdminPresignReq {
    /// Original filename, e.g. "avatar.png"
    pub filename: String,
    /// MIME type, e.g. "image/png"
    pub content_type: String,
    /// Subfolder within platform-admin/, e.g. "avatars" or "transcripts"
    #[serde(default = "default_folder")]
    pub folder: String,
}

fn default_folder() -> String {
    "uploads".to_string()
}

#[derive(Debug, Serialize)]
pub struct AdminPresignResp {
    /// Presigned PUT URL — client uploads directly here
    pub upload_url: String,
    /// R2 object key — store this as storage_path in the file record
    pub file_key: String,
    /// Public-facing URL (if bucket has public access on this prefix)
    pub public_url: String,
}

pub async fn admin_presign_upload(
    Extension(current_user): Extension<user::Model>,
    Json(payload): Json<AdminPresignReq>,
) -> impl IntoResponse {
    let access_key = std::env::var("R2_ACCESS_KEY_ID").unwrap_or_default();
    let secret = std::env::var("R2_SECRET_ACCESS_KEY").unwrap_or_default();
    let endpoint = std::env::var("R2_ENDPOINT").unwrap_or_default();
    let public_base = std::env::var("R2_PUBLIC_URL").unwrap_or_else(|_| endpoint.clone());

    if access_key.is_empty() || endpoint.is_empty() {
        return (
            StatusCode::NOT_IMPLEMENTED,
            "R2 not configured for this environment",
        )
            .into_response();
    }

    let bucket = "atlas-tenant-vault".to_string();

    let ext = payload
        .filename
        .rsplit('.')
        .next()
        .unwrap_or("bin")
        .to_lowercase();
    let unique_id = Uuid::new_v4();
    let safe_folder = payload.folder.replace("..", "").replace('/', "");
    let file_key = format!(
        "platform-admin/{}/{}/{}.{}",
        safe_folder, current_user.id, unique_id, ext
    );

    let credentials =
        aws_sdk_s3::config::Credentials::new(&access_key, &secret, None, None, "cloudflare");
    let s3_config = aws_sdk_s3::config::Builder::new()
        .credentials_provider(credentials)
        .region(aws_sdk_s3::config::Region::new("auto"))
        .endpoint_url(&endpoint)
        .build();
    let client = Client::from_conf(s3_config);

    let expires_in = std::time::Duration::from_secs(3600);
    let presigning_config = match PresigningConfig::expires_in(expires_in) {
        Ok(c) => c,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Presign config error").into_response();
        }
    };

    let presigned = client
        .put_object()
        .bucket(&bucket)
        .key(&file_key)
        .content_type(&payload.content_type)
        .presigned(presigning_config)
        .await;

    match presigned {
        Ok(req) => {
            let public_url = format!("{}/{}", public_base.trim_end_matches('/'), file_key);
            (
                StatusCode::OK,
                Json(AdminPresignResp {
                    upload_url: req.uri().to_string(),
                    file_key,
                    public_url,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("admin_presign_upload: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, "S3 presign failed").into_response()
        }
    }
}

pub fn routes() -> Router<DatabaseConnection> {
    Router::new().route("/api/admin/upload-presign", post(admin_presign_upload))
}
