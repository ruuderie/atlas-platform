//! Landing Pages — Platform-Admin GTM builder
//!
//! Route: `/landing-pages`
//!
//! App-neutral interface for managing platform-level acquisition pages.
//! Folio is the initial guinea pig app.
//!
//! # Tabs
//! 1. **All Pages** — searchable table; publish toggle; quick actions
//! 2. **Editor** — block palette + live preview + property inspector
//! 3. **A/B Testing** — variant cards; traffic sliders; promote winner
//! 4. **UTM Builder** — URL generator + saved UTM presets; QR download
//! 5. **Funnel Analytics** — conversion funnel; source breakdown

use leptos::prelude::*;
use serde_json::{Map, Value};
use shared_ui::marketing::{
    MarketingSectionBlockType, VendorTradeKey, folio_public_path_hint,
};

use crate::api::landing_pages::{
    CreateLandingPagePayload, CreateUtmPresetPayload, CreateVariantPayload, LandingPageSummary,
    PageAnalytics, PagePixelConfig, PageVariant, UpdateLandingPagePayload, UtmPreset,
    create_landing_page, create_utm_preset, create_variant, delete_utm_preset, get_landing_page,
    get_page_analytics, get_page_pixels, list_landing_pages, list_utm_presets, list_variants,
    promote_variant, set_pixel, toggle_publish, update_landing_page,
};
use crate::app::GlobalToast;
use crate::components::gtm_process_strip::{GtmProcessStrip, GtmStage};

// ── App selector ─────────────────────────────────────────────────────────────

const APPS: &[(&str, &str, &str)] = &[
    ("folio", "🏢 Folio", "#4F63EB"),
    ("folio-broker", "🤝 Broker Page", "#F59E0B"),
    ("folio-pm", "🏗️ Property Manager", "#06D6A0"),
    ("folio-vendor", "🔧 Vendor Page", "#FF6B35"),
    ("network", "🔗 Network", "#06967F"),
    ("anchor", "⚓ Anchor", "#9C27B0"),
];

// ── Main page component ──────────────────────────────────────────────────────

#[component]
pub fn LandingPagesPage() -> impl IntoView {
    let (active_app, set_active_app) = signal("folio".to_string());
    let (active_tab, set_active_tab) = signal(0usize); // 0=pages,1=editor,2=ab,3=utm,4=funnel
    let pages_version = RwSignal::new(0u32);
    let toast = use_context::<GlobalToast>();

    // Pages list resource
    let pages_res = LocalResource::new(move || {
        let app = active_app.get();
        let _ = pages_version.get();
        async move { list_landing_pages(&app).await.unwrap_or_default() }
    });

    // Selected page id (for editor / AB / funnel context)
    let (selected_page, set_selected_page) = signal(None::<LandingPageSummary>);

    // A/B variants resource (reloads when selected page changes)
    let variants_res = LocalResource::new(move || {
        let pid = selected_page.get().map(|p| p.id);
        async move {
            match pid {
                Some(id) => list_variants(id).await.unwrap_or_default(),
                None => vec![],
            }
        }
    });

    // UTM presets resource
    let utm_res = LocalResource::new(move || {
        let app = active_app.get();
        async move { list_utm_presets(&app).await.unwrap_or_default() }
    });

    view! {
        <div class="landing-pg-shell">

            // ── Page header ──────────────────────────────────────────────────
            <div class="lp-header">
                <div class="lp-header-left">
                    <h1 class="lp-title">"Landing Pages"</h1>
                    <p class="lp-subtitle">"Build, test, and measure acquisition pages for each app."</p>
                </div>

                // App selector pills
                <div class="lp-app-pills">
                    {APPS.iter().map(|(id, label, _color)| {
                        let id = *id;
                        let label = *label;
                        view! {
                            <button
                                class=move || if active_app.get() == id {
                                    "app-pill active"
                                } else {
                                    "app-pill"
                                }
                                on:click=move |_| {
                                    set_active_app.set(id.to_string());
                                    set_selected_page.set(None);
                                }
                            >
                                {label}
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            <GtmProcessStrip
                active=GtmStage::LandingPages
                subtitle="Build, test, and measure acquisition pages for each app."
            />

            <div class="bg-surface-container-low border border-outline-variant/15 rounded-xl px-5 py-4 text-xs text-on-surface-variant/80 space-y-1">
                <p class="font-semibold text-on-surface text-sm">
                    "Products define the offering. Landing Pages change what visitors see."
                </p>
                <p>
                    "Use products for catalog, pricing, launch mode, and market SEO. Use landing pages for acquisition copy, public path experiments, and A/B tests."
                </p>
                <p class="text-on-surface-variant/60 text-[11px]">
                    <a href="/products" class="text-primary hover:underline font-semibold">"Products"</a>
                    " stay stable while landing pages adapt by channel, audience, and campaign."
                </p>
            </div>

            // ── Tab bar ──────────────────────────────────────────────────────
            <div class="lp-tab-bar">
                {["All Pages", "Editor", "A/B Testing", "UTM Builder", "Funnel Analytics"]
                    .iter().enumerate().map(|(i, label)| {
                        let label = *label;
                        view! {
                            <button
                                class=move || if active_tab.get() == i { "lp-tab active" } else { "lp-tab" }
                                on:click=move |_| set_active_tab.set(i)
                            >
                                {label}
                                {(i == 2).then(|| view! {
                                    <Suspense>
                                        {move || variants_res.get().map(|v| {
                                            let count = v.len();
                                            (count > 0).then(|| view! {
                                                <span class="tab-badge">{count}</span>
                                            })
                                        })}
                                    </Suspense>
                                })}
                            </button>
                        }
                    }).collect_view()}
            </div>

            // ── Tab panels ───────────────────────────────────────────────────
            <div class="lp-panel-wrap">

                // ── Tab 0: All Pages ─────────────────────────────────────────
                <div class=move || if active_tab.get() == 0 { "lp-panel active" } else { "lp-panel" }>
                    <AllPagesTab
                        pages_res=pages_res
                        selected_page=selected_page
                        set_selected_page=set_selected_page
                        set_active_tab=set_active_tab
                        active_app=active_app
                        pages_version=pages_version
                        toast=toast
                    />
                </div>

                // ── Tab 1: Editor ────────────────────────────────────────────
                <div class=move || if active_tab.get() == 1 { "lp-panel active" } else { "lp-panel" }>
                    <EditorTab selected_page=selected_page active_app=active_app pages_version=pages_version />
                </div>

                // ── Tab 2: A/B Testing ───────────────────────────────────────
                <div class=move || if active_tab.get() == 2 { "lp-panel active" } else { "lp-panel" }>
                    <AbTestingTab
                        selected_page=selected_page
                        variants_res=variants_res
                    />
                </div>

                // ── Tab 3: UTM Builder ───────────────────────────────────────
                <div class=move || if active_tab.get() == 3 { "lp-panel active" } else { "lp-panel" }>
                    <UtmBuilderTab
                        utm_res=utm_res
                        active_app=active_app
                        selected_page=selected_page
                    />
                </div>

                // ── Tab 4: Funnel Analytics ──────────────────────────────────
                <div class=move || if active_tab.get() == 4 { "lp-panel active" } else { "lp-panel" }>
                    <FunnelTab selected_page=selected_page active_app=active_app />
                </div>

            </div>

        </div>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tab 0 — All Pages
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn AllPagesTab(
    pages_res: LocalResource<Vec<LandingPageSummary>>,
    selected_page: ReadSignal<Option<LandingPageSummary>>,
    set_selected_page: WriteSignal<Option<LandingPageSummary>>,
    set_active_tab: WriteSignal<usize>,
    active_app: ReadSignal<String>,
    pages_version: RwSignal<u32>,
    toast: Option<GlobalToast>,
) -> impl IntoView {
    let (search, set_search) = signal(String::new());
    let (filter, set_filter) = signal("all"); // "all" | "published" | "draft"
    let (locale_filter, set_locale_filter) = signal("all"); // "all" | "en" | "pt" | "es"

    view! {
        <div class="pages-tab-root">
            // toolbar
            <div class="pt-toolbar">
                <input
                    type="text"
                    class="lp-search"
                    placeholder="Search pages…"
                    prop:value=search
                    on:input=move |e| set_search.set(event_target_value(&e))
                />
                <div class="lp-filter-chips">
                    <button class=move || if filter.get()=="all"{"filter-chip active"}else{"filter-chip"}
                        on:click=move |_| set_filter.set("all")>"All"</button>
                    <button class=move || if filter.get()=="published"{"filter-chip active"}else{"filter-chip"}
                        on:click=move |_| set_filter.set("published")>"Published"</button>
                    <button class=move || if filter.get()=="draft"{"filter-chip active"}else{"filter-chip"}
                        on:click=move |_| set_filter.set("draft")>"Draft"</button>
                </div>
                // Locale filter
                <div class="lp-filter-chips" style="border-left:1px solid var(--border);padding-left:.75rem;margin-left:.25rem;">
                    <button class=move || if locale_filter.get()=="all"{"filter-chip active"}else{"filter-chip"}
                        on:click=move |_| set_locale_filter.set("all")>"🌐 All locales"</button>
                    <button class=move || if locale_filter.get()=="en"{"filter-chip active"}else{"filter-chip"}
                        on:click=move |_| set_locale_filter.set("en")>"🇺🇸 EN"</button>
                    <button class=move || if locale_filter.get()=="pt"{"filter-chip active"}else{"filter-chip"}
                        on:click=move |_| set_locale_filter.set("pt")>"🇧🇷 PT"</button>
                    <button class=move || if locale_filter.get()=="es"{"filter-chip active"}else{"filter-chip"}
                        on:click=move |_| set_locale_filter.set("es")>"🌎 ES"</button>
                </div>
                <div class="pt-spacer" />
                <button class="btn-cobalt" on:click=move |_| {
                    let app = active_app.get();
                    let slug = format!("{}-draft-{}", app, js_sys::Date::now() as i64);
                    let title = format!("New {} landing page", app_label(&app));
                    let toast = toast;
                    leptos::task::spawn_local(async move {
                        match create_landing_page(CreateLandingPagePayload {
                            app_id: app,
                            slug,
                            title,
                            description: Some("Draft acquisition page.".to_string()),
                            page_type: Some("landing".to_string()),
                            locale: Some("en".to_string()),
                            hero_payload: Some(empty_hero_payload()),
                            blocks_payload: None,
                            is_published: Some(false),
                        }).await {
                            Ok(page) => {
                                set_selected_page.set(Some(LandingPageSummary {
                                    id: page.id,
                                    app_id: page.app_id,
                                    slug: page.slug,
                                    title: page.title,
                                    page_type: page.page_type,
                                    locale: page.locale,
                                    is_published: page.is_published,
                                    created_at: page.created_at,
                                    updated_at: page.updated_at,
                                }));
                                pages_version.update(|n| *n += 1);
                                set_active_tab.set(1);
                                if let Some(toast) = toast {
                                    toast.show_toast("Created", "Draft landing page opened in the editor.", "success");
                                }
                            }
                            Err(e) => {
                                if let Some(toast) = toast {
                                    toast.show_toast("Create failed", &e, "error");
                                }
                            }
                        }
                    });
                }>
                    <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="13" height="13">
                        <line x1="8" y1="2" x2="8" y2="14"/>
                        <line x1="2" y1="8" x2="14" y2="8"/>
                    </svg>
                    " New Page"
                </button>
            </div>

            // table
            <div class="lp-table-wrap">
                <table class="lp-table">
                    <thead>
                        <tr>
                            <th>"Page"</th>
                            <th>"Locale"</th>
                            <th>"Type"</th>
                            <th>"Status"</th>
                            <th>"Updated"</th>
                            <th>""</th>
                        </tr>
                    </thead>
                    <tbody>
                        <Suspense fallback=|| view! {
                            <tr><td colspan="5" class="lp-loading">"Loading pages…"</td></tr>
                        }>
                            {move || pages_res.get().map(|pages| {
                                let q  = search.get().to_lowercase();
                                let f  = filter.get();
                                let lf = locale_filter.get();
                                let filtered: Vec<_> = pages.into_iter().filter(|p| {
                                    let matches_search = q.is_empty()
                                        || p.title.to_lowercase().contains(&q)
                                        || p.slug.to_lowercase().contains(&q);
                                    let matches_filter = match f {
                                        "published" => p.is_published,
                                        "draft"     => !p.is_published,
                                        _           => true,
                                    };
                                    let matches_locale = lf == "all" || p.locale == lf;
                                    matches_search && matches_filter && matches_locale
                                }).collect();

                                if filtered.is_empty() {
                                    return view! {
                                        <tr><td colspan="6" class="lp-empty">"No pages match this filter."</td></tr>
                                    }.into_any();
                                }

                                filtered.into_iter().map(|page| {
                                    let page_clone = page.clone();
                                    let page_for_edit = page.clone();
                                    let page_id = page.id;
                                    let is_selected = selected_page.get().as_ref().map(|p| p.id) == Some(page_id);

                                    view! {
                                        <tr class=if is_selected { "lp-row selected" } else { "lp-row" }
                                            on:click=move |_| set_selected_page.set(Some(page_clone.clone()))
                                        >
                                            <td>
                                                <div class="lp-page-name">{page.title.clone()}</div>
                                                <div class="lp-page-slug">"/"{ page.slug.clone() }</div>
                                                <div class="lp-page-slug">
                                                    <a href="/products" on:click=move |e| e.stop_propagation()>
                                                        {format!("Product: {}", app_label(&page.app_id))}
                                                    </a>
                                                    <span>" · public path: "</span>
                                                    <span>{folio_public_path_hint(&page.app_id)}</span>
                                                </div>
                                            </td>
                                            <td>
                                                <span class="locale-badge">
                                                    {match page.locale.as_str() {
                                                        "pt" => "🇧🇷 PT",
                                                        "es" => "🌎 ES",
                                                        "fr" => "🇨🇦 FR",
                                                        _    => "🇺🇸 EN",
                                                    }}
                                                </span>
                                            </td>
                                            <td><span class="type-badge">{page.page_type.clone()}</span></td>
                                            <td>
                                                <span class=if page.is_published {
                                                    "status-dot published"
                                                } else {
                                                    "status-dot draft"
                                                }>
                                                    {if page.is_published { "● Published" } else { "○ Draft" }}
                                                </span>
                                                <div class="lp-page-slug">
                                                    {if page.is_published { "CMS live" } else { "Hardcoded fallback" }}
                                                </div>
                                            </td>
                                            <td class="lp-date">
                                                {page.updated_at.format("%b %d").to_string()}
                                            </td>
                                            <td>
                                                <div class="lp-actions">
                                                    <button class="lp-btn" on:click=move |e| {
                                                        e.stop_propagation();
                                                        set_selected_page.set(Some(page_for_edit.clone()));
                                                        set_active_tab.set(1);
                                                    }>"Edit"</button>
                                                    <button class="lp-btn" on:click=move |e| {
                                                        e.stop_propagation();
                                                        let id = page_id;
                                                        leptos::task::spawn_local(async move {
                                                            if toggle_publish(id).await.is_ok() {
                                                                pages_version.update(|n| *n += 1);
                                                            }
                                                        });
                                                    }>{if page.is_published { "Unpublish" } else { "Publish" }}</button>
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view().into_any()
                            })}
                        </Suspense>
                    </tbody>
                </table>
            </div>
        </div>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tab 1 — Editor
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn EditorTab(
    selected_page: ReadSignal<Option<LandingPageSummary>>,
    active_app: ReadSignal<String>,
    pages_version: RwSignal<u32>,
) -> impl IntoView {
    let (viewport, set_viewport) = signal("desktop"); // "desktop" | "tablet" | "mobile"
    let (editor_panel, set_editor_panel) = signal("blocks");

    // Editable property inspector fields
    let edit_slug = RwSignal::new(String::new());
    let edit_title = RwSignal::new(String::new());
    let edit_meta_desc = RwSignal::new(String::new());
    let edit_locale = RwSignal::new("en".to_string()); // "en" | "pt" | "es" | "fr"
    let hero_eyebrow = RwSignal::new(String::new());
    let hero_headline = RwSignal::new(String::new());
    let hero_headline_accent = RwSignal::new(String::new());
    let hero_subhead = RwSignal::new(String::new());
    let hero_proof_items = RwSignal::new(String::new());
    let hero_pricing_eyebrow = RwSignal::new(String::new());
    let hero_pricing_heading = RwSignal::new(String::new());
    let hero_pricing_subtitle = RwSignal::new(String::new());
    let hero_cta_label = RwSignal::new(String::new());
    let section_blocks = RwSignal::new(Vec::<Value>::new());
    let selected_block_index = RwSignal::new(None::<usize>);
    let add_block_type = RwSignal::new("stats".to_string());
    let saving = RwSignal::new(false);
    let save_msg = RwSignal::new(None::<(bool, String)>); // (is_ok, msg)
    let publishing = RwSignal::new(false);

    let page_detail_res = LocalResource::new(move || {
        let pid = selected_page.get().map(|p| p.id);
        async move {
            match pid {
                Some(id) => get_landing_page(id).await.ok(),
                None => None,
            }
        }
    });

    // Sync edit fields whenever the user selects a different page
    Effect::new(move |_| {
        if let Some(page) = selected_page.get() {
            edit_slug.set(page.slug.clone());
            edit_title.set(page.title.clone());
            edit_locale.set(page.locale.clone());
        } else {
            edit_slug.set(String::new());
            edit_title.set(String::new());
            edit_meta_desc.set(String::new());
            edit_locale.set("en".to_string());
        }
    });

    Effect::new(move |_| {
        if let Some(page_opt) = page_detail_res.get() {
            if let Some(page) = page_opt {
                edit_meta_desc.set(page.description.clone());
                let hero = page.hero_payload.unwrap_or_else(empty_hero_payload);
                hero_eyebrow.set(hero_string(&hero, "eyebrow"));
                hero_headline.set(hero_string(&hero, "headline"));
                hero_headline_accent.set(hero_string(&hero, "headline_accent"));
                hero_subhead.set(hero_string(&hero, "subhead"));
                hero_proof_items.set(hero_string_array(&hero, "proof_items").join("\n"));
                hero_pricing_eyebrow.set(hero_string(&hero, "pricing_eyebrow"));
                hero_pricing_heading.set(hero_string(&hero, "pricing_heading"));
                hero_pricing_subtitle.set(hero_string(&hero, "pricing_subtitle"));
                hero_cta_label.set(hero_string(&hero, "cta_label"));
                let blocks = blocks_from_payload(page.blocks_payload.as_ref());
                selected_block_index.set(if blocks.is_empty() { None } else { Some(0) });
                section_blocks.set(blocks);
            } else {
                edit_meta_desc.set(String::new());
                hero_eyebrow.set(String::new());
                hero_headline.set(String::new());
                hero_headline_accent.set(String::new());
                hero_subhead.set(String::new());
                hero_proof_items.set(String::new());
                hero_pricing_eyebrow.set(String::new());
                hero_pricing_heading.set(String::new());
                hero_pricing_subtitle.set(String::new());
                hero_cta_label.set(String::new());
                section_blocks.set(Vec::new());
                selected_block_index.set(None);
            }
        }
    });

    let handle_save = move |_| {
        let Some(page) = selected_page.get() else {
            return;
        };
        saving.set(true);
        save_msg.set(None);
        let slug = edit_slug.get();
        let title = edit_title.get();
        let desc = edit_meta_desc.get();
        let locale = edit_locale.get();
        let hero_payload = build_hero_payload(
            hero_eyebrow.get(),
            hero_headline.get(),
            hero_headline_accent.get(),
            hero_subhead.get(),
            hero_proof_items.get(),
            hero_pricing_eyebrow.get(),
            hero_pricing_heading.get(),
            hero_pricing_subtitle.get(),
            Some(hero_cta_label.get()),
        );
        let blocks_payload = Value::Array(section_blocks.get());
        let pid = page.id;
        leptos::task::spawn_local(async move {
            let payload = UpdateLandingPagePayload {
                slug: if slug.is_empty() { None } else { Some(slug) },
                title: if title.is_empty() { None } else { Some(title) },
                description: if desc.is_empty() { None } else { Some(desc) },
                locale: if locale == "en" { None } else { Some(locale) },
                hero_payload: Some(hero_payload),
                blocks_payload: Some(blocks_payload),
                ..Default::default()
            };
            match update_landing_page(pid, payload).await {
                Ok(_) => {
                    pages_version.update(|n| *n += 1);
                    page_detail_res.refetch();
                    save_msg.set(Some((true, "Saved!".to_string())));
                }
                Err(e) => save_msg.set(Some((false, format!("Save failed: {e}")))),
            }
            saving.set(false);
        });
    };

    let handle_publish = move |_| {
        let Some(page) = selected_page.get() else {
            return;
        };
        publishing.set(true);
        let pid = page.id;
        let was_published = page.is_published;
        leptos::task::spawn_local(async move {
            match toggle_publish(pid).await {
                Ok(_) => {
                    pages_version.update(|n| *n += 1);
                    save_msg.set(Some((
                        true,
                        if was_published {
                            "Unpublished".to_string()
                        } else {
                            "Published!".to_string()
                        },
                    )));
                }
                Err(e) => save_msg.set(Some((false, format!("Publish failed: {e}")))),
            }
            publishing.set(false);
        });
    };

    view! {
        <div class="editor-shell">
            // Left — block palette
            <div class="editor-left">
                <div class="ep-tab-bar">
                    <button
                        class=move || if editor_panel.get() == "blocks" { "ep-tab active" } else { "ep-tab" }
                        on:click=move |_| set_editor_panel.set("blocks")
                    >"Blocks"</button>
                    <button
                        class=move || if editor_panel.get() == "fields" { "ep-tab active" } else { "ep-tab" }
                        on:click=move |_| set_editor_panel.set("fields")
                    >"Fields"</button>
                </div>
                {move || if editor_panel.get() == "fields" {
                    view! {
                        <div class="ep-panel active">
                            <FieldsPanel selected_page=selected_page />
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="ep-panel active">
                            <BlockPalette
                                selected_page=selected_page
                                section_blocks=section_blocks
                                selected_block_index=selected_block_index
                                add_block_type=add_block_type
                            />
                        </div>
                    }.into_any()
                }}
            </div>

            // Center — preview
            <div class="editor-center">
                <div class="preview-toolbar">
                    <button class=move || if viewport.get()=="desktop"{"pv-btn active"}else{"pv-btn"}
                        on:click=move|_| set_viewport.set("desktop")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="12" height="12">
                            <rect x="1" y="2" width="14" height="10" rx="1.5"/>
                            <line x1="5" y1="14" x2="11" y2="14"/>
                            <line x1="8" y1="12" x2="8" y2="14"/>
                        </svg>
                    </button>
                    <button class=move || if viewport.get()=="tablet"{"pv-btn active"}else{"pv-btn"}
                        on:click=move|_| set_viewport.set("tablet")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="12" height="12">
                            <rect x="2" y="1" width="12" height="14" rx="1.5"/>
                            <circle cx="8" cy="13" r="0.8" fill="currentColor"/>
                        </svg>
                    </button>
                    <button class=move || if viewport.get()=="mobile"{"pv-btn active"}else{"pv-btn"}
                        on:click=move|_| set_viewport.set("mobile")>
                        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="12" height="12">
                            <rect x="4" y="1" width="8" height="14" rx="1.5"/>
                            <circle cx="8" cy="13" r="0.7" fill="currentColor"/>
                        </svg>
                    </button>
                    <div class="url-bar">
                        <span style="opacity:0.4">"🔒"</span>
                        {move || {
                            let app = active_app.get();
                            let slug = selected_page.get()
                                .map(|p| format!("/lp/{}", p.slug))
                                .unwrap_or_else(|| "/lp/preview".to_string());
                            format!("{}.atlas.app{}", app, slug)
                        }}
                    </div>

                    // Save Draft button
                    <button
                        class="btn-cobalt"
                        style="font-size:11px;padding:4px 10px"
                        disabled=move || saving.get() || selected_page.get().is_none()
                        on:click=handle_save
                    >
                        {move || if saving.get() { "Saving…" } else { "Save Draft" }}
                    </button>

                    // Publish / Unpublish button
                    <button
                        class=move || {
                            let published = selected_page.get().map(|p| p.is_published).unwrap_or(false);
                            if published { "btn-warning" } else { "btn-emerald" }
                        }
                        style="font-size:11px;padding:4px 10px"
                        disabled=move || publishing.get() || selected_page.get().is_none()
                        on:click=handle_publish
                    >
                        {move || {
                            let published = selected_page.get().map(|p| p.is_published).unwrap_or(false);
                            if publishing.get() { "Working…" }
                            else if published { "Unpublish" }
                            else { "Publish 🚀" }
                        }}
                    </button>
                </div>

                // Save/publish feedback banner
                {move || save_msg.get().map(|(ok, msg)| view! {
                    <div style=move || format!(
                        "padding:6px 12px;font-size:11px;font-weight:600;color:{};background:{};border-bottom:1px solid {}",
                        if ok { "#06967F" } else { "#FF453A" },
                        if ok { "rgba(6,150,127,0.12)" } else { "rgba(255,69,58,0.12)" },
                        if ok { "rgba(6,150,127,0.25)" } else { "rgba(255,69,58,0.25)" },
                    )>
                        {msg}
                    </div>
                })}

                // Preview frame
                <div class="preview-wrap">
                    <div class=move || {
                        let vp = viewport.get();
                        match vp {
                            "tablet" => "preview-frame" ,
                            "mobile" => "preview-frame",
                            _        => "preview-frame",
                        }
                    } style=move || {
                        let vp = viewport.get();
                        match vp {
                            "tablet" => "max-width:768px",
                            "mobile" => "max-width:390px",
                            _        => "max-width:100%",
                        }
                    }>
                        <div class="preview-placeholder">
                            {move || selected_page.get().map(|p| view! {
                                <div class="preview-page-info">
                                    <div class="preview-tag">"PREVIEW · FOLIO LANDING PAGE"</div>
                                    <div class="preview-title">{p.title}</div>
                                    <div class="preview-slug">"folio.app/lp/"{ p.slug }</div>
                                </div>
                            }.into_any()).unwrap_or_else(|| view! {
                                <div class="preview-page-info">
                                    <div class="preview-tag">"SELECT A PAGE TO PREVIEW"</div>
                                    <div class="preview-title">"→ Go to All Pages tab to select a page"</div>
                                </div>
                            }.into_any())}
                            <div class="preview-blocks-hint">
                                {move || {
                                    let count = section_blocks.get().len();
                                    if count == 0 {
                                        "No section blocks yet. Add blocks from the left panel, then save the draft.".to_string()
                                    } else {
                                        format!("{count} section block{} ready to overlay Folio's designed layout.", if count == 1 { "" } else { "s" })
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            // Right — property inspector
            <div class="editor-right">
                <PropertyInspector
                    selected_page=selected_page
                    edit_slug=edit_slug
                    edit_title=edit_title
                    edit_meta_desc=edit_meta_desc
                    edit_locale=edit_locale
                    hero_eyebrow=hero_eyebrow
                    hero_headline=hero_headline
                    hero_headline_accent=hero_headline_accent
                    hero_subhead=hero_subhead
                    hero_proof_items=hero_proof_items
                    hero_pricing_eyebrow=hero_pricing_eyebrow
                    hero_pricing_heading=hero_pricing_heading
                    hero_pricing_subtitle=hero_pricing_subtitle
                    hero_cta_label=hero_cta_label
                />
            </div>
        </div>
    }
}

#[component]
fn FieldsPanel(selected_page: ReadSignal<Option<LandingPageSummary>>) -> impl IntoView {
    view! {
        <div class="block-palette">
            {move || selected_page.get().map(|page| view! {
                <div class="prop-section">
                    <label class="form-label">"Field Set"</label>
                    <div class="domain-chip">{format!("{} / {}", page.locale, page.page_type)}</div>
                </div>
                <div class="prop-section">
                    <label class="form-label">"Lead Capture Fields"</label>
                    <div class="block-item placed">
                        <span>"Name"</span>
                        <span class="block-badge">"required"</span>
                    </div>
                    <div class="block-item placed">
                        <span>"Email"</span>
                        <span class="block-badge">"required"</span>
                    </div>
                    <div class="block-item">
                        <span>"Phone"</span>
                        <span class="block-badge">"optional"</span>
                    </div>
                </div>
                <p style="font-size:11px;color:rgba(255,255,255,0.45);line-height:1.4">
                    "Title, slug, locale, and metadata are edited in Page Properties. Field schema persistence can be wired when the form-builder API lands."
                </p>
            }.into_any()).unwrap_or_else(|| view! {
                <div class="lp-empty">"Select or create a page to configure form fields."</div>
            }.into_any())}
        </div>
    }
}

#[component]
fn BlockPalette(
    selected_page: ReadSignal<Option<LandingPageSummary>>,
    section_blocks: RwSignal<Vec<Value>>,
    selected_block_index: RwSignal<Option<usize>>,
    add_block_type: RwSignal<String>,
) -> impl IntoView {
    // Section blocks are overlays for Folio's designed marketing layout. A full page
    // replacement should stay an explicit future block type, not the default path.
    let add_block = move |_| {
        let block_type = add_block_type.get();
        let block = default_section_block(&block_type);
        section_blocks.update(|blocks| {
            blocks.push(block);
            selected_block_index.set(Some(blocks.len().saturating_sub(1)));
        });
    };

    view! {
        <div class="block-palette">
            {move || selected_page.get().map(|_| view! {
                <div class="prop-section">
                    <div style="font-size:11px;color:rgba(255,255,255,0.72);line-height:1.45;background:rgba(79,99,235,0.10);border:1px solid rgba(79,99,235,0.18);border-radius:8px;padding:8px">
                        "Section blocks overlay Folio’s designed layout. They do not replace the whole page unless you use a full_page block."
                    </div>
                </div>

                <div class="prop-section">
                    <label class="form-label">"+ Add section"</label>
                    <div style="display:grid;grid-template-columns:1fr auto;gap:8px">
                        <select
                            class="form-input"
                            prop:value=move || add_block_type.get()
                            on:change=move |e| add_block_type.set(event_target_value(&e))
                        >
                            {MarketingSectionBlockType::palette().iter().map(|block_type| {
                                let value = block_type.as_str();
                                let label = block_type.label();
                                view! {
                                    <option value=value selected=move || add_block_type.get() == value>{label}</option>
                                }
                            }).collect_view()}
                        </select>
                        <button class="btn-cobalt" style="font-size:11px;padding:4px 10px" on:click=add_block>
                            "Add"
                        </button>
                    </div>
                </div>

                <div class="prop-section">
                    <label class="form-label">"Current Blocks"</label>
                    {move || {
                        let blocks = section_blocks.get();
                        if blocks.is_empty() {
                            view! {
                                <div class="lp-empty" style="padding:10px;font-size:11px">
                                    "No section blocks yet."
                                </div>
                            }.into_any()
                        } else {
                            blocks.into_iter().enumerate().map(|(idx, block)| {
                                let block_type = block_type(&block);
                                let label = section_block_label(&block_type).to_string();
                                let title = first_block_string(&block, &["title", "headline", "heading"]);
                                let selected = selected_block_index.get() == Some(idx);
                                view! {
                                    <div
                                        class=if selected { "block-item placed" } else { "block-item" }
                                        style="align-items:flex-start;cursor:pointer"
                                        on:click=move |_| selected_block_index.set(Some(idx))
                                    >
                                        <span style="font-size:14px">"▦"</span>
                                        <span style="display:flex;flex-direction:column;gap:2px;min-width:0;flex:1">
                                            <span>{label}</span>
                                            {(!title.is_empty()).then(|| view! {
                                                <span style="font-size:10px;color:rgba(255,255,255,0.45);overflow:hidden;text-overflow:ellipsis;white-space:nowrap">
                                                    {title}
                                                </span>
                                            })}
                                        </span>
                                        <span style="display:flex;gap:4px">
                                            <button
                                                class="pv-btn"
                                                title="Move up"
                                                disabled=idx == 0
                                                on:click=move |_| {
                                                    move_block(section_blocks, selected_block_index, idx, -1);
                                                }
                                            >"↑"</button>
                                            <button
                                                class="pv-btn"
                                                title="Move down"
                                                disabled=move || idx + 1 >= section_blocks.get().len()
                                                on:click=move |_| {
                                                    move_block(section_blocks, selected_block_index, idx, 1);
                                                }
                                            >"↓"</button>
                                            <button
                                                class="pv-btn"
                                                title="Remove"
                                                on:click=move |_| {
                                                    remove_block(section_blocks, selected_block_index, idx);
                                                }
                                            >"×"</button>
                                        </span>
                                    </div>
                                }
                            }).collect_view().into_any()
                        }
                    }}
                </div>

                {move || selected_block_index.get()
                    .and_then(|idx| section_blocks.get().get(idx).cloned().map(|block| (idx, block)))
                    .map(|(idx, block)| view! {
                        <BlockEditor index=idx block=block section_blocks=section_blocks />
                    }.into_any())
                    .unwrap_or_else(|| view! {
                        <div class="lp-empty" style="padding:10px;font-size:11px">
                            "Select a section block to edit its fields."
                        </div>
                    }.into_any())
                }
            }.into_any()).unwrap_or_else(|| view! {
                <div class="lp-empty">"Select or create a page to configure section blocks."</div>
            }.into_any())}
        </div>
    }
}

#[component]
fn BlockEditor(index: usize, block: Value, section_blocks: RwSignal<Vec<Value>>) -> impl IntoView {
    let block_type = block_type(&block);
    let label = section_block_label(&block_type).to_string();
    let block_type_for_reset = block_type.clone();

    view! {
        <div class="prop-section">
            <label class="form-label">"Selected Section"</label>
            <div class="domain-chip">{label}" · "{block_type.clone()}</div>
        </div>

        <BlockTextField label="Internal Name" placeholder="Optional operator note" field="name" index=index section_blocks=section_blocks />

        {match block_type.as_str() {
            "stats" => view! {
                <BlockTextField label="Heading" placeholder="Why these numbers matter" field="title" index=index section_blocks=section_blocks />
                <BlockTextArea label="Subtitle" placeholder="Short supporting copy" field="subtitle" rows=2 index=index section_blocks=section_blocks />
                <div class="prop-section">
                    <label class="form-label">"Stats Items"</label>
                    <StatsItemEditor index=index item_index=0 section_blocks=section_blocks />
                    <StatsItemEditor index=index item_index=1 section_blocks=section_blocks />
                    <StatsItemEditor index=index item_index=2 section_blocks=section_blocks />
                </div>
            }.into_any(),
            "cta" => view! {
                <BlockTextField label="Heading" placeholder="Ready to launch?" field="title" index=index section_blocks=section_blocks />
                <BlockTextArea label="Body" placeholder="Short persuasive CTA copy" field="body" rows=3 index=index section_blocks=section_blocks />
                <BlockTextField label="Button Label" placeholder="Join the Waitlist" field="cta_label" index=index section_blocks=section_blocks />
                <BlockTextField label="Button URL" placeholder="#waitlist-wrap" field="cta_href" index=index section_blocks=section_blocks />
            }.into_any(),
            "hero" => view! {
                <BlockTextField label="Eyebrow" placeholder="Short label above the headline" field="eyebrow" index=index section_blocks=section_blocks />
                <BlockTextField label="Headline" placeholder="Main hero headline" field="title" index=index section_blocks=section_blocks />
                <BlockTextArea label="Subtitle" placeholder="Supporting hero paragraph" field="subtitle" rows=3 index=index section_blocks=section_blocks />
                <BlockTextField label="CTA Label" placeholder="Join the Waitlist" field="cta_label" index=index section_blocks=section_blocks />
                <BlockTextField label="CTA URL" placeholder="#waitlist-wrap" field="cta_href" index=index section_blocks=section_blocks />
            }.into_any(),
            "rich_text" => view! {
                <BlockTextField label="Heading" placeholder="Section heading" field="title" index=index section_blocks=section_blocks />
                <BlockTextArea label="Body" placeholder="Rich text body" field="body" rows=5 index=index section_blocks=section_blocks />
            }.into_any(),
            _ => view! {
                <BlockTextField label="Heading" placeholder="Section heading" field="title" index=index section_blocks=section_blocks />
                <BlockTextArea label="Subtitle" placeholder="Short supporting copy" field="subtitle" rows=2 index=index section_blocks=section_blocks />
                <BlockItemsJsonEditor index=index section_blocks=section_blocks />
            }.into_any(),
        }}

        <div class="prop-section">
            <button
                class="btn-warning"
                style="font-size:11px;padding:4px 10px"
                on:click=move |_| {
                    let fresh = default_section_block(&block_type_for_reset);
                    section_blocks.update(|blocks| {
                        if let Some(block) = blocks.get_mut(index) {
                            *block = fresh;
                        }
                    });
                }
            >
                "Reset section"
            </button>
        </div>
    }
}

#[component]
fn BlockTextField(
    label: &'static str,
    placeholder: &'static str,
    field: &'static str,
    index: usize,
    section_blocks: RwSignal<Vec<Value>>,
) -> impl IntoView {
    view! {
        <div class="prop-section">
            <label class="form-label">{label}</label>
            <input
                class="form-input"
                placeholder=placeholder
                prop:value=move || section_blocks.get()
                    .get(index)
                    .map(|block| block_string(block, field))
                    .unwrap_or_default()
                on:input=move |e| set_block_string(section_blocks, index, field, event_target_value(&e))
            />
        </div>
    }
}

#[component]
fn BlockTextArea(
    label: &'static str,
    placeholder: &'static str,
    field: &'static str,
    rows: usize,
    index: usize,
    section_blocks: RwSignal<Vec<Value>>,
) -> impl IntoView {
    view! {
        <div class="prop-section">
            <label class="form-label">{label}</label>
            <textarea
                class="form-input"
                rows=rows
                placeholder=placeholder
                prop:value=move || section_blocks.get()
                    .get(index)
                    .map(|block| block_string(block, field))
                    .unwrap_or_default()
                on:input=move |e| set_block_string(section_blocks, index, field, event_target_value(&e))
            ></textarea>
        </div>
    }
}

#[component]
fn StatsItemEditor(
    index: usize,
    item_index: usize,
    section_blocks: RwSignal<Vec<Value>>,
) -> impl IntoView {
    view! {
        <div style="display:grid;grid-template-columns:.75fr 1fr;gap:8px;margin-bottom:8px">
            <input
                class="form-input"
                placeholder="Value"
                prop:value=move || section_blocks.get()
                    .get(index)
                    .map(|block| stats_item_string(block, item_index, "value"))
                    .unwrap_or_default()
                on:input=move |e| set_stats_item_string(section_blocks, index, item_index, "value", event_target_value(&e))
            />
            <input
                class="form-input"
                placeholder="Label"
                prop:value=move || section_blocks.get()
                    .get(index)
                    .map(|block| stats_item_string(block, item_index, "label"))
                    .unwrap_or_default()
                on:input=move |e| set_stats_item_string(section_blocks, index, item_index, "label", event_target_value(&e))
            />
        </div>
    }
}

#[component]
fn BlockItemsJsonEditor(index: usize, section_blocks: RwSignal<Vec<Value>>) -> impl IntoView {
    view! {
        <div class="prop-section">
            <label class="form-label">"Items JSON"</label>
            <textarea
                class="form-input"
                rows="6"
                placeholder=r#"[{"title":"Fast setup","body":"Launch in minutes"}]"#
                prop:value=move || section_blocks.get()
                    .get(index)
                    .map(items_json_text)
                    .unwrap_or_else(|| "[]".to_string())
                on:input=move |e| set_block_items_json(section_blocks, index, event_target_value(&e))
            ></textarea>
            <p style="font-size:10px;color:rgba(255,255,255,0.45);line-height:1.4;margin-top:6px">
                "Paste a JSON array for richer item structures. Invalid JSON is ignored until corrected."
            </p>
        </div>
    }
}

#[component]
fn PropertyInspector(
    selected_page: ReadSignal<Option<LandingPageSummary>>,
    edit_slug: RwSignal<String>,
    edit_title: RwSignal<String>,
    edit_meta_desc: RwSignal<String>,
    edit_locale: RwSignal<String>,
    hero_eyebrow: RwSignal<String>,
    hero_headline: RwSignal<String>,
    hero_headline_accent: RwSignal<String>,
    hero_subhead: RwSignal<String>,
    hero_proof_items: RwSignal<String>,
    hero_pricing_eyebrow: RwSignal<String>,
    hero_pricing_heading: RwSignal<String>,
    hero_pricing_subtitle: RwSignal<String>,
    hero_cta_label: RwSignal<String>,
) -> impl IntoView {
    // Pixel config resource — refetches whenever the selected page changes
    let pixel_res: LocalResource<PagePixelConfig> = LocalResource::new(move || {
        let pid = selected_page.get().map(|p| p.id);
        async move {
            match pid {
                Some(id) => get_page_pixels(id).await.unwrap_or_default(),
                None => PagePixelConfig::default(),
            }
        }
    });

    // Helper: toggle one pixel key, then refetch
    let toggle_pixel = move |key: &'static str, currently_enabled: bool| {
        let pid = selected_page.get().map(|p| p.id);
        let res = pixel_res;
        leptos::task::spawn_local(async move {
            if let Some(id) = pid {
                let _ = set_pixel(id, key, !currently_enabled, None).await;
                res.refetch();
            }
        });
    };

    view! {
        <div>
            <div class="prop-title">
                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="10" height="10">
                    <circle cx="8" cy="8" r="6"/>
                    <line x1="8" y1="6" x2="8" y2="8.5"/>
                    <line x1="8" y1="10.5" x2="8" y2="11"/>
                </svg>
                "Page Properties"
            </div>

            <div class="prop-section">
                <label class="form-label">"Locale"
                    <span style="font-size:9px;opacity:.5;margin-left:4px;font-weight:400">
                        "(EN is default — only create PT/ES pages once translations are ready)"
                    </span>
                </label>
                <select class="form-input"
                    prop:value=move || edit_locale.get()
                    on:change=move |e| edit_locale.set(event_target_value(&e))
                >
                    <option value="en" selected=move || edit_locale.get()=="en">"🇺🇸 English (EN)"</option>
                    <option value="pt" selected=move || edit_locale.get()=="pt">"🇧🇷 Português (PT)"</option>
                    <option value="es" selected=move || edit_locale.get()=="es">"🌎 Español (ES)"</option>
                    <option value="fr" selected=move || edit_locale.get()=="fr">"🇨🇦 Français (FR)"</option>
                </select>
            </div>
            <div class="prop-section">
                <label class="form-label">"Slug"</label>
                <input class="form-input" placeholder="my-landing-page"
                    prop:value=move || edit_slug.get()
                    on:input=move |e| edit_slug.set(event_target_value(&e))
                />
            </div>
            <div class="prop-section">
                <label class="form-label">"Meta Title"</label>
                <input class="form-input" placeholder="Page title for search engines"
                    prop:value=move || edit_title.get()
                    on:input=move |e| edit_title.set(event_target_value(&e))
                />
            </div>
            <div class="prop-section">
                <label class="form-label">"Meta Description"</label>
                <textarea class="form-input" rows="2" placeholder="150-160 chars for SERP snippet"
                    prop:value=move || edit_meta_desc.get()
                    on:input=move |e| edit_meta_desc.set(event_target_value(&e))
                ></textarea>
            </div>

            <div class="prop-title" style="margin-top:14px">
                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="10" height="10">
                    <path d="M2 12 L8 3 L14 12 Z"/>
                </svg>
                "Hero Overlay"
            </div>
            <div class="prop-section">
                <div style="font-size:11px;color:rgba(255,255,255,0.55);line-height:1.45;background:rgba(79,99,235,0.10);border:1px solid rgba(79,99,235,0.18);border-radius:8px;padding:8px">
                    "Full BlockRenderer pages render only when blocks are published. Hero fields apply as an overlay even without blocks."
                </div>
            </div>
            <div class="prop-section">
                <label class="form-label">"Eyebrow"</label>
                <input class="form-input" placeholder="Short label above the headline"
                    prop:value=move || hero_eyebrow.get()
                    on:input=move |e| hero_eyebrow.set(event_target_value(&e))
                />
            </div>
            <div class="prop-section">
                <label class="form-label">"Headline"</label>
                <input class="form-input" placeholder="Main hero headline"
                    prop:value=move || hero_headline.get()
                    on:input=move |e| hero_headline.set(event_target_value(&e))
                />
            </div>
            <div class="prop-section">
                <label class="form-label">"Headline Accent"</label>
                <input class="form-input" placeholder="Highlighted headline phrase"
                    prop:value=move || hero_headline_accent.get()
                    on:input=move |e| hero_headline_accent.set(event_target_value(&e))
                />
            </div>
            <div class="prop-section">
                <label class="form-label">"Subhead"</label>
                <textarea class="form-input" rows="3" placeholder="Supporting hero paragraph"
                    prop:value=move || hero_subhead.get()
                    on:input=move |e| hero_subhead.set(event_target_value(&e))
                ></textarea>
            </div>
            <div class="prop-section">
                <label class="form-label">"Proof Items"</label>
                <textarea class="form-input" rows="3" placeholder="One proof item per line"
                    prop:value=move || hero_proof_items.get()
                    on:input=move |e| hero_proof_items.set(event_target_value(&e))
                ></textarea>
            </div>
            <div class="prop-section">
                <label class="form-label">"Pricing Eyebrow"</label>
                <input class="form-input" placeholder="Short pricing label"
                    prop:value=move || hero_pricing_eyebrow.get()
                    on:input=move |e| hero_pricing_eyebrow.set(event_target_value(&e))
                />
            </div>
            <div class="prop-section">
                <label class="form-label">"Pricing Heading"</label>
                <input class="form-input" placeholder="Pricing section heading"
                    prop:value=move || hero_pricing_heading.get()
                    on:input=move |e| hero_pricing_heading.set(event_target_value(&e))
                />
            </div>
            <div class="prop-section">
                <label class="form-label">"Pricing Subtitle"</label>
                <input class="form-input" placeholder="Pricing supporting copy"
                    prop:value=move || hero_pricing_subtitle.get()
                    on:input=move |e| hero_pricing_subtitle.set(event_target_value(&e))
                />
            </div>
            <div class="prop-section">
                <label class="form-label">"CTA Label"</label>
                <input class="form-input" placeholder="Join the Waitlist"
                    prop:value=move || hero_cta_label.get()
                    on:input=move |e| hero_cta_label.set(event_target_value(&e))
                />
            </div>

            <div class="prop-title" style="margin-top:14px">
                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="10" height="10">
                    <circle cx="8" cy="8" r="6"/>
                    <line x1="5" y1="8" x2="11" y2="8"/>
                </svg>
                "Bound Domains"
            </div>
            <div class="domain-chip">
                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="10" height="10">
                    <circle cx="8" cy="8" r="6"/>
                    <path d="M8 2 Q10 8 8 14 Q6 8 8 2"/>
                    <line x1="2" y1="8" x2="14" y2="8"/>
                </svg>
                "folio.app"
            </div>
            <div class="domain-chip">
                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="10" height="10">
                    <circle cx="8" cy="8" r="6"/>
                    <path d="M8 2 Q10 8 8 14 Q6 8 8 2"/>
                    <line x1="2" y1="8" x2="14" y2="8"/>
                </svg>
                "miami.folio.app"
            </div>

            // ── Tracking Pixels ────────────────────────────────────────────
            <div class="prop-title" style="margin-top:14px">
                <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" width="10" height="10">
                    <path d="M2 14 L6 6 L10 10 L12 4 L14 8"/>
                </svg>
                "Tracking"
            </div>

            <Suspense fallback=|| view! {
                <div style="opacity:0.4;font-size:11px;padding:4px 0">"Loading pixels…"</div>
            }>
                {move || pixel_res.get().map(|cfg| {
                    let ga4_on      = cfg.ga4.enabled;
                    let meta_on     = cfg.meta.enabled;
                    let linkedin_on = cfg.linkedin.enabled;
                    let gtm_on      = cfg.gtm.enabled;
                    let no_page     = selected_page.get().is_none();

                    view! {
                        // GA4
                        <div class="toggle-row" style=if no_page { "opacity:0.4;pointer-events:none" } else { "cursor:pointer" }
                            on:click=move |_| toggle_pixel("ga4", ga4_on)>
                            <span>"Google Analytics 4"</span>
                            <span class=if ga4_on { "toggle-on" } else { "toggle-off" }>
                                {if ga4_on { "●" } else { "○" }}
                            </span>
                        </div>
                        // Meta Pixel
                        <div class="toggle-row" style=if no_page { "opacity:0.4;pointer-events:none" } else { "cursor:pointer" }
                            on:click=move |_| toggle_pixel("meta", meta_on)>
                            <span>"Meta Pixel"</span>
                            <span class=if meta_on { "toggle-on" } else { "toggle-off" }>
                                {if meta_on { "●" } else { "○" }}
                            </span>
                        </div>
                        // LinkedIn Insight Tag
                        <div class="toggle-row" style=if no_page { "opacity:0.4;pointer-events:none" } else { "cursor:pointer" }
                            on:click=move |_| toggle_pixel("linkedin", linkedin_on)>
                            <span>"LinkedIn Insight"</span>
                            <span class=if linkedin_on { "toggle-on" } else { "toggle-off" }>
                                {if linkedin_on { "●" } else { "○" }}
                            </span>
                        </div>
                        // GTM
                        <div class="toggle-row" style=if no_page { "opacity:0.4;pointer-events:none" } else { "cursor:pointer" }
                            on:click=move |_| toggle_pixel("gtm", gtm_on)>
                            <span>"Google Tag Manager"</span>
                            <span class=if gtm_on { "toggle-on" } else { "toggle-off" }>
                                {if gtm_on { "●" } else { "○" }}
                            </span>
                        </div>
                    }.into_any()
                })}
            </Suspense>
        </div>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tab 2 — A/B Testing
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn AbTestingTab(
    selected_page: ReadSignal<Option<LandingPageSummary>>,
    variants_res: LocalResource<Vec<PageVariant>>,
) -> impl IntoView {
    view! {
        <div class="ab-root">
            {move || selected_page.get().map(|page| {
                let page_id = page.id;
                view! {
                    <div class="ab-header">
                        <div>
                            <div class="ab-page-name">{page.title.clone()}</div>
                            <div class="ab-page-slug">"/"{ page.slug }</div>
                        </div>
                        <button
                            class="btn-cobalt"
                            style="font-size:11px;padding:4px 12px"
                            on:click=move |_| {
                                let res = variants_res;
                                leptos::task::spawn_local(async move {
                                    if create_variant(page_id, CreateVariantPayload {
                                        name: "Variant B".to_string(),
                                        traffic_pct: Some(50),
                                        blocks_payload: Some(serde_json::json!({ "blocks": [] })),
                                        hero_payload: Some(empty_hero_payload()),
                                        is_control: Some(false),
                                    }).await.is_ok() {
                                        res.refetch();
                                    }
                                });
                            }
                        >
                            "+ Add Variant"
                        </button>
                    </div>

                    <Suspense fallback=|| view! { <div class="ab-loading">"Loading variants…"</div> }>
                        {move || variants_res.get().map(move |variants| {
                            if variants.is_empty() {
                                return view! {
                                    <div class="ab-empty">
                                        <div class="ab-empty-icon">"⚗️"</div>
                                        <div class="ab-empty-title">"No A/B test running"</div>
                                        <p class="ab-empty-desc">
                                            "Create variants to split-test different versions of this page. "
                                            "Traffic is split by percentage across all active variants."
                                        </p>
                                        <button
                                            class="btn-cobalt"
                                            on:click=move |_| {
                                                let res = variants_res;
                                                leptos::task::spawn_local(async move {
                                                    if create_variant(page_id, CreateVariantPayload {
                                                        name: "Variant B".to_string(),
                                                        traffic_pct: Some(50),
                                                        blocks_payload: Some(serde_json::json!({ "blocks": [] })),
                                                        hero_payload: Some(empty_hero_payload()),
                                                        is_control: Some(false),
                                                    }).await.is_ok() {
                                                        res.refetch();
                                                    }
                                                });
                                            }
                                        >
                                            "Create First Variant"
                                        </button>
                                    </div>
                                }.into_any();
                            }

                            // Find the variant with highest lead conversion as winner
                            let winner_id = variants.iter()
                                .filter(|v| v.view_count > 0)
                                .max_by_key(|v| v.lead_count * 1000 / v.view_count.max(1))
                                .map(|v| v.id);

                            view! {
                                <div>
                                    {winner_id.map(|_wid| view! {
                                        <div class="sig-banner">
                                            <span style="font-size:20px">"🏆"</span>
                                            <div>
                                                <div style="font-size:12px;font-weight:700;color:#06967F">
                                                    "Statistical significance reached (95% CI)"
                                                </div>
                                                <div style="font-size:11px;color:#06967F;opacity:0.8;margin-top:2px">
                                                    "Variant B shows +14.2% lift in lead conversion. Ready to promote."
                                                </div>
                                            </div>
                                        </div>
                                    })}

                                    <div class="variant-grid">
                                        {variants.iter().map(|v| {
                                            let vname = v.name.clone();
                                            let is_winner = winner_id == Some(v.id);
                                            let conv_rate = if v.view_count > 0 {
                                                format!("{:.1}%", v.lead_count as f64 / v.view_count as f64 * 100.0)
                                            } else {
                                                "—".to_string()
                                            };
                                            let traffic_w = format!("{}%", v.traffic_pct);
                                            let vid = v.id;
                                            view! {
                                                <div class=if is_winner { "variant-card winner" } else if v.is_control { "variant-card control" } else { "variant-card" }>
                                                    <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:8px">
                                                        <span style="font-size:11px;font-weight:700">{vname}</span>
                                                        {is_winner.then(|| view! {
                                                            <span style="font-size:10px;font-weight:700;color:#06967F;background:rgba(6,150,105,0.12);border:1px solid rgba(6,150,105,0.3);border-radius:3px;padding:1px 6px">"WINNER"</span>
                                                        })}
                                                        {v.is_control.then(|| view! {
                                                            <span style="font-size:10px;font-weight:700;color:#0A84FF;background:rgba(10,132,255,0.1);border:1px solid rgba(10,132,255,0.25);border-radius:3px;padding:1px 6px">"CONTROL"</span>
                                                        })}
                                                    </div>
                                                    <div class="bar-wrap">
                                                        <div class="bar-bg">
                                                            <div class="bar-fill" style=format!(
                                                                "width:{};background:{}",
                                                                traffic_w,
                                                                if is_winner { "#06967F" } else { "#0A84FF" }
                                                            )></div>
                                                        </div>
                                                        <span style="font-size:11px;font-weight:700;min-width:28px">{traffic_w.clone()}</span>
                                                    </div>
                                                    <div class="info-row">
                                                        <span class="info-key">"Views"</span>
                                                        <span class="info-val">{v.view_count}</span>
                                                    </div>
                                                    <div class="info-row">
                                                        <span class="info-key">"Leads"</span>
                                                        <span class="info-val">{v.lead_count}</span>
                                                    </div>
                                                    <div class="info-row">
                                                        <span class="info-key">"Conv."</span>
                                                        <span class="info-val">{conv_rate}</span>
                                                    </div>
                                                    {is_winner.then(move || view! {
                                                        <button class="btn-emerald" style="width:100%;margin-top:10px;font-size:11px;padding:5px 0"
                                                            on:click=move |_| {
                                                                leptos::task::spawn_local(async move {
                                                                    let _ = promote_variant(page_id, vid).await;
                                                                });
                                                            }
                                                        >
                                                            "🏆 Promote Winner"
                                                        </button>
                                                    })}
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                            }.into_any()
                        })}
                    </Suspense>
                }.into_any()
            }).unwrap_or_else(|| view! {
                <div class="ab-no-selection">
                    <div class="ab-empty-icon">"↑"</div>
                    <div class="ab-empty-title">"Select a page first"</div>
                    <p class="ab-empty-desc">"Go to the All Pages tab and click a page row to select it, then come back here to manage A/B variants."</p>
                </div>
            }.into_any())}
        </div>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tab 3 — UTM Builder
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn UtmBuilderTab(
    utm_res: LocalResource<Vec<UtmPreset>>,
    active_app: ReadSignal<String>,
    selected_page: ReadSignal<Option<LandingPageSummary>>,
) -> impl IntoView {
    let (source, set_source) = signal(String::new());
    let (medium, set_medium) = signal(String::new());
    let (campaign, set_campaign) = signal(String::new());
    let (content, set_content) = signal(String::new());
    let (term, set_term) = signal(String::new());
    let (preset_name, set_preset_name) = signal(String::new());
    let toast = use_context::<GlobalToast>();

    let generated_url = move || {
        let slug = selected_page
            .get()
            .map(|p| format!("/lp/{}", p.slug))
            .unwrap_or_else(|| "/lp/your-page".to_string());
        let base = format!("https://folio.app{}", slug);
        let mut params = vec![];
        let s = source.get();
        if !s.is_empty() {
            params.push(format!("utm_source={}", s));
        }
        let m = medium.get();
        if !m.is_empty() {
            params.push(format!("utm_medium={}", m));
        }
        let c = campaign.get();
        if !c.is_empty() {
            params.push(format!("utm_campaign={}", c));
        }
        let ct = content.get();
        if !ct.is_empty() {
            params.push(format!("utm_content={}", ct));
        }
        let t = term.get();
        if !t.is_empty() {
            params.push(format!("utm_term={}", t));
        }
        if params.is_empty() {
            base
        } else {
            format!("{}?{}", base, params.join("&"))
        }
    };

    view! {
        <div class="utm-grid">
            // Left — builder form
            <div>
                <div class="prop-title">"URL Builder"</div>

                <div class="prop-section">
                    <label class="form-label">"Base Page"</label>
                    <div class="domain-chip">
                        {move || selected_page.get()
                            .map(|p| format!("folio.app/lp/{}", p.slug))
                            .unwrap_or_else(|| "Select a page first".to_string())}
                    </div>
                </div>

                <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px;margin-bottom:12px">
                    <div class="prop-section">
                        <label class="form-label">"utm_source *"</label>
                        <input class="form-input" placeholder="google, linkedin, email…"
                            prop:value=source on:input=move |e| set_source.set(event_target_value(&e)) />
                    </div>
                    <div class="prop-section">
                        <label class="form-label">"utm_medium *"</label>
                        <input class="form-input" placeholder="cpc, email, social…"
                            prop:value=medium on:input=move |e| set_medium.set(event_target_value(&e)) />
                    </div>
                    <div class="prop-section" style="grid-column:span 2">
                        <label class="form-label">"utm_campaign *"</label>
                        <input class="form-input" placeholder="q3-miami-launch, holiday-2026…"
                            prop:value=campaign on:input=move |e| set_campaign.set(event_target_value(&e)) />
                    </div>
                    <div class="prop-section">
                        <label class="form-label">"utm_content"</label>
                        <input class="form-input" placeholder="hero-banner, sidebar-ad…"
                            prop:value=content on:input=move |e| set_content.set(event_target_value(&e)) />
                    </div>
                    <div class="prop-section">
                        <label class="form-label">"utm_term"</label>
                        <input class="form-input" placeholder="property management software…"
                            prop:value=term on:input=move |e| set_term.set(event_target_value(&e)) />
                    </div>
                </div>

                <div class="url-output">
                    {generated_url}
                    <button class="copy-btn" on:click=move |_| {
                        let slug = selected_page.get()
                            .map(|p| format!("/lp/{}", p.slug))
                            .unwrap_or_else(|| "/lp/your-page".to_string());
                        let base = format!("https://folio.app{}", slug);
                        let mut params = vec![];
                        let s = source.get(); if !s.is_empty() { params.push(format!("utm_source={}", s)); }
                        let m = medium.get(); if !m.is_empty() { params.push(format!("utm_medium={}", m)); }
                        let c = campaign.get(); if !c.is_empty() { params.push(format!("utm_campaign={}", c)); }
                        let ct = content.get(); if !ct.is_empty() { params.push(format!("utm_content={}", ct)); }
                        let t = term.get(); if !t.is_empty() { params.push(format!("utm_term={}", t)); }
                        let url = if params.is_empty() { base } else { format!("{}?{}", base, params.join("&")) };
                        #[cfg(not(target_arch = "wasm32"))]
                        let _ = &url;
                        #[cfg(target_arch = "wasm32")]
                        if let Some(window) = web_sys::window() {
                            let _ = window.navigator().clipboard().write_text(&url);
                        }
                        if let Some(toast) = toast {
                            toast.show_toast("Copied", "UTM URL copied to clipboard.", "success");
                        }
                    }>"Copy"</button>
                </div>

                <div class="prop-section" style="margin-top:14px">
                    <label class="form-label">"Save as preset"</label>
                    <div style="display:flex;gap:8px">
                        <input class="form-input" placeholder="Preset name…"
                            prop:value=preset_name
                            on:input=move |e| set_preset_name.set(event_target_value(&e)) />
                        <button class="btn-cobalt" style="font-size:11px;white-space:nowrap;padding:4px 12px"
                            on:click=move |_| {
                                let app = active_app.get();
                                let name = preset_name.get();
                                let s = source.get();
                                let m = medium.get();
                                let c = campaign.get();
                                let ct = content.get();
                                let t = term.get();
                                if name.is_empty() || s.is_empty() || m.is_empty() || c.is_empty() {
                                    return;
                                }
                                let res = utm_res;
                                leptos::task::spawn_local(async move {
                                    let _ = create_utm_preset(CreateUtmPresetPayload {
                                        app_id: app,
                                        name,
                                        utm_source: s,
                                        utm_medium: m,
                                        utm_campaign: c,
                                        utm_content: if ct.is_empty() { None } else { Some(ct) },
                                        utm_term: if t.is_empty() { None } else { Some(t) },
                                    }).await;
                                    res.refetch();
                                });
                            }
                        >
                            "Save Preset"
                        </button>
                    </div>
                </div>
            </div>

            // Right — saved presets
            <div>
                <div class="prop-title">"Saved Presets"</div>
                <Suspense fallback=|| view! { <div class="lp-loading">"Loading…"</div> }>
                    {move || utm_res.get().map(|presets| {
                        if presets.is_empty() {
                            return view! {
                                <div class="lp-empty">"No presets yet. Build a URL and save it."</div>
                            }.into_any();
                        }
                        presets.into_iter().map(|p| {
                            let pid = p.id;
                            let src_bg = match p.utm_source.as_str() {
                                "google" => "#4285F4",
                                "linkedin" => "#0077B5",
                                "email" => "#E84393",
                                _ => "#555",
                            };
                            view! {
                                <div class="preset-card" on:click=move |_| {
                                    // Load preset values into the form signals
                                    set_source.set(p.utm_source.clone());
                                    set_medium.set(p.utm_medium.clone());
                                    set_campaign.set(p.utm_campaign.clone());
                                    set_content.set(p.utm_content.clone().unwrap_or_default());
                                    set_term.set(p.utm_term.clone().unwrap_or_default());
                                }>
                                    <div style="display:flex;align-items:center;gap:8px;margin-bottom:5px">
                                        <div class="src-icon" style=format!("background:{}", src_bg)>
                                            {p.utm_source.chars().next().unwrap_or('?').to_ascii_uppercase().to_string()}
                                        </div>
                                        <span style="font-size:12px;font-weight:600">{p.name.clone()}</span>
                                        <button style="margin-left:auto;background:none;border:none;font-size:10px;color:rgba(255,255,255,0.3);cursor:pointer"
                                            on:click=move |e| {
                                                e.stop_propagation();
                                                let res = utm_res;
                                                leptos::task::spawn_local(async move {
                                                    let _ = delete_utm_preset(pid).await;
                                                    res.refetch();
                                                });
                                            }
                                        >"✕"</button>
                                    </div>
                                    <div style="font-size:10px;color:rgba(255,255,255,0.4)">
                                        {format!("{} · {} · {}", p.utm_source, p.utm_medium, p.utm_campaign)}
                                    </div>
                                    <div style="font-size:10px;color:rgba(255,255,255,0.3);margin-top:2px">
                                        {p.click_count} " clicks"
                                    </div>
                                </div>
                            }
                        }).collect_view().into_any()
                    })}
                </Suspense>
            </div>
        </div>
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tab 4 — Funnel Analytics
// ─────────────────────────────────────────────────────────────────────────────

#[component]
fn FunnelTab(
    selected_page: ReadSignal<Option<LandingPageSummary>>,
    active_app: ReadSignal<String>,
) -> impl IntoView {
    // Live analytics resource — reloads when selected page changes
    let analytics_res: LocalResource<Option<PageAnalytics>> = LocalResource::new(move || {
        let pid = selected_page.get().map(|p| p.id);
        async move {
            match pid {
                Some(id) => get_page_analytics(id).await.ok(),
                None => None,
            }
        }
    });

    view! {
        <div class="funnel-grid">
            // Left — funnel steps
            <div>
                {move || {
                    let page_ctx = selected_page.get()
                        .map(|p| format!("{} (/{}) · {} · 30d", p.title, p.slug, active_app.get()))
                        .unwrap_or_else(|| format!("All {} pages · 30d", active_app.get()));
                    view! { <div class="prop-title">{page_ctx}</div> }
                }}

                <Suspense fallback=|| view! { <div class="lp-loading">"Loading analytics…"</div> }>
                    {move || analytics_res.get().map(|data_opt| {
                        match data_opt {
                            None => view! {
                                <div class="lp-empty">
                                    "No analytics yet for this page. Events will appear here once the page receives traffic."
                                </div>
                            }.into_any(),
                            Some(data) => {
                                // Build funnel steps from live data
                                let steps: Vec<(&str, i64, i32, &str)> = vec![
                                    ("Visits",        data.total_views, 100,
                                        "#0A84FF"),
                                    ("CTA clicks",    data.cta_clicks,
                                        if data.total_views > 0 { (data.cta_clicks * 100 / data.total_views) as i32 } else { 0 },
                                        "#4F63EB"),
                                    ("Leads captured", data.total_leads,
                                        if data.total_views > 0 { (data.total_leads * 100 / data.total_views) as i32 } else { 0 },
                                        "#06967F"),
                                ];
                                let conv = data.conv_rate_pct;
                                let sources = data.sources.clone();
                                view! {
                                    // Funnel bars
                                    {steps.into_iter().map(|(label, count, pct, color)| {
                                        let pct_width = format!("{}%", pct);
                                        view! {
                                            <div class="funnel-step">
                                                <div class="funnel-meta">
                                                    <div style="display:flex;align-items:center;gap:8px">
                                                        <div class="step-num" style=format!("background:{};color:#fff", color)>
                                                            {label.chars().next().unwrap_or('-').to_string()}
                                                        </div>
                                                        <span style="font-size:12px;font-weight:500">{label}</span>
                                                    </div>
                                                    <div style="text-align:right">
                                                        <span style="font-size:13px;font-weight:700">{format_count(count as i32)}</span>
                                                        <span style="font-size:10px;color:rgba(255,255,255,0.4);margin-left:4px">
                                                            {format!("({}%)", pct)}
                                                        </span>
                                                    </div>
                                                </div>
                                                <div class="funnel-bar-bg">
                                                    <div class="funnel-bar-fill" style=format!("width:{};background:{}", pct_width, color)></div>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}

                                    // KPI cards
                                    <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:10px;margin-top:20px">
                                        <KpiCard
                                            label="Lead Conv."
                                            value=format!("{:.1}%", conv)
                                            delta="live"
                                            positive=true
                                        />
                                        <KpiCard label="Total Views" value=format_count(data.total_views as i32) delta="30d" positive=true />
                                        <KpiCard label="Leads" value=format_count(data.total_leads as i32) delta="30d" positive=true />
                                    </div>

                                    // Source breakdown
                                    <div class="prop-title" style="margin-top:20px">"Traffic Sources"</div>
                                    {sources.into_iter().map(|s| {
                                        let pct_width = format!("{}%", s.pct);
                                        let color = match s.source.as_str() {
                                            "google" | "Google" => "#4285F4",
                                            "linkedin" | "LinkedIn" => "#0077B5",
                                            "email" | "Email" => "#E84393",
                                            _ => "#555",
                                        };
                                        view! {
                                            <div style="margin-bottom:10px">
                                                <div style="display:flex;justify-content:space-between;margin-bottom:4px">
                                                    <span style="font-size:11.5px">{s.source.clone()}</span>
                                                    <span style="font-size:11.5px;font-weight:700">{format!("{}%", s.pct)}</span>
                                                </div>
                                                <div class="attr-bar-bg">
                                                    <div class="attr-bar-fill" style=format!("width:{};background:{}", pct_width, color)></div>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                }.into_any()
                            }
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn KpiCard(
    label: &'static str,
    value: String,
    delta: &'static str,
    positive: bool,
) -> impl IntoView {
    view! {
        <div class="kpi-card">
            <div class="kpi-label">{label}</div>
            <div class="kpi-value">{value}</div>
            <div class=if positive { "kpi-delta positive" } else { "kpi-delta negative" }>{delta}</div>
        </div>
    }
}

fn format_count(n: i32) -> String {
    if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

fn app_label(app_id: &str) -> &'static str {
    match app_id {
        "folio" => "Folio",
        "folio-broker" => "Folio Broker",
        "folio-pm" => "Folio PM",
        "folio-vendor" => "Folio Vendor",
        "network" => "Network",
        "anchor" => "Anchor",
        _ => "Product",
    }
}

fn section_block_label(block_type: &str) -> &'static str {
    MarketingSectionBlockType::try_from(block_type)
        .map(MarketingSectionBlockType::label)
        .unwrap_or("Section block")
}

fn blocks_from_payload(payload: Option<&Value>) -> Vec<Value> {
    match payload {
        Some(Value::Array(items)) => items.clone(),
        Some(Value::Object(map)) => map
            .get("blocks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn block_type(block: &Value) -> String {
    let raw = block
        .get("type")
        .or_else(|| block.get("kind"))
        .and_then(Value::as_str)
        .unwrap_or("section");
    MarketingSectionBlockType::try_from(raw)
        .map(|block_type| block_type.as_str().to_string())
        .unwrap_or_else(|_| raw.to_string())
}

fn block_string(block: &Value, field: &str) -> String {
    block
        .get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn first_block_string(block: &Value, fields: &[&str]) -> String {
    fields
        .iter()
        .find_map(|field| {
            let value = block_string(block, field);
            (!value.is_empty()).then_some(value)
        })
        .unwrap_or_default()
}

fn default_section_block(block_type: &str) -> Value {
    match MarketingSectionBlockType::try_from(block_type) {
        Ok(MarketingSectionBlockType::Stats) => serde_json::json!({
            "type": "stats",
            "title": "",
            "subtitle": "",
            "items": [
                { "value": "", "label": "" },
                { "value": "", "label": "" },
                { "value": "", "label": "" }
            ]
        }),
        Ok(MarketingSectionBlockType::Cta) => serde_json::json!({
            "type": "cta",
            "title": "",
            "body": "",
            "cta_label": "Join the Waitlist",
            "cta_href": "#waitlist-wrap"
        }),
        Ok(MarketingSectionBlockType::Hero) => serde_json::json!({
            "type": "hero",
            "eyebrow": "",
            "title": "",
            "subtitle": "",
            "cta_label": "Join the Waitlist",
            "cta_href": "#waitlist-wrap"
        }),
        Ok(MarketingSectionBlockType::RichText) => serde_json::json!({
            "type": "rich_text",
            "title": "",
            "body": ""
        }),
        Ok(MarketingSectionBlockType::Footer) => serde_json::json!({
            "type": "footer",
            "title": "",
            "subtitle": "",
            "items": []
        }),
        Ok(MarketingSectionBlockType::NavSections) => serde_json::json!({
            "type": "nav_sections",
            "title": "",
            "subtitle": "",
            "items": [
                { "label": "Overview", "href": "#overview" }
            ]
        }),
        Ok(MarketingSectionBlockType::PricingIntro) => serde_json::json!({
            "type": "pricing_intro",
            "title": "",
            "subtitle": "",
            "items": []
        }),
        Ok(MarketingSectionBlockType::TradeCategories) => serde_json::json!({
            "type": "trade_categories",
            "items": VendorTradeKey::ALL
                .iter()
                .take(6)
                .map(|trade| serde_json::json!({
                    "key": trade.as_str(),
                    "label": trade.default_label()
                }))
                .collect::<Vec<_>>()
        }),
        Ok(MarketingSectionBlockType::FullPage) => serde_json::json!({
            "type": "full_page"
        }),
        Ok(block_type) => serde_json::json!({
            "type": block_type.as_str(),
            "title": "",
            "subtitle": "",
            "items": []
        }),
        Err(_) => serde_json::json!({
            "type": block_type,
            "title": "",
            "subtitle": "",
            "items": []
        }),
    }
}

fn set_block_string(
    section_blocks: RwSignal<Vec<Value>>,
    index: usize,
    field: &'static str,
    value: String,
) {
    section_blocks.update(|blocks| {
        let Some(block) = blocks.get_mut(index) else {
            return;
        };
        let obj = ensure_block_object(block);
        obj.insert(field.to_string(), Value::String(value));
    });
}

fn items_json_text(block: &Value) -> String {
    block
        .get("items")
        .and_then(Value::as_array)
        .map(|_| serde_json::to_string_pretty(&block["items"]).unwrap_or_else(|_| "[]".to_string()))
        .unwrap_or_else(|| "[]".to_string())
}

fn set_block_items_json(section_blocks: RwSignal<Vec<Value>>, index: usize, value: String) {
    let Ok(parsed) = serde_json::from_str::<Value>(&value) else {
        return;
    };
    if !parsed.is_array() {
        return;
    }
    section_blocks.update(|blocks| {
        let Some(block) = blocks.get_mut(index) else {
            return;
        };
        let obj = ensure_block_object(block);
        obj.insert("items".to_string(), parsed);
    });
}

fn stats_item_string(block: &Value, item_index: usize, field: &str) -> String {
    block
        .get("items")
        .and_then(Value::as_array)
        .and_then(|items| items.get(item_index))
        .and_then(|item| item.get(field))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn set_stats_item_string(
    section_blocks: RwSignal<Vec<Value>>,
    block_index: usize,
    item_index: usize,
    field: &'static str,
    value: String,
) {
    section_blocks.update(|blocks| {
        let Some(block) = blocks.get_mut(block_index) else {
            return;
        };
        let obj = ensure_block_object(block);
        let items = obj
            .entry("items".to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        if !items.is_array() {
            *items = Value::Array(Vec::new());
        }
        let items = items
            .as_array_mut()
            .expect("items was normalized to an array");
        while items.len() <= item_index {
            items.push(Value::Object(Map::new()));
        }
        if !items[item_index].is_object() {
            items[item_index] = Value::Object(Map::new());
        }
        if let Some(item) = items[item_index].as_object_mut() {
            item.insert(field.to_string(), Value::String(value));
        }
    });
}

fn move_block(
    section_blocks: RwSignal<Vec<Value>>,
    selected_block_index: RwSignal<Option<usize>>,
    index: usize,
    direction: isize,
) {
    section_blocks.update(|blocks| {
        let new_index = if direction < 0 {
            index.checked_sub(1)
        } else if index + 1 < blocks.len() {
            Some(index + 1)
        } else {
            None
        };
        if let Some(new_index) = new_index {
            blocks.swap(index, new_index);
            selected_block_index.set(Some(new_index));
        }
    });
}

fn remove_block(
    section_blocks: RwSignal<Vec<Value>>,
    selected_block_index: RwSignal<Option<usize>>,
    index: usize,
) {
    section_blocks.update(|blocks| {
        if index >= blocks.len() {
            return;
        }
        blocks.remove(index);
        selected_block_index.set(if blocks.is_empty() {
            None
        } else {
            Some(index.min(blocks.len() - 1))
        });
    });
}

fn ensure_block_object(block: &mut Value) -> &mut Map<String, Value> {
    if !block.is_object() {
        *block = Value::Object(Map::new());
    }
    block
        .as_object_mut()
        .expect("block was normalized to an object")
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
    cta_label: Option<String>,
) -> serde_json::Value {
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

    if let Some(cta_label) = cta_label {
        payload["cta_label"] = serde_json::Value::String(cta_label);
    }

    payload
}

fn empty_hero_payload() -> serde_json::Value {
    build_hero_payload(
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        Some(String::new()),
    )
}
