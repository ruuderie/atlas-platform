use leptos::prelude::*;
use shared_ui::components::file_attachments::FileAttachments;
use shared_ui::components::ui::textarea::Textarea;
use shared_ui::components::tabs::{Tabs, TabButton};
use shared_ui::components::ui::tabs::{TabsContent, TabsList};
use shared_ui::components::data_table::DataTable;
use leptos_router::hooks::use_query_map;

// Phase 5+6: uses app_pages CRUD and block-aware visual editor
use crate::api::pages::{list_pages, create_page, PageSummary, CreatePagePayload};
use crate::api::files::create_file;
use crate::api::models::CreateFileInput;
use crate::pages::block_editor::{parse_blocks, BlockPreview, block_templates};

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
    let blocks_json = RwSignal::new("[]".to_string());
    let page_type = RwSignal::new("standard".to_string());
    let is_published = RwSignal::new(false);
    let show_raw_json = RwSignal::new(false);

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

    // ── Derived: live block parse ─────────────────────────────────────────────
    // Re-parses blocks_json on every keystroke to drive the right-pane preview.
    let parsed_blocks = Signal::derive(move || {
        let (blocks, err) = parse_blocks(&blocks_json.get());
        (blocks, err)
    });

    // ── Publish handler ───────────────────────────────────────────────────────
    let handle_publish = move |_| {
        leptos::task::spawn_local(async move {
            let blocks_value: Option<serde_json::Value> =
                serde_json::from_str(&blocks_json.get()).ok();

            let payload = CreatePagePayload {
                slug: slug.get(),
                title: title.get(),
                description: summary.get(),
                page_type: Some(page_type.get()),
                hero_payload: None,
                blocks_payload: blocks_value,
                is_published: Some(is_published.get()),
            };

            match create_page(tenant_id, payload).await {
                Ok(_) => {
                    set_trigger_fetch.update(|v| *v += 1);
                    title.set("".to_string());
                    slug.set("".to_string());
                    summary.set("".to_string());
                    blocks_json.set("[]".to_string());
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

                                // ── Blocks Editor ────────────────────────────────────────────────
                                <div class="space-y-4">
                                    <div class="flex justify-between items-center">
                                        <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Page Blocks"</label>
                                        <button
                                            class="text-[10px] text-on-surface-variant hover:text-primary transition-colors"
                                            on:click=move |_| show_raw_json.update(|v| *v = !*v)
                                        >
                                            {move || if show_raw_json.get() { "← Visual" } else { "{ } Raw JSON" }}
                                        </button>
                                    </div>

                                    // Block Palette
                                    {move || if !show_raw_json.get() {
                                        let templates = block_templates();
                                        view! {
                                            <div class="space-y-3">
                                                // Add block buttons
                                                <div class="grid grid-cols-3 gap-2">
                                                    {templates.into_iter().map(|tmpl| {
                                                        let json_snippet = tmpl.json.to_string();
                                                        view! {
                                                            <button
                                                                class="flex flex-col items-center gap-1 p-3 bg-surface-container-high hover:bg-surface-bright/50 rounded-lg border border-outline-variant/20 hover:border-primary/30 transition-all group text-center"
                                                                on:click=move |_| {
                                                                    let current = blocks_json.get();
                                                                    let trimmed = current.trim();
                                                                    let new_json = if trimmed == "[]" || trimmed.is_empty() {
                                                                        format!("[{}]", json_snippet)
                                                                    } else {
                                                                        // Insert before the closing ]
                                                                        let without_bracket = trimmed.trim_end_matches(']').trim_end_matches(',');
                                                                        format!("{},{}]", without_bracket, json_snippet)
                                                                    };
                                                                    blocks_json.set(new_json);
                                                                }
                                                            >
                                                                <span class="material-symbols-outlined text-on-surface-variant group-hover:text-primary text-lg transition-colors">{tmpl.icon}</span>
                                                                <span class="text-[9px] font-bold uppercase tracking-wider text-on-surface-variant group-hover:text-on-surface">{tmpl.label}</span>
                                                            </button>
                                                        }
                                                    }).collect_view()}
                                                </div>

                                                // Parse error banner
                                                {move || {
                                                    let (_, err) = parsed_blocks.get();
                                                    err.map(|e| view! {
                                                        <div class="flex items-center gap-2 px-3 py-2 bg-error-container text-error rounded text-xs">
                                                            <span class="material-symbols-outlined text-sm">"error"</span>
                                                            {e}
                                                        </div>
                                                    })
                                                }}

                                                // Block stack preview
                                                <div class="space-y-2">
                                                    {move || {
                                                        let (blocks, _) = parsed_blocks.get();
                                                        if blocks.is_empty() {
                                                            view! {
                                                                <div class="text-center py-8 border-2 border-dashed border-outline-variant/20 rounded-lg">
                                                                    <span class="material-symbols-outlined text-on-surface-variant text-3xl">"view_carousel"</span>
                                                                    <p class="text-xs text-on-surface-variant mt-2">"Click a block type above to add it to this page"</p>
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="space-y-2">
                                                                    {blocks.into_iter().enumerate().map(|(i, block)| view! {
                                                                        <BlockPreview block=block index=i />
                                                                    }).collect_view()}
                                                                </div>
                                                            }.into_any()
                                                        }
                                                    }}
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <Textarea rows=10u32 placeholder="[]" bind_value=blocks_json />
                                        }.into_any()
                                    }}
                                </div>

                                // Asset Upload
                                <div class="space-y-2">
                                    <label class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Asset Management"</label>
                                    <FileAttachments entity_type="Page".to_string() on_file_drop=Callback::new(handle_file_drop) />
                                </div>
                            </div>
                        </section>

                        // ── Right Pane: Block Structure Preview ──
                        <section class="flex-1 bg-surface-container-lowest p-10 overflow-hidden flex flex-col">
                            <div class="flex justify-between items-center mb-6">
                                <div class="flex items-center gap-3">
                                    <span class="material-symbols-outlined text-secondary">"view_carousel"</span>
                                    <span class="text-[10px] font-bold uppercase tracking-widest text-secondary">"Block Stack Preview"</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    {move || {
                                        let (blocks, _) = parsed_blocks.get();
                                        let count = blocks.len();
                                        view! {
                                            <span class="text-[10px] text-on-surface-variant">{count}" blocks"</span>
                                        }
                                    }}
                                </div>
                            </div>

                            // Page meta summary card
                            <div class="bg-surface-container rounded-xl p-4 mb-4 border border-outline-variant/10">
                                <div class="flex justify-between items-start mb-2">
                                    <div>
                                        <p class="text-sm font-bold text-on-surface">
                                            {move || if title.get().is_empty() { "Untitled Page".to_string() } else { title.get() }}
                                        </p>
                                        <p class="text-[10px] text-on-surface-variant font-mono mt-0.5">"/"
                                            {move || slug.get()}
                                        </p>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <span class="px-2 py-0.5 bg-primary/10 text-primary text-[9px] font-bold rounded uppercase">
                                            {move || page_type.get()}
                                        </span>
                                        {move || if is_published.get() {
                                            view! {
                                                <span class="px-2 py-0.5 bg-tertiary/10 text-tertiary text-[9px] font-bold rounded uppercase">"Published"</span>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <span class="px-2 py-0.5 bg-surface-container-high text-on-surface-variant text-[9px] font-bold rounded uppercase">"Draft"</span>
                                            }.into_any()
                                        }}
                                    </div>
                                </div>
                                <p class="text-xs text-on-surface-variant italic line-clamp-2">
                                    {move || if summary.get().is_empty() { "No description set".to_string() } else { summary.get() }}
                                </p>
                            </div>

                            // Block preview list (scrollable)
                            <div class="flex-1 overflow-y-auto space-y-3">
                                {move || {
                                    let (blocks, err) = parsed_blocks.get();
                                    if let Some(e) = err {
                                        view! {
                                            <div class="flex flex-col items-center justify-center h-full text-center">
                                                <span class="material-symbols-outlined text-error text-3xl mb-2">"error"</span>
                                                <p class="text-xs text-error font-mono">{e}</p>
                                                <p class="text-[10px] text-on-surface-variant mt-2">"Switch to Raw JSON tab to fix the syntax."</p>
                                            </div>
                                        }.into_any()
                                    } else if blocks.is_empty() {
                                        view! {
                                            <div class="flex flex-col items-center justify-center h-full text-center">
                                                <span class="material-symbols-outlined text-on-surface-variant text-4xl mb-3">"layers"</span>
                                                <p class="text-sm font-bold text-on-surface">"No blocks yet"</p>
                                                <p class="text-xs text-on-surface-variant mt-1">"Add blocks from the palette on the left."</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="space-y-3">
                                                {blocks.into_iter().enumerate().map(|(i, block)| view! {
                                                    <BlockPreview block=block index=i />
                                                }).collect_view()}
                                            </div>
                                        }.into_any()
                                    }
                                }}
                            </div>
                        </section>
                    </div>
                </TabsContent>
            </Tabs>
        </div>
    }
}
