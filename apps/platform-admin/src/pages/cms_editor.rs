use leptos::prelude::*;
use shared_ui::components::file_attachments::FileAttachments;
use shared_ui::components::ui::textarea::Textarea;
use shared_ui::components::tabs::{Tabs, TabButton};
use shared_ui::components::ui::tabs::{TabsContent, TabsList};
use shared_ui::components::data_table::DataTable;
use leptos_router::hooks::use_query_map;

// Phase 5: now uses the proper app_pages CRUD endpoint instead of api/listings
use crate::api::pages::{list_pages, create_page, delete_page, PageSummary, CreatePagePayload};
use crate::api::files::create_file;
use crate::api::models::CreateFileInput;

/// Parses the tenant_id from the query string (?tenant_id=...).
/// Falls back to the nil UUID so the editor doesn't crash without a tenant context.
fn get_tenant_id() -> uuid::Uuid {
    use leptos_router::hooks::use_query_map;
    let query = use_query_map();
    query
        .get_untracked()
        .get("tenant_id")
        .and_then(|s| s.parse::<uuid::Uuid>().ok())
        .unwrap_or(uuid::Uuid::nil())
}

#[component]
pub fn CmsEditor() -> impl IntoView {
    let query = use_query_map();
    let default_t = query.get_untracked().get("tab").unwrap_or_else(|| "pages".to_string());

    // ── Form signals ──────────────────────────────────────────────────────────
    let title = RwSignal::new("".to_string());
    let slug = RwSignal::new("".to_string());
    let summary = RwSignal::new("".to_string());
    let blocks_content = RwSignal::new("".to_string());
    let page_type = RwSignal::new("standard".to_string());
    let is_published = RwSignal::new(false);

    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let tenant_id = get_tenant_id();

    let (trigger_fetch, set_trigger_fetch) = signal(0);

    // ── Data ──────────────────────────────────────────────────────────────────
    let pages_res = LocalResource::new(move || {
        trigger_fetch.get();
        async move { list_pages(tenant_id).await.unwrap_or_default() }
    });

    let page_headers = vec![
        "Slug".to_string(),
        "Title".to_string(),
        "Type".to_string(),
        "Published".to_string(),
        "Updated".to_string(),
    ];

    let page_data = Signal::derive(move || {
        pages_res
            .get()
            .unwrap_or_default()
            .into_iter()
            .map(|p: PageSummary| {
                vec![
                    p.slug.clone(),
                    p.title.clone(),
                    p.page_type.clone(),
                    if p.is_published { "✓".to_string() } else { "Draft".to_string() },
                    p.updated_at.format("%Y-%m-%d").to_string(),
                ]
            })
            .collect::<Vec<Vec<String>>>()
    });

    // ── Publish handler ───────────────────────────────────────────────────────
    let handle_publish = move |_| {
        leptos::task::spawn_local(async move {
            let blocks: Option<serde_json::Value> = if blocks_content.get().is_empty() {
                Some(serde_json::json!([]))
            } else {
                serde_json::from_str(&blocks_content.get()).ok()
            };

            let payload = CreatePagePayload {
                slug: slug.get(),
                title: title.get(),
                description: summary.get(),
                page_type: Some(page_type.get()),
                hero_payload: None,
                blocks_payload: blocks,
                is_published: Some(is_published.get()),
            };

            match create_page(tenant_id, payload).await {
                Ok(_) => {
                    set_trigger_fetch.update(|v| *v += 1);
                    title.set("".to_string());
                    slug.set("".to_string());
                    summary.set("".to_string());
                    blocks_content.set("".to_string());
                    is_published.set(false);
                    toast.message.set(Some("Page published successfully.".to_string()));
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
                Ok(_) => {}
                Err(e) => {
                    toast.message.set(Some(e));
                }
            }
        });
    };

    view! {
        <div class="flex flex-col min-h-[calc(100vh-128px)] -mx-8 -mt-8">
            // ── Tabs + Actions Bar ──
            <Tabs default_value=default_t>
                <div class="h-14 flex items-center px-8 bg-surface-container-low border-b border-outline-variant/5">
                    <TabsList class="flex gap-8 h-full items-center".to_string()>
                        <TabButton label="All Pages" value="pages" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                        <TabButton label="Editor" value="editor" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                    </TabsList>
                    <div class="ml-auto flex items-center gap-4">
                        <span class="text-[10px] uppercase tracking-widest text-secondary/60">"Auto-saved 2m ago"</span>
                        <button class="px-4 py-1.5 text-xs font-bold border border-outline-variant/40 rounded hover:bg-surface-bright/20 transition-colors text-on-surface">"Discard"</button>
                        <button on:click=handle_publish class="px-6 py-1.5 text-xs font-bold btn-primary-gradient text-on-primary rounded-md shadow-lg shadow-primary/10">"Publish Page"</button>
                    </div>
                </div>

                // ── Pages List ──
                <TabsContent value="pages".to_string()>
                    <div class="p-8 bg-surface-container">
                        <div class="flex justify-between items-center mb-6">
                            <div>
                                <h2 class="text-xl font-bold text-on-surface">"Site Pages"</h2>
                                <p class="text-xs text-on-surface-variant mt-1">
                                    "Manage all pages for this tenant. Use the Editor tab to create new pages."
                                </p>
                            </div>
                            {if tenant_id == uuid::Uuid::nil() {
                                view! {
                                    <div class="flex items-center gap-2 px-4 py-2 bg-error-container text-error rounded-lg text-xs">
                                        <span class="material-symbols-outlined text-sm">"warning"</span>
                                        "No tenant selected — add ?tenant_id=<uuid> to the URL."
                                    </div>
                                }.into_any()
                            } else {
                                view! { <span /> }.into_any()
                            }}
                        </div>
                        <Suspense fallback=move || view! { <div class="p-4 text-on-surface-variant">"Loading pages..."</div> }>
                            <DataTable headers=page_headers.clone() data=page_data />
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
                                    <h1 class="text-2xl font-semibold tracking-tight text-on-surface">"Page Editor"</h1>
                                    <p class="text-sm text-on-surface-variant">"Create and publish CMS pages for your tenant site."</p>
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
                                            <span class="pl-3 text-xs text-on-surface-variant select-none">"/"</span>
                                            <input
                                                type="text"
                                                class="flex-1 bg-transparent border-none p-3 text-sm focus:ring-0 text-on-surface"
                                                placeholder="about"
                                                prop:value=move || slug.get()
                                                on:input=move |ev| slug.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="space-y-2">
                                        <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Page Type"</label>
                                        <select
                                            class="w-full bg-surface-container-high border-none rounded p-3 text-sm focus:ring-1 focus:ring-primary-dim text-on-surface appearance-none"
                                            on:change=move |ev| page_type.set(event_target_value(&ev))
                                            prop:value=move || page_type.get()
                                        >
                                            <option value="standard">"Standard"</option>
                                            <option value="landing">"Landing Page"</option>
                                            <option value="blog">"Blog Post"</option>
                                        </select>
                                    </div>
                                </div>

                                // Publish toggle
                                <div class="flex items-center gap-3">
                                    <input
                                        type="checkbox"
                                        id="is_published"
                                        class="rounded border-outline-variant bg-surface-container text-primary focus:ring-0"
                                        prop:checked=move || is_published.get()
                                        on:change=move |ev| is_published.set(event_target_checked(&ev))
                                    />
                                    <label for="is_published" class="text-sm text-on-surface cursor-pointer">
                                        "Publish immediately"
                                    </label>
                                </div>

                                // Summary
                                <div class="space-y-2">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Page Description / Summary"</label>
                                    <Textarea rows=2u32 placeholder="A brief description of this page..." bind_value=summary />
                                </div>

                                // Blocks JSON
                                <div class="space-y-2">
                                    <div class="flex justify-between items-end">
                                        <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Blocks Payload (JSON)"</label>
                                        <span class="text-[10px] text-on-surface-variant">"Raw block editor — visual editor coming in Phase 6"</span>
                                    </div>
                                    <Textarea rows=12u32 placeholder="[]" bind_value=blocks_content />
                                </div>

                                // Asset Upload
                                <div class="space-y-2">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Asset Management"</label>
                                    <FileAttachments entity_type="Page".to_string() on_file_drop=Callback::new(handle_file_drop) />
                                </div>
                            </div>
                        </section>

                        // ── Right Pane: Live Preview ──
                        <section class="flex-1 bg-surface-container-lowest p-10 overflow-hidden flex flex-col">
                            <div class="flex justify-between items-center mb-6">
                                <div class="flex items-center gap-3">
                                    <span class="material-symbols-outlined text-secondary">"visibility"</span>
                                    <span class="text-[10px] font-bold uppercase tracking-widest text-secondary">"Page Preview"</span>
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
                                        <div class="font-black text-xl tracking-tighter italic text-slate-900">"YOUR SITE"</div>
                                        <div class="flex gap-8 text-xs font-bold uppercase text-slate-500">
                                            <span>"Home"</span>
                                            <span>"About"</span>
                                            <span>"Contact"</span>
                                        </div>
                                    </nav>
                                    // Content preview
                                    <div class="p-10 max-w-2xl">
                                        <div class="flex gap-2 mb-4">
                                            <span class="px-2 py-0.5 bg-blue-50 text-blue-600 text-[10px] font-bold uppercase rounded">{move || page_type.get()}</span>
                                        </div>
                                        <h2 class="text-2xl font-bold text-slate-800 mb-6">
                                            {move || if title.get().is_empty() { "Untitled Page".to_string() } else { title.get() }}
                                        </h2>
                                        <p class="text-slate-500 leading-relaxed mb-6 italic border-l-4 border-blue-500 pl-4">
                                            {move || if summary.get().is_empty() { "Page description will appear here...".to_string() } else { summary.get() }}
                                        </p>
                                        <div class="text-xs font-mono text-slate-400 bg-slate-50 rounded p-4">
                                            "Slug: /"
                                            {move || slug.get()}
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
