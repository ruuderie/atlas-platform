#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use anchor::app::*;
    use anchor::state::AppState;
    use axum::http::{header, HeaderValue};
    use axum::Router;
    use leptos::context::provide_context;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::PgPool;
    use tower::ServiceBuilder;
    use tower_http::services::ServeDir;
    use tower_http::set_header::SetResponseHeaderLayer;

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Initialize Database
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Migrations are now managed structurally by the backend via SeaORM.
    // We no longer run sqlx migrations on boot here.

    let app_state = AppState {
        leptos_options: leptos_options.clone(),
        pool,
    };

    let site_root = leptos_options.site_root.clone();

    let (prometheus_layer, metric_handle) = axum_prometheus::PrometheusMetricLayer::pair();

    let app = Router::new()
        .route(
            "/metrics",
            axum::routing::get(|| async move { metric_handle.render() }),
        )
        .route(
            "/api/{*fn_name}",
            axum::routing::get(leptos_axum::handle_server_fns).post(leptos_axum::handle_server_fns),
        )
        .route(
            "/robots.txt",
            axum::routing::get({
                let site_root = site_root.clone();
                move || {
                    let path = format!("{}/robots.txt", site_root);
                    async move { std::fs::read_to_string(path).unwrap_or_default() }
                }
            }),
        )
        .route(
            "/sitemap.xml",
            axum::routing::get({
                let site_root = site_root.clone();
                move || {
                    let path = format!("{}/sitemap.xml", site_root);
                    async move {
                        (
                            [(axum::http::header::CONTENT_TYPE, "application/xml")],
                            std::fs::read_to_string(path).unwrap_or_default(),
                        )
                    }
                }
            }),
        )
        .route(
            "/api/blog/{slug}/pdf",
            axum::routing::get(anchor::handlers::blog_pdf::blog_pdf::blog_pdf_handler),
        )
        // /pkg assets (JS, WASM, CSS) must never be served stale.
        // Cache-Control: no-store forces the browser to re-fetch on every
        // navigation, which is the only reliable solution when file names
        // are stable across builds (no content-hash in filename).
        .nest_service(
            "/pkg",
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    HeaderValue::from_static("no-store"),
                ))
                .service(ServeDir::new(format!("{}/pkg", site_root))),
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
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<
            anchor::state::AppState,
            _,
        >(shell))
        .layer(prometheus_layer)
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            extract_tenant_header,
        ))
        .layer(axum::Extension(app_state.clone()))
        .with_state(app_state);

    leptos::logging::log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(feature = "ssr")]
pub fn shell(options: leptos::prelude::LeptosOptions) -> impl leptos::IntoView {
    use leptos::prelude::*;
    use leptos_meta::MetaTags;

    use anchor::app::App;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[cfg(feature = "ssr")]
async fn extract_tenant_header(
    axum::extract::State(state): axum::extract::State<anchor::state::AppState>,
    headers: axum::http::HeaderMap,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, axum::http::StatusCode> {
    use anchor::state::TenantContext;
    use std::str::FromStr;
    use uuid::Uuid;

    let mut tenant_id = if let Some(tenant_str) = headers
        .get("x-tenant-id")
        .and_then(|h: &axum::http::HeaderValue| h.to_str().ok())
    {
        if tenant_str.eq_ignore_ascii_case("null") || tenant_str.is_empty() {
            None
        } else {
            Uuid::from_str(tenant_str).ok()
        }
    } else {
        None
    };

    if tenant_id.is_none() {
        if let Some(host) = headers.get("host").and_then(|h| h.to_str().ok()) {
            let domain = host.split(':').next().unwrap_or(host).to_string();
            use sqlx::Row;
            let row = sqlx::query(
                "SELECT t.id as tenant_id 
                 FROM app_domains ad 
                 JOIN app_instances ai ON ad.app_instance_id = ai.id 
                 JOIN tenant t ON ai.tenant_id = t.id 
                 WHERE ad.domain_name = $1",
            )
            .bind(domain)
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten();

            if let Some(r) = row {
                if let Ok(id) = r.try_get("tenant_id") {
                    tenant_id = Some(id);
                }
            }
        }
    }

    if tenant_id.is_none() {
        match std::env::var("DEFAULT_TENANT_ID") {
            Ok(val) => tenant_id = Uuid::from_str(&val).ok(),
            Err(_) => {}
        }
    }

    // Strict rejection in production if tenant is completely missing
    if tenant_id.is_none() && std::env::var("DEFAULT_TENANT_ID").is_err() {
        // For development server fallback if needed, uncomment this to temporarily bypass:
        // return Ok(next.run(req).await);
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }

    req.extensions_mut().insert(TenantContext(tenant_id));

    Ok(next.run(req).await)
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
}

// Trigger build
