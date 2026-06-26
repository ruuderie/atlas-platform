use leptos::prelude::*;
use uuid::Uuid;
use crate::api::models::{
    CopyStrategy, LaunchMode, LocalizationStatus,
    PlatformProductModel, ProductVariantModel, UpdateProductBody,
    BulkGenerateBody, MarketSpec,
};

#[component]
pub fn BillingProducts() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    let trigger_fetch = RwSignal::new(0);
    
    // Resource for database platform products
    let db_products = LocalResource::new(move || {
        trigger_fetch.get();
        async move {
            crate::api::products::get_products().await.unwrap_or_default()
        }
    });

    let products_list = Signal::derive(move || {
        db_products.get().unwrap_or_default()
    });

    // Reactive states
    let search_query = RwSignal::new(String::new());
    let filter_status = RwSignal::new("all".to_string());
    let selected_id = RwSignal::new(None::<Uuid>);
    let active_tab = RwSignal::new("overview".to_string());

    // Modal signal states
    let show_create_modal = RwSignal::new(false);
    let new_prod_name = RwSignal::new(String::new());
    let new_prod_slug = RwSignal::new(String::new());

    let show_publish_modal = RwSignal::new(false);
    let show_variant_gen_modal = RwSignal::new(false);
    let show_add_alias_modal = RwSignal::new(false);
    let show_add_locale_modal = RwSignal::new(false);

    // Filtered list of products
    let filtered_products = Signal::derive(move || {
        let query = search_query.get().to_lowercase();
        let status_filter = filter_status.get();
        products_list.get().into_iter().filter(move |p| {
            let matches_search = p.name.to_lowercase().contains(&query) || p.slug.contains(&query);
            let matches_status = match status_filter.as_str() {
                "all" => true,
                "live" => p.status.to_lowercase() == "active" || p.status.to_lowercase() == "live",
                "beta" => p.status.to_lowercase() == "beta",
                "pre-launch" => p.status.to_lowercase() == "pre_launch" || p.status.to_lowercase() == "pre-launch" || p.status.to_lowercase() == "waitlist",
                "ai" => p.status.to_lowercase() == "ai" || p.slug.contains("brazil"),
                _ => true,
            };
            matches_search && matches_status
        }).collect::<Vec<PlatformProductModel>>()
    });

    // Set first product as default selection on mount/fetch
    Effect::new(move |_| {
        let list = filtered_products.get();
        if !list.is_empty() && selected_id.get().is_none() {
            selected_id.set(Some(list[0].id));
        }
    });

    let selected_product = Signal::derive(move || {
        let sid = selected_id.get();
        products_list.get().into_iter().find(|p| Some(p.id) == sid)
    });

    // Selected Product Resource dependencies
    let product_variants = LocalResource::new(move || {
        let sid = selected_id.get();
        async move {
            if let Some(id) = sid {
                crate::api::products::get_variants(id).await.ok()
            } else {
                None
            }
        }
    });

    let product_waitlist = LocalResource::new(move || {
        let sid = selected_id.get();
        async move {
            if let Some(id) = sid {
                crate::api::products::get_waitlist(id).await.ok()
            } else {
                None
            }
        }
    });

    let product_deploy_status = LocalResource::new(move || {
        let sid = selected_id.get();
        async move {
            if let Some(id) = sid {
                crate::api::products::get_deploy_status(id).await.ok()
            } else {
                None
            }
        }
    });

    let product_template = LocalResource::new(move || {
        let sid = selected_id.get();
        async move {
            if let Some(id) = sid {
                crate::api::products::get_template(id).await.ok()
            } else {
                None
            }
        }
    });

    let variants_list = Signal::derive(move || {
        if let Some(Some(list)) = product_variants.get() {
            if !list.is_empty() {
                return list;
            }
        }
        // Fallback to mock variants
        vec![
            ProductVariantModel {
                id: Uuid::parse_str("99999999-9999-9999-9999-999999999999").unwrap(),
                product_id: Uuid::nil(),
                template_id: Uuid::nil(),
                variant_slug: "en-US".to_string(),
                locale: "en-US".to_string(),
                country_code: Some("US".to_string()),
                region: None,
                city: None,
                geo_lat: None,
                geo_lng: None,
                hero_overrides: serde_json::Value::Null,
                block_overrides: serde_json::Value::Null,
                meta_title: Some("English (US)".to_string()),
                meta_description: None,
                og_image_url: None,
                canonical_url: None,
                structured_data: None,
                launch_mode: LaunchMode::Active,
                is_published: true,
                cta_label: None,
                cta_action: None,
                pre_order_cap: None,
                pre_order_sold: 0,
                lead_count: 0,
                view_count: 0,
                copy_strategy: CopyStrategy::BaseCopy,
                localization_status: LocalizationStatus::Base,
                localization_task_id: None,
                subdomain_override: None,
                created_at: "".to_string(),
                updated_at: "".to_string(),
            },
            ProductVariantModel {
                id: Uuid::parse_str("88888888-8888-8888-8888-888888888888").unwrap(),
                product_id: Uuid::nil(),
                template_id: Uuid::nil(),
                variant_slug: "pt-BR".to_string(),
                locale: "pt-BR".to_string(),
                country_code: Some("BR".to_string()),
                region: None,
                city: None,
                geo_lat: None,
                geo_lng: None,
                hero_overrides: serde_json::Value::Null,
                block_overrides: serde_json::Value::Null,
                meta_title: Some("Portuguese (Brazil)".to_string()),
                meta_description: None,
                og_image_url: None,
                canonical_url: None,
                structured_data: None,
                launch_mode: LaunchMode::Active,
                is_published: true,
                cta_label: None,
                cta_action: None,
                pre_order_cap: None,
                pre_order_sold: 0,
                lead_count: 0,
                view_count: 0,
                copy_strategy: CopyStrategy::AiGenerated,
                localization_status: LocalizationStatus::AiLocalized,
                localization_task_id: None,
                subdomain_override: None,
                created_at: "Jun 08".to_string(),
                updated_at: "".to_string(),
            },
            ProductVariantModel {
                id: Uuid::parse_str("77777777-7777-7777-7777-777777777777").unwrap(),
                product_id: Uuid::nil(),
                template_id: Uuid::nil(),
                variant_slug: "fr-FR".to_string(),
                locale: "fr-FR".to_string(),
                country_code: Some("FR".to_string()),
                region: None,
                city: None,
                geo_lat: None,
                geo_lng: None,
                hero_overrides: serde_json::Value::Null,
                block_overrides: serde_json::Value::Null,
                meta_title: Some("French".to_string()),
                meta_description: None,
                og_image_url: None,
                canonical_url: None,
                structured_data: None,
                launch_mode: LaunchMode::PreLaunch,
                is_published: false,
                cta_label: None,
                cta_action: None,
                pre_order_cap: None,
                pre_order_sold: 0,
                lead_count: 0,
                view_count: 0,
                copy_strategy: CopyStrategy::AiGenerated,
                localization_status: LocalizationStatus::Pending,
                localization_task_id: None,
                subdomain_override: None,
                created_at: "Jun 09".to_string(),
                updated_at: "".to_string(),
            }
        ]
    });

    let waitlist_summary = Signal::derive(move || {
        if let Some(Some(w)) = product_waitlist.get() {
            format!("Total: {}, Markets: {}", w.total_leads, w.variant_count)
        } else {
            "No active waitlist".to_string()
        }
    });

    let template_summary = Signal::derive(move || {
        if let Some(Some(t)) = product_template.get() {
            format!("Template ID: {}", t.id)
        } else {
            "Default Layout".to_string()
        }
    });

    // Derived style closures
    let pill_class = move |status: &'static str| {
        let active = filter_status.get() == status;
        if active {
            "px-3 py-1.5 rounded-md bg-surface-container-high text-primary text-xs font-bold transition-all border border-outline-variant/50"
        } else {
            "px-3 py-1.5 rounded-md text-on-surface-variant text-xs font-bold hover:text-on-surface transition-all border border-transparent"
        }
    };

    let tab_class = move |tab: &'static str| {
        let active = active_tab.get() == tab;
        if active {
            "px-4 py-3 text-xs font-bold text-primary border-b-2 border-primary outline-none transition-all"
        } else {
            "px-4 py-3 text-xs font-semibold text-on-surface-variant hover:text-on-surface outline-none transition-all"
        }
    };

    let card_class = move |pid: Uuid| {
        let active = selected_id.get() == Some(pid);
        if active {
            "flex items-center gap-3.5 p-4 bg-[var(--bg-surface)] border border-primary/50 shadow-md rounded-xl cursor-pointer"
        } else {
            "flex items-center gap-3.5 p-4 bg-[var(--bg-surface)]/30 hover:bg-[var(--bg-surface)]/60 border border-outline-variant/10 rounded-xl cursor-pointer transition-all duration-150"
        }
    };

    let handle_publish = move |_| {
        show_publish_modal.set(false);
        if let Some(p) = selected_product.get() {
            let pid = p.id;
            leptos::task::spawn_local(async move {
                match crate::api::products::publish_marketing(pid).await {
                    Ok(_) => toast.show_toast("Success", "Marketing page successfully deployed.", "success"),
                    Err(e) => toast.show_toast("Error", &e, "error"),
                }
            });
        }
    };

    let handle_bulk_generate = move |_| {
        show_variant_gen_modal.set(false);
        if let Some(p) = selected_product.get() {
            let pid = p.id;
            leptos::task::spawn_local(async move {
                let spec = BulkGenerateBody {
                    markets: vec![
                        MarketSpec { slug: "nyc-str".into(), locale: "en-US".into(), city: Some("New York".into()), region: Some("NY".into()), country_code: Some("US".into()), geo_lat: Some(40.7128), geo_lng: Some(-74.0060), subdomain_override: None, pre_order_cap: None },
                        MarketSpec { slug: "rio-str".into(), locale: "pt-BR".into(), city: Some("Rio de Janeiro".into()), region: Some("RJ".into()), country_code: Some("BR".into()), geo_lat: Some(-22.9068), geo_lng: Some(-43.1729), subdomain_override: None, pre_order_cap: None },
                    ],
                    launch_mode: Some("beta".into()),
                    copy_strategy: Some("ai_localize".into()),
                };
                match crate::api::products::bulk_generate_variants(pid, spec).await {
                    Ok(_) => toast.show_toast("Success", "Pricing variant bulk generation triggered.", "success"),
                    Err(e) => toast.show_toast("Error", &e, "error"),
                }
            });
        }
    };

    let handle_create_product = move |_| {
        show_create_modal.set(false);
        let name = new_prod_name.get();
        let slug = new_prod_slug.get();
        if name.is_empty() || slug.is_empty() {
            return;
        }
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match crate::api::products::create_product(name.clone(), slug).await {
                Ok(_) => {
                    t_toast.show_toast("Product Created", &format!("Product '{}' created successfully.", name), "success");
                    trigger_fetch.update(|v| *v += 1);
                }
                Err(e) => t_toast.show_toast("Error", &format!("Failed to create product: {}", e), "error"),
            }
        });
        new_prod_name.set(String::new());
        new_prod_slug.set(String::new());
    };

    let handle_save_settings = move |_| {
        if let Some(p) = selected_product.get() {
            let pid = p.id;
            let body = UpdateProductBody {
                name: Some(p.name.clone()),
                tagline: p.tagline.clone(),
                status: Some(p.status.clone()),
                deploy_hook_url: p.deploy_hook_url.clone(),
                marketing_page_cms_id: p.marketing_page_cms_id,
            };
            leptos::task::spawn_local(async move {
                match crate::api::products::update_product_detail(pid, body).await {
                    Ok(_) => {
                        toast.show_toast("Success", "Settings saved successfully.", "success");
                        trigger_fetch.set(trigger_fetch.get() + 1);
                    }
                    Err(e) => toast.show_toast("Error", &e, "error"),
                }
            });
        }
    };

    view! {
        <div class="space-y-6">
            // ── Header Section ──
            <div class="flex flex-col md:flex-row md:items-end justify-between gap-4">
                <div>
                    <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                        <span>"Financials"</span>
                        <span class="material-symbols-outlined text-xs">"chevron_right"</span>
                        <span class="text-primary/70">"Products & Plans"</span>
                    </nav>
                    <h1 class="text-4xl font-extrabold tracking-tight text-on-surface mb-2">"Products & Plans"</h1>
                    <p class="text-on-surface-variant text-sm max-w-2xl">"Manage regional product lines, waitlist pages, domain aliases, and auto-generated localization assets."</p>
                </div>
                <div class="flex items-center gap-3">
                    <button class="px-4 py-2 border border-outline-variant/30 text-on-surface bg-surface-container-high hover:bg-surface-bright/20 rounded-lg text-xs font-semibold uppercase tracking-wider" on:click=move |_| show_variant_gen_modal.set(true)>"Bulk Generate Variants"</button>
                    <button class="btn-primary-gradient px-4 py-2 text-on-primary-container rounded-lg text-xs font-bold uppercase tracking-wider shadow-lg shadow-primary/10" on:click=move |_| show_create_modal.set(true)>"+ New Product"</button>
                </div>
            </div>

            // ── Workspace: Left (List) & Right (Details) ──
            <div class="grid grid-cols-1 lg:grid-cols-[360px_1fr] h-[720px] border border-outline-variant/20 rounded-2xl overflow-hidden bg-[var(--bg-base)]">
                
                // Left Panel: Product Registry
                <div class="flex flex-col border-r border-outline-variant/10 bg-[var(--bg-surface)]/10 overflow-hidden">
                    <div class="p-4 border-b border-outline-variant/10 flex-shrink-0">
                        <div class="relative items-center">
                            <span class="material-symbols-outlined absolute left-3 top-2.5 text-on-surface-variant/70 text-sm">"search"</span>
                            <input
                                type="text"
                                placeholder="Search products..."
                                class="w-full bg-[var(--bg-elevated)] border border-outline-variant/30 text-on-surface text-xs rounded-lg pl-9 pr-3 py-2.5 outline-none focus:ring-1 focus:ring-primary focus:border-primary placeholder:text-[#91aaeb]/60"
                                on:input=move |ev| search_query.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                    
                    <div class="flex gap-1.5 p-3 border-b border-outline-variant/5 overflow-x-auto scrollbar-none flex-shrink-0">
                        <button class=move || pill_class("all") on:click=move |_| filter_status.set("all".into())>"All"</button>
                        <button class=move || pill_class("live") on:click=move |_| filter_status.set("live".into())>"Live"</button>
                        <button class=move || pill_class("pre-launch") on:click=move |_| filter_status.set("pre-launch".into())>"Pre-Launch"</button>
                        <button class=move || pill_class("beta") on:click=move |_| filter_status.set("beta".into())>"Beta"</button>
                        <button class=move || pill_class("ai") on:click=move |_| filter_status.set("ai".into())>"AI ✦"</button>
                    </div>

                    // Scrollable Product Cards
                    <div class="flex-1 overflow-y-auto p-3 space-y-2">
                        <For
                            each=move || filtered_products.get()
                            key=|p| p.id
                            children=move |p| {
                                let pid = p.id;
                                let status_badge_class = match p.status.to_lowercase().as_str() {
                                    "active" | "live" => "bg-[#c6fff3]/10 text-[#c6fff3] border-[#c6fff3]/20",
                                    "beta" => "bg-primary/10 text-primary border-primary/20",
                                    "pre-launch" | "waitlist" => "bg-amber-400/10 text-amber-400 border-amber-400/20",
                                    "ai" => "bg-[#7C3AED]/10 text-[#a78bfa] border-[#7C3AED]/20",
                                    _ => "bg-surface-container-high/40 text-on-surface-variant border-outline-variant/30",
                                };

                                let icon_txt = match p.name.as_str() {
                                    n if n.contains("STR") => "STR",
                                    n if n.contains("Commercial") => "COM",
                                    n if n.contains("Wholesale") => "WHL",
                                    _ => "PM",
                                };

                                let icon_badge_class = match icon_txt {
                                    "PM" => "bg-[#0a84ff]/15 text-[#7bd0ff] border border-[#0a84ff]/30",
                                    "STR" => "bg-amber-400/15 text-amber-400 border border-amber-400/30",
                                    "COM" => "bg-[#7c3aed]/15 text-[#a78bfa] border border-[#7c3aed]/30",
                                    _ => "bg-[#069669]/15 text-[#c6fff3] border border-[#069669]/30",
                                };

                                view! {
                                    <div class=move || card_class(pid) on:click=move |_| selected_id.set(Some(pid))>
                                        <div class=format!("w-9 h-9 rounded-lg flex items-center justify-center font-bold text-xs flex-shrink-0 {}", icon_badge_class)>
                                            {icon_txt}
                                        </div>
                                        <div class="flex-1 min-w-0">
                                            <div class="font-bold text-xs text-on-surface truncate">{p.name.clone()}</div>
                                            <div class="text-[10px] font-mono text-on-surface-variant/70 truncate">{p.apex_domain.clone().unwrap_or_else(|| "no apex domain".to_string())}</div>
                                        </div>
                                        <div class="flex flex-col items-end gap-1.5 flex-shrink-0">
                                            <span class=format!("px-2 py-0.5 rounded text-[8px] font-bold border uppercase tracking-wider {}", status_badge_class)>
                                                {p.status.clone()}
                                            </span>
                                            <span class="text-[9.5px] text-on-surface-variant/70 font-semibold">{p.waitlist_count} " leads"</span>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>

                // Right Panel: Product Workspace Details
                <div class="flex flex-col overflow-hidden bg-surface-container/20">
                    <Show when=move || selected_product.get().is_some() fallback=|| view! {
                        <div class="flex-1 flex items-center justify-center text-on-surface-variant text-sm">"Select a product from the registry to view details."</div>
                    }>
                        {move || {
                            let p = selected_product.get().unwrap();
                            let status_badge_class = match p.status.to_lowercase().as_str() {
                                "active" | "live" => "bg-[#c6fff3]/10 text-[#c6fff3] border-[#c6fff3]/20",
                                "beta" => "bg-primary/10 text-primary border-primary/20",
                                "pre-launch" | "waitlist" => "bg-amber-400/10 text-amber-400 border-amber-400/20",
                                "ai" => "bg-[#7C3AED]/10 text-[#a78bfa] border-[#7C3AED]/20",
                                _ => "bg-surface-container-high/40 text-on-surface-variant border-outline-variant/30",
                            };

                            view! {
                                // Detail Header
                                <div class="p-6 border-b border-outline-variant/10 flex flex-col md:flex-row md:items-start justify-between gap-4 flex-shrink-0">
                                    <div>
                                        <h2 class="text-2xl font-bold tracking-tight text-on-surface mb-1">{p.name.clone()}</h2>
                                        <p class="text-xs font-mono text-on-surface-variant/80">{p.apex_domain.clone().unwrap_or_default()} " · product_id: " {p.id.to_string()}</p>
                                        <div class="flex items-center gap-2 mt-3 flex-wrap">
                                            <span class=format!("px-2 py-0.5 rounded text-[8px] font-bold border uppercase tracking-wider {}", status_badge_class)>
                                                {p.status.clone()}
                                            </span>
                                            <span class="px-2 py-0.5 text-[9px] font-bold bg-[#0a84ff]/10 text-[#7bd0ff] border border-[#0a84ff]/20 rounded-md uppercase tracking-wider">"PM Engine"</span>
                                            <span class="text-xs text-on-surface-variant ml-2">{p.waitlist_count} " waitlists · 7 locales · 28 variants"</span>
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-2 flex-shrink-0">
                                        <button class="px-3.5 py-2 border border-outline-variant/20 text-on-surface-variant/40 rounded-lg text-xs font-semibold cursor-not-allowed" title="Export endpoint pending — not yet available" disabled>"Export Waitlist"</button>
                                        <button class="px-3.5 py-2 bg-[#0a84ff] text-white hover:opacity-90 rounded-lg text-xs font-bold uppercase tracking-wider" on:click=move |_| show_publish_modal.set(true)>"Publish Marketing →"</button>
                                    </div>
                                </div>

                                // Tabs Navigation
                                <div class="flex border-b border-outline-variant/10 px-4 bg-[var(--bg-base)]/40 flex-shrink-0">
                                    <button class=move || tab_class("overview") on:click=move |_| active_tab.set("overview".into())>"Overview"</button>
                                    <button class=move || tab_class("pages") on:click=move |_| active_tab.set("pages".into())>"Templates & Variants"</button>
                                    <button class=move || tab_class("localization") on:click=move |_| active_tab.set("localization".into())>"AI Localization"</button>
                                    <button class=move || tab_class("settings") on:click=move |_| active_tab.set("settings".into())>"Settings"</button>
                                    <button class=move || tab_class("deploy") on:click=move |_| active_tab.set("deploy".into())>"Deploy Status"</button>
                                </div>

                                // Detail Pane Content
                                <div class="flex-1 overflow-y-auto p-6">
                                    
                                    // 1. Overview Tab
                                    <div class=move || format!("space-y-6 {}", if active_tab.get() == "overview" { "block" } else { "hidden" })>
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                            // Product Details Card
                                            <div class="bg-surface-container border border-outline-variant/20 rounded-xl overflow-hidden">
                                                <div class="px-4 py-3 border-b border-outline-variant/10 bg-[var(--bg-base)]/30 font-bold text-xs">"Product Catalog Info"</div>
                                                <div class="divide-y divide-outline-variant/10">
                                                    <div class="flex justify-between items-center px-4 py-3 text-xs">
                                                        <span class="text-on-surface-variant">"Launch Mode"</span>
                                                        <span class="font-semibold text-primary">{p.launch_mode.clone()}</span>
                                                    </div>
                                                    <div class="flex justify-between items-center px-4 py-3 text-xs">
                                                        <span class="text-on-surface-variant">"Pre-Order Pricing"</span>
                                                        <span class="font-semibold">{format!("{} {}", p.pre_order_price_cents.map(|c| format!("${:.2}", c as f64 / 100.0)).unwrap_or_else(|| "$0".to_string()), p.pre_order_currency)}</span>
                                                    </div>
                                                    <div class="flex justify-between items-center px-4 py-3 text-xs">
                                                        <span class="text-on-surface-variant">"Apex Domain"</span>
                                                        <span class="font-semibold font-mono text-[11px]">{p.apex_domain.clone().unwrap_or_else(|| "none".into())}</span>
                                                    </div>
                                                    <div class="flex justify-between items-center px-4 py-3 text-xs">
                                                        <span class="text-on-surface-variant">"Product Slug"</span>
                                                        <span class="font-semibold font-mono text-[11px]">{p.slug.clone()}</span>
                                                    </div>
                                                    <div class="flex justify-between items-center px-4 py-3 text-xs">
                                                        <span class="text-on-surface-variant">"Active Waitlist Info"</span>
                                                        <span class="font-semibold text-primary">{waitlist_summary.get()}</span>
                                                    </div>
                                                    <div class="flex justify-between items-center px-4 py-3 text-xs">
                                                        <span class="text-on-surface-variant">"Page Template ID"</span>
                                                        <span class="font-semibold font-mono text-[10px]">{template_summary.get()}</span>
                                                    </div>
                                                </div>
                                            </div>

                                            // Performance Card
                                            <div class="bg-surface-container border border-outline-variant/20 rounded-xl overflow-hidden">
                                                <div class="px-4 py-3 border-b border-outline-variant/10 bg-[var(--bg-base)]/30 font-bold text-xs">"Performance & Leads"</div>
                                                <div class="divide-y divide-outline-variant/10">
                                                    <div class="flex justify-between items-center px-4 py-3 text-xs">
                                                        <span class="text-on-surface-variant">"Waitlist Leads"</span>
                                                        <span class="font-semibold font-mono">{p.waitlist_count}</span>
                                                    </div>
                                                    <div class="flex justify-between items-center px-4 py-3 text-xs">
                                                        <span class="text-on-surface-variant">"Conversion MTD"</span>
                                                        <span class="font-semibold font-mono">"12.4%"</span>
                                                    </div>
                                                    <div class="flex justify-between items-center px-4 py-3 text-xs">
                                                        <span class="text-on-surface-variant">"Pre-Order Cap"</span>
                                                        <span class="font-semibold font-mono">{p.pre_order_cap.map(|c| c.to_string()).unwrap_or_else(|| "unlimited".into())}</span>
                                                    </div>
                                                    <div class="flex justify-between items-center px-4 py-3 text-xs">
                                                        <span class="text-on-surface-variant">"Pre-Orders Sold"</span>
                                                        <span class="font-semibold text-emerald-400 font-mono">{p.pre_order_sold}</span>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // Domain Aliases Table
                                        <div class="bg-surface-container border border-outline-variant/20 rounded-xl overflow-hidden">
                                            <div class="px-4 py-3 border-b border-outline-variant/10 bg-[var(--bg-base)]/30 flex justify-between items-center">
                                                <span class="font-bold text-xs">"Domain Aliases"</span>
                                                <button class="px-2.5 py-1 border border-outline-variant/30 text-on-surface hover:bg-surface-bright/20 rounded text-[11px] font-semibold" on:click=move |_| show_add_alias_modal.set(true)>"+ Add Alias"</button>
                                            </div>
                                            <div class="overflow-x-auto">
                                                <table class="w-full border-collapse text-left">
                                                    <thead>
                                                        <tr class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider border-b border-outline-variant/10 bg-[var(--bg-base)]/10">
                                                            <th class="py-2.5 px-4 font-semibold">"Domain Alias"</th>
                                                            <th class="py-2.5 px-4 font-semibold">"Alias Type"</th>
                                                            <th class="py-2.5 px-4 font-semibold">"DNS Routing"</th>
                                                            <th class="py-2.5 px-4 font-semibold">"SSL Cert"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody class="divide-y divide-outline-variant/5 text-xs">
                                                        <tr class="hover:bg-surface-container-high/40">
                                                            <td class="py-3 px-4 font-mono text-[11px]">{format!("pm.{}", p.apex_domain.clone().unwrap_or_default())}</td>
                                                            <td class="py-3 px-4 text-on-surface-variant">"Platform Subdomain"</td>
                                                            <td class="py-3 px-4 text-[#c6fff3] font-semibold">"● Active"</td>
                                                            <td class="py-3 px-4 text-[#c6fff3]">"✓ Verified"</td>
                                                        </tr>
                                                        <tr class="hover:bg-surface-container-high/40">
                                                            <td class="py-3 px-4 font-mono text-[11px]">{format!("{}/pm", p.apex_domain.clone().unwrap_or_default())}</td>
                                                            <td class="py-3 px-4 text-on-surface-variant">"Path Alias"</td>
                                                            <td class="py-3 px-4 text-[#c6fff3] font-semibold">"● Active"</td>
                                                            <td class="py-3 px-4 text-[#c6fff3]">"✓ Verified"</td>
                                                        </tr>
                                                        <tr class="hover:bg-surface-container-high/40">
                                                            <td class="py-3 px-4 font-mono text-[11px]">"propertymanagement.atlas.app"</td>
                                                            <td class="py-3 px-4 text-on-surface-variant">"Vanity Alias"</td>
                                                            <td class="py-3 px-4 text-amber-400 font-semibold">"⚠ Pending CNAME"</td>
                                                            <td class="py-3 px-4 text-on-surface-variant">"— Waiting"</td>
                                                        </tr>
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                    </div>

                                    // 2. Templates & Variants Tab
                                    <div class=move || format!("space-y-6 {}", if active_tab.get() == "pages" { "block" } else { "hidden" })>
                                        // Templates Table
                                        <div class="bg-surface-container border border-outline-variant/20 rounded-xl overflow-hidden">
                                            <div class="px-4 py-3 border-b border-outline-variant/10 bg-[var(--bg-base)]/30 flex justify-between items-center">
                                                <span class="font-bold text-xs">"Page Templates"</span>
                                                <button class="px-2.5 py-1 border border-outline-variant/20 text-on-surface-variant/40 rounded text-[11px] font-semibold cursor-not-allowed" title="Template creation pending CMS editor integration" disabled>"+ New Template"</button>
                                            </div>
                                            <div class="overflow-x-auto">
                                                <table class="w-full border-collapse text-left">
                                                    <thead>
                                                        <tr class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider border-b border-outline-variant/10 bg-[var(--bg-base)]/10">
                                                            <th class="py-2.5 px-4 font-semibold">"Template Name"</th>
                                                            <th class="py-2.5 px-4 font-semibold">"Format Type"</th>
                                                            <th class="py-2.5 px-4 font-semibold">"Locales Generated"</th>
                                                            <th class="py-2.5 px-4 font-semibold">"Launch Status"</th>
                                                            <th class="py-2.5 px-4 font-semibold text-right">"Action"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody class="divide-y divide-outline-variant/5 text-xs">
                                                        <tr class="hover:bg-surface-container-high/40">
                                                            <td class="py-3 px-4 font-bold">"Homepage"</td>
                                                            <td class="py-3 px-4 text-on-surface-variant">"hero_full"</td>
                                                            <td class="py-3 px-4 font-mono">"7"</td>
                                                            <td class="py-3 px-4"><span class="px-2 py-0.5 rounded text-[9px] font-bold bg-[#c6fff3]/10 text-[#c6fff3] border border-[#c6fff3]/20 uppercase">"Published"</span></td>
                                                            <td class="py-3 px-4 text-right">
                                                                <button class="px-2 py-1 bg-surface-container-high border border-outline-variant/20 text-on-surface-variant/40 rounded text-[10px] font-bold uppercase cursor-not-allowed" title="CMS editor integration pending" disabled>"Edit"</button>
                                                            </td>
                                                        </tr>
                                                        <tr class="hover:bg-surface-container-high/40">
                                                            <td class="py-3 px-4 font-bold">"Property Listings"</td>
                                                            <td class="py-3 px-4 text-on-surface-variant">"grid_filter"</td>
                                                            <td class="py-3 px-4 font-mono">"7"</td>
                                                            <td class="py-3 px-4"><span class="px-2 py-0.5 rounded text-[9px] font-bold bg-[#c6fff3]/10 text-[#c6fff3] border border-[#c6fff3]/20 uppercase">"Published"</span></td>
                                                            <td class="py-3 px-4 text-right">
                                                                <button class="px-2 py-1 bg-surface-container-high border border-outline-variant/20 text-on-surface-variant/40 rounded text-[10px] font-bold uppercase cursor-not-allowed" title="CMS editor integration pending" disabled>"Edit"</button>
                                                            </td>
                                                        </tr>
                                                        <tr class="hover:bg-surface-container-high/40">
                                                            <td class="py-3 px-4 font-bold">"Lead Capture"</td>
                                                            <td class="py-3 px-4 text-on-surface-variant">"form_hero"</td>
                                                            <td class="py-3 px-4 font-mono">"7"</td>
                                                            <td class="py-3 px-4"><span class="px-2 py-0.5 rounded text-[9px] font-bold bg-[#c6fff3]/10 text-[#c6fff3] border border-[#c6fff3]/20 uppercase">"Published"</span></td>
                                                            <td class="py-3 px-4 text-right">
                                                                <button class="px-2 py-1 bg-surface-container-high border border-outline-variant/20 text-on-surface-variant/40 rounded text-[10px] font-bold uppercase cursor-not-allowed" title="CMS editor integration pending" disabled>"Edit"</button>
                                                            </td>
                                                        </tr>
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>

                                        // Variants Table
                                        <div class="bg-surface-container border border-outline-variant/20 rounded-xl overflow-hidden">
                                            <div class="px-4 py-3 border-b border-outline-variant/10 bg-[var(--bg-base)]/30 flex justify-between items-center">
                                                <span class="font-bold text-xs">"Locale Variants — Homepage"</span>
                                                <button class="px-2.5 py-1 border border-outline-variant/30 text-on-surface hover:bg-surface-bright/20 rounded text-[11px] font-semibold" on:click=move |_| show_add_locale_modal.set(true)>"+ Add Locale"</button>
                                            </div>
                                            <div class="overflow-x-auto">
                                                <table class="w-full border-collapse text-left">
                                                    <thead>
                                                        <tr class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider border-b border-outline-variant/10 bg-[var(--bg-base)]/10">
                                                            <th class="py-2.5 px-4 font-semibold">"Locale"</th>
                                                            <th class="py-2.5 px-4 font-semibold">"Language/Market"</th>
                                                            <th class="py-2.5 px-4 font-semibold">"Status"</th>
                                                            <th class="py-2.5 px-4 font-semibold">"Last AI Run"</th>
                                                            <th class="py-2.5 px-4 font-semibold">"Translation Source"</th>
                                                            <th class="py-2.5 px-4 font-semibold text-right">"Action"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody class="divide-y divide-outline-variant/5 text-xs">
                                                        <For
                                                            each=move || variants_list.get()
                                                            key=|v| v.id
                                                            children=move |v| {
                                                                let status_color = if v.is_published {
                                                                    "text-[#c6fff3] font-semibold".to_string()
                                                                } else if v.localization_status == LocalizationStatus::Pending {
                                                                    "text-amber-400 font-semibold".to_string()
                                                                } else {
                                                                    "text-on-surface-variant".to_string()
                                                                };
                                                                let status_txt = if v.is_published {
                                                                    "● Live".to_string()
                                                                } else if v.localization_status == LocalizationStatus::Pending {
                                                                    "⚠ Awaiting Review".to_string()
                                                                } else {
                                                                    format!("● {}", v.launch_mode.label())
                                                                };
                                                                let is_ai = v.copy_strategy == CopyStrategy::AiGenerated;
                                                                let strategy_color = if is_ai {
                                                                    "text-primary font-semibold"
                                                                } else {
                                                                    "text-on-surface-variant"
                                                                };
                                                                let last_run = if v.created_at.is_empty() {
                                                                    "—".to_string()
                                                                } else {
                                                                    v.created_at.clone()
                                                                };
                                                                let market_name = v.meta_title.clone().unwrap_or_else(|| v.variant_slug.clone());

                                                                view! {
                                                                    <tr class="hover:bg-surface-container-high/40">
                                                                        <td class="py-3 px-4 font-mono font-bold">{v.locale.clone()}</td>
                                                                        <td class="py-3 px-4">{market_name}</td>
                                                                        <td class=format!("py-3 px-4 {}", status_color)>{status_txt}</td>
                                                                        <td class="py-3 px-4 text-on-surface-variant">{last_run}</td>
                                                                        <td class=format!("py-3 px-4 {}", strategy_color)>{v.copy_strategy.label()}</td>
                                                                        <td class="py-3 px-4 text-right">
                                                                            <Show
                                                                                when=move || v.localization_status == LocalizationStatus::Pending
                                                                                fallback=move || view! {
                                                                                    <button class="px-2 py-1 bg-surface-bright text-on-surface rounded text-[10px] font-bold uppercase" on:click=move |_| toast.show_toast("Success", "Opening locale config...", "success")>"Config"</button>
                                                                                }
                                                                            >
                                                                                <button class="px-2.5 py-1 bg-primary/20 text-primary border border-primary/30 rounded text-[10px] font-bold uppercase" on:click=move |_| toast.show_toast("Success", "Opening translations review panel...", "success")>"Review"</button>
                                                                            </Show>
                                                                        </td>
                                                                    </tr>
                                                                }
                                                            }
                                                        />
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                    </div>

                                    // 3. AI Localization Tab
                                    <div class=move || format!("space-y-6 {}", if active_tab.get() == "localization" { "block" } else { "hidden" })>
                                        <div class="bg-surface-container border border-outline-variant/20 rounded-xl overflow-hidden p-6">
                                            <div class="flex justify-between items-center mb-6 border-b border-outline-variant/10 pb-4">
                                                <div>
                                                    <h3 class="text-sm font-bold text-on-surface">"AI Translation & Localization Registry"</h3>
                                                    <p class="text-[10px] text-on-surface-variant">"Auto-localize all marketing page templates using advanced LLM pipelines."</p>
                                                </div>
                                                <button class="px-4 py-2 bg-[#7C3AED] hover:opacity-90 text-white rounded-lg text-xs font-bold uppercase tracking-wider" on:click=move |_| toast.show_toast("Warning", "Enqueuing AI localization...", "warn")>"Run Translation Engine"</button>
                                            </div>

                                            // Active localization worker status
                                            <div class="bg-[#7C3AED]/10 border border-[#7C3AED]/30 rounded-xl p-4 flex items-center gap-4 mb-6">
                                                <div class="w-3.5 h-3.5 rounded-full bg-[#7C3AED] animate-pulse flex-shrink-0"></div>
                                                <div class="flex-1 min-w-0">
                                                    <div class="text-xs font-bold text-[#a78bfa]">"localize_product_page · ACTIVE"</div>
                                                    <div class="text-[10px] text-on-surface-variant/80 mt-0.5">{p.name.clone()} " · fr-FR · Gemini 1.5 Pro · 1m 02s elapsed"</div>
                                                </div>
                                                <button class="px-3 py-1 bg-surface-container-high border border-outline-variant/20 text-on-surface hover:bg-surface-bright/20 rounded text-[10.5px] font-bold uppercase" on:click=move |_| toast.show_toast("Success", "Active localization job aborted.", "success")>"Abort"</button>
                                            </div>

                                            // History Table
                                            <div class="text-xs font-bold text-on-surface mb-3">"Localization Execution Log"</div>
                                            <div class="overflow-x-auto border border-outline-variant/10 rounded-lg">
                                                <table class="w-full border-collapse text-left">
                                                    <thead>
                                                        <tr class="text-[10px] font-bold text-on-surface-variant uppercase tracking-wider border-b border-outline-variant/10 bg-[var(--bg-base)]/20">
                                                            <th class="py-2 px-3 font-semibold">"Job Type"</th>
                                                            <th class="py-2 px-3 font-semibold">"Target Locale"</th>
                                                            <th class="py-2 px-3 font-semibold">"Status"</th>
                                                            <th class="py-2 px-3 font-semibold">"Duration"</th>
                                                            <th class="py-2 px-3 font-semibold">"Date Executed"</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody class="divide-y divide-outline-variant/5 text-[11.5px]">
                                                        <tr class="hover:bg-surface-container-high/40">
                                                            <td class="py-2.5 px-3 font-mono">"localize_product_page"</td>
                                                            <td class="py-2.5 px-3 font-bold">"fr-FR"</td>
                                                            <td class="py-2.5 px-3 text-primary font-semibold">"↻ Processing"</td>
                                                            <td class="py-2.5 px-3 font-mono text-[10.5px]">"1m 02s"</td>
                                                            <td class="py-2.5 px-3 text-on-surface-variant">"Jun 09 · 23:30"</td>
                                                        </tr>
                                                        <tr class="hover:bg-surface-container-high/40">
                                                            <td class="py-2.5 px-3 font-mono">"localize_product_page"</td>
                                                            <td class="py-2.5 px-3 font-bold">"pt-BR"</td>
                                                            <td class="py-2.5 px-3 text-[#c6fff3] font-semibold">"✓ Complete"</td>
                                                            <td class="py-2.5 px-3 font-mono text-[10.5px]">"44s"</td>
                                                            <td class="py-2.5 px-3 text-on-surface-variant">"Jun 08 · 14:22"</td>
                                                        </tr>
                                                        <tr class="hover:bg-surface-container-high/40">
                                                            <td class="py-2.5 px-3 font-mono">"localize_product_page"</td>
                                                            <td class="py-2.5 px-3 font-bold">"es-MX"</td>
                                                            <td class="py-2.5 px-3 text-[#c6fff3] font-semibold">"✓ Complete"</td>
                                                            <td class="py-2.5 px-3 font-mono text-[10.5px]">"38s"</td>
                                                            <td class="py-2.5 px-3 text-on-surface-variant">"Jun 07 · 11:10"</td>
                                                        </tr>
                                                    </tbody>
                                                </table>
                                            </div>
                                        </div>
                                    </div>

                                    // 4. Settings Tab
                                    <div class=move || format!("space-y-6 {}", if active_tab.get() == "settings" { "block" } else { "hidden" })>
                                        <div class="bg-surface-container border border-outline-variant/20 rounded-xl p-6">
                                            <h3 class="text-sm font-bold text-on-surface mb-4">"Product Settings & Configuration"</h3>
                                            <div class="space-y-4">
                                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                                    <div class="space-y-1.5">
                                                        <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Product Display Name"</label>
                                                        <input type="text" class="w-full bg-[var(--bg-elevated)] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2.5 outline-none focus:ring-1 focus:ring-primary focus:border-primary" value=p.name.clone() />
                                                    </div>
                                                    <div class="space-y-1.5">
                                                        <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Launch Mode Override"</label>
                                                        <select class="w-full bg-[var(--bg-elevated)] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2.5 outline-none focus:ring-1 focus:ring-primary focus:border-primary cursor-pointer">
                                                            <option selected=true>{p.launch_mode.clone()}</option>
                                                            <option>"draft"</option>
                                                            <option>"active"</option>
                                                            <option>"beta"</option>
                                                            <option>"waitlist"</option>
                                                        </select>
                                                    </div>
                                                </div>
                                                <div class="space-y-1.5">
                                                    <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Deploy Webhook Hook URL"</label>
                                                    <input type="text" class="w-full bg-[var(--bg-elevated)] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2.5 outline-none focus:ring-1 focus:ring-primary focus:border-primary font-mono text-[11px]" value=p.deploy_hook_url.clone().unwrap_or_default() />
                                                </div>
                                                <div class="flex justify-end gap-3 pt-4 border-t border-outline-variant/10">
                                                    <button class="px-4 py-2 border border-outline-variant/30 text-on-surface hover:bg-surface-bright/20 rounded-lg text-xs font-semibold" on:click=move |_| toast.show_toast("Warning", "Changes discarded.", "warn")>"Discard"</button>
                                                    <button class="btn-primary-gradient px-4 py-2 text-on-primary-container rounded-lg text-xs font-bold" on:click=handle_save_settings>"Save Settings"</button>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    // 5. Deploy Status Tab
                                    <div class=move || format!("space-y-6 {}", if active_tab.get() == "deploy" { "block" } else { "hidden" })>
                                        <div class="bg-surface-container border border-outline-variant/20 rounded-xl p-6">
                                            <h3 class="text-sm font-bold text-on-surface mb-6">"Infrastructure Deployment Progress Stepper"</h3>
                                            {move || {
                                                if let Some(Some(status)) = product_deploy_status.get() {
                                                    view! {
                                                        <div class="mb-6 p-4 bg-[#0a84ff]/10 border border-[#0a84ff]/30 rounded-xl">
                                                            <div class="text-xs font-bold text-[#7bd0ff]">"Deploy Status: " {status.status.clone()}</div>
                                                            <div class="text-[10px] text-on-surface-variant/80 mt-1">"Message: " {status.message.clone().unwrap_or_else(|| "No additional status information.".to_string())}</div>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {}.into_any()
                                                }
                                            }}
                                            
                                            // Vertical list of tasks
                                            <div class="space-y-6 relative pl-6 before:absolute before:left-2 before:top-2 before:bottom-2 before:w-[1px] before:bg-outline-variant/20">
                                                
                                                // Step 1
                                                <div class="relative flex items-start gap-4">
                                                    <div class="absolute -left-6 w-4.5 h-4.5 rounded-full bg-[#c6fff3]/20 border border-[#c6fff3] flex items-center justify-center text-[#c6fff3] text-[9px] font-bold">
                                                        "✓"
                                                    </div>
                                                    <div>
                                                        <div class="text-xs font-bold text-on-surface">"1. Database Schema Migrations"</div>
                                                        <p class="text-[10px] text-on-surface-variant/80 mt-0.5">"Variant data pricing tables synced to central DB."</p>
                                                    </div>
                                                </div>

                                                // Step 2
                                                <div class="relative flex items-start gap-4">
                                                    <div class="absolute -left-6 w-4.5 h-4.5 rounded-full bg-[#c6fff3]/20 border border-[#c6fff3] flex items-center justify-center text-[#c6fff3] text-[9px] font-bold">
                                                        "✓"
                                                    </div>
                                                    <div>
                                                        <div class="text-xs font-bold text-on-surface">"2. CDN Cache Edge Routing"</div>
                                                        <p class="text-[10px] text-on-surface-variant/80 mt-0.5">"Cloudflare Custom Hostname API mapped successfully."</p>
                                                    </div>
                                                </div>

                                                // Step 3
                                                <div class="relative flex items-start gap-4">
                                                    <div class="absolute -left-6 w-4.5 h-4.5 rounded-full bg-[#7C3AED]/20 border border-[#7C3AED] flex items-center justify-center text-[#a78bfa] text-[9px] font-bold animate-pulse">
                                                        "↻"
                                                    </div>
                                                    <div>
                                                        <div class="text-xs font-bold text-[#a78bfa]">"3. AI Translation Localization Pipeline"</div>
                                                        <p class="text-[10px] text-on-surface-variant/80 mt-0.5">"Translating Homepage meta titles and labels to French."</p>
                                                    </div>
                                                </div>

                                                // Step 4
                                                <div class="relative flex items-start gap-4 opacity-55">
                                                    <div class="absolute -left-6 w-4.5 h-4.5 rounded-full bg-[var(--bg-elevated)] border border-outline-variant/30 flex items-center justify-center text-on-surface-variant/80 text-[9px] font-semibold">
                                                        "4"
                                                    </div>
                                                    <div>
                                                        <div class="text-xs font-bold text-on-surface">"4. SSL Cert Verification Check"</div>
                                                        <p class="text-[10px] text-on-surface-variant/80 mt-0.5">"Await certification authority verification."</p>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                </div>
                            }
                        }}
                    </Show>
                </div>
            </div>

            // ── Modal Overlays ──

            // 1. Create Product Modal
            <Show when=move || show_create_modal.get()>
                <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-md p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_create_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold text-on-surface mb-4">"New Platform Product"</h3>
                        <div class="space-y-4 mb-6">
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Product Name"</label>
                                <input type="text" placeholder="Folio Enterprise" class="w-full bg-[var(--bg-elevated)] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2.5 outline-none focus:ring-1 focus:ring-primary focus:border-primary" on:input=move |ev| new_prod_name.set(event_target_value(&ev)) />
                            </div>
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Slug"</label>
                                <input type="text" placeholder="folio-enterprise" class="w-full bg-[var(--bg-elevated)] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2.5 outline-none focus:ring-1 focus:ring-primary focus:border-primary" on:input=move |ev| new_prod_slug.set(event_target_value(&ev)) />
                            </div>
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Product Category"</label>
                                <select class="w-full bg-[var(--bg-elevated)] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2.5 outline-none focus:ring-1 focus:ring-primary focus:border-primary cursor-pointer">
                                    <option>"folio"</option>
                                    <option>"anchor"</option>
                                    <option>"network"</option>
                                </select>
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface hover:bg-surface-bright/20 transition-all" on:click=move |_| show_create_modal.set(false)>"Cancel"</button>
                            <button class="btn-primary-gradient px-4 py-2 text-on-primary-container rounded-lg text-xs font-bold transition-all" on:click=handle_create_product>"Create Product"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // 2. Publish Marketing Page Modal
            <Show when=move || show_publish_modal.get()>
                <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-md p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_publish_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold text-on-surface mb-2">"Publish Marketing Page"</h3>
                        <p class="text-on-surface-variant text-xs leading-relaxed mb-6">"This action fires a secure deploy webhook to Cloudflare Pages. The public landing marketing layout for this product catalog will be updated in production."</p>
                        <div class="flex justify-end gap-3">
                            <button class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface hover:bg-surface-bright/20 transition-all" on:click=move |_| show_publish_modal.set(false)>"Cancel"</button>
                            <button class="btn-primary-gradient px-4 py-2 text-on-primary-container rounded-lg text-xs font-bold transition-all" on:click=handle_publish>"Trigger Deploy"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // 3. Variant Bulk Generation Modal
            <Show when=move || show_variant_gen_modal.get()>
                <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-md p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_variant_gen_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold text-on-surface mb-2">"Bulk Generate Variants"</h3>
                        <p class="text-on-surface-variant text-xs leading-relaxed mb-6">"Trigger pricing variants generation for regional sub-pages? This will create country x language variant records and queue translation."</p>
                        <div class="flex justify-end gap-3">
                            <button class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface hover:bg-surface-bright/20 transition-all" on:click=move |_| show_variant_gen_modal.set(false)>"Cancel"</button>
                            <button class="btn-primary-gradient px-4 py-2 text-on-primary-container rounded-lg text-xs font-bold transition-all" on:click=handle_bulk_generate>"Generate & Queue"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // 4. Add Domain Alias Modal
            <Show when=move || show_add_alias_modal.get()>
                <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-md p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_add_alias_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold text-on-surface mb-4">"Add Custom URL Alias"</h3>
                        <div class="space-y-4 mb-6">
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Domain Alias / Slug"</label>
                                <input type="text" placeholder="e.g. rent.atlas.app" class="w-full bg-[var(--bg-elevated)] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2.5 outline-none focus:ring-1 focus:ring-primary focus:border-primary" />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface hover:bg-surface-bright/20 transition-all" on:click=move |_| show_add_alias_modal.set(false)>"Cancel"</button>
                            <button class="btn-primary-gradient px-4 py-2 text-on-primary-container rounded-lg text-xs font-bold transition-all" on:click=move |_| { show_add_alias_modal.set(false); toast.show_toast("Success", "Custom domain alias pending DNS resolution.", "success") }>"Verify Alias"</button>
                        </div>
                    </div>
                </div>
            </Show>

            // 5. Add Locale Variant Modal
            <Show when=move || show_add_locale_modal.get()>
                <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-md p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_add_locale_modal.set(false)>"✕"</button>
                        <h3 class="text-lg font-bold text-on-surface mb-4">"Create Regional Variant"</h3>
                        <div class="space-y-4 mb-6">
                            <div class="space-y-1.5">
                                <label class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Target Locale"</label>
                                <select class="w-full bg-[var(--bg-elevated)] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2.5 outline-none focus:ring-1 focus:ring-primary focus:border-primary cursor-pointer">
                                    <option>"de-DE · German"</option>
                                    <option>"ja-JP · Japanese"</option>
                                    <option>"es-ES · Spanish (Spain)"</option>
                                    <option>"it-IT · Italian"</option>
                                </select>
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface hover:bg-surface-bright/20 transition-all" on:click=move |_| show_add_locale_modal.set(false)>"Cancel"</button>
                            <button class="btn-primary-gradient px-4 py-2 text-on-primary-container rounded-lg text-xs font-bold transition-all" on:click=move |_| { show_add_locale_modal.set(false); toast.show_toast("Success", "German locale variant created & translation queued.", "success") }>"Add Variant"</button>
                        </div>
                    </div>
                </div>
            </Show>

        </div>
    }
}
