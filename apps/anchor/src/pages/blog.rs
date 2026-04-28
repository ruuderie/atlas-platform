use leptos::*;
use leptos_router::*;
use crate::components::design_mode::use_kami_mode;
use crate::components::content_feed::{ContentFeed, ContentNode, LayoutMode};

#[server(GetPosts, "/api")]
pub async fn get_posts() -> Result<Vec<ContentNode>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let rows = sqlx::query(
        "SELECT id, title, to_char(created_at, 'YYYY.MM.DD') as created_at, payload \
         FROM app_content WHERE collection_type = 'blog_post' AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY created_at DESC"
    )
        .bind(tenant.0)
        .fetch_all(&state.pool)
        .await?;

    let posts = rows
        .into_iter()
        .map(|row| {
            let id: uuid::Uuid = row.get("id");
            let payload: serde_json::Value = row.get("payload");

            let slug = payload.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let tags: Vec<String> = payload.get("tags").and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            let content_format = payload.get("content_format").and_then(|v| v.as_str()).unwrap_or("markdown").to_string();
            let link_url = format!("/blog/{}", slug);

            ContentNode {
                id: id.to_string(),
                category: "blog_post".to_string(),
                title: row.get("title"),
                subtitle: Some(slug),
                date_label: row.try_get("created_at").unwrap_or(None),
                status: None,
                tags,
                bullets: vec![],
                markdown: Some(content),
                link_url: Some(link_url),
                is_highlight: false,
                content_format,
            }
        })
        .collect();

    Ok(posts)
}

#[server(GetPostBySlug, "/api")]
pub async fn get_post_by_slug(slug: String) -> Result<Option<ContentNode>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let row = sqlx::query(
        "SELECT id, title, to_char(created_at, 'YYYY.MM.DD') as created_at, payload \
         FROM app_content WHERE collection_type = 'blog_post' \
         AND tenant_id IS NOT DISTINCT FROM $1 AND payload->>'slug' = $2 LIMIT 1"
    )
        .bind(tenant.0)
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await?;

    Ok(row.map(|row| {
        let id: uuid::Uuid = row.get("id");
        let payload: serde_json::Value = row.get("payload");
        let content = payload.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let tags: Vec<String> = payload.get("tags").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let content_format = payload.get("content_format").and_then(|v| v.as_str()).unwrap_or("markdown").to_string();

        ContentNode {
            id: id.to_string(),
            category: "blog_post".to_string(),
            title: row.get("title"),
            subtitle: Some(slug),
            date_label: row.try_get("created_at").unwrap_or(None),
            status: None,
            tags,
            bullets: vec![],
            markdown: Some(content),
            link_url: None,
            is_highlight: false,
            content_format,
        }
    }))
}

#[server(AddPost, "/api")]
pub async fn add_post(
    slug: String,
    title: String,
    content: String,
    tags: Vec<String>,
    pdf_attachment_url: Option<String>,
    pdf_generate_from_content: bool,
    pdf_require_lead_capture: bool,
    pdf_lead_capture_label: Option<String>,
    pdf_lead_notification_email: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let payload = serde_json::json!({
        "slug": slug,
        "content": content,
        "tags": tags,
        "content_format": "markdown",
        "pdf_attachment_url": pdf_attachment_url,
        "pdf_generate_from_content": pdf_generate_from_content,
        "pdf_require_lead_capture": pdf_require_lead_capture,
        "pdf_lead_capture_label": pdf_lead_capture_label,
        "pdf_lead_notification_email": pdf_lead_notification_email,
    });

    sqlx::query("INSERT INTO app_content (tenant_id, collection_type, title, payload) VALUES ($1, 'blog_post', $2, $3)")
        .bind(tenant.0)
        .bind(title)
        .bind(payload)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[server(UpdatePost, "/api")]
pub async fn update_post(
    id: String,
    slug: String,
    title: String,
    content: String,
    tags: Vec<String>,
    pdf_attachment_url: Option<String>,
    pdf_generate_from_content: bool,
    pdf_require_lead_capture: bool,
    pdf_lead_capture_label: Option<String>,
    pdf_lead_notification_email: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let uuid_id = match uuid::Uuid::parse_str(&id) {
        Ok(v) => v,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let payload = serde_json::json!({
        "slug": slug,
        "content": content,
        "tags": tags,
        "content_format": "markdown",
        "pdf_attachment_url": pdf_attachment_url,
        "pdf_generate_from_content": pdf_generate_from_content,
        "pdf_require_lead_capture": pdf_require_lead_capture,
        "pdf_lead_capture_label": pdf_lead_capture_label,
        "pdf_lead_notification_email": pdf_lead_notification_email,
    });

    sqlx::query(
        "UPDATE app_content SET title = $1, payload = $2 WHERE id = $3 AND tenant_id IS NOT DISTINCT FROM $4 AND collection_type = 'blog_post'",
    )
    .bind(title)
    .bind(payload)
    .bind(uuid_id)
    .bind(tenant.0)
    .execute(&state.pool)
    .await?;
    Ok(())
}

#[server(DeletePost, "/api")]
pub async fn delete_post(id: String) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let uuid_id = match uuid::Uuid::parse_str(&id) {
        Ok(v) => v,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM app_content WHERE id = $1 AND tenant_id IS NOT DISTINCT FROM $2 AND collection_type = 'blog_post'")
        .bind(uuid_id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;
    Ok(())
}

// ─── Blog PDF server functions ─────────────────────────────────────────────────

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct BlogPdfConfig {
    pub pdf_attachment_url: Option<String>,
    pub pdf_generate_from_content: bool,
    pub pdf_require_lead_capture: bool,
    pub pdf_lead_capture_label: Option<String>,
    pub pdf_lead_notification_email: Option<String>,
}

#[server(GetBlogPdfConfig, "/api")]
pub async fn get_blog_pdf_config(slug: String) -> Result<Option<BlogPdfConfig>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let row = sqlx::query(
        "SELECT payload FROM app_content \
         WHERE collection_type = 'blog_post' \
         AND tenant_id IS NOT DISTINCT FROM $1 AND payload->>'slug' = $2 LIMIT 1",
    )
    .bind(tenant.0)
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await?;

    Ok(row.map(|row| {
        let payload: serde_json::Value = row.get("payload");
        BlogPdfConfig {
            pdf_attachment_url: payload.get("pdf_attachment_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
            pdf_generate_from_content: payload.get("pdf_generate_from_content").and_then(|v| v.as_bool()).unwrap_or(false),
            pdf_require_lead_capture: payload.get("pdf_require_lead_capture").and_then(|v| v.as_bool()).unwrap_or(false),
            pdf_lead_capture_label: payload.get("pdf_lead_capture_label").and_then(|v| v.as_str()).map(|s| s.to_string()),
            pdf_lead_notification_email: payload.get("pdf_lead_notification_email").and_then(|v| v.as_str()).map(|s| s.to_string()),
        }
    }))
}

#[server(SaveBlogPdfSettings, "/api")]
pub async fn save_blog_pdf_settings(
    id: String,
    pdf_attachment_url: Option<String>,
    pdf_generate_from_content: bool,
    pdf_require_lead_capture: bool,
    pdf_lead_capture_label: Option<String>,
    pdf_lead_notification_email: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let uuid_id = match uuid::Uuid::parse_str(&id) {
        Ok(v) => v,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    sqlx::query(
        "UPDATE app_content SET payload = payload || $1::jsonb \
         WHERE id = $2 AND tenant_id IS NOT DISTINCT FROM $3 AND collection_type = 'blog_post'",
    )
    .bind(serde_json::json!({
        "pdf_attachment_url": pdf_attachment_url,
        "pdf_generate_from_content": pdf_generate_from_content,
        "pdf_require_lead_capture": pdf_require_lead_capture,
        "pdf_lead_capture_label": pdf_lead_capture_label,
        "pdf_lead_notification_email": pdf_lead_notification_email,
    }))
    .bind(uuid_id)
    .bind(tenant.0)
    .execute(&state.pool)
    .await?;
    Ok(())
}

#[server(GetR2PresignedUploadUrl, "/api")]
pub async fn get_r2_presigned_upload_url(filename: String) -> Result<String, ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    use hmac::{Hmac, Mac};
    use sha2::{Sha256, Digest};

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tenant_id = tenant.0.map(|id| id.to_string()).unwrap_or_else(|| "global".to_string());

    let access_key = match std::env::var("R2_ACCESS_KEY_ID") {
        Ok(v) => v,
        Err(_) => return Err(ServerFnError::ServerError("R2_ACCESS_KEY_ID not configured".into())),
    };
    let secret_key = match std::env::var("R2_SECRET_ACCESS_KEY") {
        Ok(v) => v,
        Err(_) => return Err(ServerFnError::ServerError("R2_SECRET_ACCESS_KEY not configured".into())),
    };
    let endpoint = match std::env::var("R2_ENDPOINT") {
        Ok(v) => v,
        Err(_) => return Err(ServerFnError::ServerError("R2_ENDPOINT not configured".into())),
    };
    let bucket = match std::env::var("R2_VAULT_BUCKET") {
        Ok(v) => v,
        Err(_) => return Err(ServerFnError::ServerError("R2_VAULT_BUCKET not configured".into())),
    };

    // Sanitize filename
    let safe_name: String = filename.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect();
    let object_key = format!("{}/pdf-attachments/{}", tenant_id, safe_name);

    // AWS SigV4 presigned URL (query-string signing, S3-compatible for R2)
    let now = chrono::Utc::now();
    let date_str = now.format("%Y%m%d").to_string();
    let datetime_str = now.format("%Y%m%dT%H%M%SZ").to_string();
    let region = "auto";
    let service = "s3";
    let expires = "900"; // 15 minutes
    let credential = format!("{}/{}/{}/{}/aws4_request", access_key, date_str, region, service);

    // URL-encode the object key
    let encoded_key: String = object_key.split('/').map(|s| urlencoding::encode(s).into_owned()).collect::<Vec<_>>().join("/");
    let host = endpoint.trim_start_matches("https://").trim_start_matches("http://");
    let url_path = format!("/{}/{}", bucket, encoded_key);

    // Canonical query string (params must be sorted)
    let canonical_qs = format!(
        "X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential={}&X-Amz-Date={}&X-Amz-Expires={}&X-Amz-SignedHeaders=host",
        urlencoding::encode(&credential),
        datetime_str,
        expires
    );

    let canonical_headers = format!("host:{}\n", host);
    let canonical_request = format!(
        "PUT\n{}\n{}\n{}\nhost\nUNSIGNED-PAYLOAD",
        url_path, canonical_qs, canonical_headers
    );

    // String to sign
    let canonical_hash = format!("{:x}", Sha256::digest(canonical_request.as_bytes()));
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}/{}/{}/aws4_request\n{}",
        datetime_str, date_str, region, service, canonical_hash
    );

    // Derive signing key: HMAC(HMAC(HMAC(HMAC("AWS4" + secret, date), region), service), "aws4_request")
    let hmac_sign = |key: &[u8], msg: &[u8]| -> Vec<u8> {
        let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC key");
        mac.update(msg);
        mac.finalize().into_bytes().to_vec()
    };
    let signing_key = hmac_sign(
        &hmac_sign(
            &hmac_sign(
                &hmac_sign(format!("AWS4{}", secret_key).as_bytes(), date_str.as_bytes()),
                region.as_bytes(),
            ),
            service.as_bytes(),
        ),
        b"aws4_request",
    );

    let mut mac = match Hmac::<Sha256>::new_from_slice(&signing_key) {
        Ok(m) => m,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };
    mac.update(string_to_sign.as_bytes());
    let signature = format!("{:x}", mac.finalize().into_bytes());

    let signed_url = format!(
        "{}{url_path}?{}&X-Amz-Signature={}",
        endpoint,
        canonical_qs,
        signature,
        url_path = url_path,
    );

    Ok(signed_url)
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DownloadTokenResponse {
    pub token: String,
}

#[server(SubmitDownloadLead, "/api")]
pub async fn submit_download_lead(
    slug: String,
    email: String,
    name: Option<String>,
) -> Result<DownloadTokenResponse, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    if email.is_empty() || !email.contains('@') {
        return Err(ServerFnError::ServerError("Invalid email address".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    // Fetch the post to verify it exists and get its id
    let row = sqlx::query(
        "SELECT id, title, payload FROM app_content \
         WHERE collection_type = 'blog_post' \
         AND tenant_id IS NOT DISTINCT FROM $1 AND payload->>'slug' = $2 LIMIT 1",
    )
    .bind(tenant.0)
    .bind(&slug)
    .fetch_optional(&state.pool)
    .await?;

    let row = match row {
        Some(r) => r,
        None => return Err(ServerFnError::ServerError("Post not found".into())),
    };
    let post_id: uuid::Uuid = sqlx::Row::get(&row, "id");
    let post_title: String = sqlx::Row::get(&row, "title");
    let payload: serde_json::Value = sqlx::Row::get(&row, "payload");

    let tenant_id = match tenant.0 {
        Some(id) => id,
        None => return Err(ServerFnError::ServerError("No tenant".into())),
    };

    // Insert lead row
    sqlx::query(
        "INSERT INTO blog_download_leads (tenant_id, post_id, email, name) VALUES ($1, $2, $3, $4) \
         ON CONFLICT DO NOTHING",
    )
    .bind(tenant_id)
    .bind(post_id)
    .bind(&email)
    .bind(&name)
    .execute(&state.pool)
    .await?;

    // Send notification email if configured
    let notification_email = payload
        .get("pdf_lead_notification_email")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            // Fall back to tenant site_settings pdf_lead_notification_email
            None // Fetching settings is async; skip for now — handled below
        });

    // Kick off email notification (best-effort, non-blocking)
    if let Some(notify_to) = notification_email {
        let email_clone = email.clone();
        let name_clone = name.clone();
        let title_clone = post_title.clone();
        tokio::spawn(async move {
            let _ = crate::email::send_email(
                notify_to,
                format!("New PDF Download Lead — \"{}\"", title_clone),
                format!(
                    "<p><strong>Name:</strong> {}</p><p><strong>Email:</strong> {}</p><p><strong>Post:</strong> {}</p><p><strong>Time:</strong> {}</p>",
                    name_clone.as_deref().unwrap_or("(not provided)"),
                    email_clone,
                    title_clone,
                    chrono::Utc::now().format("%Y-%m-%d %H:%M UTC")
                ),
            ).await;
        });
    }

    // Generate a short-lived HMAC token: HMAC-SHA256(secret, "post_id:email:unix_timestamp_5min_bucket")
    let secret = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "fallback-secret".to_string());
    let bucket_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() / 300; // 5-minute window

    let message = format!("{}:{}:{}", post_id, email, bucket_ts);
    let mut mac = match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(e) => return Err(ServerFnError::ServerError(e.to_string())),
    };
    mac.update(message.as_bytes());
    let token = base64::encode(mac.finalize().into_bytes());

    Ok(DownloadTokenResponse { token })
}


// ─── Blog list page ───────────────────────────────────────────────────────────

#[component]
pub fn Blog() -> impl IntoView {
    let posts_resource = create_resource(
        || (),
        |_| async move { get_posts().await.unwrap_or_else(|_| vec![]) },
    );

    view! {
        <main class="pt-32 pb-24 bg-surface-container-low min-h-screen">
            <Suspense fallback=move || view! { <div class="px-4 md:px-[8.5rem] text-on-surface-variant font-bold jetbrains uppercase">"⏳ Fetching remote Markdown streams..."</div> }>
                {move || {
                    let posts = posts_resource.get().unwrap_or_default();
                    if use_kami_mode() {
                        view! { <KamiBlogIndex posts=posts /> }.into_view()
                    } else {
                        view! {
                            <div class="px-4 md:px-[8.5rem]">
                                <crate::components::dynamic_header::DynamicPageHeader route_path="/blog".to_string() badge_color="primary".to_string() />
                                <ContentFeed nodes=posts layout=LayoutMode::List />
                            </div>
                        }.into_view()
                    }
                }}
            </Suspense>
        </main>
    }
}

// ─── Blog post detail page ────────────────────────────────────────────────────

#[component]
pub fn BlogPost() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").cloned().unwrap_or_default());

    let post_resource = create_resource(slug, |s| async move {
        get_post_by_slug(s).await.unwrap_or(None)
    });

    view! {
        <main class="pt-28 pb-24 bg-surface-container-low min-h-screen">
            // KaTeX CSS — only loaded on this page
            <link rel="stylesheet"
                href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css"
                crossorigin="anonymous" />

            <Suspense fallback=move || view! {
                <div class="max-w-3xl mx-auto px-6 pt-12">
                    <div class="text-on-surface-variant font-bold jetbrains uppercase animate-pulse">"Loading paper..."</div>
                </div>
            }>
                {move || {
                    match post_resource.get() {
                        None => view! { <div /> }.into_view(),
                        Some(None) => view! {
                            <div class="max-w-3xl mx-auto px-6 pt-12 text-on-surface-variant">
                                "Post not found."
                                <a href="/blog" class="ml-4 text-secondary underline">"← Back to blog"</a>
                            </div>
                        }.into_view(),
                        Some(Some(post)) => {
                            let content_format = post.content_format.clone();
                            let html = render_post_html(&post);
                            let title = post.title.clone();
                            let date = post.date_label.clone().unwrap_or_default();
                            let tags = post.tags.clone();
                            let needs_katex = content_format == "mdlatex" || content_format == "latex";

                            view! {
                                // Outer page background stays dark; the paper itself is parchment
                                <div class="max-w-4xl mx-auto px-4 md:px-0 pb-24">
                                    // Breadcrumb — above the paper card
                                    <nav class="mb-8">
                                        <a href="/blog"
                                           class="inline-flex items-center gap-2 text-on-surface-variant hover:text-secondary transition-colors jetbrains text-[0.65rem] uppercase tracking-widest">
                                            <span class="material-symbols-outlined text-sm">"arrow_back"</span>
                                            "Technical Writing"
                                        </a>
                                    </nav>

                                    // ── Kami Parchment Paper Card ──────────────────────────────────────
                                    <article class="bg-[#f5f4ed] shadow-2xl px-10 py-16 md:px-20 md:py-20 font-display">

                                        // Academic title block — centered like an arXiv preprint
                                        <header class="text-center mb-16 pb-12 border-b border-[#1B365D]/20">
                                            <h1 class="text-[1.85rem] md:text-[2.25rem] font-extrabold text-[#1B365D] leading-[1.2] mb-8 tracking-tight">
                                                {title}
                                            </h1>
                                            <div class="flex flex-wrap items-center justify-center gap-4 mb-6">
                                                <span class="jetbrains text-[0.65rem] uppercase text-[#6b6a64] tracking-widest font-medium">
                                                    {date}
                                                </span>
                                            </div>
                                            <div class="flex flex-wrap items-center justify-center gap-2">
                                                {tags.into_iter().map(|tag| view! {
                                                    <span class="bg-[#1B365D]/8 border border-[#1B365D]/20 text-[#1B365D] px-3 py-1 jetbrains text-[0.6rem] uppercase font-bold tracking-wider">
                                                        {tag}
                                                    </span>
                                                }).collect_view()}
                                            </div>
                                        </header>

                                        // ── PDF Download CTA ──────────────────────────────────────────
                                        // BlogPdfCta checks the post payload for PDF config and renders
                                        // either a direct download link or a lead-gate modal.
                                        {
                                            let pdf_url = post.markdown.as_deref()
                                                .map(|_| "") // placeholder — pdf_attachment_url from payload below
                                                .unwrap_or("");
                                            // Extract PDF config from payload stored in subtitle field (slug)
                                            // The actual post id is used for the slug-based Axum route.
                                            let post_slug = post.subtitle.clone().unwrap_or_default();
                                            view! { <BlogPdfCta slug=post_slug /> }
                                        }

                                        // ── Paper body — Kami academic prose styling ───────────────────
                                        <div
                                            class="prose max-w-none
                                                   prose-headings:font-extrabold prose-headings:text-[#1B365D] prose-headings:mt-12 prose-headings:mb-5 prose-headings:tracking-tight
                                                   prose-h2:text-[1.4rem] prose-h2:border-b prose-h2:border-[#1B365D]/20 prose-h2:pb-2
                                                   prose-h3:text-[1.15rem] prose-h4:text-[1rem]
                                                   prose-p:text-[#141413] prose-p:leading-[1.85] prose-p:text-[1rem] prose-p:my-8 prose-p:text-justify
                                                   prose-strong:text-[#141413] prose-strong:font-bold
                                                   prose-ul:text-[#141413] prose-ul:my-6 prose-ul:space-y-2 prose-ul:list-disc prose-ul:pl-6
                                                   prose-ol:text-[#141413] prose-ol:my-6 prose-ol:space-y-2
                                                   prose-li:text-[1rem] prose-li:leading-[1.75]
                                                   prose-code:text-[#504e49] prose-code:bg-[#ecebd4] prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-[0.875rem] prose-code:font-mono
                                                   prose-pre:bg-[#ecebd4] prose-pre:text-[#141413] prose-pre:p-6 prose-pre:rounded-none prose-pre:text-sm
                                                   prose-blockquote:border-l-4 prose-blockquote:border-[#1B365D]/40 prose-blockquote:text-[#504e49] prose-blockquote:italic prose-blockquote:pl-6
                                                   [&_.math-inline]:text-[#141413] [&_.math-display]:text-[#141413]
                                                   [&_.katex]:text-[#141413] [&_.katex-display]:my-10 [&_.katex-display]:overflow-x-auto [&_.katex-display]:py-2"
                                            inner_html=html
                                        >
                                        </div>

                                    // KaTeX auto-render script (only when needed)
                                    {if needs_katex {
                                        view! {
                                            <script
                                                src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.js"
                                                crossorigin="anonymous"
                                                defer=true
                                            />
                                            <script>
                                                "document.addEventListener('DOMContentLoaded', function() { \
                                                    if (typeof katex !== 'undefined') { \
                                                        document.querySelectorAll('.math-inline').forEach(function(el) { \
                                                            katex.render(el.textContent, el, {displayMode: false, throwOnError: false}); \
                                                        }); \
                                                        document.querySelectorAll('.math-display').forEach(function(el) { \
                                                            katex.render(el.textContent, el, {displayMode: true, throwOnError: false}); \
                                                        }); \
                                                    } \
                                                });"
                                            </script>
                                        }.into_view()
                                    } else {
                                        view! { <span /> }.into_view()
                                    }}

                                    </article>
                                </div>
                            }.into_view()
                        }
                    }
                }}
            </Suspense>
        </main>
    }
}

fn render_post_html(post: &ContentNode) -> String {
    if let Some(md) = &post.markdown {
        match post.content_format.as_str() {
            "latex" => {
                format!("<div class='katex-content'><pre class='katex-source'>{}</pre></div>",
                    html_escape::encode_text(md))
            }
            // mdlatex and markdown both go through pulldown_cmark;
            _ => {
                let mut options = pulldown_cmark::Options::empty();
                options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
                options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
                options.insert(pulldown_cmark::Options::ENABLE_MATH);
                let parser = pulldown_cmark::Parser::new_ext(md, options);
                let mut html = String::new();
                pulldown_cmark::html::push_html(&mut html, parser);
                html
            }
        }
    } else {
        String::new()
    }
}

// ─── Kami Blog Index ─────────────────────────────────────────────────────────
// Parchment mini-card index for the /blog list when kami_mode is enabled.

#[component]
fn KamiBlogIndex(posts: Vec<ContentNode>) -> impl IntoView {
    use crate::utils::text::markdown_excerpt;

    view! {
        <div class="max-w-3xl mx-auto px-4 pb-24">
            // Archive header
            <div class="mb-12">
                <div class="jetbrains text-[0.6rem] uppercase tracking-[0.25em] text-[#6b6a64] mb-2">"Technical Writing"</div>
                <h1 class="font-display text-3xl font-bold text-[#1B365D] mb-4">"Papers &amp; Notes"</h1>
                <div class="w-16 h-px bg-[#1B365D]/30"></div>
            </div>

            // Post list
            <div class="space-y-4">
                {posts.into_iter().map(|post| {
                    let href = post.link_url.clone().unwrap_or_else(|| "/blog".to_string());
                    let exc = post.markdown.as_deref().map(|md| markdown_excerpt(md, 180)).unwrap_or_default();
                    let date = post.date_label.clone().unwrap_or_default();
                    let tags = post.tags.clone();

                    view! {
                        <a href=href class="block no-underline">
                            <article class="bg-[#f5f4ed] border border-[#1B365D]/10 px-8 py-6 shadow-sm hover:shadow-md hover:border-[#1B365D]/25 transition-all group">
                                <div class="flex items-baseline justify-between gap-4 mb-2">
                                    <h2 class="font-display text-lg font-bold text-[#1B365D] leading-snug group-hover:text-[#2a4d87] transition-colors">
                                        {post.title}
                                    </h2>
                                    <span class="jetbrains text-[0.58rem] uppercase tracking-widest text-[#6b6a64] whitespace-nowrap shrink-0">
                                        {date}
                                    </span>
                                </div>

                                {(!exc.is_empty()).then(|| view! {
                                    <p class="text-[#504e49] text-sm leading-relaxed mt-1 mb-3">{exc}</p>
                                })}

                                {(!tags.is_empty()).then(|| view! {
                                    <div class="flex flex-wrap gap-2 mt-3">
                                        {tags.into_iter().map(|tag| view! {
                                            <span class="border border-[#1B365D]/20 text-[#1B365D] px-2 py-0.5 jetbrains text-[0.55rem] uppercase tracking-wider">
                                                {tag}
                                            </span>
                                        }).collect_view()}
                                    </div>
                                })}
                            </article>
                        </a>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

// ─── Blog PDF CTA ────────────────────────────────────────────────────────────
// Rendered inside the blog post detail for posts with PDF config.
// Checks the GetBlogPdfConfig server fn and conditionally shows gated/non-gated UI.

#[component]
fn BlogPdfCta(slug: String) -> impl IntoView {
    let slug_clone = slug.clone();
    let config_resource = create_resource(
        move || slug.clone(),
        |s| async move { get_blog_pdf_config(s).await.unwrap_or(None) },
    );

    // Lead capture state
    let (show_modal, set_show_modal) = create_signal(false);
    let (lead_name, set_lead_name) = create_signal(String::new());
    let (lead_email, set_lead_email) = create_signal(String::new());
    let (submitting, set_submitting) = create_signal(false);
    let (download_token, set_download_token) = create_signal(Option::<String>::None);

    let slug_for_submit = slug_clone.clone();
    // Store slug in a StoredValue so multiple Fn closures can each call .get_value()
    // without consuming the String (which would make the outer closure FnOnce).
    let slug_store = store_value(slug_clone);
    let submit_action = create_action(move |_: &()| {
        let slug = slug_for_submit.clone();
        let email = lead_email.get_untracked();
        let name_str = lead_name.get_untracked();
        let name = if name_str.is_empty() { None } else { Some(name_str) };
        async move {
            set_submitting.set(true);
            match submit_download_lead(slug, email, name).await {
                Ok(resp) => {
                    set_download_token.set(Some(resp.token));
                    set_show_modal.set(false);
                }
                Err(_) => {}
            }
            set_submitting.set(false);
        }
    });

    view! {
        <Suspense fallback=move || view! {}>
            {move || {
                let cfg = config_resource.get().and_then(|c| c);
                let Some(cfg) = cfg else { return view! {}.into_view(); };

                let has_pdf = cfg.pdf_attachment_url.is_some() || cfg.pdf_generate_from_content;
                if !has_pdf { return view! {}.into_view(); }

                let label = cfg.pdf_lead_capture_label.clone().unwrap_or_else(|| "Download PDF".to_string());
                let requires_lead = cfg.pdf_require_lead_capture;

                view! {
                    <div class="mb-12">
                        // CTA strip
                        <div class="flex items-center justify-between gap-4 border border-[#1B365D]/20 bg-[#ecebd4] px-6 py-4">
                            <div class="flex items-center gap-3">
                                <span class="text-[#1B365D] text-xl">"📄"</span>
                                <span class="font-display text-sm text-[#504e49]">
                                    "A formatted Kami PDF of this paper is available."
                                </span>
                            </div>

                            {move || {
                                let slug_dl = slug_store.get_value();
                                if let Some(token) = download_token.get() {
                                    // Token received — trigger download via link
                                    let href = format!("/api/blog/{}/pdf?token={}", slug_dl, token);
                                    view! {
                                        <a href=href
                                           download=true
                                           class="jetbrains text-[0.65rem] uppercase tracking-widest px-5 py-2.5 bg-[#1B365D] text-[#f5f4ed] hover:bg-[#2a4d87] transition-colors whitespace-nowrap">
                                            "↓ Download"
                                        </a>
                                    }.into_view()
                                } else if requires_lead {
                                    view! {
                                        <button
                                            id="blog-pdf-download-btn"
                                            on:click=move |_| set_show_modal.set(true)
                                            class="jetbrains text-[0.65rem] uppercase tracking-widest px-5 py-2.5 bg-[#1B365D] text-[#f5f4ed] hover:bg-[#2a4d87] transition-colors whitespace-nowrap">
                                            {label.clone()}
                                        </button>
                                    }.into_view()
                                } else {
                                    let href = format!("/api/blog/{}/pdf", slug_dl);
                                    view! {
                                        <a href=href
                                           class="jetbrains text-[0.65rem] uppercase tracking-widest px-5 py-2.5 bg-[#1B365D] text-[#f5f4ed] hover:bg-[#2a4d87] transition-colors whitespace-nowrap">
                                            {label.clone()}
                                        </a>
                                    }.into_view()
                                }
                            }}
                        </div>

                        // Lead-gate modal (slide-in)
                        {move || show_modal.get().then(|| view! {
                            <div class="mt-0 border border-[#1B365D]/25 bg-[#f5f4ed] px-8 py-8 shadow-lg">
                                <h3 class="font-display text-base font-bold text-[#1B365D] mb-6">
                                    "Access the PDF"
                                </h3>
                                <div class="space-y-5">
                                    <div>
                                        <label class="jetbrains text-[0.6rem] uppercase tracking-widest text-[#6b6a64] block mb-1">
                                            "Name (optional)"
                                        </label>
                                        <input
                                            id="pdf-lead-name"
                                            type="text"
                                            placeholder="Your name"
                                            prop:value=lead_name
                                            on:input=move |ev| set_lead_name.set(event_target_value(&ev))
                                            class="w-full bg-transparent border-b border-[#1B365D]/30 focus:border-[#1B365D] outline-none py-2 font-display text-sm text-[#141413] placeholder:text-[#6b6a64]/60"
                                        />
                                    </div>
                                    <div>
                                        <label class="jetbrains text-[0.6rem] uppercase tracking-widest text-[#6b6a64] block mb-1">
                                            "Email *"
                                        </label>
                                        <input
                                            id="pdf-lead-email"
                                            type="email"
                                            placeholder="you@example.com"
                                            prop:value=lead_email
                                            on:input=move |ev| set_lead_email.set(event_target_value(&ev))
                                            class="w-full bg-transparent border-b border-[#1B365D]/30 focus:border-[#1B365D] outline-none py-2 font-display text-sm text-[#141413] placeholder:text-[#6b6a64]/60"
                                        />
                                    </div>
                                    <div class="flex justify-between items-center pt-2">
                                        <button
                                            on:click=move |_| set_show_modal.set(false)
                                            class="jetbrains text-[0.6rem] uppercase text-[#6b6a64] hover:text-[#141413] transition-colors">
                                            "Cancel"
                                        </button>
                                        <button
                                            id="pdf-lead-submit"
                                            on:click=move |_| submit_action.dispatch(())
                                            disabled=submitting
                                            class="jetbrains text-[0.65rem] uppercase tracking-widest px-6 py-3 bg-[#1B365D] text-[#f5f4ed] hover:bg-[#2a4d87] transition-colors disabled:opacity-50">
                                            {move || if submitting.get() { "⏳ Sending..." } else { "↓ Send & Download" }}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        })}
                    </div>
                }.into_view()
            }}
        </Suspense>
    }
}
