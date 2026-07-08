#![recursion_limit = "512"]

#[cfg(feature = "ssr")]
use axum::http::{HeaderValue, header};
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

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // WASM entry point is handled by lib.rs::hydrate() via HydrationScripts.
}
