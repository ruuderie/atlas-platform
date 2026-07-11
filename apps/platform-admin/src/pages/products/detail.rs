use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::marketing::{FoundingSpotTier, folio_public_path_hint};
use uuid::Uuid;

use crate::api::crm::get_leads;
use crate::api::models::UpdateProductBody;
use crate::api::products::{
    ProductPlanBillingInterval, ProductPlanInput, ProductPlanModel, UpsertProductTemplateBody,
    create_product_plan, delete_product_plan, get_product_detail, get_template, get_variants,
    list_product_plans, publish_marketing, update_product_detail, update_product_plan,
    upsert_template,
};
use crate::app::GlobalToast;

#[component]
pub fn ProductDetail() -> impl IntoView {
    let params = use_params_map();
    let toast = use_context::<GlobalToast>().expect("toast context missing");

    // Parse UUID from route param — fall back gracefully on bad ID
    let product_uuid = move || params.with(|p| p.get("id").and_then(|s| Uuid::parse_str(&s).ok()));

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
    let hero_eyebrow = RwSignal::new(String::new());
    let hero_headline = RwSignal::new(String::new());
    let hero_headline_accent = RwSignal::new(String::new());
    let hero_subhead = RwSignal::new(String::new());
    let hero_proof_items = RwSignal::new(String::new());
    let hero_pricing_eyebrow = RwSignal::new(String::new());
    let hero_pricing_heading = RwSignal::new(String::new());
    let hero_pricing_subtitle = RwSignal::new(String::new());
    let hero_cta_label = RwSignal::new(String::new());
    let hero_spot_inventory = RwSignal::new(String::new());

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

    // Product marketing template — optional; Folio falls back when absent.
    let template_res = LocalResource::new(move || async move {
        match product_uuid() {
            Some(id) => get_template(id).await.ok(),
            None => None,
        }
    });

    Effect::new(move |_| {
        if let Some(template_opt) = template_res.get() {
            if let Some(template) = template_opt {
                hero_eyebrow.set(hero_string(&template.hero_payload, "eyebrow"));
                hero_headline.set(hero_string(&template.hero_payload, "headline"));
                hero_headline_accent.set(hero_string(&template.hero_payload, "headline_accent"));
                hero_subhead.set(hero_string(&template.hero_payload, "subhead"));
                hero_proof_items
                    .set(hero_string_array(&template.hero_payload, "proof_items").join("\n"));
                hero_pricing_eyebrow.set(hero_string(&template.hero_payload, "pricing_eyebrow"));
                hero_pricing_heading.set(hero_string(&template.hero_payload, "pricing_heading"));
                hero_pricing_subtitle.set(hero_string(&template.hero_payload, "pricing_subtitle"));
                hero_spot_inventory.set(hero_spot_inventory_json(&template.hero_payload));
                hero_cta_label.set(template.cta_label);
            } else {
                hero_eyebrow.set(String::new());
                hero_headline.set(String::new());
                hero_headline_accent.set(String::new());
                hero_subhead.set(String::new());
                hero_proof_items.set(String::new());
                hero_pricing_eyebrow.set(String::new());
                hero_pricing_heading.set(String::new());
                hero_pricing_subtitle.set(String::new());
                hero_spot_inventory.set(String::new());
                hero_cta_label.set(String::new());
            }
        }
    });

    // Waitlist leads — fetched on-demand when slug is known
    let waitlist_leads_res = LocalResource::new(move || async move {
        let slug = product_slug.get();
        if slug.is_empty() {
            return vec![];
        }
        let prefix = format!("waitlist:{}", slug);
        get_leads(None, 1, 200, None, Some(&prefix))
            .await
            .unwrap_or_default()
    });

    // Variants — fetched when product_uuid is known
    let variants_res = LocalResource::new(move || async move {
        match product_uuid() {
            Some(id) => get_variants(id).await.unwrap_or_default(),
            None => vec![],
        }
    });

    // ── Product-scoped marketing plans (for the Pricing tab) ─────────────────
    let plans_trigger = RwSignal::new(0u32);
    let product_plans_res = LocalResource::new(move || async move {
        let _ = plans_trigger.get();
        match product_uuid() {
            Some(id) => list_product_plans(id).await.unwrap_or_default(),
            None => vec![],
        }
    });

    // Plan edit modal state
    let edit_plan_id = RwSignal::new(Option::<Uuid>::None); // Some(id) = editing, None = creating
    let edit_plan_slug = RwSignal::new(String::new());
    let edit_plan_name = RwSignal::new(String::new());
    let edit_plan_tagline = RwSignal::new(String::new());
    let edit_plan_price = RwSignal::new(String::new()); // cents as string for input
    let edit_plan_interval = RwSignal::new(ProductPlanBillingInterval::Month);
    let edit_plan_features = RwSignal::new(String::new());
    let edit_plan_cta_label = RwSignal::new(String::new());
    let edit_plan_cta_href = RwSignal::new(String::new());
    let edit_plan_featured = RwSignal::new(false);
    let edit_plan_active = RwSignal::new(true);
    let edit_plan_sort_order = RwSignal::new(String::new());
    let show_plan_modal = RwSignal::new(false);

    let open_edit_plan = move |plan: &ProductPlanModel| {
        edit_plan_id.set(Some(plan.id));
        edit_plan_slug.set(plan.slug.clone());
        edit_plan_name.set(plan.name.clone());
        edit_plan_tagline.set(plan.tagline.clone());
        edit_plan_price.set(plan.price_cents.to_string());
        edit_plan_interval.set(plan.billing_interval.clone());
        edit_plan_features.set(plan.features.join("\n"));
        edit_plan_cta_label.set(plan.cta_label.clone());
        edit_plan_cta_href.set(plan.cta_href.clone().unwrap_or_default());
        edit_plan_featured.set(plan.is_featured);
        edit_plan_active.set(plan.is_active);
        edit_plan_sort_order.set(plan.sort_order.to_string());
        show_plan_modal.set(true);
    };

    let open_create_plan = move |_| {
        edit_plan_id.set(None);
        edit_plan_slug.set(String::new());
        edit_plan_name.set(String::new());
        edit_plan_tagline.set(String::new());
        edit_plan_price.set(String::new());
        edit_plan_interval.set(ProductPlanBillingInterval::Month);
        edit_plan_features.set(String::new());
        edit_plan_cta_label.set("Get started".to_string());
        edit_plan_cta_href.set(String::new());
        edit_plan_featured.set(false);
        edit_plan_active.set(true);
        edit_plan_sort_order.set(String::new());
        show_plan_modal.set(true);
    };

    let toast_plan = toast.clone();
    let handle_save_plan = move |_| {
        let Some(product_id) = product_uuid() else {
            toast_plan.show_toast("Error", "Invalid product ID.", "error");
            return;
        };
        let slug = edit_plan_slug.get_untracked();
        let name = edit_plan_name.get_untracked();
        let tagline = edit_plan_tagline.get_untracked();
        let price_str = edit_plan_price.get_untracked();
        let interval = edit_plan_interval.get_untracked();
        let features_text = edit_plan_features.get_untracked();
        let cta_label = edit_plan_cta_label.get_untracked();
        let cta_href = edit_plan_cta_href.get_untracked();
        let sort_order_str = edit_plan_sort_order.get_untracked();
        let id_opt = edit_plan_id.get_untracked();
        let is_featured = edit_plan_featured.get_untracked();
        let is_active = edit_plan_active.get_untracked();
        let toast_c = toast_plan.clone();

        if slug.trim().is_empty() {
            toast_c.show_toast("Error", "Plan slug is required.", "error");
            return;
        }
        if name.trim().is_empty() {
            toast_c.show_toast("Error", "Plan name is required.", "error");
            return;
        }
        let price_cents: i32 = price_str.trim().parse().unwrap_or(0);
        let sort_order: i32 = sort_order_str.trim().parse().unwrap_or(0);
        let features = features_text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        let input = ProductPlanInput {
            slug,
            name,
            tagline: Some(tagline),
            price_cents: Some(price_cents),
            currency: Some("USD".to_string()),
            billing_interval: Some(interval),
            features,
            cta_label: Some(cta_label).filter(|s| !s.trim().is_empty()),
            cta_href: Some(cta_href).filter(|s| !s.trim().is_empty()),
            is_featured: Some(is_featured),
            sort_order: Some(sort_order),
            is_active: Some(is_active),
            billing_plan_id: None,
        };

        leptos::task::spawn_local(async move {
            let result = if let Some(plan_id) = id_opt {
                update_product_plan(product_id, plan_id, input).await
            } else {
                create_product_plan(product_id, input).await
            };
            match result {
                Ok(_) => {
                    show_plan_modal.set(false);
                    plans_trigger.update(|n| *n += 1);
                    toast_c.show_toast("Saved", "Product plan updated.", "success");
                }
                Err(e) => toast_c.show_toast("Error", &e, "error"),
            }
        });
    };

    let toast_del = toast.clone();
    let handle_delete_plan = move |plan_id: Uuid| {
        let Some(product_id) = product_uuid() else {
            toast_del.show_toast("Error", "Invalid product ID.", "error");
            return;
        };
        let toast_c = toast_del.clone();
        leptos::task::spawn_local(async move {
            match delete_product_plan(product_id, plan_id).await {
                Ok(_) => {
                    show_plan_modal.set(false);
                    plans_trigger.update(|n| *n += 1);
                    toast_c.show_toast("Deleted", "Plan removed.", "success");
                }
                Err(e) => toast_c.show_toast("Error", &e, "error"),
            }
        });
    };

    // ── Pixel / domain alias inline form state ────────────────────────────────
    let show_pixel_form = RwSignal::new(false);
    let pixel_name_input = RwSignal::new(String::new());
    let pixel_url_input = RwSignal::new(String::new());
    let show_domain_form = RwSignal::new(false);
    let domain_alias_input = RwSignal::new(String::new());
    let toast_px = toast.clone();
    let handle_add_pixel = move |_: web_sys::MouseEvent| {
        let name = pixel_name_input.get_untracked();
        let url = pixel_url_input.get_untracked();
        if name.trim().is_empty() || url.trim().is_empty() {
            toast_px.show_toast("Error", "Pixel name and script URL are required.", "error");
            return;
        }
        // API: POST /api/admin/platform/products/{id}/pixels (future endpoint)
        // For now we save the intent via toast + reset form
        toast_px.show_toast(
            "Queued",
            &format!("Pixel '{}' queued for API wiring.", name),
            "info",
        );
        pixel_name_input.set(String::new());
        pixel_url_input.set(String::new());
        show_pixel_form.set(false);
    };
    let toast_da = toast.clone();
    let handle_add_domain = move |_: web_sys::MouseEvent| {
        let alias = domain_alias_input.get_untracked();
        if alias.trim().is_empty() {
            toast_da.show_toast("Error", "Domain alias is required.", "error");
            return;
        }
        toast_da.show_toast(
            "Queued",
            &format!("Domain '{}' queued for API wiring.", alias),
            "info",
        );
        domain_alias_input.set(String::new());
        show_domain_form.set(false);
    };

    // ── SEO score derived from completeness of real fields ─────────────────
    let seo_score = Signal::derive(move || {
        let mut score = 40i32; // base
        if !product_name.get().is_empty() {
            score += 20;
        }
        if !product_tagline.get().is_empty() {
            score += 20;
        }
        if !product_domain.get().is_empty() {
            score += 20;
        }
        score.min(100)
    });

    // ── Save handler → PATCH /api/admin/platform/products/:id ─────────────
    let saving = RwSignal::new(false);
    let saving_hero = RwSignal::new(false);
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

    let toast_hero = toast.clone();
    let handle_save_hero = move |_| {
        let Some(id) = product_uuid() else {
            toast_hero.show_toast("Error", "Invalid product ID", "error");
            return;
        };
        saving_hero.set(true);
        let hero_payload = match build_hero_payload(
            hero_eyebrow.get(),
            hero_headline.get(),
            hero_headline_accent.get(),
            hero_subhead.get(),
            hero_proof_items.get(),
            hero_pricing_eyebrow.get(),
            hero_pricing_heading.get(),
            hero_pricing_subtitle.get(),
            hero_spot_inventory.get(),
        ) {
            Ok(payload) => payload,
            Err(msg) => {
                toast_hero.show_toast("Invalid Spot Inventory", &msg, "error");
                saving_hero.set(false);
                return;
            }
        };
        let cta_label = hero_cta_label.get();
        let toast_c = toast_hero.clone();
        leptos::task::spawn_local(async move {
            match upsert_template(
                id,
                UpsertProductTemplateBody {
                    hero_payload: Some(hero_payload),
                    cta_label: Some(cta_label),
                    ..Default::default()
                },
            )
            .await
            {
                Ok(template) => {
                    hero_cta_label.set(template.cta_label);
                    template_res.refetch();
                    toast_c.show_toast("Saved", "Marketing hero updated.", "success");
                }
                Err(e) => toast_c.show_toast("Save Failed", &e, "error"),
            }
            saving_hero.set(false);
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
        <div class="main-canvas">

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
                    <div class="flex items-center gap-2 shrink-0">
                        <button
                            class=move || format!(
                                "btn btn-ghost {}",
                                if saving.get() { "opacity-40 cursor-not-allowed" } else { "" }
                            )
                            disabled=move || saving.get()
                            on:click=handle_save
                        >
                            {move || if saving.get() { "Saving…" } else { "Save Changes" }}
                        </button>
                        <button
                            class=move || format!(
                                "btn btn-primary {}",
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
                <div class="tab-bar">
                    {
                        let tab_btn = move |id: &str, label: &str| {
                            let id = id.to_string();
                            let label = label.to_string();
                            let id_class = id.clone();
                            let id_click = id.clone();
                            view! {
                                <button
                                    class=move || if active_tab.get() == id_class {
                                        "tab active"
                                    } else {
                                        "tab"
                                    }
                                    on:click=move |_| active_tab.set(id_click.clone())
                                >
                                    {label.clone()}
                                </button>
                            }
                        };
                        view! {
                            {tab_btn("general", "General Info")}
                            {tab_btn("marketing", "Marketing Hero")}
                            {tab_btn("pricing", "Pricing & Plans")}
                            {tab_btn("variants", "Market & SEO")}
                            {tab_btn("pixels", "Pixels")}
                            {tab_btn("domains", "Domains")}
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

                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-3">
                                <div class="flex items-center justify-between gap-4">
                                    <div>
                                        <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Acquisition Pages"</h3>
                                        <p class="text-xs text-on-surface-variant/70 mt-1">
                                            "Landing pages control the visitor-facing copy, campaign paths, and A/B tests for this product."
                                        </p>
                                    </div>
                                    <a href="/landing-pages" class="btn btn-primary btn-sm" style="text-decoration:none">
                                        "Open Landing Pages →"
                                    </a>
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

                // ── TAB CONTENT: Marketing Hero ──
                <Show when=move || active_tab.get() == "marketing">
                    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                        <div class="lg:col-span-2 space-y-6">
                            <div class="bg-primary/10 border border-primary/20 rounded-xl p-4 text-xs text-primary">
                                "These fields appear on the Folio public page for this product. Empty fields fall back to Folio defaults."
                            </div>
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-5">
                                <div class="flex items-center justify-between gap-4">
                                    <div>
                                        <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Marketing Hero"</h3>
                                        <p class="text-xs text-on-surface-variant/70 mt-1">
                                            "Controls product-level copy above the fold and the pricing intro on the public page."
                                        </p>
                                    </div>
                                    <button
                                        class=move || format!(
                                            "btn btn-primary {}",
                                            if saving_hero.get() { "opacity-40 cursor-not-allowed" } else { "" }
                                        )
                                        disabled=move || saving_hero.get()
                                        on:click=handle_save_hero
                                    >
                                        {move || if saving_hero.get() { "Saving…" } else { "Save Hero" }}
                                    </button>
                                </div>

                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <div class="space-y-1">
                                        <label class="text-xs font-semibold text-on-surface-variant">"Eyebrow"</label>
                                        <input type="text" class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5" prop:value=hero_eyebrow on:input=move |ev| hero_eyebrow.set(event_target_value(&ev)) placeholder="e.g. Rental operations, simplified" />
                                    </div>
                                    <div class="space-y-1">
                                        <label class="text-xs font-semibold text-on-surface-variant">"CTA Label"</label>
                                        <input type="text" class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5" prop:value=hero_cta_label on:input=move |ev| hero_cta_label.set(event_target_value(&ev)) placeholder="Join the Waitlist" />
                                    </div>
                                </div>

                                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                    <div class="space-y-1">
                                        <label class="text-xs font-semibold text-on-surface-variant">"Headline"</label>
                                        <input type="text" class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5" prop:value=hero_headline on:input=move |ev| hero_headline.set(event_target_value(&ev)) placeholder="Run your rental business" />
                                    </div>
                                    <div class="space-y-1">
                                        <label class="text-xs font-semibold text-on-surface-variant">"Headline Accent"</label>
                                        <input type="text" class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5" prop:value=hero_headline_accent on:input=move |ev| hero_headline_accent.set(event_target_value(&ev)) placeholder="without the busywork" />
                                    </div>
                                </div>

                                <div class="space-y-1">
                                    <label class="text-xs font-semibold text-on-surface-variant">"Subhead"</label>
                                    <textarea class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 h-24 resize-none" prop:value=hero_subhead on:input=move |ev| hero_subhead.set(event_target_value(&ev)) placeholder="Short supporting paragraph for the public hero."></textarea>
                                </div>

                                <div class="space-y-1">
                                    <label class="text-xs font-semibold text-on-surface-variant">"Proof Items"</label>
                                    <textarea class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 h-28 resize-none" prop:value=hero_proof_items on:input=move |ev| hero_proof_items.set(event_target_value(&ev)) placeholder="One proof point per line"></textarea>
                                    <p class="text-[10px] text-on-surface-variant/50">"One proof point per line. Empty lines are ignored."</p>
                                </div>

                                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                    <div class="space-y-1">
                                        <label class="text-xs font-semibold text-on-surface-variant">"Pricing Eyebrow"</label>
                                        <input type="text" class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5" prop:value=hero_pricing_eyebrow on:input=move |ev| hero_pricing_eyebrow.set(event_target_value(&ev)) placeholder="Simple pricing" />
                                    </div>
                                    <div class="space-y-1">
                                        <label class="text-xs font-semibold text-on-surface-variant">"Pricing Heading"</label>
                                        <input type="text" class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5" prop:value=hero_pricing_heading on:input=move |ev| hero_pricing_heading.set(event_target_value(&ev)) placeholder="Choose your plan" />
                                    </div>
                                    <div class="space-y-1">
                                        <label class="text-xs font-semibold text-on-surface-variant">"Pricing Subtitle"</label>
                                        <input type="text" class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5" prop:value=hero_pricing_subtitle on:input=move |ev| hero_pricing_subtitle.set(event_target_value(&ev)) placeholder="Start small and grow." />
                                    </div>
                                </div>

                                <Show when=move || product_slug.get() == "folio-founding">
                                    <div class="space-y-1 border-t border-outline-variant/20 pt-5">
                                        <label class="text-xs font-semibold text-on-surface-variant">"Founding Spot Inventory (JSON)"</label>
                                        <textarea
                                            class="w-full bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2.5 h-48 font-mono resize-y"
                                            prop:value=hero_spot_inventory
                                            on:input=move |ev| hero_spot_inventory.set(event_target_value(&ev))
                                            placeholder=r#"{ "ll-grow": { "total": 500, "taken": 47 } }"#
                                        ></textarea>
                                        <p class="text-[10px] text-on-surface-variant/50">
                                            {format!(
                                                "Keys: {}. Each value needs total and taken integers.",
                                                founding_spot_tier_keys()
                                            )}
                                        </p>
                                    </div>
                                </Show>
                            </div>
                        </div>

                        <div class="space-y-6">
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm space-y-3">
                                <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Public Path"</h3>
                                <p class="text-xs text-on-surface-variant/70">
                                    "Slug "
                                    <code class="text-primary">{move || product_slug.get()}</code>
                                    " maps to "
                                    <code class="text-primary">{move || folio_public_path_hint(&product_slug.get()).to_string()}</code>
                                    "."
                                </p>
                                <p class="text-[10px] text-on-surface-variant/50">
                                    "Known Folio paths: folio → /, folio-broker → /brokers, folio-pm → /property-managers, folio-vendor → /vendors, folio-founding → /founding, folio-beta → /beta."
                                </p>
                            </div>
                        </div>
                    </div>
                </Show>

                // ── TAB CONTENT: Pricing ──
                <Show when=move || active_tab.get() == "pricing">
                    // Plan Edit/Create Modal
                    <Show when=move || show_plan_modal.get()>
                        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-2xl shadow-2xl w-full max-w-md p-6 space-y-4">
                                <div class="flex items-center justify-between">
                                    <h3 class="text-sm font-bold">
                                        {move || if edit_plan_id.get().is_some() { "Edit Product Plan" } else { "Create Product Plan" }}
                                    </h3>
                                    <button class="btn btn-ghost btn-icon" on:click=move |_| show_plan_modal.set(false)>
                                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><line x1="3" y1="3" x2="13" y2="13"/><line x1="13" y1="3" x2="3" y2="13"/></svg>
                                    </button>
                                </div>
                                <div class="space-y-3 max-h-[70vh] overflow-y-auto pr-1">
                                    <div class="grid grid-cols-2 gap-3">
                                        <div>
                                            <label class="text-xs font-semibold text-on-surface-variant">"Plan Slug"</label>
                                            <input
                                                type="text"
                                                class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                                prop:value=move || edit_plan_slug.get()
                                                on:input=move |ev| edit_plan_slug.set(event_target_value(&ev))
                                                placeholder="e.g. pro"
                                            />
                                        </div>
                                        <div>
                                            <label class="text-xs font-semibold text-on-surface-variant">"Plan Name"</label>
                                            <input
                                                type="text"
                                                class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                                prop:value=move || edit_plan_name.get()
                                                on:input=move |ev| edit_plan_name.set(event_target_value(&ev))
                                                placeholder="e.g. Pro"
                                            />
                                        </div>
                                    </div>
                                    <div>
                                        <label class="text-xs font-semibold text-on-surface-variant">"Tagline"</label>
                                        <input
                                            type="text"
                                            class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                            prop:value=move || edit_plan_tagline.get()
                                            on:input=move |ev| edit_plan_tagline.set(event_target_value(&ev))
                                            placeholder="e.g. Up to 30 units"
                                        />
                                    </div>
                                    <div class="grid grid-cols-3 gap-3">
                                        <div>
                                            <label class="text-xs font-semibold text-on-surface-variant">"Price (cents)"</label>
                                            <input
                                                type="number"
                                                class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                                prop:value=move || edit_plan_price.get()
                                                on:input=move |ev| edit_plan_price.set(event_target_value(&ev))
                                                placeholder="e.g. 7900"
                                            />
                                            <p class="text-[10px] text-on-surface-variant/50 mt-0.5">"7900 = $79.00"</p>
                                        </div>
                                        <div>
                                            <label class="text-xs font-semibold text-on-surface-variant">"Interval"</label>
                                            <select
                                                class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                                on:change=move |ev| {
                                                    let interval = match event_target_value(&ev).as_str() {
                                                        "year" => ProductPlanBillingInterval::Year,
                                                        "forever" => ProductPlanBillingInterval::Forever,
                                                        "custom" => ProductPlanBillingInterval::Custom,
                                                        _ => ProductPlanBillingInterval::Month,
                                                    };
                                                    edit_plan_interval.set(interval);
                                                }
                                            >
                                                <option value="month" prop:selected=move || edit_plan_interval.get() == ProductPlanBillingInterval::Month>"Monthly"</option>
                                                <option value="year" prop:selected=move || edit_plan_interval.get() == ProductPlanBillingInterval::Year>"Annually"</option>
                                                <option value="forever" prop:selected=move || edit_plan_interval.get() == ProductPlanBillingInterval::Forever>"Forever"</option>
                                                <option value="custom" prop:selected=move || edit_plan_interval.get() == ProductPlanBillingInterval::Custom>"Custom"</option>
                                            </select>
                                        </div>
                                        <div>
                                            <label class="text-xs font-semibold text-on-surface-variant">"Sort Order"</label>
                                            <input
                                                type="number"
                                                class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                                prop:value=move || edit_plan_sort_order.get()
                                                on:input=move |ev| edit_plan_sort_order.set(event_target_value(&ev))
                                                placeholder="0"
                                            />
                                        </div>
                                    </div>
                                    <div>
                                        <label class="text-xs font-semibold text-on-surface-variant">"Features"</label>
                                        <textarea
                                            class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2 h-28 resize-none"
                                            prop:value=move || edit_plan_features.get()
                                            on:input=move |ev| edit_plan_features.set(event_target_value(&ev))
                                            placeholder={"One feature per line"}
                                        ></textarea>
                                    </div>
                                    <div class="grid grid-cols-2 gap-3">
                                        <div>
                                            <label class="text-xs font-semibold text-on-surface-variant">"CTA Label"</label>
                                            <input
                                                type="text"
                                                class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                                prop:value=move || edit_plan_cta_label.get()
                                                on:input=move |ev| edit_plan_cta_label.set(event_target_value(&ev))
                                                placeholder="Get started"
                                            />
                                        </div>
                                        <div>
                                            <label class="text-xs font-semibold text-on-surface-variant">"CTA Href"</label>
                                            <input
                                                type="text"
                                                class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                                prop:value=move || edit_plan_cta_href.get()
                                                on:input=move |ev| edit_plan_cta_href.set(event_target_value(&ev))
                                                placeholder="#waitlist-wrap"
                                            />
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-4">
                                        <label class="inline-flex items-center gap-2 text-xs font-semibold text-on-surface-variant">
                                            <input
                                                type="checkbox"
                                                prop:checked=move || edit_plan_featured.get()
                                                on:change=move |ev| edit_plan_featured.set(event_target_checked(&ev))
                                            />
                                            "Featured"
                                        </label>
                                        <label class="inline-flex items-center gap-2 text-xs font-semibold text-on-surface-variant">
                                            <input
                                                type="checkbox"
                                                prop:checked=move || edit_plan_active.get()
                                                on:change=move |ev| edit_plan_active.set(event_target_checked(&ev))
                                            />
                                            "Active on public API"
                                        </label>
                                    </div>
                                </div>
                                <div class="flex justify-end gap-3 pt-2">
                                    <Show when=move || edit_plan_id.get().is_some()>
                                        {{
                                            let pid = edit_plan_id.get_untracked().unwrap();
                                            view! {
                                                <button
                                                    class="btn btn-ghost"
                                                    style="color:var(--error)"
                                                    on:click=move |_| handle_delete_plan(pid.clone())
                                                >"Delete Plan"</button>
                                            }
                                        }}
                                    </Show>
                                    <button class="btn btn-ghost" on:click=move |_| show_plan_modal.set(false)>"Cancel"</button>
                                    <button class="btn btn-primary" on:click=handle_save_plan>"Save Plan"</button>
                                </div>
                            </div>
                        </div>
                    </Show>

                    <div class="space-y-4">
                        <div class="bg-primary/10 border border-primary/20 rounded-xl p-4 text-xs text-primary">
                            "These plans appear on Folio marketing pages for this product. Publishing is live via public API — no separate publish step."
                        </div>
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm">
                            <div class="flex justify-between items-center mb-6">
                                <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Pricing Plans & Feature Matrix"</h3>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    on:click=open_create_plan
                                >"+ Add Tier"</button>
                            </div>

                            {move || {
                                let plans = product_plans_res.get().unwrap_or_default();
                                if plans.is_empty() {
                                    view! {
                                        <div class="text-center py-10 text-xs text-on-surface-variant/60">
                                            <p>"No product plans defined. Click '+ Add Tier' to create the first plan."</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                            {plans.into_iter().map(|plan| {
                                                let plan_clone = plan.clone();
                                                let is_featured = plan.is_featured;
                                                let is_active = plan.is_active;
                                                let plan_name = plan.name.clone();
                                                let plan_slug = plan.slug.clone();
                                                let plan_tagline = plan.tagline.clone();
                                                let features = plan.features.clone();
                                                let price_display = if plan.billing_interval == ProductPlanBillingInterval::Custom {
                                                    "Custom".to_string()
                                                } else if plan.price_cents == 0 {
                                                    "$0".to_string()
                                                } else {
                                                    format!("${:.2}/{}", plan.price_cents as f64 / 100.0, plan.billing_interval.short_label())
                                                };
                                                view! {
                                                    <div class=move || if is_featured {
                                                        "bg-surface-container p-5 rounded-xl border border-primary/50 flex flex-col justify-between shadow-sm"
                                                    } else {
                                                        "bg-surface-container p-5 rounded-xl border border-outline-variant/20 flex flex-col justify-between"
                                                    }>
                                                        <div>
                                                            <div class="flex items-center justify-between mb-2">
                                                                <div>
                                                                    <h4 class="font-bold text-on-surface">{plan_name}</h4>
                                                                    <code class="text-[10px] text-on-surface-variant/60">{plan_slug}</code>
                                                                </div>
                                                                <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-primary/10 text-primary border border-primary/20">{price_display}</span>
                                                            </div>
                                                            <p class="text-xs text-on-surface-variant/70 mb-4">{plan_tagline}</p>
                                                            <ul class="space-y-1">
                                                                {features.iter().take(4).cloned().map(|feature| view! {
                                                                    <li class="text-[11px] text-on-surface-variant/80 flex gap-1">
                                                                        <span>"•"</span>
                                                                        <span>{feature}</span>
                                                                    </li>
                                                                }).collect_view()}
                                                            </ul>
                                                            <div class="flex gap-2 mt-4">
                                                                <Show when=move || is_featured>
                                                                    <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-primary/10 text-primary border border-primary/20">"Featured"</span>
                                                                </Show>
                                                                <span class=if is_active {
                                                                    "px-2 py-0.5 rounded text-[9px] font-bold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
                                                                } else {
                                                                    "px-2 py-0.5 rounded text-[9px] font-bold bg-outline-variant/10 text-on-surface-variant border border-outline-variant/20"
                                                                }>
                                                                    {if is_active { "Active" } else { "Inactive" }}
                                                                </span>
                                                            </div>
                                                        </div>
                                                        <button
                                                            class="btn btn-ghost btn-sm w-full mt-4 justify-center"
                                                            on:click=move |_| open_edit_plan(&plan_clone)
                                                        >"Edit plan"</button>
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    }.into_any()
                                }
                            }}
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
                        <Suspense fallback=move || view! {
                            <div class="p-8 text-center text-xs text-on-surface-variant/60">"Loading waitlist…"</div>
                        }>
                        {move || {
                            let leads = waitlist_leads_res.get().unwrap_or_default();
                            if leads.is_empty() {
                                view! {
                                    <div class="p-8 text-center text-xs text-on-surface-variant/60 flex flex-col items-center gap-3">
                                        <span class="material-symbols-outlined text-[32px] text-on-surface-variant/30">"hourglass_empty"</span>
                                        <p>"No waitlist signups yet. Share the product page to start collecting leads."</p>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="overflow-x-auto">
                                        <table class="w-full text-left text-sm">
                                            <thead class="bg-surface-container-highest text-on-surface-variant uppercase text-xs tracking-wider">
                                                <tr>
                                                    <th class="px-4 py-3 font-medium rounded-tl-lg">"Name"</th>
                                                    <th class="px-4 py-3 font-medium">"Email"</th>
                                                    <th class="px-4 py-3 font-medium">"Status"</th>
                                                    <th class="px-4 py-3 font-medium rounded-tr-lg">"Signed Up"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-outline-variant/10">
                                                <For
                                                    each=move || leads.clone()
                                                    key=|l| l.id.clone()
                                                    children=move |lead| view! {
                                                        <tr class="hover:bg-surface-bright/5">
                                                            <td class="px-4 py-3 font-medium text-on-surface">{lead.name}</td>
                                                            <td class="px-4 py-3 text-on-surface-variant font-mono text-xs">
                                                                {lead.email.unwrap_or_else(|| "—".to_string())}
                                                            </td>
                                                            <td class="px-4 py-3">
                                                                <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold bg-primary/10 text-primary">
                                                                    {lead.lead_status.unwrap_or_else(|| "New".to_string())}
                                                                </span>
                                                            </td>
                                                            <td class="px-4 py-3 text-xs text-on-surface-variant">
                                                                {lead.created_at.unwrap_or_else(|| "—".to_string())}
                                                            </td>
                                                        </tr>
                                                    }
                                                />
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                        }}
                        </Suspense>
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

                // ── TAB CONTENT: Market & SEO (GTM Launcher) ──
                <Show when=move || active_tab.get() == "variants">
                    <div class="space-y-4">
                        // Header
                        <div class="flex items-center justify-between">
                            <div>
                                <h3 class="text-sm font-bold text-on-surface">"Market & SEO Variants"</h3>
                                <p class="text-xs text-on-surface-variant/70 mt-0.5">
                                    "Each market record targets a specific city, region, locale, or niche for SEO and launch-mode planning."
                                </p>
                            </div>
                            <button
                                class="btn btn-ghost btn-sm opacity-60 cursor-not-allowed"
                                id="btn-new-variant-disabled"
                                disabled=true
                                title="Market creation is not wired yet"
                            >
                                <span class="material-symbols-outlined text-[14px]">"add"</span>
                                "New Market"
                            </button>
                        </div>

                        // Variants table
                        <Suspense fallback=|| view! {
                            <div class="p-8 text-center text-xs text-on-surface-variant/60">"Loading variants…"</div>
                        }>
                        {move || {
                            let variants = variants_res.get().unwrap_or_default();
                            if variants.is_empty() {
                                view! {
                                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-12 flex flex-col items-center gap-3 text-center">
                                        <span class="material-symbols-outlined text-[36px] text-on-surface-variant/30">"travel_explore"</span>
                                        <p class="text-sm font-semibold text-on-surface-variant">"No market records yet"</p>
                                        <p class="text-xs text-on-surface-variant/60 max-w-xs">
                                            "Market creation is not wired in this screen yet. Use seed migrations or the bulk generation API, then manage acquisition copy in Landing Pages."
                                        </p>
                                        <a href="/landing-pages" class="btn btn-ghost btn-sm" style="text-decoration:none">
                                            "Open Acquisition Pages →"
                                        </a>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden shadow-sm">
                                        <table class="w-full text-left text-sm">
                                            <thead>
                                                <tr class="border-b border-outline-variant/10 bg-surface-container">
                                                    <th class="px-4 py-3 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Slug"</th>
                                                    <th class="px-4 py-3 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Market"</th>
                                                    <th class="px-4 py-3 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Launch Mode"</th>
                                                    <th class="px-4 py-3 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Status"</th>
                                                    <th class="px-4 py-3 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Leads"</th>
                                                    <th class="px-4 py-3 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Views"</th>
                                                    <th class="px-4 py-3 text-[10px] font-bold uppercase tracking-wider text-on-surface-variant/60">"Actions"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-outline-variant/10">
                                                <For
                                                    each=move || variants_res.get().unwrap_or_default()
                                                    key=|v| v.id
                                                    children=move |v| {
                                                        let preview_url = format!(
                                                            "/api/pub/products/{}/{}",
                                                            product_slug.get(),
                                                            v.variant_slug
                                                        );
                                                        let market = match (&v.city, &v.region) {
                                                            (Some(c), Some(r)) => format!("{}, {}", c, r),
                                                            (Some(c), None)    => c.clone(),
                                                            (None, Some(r))    => r.clone(),
                                                            _                  => v.locale.clone(),
                                                        };
                                                        // — Use typed enum methods; compiler enforces exhaustive coverage —
                                                        let launch_mode_class = v.launch_mode.badge_class();
                                                        let launch_mode_label = v.launch_mode.label();
                                                        let loc_badge = v.localization_status
                                                            .badge_label()
                                                            .zip(v.localization_status.badge_class());
                                                        view! {
                                                            <tr class="hover:bg-surface-container-high/30 transition-colors">
                                                                <td class="px-4 py-3">
                                                                    <code class="text-xs font-mono text-primary/80">
                                                                        {v.variant_slug.clone()}
                                                                    </code>
                                                                    {loc_badge.map(|(label, cls)| view! {
                                                                        <span class=format!("ml-2 inline-flex items-center px-1.5 py-0.5 rounded text-[8px] font-bold border {}", cls)>
                                                                            {label}
                                                                        </span>
                                                                    })}
                                                                </td>
                                                                <td class="px-4 py-3 text-xs text-on-surface-variant">{market}</td>
                                                                <td class="px-4 py-3">
                                                                    <span class=launch_mode_class>
                                                                        {launch_mode_label}
                                                                    </span>
                                                                </td>
                                                                <td class="px-4 py-3">
                                                                    <span class=if v.is_published {
                                                                        "inline-flex items-center gap-1 text-[9px] font-bold text-emerald-400"
                                                                    } else {
                                                                        "inline-flex items-center gap-1 text-[9px] font-bold text-on-surface-variant/50"
                                                                    }>
                                                                        {if v.is_published { "● Live" } else { "○ Draft" }}
                                                                    </span>
                                                                </td>
                                                                <td class="px-4 py-3 text-xs font-mono font-bold text-on-surface">
                                                                    {v.lead_count}
                                                                </td>
                                                                <td class="px-4 py-3 text-xs font-mono text-on-surface-variant">
                                                                    {v.view_count}
                                                                </td>
                                                                <td class="px-4 py-3">
                                                                    <div class="flex items-center gap-2">
                                                                        <a
                                                                            href=preview_url
                                                                            target="_blank"
                                                                            class="btn btn-ghost btn-sm"
                                                                            title="Preview page data"
                                                                        >
                                                                            <span class="material-symbols-outlined text-[11px]">"open_in_new"</span>
                                                                            "Preview"
                                                                        </a>
                                                                        <button
                                                                            class="btn btn-ghost btn-sm"
                                                                            title="Duplicate variant"
                                                                        >
                                                                            "Duplicate"
                                                                        </button>
                                                                    </div>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }
                                                />
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                        }}
                        </Suspense>

                        // Bulk ops bar
                        <div class="flex items-center gap-3 pt-2">
                            <span class="text-xs text-on-surface-variant/60">
                                {move || format!("{} variant(s)", variants_res.get().unwrap_or_default().len())}
                            </span>
                            <div class="ml-auto flex gap-2">
                                <button class="btn btn-ghost btn-sm">
                                    "Bulk Localize (AI)"
                                </button>
                                <button class="btn btn-ghost btn-sm">
                                    "Bulk Publish"
                                </button>
                            </div>
                        </div>
                    </div>
                </Show>

                // ── TAB CONTENT: Pixels ──
                <Show when=move || active_tab.get() == "pixels">
                    <div class="space-y-4">
                        <div class="flex items-center justify-between">
                            <div>
                                <h3 class="text-sm font-bold text-on-surface">"Tracking Pixels"</h3>
                                <p class="text-xs text-on-surface-variant/70 mt-0.5">
                                    "Snippets injected into every landing page for this product at serve time."
                                </p>
                            </div>
                            <button
                                class="btn btn-primary btn-sm"
                                id="btn-add-pixel"
                                on:click=move |_: web_sys::MouseEvent| show_pixel_form.update(|v| *v = !*v)
                            >
                                <span class="material-symbols-outlined text-[14px]">"add"</span>
                                "Add Pixel"
                            </button>
                        </div>

                        // Inline Add Pixel Form
                        <Show when=move || show_pixel_form.get()>
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm space-y-3">
                                <h4 class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"New Tracking Pixel"</h4>
                                <div class="grid grid-cols-2 gap-3">
                                    <div>
                                        <label class="text-xs font-semibold text-on-surface-variant">"Pixel Name"</label>
                                        <input
                                            type="text" placeholder="e.g. GA4 Main"
                                            class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                            prop:value=move || pixel_name_input.get()
                                            on:input=move |ev| pixel_name_input.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div>
                                        <label class="text-xs font-semibold text-on-surface-variant">"Script / Measurement ID"</label>
                                        <input
                                            type="text" placeholder="e.g. G-XXXXXXXX"
                                            class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                            prop:value=move || pixel_url_input.get()
                                            on:input=move |ev| pixel_url_input.set(event_target_value(&ev))
                                        />
                                    </div>
                                </div>
                                <div class="flex justify-end gap-2">
                                    <button class="btn btn-ghost btn-sm" on:click=move |_: web_sys::MouseEvent| show_pixel_form.set(false)>"Cancel"</button>
                                    <button class="btn btn-primary btn-sm" on:click=handle_add_pixel>"Add Pixel"</button>
                                </div>
                            </div>
                        </Show>

                        // Pixel types guide
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm">
                            <p class="text-xs font-bold text-on-surface-variant uppercase tracking-wider mb-4">"Configured Pixels"</p>
                            <div class="space-y-3">
                                // Empty state — pixels come from product_tracking_pixels table
                                <div class="flex flex-col items-center gap-3 py-8 text-center">
                                    <span class="material-symbols-outlined text-[32px] text-on-surface-variant/30">"track_changes"</span>
                                    <p class="text-xs text-on-surface-variant/60 max-w-xs">
                                        "No tracking pixels configured. Add GA4, Meta Pixel, GTM, or LinkedIn tags — "
                                        "they'll be injected into every landing page for this product."
                                    </p>
                                </div>
                            </div>
                        </div>

                        // Pixel type reference
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm">
                            <p class="text-xs font-bold text-on-surface-variant uppercase tracking-wider mb-4">"Supported Pixel Types"</p>
                            <div class="grid grid-cols-2 md:grid-cols-3 gap-3">
                                {["GA4", "GTM", "Meta", "LinkedIn", "TikTok", "Custom"].into_iter().map(|t| view! {
                                    <div class="flex items-center gap-2 p-3 rounded-lg border border-outline-variant/20 bg-surface-container">
                                        <span class="material-symbols-outlined text-[16px] text-on-surface-variant/60">"code"</span>
                                        <span class="text-xs font-semibold text-on-surface">{t}</span>
                                    </div>
                                }).collect_view()}
                            </div>
                        </div>
                    </div>
                </Show>

                // ── TAB CONTENT: Domains ──
                <Show when=move || active_tab.get() == "domains">
                    <div class="space-y-4">
                        <div class="flex items-center justify-between">
                            <div>
                                <h3 class="text-sm font-bold text-on-surface">"Domain Aliases"</h3>
                                <p class="text-xs text-on-surface-variant/70 mt-0.5">
                                    "Custom domains or subdomains that route to this product's landing pages."
                                </p>
                            </div>
                            <button
                                class="btn btn-primary btn-sm"
                                id="btn-add-domain"
                                on:click=move |_: web_sys::MouseEvent| show_domain_form.update(|v| *v = !*v)
                            >
                                <span class="material-symbols-outlined text-[14px]">"add"</span>
                                "Add Domain"
                            </button>
                        </div>

                        // Inline Add Domain Form
                        <Show when=move || show_domain_form.get()>
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm space-y-3">
                                <h4 class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"New Domain Alias"</h4>
                                <div>
                                    <label class="text-xs font-semibold text-on-surface-variant">"Domain"</label>
                                    <input
                                        type="text" placeholder="e.g. mycompany.com or app.mycompany.com"
                                        class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                        prop:value=move || domain_alias_input.get()
                                        on:input=move |ev| domain_alias_input.set(event_target_value(&ev))
                                    />
                                    <p class="text-[10px] text-on-surface-variant/50 mt-0.5">"Must be DNS-pointed to the Atlas platform edge before activating."</p>
                                </div>
                                <div class="flex justify-end gap-2">
                                    <button class="btn btn-ghost btn-sm" on:click=move |_: web_sys::MouseEvent| show_domain_form.set(false)>"Cancel"</button>
                                    <button class="btn btn-primary btn-sm" on:click=handle_add_domain>"Add Domain"</button>
                                </div>
                            </div>
                        </Show>

                        // Apex domain card
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm">
                            <p class="text-xs font-bold text-on-surface-variant uppercase tracking-wider mb-3">"Apex Domain"</p>
                            <div class="flex items-center gap-3">
                                <span class="material-symbols-outlined text-emerald-400 text-[18px]">"language"</span>
                                <div>
                                    <p class="text-sm font-mono font-semibold text-on-surface">
                                        {move || if product_domain.get().is_empty() {
                                            "Not configured".to_string()
                                        } else {
                                            product_domain.get()
                                        }}
                                    </p>
                                    <p class="text-[10px] text-on-surface-variant/60 mt-0.5">
                                        "Set in General Info. All subdomain variants resolve under this apex."
                                    </p>
                                </div>
                            </div>
                        </div>

                        // How domain routing works
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-5 shadow-sm space-y-3">
                            <p class="text-xs font-bold text-on-surface-variant uppercase tracking-wider">"How Domain Resolution Works"</p>
                            {[
                                ("Subdomain", "miami.folio.app → variant with subdomain_override=\"miami\""),
                                ("Path", "folio.app/miami → variant with variant_slug=\"miami\""),
                                ("Custom domain", "yourdomain.com → matched via product_domain_aliases table"),
                                ("Apex fallback", "folio.app → product default (master template)"),
                            ].into_iter().map(|(kind, desc)| view! {
                                <div class="flex items-start gap-3 py-3 border-t border-outline-variant/10 first:border-0 first:pt-0">
                                    <span class="text-[9px] font-bold px-1.5 py-0.5 rounded bg-primary/10 text-primary border border-primary/20 shrink-0 mt-0.5 uppercase tracking-wider">{{kind}}</span>
                                    <p class="text-xs text-on-surface-variant font-mono">{desc}</p>
                                </div>
                            }).collect_view()}
                        </div>
                    </div>
                </Show>

            </Show>
        </div>
    }
}

fn hero_string(payload: &serde_json::Value, key: &str) -> String {
    payload
        .get(key)
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string()
}

fn hero_string_array(payload: &serde_json::Value, key: &str) -> Vec<String> {
    payload
        .get(key)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn build_hero_payload(
    eyebrow: String,
    headline: String,
    headline_accent: String,
    subhead: String,
    proof_items: String,
    pricing_eyebrow: String,
    pricing_heading: String,
    pricing_subtitle: String,
    spot_inventory_raw: String,
) -> Result<serde_json::Value, String> {
    let proof_items = proof_items
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    let mut payload = serde_json::json!({
        "eyebrow": eyebrow,
        "headline": headline,
        "headline_accent": headline_accent,
        "subhead": subhead,
        "proof_items": proof_items,
        "pricing_eyebrow": pricing_eyebrow,
        "pricing_heading": pricing_heading,
        "pricing_subtitle": pricing_subtitle,
    });

    let spot_raw = spot_inventory_raw.trim();
    if !spot_raw.is_empty() {
        let spot_inventory: serde_json::Value = serde_json::from_str(spot_raw)
            .map_err(|e| format!("spot_inventory must be valid JSON: {e}"))?;
        if !spot_inventory.is_object() {
            return Err("spot_inventory must be a JSON object of tier keys".into());
        }
        validate_spot_inventory(&spot_inventory)?;
        if let Some(obj) = payload.as_object_mut() {
            obj.insert("spot_inventory".into(), spot_inventory);
        }
    }

    Ok(payload)
}

fn hero_spot_inventory_json(hero: &serde_json::Value) -> String {
    hero.get("spot_inventory")
        .and_then(|v| serde_json::to_string_pretty(v).ok())
        .unwrap_or_default()
}

fn founding_spot_tier_keys() -> String {
    FoundingSpotTier::ALL
        .iter()
        .map(|tier| tier.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn validate_spot_inventory(spot_inventory: &serde_json::Value) -> Result<(), String> {
    let tiers = spot_inventory
        .as_object()
        .ok_or_else(|| "spot_inventory must be a JSON object of tier keys".to_string())?;

    for (tier_key, value) in tiers {
        FoundingSpotTier::try_from(tier_key.as_str())
            .map_err(|_| format!("unknown founding spot tier '{tier_key}'"))?;

        let Some(entry) = value.as_object() else {
            return Err(format!("spot_inventory.{tier_key} must be an object"));
        };
        for field in ["total", "taken"] {
            let Some(amount) = entry.get(field) else {
                return Err(format!("spot_inventory.{tier_key}.{field} is required"));
            };
            if amount.as_i64().is_none() {
                return Err(format!("spot_inventory.{tier_key}.{field} must be an integer"));
            }
        }
    }

    Ok(())
}
