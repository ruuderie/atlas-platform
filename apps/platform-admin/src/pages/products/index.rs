use crate::api::products::{create_product, get_products};
use crate::app::GlobalToast;
use crate::components::gtm_process_strip::{GtmProcessStrip, GtmStage};
use leptos::prelude::*;

#[component]
pub fn PlatformProducts() -> impl IntoView {
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");
    let products_version = RwSignal::new(0u32);
    let show_new_modal = RwSignal::new(false);
    let new_product_name = RwSignal::new(String::new());
    let new_product_slug = RwSignal::new(String::new());

    // Load real platform products from database
    let products_res = LocalResource::new(move || async move {
        let _ = products_version.get();
        get_products().await.unwrap_or_default()
    });

    let open_new_product = move |_| {
        new_product_name.set(String::new());
        new_product_slug.set(String::new());
        show_new_modal.set(true);
    };

    let handle_create_product = move |_| {
        let name = new_product_name.get().trim().to_string();
        let explicit_slug = new_product_slug.get().trim().to_string();
        if name.is_empty() {
            toast.show_toast("Validation", "Product name is required.", "error");
            return;
        }
        let slug = if explicit_slug.is_empty() {
            slugify(&name)
        } else {
            slugify(&explicit_slug)
        };
        if slug.is_empty() {
            toast.show_toast("Validation", "Product slug is required.", "error");
            return;
        }
        leptos::task::spawn_local(async move {
            match create_product(name, slug).await {
                Ok(_) => {
                    show_new_modal.set(false);
                    products_version.update(|n| *n += 1);
                    toast.show_toast("Created", "Product created from catalog API.", "success");
                }
                Err(e) => toast.show_toast("Create failed", &e, "error"),
            }
        });
    };

    view! {
        <div class="main-canvas">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Products"</h1>
                    <p class="page-subtitle">"Catalog, pricing, launch mode, and market SEO for each offering."</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-primary" id="btn-new-product" on:click=open_new_product>
                        "+ New Product"
                    </button>
                </div>
            </div>

            <GtmProcessStrip
                active=GtmStage::Products
                subtitle="Catalog, pricing, launch mode, and market SEO for each offering."
            />

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
                                "Products define the offer. Landing Pages acquire the audience."
                            </p>
                            <p>
                                "Products hold the durable catalog, pricing, launch mode, and market SEO for each offering. "
                                "Landing Pages are the visitor-facing acquisition surfaces you can test by channel, market, and campaign."
                            </p>
                            <p class="text-on-surface-variant/60 text-[11px]">
                                "Manage acquisition pages in "
                                <a href="/landing-pages" class="text-primary hover:underline font-semibold">"Landing Pages"</a>
                                ", then connect them to "
                                <a href="/campaigns" class="text-primary hover:underline font-semibold">"Campaigns"</a>
                                " with UTM tracking."
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
                                        <div
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
                                            <div class="flex items-center gap-2 pt-1">
                                                <a href=href class="btn btn-ghost btn-sm" style="text-decoration:none">
                                                    "Edit Product"
                                                </a>
                                                <a href="/landing-pages" class="btn btn-ghost btn-sm" style="text-decoration:none">
                                                    "Acquisition Pages →"
                                                </a>
                                            </div>
                                        </div>
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
                                        <p class="text-sm font-semibold text-on-surface-variant">"No products yet"</p>
                                        <p class="text-xs text-on-surface-variant/60 mt-1">"Create your first product to define an offering."</p>
                                    </div>
                                    <button
                                        class="btn btn-primary"
                                        on:click=open_new_product
                                    >
                                        "+ New Product"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <></> }.into_any()
                        }}

                    </div>
                }
            }}
            </Suspense>

            <Show when=move || show_new_modal.get()>
                <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                    <div class="bg-surface-container border border-outline-variant/30 rounded-2xl shadow-2xl w-full max-w-md mx-4 overflow-hidden">
                        <div class="px-6 py-4 border-b border-outline-variant/20 flex items-center justify-between">
                            <h2 class="text-sm font-bold text-on-surface">"New Product"</h2>
                            <button class="btn btn-ghost btn-icon btn-sm" on:click=move |_| show_new_modal.set(false)>
                                <svg class="w-4 h-4" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4l8 8M12 4l-8 8"/></svg>
                            </button>
                        </div>
                        <div class="px-6 py-5 space-y-4">
                            <div>
                                <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Product Name"</label>
                                <input
                                    type="text"
                                    placeholder="e.g. Folio Broker"
                                    class="w-full bg-surface-container-low border border-outline-variant/30 rounded-lg px-3 py-2 text-xs text-on-surface placeholder:text-on-surface-variant/40 focus:border-primary/60 outline-none"
                                    prop:value=move || new_product_name.get()
                                    on:input=move |ev| new_product_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div>
                                <label class="block text-[10px] font-bold uppercase tracking-wider text-on-surface-variant mb-1.5">"Slug"</label>
                                <input
                                    type="text"
                                    placeholder="Auto-generated if blank"
                                    class="w-full bg-surface-container-low border border-outline-variant/30 rounded-lg px-3 py-2 text-xs text-on-surface placeholder:text-on-surface-variant/40 focus:border-primary/60 outline-none"
                                    prop:value=move || new_product_slug.get()
                                    on:input=move |ev| new_product_slug.set(event_target_value(&ev))
                                />
                                <p class="text-[10px] text-on-surface-variant/50 mt-1">
                                    "Creates the catalog product. Add acquisition pages from Landing Pages."
                                </p>
                            </div>
                        </div>
                        <div class="px-6 py-4 border-t border-outline-variant/20 flex justify-end gap-3">
                            <button class="btn btn-ghost" on:click=move |_| show_new_modal.set(false)>"Cancel"</button>
                            <button class="btn btn-primary" on:click=handle_create_product>"+ New Product"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in input.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}
