use leptos::*;
use leptos_router::A;
use std::time::Duration;

#[server(GetBlockHeight, "/api")]
pub async fn get_block_height() -> Result<u64, ServerFnError> {
    use axum::Extension;
    use chrono::Utc;
    use leptos_axum::extract;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    use sqlx::Row;
    let latest_db = sqlx::query(
        "SELECT height, timestamp, fetched_at FROM bitcoin_blocks ORDER BY height DESC LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await?;

    let mut needs_fetch = true;
    let mut current_height = 0;

    if let Some(row) = latest_db {
        current_height = row.get::<i64, _>("height");
        let block_timestamp = row.get::<i64, _>("timestamp");
        let fetched_at = row.get::<chrono::DateTime<Utc>, _>("fetched_at");
        let now = Utc::now();

        let time_since_fetch = now.signed_duration_since(fetched_at).num_seconds();
        let time_since_block = now.timestamp() - block_timestamp;

        // Skip fetching if either:
        // 1. We just fetched within the last 60 seconds (prevents hammering API)
        // 2. OR the block was mined less than 10 mins (600s) ago
        if time_since_fetch < 60 || time_since_block < 600 {
            needs_fetch = false;
        }
    }

    if needs_fetch {
        let url = "https://mempool.space/api/blocks";
        let res = reqwest::get(url).await?;
        let blocks: Vec<serde_json::Value> = res.json().await?;

        if let Some(latest) = blocks.first() {
            current_height = latest["height"].as_i64().unwrap_or(0);
        }

        for block in blocks {
            let id = block["id"].as_str().unwrap_or_default().to_string();
            let height = block["height"].as_i64().unwrap_or(0);
            let version = block["version"].as_i64().unwrap_or(0);
            let timestamp = block["timestamp"].as_i64().unwrap_or(0);
            let tx_count = block["tx_count"].as_i64().unwrap_or(0) as i32;
            let size = block["size"].as_i64().unwrap_or(0) as i32;
            let weight = block["weight"].as_i64().unwrap_or(0) as i32;
            let merkle_root = block["merkle_root"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let previousblockhash = block["previousblockhash"]
                .as_str()
                .unwrap_or_default()
                .to_string();
            let mediantime = block["mediantime"].as_i64().unwrap_or(0);
            let nonce = block["nonce"].as_i64().unwrap_or(0);
            let bits = block["bits"].as_i64().unwrap_or(0);
            let difficulty = block["difficulty"].as_f64().unwrap_or(0.0);

            let _ = sqlx::query(
                r#"
                INSERT INTO bitcoin_blocks (id, height, version, timestamp, tx_count, size, weight, merkle_root, previousblockhash, mediantime, nonce, bits, difficulty)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT (id) DO NOTHING
                "#)
                .bind(id)
                .bind(height)
                .bind(version)
                .bind(timestamp)
                .bind(tx_count)
                .bind(size)
                .bind(weight)
                .bind(merkle_root)
                .bind(previousblockhash)
                .bind(mediantime)
                .bind(nonce)
                .bind(bits)
                .bind(difficulty)
                .execute(&state.pool)
                .await;
        }

        let _ = sqlx::query(
            "DELETE FROM bitcoin_blocks WHERE fetched_at < NOW() - INTERVAL '24 hours'",
        )
        .execute(&state.pool)
        .await;

        let _ = sqlx::query("INSERT INTO api_requests_log (endpoint) VALUES ('mempool_api')")
            .execute(&state.pool)
            .await;
    }

    Ok(current_height as u64)
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

    let row = sqlx::query("SELECT difficulty, tx_count, size, weight FROM bitcoin_blocks ORDER BY height DESC LIMIT 1")
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
    
    if let Some(tenant_id) = tenant.0 {
        let endpoint = format!("/api/public/menus/{}/tree/header", tenant_id);
        if let Ok(menus) = fetch_atlas_data::<Vec<NavItemRecord>>(&endpoint, Some(tenant_id)).await {
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
    
    if let Some(tenant_id) = tenant.0 {
        let endpoint = format!("/api/public/menus/{}/tree/header", tenant_id);
        if let Ok(menus) = fetch_atlas_data::<Vec<NavItemRecord>>(&endpoint, Some(tenant_id)).await {
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
    let (tick, set_tick) = create_signal(0);
    let (mobile_menu_open, set_mobile_menu_open) = create_signal(false);

    create_effect(move |_| {
        let handle = set_interval_with_handle(
            move || set_tick.update(|t| *t += 1),
            Duration::from_secs(60),
        )
        .ok();

        on_cleanup(move || {
            if let Some(h) = handle {
                h.clear();
            }
        });
    });

    let height_resource = create_resource(move || tick.get(), |_| get_block_height());
    let settings_resource = create_resource(|| (), |_| crate::pages::landing::get_site_settings());
    let nav_resource = create_resource(|| (), |_| get_nav_items());

    view! {
        <>
        <nav class="fixed top-0 left-0 w-full flex justify-between items-center px-4 md:px-[8.5rem] py-6 bg-white/80 dark:bg-slate-900/80 backdrop-blur-[20px] z-[60]">
            <A href="/" class="text-xl font-bold font-mono text-cyan-800 dark:text-cyan-400 truncate relative z-[70]">
                <Suspense fallback=move || view! { <span>"Build With Ruud"</span> }>
                    {move || settings_resource.get().unwrap_or(Ok(crate::pages::landing::SiteSettings::default())).unwrap_or(crate::pages::landing::SiteSettings::default()).site_title}
                </Suspense>
            </A>
            <div class="hidden md:flex items-center space-x-8">
                <Suspense fallback=move || view! { <div class="w-24 h-4 bg-slate-200 dark:bg-slate-700 animate-pulse rounded"></div> }>
                    {move || {
                        let items = nav_resource.get().unwrap_or(Ok(vec![])).unwrap_or_default();

                        let root_items: Vec<_> = items.iter().filter(|i| i.parent_id.is_none()).collect();

                        root_items.into_iter().map(|root| {
                            let children: Vec<_> = items.iter().filter(|i| i.parent_id == Some(root.id)).collect();

                            if children.is_empty() {
                                view! {
                                    <a href=root.href.clone().unwrap_or_else(|| "#".to_string()) class="text-slate-600 dark:text-slate-400 font-medium hover:bg-slate-100/50 dark:hover:bg-slate-800/50 transition-colors uppercase">
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

                                        <div class="absolute top-full left-0 mt-0 w-48 bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-800 shadow-xl opacity-0 invisible group-hover:visible group-hover:opacity-100 transition-all flex flex-col pointer-events-none group-hover:pointer-events-auto">
                                            {children.into_iter().map(|child| {
                                                view! {
                                                    <a href=child.href.clone().unwrap_or_else(|| "#".to_string()) class="block px-4 py-3 text-sm text-slate-600 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800/80 hover:text-primary transition-colors border-b border-slate-100 dark:border-slate-800/50 last:border-0 uppercase font-medium">
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
            <div class="flex items-center space-x-4 md:space-x-6 z-50">
                <button
                    on:click=move |_| set_mobile_menu_open.update(|o| *o = !*o)
                    class="md:hidden text-primary focus:outline-none flex items-center justify-center p-2 rounded-md hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors"
                >
                    <span class="material-symbols-outlined text-3xl">
                        {move || if mobile_menu_open.get() { "close" } else { "menu" }}
                    </span>
                </button>
                <a href="/admin" class="material-symbols-outlined text-primary cursor-pointer hover:opacity-80 transition-opacity hidden sm:block">"terminal"</a>
                <Suspense fallback=move || view! {
                    <a href="#" class="bg-surface border border-outline-variant/30 px-6 py-2 jetbrains text-[0.65rem] font-bold tracking-wider opacity-50 block whitespace-nowrap">
                        <div class="flex flex-col items-center leading-none justify-center">
                            <span class="text-[0.55rem] text-on-surface-variant uppercase font-medium">"CURRENT BLOCK"</span>
                            <div class="mt-1 flex items-center text-on-surface">
                                <span class="material-symbols-outlined text-[0.8rem] inline mr-1 text-[#f7931a] align-text-bottom">"currency_bitcoin"</span>
                                <span>"..."</span>
                            </div>
                        </div>
                    </a>
                }>
                    {move || {
                        let h = height_resource.get().unwrap_or(Ok(0)).unwrap_or(0);
                        view! {
                            <a href=format!("https://mempool.space/block/{}", h) target="_blank" rel="noopener noreferrer" class="bg-surface border border-outline-variant/50 hover:border-[#f7931a]/50 shadow-sm px-6 py-2 jetbrains text-[0.65rem] font-bold tracking-wider hover:bg-surface-container-low transition-all block whitespace-nowrap">
                                <div class="flex flex-col items-center leading-none justify-center">
                                    <span class="text-[0.55rem] text-on-surface-variant uppercase font-medium tracking-[0.1em]">"CURRENT BLOCK"</span>
                                    <div class="mt-1 flex items-center text-on-surface">
                                        <span class="material-symbols-outlined text-[0.8rem] inline mr-1 text-[#f7931a] align-text-bottom">"currency_bitcoin"</span>
                                        <span>"#" {h}</span>
                                    </div>
                                </div>
                            </a>
                        }
                    }}
                </Suspense>
            </div>

        </nav>

        // Mobile Menu Overlay
        <div
            class="fixed inset-0 bg-surface dark:bg-slate-900 z-50 flex flex-col pt-32 px-6 transition-all duration-300 ease-in-out md:hidden"
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
