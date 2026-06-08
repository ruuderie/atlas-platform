use axum::http::{HeaderValue, header};
use folio::app::App;
use folio::state::AppState;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(rust_log))
        .init();

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
        leptos_options: leptos_options.clone(),
        atlas_api_url: atlas_api_url.clone(),
        public_api_base_url: public_api_base_url.clone(),
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

    tracing::info!("Folio listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(feature = "ssr")]
pub fn shell(
    options: LeptosOptions,
    public_api_base_url: String,
) -> impl IntoView {
    use leptos_meta::MetaTags;

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
