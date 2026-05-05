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
        // ── Fixed top nav bar ──────────────────────────────────────────────────
        <nav class=format!("fixed top-0 left-0 w-full flex justify-between items-center py-4 z-[60] {} {}",
            if design.nav_layout == "floating-glass" { "bg-surface/80 backdrop-blur-[20px] shadow-sm" } else { "bg-surface shadow-[0_2px_12px_rgba(0,0,0,0.08)]" },
            if design.container_strategy == "asymmetrical-gutters" { "px-5 md:px-[8.5rem]" } else { "px-5 md:px-12" }
        )>
            // Logo / site title
            <A href="/" class=format!("text-xl font-bold truncate relative z-[70] {} {}", &design.meta_font, if design.elevation_strategy == "tonal-shifts" { "text-primary" } else { "text-on-surface" })>
                <Suspense fallback=move || view! { <span>"Portfolio"</span> }>
                    {move || settings_resource.get().unwrap_or(Ok(crate::pages::landing::SiteSettings::default())).unwrap_or(crate::pages::landing::SiteSettings::default()).site_title}
                </Suspense>
            </A>

            // ── Desktop nav links (md+) ────────────────────────────────────────
            <div class="hidden md:flex items-center space-x-8">
                <Suspense fallback=move || view! { <div class="w-24 h-4 bg-outline-variant/30 animate-pulse rounded"></div> }>
                    {
                        let root_class = format!("font-medium transition-colors uppercase text-sm tracking-wide {}", if &design.elevation_strategy == "tonal-shifts" { "text-on-surface hover:text-primary" } else { "text-on-surface-variant hover:text-on-surface" });
                        move || {
                            let items = nav_resource.get().unwrap_or(Ok(vec![])).unwrap_or_default();
                            let root_class = root_class.clone();
                            let root_items: Vec<_> = items.iter().filter(|i| i.parent_id.is_none()).collect();

                            // Wrap in a stable single container — prevents Leptos 0.6 dyn_child
                            // hydration panic caused by SSR emitting 0 children (resource pending)
                            // while WASM hydrates with N children once the resource resolves.
                            view! {
                                <div class="flex items-center space-x-8">
                                    {root_items.into_iter().map(|root| {
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
                                                <div class="relative group cursor-pointer font-medium transition-colors uppercase text-sm tracking-wide text-on-surface-variant flex items-center gap-1 z-50">
                                                    <a href=root.href.clone().unwrap_or_else(|| "#".to_string()) class="hover:text-primary block py-2 select-none">
                                                        {root.label.clone()}
                                                    </a>
                                                    <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3 group-hover:rotate-180 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" /></svg>
                                                    <div class="absolute top-full left-0 mt-1 w-52 bg-surface border border-outline-variant/30 shadow-xl opacity-0 invisible group-hover:visible group-hover:opacity-100 transition-all flex flex-col pointer-events-none group-hover:pointer-events-auto rounded-sm">
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
                                    }).collect_view()}
                                </div>
                            }
                        }
                    }
                </Suspense>
            </div>

            // ── Right side: admin icon + widgets (desktop) + hamburger (mobile) ─
            <div class="flex items-center gap-2 md:gap-5 z-[70]">
                // Admin terminal icon — desktop only
                <a href="/admin" class="material-symbols-outlined text-[22px] text-on-surface-variant hover:text-primary cursor-pointer transition-colors hidden md:block">"terminal"</a>
                // Tenant-configured nav widgets — desktop only
                <div class="hidden md:flex items-center gap-2">
                    <Suspense fallback=move || view! { <span class="hidden"></span> }>
                        {move || {
                            let widgets = nav_widgets().into_iter().map(|widget| {
                                view! { <crate::components::widget_registry::WidgetShell widget=widget /> }
                            }).collect_view();
                            view! {
                                <div class="contents">
                                    {widgets}
                                </div>
                            }
                        }}
                    </Suspense>
                </div>
                // Hamburger button — mobile only
                // z-[70] keeps it above the overlay (z-[55]) so the close icon is always tappable
                <button
                    id="mobile-menu-toggle"
                    aria-label="Toggle navigation menu"
                    aria-expanded=move || if mobile_menu_open.get() { "true" } else { "false" }
                    aria-controls="mobile-menu-overlay"
                    on:click=move |_| set_mobile_menu_open.update(|o| *o = !*o)
                    class="md:hidden relative z-[70] flex items-center justify-center w-10 h-10 rounded-lg text-on-surface hover:bg-surface-container-high active:scale-95 transition-all focus:outline-none focus-visible:ring-2 focus-visible:ring-primary"
                >
                    <span class="material-symbols-outlined text-[26px] leading-none select-none">
                        {move || if mobile_menu_open.get() { "close" } else { "menu" }}
                    </span>
                </button>
            </div>
        </nav>

        // ── Mobile menu overlay ────────────────────────────────────────────────
        // Slides in from the right edge. z-[55] keeps it above page content
        // but below the nav bar (z-[60]) so the hamburger/close remains tappable.
        <div
            id="mobile-menu-overlay"
            role="dialog"
            aria-label="Navigation menu"
            aria-hidden=move || if mobile_menu_open.get() { "false" } else { "true" }
            class="fixed inset-0 z-[55] md:hidden flex flex-col bg-surface"
            style=move || if mobile_menu_open.get() {
                "transform: translateX(0); opacity: 1; pointer-events: auto; transition: transform 0.3s cubic-bezier(0.4,0,0.2,1), opacity 0.25s ease;"
            } else {
                "transform: translateX(100%); opacity: 0; pointer-events: none; transition: transform 0.3s cubic-bezier(0.4,0,0.2,1), opacity 0.25s ease;"
            }
        >
            // Scrollable content — top padding clears the fixed nav bar
            <div class="flex flex-col flex-1 overflow-y-auto pt-20 px-6 pb-16">
                // ── Nav links list ──────────────────────────────────────────
                <Suspense fallback=move || view! {
                    <div class="flex flex-col gap-4 pt-6 animate-pulse">
                        <div class="h-9 w-36 bg-outline-variant/20 rounded"></div>
                        <div class="h-9 w-28 bg-outline-variant/20 rounded"></div>
                        <div class="h-9 w-32 bg-outline-variant/20 rounded"></div>
                    </div>
                }>
                    <nav class="flex flex-col">
                        {move || {
                            let items = nav_resource.get().unwrap_or(Ok(vec![])).unwrap_or_default();
                            let root_items: Vec<_> = items.iter().filter(|i| i.parent_id.is_none()).collect();

                            // Same stable-container fix as desktop — prevents dyn_child panic
                            view! {
                                <div class="flex flex-col">
                                    {root_items.into_iter().map(|root| {
                                        let children: Vec<_> = items.iter().filter(|i| i.parent_id == Some(root.id)).collect();

                                        if children.is_empty() {
                                            view! {
                                                <a
                                                    href=root.href.clone().unwrap_or_else(|| "#".to_string())
                                                    on:click=move |_| set_mobile_menu_open.set(false)
                                                    class="py-4 text-2xl font-bold text-on-surface uppercase hover:text-primary transition-colors border-b border-outline-variant/15 last:border-0"
                                                >
                                                    {root.label.clone()}
                                                </a>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <div class="flex flex-col border-b border-outline-variant/15">
                                                    <div class="py-4 text-2xl font-bold text-on-surface-variant uppercase">
                                                        {root.label.clone()}
                                                    </div>
                                                    <div class="flex flex-col pl-4 mb-3 border-l-2 border-primary/25">
                                                        {children.into_iter().map(|child| {
                                                            view! {
                                                                <a
                                                                    href=child.href.clone().unwrap_or_else(|| "#".to_string())
                                                                    on:click=move |_| set_mobile_menu_open.set(false)
                                                                    class="py-3 text-lg font-medium text-on-surface-variant hover:text-primary transition-colors border-b border-outline-variant/10 last:border-0 block"
                                                                >
                                                                    {child.label.clone()}
                                                                </a>
                                                            }
                                                        }).collect_view()}
                                                    </div>
                                                </div>
                                            }.into_view()
                                        }
                                    }).collect_view()}
                                </div>
                            }
                        }}
                    </nav>
                </Suspense>

                // ── Admin terminal shortcut ─────────────────────────────────
                <a
                    href="/admin"
                    on:click=move |_| set_mobile_menu_open.set(false)
                    class="mt-8 flex items-center justify-center gap-2 py-4 px-6 border border-outline-variant/40 text-primary text-sm font-bold uppercase tracking-widest hover:bg-surface-container transition-colors rounded-sm"
                >
                    <span class="material-symbols-outlined text-[18px]">"terminal"</span>
                    <span>"Admin Terminal"</span>
                </a>
            </div>
        </div>
        </>
    }
}
