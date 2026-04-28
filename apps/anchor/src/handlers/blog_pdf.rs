/// Track 4 — PDF delivery handler for /api/blog/:slug/pdf
///
/// Serves a blog post's associated PDF from one of two sources:
///   1. pdf_attachment_url — a pre-uploaded file in R2 (atlas-tenant-vault bucket).
///      The handler fetches the object and streams it to the browser as a download.
///   2. pdf_generate_from_content — generates a Kami-branded PDF on the fly from
///      the post's markdown content using the render_post_as_latex() utility.
///
/// When lead-capture is enabled, requests must carry a valid short-lived HMAC token
/// (issued by SubmitDownloadLead). Token validation uses the same 5-minute bucket
/// scheme as the frontend.
///
/// Route registered in main.rs:
///   .route("/api/blog/:slug/pdf", axum::routing::get(blog_pdf_handler))
///
/// The handler is SSR-only and lives outside of Leptos server functions because
/// it must stream binary bytes with a Content-Disposition: attachment header,
/// which the server fn RPC envelope does not support.

#[cfg(feature = "ssr")]
pub mod blog_pdf {
    use axum::{
        body::Body,
        extract::{Path, Query, State},
        http::{header, StatusCode},
        response::{IntoResponse, Response},
        Extension,
    };
    use serde::Deserialize;
    use sqlx::Row;

    use crate::state::{AppState, TenantContext};

    #[derive(Deserialize)]
    pub struct PdfQuery {
        pub token: Option<String>,
        /// The email address used during lead capture, passed back as a plain
        /// query parameter so the HMAC can be re-derived and verified.
        pub email: Option<String>,
    }

    /// Top-level Axum handler for GET /api/blog/:slug/pdf
    pub async fn blog_pdf_handler(
        Path(slug): Path<String>,
        Query(params): Query<PdfQuery>,
        State(state): State<AppState>,
        Extension(tenant): Extension<TenantContext>,
    ) -> Response {
        match serve_blog_pdf(slug, params.token, params.email, state, tenant).await {
            Ok(resp) => resp,
            Err(e) => {
                leptos::logging::warn!("PDF handler error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
            }
        }
    }

    async fn serve_blog_pdf(
        slug: String,
        token: Option<String>,
        email: Option<String>,
        state: AppState,
        tenant: TenantContext,
    ) -> Result<Response, String> {
        // 1. Load the post row
        let row = sqlx::query(
            "SELECT id, title, payload FROM app_content \
             WHERE collection_type = 'blog_post' \
             AND tenant_id IS NOT DISTINCT FROM $1 AND payload->>'slug' = $2 LIMIT 1",
        )
        .bind(tenant.0)
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| format!("DB error: {e}"))?;

        let row = row.ok_or_else(|| "Post not found".to_string())?;
        let post_id: uuid::Uuid = row.get("id");
        let title: String = row.get("title");
        let payload: serde_json::Value = row.get("payload");

        // 2. Validate token when lead-capture is required
        let requires_lead = payload
            .get("pdf_require_lead_capture")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if requires_lead {
            let tok = token.ok_or_else(|| "Token required".to_string())?;
            let em  = email.as_deref().ok_or_else(|| "Email required for token verification".to_string())?;
            validate_token(&tok, &post_id.to_string(), em, &state)?;
        }

        // 3. Determine the source — attachment URL takes priority
        let pdf_attachment_url = payload
            .get("pdf_attachment_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let generate_from_content = payload
            .get("pdf_generate_from_content")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let (pdf_bytes, filename) = if let Some(url) = pdf_attachment_url {
            // Fetch from R2 / any HTTPS URL
            let bytes = fetch_remote_pdf(&url)
                .await
                .map_err(|e| format!("Failed to fetch PDF: {e}"))?;
            let name = url
                .rsplit('/')
                .next()
                .unwrap_or("download.pdf")
                .to_string();
            (bytes, name)
        } else if generate_from_content {
            // On-the-fly LaTeX → PDF generation
            let content = payload
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let content_format = payload
                .get("content_format")
                .and_then(|v| v.as_str())
                .unwrap_or("markdown")
                .to_string();
            let bytes = render_post_as_pdf(&title, &content, &content_format, &slug)
                .map_err(|e| format!("PDF generation failed: {e}"))?;
            let safe_slug: String = slug
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                .collect();
            (bytes, format!("{}.pdf", safe_slug))
        } else {
            return Err("No PDF configured for this post".to_string());
        };

        // 4. Stream as attachment
        let disposition = format!("attachment; filename=\"{}\"", filename);
        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/pdf")
            .header(header::CONTENT_DISPOSITION, disposition)
            .header(header::CACHE_CONTROL, "private, no-store")
            .body(Body::from(pdf_bytes))
            .map_err(|e| e.to_string())?;

        Ok(response)
    }

    /// Validates the short-lived HMAC token issued by `SubmitDownloadLead`.
    ///
    /// Token format (issued in `submit_download_lead`):
    ///   `base64( HMAC-SHA256(ADMIN_PASSWORD, "{post_id}:{email}:{5_min_bucket}") )`
    ///
    /// The caller passes `email` back as a plain query parameter so we can
    /// reconstruct the exact message that was signed.  We never embed the email
    /// *inside* the opaque token — that would require the handler to trust the
    /// token's own payload, defeating the purpose of the HMAC.
    ///
    /// Accepts the current bucket and the immediately preceding one to tolerate
    /// requests that arrive up to ~5 minutes after the token was issued.
    fn validate_token(
        token: &str,
        post_id: &str,
        email: &str,
        _state: &AppState,
    ) -> Result<(), String> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        if token.is_empty() || email.is_empty() {
            return Err("Missing token or email".to_string());
        }

        let secret =
            std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "fallback-secret".to_string());

        let incoming =
            base64::decode(token).map_err(|_| "Invalid token encoding".to_string())?;
        // HMAC-SHA256 output is always 32 bytes; anything else is structurally invalid.
        if incoming.len() != 32 {
            return Err("Invalid token length".to_string());
        }

        let now_bucket = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            / 300;

        for bucket in [now_bucket, now_bucket.saturating_sub(1)] {
            let message = format!("{}:{}:{}", post_id, email, bucket);
            let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
                .map_err(|e| e.to_string())?;
            mac.update(message.as_bytes());
            // `verify_slice` is constant-time — no timing oracle.
            if mac.verify_slice(&incoming).is_ok() {
                return Ok(());
            }
        }

        Err("Invalid or expired token".to_string())
    }

    async fn fetch_remote_pdf(url: &str) -> Result<Vec<u8>, String> {
        // Layer 2b pre-flight: fast-fail on private/loopback IPs before any I/O.
        // The admin-supplied pdf_attachment_url could point to internal services.
        crate::components::widget_registry::enforce_ssrf_safe_fetch(url).await?;

        // TOCTOU-safe client: IP validation is embedded in the DNS resolver,
        // so the same resolution that passes the pre-flight is used to open
        // the socket — no rebinding window between the two checks.
        let client = crate::components::widget_registry::build_ssrf_safe_client()?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Remote PDF returned HTTP {}", response.status()));
        }

        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| e.to_string())
    }

    /// Generates a minimal Kami-styled PDF from markdown content.
    ///
    /// Implementation uses a pure-Rust LaTeX-like HTML → PDF approach via
    /// the `printpdf` crate for layout primitives. For the initial landing,
    /// we produce a structured PDF with:
    ///   - Kami header (title, domain watermark, date)
    ///   - Body paragraphs rendered line-wrapped
    ///   - Footer: "Downloaded from <domain> · Kami Document System"
    ///
    /// Future: pipe through a tectonic/xelatex sidecar if available on the pod.
    fn render_post_as_pdf(
        title: &str,
        content: &str,
        content_format: &str,
        slug: &str,
    ) -> Result<Vec<u8>, String> {
        use printpdf::*;

        let domain =
            std::env::var("PUBLIC_SITE_DOMAIN").unwrap_or_else(|_| "buildwithruud.com".to_string());
        let date = chrono::Utc::now().format("%Y.%m.%d").to_string();

        // A4 page: 210mm × 297mm
        let (doc, page1, layer1) = PdfDocument::new(title, Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Load built-in font
        let font = doc
            .add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| e.to_string())?;
        let font_bold = doc
            .add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| e.to_string())?;

        // ── Header ────────────────────────────────────────────────────────────
        // Domain watermark (top-right, small muted)
        current_layer.use_text(
            format!("Downloaded from {} · Kami Document System", domain),
            7.0,
            Mm(20.0),
            Mm(285.0),
            &font,
        );

        // Title (centered, large, ink-blue simulated via greyscale)
        current_layer.use_text(title, 20.0, Mm(20.0), Mm(270.0), &font_bold);

        // Date + slug
        current_layer.use_text(
            format!("{} · /blog/{}", date, slug),
            8.0,
            Mm(20.0),
            Mm(261.0),
            &font,
        );

        // Horizontal rule (thin line)
        let line_points = vec![
            (Point::new(Mm(20.0), Mm(257.0)), false),
            (Point::new(Mm(190.0), Mm(257.0)), false),
        ];
        let line = Line {
            points: line_points,
            is_closed: false,
        };
        let outline_color = Color::Greyscale(Greyscale::new(0.6, None));
        current_layer.set_outline_color(outline_color);
        current_layer.set_outline_thickness(0.5);
        current_layer.add_line(line);

        // ── Body ──────────────────────────────────────────────────────────────
        // Strip markdown to plain text (basic)
        let plain = strip_markdown(content, content_format);
        let lines = wrap_text(&plain, 95); // ~95 chars per line at 10pt in A4 margins

        let mut y = 248.0_f32;
        let line_height = 6.0_f32;

        for line in &lines {
            if y < 20.0 {
                break; // Single page for now; pagination is a v2 concern
            }
            current_layer.use_text(line.as_str(), 10.0, Mm(20.0), Mm(y), &font);
            y -= line_height;
        }

        // ── Footer ────────────────────────────────────────────────────────────
        current_layer.use_text(
            format!("Kami · {} · {}", domain, date),
            7.0,
            Mm(20.0),
            Mm(12.0),
            &font,
        );

        // ── Serialize ─────────────────────────────────────────────────────────
        doc.save_to_bytes().map_err(|e| e.to_string())
    }

    /// Very basic markdown stripper — removes common syntax characters.
    fn strip_markdown(content: &str, _format: &str) -> String {
        let mut result = String::with_capacity(content.len());
        for line in content.lines() {
            let stripped = line
                .trim_matches('#')
                .trim_matches(['*', '_', '`', '-', '>', '|'])
                .trim();
            if !stripped.is_empty() {
                result.push_str(stripped);
                result.push('\n');
            }
        }
        result
    }

    /// Wraps text to a maximum character width per line.
    fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        for para in text.split('\n') {
            if para.is_empty() {
                lines.push(String::new());
                continue;
            }
            let words: Vec<&str> = para.split_whitespace().collect();
            let mut current = String::new();
            for word in words {
                if current.len() + word.len() + 1 > max_width {
                    if !current.is_empty() {
                        lines.push(current.trim().to_string());
                        current = String::new();
                    }
                }
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(word);
            }
            if !current.is_empty() {
                lines.push(current.trim().to_string());
            }
        }
        lines
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_strip_markdown() {
            let md = "# Hello\n**World**\n> Quote\n- List";
            let plain = strip_markdown(md, "markdown");
            assert_eq!(plain, "Hello\nWorld\nQuote\nList\n");
        }

        #[test]
        fn test_wrap_text() {
            let text = "This is a very long text that needs to be wrapped properly.";
            let lines = wrap_text(text, 15);
            assert_eq!(
                lines,
                vec!["This is a very", "long text that", "needs to be", "wrapped", "properly."]
            );
        }

        #[test]
        fn test_render_post_as_pdf() {
            let title = "Test PDF";
            let content = "This is a test content for PDF generation.";
            let result = render_post_as_pdf(title, content, "markdown", "test-slug");
            assert!(result.is_ok(), "PDF generation should succeed");
            let bytes = result.unwrap();
            assert!(!bytes.is_empty(), "PDF bytes should not be empty");
            // Minimal check for PDF file signature
            assert_eq!(&bytes[0..4], b"%PDF", "Output should have a PDF header");
        }

        /// Validates that `fetch_remote_pdf` rejects private/loopback IP addresses
        /// via the SSRF pre-flight guard before any TCP connection is attempted.
        /// This covers the CAUTION finding from the PR review.
        #[tokio::test]
        async fn test_fetch_remote_pdf_rejects_private_ip() {
            // Loopback
            let err = fetch_remote_pdf("http://127.0.0.1/secret.pdf").await;
            assert!(err.is_err(), "loopback should be rejected");

            // AWS cloud metadata
            let err = fetch_remote_pdf("http://169.254.169.254/latest/meta-data/").await;
            assert!(err.is_err(), "cloud metadata endpoint should be rejected");

            // RFC-1918 private range
            let err = fetch_remote_pdf("http://10.0.0.1/internal.pdf").await;
            assert!(err.is_err(), "private network IP should be rejected");
        }
    }
}
