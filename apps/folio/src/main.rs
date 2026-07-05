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
        // Static assets — no-store so CDN never serves stale WASM/JS
        .nest_service(
            "/pkg",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("no-store"),
                ))
                .service(ServeDir::new(format!("{}/pkg", site_root))),
        )
        .nest_service(
            "/assets",
            ServeDir::new(format!("{}/assets", site_root)),
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

    // ── Inline critical CSS: branded loading screen ─────────────────────────────
    // Shown immediately with the HTML before any external asset loads.
    // Dismissed by a tiny inline script once /pkg/folio-v1.css fires its load event.
    // This eliminates the unstyled flash on slow CDN / cold cache loads.
    let loading_style = r#"
        #folio-loader{
            position:fixed;inset:0;z-index:9999;
            background:#070d18;
            display:flex;flex-direction:column;align-items:center;justify-content:center;
            gap:1.25rem;
            transition:opacity .35s ease,visibility .35s ease;
        }
        #folio-loader.fl-done{opacity:0;visibility:hidden;pointer-events:none;}
        .fl-logo{
            font-size:2.25rem;font-weight:800;letter-spacing:-.04em;
            background:linear-gradient(135deg,#06d6a0,#3b82f6);
            -webkit-background-clip:text;-webkit-text-fill-color:transparent;
            background-clip:text;
        }
        .fl-bar{width:120px;height:3px;border-radius:2px;background:rgba(255,255,255,.08);overflow:hidden;}
        .fl-fill{
            height:100%;width:40%;border-radius:2px;
            background:linear-gradient(90deg,#06d6a0,#3b82f6);
            animation:fl-slide 1.2s ease-in-out infinite alternate;
        }
        @keyframes fl-slide{0%{transform:translateX(-100%)}100%{transform:translateX(250%)}}
    "#;
    // Dismiss script: waits for the CSS <link> to fire onload, then fades the loader.
    // Falls back to a 3s timeout so it never blocks the page.
    let dismiss_script = r#"
        (function(){
            var el=document.getElementById('folio-loader');
            if(!el)return;
            var done=function(){el.classList.add('fl-done');};
            var lnk=document.querySelector('link[href*="folio-v1"]');
            if(lnk){lnk.addEventListener('load',done);lnk.addEventListener('error',done);}
            setTimeout(done,3000);
        })();
    "#;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="robots" content="noindex, nofollow"/>
                <title>"Folio – Property Management"</title>
                // ── Branded loading screen (inline, no external deps) ────────────
                <style inner_html=loading_style></style>
                // ── CSS preload — tells browser to fetch at highest priority ─────
                <link rel="preload" as_="style" href="/pkg/folio-v1.css"/>
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
                <Link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200&display=swap"/>
            </head>
            <body>
                // ── Loading screen markup ─────────────────────────────────────
                <div id="folio-loader" aria-hidden="true">
                    <div class="fl-logo">"Folio"</div>
                    <div class="fl-bar"><div class="fl-fill"></div></div>
                </div>
                // ── Dismiss script ────────────────────────────────────────────
                <script inner_html=dismiss_script></script>
                <App/>
            </body>
        </html>
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // WASM entry point is handled by lib.rs::hydrate() via HydrationScripts.
}
