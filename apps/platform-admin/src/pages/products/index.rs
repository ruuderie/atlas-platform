use leptos::prelude::*;
use crate::api::products::get_products;

#[component]
pub fn PlatformProducts() -> impl IntoView {

    // Load real platform products from database
    let products_res = LocalResource::new(move || async move {
        get_products().await.unwrap_or_default()
    });

    view! {
        <Suspense fallback=|| view! {
            <div class="p-12 flex flex-col items-center justify-center gap-3 text-on-surface-variant/60">
                <span class="material-symbols-outlined text-4xl animate-spin opacity-40">"sync"</span>
                <p class="text-sm">"Loading platform products…"</p>
            </div>
        }>
            {move || {
                let products = products_res.get().unwrap_or_default();
                let total_products = products.len();
                let live_pages = products.iter().filter(|p| {
                    let s = p.status.to_lowercase();
                    s == "active" || s == "live"
                }).count();
                let total_leads: i32 = products.iter().map(|p| p.waitlist_count + p.pre_order_sold).sum();

                view! {
                    <div class="w-full space-y-6">

                        // ── Page Header ──
                        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                            <div>
                                <h1 class="text-2xl font-extrabold text-on-surface tracking-tight">"Landing Pages"</h1>
                                <p class="text-sm text-on-surface-variant mt-1">"Manage product landing pages, markets, localization, and tracking."</p>
                            </div>
                            <div class="flex items-center gap-2 shrink-0">
                                <a
                                    href="/products/new"
                                    class="inline-flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-semibold bg-primary text-on-primary hover:opacity-90 transition-all shadow-sm"
                                    id="btn-new-product"
                                >
                                    "+ New Product"
                                </a>
                            </div>
                        </div>

                        // ── KPIs ──
                        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4">
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col gap-1">
                                <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Total Products"</span>
                                <span class="text-3xl font-black text-on-surface font-mono">{total_products}</span>
                            </div>
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col gap-1">
                                <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Live Pages"</span>
                                <span class="text-3xl font-black text-emerald-400 font-mono">{live_pages}</span>
                            </div>
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm flex flex-col gap-1">
                                <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Leads (30 d)"</span>
                                <span class="text-3xl font-black text-on-surface font-mono">{total_leads}</span>
                            </div>
                        </div>

                        // ── Products Grid ──
                        <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-5">
                            <For
                                each=move || products_res.get().unwrap_or_default()
                                key=|p| p.id
                                children=move |p| {
                                    let slug = p.slug.clone();

                                    // Card accent color per product
                                    let (accent_bg, accent_border, accent_dot) = match slug.as_str() {
                                        "folio"   => ("bg-violet-500/10", "border-violet-500/30", "bg-violet-400"),
                                        "network" => ("bg-sky-500/10",    "border-sky-500/30",    "bg-sky-400"),
                                        "meridian"=> ("bg-amber-500/10",  "border-amber-500/30",  "bg-amber-400"),
                                        _         => ("bg-indigo-500/10", "border-indigo-500/30", "bg-indigo-400"),
                                    };

                                    let is_live = {
                                        let s = p.status.to_lowercase();
                                        s == "active" || s == "live"
                                    };
                                    let domain_text = p.apex_domain.clone()
                                        .unwrap_or_else(|| format!("{}.rentals/landing", slug));
                                    let desc_text = p.tagline.clone()
                                        .unwrap_or_else(|| format!("Enterprise-grade service engine for {}.", p.name));
                                    let href = format!("/products/{}", slug);

                                    view! {
                                        <a
                                            href=href
                                            class=format!(
                                                "group relative flex flex-col gap-3 p-5 rounded-2xl border {} {} shadow-sm hover:shadow-md transition-all bg-surface-container-low",
                                                accent_bg, accent_border
                                            )
                                        >
                                            // ── Status badge — absolutely positioned top-right ──
                                            <div class="flex items-start justify-between gap-2">
                                                // Product icon dot
                                                <div class=format!("w-9 h-9 rounded-xl {} flex items-center justify-center shrink-0", accent_bg)>
                                                    <span class=format!("w-3 h-3 rounded-full {}", accent_dot)></span>
                                                </div>
                                                // Live/Draft badge
                                                <span class=if is_live {
                                                    "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 shrink-0"
                                                } else {
                                                    "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider bg-outline-variant/20 text-on-surface-variant border border-outline-variant/30 shrink-0"
                                                }>
                                                    {if is_live { "● Live" } else { "○ Draft" }}
                                                </span>
                                            </div>

                                            // ── Product name ──
                                            <div class="space-y-0.5">
                                                <p class="text-base font-extrabold text-on-surface leading-tight group-hover:text-primary transition-colors">
                                                    {p.name.clone()}
                                                </p>
                                                <p class="text-[10px] font-mono text-on-surface-variant/60 truncate">
                                                    {domain_text}
                                                </p>
                                            </div>

                                            // ── Description ──
                                            <p class="text-xs text-on-surface-variant/80 leading-relaxed line-clamp-2 flex-1">
                                                {desc_text}
                                            </p>

                                            // ── Stats row ──
                                            <div class="flex items-center gap-4 pt-2 border-t border-outline-variant/10 text-xs">
                                                <span class="text-on-surface-variant">
                                                    <strong class="text-on-surface font-bold font-mono">{p.waitlist_count}</strong>
                                                    " leads"
                                                </span>
                                                <span class="text-on-surface-variant">
                                                    <strong class="text-on-surface font-bold font-mono">{p.pre_order_sold}</strong>
                                                    " pre-orders"
                                                </span>
                                            </div>
                                        </a>
                                    }
                                }
                            />
                        </div>

                        // ── Empty state ──
                        {move || if products_res.get().unwrap_or_default().is_empty() {
                            view! {
                                <div class="flex flex-col items-center justify-center gap-4 py-20 text-center">
                                    <span class="material-symbols-outlined text-[48px] text-on-surface-variant/30">"inventory_2"</span>
                                    <div>
                                        <p class="text-sm font-semibold text-on-surface-variant">"No products yet"</p>
                                        <p class="text-xs text-on-surface-variant/60 mt-1">"Create your first product to start building marketing pages."</p>
                                    </div>
                                    <a
                                        href="/products/new"
                                        class="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-semibold bg-primary text-on-primary hover:opacity-90 transition-all shadow-sm"
                                    >
                                        "+ Create Product"
                                    </a>
                                </div>
                            }.into_any()
                        } else {
                            view! { <></> }.into_any()
                        }}

                    </div>
                }
            }}
        </Suspense>
    }
}
