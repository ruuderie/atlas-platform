#![recursion_limit = "512"]

#[cfg(feature = "ssr")]
use axum::http::{HeaderValue, header};
#[cfg(feature = "ssr")]
use axum::response::IntoResponse as AxumIntoResponse;

use folio::app::App;
#[cfg(feature = "ssr")]
use folio::state::{AppState, AtlasApiUrl, PublicApiBaseUrl};
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::{LeptosRoutes, generate_route_list};
#[cfg(feature = "ssr")]
use tower::ServiceBuilder;
#[cfg(feature = "ssr")]
use tower_http::compression::CompressionLayer;
#[cfg(feature = "ssr")]
use tower_http::services::ServeDir;
#[cfg(feature = "ssr")]
use tower_http::set_header::SetResponseHeaderLayer;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    eprintln!("[folio] log level: {rust_log}");

    // cargo-leptos writes the leptos.toml next to the binary in the site root.
    let conf = leptos_config::get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options.clone();
    let addr = leptos_options.site_addr;
    let site_root = leptos_options.site_root.clone();

    let atlas_api_url = std::env::var("ATLAS_API_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    let public_api_base_url = std::env::var("PUBLIC_API_BASE_URL")
        .unwrap_or_else(|_| atlas_api_url.clone());

    let app_state = AppState {
        leptos_options:      leptos_options.clone(),
        atlas_api_url:       AtlasApiUrl(atlas_api_url.clone()),
        public_api_base_url: PublicApiBaseUrl(public_api_base_url.clone()),
    };

    let routes = generate_route_list(App);

    let app = axum::Router::new()
        .route(
            "/api/health",
            axum::routing::get(|| async { axum::http::StatusCode::OK }),
        )
        // ── Magic-link verify: SSR-only Axum route ───────────────────────────────
        // Must be registered BEFORE leptos_routes so it intercepts GET /verify
        // before the Leptos catch-all. Runs the token exchange server-side,
        // sets the session cookie directly on the HTTP 302 response.
        // The Leptos Verify component only handles the error-state UI.
        .route("/verify", axum::routing::get(verify_handler))
        .route(
            "/api/{*fn_name}",
            axum::routing::get(leptos_axum::handle_server_fns)
                .post(leptos_axum::handle_server_fns),
        )
        // Static assets — versioned filename is the cache-bust key.
        // max-age=31536000,immutable → browser caches indefinitely; never re-requests
        // until the filename changes (which cargo-leptos handles via output-name versioning).
        // CompressionLayer below handles brotli/gzip for WASM, JS, and CSS on the wire.
        .nest_service(
            "/pkg",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=31536000, immutable"),
                ))
                .service(ServeDir::new(format!("{}/pkg", site_root))),
        )
        .nest_service(
            "/assets",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("public, max-age=86400"),
                ))
                .service(ServeDir::new(format!("{}/assets", site_root))),
        )
        .leptos_routes_with_context(
            &app_state,
            routes,
            {
                let app_state = app_state.clone();
                move || provide_context(app_state.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                let public_api_base_url = public_api_base_url.clone();
                move || shell(leptos_options.clone(), public_api_base_url.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>({
            let leptos_options = leptos_options.clone();
            let public_api_base_url = public_api_base_url.clone();
            move |opts| shell(opts, public_api_base_url.clone())
        }))
        .layer(axum::Extension(app_state.clone()))
        // ── Response compression (brotli preferred, gzip fallback) ──────────────
        // Applied to ALL responses: WASM 9.3MB→~2.8MB, CSS 305KB→~30KB.
        // Content-negotiated via Accept-Encoding request header.
        .layer(CompressionLayer::new())
        .with_state(app_state);

    eprintln!("[folio] listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(feature = "ssr")]
pub fn shell(
    options: LeptosOptions,
    public_api_base_url: String,
) -> impl IntoView {
    use leptos_meta::{MetaTags, Stylesheet, Link};

    let env_script = format!(
        "window.__ENV__ = {{ API_BASE_URL: '{}' }};",
        public_api_base_url.trim_end_matches('/')
    );

    let env = std::env::var("ENVIRONMENT").unwrap_or_default();
    let is_deployed = !env.is_empty() && env != "local";
    let reload = (!is_deployed).then(|| view! { <AutoReload options=options.clone() /> });

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="robots" content="noindex, nofollow"/>
                <title>"Folio – Property Management"</title>
                // Inline SVG favicon — prevents 404 noise; no separate file needed
                <link rel="icon" type="image/svg+xml" href="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 32 32'%3E%3Crect width='32' height='32' rx='8' fill='%230a0f1a'/%3E%3Cpath d='M16 6 L27 14 L27 26 L19 26 L19 20 L13 20 L13 26 L5 26 L5 14 Z' fill='%2306d6a0'/%3E%3C/svg%3E"/>
                // Inject API base URL before WASM loads
                <script inner_html=env_script></script>
                {reload}
                <HydrationScripts options />
                <MetaTags/>
                // ── Stylesheets ─────────────────────────────────────────────────
                // Leptos does NOT auto-inject the CSS link — it must be explicit.
                // The href must match output-name in Cargo.toml + the pkg/ serve path.
                <Stylesheet id="folio" href="/pkg/folio-v1.css"/>
                // Google Fonts + Material Symbols
                <Link rel="preconnect" href="https://fonts.googleapis.com"/>
                <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous"/>
                <Link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&family=DM+Sans:wght@300;400;500;600;700&display=swap"/>
                <Link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200&display=block"/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

/// SSR-only handler for GET /verify?token=...
///
/// Runs the magic-link token exchange entirely on the server:
///   1. POSTs to the backend `/api/auth/magic-link/verify`
///   2. Reads the `Set-Cookie: session=...` from the backend response
///   3. Returns an HTTP 302 to the right destination with the cookie attached
///
/// Error states are returned as inline HTML — /verify is NOT a Leptos route
/// (removing it fixed the "Overlapping method route" startup panic).
#[cfg(feature = "ssr")]
async fn verify_handler(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl AxumIntoResponse {
    use axum::http::StatusCode;
    use axum::response::Response;

    /// Render a minimal error HTML page for the verify flow.
    fn error_html(title: &str, message: &str) -> Response {
        let body = format!(
            r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8">
<title>Login link error — Folio</title>
<link rel="stylesheet" href="/pkg/folio-v1.css">
</head><body>
<div class="verify-page">
  <div class="verify-error">
    <p><strong>{title}</strong></p>
    <p class="error-detail">{message}</p>
    <a href="/login">Try again →</a>
  </div>
</div></body></html>"#,
        );
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(axum::body::Body::from(body))
            .unwrap()
    }

    let token = match params.get("token") {
        Some(t) if !t.is_empty() => t.clone(),
        _ => {
            return error_html(
                "Login link invalid or expired.",
                "No token found in the link. Please request a new one.",
            );
        }
    };

    let atlas_url = std::env::var("ATLAS_API_URL")
        .unwrap_or_else(|_| "http://localhost:8000".to_string());
    let url = format!("{}/api/auth/magic-link/verify", atlas_url);

    let client = reqwest::Client::new();
    let backend_res = match client
        .post(&url)
        .json(&serde_json::json!({ "token": token }))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[folio] verify: backend unreachable: {e}");
            return error_html(
                "Login link invalid or expired.",
                "Unable to reach the authentication server. Please try again.",
            );
        }
    };

    if !backend_res.status().is_success() {
        let body = backend_res.text().await.unwrap_or_default();
        let (title, detail) = match body.strip_prefix("error_code:").unwrap_or("invalid") {
            "token_expired"      => ("Login link expired.", "This link is no longer valid. Please request a new one."),
            "token_already_used" => ("Login link already used.", "Each link can only be used once. Please request a new one."),
            "token_not_found"    => ("Login link invalid or expired.", "This link could not be found. Please request a new one."),
            _                    => ("Login link invalid or expired.", "error running server function"),
        };
        return error_html(title, detail);
    }

    // Extract the session token from `Set-Cookie: session=TOKEN; ...`
    // The verify endpoint returns it here; we need it both to forward to the
    // browser AND to call /api/folio/me for routing info.
    let session_token = backend_res
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .find_map(|v| {
            let s = v.to_str().ok()?;
            s.split(';')
                .next()
                .and_then(|kv| kv.trim().strip_prefix("session="))
                .map(|t| t.to_string())
        });

    // Rebuild the full Set-Cookie string to forward to the browser
    let cookie_header = backend_res
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .find_map(|v| {
            let s = v.to_str().ok()?;
            if s.contains("session=") { Some(s.to_string()) } else { None }
        });

    let token = match session_token {
        Some(t) => t,
        None => {
            return error_html(
                "Login link invalid or expired.",
                "No session cookie after verify. Please try again.",
            );
        }
    };

    // Second call: GET /api/folio/me with the session token.
    // This is the Folio-specific endpoint that returns has_passkey and
    // onboarding_complete — the verify endpoint does not include these fields.
    #[derive(serde::Deserialize)]
    struct FolioMe {
        has_passkey: bool,
        onboarding_complete: bool,
    }

    let me_url = format!("{}/api/folio/me", atlas_url);
    let me_res = match client
        .get(&me_url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[folio] verify: /api/folio/me unreachable: {e}");
            return error_html(
                "Login link invalid or expired.",
                "Unable to reach the authentication server. Please try again.",
            );
        }
    };

    // If /api/folio/me returns 4xx (e.g. 403 = no RBAC role assigned yet for a
    // freshly-provisioned user), we still have a valid session — don't block login.
    // Send to /onboarding so the wizard can complete setup that seeds the role.
    if !me_res.status().is_success() {
        let status = me_res.status();
        eprintln!("[folio] verify: /api/folio/me returned {status} — redirecting to /onboarding");
        let mut builder = Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, "/onboarding");
        if let Some(cookie) = cookie_header {
            if let Ok(val) = axum::http::HeaderValue::from_str(&cookie) {
                builder = builder.header(header::SET_COOKIE, val);
            }
        }
        return builder.body(axum::body::Body::empty()).unwrap();
    }

    let me: FolioMe = match me_res.json().await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[folio] verify: /api/folio/me parse error: {e}");
            return error_html(
                "Login link invalid or expired.",
                "Unexpected response from the authentication server.",
            );
        }
    };

    let dest = if !me.has_passkey {
        "/auth/passkey-setup"
    } else if !me.onboarding_complete {
        "/onboarding"
    } else {
        "/dashboard"
    };

    // Build redirect with the session cookie attached so the browser is authenticated
    let mut builder = Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, dest);

    if let Some(cookie) = cookie_header {
        if let Ok(val) = axum::http::HeaderValue::from_str(&cookie) {
            builder = builder.header(header::SET_COOKIE, val);
        }
    }

    builder.body(axum::body::Body::empty()).unwrap()
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // WASM entry point is handled by lib.rs::hydrate() via HydrationScripts.
}
