#![recursion_limit = "512"]

#[cfg(feature = "ssr")]
use axum::http::{header, HeaderValue};
#[cfg(feature = "ssr")]
use axum::response::IntoResponse as AxumIntoResponse;

use folio::app::App;
#[cfg(feature = "ssr")]
use folio::state::{AppState, AtlasApiUrl, PublicApiBaseUrl};
use leptos::prelude::*;
#[cfg(feature = "ssr")]
use leptos_axum::{generate_route_list, LeptosRoutes};
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

    let atlas_api_url =
        std::env::var("ATLAS_API_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let public_api_base_url =
        std::env::var("PUBLIC_API_BASE_URL").unwrap_or_else(|_| atlas_api_url.clone());

    // Warm the container→API path before accepting traffic. Uses the env-provided
    // ATLAS_API_URL only (Compose `backend` / K8s Service DNS) — no URL overrides.
    // Avoids magic-link /verify racing a still-settling Docker DNS route after refresh.
    wait_for_atlas_api(&atlas_api_url).await;

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        atlas_api_url: AtlasApiUrl(atlas_api_url.clone()),
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
        // ── Passkeys: proxy to Atlas (must be BEFORE Leptos /api/{*fn_name}) ───
        // Browser WebAuthn posts same-origin `/api/passkeys/*` with the session
        // cookie. Without this proxy, Leptos treats the path as a missing server
        // fn and returns HTTP 400 ("Could not find a server function…").
        .route(
            "/api/passkeys",
            axum::routing::any(proxy_passkeys_to_atlas),
        )
        .route(
            "/api/passkeys/{*path}",
            axum::routing::any(proxy_passkeys_to_atlas),
        )
        .route(
            "/api/{*fn_name}",
            axum::routing::get(leptos_axum::handle_server_fns).post(leptos_axum::handle_server_fns),
        )
        // Static assets — long-lived immutable cache is only safe when the
        // hashed/versioned filename changes on every deploy. Locally the name
        // stays `folio-v1.*`, so immutable caching traps browsers on stale WASM
        // across `atlas-local refresh folio` (hard refresh still keeps it).
        .nest_service(
            "/pkg",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    {
                        let env = std::env::var("ENVIRONMENT").unwrap_or_default();
                        let local = env.is_empty()
                            || matches!(
                                env.to_lowercase().as_str(),
                                "local" | "development" | "dev"
                            );
                        if local {
                            HeaderValue::from_static("no-cache, must-revalidate")
                        } else {
                            HeaderValue::from_static("public, max-age=31536000, immutable")
                        }
                    },
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
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Reverse-proxy `/api/passkeys` (+ optional `/{*path}`) to Atlas so browser
/// WebAuthn stays same-origin (session cookie + Origin: folio.localhost).
#[cfg(feature = "ssr")]
async fn proxy_passkeys_to_atlas(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: axum::http::Request<axum::body::Body>,
) -> axum::response::Response {
    let method = req.method().clone();
    let headers = req.headers().clone();
    let path = req.uri().path().to_string();
    let suffix = path
        .strip_prefix("/api/passkeys")
        .unwrap_or("")
        .to_string();
    let url = format!(
        "{}/api/passkeys{}",
        state.atlas_api_url.0.trim_end_matches('/'),
        suffix
    );

    let body = match axum::body::to_bytes(req.into_body(), 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                format!("Failed to read request body: {e}"),
            )
                .into_response();
        }
    };

    let origin = headers
        .get(header::ORIGIN)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .or_else(|| {
            let host = headers.get(header::HOST)?.to_str().ok()?;
            let scheme = if host.ends_with(".localhost")
                || host.starts_with("localhost")
                || host.starts_with("127.")
            {
                "http"
            } else {
                "https"
            };
            Some(format!("{scheme}://{host}"))
        });

    let client = folio::atlas_client::http_client();
    let mut upstream_req = client.request(
        reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap_or(reqwest::Method::POST),
        &url,
    );

    if let Some(origin) = origin.as_deref() {
        upstream_req = upstream_req.header(header::ORIGIN, origin);
    }
    for name in [
        header::COOKIE,
        header::AUTHORIZATION,
        header::CONTENT_TYPE,
        header::HeaderName::from_static("x-forwarded-host"),
        header::HeaderName::from_static("x-forwarded-for"),
    ] {
        if let Some(val) = headers.get(&name) {
            upstream_req = upstream_req.header(&name, val);
        }
    }
    if headers.get("x-forwarded-host").is_none() {
        if let Some(host) = headers.get(header::HOST) {
            upstream_req = upstream_req.header("x-forwarded-host", host);
        }
    }

    match upstream_req.body(body).send().await {
        Ok(upstream) => {
            let status = axum::http::StatusCode::from_u16(upstream.status().as_u16())
                .unwrap_or(axum::http::StatusCode::BAD_GATEWAY);
            let mut builder = axum::response::Response::builder().status(status);
            for (k, v) in upstream.headers().iter() {
                if matches!(
                    k.as_str(),
                    "transfer-encoding" | "connection" | "keep-alive" | "content-length"
                ) {
                    continue;
                }
                builder = builder.header(k, v);
            }
            let bytes = upstream.bytes().await.unwrap_or_default();
            builder
                .body(axum::body::Body::from(bytes))
                .unwrap_or_else(|_| {
                    (
                        axum::http::StatusCode::BAD_GATEWAY,
                        "Failed to build proxied response",
                    )
                        .into_response()
                })
        }
        Err(e) => {
            eprintln!("[folio] passkey proxy to {url} failed: {e}");
            (
                axum::http::StatusCode::BAD_GATEWAY,
                format!("Passkey proxy failed: {e}"),
            )
                .into_response()
        }
    }
}

/// Poll `ATLAS_API_URL/health` until it responds (or we exhaust attempts).
///
/// Does not rewrite the URL — only confirms the already-configured API base is
/// reachable from this process (Docker DNS / cluster Service DNS as set by env).
/// Uses the shared Atlas client so the keep-alive pool is warm before /verify.
#[cfg(feature = "ssr")]
async fn wait_for_atlas_api(atlas_api_url: &str) {
    let health = format!("{}/health", atlas_api_url.trim_end_matches('/'));
    let client = folio::atlas_client::http_client();
    for attempt in 1..=20u8 {
        match client.get(&health).send().await {
            Ok(res) if res.status().is_success() => {
                if attempt > 1 {
                    eprintln!(
                        "[folio] atlas api ready at {health} (after {attempt} attempts)"
                    );
                } else {
                    eprintln!("[folio] atlas api ready at {health}");
                }
                return;
            }
            Ok(res) => {
                eprintln!(
                    "[folio] atlas api {health} returned {} (attempt {attempt}/20)",
                    res.status()
                );
            }
            Err(e) => {
                eprintln!(
                    "[folio] atlas api not reachable yet ({health}, attempt {attempt}/20): {e}"
                );
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }
    eprintln!(
        "[folio] warning: atlas api still unreachable at {health} — continuing; /verify may fail until DNS/API settles"
    );
}

#[cfg(feature = "ssr")]
pub fn shell(options: LeptosOptions, public_api_base_url: String) -> impl IntoView {
    use leptos_meta::{Link, MetaTags, Stylesheet};

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
                // WizardShell CSS — SSR shell head only (not inside <App/>).
                // Must not live in the hydrated component tree: leptos_meta::Style
                // as a WizardShell sibling emits a body marker hydrate never consumes.
                <style>{include_str!("../style/wizard_shell.css")}</style>
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
    headers: axum::http::HeaderMap,
) -> impl AxumIntoResponse {
    use axum::http::StatusCode;
    use axum::response::Response;

    let forwarded_host = headers
        .get("x-forwarded-host")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get(axum::http::header::HOST)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.split(':').next().unwrap_or(s).trim().to_string())
                .filter(|s| !s.is_empty())
        });

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

    // Uses whatever ATLAS_API_URL the environment already sets (Compose service
    // name locally, cluster Service DNS in K8s).
    //
    // IMPORTANT: magic-link verify is NOT idempotent. The backend marks the token
    // used before returning the session. A transport error after the request is
    // accepted + a client retry yields token_already_used and loses the Set-Cookie.
    // Do not retry this POST. Preflight /health is safe to use for dial warmup.
    let atlas_url =
        std::env::var("ATLAS_API_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
    let url = format!("{}/api/auth/magic-link/verify", atlas_url);
    let health_url = format!("{}/health", atlas_url.trim_end_matches('/'));

    let client = folio::atlas_client::http_client();

    // Warm dial path only (idempotent). Retry health, never the verify POST.
    let mut health_ok = false;
    for health_attempt in 1..=3u8 {
        match client.get(&health_url).send().await {
            Ok(res) if res.status().is_success() => {
                health_ok = true;
                break;
            }
            Ok(res) => {
                eprintln!(
                    "[folio] verify: preflight /health status {} (attempt {health_attempt}/3)",
                    res.status()
                );
            }
            Err(e) => {
                eprintln!(
                    "[folio] verify: preflight /health failed (attempt {health_attempt}/3): {e}"
                );
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100 * u64::from(health_attempt)))
            .await;
    }
    if !health_ok {
        eprintln!("[folio] verify: proceeding without healthy preflight");
    }

    let backend_res = match client
        .post(&url)
        .json(&serde_json::json!({ "token": token }))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!(
                "[folio] verify: transport error (not retrying one-time token POST): connect={} timeout={} request={} body={} err={e}",
                e.is_connect(),
                e.is_timeout(),
                e.is_request(),
                e.is_body()
            );
            return error_html(
                "Could not finish sign-in.",
                "The login link may already have been used if the request reached the server. Open Folio and check if you are signed in, or request a new login link.",
            );
        }
    };

    if !backend_res.status().is_success() {
        let body = backend_res.text().await.unwrap_or_default();
        let (title, detail) = match body.strip_prefix("error_code:").unwrap_or("invalid") {
            "token_expired" => (
                "Login link expired.",
                "This link is no longer valid. Please request a new one.",
            ),
            "token_already_used" => (
                "Login link already used.",
                "Each link can only be used once. Please request a new one.",
            ),
            "token_not_found" => (
                "Login link invalid or expired.",
                "This link could not be found. Please request a new one.",
            ),
            _ => (
                "Login link invalid or expired.",
                "error running server function",
            ),
        };
        return error_html(title, detail);
    }

    // Capture Set-Cookie before consuming the body.
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

    let cookie_header = backend_res
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .find_map(|v| {
            let s = v.to_str().ok()?;
            if s.contains("session=") {
                Some(s.to_string())
            } else {
                None
            }
        });

    // Session JSON (tokens are skip_serializing — only `user` is present).
    // Email handoff: Axum /verify cannot write sessionStorage, so set a short-lived
    // readable cookie for WizardShell to skip OTP when /api/folio/me is still 403.
    let verify_json: serde_json::Value = match backend_res.json().await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[folio] verify: response parse error: {e}");
            return error_html(
                "Login link invalid or expired.",
                "Unexpected response from the authentication server.",
            );
        }
    };
    let verified_email = verify_json
        .get("user")
        .and_then(|u| u.get("email"))
        .and_then(|e| e.as_str())
        .map(str::trim)
        .filter(|e| !e.is_empty())
        .map(str::to_string);

    let token = match session_token {
        Some(t) => t,
        None => {
            return error_html(
                "Login link invalid or expired.",
                "No session cookie after verify. Please try again.",
            );
        }
    };

    /// 302 with session (+ optional folio_verified_email) cookies.
    fn redirect_authed(
        dest: &str,
        session_cookie: Option<String>,
        email: Option<&str>,
    ) -> Response {
        let secure = session_cookie
            .as_deref()
            .map(|c| c.split(';').any(|p| p.trim().eq_ignore_ascii_case("Secure")))
            .unwrap_or(false);
        let mut res = Response::builder()
            .status(StatusCode::FOUND)
            .header(header::LOCATION, dest)
            .body(axum::body::Body::empty())
            .unwrap();
        if let Some(cookie) = session_cookie {
            if let Ok(val) = axum::http::HeaderValue::from_str(&cookie) {
                res.headers_mut().append(header::SET_COOKIE, val);
            }
        }
        if let Some(email) = email.filter(|e| !e.is_empty()) {
            // Not HttpOnly — client WizardShell reads this as same-browser handoff.
            // Encode cookie-unsafe chars; emails are mostly safe already.
            let encoded: String = email
                .bytes()
                .flat_map(|b| match b {
                    b'A'..=b'Z'
                    | b'a'..=b'z'
                    | b'0'..=b'9'
                    | b'-'
                    | b'.'
                    | b'_'
                    | b'@'
                    | b'+' => vec![b as char],
                    _ => format!("%{b:02X}").chars().collect(),
                })
                .collect();
            let secure_flag = if secure { "; Secure" } else { "" };
            let handoff = format!(
                "folio_verified_email={encoded}; Path=/; Max-Age=900; SameSite=Lax{secure_flag}"
            );
            if let Ok(val) = axum::http::HeaderValue::from_str(&handoff) {
                res.headers_mut().append(header::SET_COOKIE, val);
            }
        }
        res
    }

    // Second call: GET /api/folio/me with the session token.
    // This is the Folio-specific endpoint that returns has_passkey and
    // onboarding_complete — the verify endpoint does not include these fields.
    #[derive(serde::Deserialize)]
    struct FolioMe {
        has_passkey: bool,
        onboarding_complete: bool,
        #[serde(default)]
        folio_role: folio::auth::FolioRole,
    }

    let me_url = format!("{}/api/folio/me", atlas_url);
    let me_res = {
        let mut last_err = None;
        let mut ok_res = None;
        for attempt in 1..=5u8 {
            let mut req = client
                .get(&me_url)
                .header("Authorization", format!("Bearer {}", token));
            // Atlas mounts Folio routes by Host / X-Forwarded-Host (app_domains).
            // Without this, Host=backend:8000 returns 404 instead of 403/200.
            if let Some(host) = forwarded_host.as_deref() {
                req = req.header("x-forwarded-host", host);
            }
            match req.send().await {
                Ok(r) => {
                    if attempt > 1 {
                        eprintln!(
                            "[folio] verify: /api/folio/me reachable on attempt {attempt}/5"
                        );
                    }
                    ok_res = Some(r);
                    break;
                }
                Err(e) => {
                    eprintln!(
                        "[folio] verify: /api/folio/me unreachable (attempt {attempt}/5): {e}"
                    );
                    last_err = Some(e);
                    if attempt < 5 {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            200 * u64::from(attempt),
                        ))
                        .await;
                    }
                }
            }
        }
        match ok_res {
            Some(r) => r,
            None => {
                let detail = last_err
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "unknown".into());
                eprintln!("[folio] verify: /api/folio/me unreachable after retries: {detail}");
                // Session cookie may already be valid — send to onboarding rather
                // than pretending the magic link itself was invalid.
                return redirect_authed(
                    "/onboarding",
                    cookie_header,
                    verified_email.as_deref(),
                );
            }
        }
    };

    // If /api/folio/me returns 4xx (e.g. 403 = no RBAC role assigned yet for a
    // freshly-provisioned user), we still have a valid session — don't block login.
    // Send to /onboarding so the wizard can complete setup that seeds the role.
    if !me_res.status().is_success() {
        let status = me_res.status();
        eprintln!("[folio] verify: /api/folio/me returned {status} — redirecting to /onboarding");
        return redirect_authed(
            "/onboarding",
            cookie_header,
            verified_email.as_deref(),
        );
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
        me.folio_role.home_path()
    };

    redirect_authed(dest, cookie_header, verified_email.as_deref())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // WASM entry point is handled by lib.rs::hydrate() via HydrationScripts.
}
