use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::file_attachments::FileAttachments;
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::textarea::Textarea;
use shared_ui::components::ui::label::Label;
use shared_ui::components::ui::button::{Button, ButtonVariant};
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
    
    // Default directory and profile for MVP creation
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
        <div class="w-full max-w-[1600px] mx-auto space-y-8 p-6">
            <header class="flex justify-between items-end mb-6">
                <div class="space-y-2">
                    <h2 class="text-3xl font-bold tracking-tight">"CMS Content Manager"</h2>
                    <p class="text-muted-foreground text-lg">"Manage your organization's publications and rich text content."</p>
                </div>
            </header>

            <Card class="p-6 bg-card border border-border flex flex-col min-h-[600px]".to_string()>
                <Tabs default_value=default_t>
                    <div class="flex justify-between items-center mb-6">
                        <TabsList class="inline-flex h-9 items-center justify-center rounded-md bg-muted p-1 text-muted-foreground self-start".to_string()>
                            <TabButton label="All Articles" value="articles" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                            <TabButton label="Editor" value="editor" active_value=Signal::derive(|| "".to_string()) on_select=move |_| {} />
                        </TabsList>
                        
                        <Button class="!bg-[var(--color-accent-primary)] !text-[#0f172a]".to_string()>
                            "+ New Article"
                        </Button>
                    </div>

                    <TabsContent value="articles".to_string()>
                        <div class="border border-border/50 rounded-md">
                            <Suspense fallback=move || view! { <div class="p-4 text-muted-foreground">"Loading..."</div> }>
                                <DataTable headers=article_headers.clone() data=article_data />
                            </Suspense>
                        </div>
                    </TabsContent>

                    <TabsContent value="editor".to_string()>
                        <div class="flex flex-col lg:flex-row gap-6 min-h-[700px] h-full">
                            {/* Editor Pane */}
                            <div class="flex-1 bg-card border border-border rounded-xl p-6 overflow-y-auto space-y-6">
                                <div class="flex justify-between items-center pb-4 border-b border-border">
                                    <h3 class="text-lg font-medium">"Draft Editor"</h3>
                                    <div class="flex space-x-2">
                                        <Button variant=ButtonVariant::Outline>"Save Draft"</Button>
                                        <Button on:click=handle_publish class="!bg-[var(--color-accent-primary)] !text-[#0f172a] !border-[var(--color-accent-primary)]".to_string()>"Publish"</Button>
                                    </div>
                                </div>

                                <div class="space-y-4">
                                    <div class="grid gap-2">
                                        <Label>"Title"</Label>
                                        <Input r#type=InputType::Text placeholder="Enter title..." bind_value=title />
                                    </div>

                                    <div class="grid gap-2">
                                        <Label>"Content Type"</Label>
                                        <select 
                                            class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                                            on:change=move |ev| listing_type.set(event_target_value(&ev))
                                            prop:value=listing_type
                                        >
                                            <option value="article">"Article / Blog Post"</option>
                                            <option value="landing_page">"Marketing Landing Page"</option>
                                        </select>
                                    </div>

                                    <div class="grid grid-cols-2 gap-4">
                                        <div class="grid gap-2">
                                            <Label>"URL Slug"</Label>
                                            <Input r#type=InputType::Text placeholder="/blog/my-article" bind_value=url />
                                        </div>
                                        <div class="grid gap-2">
                                            <Label>"Author Name"</Label>
                                            <Input r#type=InputType::Text placeholder="Author Name" bind_value=author_name />
                                        </div>
                                    </div>
                                    
                                    <div class="grid gap-2">
                                        <Label>"Summary"</Label>
                                        <Textarea rows=3u32 placeholder="Brief description..." bind_value=summary />
                                    </div>
                                    
                                    <div class="grid gap-2">
                                        <Label>"Content (HTML or Markdown)"</Label>
                                        <Textarea rows=10u32 placeholder="Write your content here..." bind_value=content_html />
                                    </div>

                                    {move || if listing_type.get() == "landing_page" {
                                        view! {
                                            <div class="grid gap-2 mt-2 pt-4 border-t border-border">
                                                <h4 class="text-sm font-medium text-muted-foreground">"Landing Page Configuration"</h4>
                                                <div class="grid gap-2 mt-2">
                                                    <Label>"Hero Headline"</Label>
                                                    <Input r#type=InputType::Text placeholder="e.g. Get 50% Off Today!" bind_value=hero_headline />
                                                </div>
                                                <div class="grid gap-2">
                                                    <Label>"Call-to-Action (CTA) Button Text"</Label>
                                                    <Input r#type=InputType::Text placeholder="e.g. Subscribe Now" bind_value=cta_text />
                                                </div>
                                                <div class="grid grid-cols-2 gap-4 mt-2 border-t border-border pt-4">
                                                    <div class="flex items-center space-x-2">
                                                        <input type="checkbox" id="show_price" prop:checked=show_price on:change=move |ev| show_price.set(event_target_checked(&ev)) class="h-4 w-4 rounded border-gray-300 text-primary" />
                                                        <Label>"Show Pricing Table"</Label>
                                                    </div>
                                                    <div class="flex items-center space-x-2">
                                                        <input type="checkbox" id="show_map" prop:checked=show_map on:change=move |ev| show_map.set(event_target_checked(&ev)) class="h-4 w-4 rounded border-gray-300 text-primary" />
                                                        <Label>"Show Interactive Map"</Label>
                                                    </div>
                                                </div>
                                                <div class="grid grid-cols-2 gap-4 mt-2">
                                                    <div class="grid gap-2">
                                                        <Label>"WhatsApp Number"</Label>
                                                        <Input r#type=InputType::Text placeholder="+1234567890" bind_value=whatsapp_number />
                                                    </div>
                                                    <div class="grid gap-2">
                                                        <Label>"Form Theme"</Label>
                                                        <select class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
                                                                on:change=move |ev| form_theme.set(event_target_value(&ev))
                                                                prop:value=form_theme>
                                                            <option value="light">"Light Layout"</option>
                                                            <option value="dark">"Dark Layout"</option>
                                                            <option value="brand">"Brand Contrast"</option>
                                                        </select>
                                                    </div>
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <span/> }.into_any()
                                    }}

                                    <div class="pt-4 border-t border-border mt-4">
                                        <FileAttachments entity_type="Item".to_string() on_file_drop=Callback::new(handle_file_drop) />
                                    </div>
                                </div>
                            </div>

                            {/* Live Preview Pane */}
                            <div class="w-full lg:w-[450px] shrink-0 bg-muted/20 border border-border rounded-xl p-6 overflow-y-auto flex flex-col space-y-4">
                                <h4 class="text-sm font-semibold tracking-widest text-muted-foreground uppercase">"Live Preview"</h4>
                                <div class="bg-background rounded-lg border border-border p-6 shadow-sm min-h-[500px] prose prose-sm dark:prose-invert max-w-none">
                                    <h1 class="text-2xl font-bold mb-2">
                                        {move || if title.get().is_empty() { "Untitled Article".to_string() } else { title.get() }}
                                    </h1>
                                    <div class="flex space-x-4 text-xs text-muted-foreground mb-6">
                                        <span><strong>"By:"</strong> " " {move || if author_name.get().is_empty() { "Unknown".to_string() } else { author_name.get() }}</span>
                                        <span><strong>"URL:"</strong> " " {move || url.get()}</span>
                                    </div>
                                    <p class="text-base text-muted-foreground mb-6 italic border-l-2 border-primary pl-4">
                                        {move || summary.get()}
                                    </p>
                                    <div class="mt-6" inner_html=move || {
                                        content_html.get().replace('\n', "<br/>")
                                    }>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </TabsContent>
                </Tabs>
            </Card>
        </div>
    }
}
