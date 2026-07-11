use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;

use crate::api::products::{get_product_detail, get_variants, update_product_detail, publish_marketing};
use crate::api::models::UpdateProductBody;
use crate::api::crm::get_leads;
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

    // Waitlist leads — fetched on-demand when slug is known
    let waitlist_leads_res = LocalResource::new(move || async move {
        let slug = product_slug.get();
        if slug.is_empty() { return vec![]; }
        let prefix = format!("waitlist:{}", slug);
        get_leads(None, 1, 200, None, Some(&prefix)).await.unwrap_or_default()
    });

    // Variants — fetched when product_uuid is known
    let variants_res = LocalResource::new(move || async move {
        match product_uuid() {
            Some(id) => get_variants(id).await.unwrap_or_default(),
            None => vec![],
        }
    });

    // ── Billing plans (for the Pricing tab) ───────────────────────────────────
    let plans_trigger = RwSignal::new(0u32);
    let billing_plans_res = LocalResource::new(move || async move {
        let _ = plans_trigger.get();
        crate::api::billing::list_billing_plans().await.unwrap_or_default()
    });

    // Plan edit modal state
    let edit_plan_id   = RwSignal::new(Option::<String>::None); // Some(id) = editing, None = creating
    let edit_plan_name = RwSignal::new(String::new());
    let edit_plan_price = RwSignal::new(String::new()); // cents as string for input
    let edit_plan_interval = RwSignal::new("month".to_string());
    let show_plan_modal = RwSignal::new(false);

    let open_edit_plan = move |plan: &crate::api::billing::BillingPlanModel| {
        edit_plan_id.set(Some(plan.id.to_string()));
        edit_plan_name.set(plan.name.clone());
        edit_plan_price.set(plan.price.to_string());
        edit_plan_interval.set(plan.interval.clone());
        show_plan_modal.set(true);
    };

    let open_create_plan = move |_| {
        edit_plan_id.set(None);
        edit_plan_name.set(String::new());
        edit_plan_price.set(String::new());
        edit_plan_interval.set("month".to_string());
        show_plan_modal.set(true);
    };

    let toast_plan = toast.clone();
    let handle_save_plan = move |_| {
        let name = edit_plan_name.get_untracked();
        let price_str = edit_plan_price.get_untracked();
        let interval = edit_plan_interval.get_untracked();
        let id_opt = edit_plan_id.get_untracked();
        let toast_c = toast_plan.clone();

        if name.trim().is_empty() {
            toast_c.show_toast("Error", "Plan name is required.", "error");
            return;
        }
        let price: i64 = price_str.trim().parse().unwrap_or(0);
        let input = crate::api::billing::BillingPlanInput {
            name,
            price,
            currency: Some("usd".to_string()),
            interval,
        };

        leptos::task::spawn_local(async move {
            let result = if let Some(plan_id) = id_opt {
                crate::api::billing::update_billing_plan(&plan_id, input).await
            } else {
                crate::api::billing::create_billing_plan(input).await
            };
            match result {
                Ok(_) => {
                    show_plan_modal.set(false);
                    plans_trigger.update(|n| *n += 1);
                    toast_c.show_toast("Saved", "Billing plan updated.", "success");
                }
                Err(e) => toast_c.show_toast("Error", &e, "error"),
            }
        });
    };

    let toast_del = toast.clone();
    let handle_delete_plan = move |plan_id: String| {
        let toast_c = toast_del.clone();
        leptos::task::spawn_local(async move {
            match crate::api::billing::delete_billing_plan(&plan_id).await {
                Ok(_) => {
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
    let pixel_url_input  = RwSignal::new(String::new());
    let show_domain_form = RwSignal::new(false);
    let domain_alias_input = RwSignal::new(String::new());
    let toast_px = toast.clone();
    let handle_add_pixel = move |_: web_sys::MouseEvent| {
        let name = pixel_name_input.get_untracked();
        let url  = pixel_url_input.get_untracked();
        if name.trim().is_empty() || url.trim().is_empty() {
            toast_px.show_toast("Error", "Pixel name and script URL are required.", "error");
            return;
        }
        // API: POST /api/admin/platform/products/{id}/pixels (future endpoint)
        // For now we save the intent via toast + reset form
        toast_px.show_toast("Queued", &format!("Pixel '{}' queued for API wiring.", name), "info");
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
        toast_da.show_toast("Queued", &format!("Domain '{}' queued for API wiring.", alias), "info");
        domain_alias_input.set(String::new());
        show_domain_form.set(false);
    };

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
                            {tab_btn("pricing", "Pricing & Plans")}
                            {tab_btn("variants", "Variants")}
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
                    // Plan Edit/Create Modal
                    <Show when=move || show_plan_modal.get()>
                        <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-2xl shadow-2xl w-full max-w-md p-6 space-y-4">
                                <div class="flex items-center justify-between">
                                    <h3 class="text-sm font-bold">
                                        {move || if edit_plan_id.get().is_some() { "Edit Billing Plan" } else { "Create Billing Plan" }}
                                    </h3>
                                    <button class="btn btn-ghost btn-icon" on:click=move |_| show_plan_modal.set(false)>
                                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><line x1="3" y1="3" x2="13" y2="13"/><line x1="13" y1="3" x2="3" y2="13"/></svg>
                                    </button>
                                </div>
                                <div class="space-y-3">
                                    <div>
                                        <label class="text-xs font-semibold text-on-surface-variant">"Plan Name"</label>
                                        <input
                                            type="text"
                                            class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                            prop:value=move || edit_plan_name.get()
                                            on:input=move |ev| edit_plan_name.set(event_target_value(&ev))
                                            placeholder="e.g. Professional"
                                        />
                                    </div>
                                    <div class="grid grid-cols-2 gap-3">
                                        <div>
                                            <label class="text-xs font-semibold text-on-surface-variant">"Price (cents)"</label>
                                            <input
                                                type="number"
                                                class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                                prop:value=move || edit_plan_price.get()
                                                on:input=move |ev| edit_plan_price.set(event_target_value(&ev))
                                                placeholder="e.g. 90000"
                                            />
                                            <p class="text-[10px] text-on-surface-variant/50 mt-0.5">"e.g. 90000 = $900.00"</p>
                                        </div>
                                        <div>
                                            <label class="text-xs font-semibold text-on-surface-variant">"Billing Interval"</label>
                                            <select
                                                class="w-full mt-1 bg-surface-container border border-outline-variant/30 text-on-surface text-sm rounded-lg px-3 py-2"
                                                on:change=move |ev| edit_plan_interval.set(event_target_value(&ev))
                                            >
                                                <option value="month" prop:selected=move || edit_plan_interval.get() == "month">"Monthly"</option>
                                                <option value="year" prop:selected=move || edit_plan_interval.get() == "year">"Annually"</option>
                                            </select>
                                        </div>
                                    </div>
                                </div>
                                <div class="flex justify-end gap-3 pt-2">
                                    <Show when=move || edit_plan_id.get().is_some()>
                                        {{
                                            let pid = edit_plan_id.get_untracked().unwrap_or_default();
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
                        <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-6 shadow-sm">
                            <div class="flex justify-between items-center mb-6">
                                <h3 class="text-sm font-bold uppercase tracking-wider text-on-surface-variant">"Pricing Plans & Feature Matrix"</h3>
                                <button
                                    class="btn btn-ghost btn-sm"
                                    on:click=open_create_plan
                                >"+ Add Tier"</button>
                            </div>

                            {move || {
                                let plans = billing_plans_res.get().unwrap_or_default();
                                if plans.is_empty() {
                                    view! {
                                        <div class="text-center py-10 text-xs text-on-surface-variant/60">
                                            <p>"No billing plans defined. Click '+ Add Tier' to create the first plan."</p>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                            {plans.into_iter().map(|plan| {
                                                let plan_clone = plan.clone();
                                                let price_display = format!("${:.2}/{}", plan.price as f64 / 100.0, if plan.interval == "year" { "yr" } else { "mo" });
                                                view! {
                                                    <div class="bg-surface-container p-5 rounded-xl border border-outline-variant/20 flex flex-col justify-between">
                                                        <div>
                                                            <div class="flex items-center justify-between mb-2">
                                                                <h4 class="font-bold text-on-surface">{plan.name.clone()}</h4>
                                                                <span class="px-2 py-0.5 rounded text-[9px] font-bold bg-primary/10 text-primary border border-primary/20">{price_display}</span>
                                                            </div>
                                                            <p class="text-xs text-on-surface-variant/70 mb-4">{plan.currency.to_uppercase()} " · " {plan.interval.clone()}</p>
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

                // ── TAB CONTENT: Variants (GTM Launcher) ──
                <Show when=move || active_tab.get() == "variants">
                    <div class="space-y-4">
                        // Header
                        <div class="flex items-center justify-between">
                            <div>
                                <h3 class="text-sm font-bold text-on-surface">"Market Variants"</h3>
                                <p class="text-xs text-on-surface-variant/70 mt-0.5">
                                    "Each variant is a landing page targeting a specific market, city, or niche."
                                </p>
                            </div>
                            <a
                                href=move || format!("/products/{}/variants/new", product_slug.get())
                                class="btn btn-primary btn-sm"
                                id="btn-new-variant"
                            >
                                <span class="material-symbols-outlined text-[14px]">"add"</span>
                                "New Variant"
                            </a>
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
                                        <p class="text-sm font-semibold text-on-surface-variant">"No variants yet"</p>
                                        <p class="text-xs text-on-surface-variant/60 max-w-xs">
                                            "Create a variant to build a market-specific landing page for a city, niche, or audience."
                                        </p>
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
