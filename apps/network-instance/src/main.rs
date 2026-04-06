#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use atlas_network_instance::app::*;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .route("/sitemap.xml", axum::routing::get(sitemap_handler))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(feature = "ssr")]
async fn sitemap_handler(headers: axum::http::HeaderMap) -> impl axum::response::IntoResponse {
    use axum::http::{StatusCode, header};
    let host = headers.get("host").and_then(|h| h.to_str().ok()).unwrap_or("localhost");
    let xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <url>
        <loc>https://{}/</loc>
        <priority>1.0</priority>
    </url>
    <url>
        <loc>https://{}/ct-contractor-demo</loc>
        <priority>0.8</priority>
    </url>
</urlset>"#, host, host);

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/xml")],
        xml
    )
}

#[cfg(feature = "ssr")]
pub fn shell(options: leptos::prelude::LeptosOptions) -> impl leptos::IntoView {
    use leptos::prelude::*;
    use leptos_meta::MetaTags;
    use atlas_network_instance::app::App;

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

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
}
