/* 
 * TODO(next-developer): MIGRATION TO AtlasApp API TRAIT REQUIRED
 * 
 * This legacy application currently has its routes, migrations, and background jobs
 * hardcoded into the global Atlas platform core. 
 * 
 * We have introduced a strict, standardized Rust API trait: `AtlasApp` 
 * located at `backend/src/traits/atlas_app.rs`. 
 * 
 * Future work requires refactoring this app to implement the `AtlasApp` trait 
 * (providing perfect encapsulation for its Axum Router, SeaORM Migrations, and Background Jobs) 
 * instead of manually merging them globally.
 * 
 * See the full integration protocol at: `docs/atlas_app_integration.md`
 */
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use crate::components::footer::Footer;
use crate::components::nav::Nav;
use crate::pages::admin::Admin;
use crate::pages::bitcoin::BitcoinDashboard;
use crate::pages::blog::Blog;
use crate::pages::book::BookDiscovery;
use crate::pages::dynamic_landing::{DynamicLanding, DynamicHomeLanding};
use crate::pages::dynamic_entry::DynamicEntry;
use crate::pages::landing::Landing;
use crate::pages::legal::{Privacy, Terms};

#[cfg(feature = "ssr")]
static PAGE_VIEW_CACHE: std::sync::OnceLock<moka::future::Cache<String, bool>> =
    std::sync::OnceLock::new();

#[cfg(feature = "ssr")]
fn get_view_cache() -> moka::future::Cache<String, bool> {
    PAGE_VIEW_CACHE
        .get_or_init(|| {
            moka::future::Cache::builder()
                .time_to_live(std::time::Duration::from_secs(3600))
                .max_capacity(10_000)
                .build()
        })
        .clone()
}

#[server(RecordPageView, "/api")]
pub async fn record_page_view(path: String) -> Result<(), ServerFnError> {
    use axum::http::HeaderMap;
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();

    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    let cache_key = format!("{}:{}:{}", ip, user_agent, path);
    let cache = get_view_cache();

    if cache.contains_key(&cache_key) {
        return Ok(());
    }
    cache.insert(cache_key, true).await;

    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tenant_id = tenant.0.unwrap_or_default();

    let _ = sqlx::query("INSERT INTO page_views (id, tenant_id, path, user_agent) VALUES ($1, $2, $3, $4)")
        .bind(uuid::Uuid::new_v4())
        .bind(tenant_id)
        .bind(path)
        .bind(user_agent)
        .execute(&state.pool)
        .await;
    Ok(())
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let settings_resource = create_resource(|| (), |_| crate::pages::landing::get_site_settings());

    view! {
        <Html lang="en"/>
        <Body class="text-on-surface selection:bg-secondary-container selection:text-on-secondary-container"/>
        <Link rel="preconnect" href="https://fonts.googleapis.com" />
        <Link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous" />
        <Link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&family=Newsreader:opsz,wght@6..72,400;6..72,500;6..72,600;6..72,700&family=JetBrains+Mono:wght@400;500;600&display=swap" />
        <Link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:wght,FILL@100..700,0..1&display=swap" />
        <Script src="https://cdn.jsdelivr.net/npm/mermaid@10.9.1/dist/mermaid.min.js"/>
        <Script>
            "window.addEventListener('load', () => { mermaid.initialize({ startOnLoad: false, theme: 'dark' }); });"
            "window.renderMermaid = function() { setTimeout(function() { try { "
            "document.querySelectorAll('pre > code.language-mermaid').forEach(el => { "
            "let div = document.createElement('div'); div.className = 'mermaid'; div.textContent = el.textContent; "
            "el.parentElement.replaceWith(div); }); "
            "mermaid.run({ querySelector: '.mermaid' }); } catch(e) {} }, 100); };"
        </Script>
        <Stylesheet id="leptos" href="/pkg/anchor.css"/>

        {
            let title_sig = move || settings_resource.get().and_then(Result::ok).map(|s| s.meta_title).unwrap_or("Ruud Salym Erie - Technical Architect".into());
            let desc_sig = move || settings_resource.get().and_then(Result::ok).map(|s| s.meta_description).unwrap_or("Technical Architect and Software Engineer specializing in Rust, Salesforce, and high-performance enterprise applications.".into());
            let og_image_sig = move || settings_resource.get().and_then(Result::ok).map(|s| s.og_image).unwrap_or("".into());

            view! {
                <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
                <Title text=title_sig/>
                <Meta name="description" content=desc_sig/>
                <Meta property="og:title" content=title_sig/>
                <Meta property="og:description" content=desc_sig/>
                <Meta property="og:type" content="website"/>
                <Meta property="og:image" content=og_image_sig/>
                <Meta name="twitter:card" content="summary_large_image"/>
                <Meta name="twitter:title" content=title_sig/>
                <Meta name="twitter:description" content=desc_sig/>
                <Meta name="twitter:image" content=og_image_sig/>

                <Suspense fallback=move || view! {}>
                    {move || {
                        let settings = settings_resource.get().unwrap_or(Ok(crate::pages::landing::SiteSettings::default())).unwrap_or_default();
                        let gcode = settings.google_analytics_id;
                        if !gcode.is_empty() {
                            let gurl = format!("https://www.googletagmanager.com/gtag/js?id={}", gcode);
                            let ascript = format!("window.dataLayer = window.dataLayer || []; function gtag(){{dataLayer.push(arguments);}} gtag('js', new Date()); gtag('config', '{}');", gcode);
                            view! {
                                <Script src=gurl />
                                <Script>{ascript}</Script>
                            }.into_view()
                        } else {
                            view! {}.into_view()
                        }
                    }}
                </Suspense>
            }
        }

        <Suspense fallback=move || view! { <div class="min-h-screen bg-surface"></div> }>
            {move || {
                let settings = settings_resource.get().unwrap_or(Ok(crate::pages::landing::SiteSettings::default())).unwrap_or_default();
                
                let theme_color = settings.theme_primary_color.clone().unwrap_or_else(|| "var(--color-primary, #004c6c)".to_string());
                let design_config = settings.design_config.clone().unwrap_or_default();
                
                provide_context(design_config);

                view! {
                    <crate::components::theme_provider::ThemeProvider primary_color=move || theme_color.clone()>
                        <Router>
                            <Nav />
                            {
                                view! { <PageViewTracker /> }
                            }
                            <Routes>
                                <Route path="/" view=DynamicHomeLanding/>
                                <Route path="/legacy" view=Landing/>
                                <Route path="/resume" view=|| view! { <Redirect path="/p/resume" /> }/>
                                <Route path="/work" view=|| view! { <Redirect path="/p/resume" /> }/>
                                <Route path="/projects" view=|| view! { <Redirect path="/p/projects" /> }/>
                                <Route path="/blog" view=Blog/>
                                <Route path="/certifications" view=|| view! { <Redirect path="/p/certifications" /> }/>
                                <Route path="/investments/real-estate" view=|| view! { <Redirect path="/p/real-estate-ventures" /> }/>
                                <Route path="/investments/bitcoin" view=BitcoinDashboard/>
                                <Route path="/services" view=|| view! { <Redirect path="/p/consulting" /> }/>
                                <Route path="/book" view=BookDiscovery/>
                                <Route path="/terms" view=Terms/>
                                <Route path="/privacy" view=Privacy/>
                                <Route path="/p/*slug" view=DynamicLanding/>
                                <Route path="/e/*slug" view=DynamicEntry/>
                                <Route path="/admin" view=Admin/>
                                <Route path="/*any" view=|| view! { <div class="pt-32 px-[8.5rem]">"Not Found"</div> }/>
                            </Routes>
                            <Footer />
                        </Router>
                    </crate::components::theme_provider::ThemeProvider>
                }.into_view()
            }}
        </Suspense>
    }
}

#[component]
pub fn PageViewTracker() -> impl IntoView {
    let location = use_location();
    create_effect(move |_| {
        let path = location.pathname.get();
        spawn_local(async move {
            let _ = record_page_view(path).await;
        });
    });
    view! { <div class="hidden"></div> }
}
