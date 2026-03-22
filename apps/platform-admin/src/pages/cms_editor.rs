use leptos::prelude::*;
use shared_ui::components::file_attachments::FileAttachments;
use shared_ui::components::ui::textarea::Textarea;
use shared_ui::components::tabs::{Tabs, TabButton};
use shared_ui::components::ui::tabs::{TabsContent, TabsList};
use shared_ui::components::data_table::DataTable;
use leptos_router::hooks::use_query_map;

use crate::api::listings::{search_listings, create_listing};
use crate::api::files::create_file;
use crate::api::models::{ListingModel, ListingCreate, CreateFileInput};

#[component]
pub fn CmsEditor() -> impl IntoView {
    let query = use_query_map();
    let default_t = query.get_untracked().get("tab").unwrap_or_else(|| "articles".to_string());

    let title = RwSignal::new("".to_string());
    let url = RwSignal::new("".to_string());
    let author_name = RwSignal::new("".to_string());
    let summary = RwSignal::new("".to_string());
    let content_html = RwSignal::new("".to_string());
    let listing_type = RwSignal::new("article".to_string());
    let hero_headline = RwSignal::new("".to_string());
    let cta_text = RwSignal::new("".to_string());
    let show_price = RwSignal::new(false);
    let show_map = RwSignal::new(false);
    let whatsapp_number = RwSignal::new("".to_string());
    let form_theme = RwSignal::new("light".to_string());
    
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    let default_dir = "00000000-0000-0000-0000-000000000000".to_string();
    let default_profile = "00000000-0000-0000-0000-000000000000".to_string();

    let (trigger_fetch, set_trigger_fetch) = signal(0);
    
    let listings_res = LocalResource::new(move || { 
        trigger_fetch.get();
        async move { search_listings("").await.unwrap_or_default() }
    });

    let article_headers = vec![
        "ID".to_string(), "Title".to_string(), "Type".to_string(), "Status".to_string(), "Created At".to_string()
    ];
    
    let article_data = Signal::derive(move || {
        listings_res.get().unwrap_or_default().into_iter().map(|l| {
            vec![
                l.id.clone(),
                l.title.clone(),
                l.listing_type.clone(),
                format!("{:?}", l.status),
                l.created_at.clone()
            ]
        }).collect::<Vec<Vec<String>>>()
    });

    let handle_publish = move |_| {
        let dir = default_dir.clone();
        let profile = default_profile.clone();
        leptos::task::spawn_local(async move {
            let data = ListingCreate {
                title: title.get(),
                description: summary.get(),
                directory_id: dir,
                profile_id: profile,
                category_id: None,
                listing_type: Some(listing_type.get()),
                price: None,
                price_type: None,
                country: None,
                state: None,
                city: None,
                neighborhood: None,
                latitude: None,
                longitude: None,
                additional_info: if listing_type.get() == "landing_page" {
                    Some(serde_json::json!({
                        "hero_headline": hero_headline.get(),
                        "cta_text": cta_text.get(),
                        "show_price": show_price.get(),
                        "show_map": show_map.get(),
                        "whatsapp_number": whatsapp_number.get(),
                        "form_theme": form_theme.get()
                    }))
                } else {
                    None
                },
                is_featured: Some(false),
                is_based_on_template: Some(false),
                based_on_template_id: None,
                is_ad_placement: Some(false),
                is_active: Some(true),
                slug: Some(url.get()),
            };
            match create_listing(data).await {
                Ok(_) => {
                    set_trigger_fetch.update(|v| *v += 1);
                    title.set("".to_string());
                    summary.set("".to_string());
                    content_html.set("".to_string());
                    url.set("".to_string());
                }
                Err(e) => {
                    toast.message.set(Some(e));
                }
            }
        });
    };
    
    let handle_file_drop = move |filename: String| {
        leptos::task::spawn_local(async move {
            let data = CreateFileInput {
                name: filename,
                size: 1024,
                mime_type: "image/png".to_string(),
                hash_sha256: "dummy".to_string(),
                storage_type: "L".to_string(),
                storage_path: "/tmp/dummy".to_string(),
                is_anonymous: false,
                user_id: None,
            };
            match create_file(data).await {
                Ok(_) => { /* File created */ },
                Err(e) => { toast.message.set(Some(e)); }
            }
        });
    };

    view! {
        <div class="flex flex-col min-h-[calc(100vh-128px)] -mx-8 -mt-8">
            // ── Tabs + Actions Bar ──
            <Tabs default_value=default_t>
                <div class="h-14 flex items-center px-8 bg-surface-container-low border-b border-outline-variant/5">
                    <TabsList class="flex gap-8 h-full items-center".to_string()>
                        <TabButton label="All Articles" value="articles" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                        <TabButton label="Editor" value="editor" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                    </TabsList>
                    <div class="ml-auto flex items-center gap-4">
                        <span class="text-[10px] uppercase tracking-widest text-secondary/60">"Auto-saved 2m ago"</span>
                        <button class="px-4 py-1.5 text-xs font-bold border border-outline-variant/40 rounded hover:bg-surface-bright/20 transition-colors text-on-surface">"Discard"</button>
                        <button on:click=handle_publish class="px-6 py-1.5 text-xs font-bold btn-primary-gradient text-on-primary rounded-md shadow-lg shadow-primary/10">"Publish Draft"</button>
                    </div>
                </div>

                // ── Articles List ──
                <TabsContent value="articles".to_string()>
                    <div class="p-8 bg-surface-container">
                        <Suspense fallback=move || view! { <div class="p-4 text-on-surface-variant">"Loading..."</div> }>
                            <DataTable headers=article_headers.clone() data=article_data />
                        </Suspense>
                    </div>
                </TabsContent>

                // ── Editor ──
                <TabsContent value="editor".to_string()>
                    <div class="flex-1 flex overflow-hidden" style="height: calc(100vh - 128px - 56px);">
                        // ── Left Pane: Editing Environment ──
                        <section class="w-1/2 overflow-y-auto border-r border-outline-variant/10 p-8 bg-surface-container">
                            <div class="max-w-2xl mx-auto space-y-8">
                                // Form Header
                                <div class="space-y-1">
                                    <h1 class="text-2xl font-semibold tracking-tight text-on-surface">"Article Construction"</h1>
                                    <p class="text-sm text-on-surface-variant">"Compose and configure high-fidelity system content."</p>
                                </div>
                                // Core Metadata
                                <div class="grid grid-cols-2 gap-6">
                                    <div class="col-span-2 space-y-2">
                                        <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Title"</label>
                                        <input
                                            type="text"
                                            class="w-full bg-surface-container-high border-none rounded p-3 text-sm focus:ring-1 focus:ring-primary-dim transition-all text-on-surface"
                                            prop:value=move || title.get()
                                            on:input=move |ev| title.set(event_target_value(&ev))
                                        />
                                    </div>
                                    <div class="space-y-2">
                                        <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"URL Slug"</label>
                                        <div class="flex items-center bg-surface-container-high rounded overflow-hidden">
                                            <span class="pl-3 text-xs text-on-surface-variant select-none">"/news/"</span>
                                            <input
                                                type="text"
                                                class="flex-1 bg-transparent border-none p-3 text-sm focus:ring-0 text-on-surface"
                                                prop:value=move || url.get()
                                                on:input=move |ev| url.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="space-y-2">
                                        <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Content Type"</label>
                                        <select
                                            class="w-full bg-surface-container-high border-none rounded p-3 text-sm focus:ring-1 focus:ring-primary-dim text-on-surface appearance-none"
                                            on:change=move |ev| listing_type.set(event_target_value(&ev))
                                            prop:value=move || listing_type.get()
                                        >
                                            <option value="article">"Article"</option>
                                            <option value="landing_page">"Landing Page"</option>
                                        </select>
                                    </div>
                                </div>

                                // Conditional Landing Page Settings
                                {move || if listing_type.get() == "landing_page" {
                                    view! {
                                        <div class="p-5 bg-surface-container-highest/40 rounded-xl border border-primary/10 space-y-6">
                                            <div class="flex items-center gap-2 mb-2">
                                                <span class="material-symbols-outlined text-primary text-lg">"auto_awesome"</span>
                                                <h3 class="text-xs font-bold uppercase tracking-widest text-primary">"Landing Page Protocol"</h3>
                                            </div>
                                            <div class="space-y-4">
                                                <div class="space-y-2">
                                                    <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Hero Headline"</label>
                                                    <input
                                                        type="text"
                                                        class="w-full bg-surface-container-lowest border border-outline-variant/20 rounded p-3 text-sm text-on-surface"
                                                        placeholder="Orchestrating Global Logistics"
                                                        prop:value=move || hero_headline.get()
                                                        on:input=move |ev| hero_headline.set(event_target_value(&ev))
                                                    />
                                                </div>
                                                <div class="grid grid-cols-2 gap-4">
                                                    <div class="space-y-2">
                                                        <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"CTA Text"</label>
                                                        <input
                                                            type="text"
                                                            class="w-full bg-surface-container-lowest border border-outline-variant/20 rounded p-3 text-sm text-on-surface"
                                                            placeholder="Explore Ecosystem"
                                                            prop:value=move || cta_text.get()
                                                            on:input=move |ev| cta_text.set(event_target_value(&ev))
                                                        />
                                                    </div>
                                                    <div class="flex flex-col justify-end gap-3 pb-2">
                                                        <label class="flex items-center gap-3 cursor-pointer group">
                                                            <input type="checkbox" class="rounded border-outline-variant bg-surface-container text-primary focus:ring-0 focus:ring-offset-0" prop:checked=move || show_price.get() on:change=move |ev| show_price.set(event_target_checked(&ev)) />
                                                            <span class="text-xs text-on-surface-variant group-hover:text-on-surface">"Show Price"</span>
                                                        </label>
                                                        <label class="flex items-center gap-3 cursor-pointer group">
                                                            <input type="checkbox" class="rounded border-outline-variant bg-surface-container text-primary focus:ring-0 focus:ring-offset-0" prop:checked=move || show_map.get() on:change=move |ev| show_map.set(event_target_checked(&ev)) />
                                                            <span class="text-xs text-on-surface-variant group-hover:text-on-surface">"Show Map"</span>
                                                        </label>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! { <span/> }.into_any()
                                }}

                                // Summary
                                <div class="space-y-2">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Summary"</label>
                                    <Textarea rows=2u32 placeholder="A brief description of the content..." bind_value=summary />
                                </div>

                                // Body Content
                                <div class="space-y-2">
                                    <div class="flex justify-between items-end">
                                        <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Body Content"</label>
                                        <div class="flex gap-2 mb-1">
                                            <button class="p-1 hover:bg-surface-bright/30 rounded"><span class="material-symbols-outlined text-sm">"format_bold"</span></button>
                                            <button class="p-1 hover:bg-surface-bright/30 rounded"><span class="material-symbols-outlined text-sm">"format_italic"</span></button>
                                            <button class="p-1 hover:bg-surface-bright/30 rounded"><span class="material-symbols-outlined text-sm">"link"</span></button>
                                            <button class="p-1 hover:bg-surface-bright/30 rounded"><span class="material-symbols-outlined text-sm">"image"</span></button>
                                        </div>
                                    </div>
                                    <Textarea rows=12u32 placeholder="Write your content here..." bind_value=content_html />
                                </div>

                                // Asset Upload
                                <div class="space-y-2">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Asset Management"</label>
                                    <FileAttachments entity_type="Item".to_string() on_file_drop=Callback::new(handle_file_drop) />
                                </div>
                            </div>
                        </section>

                        // ── Right Pane: Live Preview ──
                        <section class="flex-1 bg-surface-container-lowest p-10 overflow-hidden flex flex-col">
                            <div class="flex justify-between items-center mb-6">
                                <div class="flex items-center gap-3">
                                    <span class="material-symbols-outlined text-secondary">"visibility"</span>
                                    <span class="text-[10px] font-bold uppercase tracking-widest text-secondary">"Real-Time Canvas"</span>
                                </div>
                                <div class="flex gap-2">
                                    <button class="p-1.5 rounded-full bg-surface-container text-primary"><span class="material-symbols-outlined text-sm">"desktop_windows"</span></button>
                                    <button class="p-1.5 rounded-full hover:bg-surface-container text-on-surface-variant"><span class="material-symbols-outlined text-sm">"smartphone"</span></button>
                                </div>
                            </div>
                            // Framed Preview
                            <div class="flex-1 rounded-xl bg-white overflow-y-auto shadow-2xl overflow-x-hidden">
                                <div class="w-full text-slate-900 font-sans">
                                    // Mock nav
                                    <nav class="h-16 px-10 flex items-center justify-between border-b border-slate-100">
                                        <div class="font-black text-xl tracking-tighter italic text-slate-900">"DIRECTORY"</div>
                                        <div class="flex gap-8 text-xs font-bold uppercase text-slate-500">
                                            <span>"Home"</span>
                                            <span>"Browse"</span>
                                            <span>"Contact"</span>
                                        </div>
                                    </nav>
                                    // Content
                                    <div class="p-10 max-w-2xl">
                                        <div class="flex gap-2 mb-4">
                                            <span class="px-2 py-0.5 bg-blue-50 text-blue-600 text-[10px] font-bold uppercase rounded">{move || listing_type.get()}</span>
                                        </div>
                                        <h2 class="text-2xl font-bold text-slate-800 mb-6">
                                            {move || if title.get().is_empty() { "Untitled Article".to_string() } else { title.get() }}
                                        </h2>
                                        <p class="text-slate-500 leading-relaxed mb-6 italic border-l-4 border-blue-500 pl-4">
                                            {move || if summary.get().is_empty() { "Article summary will appear here...".to_string() } else { summary.get() }}
                                        </p>
                                        <div class="prose prose-slate prose-sm text-slate-700" inner_html=move || {
                                            content_html.get().replace('\n', "<br/>")
                                        }>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </section>
                    </div>
                </TabsContent>
            </Tabs>
        </div>
    }
}
