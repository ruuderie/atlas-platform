use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;

use crate::api::products::{get_product_detail, update_product_detail, publish_marketing};
use crate::api::models::UpdateProductBody;
use crate::app::GlobalToast;

#[component]
pub fn ProductDetail() -> impl IntoView {
    let params = use_params_map();
    let toast = use_context::<GlobalToast>().expect("toast context missing");

    // Parse UUID from route param — fall back gracefully on bad ID
    let product_uuid = move || {
        params.with(|p| {
            p.get("id")
                .and_then(|s| Uuid::parse_str(&s).ok())
        })
    };

    // ── Load product from API ──────────────────────────────────────────────
    let product_res = LocalResource::new(move || async move {
        match product_uuid() {
            Some(id) => get_product_detail(id).await.ok(),
            None => None,
        }
    });

    // ── Editable form signals (populated from API on load) ─────────────────
    let product_name = RwSignal::new(String::new());
    let product_tagline = RwSignal::new(String::new());
    let product_domain = RwSignal::new(String::new());
    let product_status = RwSignal::new(String::new());
    let product_slug = RwSignal::new(String::new());
    let waitlist_count = RwSignal::new(0i32);
    let pre_order_enabled = RwSignal::new(false);

    // Populate signals when product loads
    Effect::new(move |_| {
        if let Some(Some(p)) = product_res.get() {
            product_name.set(p.name.clone());
            product_tagline.set(p.tagline.clone().unwrap_or_default());
            product_domain.set(p.apex_domain.clone().unwrap_or_default());
            product_status.set(p.status.clone());
            product_slug.set(p.slug.clone());
            waitlist_count.set(p.waitlist_count);
            pre_order_enabled.set(p.pre_order_enabled);
        }
    });

    // ── SEO score derived from completeness of real fields ─────────────────
    let seo_score = Signal::derive(move || {
        let mut score = 40i32; // base
        if !product_name.get().is_empty() { score += 20; }
        if !product_tagline.get().is_empty() { score += 20; }
        if !product_domain.get().is_empty() { score += 20; }
        score.min(100)
    });

    // ── Save handler → PATCH /api/admin/platform/products/:id ─────────────
    let saving = RwSignal::new(false);
    let handle_save = move |_| {
        let Some(id) = product_uuid() else {
            toast.show_toast("Error", "Invalid product ID", "error");
            return;
        };
        if product_name.get().trim().is_empty() {
            toast.show_toast("Validation", "Product name cannot be empty", "error");
            return;
        }
        saving.set(true);
        let body = UpdateProductBody {
            name: Some(product_name.get()),
            tagline: Some(product_tagline.get()).filter(|s| !s.is_empty()),
            status: None,
            deploy_hook_url: None,
            marketing_page_cms_id: None,
        };
        leptos::task::spawn_local(async move {
            match update_product_detail(id, body).await {
                Ok(updated) => {
                    product_name.set(updated.name);
                    product_tagline.set(updated.tagline.unwrap_or_default());
                    product_domain.set(updated.apex_domain.unwrap_or_default());
                    toast.show_toast("Saved", "Product details updated successfully.", "success");
                }
                Err(e) => {
                    toast.show_toast("Save Failed", &e, "error");
                }
            }
            saving.set(false);
        });
    };

    // ── Publish handler → POST /api/admin/platform/products/:id/publish-marketing ──
    let publishing = RwSignal::new(false);
    let handle_publish = move |_| {
        let Some(id) = product_uuid() else {
            toast.show_toast("Error", "Invalid product ID", "error");
            return;
        };
        publishing.set(true);
        leptos::task::spawn_local(async move {
            match publish_marketing(id).await {
                Ok(_) => {
                    toast.show_toast("Published", "Marketing page publish job queued.", "success");
                }
                Err(e) => {
                    toast.show_toast("Publish Failed", &e, "error");
                }
            }
            publishing.set(false);
        });
    };

    // ── Tab state ──────────────────────────────────────────────────────────
    let active_tab = RwSignal::new("general".to_string());

    let loading = move || product_res.get().is_none();
    let not_found = move || matches!(product_res.get(), Some(None));

    view! {
        <div class="space-y-6">

            // ── Loading / Not Found states ──
            <Show when=loading>
                <div class="p-12 text-center text-on-surface-variant/60 text-sm">
                    "Loading product…"
                </div>
            </Show>
            <Show when=not_found>
                <div class="p-12 text-center text-on-surface-variant/60 text-sm">
                    <span class="material-symbols-outlined text-[32px] text-on-surface-variant/30 block mb-3">"error_outline"</span>
                    "Product not found or you don't have access."
                    <div class="mt-4">
                        <a href="/products" class="text-primary text-xs hover:underline">"← Back to Products"</a>
                    </div>
                </div>
            </Show>

            <Show when=move || matches!(product_res.get(), Some(Some(_)))>

                // ── Breadcrumb ──
                <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                    <a href="/products" class="hover:text-primary transition-colors">"Products"</a>
                    <span class="material-symbols-outlined text-[12px]">"chevron_right"</span>
                    <span class="text-primary/70">{move || product_name.get()}</span>
                </nav>

                // ── Product Header / Toolbar ──
                <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 bg-surface-container-low border border-outline-variant/20 p-6 rounded-2xl shadow-sm">
                    <div class="flex items-center gap-3 w-full md:w-auto">
                        <span class="w-8 h-8 rounded-lg bg-primary/20 text-primary flex items-center justify-center font-black text-sm select-none">
                            "P"
                        </span>
                        <input
                            type="text"
                            class="bg-transparent border-b border-transparent focus:border-primary text-xl font-extrabold text-on-surface tracking-tight outline-none focus:outline-none flex-1 max-w-sm transition-all"
                            prop:value=product_name
                            on:input=move |ev| product_name.set(event_target_value(&ev))
                        />
                        <span class=move || {
                            let s = product_status.get();
                            match s.as_str() {
                                "active"   => "inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 uppercase tracking-wider",
                                "waitlist" => "inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20 uppercase tracking-wider",
                                "draft"    => "inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-outline-variant/20 text-on-surface-variant border border-outline-variant/30 uppercase tracking-wider",
                                _          => "inline-flex items-center px-2 py-0.5 rounded text-[9px] font-bold bg-outline-variant/20 text-on-surface-variant border border-outline-variant/30 uppercase tracking-wider",
                            }
                        }>
                            {move || product_status.get()}
                        </span>
                    </div>
                    <div class="flex items-center gap-3">
                        <button
                            class=move || format!(
                                "btn-ghost px-4 py-2 rounded-lg text-sm font-semibold border border-outline-variant/30 transition-all {}",
                                if saving.get() { "opacity-40 cursor-not-allowed" } else { "hover:bg-surface-bright/20" }
                            )
                            disabled=move || saving.get()
                            on:click=handle_save
                        >
                            {move || if saving.get() { "Saving…" } else { "Save Changes" }}
                        </button>
                        <button
                            class=move || format!(
                                "btn-primary-gradient px-4 py-2 rounded-lg text-sm font-semibold text-on-primary-container shadow-md shadow-primary/10 transition-all {}",
                                if publishing.get() { "opacity-40 cursor-not-allowed" } else { "" }
                            )
                            disabled=move || publishing.get()
                            on:click=handle_publish
                        >
                            {move || if publishing.get() { "Publishing…" } else { "Publish Live" }}
                        </button>
                    </div>
                </div>

                // ── Tab Navigation ──
                <div class="flex border-b border-outline-variant/20 overflow-x-auto shrink-0 select-none">
                    {
                        let tab_btn = move |id: &str, label: &str| {
                            let id = id.to_string();
                            let label = label.to_string();
                            let id_class = id.clone();
                            let id_click = id.clone();
                            view! {
                                <button
                                    class=move || if active_tab.get() == id_class {
                                        "px-4 py-2.5 text-sm font-semibold text-primary border-b-2 border-primary transition-all shrink-0 bg-transparent"
                                    } else {
                                        "px-4 py-2.5 text-sm text-on-surface-variant hover:text-on-surface transition-all shrink-0 bg-transparent"
                                    }
                                    on:click=move |_| active_tab.set(id_click.clone())
                                >
                                    {label.clone()}
                                </button>
                            }
                        };
                        view! {
                            {tab_btn("general", "General Info")}
                            {tab_btn("pricing", "Pricing & Plans")}
                            {tab_btn("waitlist", "Waitlist Leads")}
                            {tab_btn("seo", "SEO & Metadata")}
                        }
                    }
                </div>

                // ── TAB CONTENT: General Info ──
                <Show when=move || active_tab.get() == "general">
                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                        <div class="lg:col-span-2 space-y-6">
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-4">
                                <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Marketing Profile"</h3>

                                // Slug (read-only — changing slug breaks URLs)
                                <div class="space-y-1">
                                    <label class="text-xs font-semibold text-on-surface-variant">"Product Slug"</label>
                                    <input
                                        type="text"
                                        class="w-full bg-surface-container/50 border border-outline-variant/20 text-on-surface-variant text-sm rounded-lg px-3 py-2.5 cursor-not-allowed"
                                        prop:value=product_slug
                                        disabled
                                    />
                                    <p class="text-[10px] text-on-surface-variant/50">"Read-only — slug changes require a migration."</p>
                                </div>

                                // Apex domain
                                <div class="space-y-1 mt-4">
                                    <label class="text-xs font-semibold text-on-surface-variant">"Apex Domain / Vanity URL"</label>
                                    <input
                                        type="text"
                                        class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 focus:ring-1 focus:ring-primary focus:border-primary transition-all"
                                        prop:value=product_domain
                                        on:input=move |ev| product_domain.set(event_target_value(&ev))
                                        placeholder="e.g. folio.rentals"
                                    />
                                    <p class="text-[10px] text-on-surface-variant/50">"The canonical vanity URL that maps inbound visitors directly to this product storefront"</p>
                                </div>

                                // Tagline / description
                                <div class="space-y-1 mt-4">
                                    <label class="text-xs font-semibold text-on-surface-variant">"Tagline / Description"</label>
                                    <textarea
                                        class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 focus:ring-1 focus:ring-primary focus:border-primary transition-all h-28 resize-none"
                                        prop:value=product_tagline
                                        on:input=move |ev| product_tagline.set(event_target_value(&ev))
                                        placeholder="Public-facing description shown on cards and directories…"
                                    >
                                    </textarea>
                                    <p class="text-[10px] text-on-surface-variant/50">"Public-facing tagline shown on cards, directories, and default SEO descriptions"</p>
                                </div>
                            </div>
                        </div>

                        // Sidebar: live stats from product model
                        <div class="space-y-6">
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-4">
                                <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Product Stats"</h3>
                                <div class="divide-y divide-outline-variant/10">
                                    <div class="flex justify-between py-3 text-xs">
                                        <span class="text-on-surface-variant">"Waitlist Signups"</span>
                                        <span class="font-bold text-on-surface font-mono">{move || waitlist_count.get().to_string()}</span>
                                    </div>
                                    <div class="flex justify-between py-3 text-xs">
                                        <span class="text-on-surface-variant">"Pre-order"</span>
                                        <span class=move || if pre_order_enabled.get() {
                                            "font-bold text-emerald-400"
                                        } else {
                                            "font-bold text-on-surface-variant/50"
                                        }>
                                            {move || if pre_order_enabled.get() { "Enabled" } else { "Disabled" }}
                                        </span>
                                    </div>
                                    <div class="flex justify-between py-3 text-xs">
                                        <span class="text-on-surface-variant">"Status"</span>
                                        <span class="font-bold text-on-surface uppercase tracking-wider text-[10px]">{move || product_status.get()}</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </Show>

                // ── TAB CONTENT: Pricing ──
                <Show when=move || active_tab.get() == "pricing">
                    <div class="space-y-4">
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm">
                            <div class="flex justify-between items-center mb-6">
                                <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Pricing Plans & Feature Matrix"</h3>
                                <button
                                    class="btn-ghost px-3 py-1.5 rounded-lg border border-outline-variant/30 text-xs font-bold uppercase tracking-wider opacity-40 cursor-not-allowed"
                                    disabled
                                    title="Pricing tier management coming soon"
                                >"+ Add Tier"</button>
                            </div>

                            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                <div class="bg-surface-container p-5 rounded-xl border border-outline-variant/20 flex flex-col justify-between">
                                    <div>
                                        <div class="flex items-center justify-between mb-2">
                                            <h4 class="font-bold text-on-surface">"Basic Plan"</h4>
                                            <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">"$400/mo"</span>
                                        </div>
                                        <p class="text-xs text-on-surface-variant/70 mb-4 leading-relaxed">"Entry level hosting context for self-managed small portfolio landlords."</p>
                                        <ul class="text-xs text-on-surface-variant space-y-2">
                                            <li class="flex items-center gap-2">
                                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                                "Up to 25 properties"
                                            </li>
                                            <li class="flex items-center gap-2">
                                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                                "Standard support"
                                            </li>
                                        </ul>
                                    </div>
                                    <button
                                        class="btn-ghost w-full mt-6 text-xs justify-center py-2 border border-outline-variant/30 rounded-md opacity-40 cursor-not-allowed"
                                        disabled
                                        title="Plan editing coming soon"
                                    >"Edit plan"</button>
                                </div>

                                <div class="bg-surface-container p-5 rounded-xl border border-primary/20 flex flex-col justify-between relative">
                                    <div class="absolute -top-2.5 right-4 bg-primary text-on-primary-container px-2 py-0.5 rounded text-[8px] font-bold uppercase tracking-widest">"Popular"</div>
                                    <div>
                                        <div class="flex items-center justify-between mb-2">
                                            <h4 class="font-bold text-on-surface">"Professional Plan"</h4>
                                            <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">"$900/mo"</span>
                                        </div>
                                        <p class="text-xs text-on-surface-variant/70 mb-4 leading-relaxed">"Automated TOT reporting and Stripe/Zelle payment routing."</p>
                                        <ul class="text-xs text-on-surface-variant space-y-2">
                                            <li class="flex items-center gap-2">
                                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                                "Unlimited properties"
                                            </li>
                                            <li class="flex items-center gap-2">
                                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                                "Priority SLA support"
                                            </li>
                                        </ul>
                                    </div>
                                    <button
                                        class="btn-primary w-full mt-6 text-xs justify-center py-2 rounded-md opacity-40 cursor-not-allowed"
                                        disabled
                                        title="Plan editing coming soon"
                                    >"Edit plan"</button>
                                </div>

                                <div class="bg-surface-container p-5 rounded-xl border border-outline-variant/20 flex flex-col justify-between">
                                    <div>
                                        <div class="flex items-center justify-between mb-2">
                                            <h4 class="font-bold text-on-surface">"Enterprise Plan"</h4>
                                            <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">"Custom"</span>
                                        </div>
                                        <p class="text-xs text-on-surface-variant/70 mb-4 leading-relaxed">"Custom SLA routing, FMCSA compliance audits, and dedicated storage reclamation."</p>
                                        <ul class="text-xs text-on-surface-variant space-y-2">
                                            <li class="flex items-center gap-2">
                                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                                "Custom cloud boundaries"
                                            </li>
                                            <li class="flex items-center gap-2">
                                                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400"></span>
                                                "Dedicated technical team"
                                            </li>
                                        </ul>
                                    </div>
                                    <button
                                        class="btn-ghost w-full mt-6 text-xs justify-center py-2 border border-outline-variant/30 rounded-md opacity-40 cursor-not-allowed"
                                        disabled
                                        title="Plan editing coming soon"
                                    >"Edit plan"</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </Show>

                // ── TAB CONTENT: Waitlist ──
                <Show when=move || active_tab.get() == "waitlist">
                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm">
                        <div class="flex items-center justify-between mb-6">
                            <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Waitlist Leads"</h3>
                            <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-primary/10 text-primary border border-primary/20 font-mono">
                                {move || format!("{} signups", waitlist_count.get())}
                            </span>
                        </div>
                        <div class="overflow-x-auto border border-outline-variant/20 rounded-lg">
                            <div class="p-8 text-center text-xs text-on-surface-variant/60 flex flex-col items-center gap-3">
                                <span class="material-symbols-outlined text-[32px] text-on-surface-variant/30">"hourglass_empty"</span>
                                <p>"Waitlist lead details API is pending. "<strong>{move || waitlist_count.get().to_string()}</strong>" signup(s) recorded. Leads will appear here once the detail endpoint is connected."</p>
                            </div>
                        </div>
                    </div>
                </Show>

                // ── TAB CONTENT: SEO ──
                <Show when=move || active_tab.get() == "seo">
                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                        <div class="lg:col-span-2 space-y-6">
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-6">
                                <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"SEO Checklist & Target Attributes"</h3>

                                <div class="divide-y divide-outline-variant/10">
                                    <div class="flex items-start gap-3 py-4 first:pt-0">
                                        <span class=move || if !product_name.get().is_empty() { "material-symbols-outlined text-emerald-400 mt-0.5" } else { "material-symbols-outlined text-amber-400 mt-0.5" }>
                                            {move || if !product_name.get().is_empty() { "check_circle" } else { "warning" }}
                                        </span>
                                        <div>
                                            <h4 class="text-xs font-bold text-on-surface">"Product Name / Page Title"</h4>
                                            <p class="text-[10px] text-on-surface-variant/70 mt-1">
                                                {move || if product_name.get().is_empty() { "No product name set. Update General Info." } else { "Title is set and will be used as the primary SEO title tag." }}
                                            </p>
                                        </div>
                                    </div>
                                    <div class="flex items-start gap-3 py-4">
                                        <span class=move || if !product_tagline.get().is_empty() { "material-symbols-outlined text-emerald-400 mt-0.5" } else { "material-symbols-outlined text-amber-400 mt-0.5" }>
                                            {move || if !product_tagline.get().is_empty() { "check_circle" } else { "warning" }}
                                        </span>
                                        <div>
                                            <h4 class="text-xs font-bold text-on-surface">"Meta Description / Tagline"</h4>
                                            <p class="text-[10px] text-on-surface-variant/70 mt-1">
                                                {move || if product_tagline.get().is_empty() { "No tagline set. Add one in General Info." } else { "Tagline will be used as the meta description." }}
                                            </p>
                                        </div>
                                    </div>
                                    <div class="flex items-start gap-3 py-4">
                                        <span class=move || if !product_domain.get().is_empty() { "material-symbols-outlined text-emerald-400 mt-0.5" } else { "material-symbols-outlined text-amber-400 mt-0.5" }>
                                            {move || if !product_domain.get().is_empty() { "check_circle" } else { "warning" }}
                                        </span>
                                        <div>
                                            <h4 class="text-xs font-bold text-on-surface">"Canonical Domain"</h4>
                                            <p class="text-[10px] text-on-surface-variant/70 mt-1">
                                                {move || if product_domain.get().is_empty() { "No apex domain set. Canonical URL will fall back to platform default." } else { "Apex domain is set and will be used as the canonical URL." }}
                                            </p>
                                        </div>
                                    </div>
                                    <div class="flex items-start gap-3 py-4 last:pb-0">
                                        <span class="material-symbols-outlined text-amber-400 mt-0.5">"warning"</span>
                                        <div>
                                            <h4 class="text-xs font-bold text-on-surface">"OpenGraph Image Missing"</h4>
                                            <p class="text-[10px] text-on-surface-variant/70 mt-1">"A generic fallback image is used. We recommend importing a custom high-fidelity banner asset."</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <div class="space-y-6">
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm flex flex-col items-center justify-center text-center">
                                <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant mb-4 self-start">"SEO Score"</h3>

                                // Progress Ring — driven by real field completeness
                                <div class="relative w-36 h-36 flex items-center justify-center">
                                    <svg class="w-full h-full transform -rotate-90">
                                        <circle cx="72" cy="72" r="60" class="stroke-outline-variant/20 fill-none stroke-[8]" />
                                        <circle
                                            cx="72"
                                            cy="72"
                                            r="60"
                                            class="stroke-primary fill-none stroke-[8] stroke-dasharray-[377] transition-all duration-500"
                                            style=move || format!("stroke-dashoffset: {};", 377 - (377 * seo_score.get()) / 100)
                                        />
                                    </svg>
                                    <div class="absolute flex flex-col items-center">
                                        <span class="text-3xl font-extrabold text-on-surface font-mono">{move || seo_score.get()}</span>
                                        <span class="text-[9px] font-bold text-on-surface-variant/50 uppercase tracking-wider">"/ 100"</span>
                                    </div>
                                </div>

                                <p class="text-xs text-on-surface-variant/80 mt-6 max-w-[200px]">
                                    {move || if seo_score.get() >= 80 {
                                        "Excellent index scores. Pages are fully optimized."
                                    } else if seo_score.get() >= 60 {
                                        "Good. Set the missing fields above to improve ranking."
                                    } else {
                                        "Needs attention. Key meta fields remain unoptimized."
                                    }}
                                </p>
                            </div>
                        </div>
                    </div>
                </Show>

            </Show>
        </div>
    }
}
