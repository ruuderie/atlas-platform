use leptos::prelude::*;
use crate::api::products::get_products;

#[component]
pub fn PlatformProducts() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // Load real platform products from database
    let products_res = LocalResource::new(move || async move {
        get_products().await.unwrap_or_default()
    });

    let handle_import = move |_| {
        toast.show_toast("Import", "Storefront page import template initialized.", "info");
    };

    view! {
        <Suspense fallback=|| view! {
            <div class="main-canvas">
                <div class="animate-pulse flex flex-col items-center justify-center h-64">
                    <span class="material-symbols-outlined text-4xl mb-2 opacity-50">"sync"</span>
                    <p>"Loading platform products..."</p>
                </div>
            </div>
        }>
            {move || {
                let products = products_res.get().unwrap_or_default();
                let total_products = products.len();
                let live_pages = products.iter().filter(|p| p.status.to_lowercase() == "active" || p.status.to_lowercase() == "live").count();
                let total_leads: i32 = products.iter().map(|p| p.waitlist_count + p.pre_order_sold).sum();
                
                view! {
                    <div class="main-canvas">
                        // ── Page Header ──
                        <div class="page-header">
                            <div>
                                <h1 class="page-title">"Platform Products"</h1>
                                <p class="page-subtitle">"Marketing pages, pricing tiers, and storefront configuration"</p>
                            </div>
                            <div style="display:flex;gap:8px;">
                                <button class="btn btn-ghost btn-sm" id="btn-import-page" on:click=handle_import>"Import page"</button>
                                <a href="/billing/products" class="btn btn-primary" id="btn-new-product">"+ New product"</a>
                            </div>
                        </div>

                        // ── KPIs ──
                        <div class="kpi-row">
                            <div class="kpi-card">
                                <span class="kpi-label">"Total products"</span>
                                <span class="kpi-value">{total_products}</span>
                            </div>
                            <div class="kpi-card">
                                <span class="kpi-label">"Live pages"</span>
                                <span class="kpi-value" style="color:var(--green);">{live_pages}</span>
                            </div>
                            <div class="kpi-card">
                                <span class="kpi-label">"Total leads (30d)"</span>
                                <span class="kpi-value">{total_leads}</span>
                                <span class="kpi-delta up">"↑ 34%"</span>
                            </div>
                            <div class="kpi-card">
                                <span class="kpi-label">"Conversion rate"</span>
                                <span class="kpi-value">"4.1%"</span>
                                <span class="kpi-delta up">"↑ 0.7pp"</span>
                            </div>
                        </div>

                        // ── Products Grid ──
                        <div class="products-grid">
                            <For
                                each=move || products_res.get().unwrap_or_default()
                                key=|p| p.id
                                children=move |p| {
                                    let slug = p.slug.clone();
                                    let pc_class = match slug.as_str() {
                                        "folio" => "product-card pc-folio",
                                        "network" => "product-card pc-network",
                                        "meridian" => "product-card pc-meridian",
                                        _ => "product-card pc-enterprise",
                                    };
                                    let is_live = p.status.to_lowercase() == "active" || p.status.to_lowercase() == "live";
                                    let badge_class = if is_live { "product-mode-badge live" } else { "product-mode-badge draft" };
                                    let badge_text = if is_live { "Live" } else { "Draft" };
                                    let domain_text = p.apex_domain.clone().unwrap_or_else(|| format!("{}.rentals/landing", slug));
                                    let desc_text = p.tagline.clone().unwrap_or_else(|| format!("Enterprise-grade service engine for {}.", p.name));
                                    
                                    view! {
                                        <a href=format!("/products/{}", slug) class=pc_class>
                                            <span class=badge_class>{badge_text}</span>
                                            <p class="product-name">{p.name.clone()}</p>
                                            <p class="product-domain">{domain_text}</p>
                                            <p class="product-desc">{desc_text}</p>
                                            <div class="product-stats">
                                                <span class="product-stat"><strong>{p.waitlist_count}</strong>" leads / 30d"</span>
                                                <span class="product-stat"><strong>{p.pre_order_sold}</strong>" pre-orders"</span>
                                            </div>
                                        </a>
                                    }
                                }
                            />
                        </div>
                    </div>
                }
            }}
        </Suspense>
    }
}
