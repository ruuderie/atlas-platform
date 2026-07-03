use leptos::prelude::*;
use crate::api::products::get_products;

#[component]
pub fn PlatformProducts() -> impl IntoView {

    // Load real platform products from database
    let products_res = LocalResource::new(move || async move {
        get_products().await.unwrap_or_default()
    });

    view! {
        <div class="main-canvas">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Landing Pages"</h1>
                    <p class="page-subtitle">"Manage product landing pages, markets, localization, and tracking."</p>
                </div>
                <div class="page-actions">
                    <a href="/products/new" class="btn btn-primary" id="btn-new-product">
                        "+ New Landing Page"
                    </a>
                </div>
            </div>

            <Suspense fallback=|| view! {
                <div style="display:flex;flex-direction:column;align-items:center;gap:12px;padding:48px;color:var(--text-muted);">
                    <span class="material-symbols-outlined" style="font-size:32px;animation:spin 1s linear infinite;opacity:0.4">"sync"</span>
                    <p style="font-size:13px">"Loading platform products…"</p>
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
                    <div style="display:flex;flex-direction:column;gap:14px;">

                        // ── Explainer banner ──
                        <div class="bg-surface-container-low border border-outline-variant/15 rounded-xl px-5 py-4 text-xs text-on-surface-variant/80 space-y-1">
                            <p class="font-semibold text-on-surface text-sm">
                                "What are Landing Pages?"
                            </p>
                            <p>
                                "Each product has a public landing page your prospects land on. "
                                "Share the link directly or attach a UTM campaign slug to track where leads come from "
                                "(e.g. a postcard, ad, or cold email)."
                            </p>
                            <p class="text-on-surface-variant/60 text-[11px]">
                                "Tip: Go to "
                                <a href="/campaigns" class="text-primary hover:underline font-semibold">"Campaigns"</a>
                                " to connect a campaign to a landing page via the utm_campaign field."
                            </p>
                        </div>

                    // ── KPI Row ──
                    <div class="kpi-row">
                        <div class="kpi-card">
                            <div class="kpi-label">"Total Products"</div>
                            <div class="kpi-value mono">{total_products}</div>
                        </div>
                        <div class="kpi-card">
                            <div class="kpi-label">"Live Pages"</div>
                            <div class="kpi-value mono" style="color:var(--green)">{live_pages}</div>
                        </div>
                        <div class="kpi-card">
                            <div class="kpi-label">"Leads (30d)"</div>
                            <div class="kpi-value mono">{total_leads}</div>
                        </div>
                    </div>                        // ── Products Grid ──
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
                                        _         => ("var(--cobalt-dim)", "var(--border-default)", "var(--cobalt)"),
                                    };

                                    let is_live = {
                                        let s = p.status.to_lowercase();
                                        s == "active" || s == "live"
                                    };
                                    let domain_text = p.apex_domain.clone()
                                        .unwrap_or_else(|| format!("{}.rentals/landing", slug));
                                    let desc_text = p.tagline.clone()
                                        .unwrap_or_else(|| format!("Enterprise-grade service engine for {}.", p.name));
                                    let href = format!("/products/{}", p.id);

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
                                                {if p.waitlist_count == 0 && p.pre_order_sold == 0 {
                                                    view! {
                                                        <span class="text-on-surface-variant/60 italic">
                                                            "No leads yet — share your page to start tracking"
                                                        </span>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <span class="text-on-surface-variant">
                                                            <strong class="text-on-surface font-bold font-mono">{p.waitlist_count}</strong>
                                                            " leads"
                                                        </span>
                                                        <span class="text-on-surface-variant">
                                                            <strong class="text-on-surface font-bold font-mono">{p.pre_order_sold}</strong>
                                                            " pre-orders"
                                                        </span>
                                                    }.into_any()
                                                }}
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
                                    <svg class="w-12 h-12 text-on-surface-variant/30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
                                        <path d="M3 7h18M3 7v13h18V7M3 7l9-4 9 4"/>
                                        <line x1="10" y1="13" x2="14" y2="13"/>
                                    </svg>
                                    <div>
                                        <p class="text-sm font-semibold text-on-surface-variant">"No landing pages yet"</p>
                                        <p class="text-xs text-on-surface-variant/60 mt-1">"Create your first landing page to start capturing leads."</p>
                                    </div>
                                    <a
                                        href="/products/new"
                                        class="inline-flex items-center gap-1.5 px-4 py-2 rounded-lg text-xs font-semibold bg-primary text-on-primary hover:opacity-90 transition-all shadow-sm"
                                    >
                                        "+ New Landing Page"
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
        </div>
    }
}
