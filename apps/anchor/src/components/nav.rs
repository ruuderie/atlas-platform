use leptos::*;
use leptos_router::A;

#[server(GetBlockHeight, "/api")]
pub async fn get_block_height() -> Result<Option<u64>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tenant_id = tenant.0.unwrap_or_default();

    let latest_db = sqlx::query(
        "SELECT height FROM bitcoin_blocks WHERE tenant_id = $1 ORDER BY height DESC LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_optional(&state.pool)
    .await?;

    if let Some(row) = latest_db {
        Ok(Some(row.get::<i64, _>("height") as u64))
    } else {
        Ok(None)
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BitcoinStats {
    pub difficulty: f64,
    pub tx_count: i32,
    pub size: i32,
    pub weight: i32,
}

#[server(GetBitcoinStats, "/api")]
pub async fn get_bitcoin_stats() -> Result<BitcoinStats, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tenant_id = tenant.0.unwrap_or_default();

    let row = sqlx::query("SELECT difficulty, tx_count, size, weight FROM bitcoin_blocks WHERE tenant_id = $1 ORDER BY height DESC LIMIT 1")
        .bind(tenant_id)
        .fetch_optional(&state.pool)
        .await?;

    if let Some(r) = row {
        Ok(BitcoinStats {
            difficulty: r.get("difficulty"),
            tx_count: r.get("tx_count"),
            size: r.get("size"),
            weight: r.get("weight"),
        })
    } else {
        Ok(BitcoinStats {
            difficulty: 0.0,
            tx_count: 0,
            size: 0,
            weight: 0,
        })
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct NavItemRecord {
    pub id: uuid::Uuid,
    pub label: String,
    pub href: Option<String>,
    pub parent_id: Option<uuid::Uuid>,
    pub display_order: i32,
    pub is_visible: bool,
}

#[server(GetNavItems, "/api")]
pub async fn get_nav_items() -> Result<Vec<NavItemRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::atlas_client::fetch_atlas_data;

    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let headers = extract::<axum::http::HeaderMap>().await.unwrap_or_default();
    let host = headers.get(axum::http::header::HOST).and_then(|h| h.to_str().ok()).map(|s| s.to_string());
    
    if let Some(tenant_id) = tenant.0 {
        let endpoint = format!("/api/public/menus/{}/tree/header", tenant_id);
        if let Ok(menus) = fetch_atlas_data::<Vec<NavItemRecord>>(&endpoint, Some(tenant_id), host).await {
            return Ok(menus);
        }
    }
    
    Ok(vec![])
}

#[server(GetAllNavItems, "/api")]
pub async fn get_all_nav_items() -> Result<Vec<NavItemRecord>, ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    use crate::atlas_client::fetch_atlas_data;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let headers = extract::<axum::http::HeaderMap>().await.unwrap_or_default();
    let host = headers.get(axum::http::header::HOST).and_then(|h| h.to_str().ok()).map(|s| s.to_string());

    if let Some(tenant_id) = tenant.0 {
        let endpoint = format!("/api/public/menus/{}/tree/header", tenant_id);
        if let Ok(menus) = fetch_atlas_data::<Vec<NavItemRecord>>(&endpoint, Some(tenant_id), host).await {
            return Ok(menus);
        }
    }
    
    Ok(vec![])
}

#[server(AddNavItem, "/api")]
pub async fn add_nav_item(
    label: String,
    href: Option<String>,
    parent_id: Option<uuid::Uuid>,
    display_order: i32,
    is_visible: bool,
) -> Result<(), ServerFnError> {
    Ok(())
}

#[server(UpdateNavItem, "/api")]
pub async fn update_nav_item(
    id: uuid::Uuid,
    label: String,
    href: Option<String>,
    parent_id: Option<uuid::Uuid>,
    display_order: i32,
    is_visible: bool,
) -> Result<(), ServerFnError> {
    Ok(())
}

#[server(DeleteNavItem, "/api")]
pub async fn delete_nav_item(id: uuid::Uuid) -> Result<(), ServerFnError> {
    Ok(())
}

#[component]
pub fn Nav() -> impl IntoView {
    let design = use_context::<crate::pages::landing::DesignConfig>()
        .unwrap_or_default();
        
    let settings_resource = create_resource(|| (), |_| crate::pages::landing::get_site_settings());
    let nav_resource = create_resource(|| (), |_| get_nav_items());
    let (mobile_menu_open, set_mobile_menu_open) = create_signal(false);

    // Derive nav-placement widgets reactively from settings
    let nav_widgets = move || {
        settings_resource
            .get()
            .and_then(|r| r.ok())
            .map(|s| s.widgets.into_iter().filter(|w| w.is_nav_widget()).collect::<Vec<_>>())
            .unwrap_or_default()
    };

    view! {
        <>
        <nav class=format!("fixed top-0 left-0 w-full flex justify-between items-center py-6 z-[60] {} {}",
            if design.nav_layout == "floating-glass" { "bg-surface/80 backdrop-blur-[20px] shadow-sm" } else { "bg-surface shadow-[0_4px_24px_rgba(0,0,0,0.06)]" },
            if design.container_strategy == "asymmetrical-gutters" { "px-4 md:px-[8.5rem]" } else { "px-4 md:px-12" }
        )>
            <A href="/" class=format!("text-xl font-bold truncate relative z-[70] {} {}", &design.meta_font, if design.elevation_strategy == "tonal-shifts" { "text-primary" } else { "text-on-surface" })>
                <Suspense fallback=move || view! { <span>"Portfolio"</span> }>
                    {move || settings_resource.get().unwrap_or(Ok(crate::pages::landing::SiteSettings::default())).unwrap_or(crate::pages::landing::SiteSettings::default()).site_title}
                </Suspense>
            </A>
            <div class="hidden md:flex items-center space-x-8">
                <Suspense fallback=move || view! { <div class="w-24 h-4 bg-slate-200 dark:bg-slate-700 animate-pulse rounded"></div> }>
                    {
                        let root_class = format!("font-medium transition-colors uppercase {}", if &design.elevation_strategy == "tonal-shifts" { "text-on-surface hover:text-primary" } else { "text-on-surface-variant hover:text-on-surface hover:bg-surface-container" });
                        move || {
                        let items = nav_resource.get().unwrap_or(Ok(vec![])).unwrap_or_default();
                        let root_class = root_class.clone();

                        let root_items: Vec<_> = items.iter().filter(|i| i.parent_id.is_none()).collect();

                        root_items.into_iter().map(|root| {
                            let root_class = root_class.clone();
                            let children: Vec<_> = items.iter().filter(|i| i.parent_id == Some(root.id)).collect();

                            if children.is_empty() {
                                view! {
                                    <a href=root.href.clone().unwrap_or_else(|| "#".to_string()) class=root_class>
                                        {root.label.clone()}
                                    </a>
                                }.into_view()
                            } else {
                                view! {
                                    <div class="relative group cursor-pointer text-slate-600 dark:text-slate-400 font-medium transition-colors uppercase flex items-center gap-1 z-50">
                                        <a href=root.href.clone().unwrap_or_else(|| "#".to_string()) class="hover:bg-slate-100/50 dark:hover:bg-slate-800/50 block py-2 select-none">
                                            {root.label.clone()}
                                        </a>
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3 group-hover:rotate-180 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>

                                        <div class="absolute top-full left-0 mt-0 w-48 bg-surface border border-outline-variant/30 shadow-xl opacity-0 invisible group-hover:visible group-hover:opacity-100 transition-all flex flex-col pointer-events-none group-hover:pointer-events-auto">
                                            {children.into_iter().map(|child| {
                                                view! {
                                                    <a href=child.href.clone().unwrap_or_else(|| "#".to_string()) class="block px-4 py-3 text-sm text-on-surface-variant hover:bg-surface-container hover:text-primary transition-colors border-b border-outline-variant/10 last:border-0 uppercase font-medium">
                                                        {child.label.clone()}
                                                    </a>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                }.into_view()
                            }
                        }).collect_view()
                    }}
                </Suspense>
            </div>
            // Right side: hamburger (mobile) + admin icon + tenant widgets (desktop only)
            <div class="flex items-center space-x-4 md:space-x-6 z-50">
                // Hamburger — mobile only, rendered first so it's never obscured by widgets
                <button
                    on:click=move |_| set_mobile_menu_open.update(|o| *o = !*o)
                    class="md:hidden text-primary focus:outline-none flex items-center justify-center p-2 rounded-md hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
                >
                    <span class="material-symbols-outlined text-3xl">
                        {move || if mobile_menu_open.get() { "close" } else { "menu" }}
                    </span>
                </button>
                <a href="/admin" class="material-symbols-outlined text-primary cursor-pointer hover:opacity-80 transition-opacity hidden sm:block">"terminal"</a>
                // Tenant-configured widgets — hidden on mobile to avoid hamburger collision
                // Each WidgetShell dispatches to the correct renderer based on WidgetRenderer variant
                <div class="hidden sm:flex items-center gap-2">
                    <Suspense fallback=move || view! {}>
                        {move || {
                            nav_widgets().into_iter().map(|widget| {
                                view! { <crate::components::widget_registry::WidgetShell widget=widget /> }
                            }).collect_view()
                        }}
                    </Suspense>
                </div>
            </div>

        </nav>

        // Mobile Menu Overlay — z-[55] so it renders above page content but below the
        // fixed nav bar (z-[60]), ensuring the close button remains accessible
        <div
            class="fixed inset-0 bg-surface dark:bg-slate-900 z-[55] flex flex-col pt-32 px-6 transition-all duration-300 ease-in-out md:hidden"
            style=move || if mobile_menu_open.get() { "transform: translateX(0); opacity: 1; pointer-events: auto;" } else { "transform: translateX(100%); opacity: 0; pointer-events: none;" }
        >
            <div class="flex flex-col space-y-8 overflow-y-auto pb-24 h-full">
                <Suspense fallback=move || view! { <div class="w-24 h-4 bg-slate-200 dark:bg-slate-700 animate-pulse rounded"></div> }>
                    {move || {
                        let items = nav_resource.get().unwrap_or(Ok(vec![])).unwrap_or_default();

                        let root_items: Vec<_> = items.iter().filter(|i| i.parent_id.is_none()).collect();

                        root_items.into_iter().map(|root| {
                            let children: Vec<_> = items.iter().filter(|i| i.parent_id == Some(root.id)).collect();

                            if children.is_empty() {
                                view! {
                                    <a href=root.href.clone().unwrap_or_else(|| "#".to_string()) on:click=move |_| set_mobile_menu_open.set(false) class="text-3xl font-bold text-slate-800 dark:text-slate-100 uppercase hover:text-primary transition-colors">
                                        {root.label.clone()}
                                    </a>
                                }.into_view()
                            } else {
                                view! {
                                    <div class="flex flex-col space-y-3 pt-4">
                                        <div class="text-2xl sm:text-3xl font-bold text-slate-400 dark:text-slate-500 uppercase bg-transparent w-full text-left">
                                            {root.label.clone()}
                                        </div>
                                        <div class="flex flex-col space-y-2 pl-4 border-l-2 border-slate-200 dark:border-slate-800">
                                            {children.into_iter().map(|child| {
                                                view! {
                                                    <a href=child.href.clone().unwrap_or_else(|| "#".to_string()) on:click=move |_| set_mobile_menu_open.set(false) class="text-xl font-medium text-slate-600 dark:text-slate-300 hover:text-primary transition-colors block py-4 border-b border-outline-variant/20 last:border-0 w-full text-left">
                                                        {child.label.clone()}
                                                    </a>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                }.into_view()
                            }
                        }).collect_view()
                    }}
                </Suspense>
                <a href="/admin" on:click=move |_| set_mobile_menu_open.set(false) class="mt-8 flex items-center space-x-2 text-primary text-xl font-bold uppercase transition-opacity border p-4 border-outline-variant/30 text-center justify-center">
                    <span class="material-symbols-outlined">"terminal"</span>
                    <span>"Admin Terminal"</span>
                </a>
            </div>
        </div>
        </>
    }
}
