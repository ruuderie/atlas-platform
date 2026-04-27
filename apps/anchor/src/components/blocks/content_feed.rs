use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::design_mode::use_kami_mode;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContentFeedBlockData {
    pub source: String,                    // "tenant_pages" | "tenant_entries" | "static"
    pub config: ContentFeedConfig,
    #[serde(default)]
    pub items: Vec<ContentFeedItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContentFeedConfig {
    #[serde(default)]
    pub filter_category: Option<String>,
    #[serde(default)]
    pub filter_page_type: Option<String>,
    #[serde(default = "default_10")]
    pub page_size: u32,
    #[serde(default = "default_cards")]
    pub layout: String,                    // "cards" | "list" | "kami_cards"
    #[serde(default)]
    pub show_tags: bool,
    #[serde(default)]
    pub show_date: bool,
    #[serde(default)]
    pub section_title: Option<String>,
}

fn default_10() -> u32 { 10 }
fn default_cards() -> String { "cards".to_string() }

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ContentFeedItem {
    pub title: String,
    pub slug: String,
    #[serde(default)]
    pub excerpt: Option<String>,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub cover_image_url: Option<String>,
}

#[component]
pub fn ContentFeedBlock(data: ContentFeedBlockData) -> impl IntoView {
    let source = data.source.clone();
    let config = store_value(data.config.clone());
    let fallback_items = data.items.clone();

    let entries_resource = create_resource(
        move || source.clone(),
        move |src| {
            let fallback = fallback_items.clone();
            async move {
                if src == "tenant_entries" || src == "tenant_pages" {
                    if let Ok(entries) = crate::resume_engine::get_tenant_entries(None).await {
                        let filter_cat = config.get_value().filter_category.clone().unwrap_or_default();
                        let mapped: Vec<ContentFeedItem> = entries
                            .into_iter()
                            .filter(|e| filter_cat.is_empty() || e.category.to_string() == filter_cat)
                            .map(|e| {
                                let mut tags = Vec::new();
                                let mut cover_image_url = None;
                                
                                if let Some(meta) = e.metadata {
                                    if let Some(t) = meta.get("tags").and_then(|v| v.as_array()) {
                                        tags = t.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect();
                                    }
                                    if let Some(img) = meta.get("cover_image_url").and_then(|v| v.as_str()) {
                                        cover_image_url = Some(img.to_string());
                                    }
                                }

                                ContentFeedItem {
                                    title: e.title,
                                    slug: e.slug.unwrap_or_else(|| e.id.to_string()),
                                    excerpt: e.subtitle,
                                    published_at: e.published_at.or(e.date_range),
                                    tags,
                                    cover_image_url,
                                }
                            })
                            .collect();
                        if !mapped.is_empty() {
                            return mapped;
                        }
                    }
                }
                fallback
            }
        }
    );

    view! {
        <section class="py-16 w-full">
            <div class="container mx-auto px-4 max-w-6xl">
                {if let Some(ref title) = data.config.section_title {
                    view! {
                        <h2 class="text-3xl font-bold text-on-surface mb-8">
                            {title}
                        </h2>
                    }.into_view()
                } else { view! {}.into_view() }}

                <Suspense fallback=move || view! { <div class="text-sm font-bold uppercase tracking-wider text-outline animate-pulse">"Loading content..."</div> }>
                    {move || {
                        let items_to_render = entries_resource.get().unwrap_or_default();
                        let cfg = config.get_value();
                        
                        if items_to_render.is_empty() {
                            view! {
                                <div class="p-12 border border-outline-variant border-dashed rounded-2xl bg-surface-container-lowest flex items-center justify-center text-on-surface-variant">
                                    "No content available."
                                </div>
                            }.into_view()
                        } else {
                            let is_kami = use_kami_mode() || cfg.layout == "kami_cards";
                            if is_kami {
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                        {items_to_render.into_iter().map(|item| {
                                            view! { <KamiFeedItemView item=item config=cfg.clone() /> }
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            } else {
                                view! {
                                    <div class={match cfg.layout.as_str() {
                                        "list" => "flex flex-col gap-6 max-w-4xl".to_string(),
                                        _ => "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8".to_string()
                                    }}>
                                        {items_to_render.into_iter().map(|item| {
                                            view! { <FeedItemView item=item config=cfg.clone() /> }
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            }
                        }
                    }}
                </Suspense>
            </div>
        </section>
    }
}

#[component]
fn FeedItemView(item: ContentFeedItem, config: ContentFeedConfig) -> impl IntoView {
    match config.layout.as_str() {
        "list" => view! {
            <article class="group relative flex flex-col sm:flex-row gap-6 p-6 bg-surface border border-outline-variant rounded-2xl hover:border-primary transition-all shadow-sm hover:shadow">
                <a href=format!("/e/{}", item.slug) class="absolute inset-0 z-0"></a>
                {if let Some(img) = item.cover_image_url {
                    view! {
                        <div class="w-full sm:w-48 h-48 sm:h-auto shrink-0 rounded-xl overflow-hidden bg-surface-container-high relative z-10 pointer-events-none">
                            <img src={img} alt="" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
                        </div>
                    }.into_view()
                } else { view! {}.into_view() }}
                
                <div class="flex flex-col flex-grow justify-center py-2 relative z-10 pointer-events-none">
                    {if config.show_date {
                        if let Some(date) = item.published_at {
                            view! { <div class="text-xs font-semibold text-primary uppercase tracking-wider mb-2">{date}</div> }.into_view()
                        } else { view! {}.into_view() }
                    } else { view! {}.into_view() }}
                    
                    <h3 class="text-2xl font-bold text-on-surface group-hover:text-primary transition-colors leading-tight mb-3">
                        {item.title}
                    </h3>
                            
                    {if let Some(excerpt) = item.excerpt {
                        view! { <p class="text-on-surface-variant text-base line-clamp-2 md:line-clamp-3 mb-4">{excerpt}</p> }.into_view()
                    } else { view! {}.into_view() }}
                    
                    {if config.show_tags && !item.tags.is_empty() {
                        view! {
                            <div class="flex flex-wrap gap-2 mt-auto">
                                {item.tags.into_iter().map(|tag| view! { <span class="text-xs bg-surface-container px-2.5 py-1 rounded-md text-on-surface-variant">{tag}</span> }).collect_view()}
                            </div>
                        }.into_view()
                    } else { view! {}.into_view() }}
                </div>
            </article>
        }.into_view(),
        _ => view! {
            <article class="group relative flex flex-col bg-surface border border-outline-variant/30 rounded-3xl overflow-hidden hover:shadow-2xl transition-all duration-500 hover:-translate-y-1">
                <a href=format!("/e/{}", item.slug) class="absolute inset-0 z-0"></a>
                {if let Some(img) = item.cover_image_url {
                    view! {
                        <div class="w-full h-56 bg-surface-container-high overflow-hidden relative z-10 pointer-events-none">
                            <img src={img} alt="" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
                        </div>
                    }.into_view()
                } else { view! {}.into_view() }}
                
                <div class="p-6 flex flex-col flex-grow relative z-10 pointer-events-none">
                    {if config.show_date {
                        if let Some(date) = item.published_at {
                            view! { <div class="text-xs font-semibold text-primary uppercase tracking-wider mb-2">{date}</div> }.into_view()
                        } else { view! {}.into_view() }
                    } else { view! {}.into_view() }}
                    
                    <h3 class="text-xl font-bold text-on-surface group-hover:text-primary transition-colors leading-tight mb-3">
                        {item.title}
                    </h3>
                    
                    {if let Some(excerpt) = item.excerpt {
                        view! { <p class="text-on-surface-variant text-sm line-clamp-3 mb-4">{excerpt}</p> }.into_view()
                    } else { view! {}.into_view() }}
                    
                    {if config.show_tags && !item.tags.is_empty() {
                        view! {
                            <div class="flex flex-wrap gap-2 mt-auto pt-4 border-t border-outline-variant/50">
                                {item.tags.into_iter().map(|tag| view! { <span class="text-xs font-medium text-on-surface-variant">{tag}</span> }).collect_view()}
                            </div>
                        }.into_view()
                    } else { view! {}.into_view() }}
                </div>
            </article>
        }.into_view() // "cards"
    }
}

/// Kami parchment project card — used when kami_mode or layout = "kami_cards".
#[component]
fn KamiFeedItemView(item: ContentFeedItem, config: ContentFeedConfig) -> impl IntoView {
    view! {
        <a href=format!("/e/{}", item.slug) class="block no-underline group">
            <article class="bg-[#f5f4ed] border border-[#1B365D]/10 hover:border-[#1B365D]/30 shadow-sm hover:shadow-md transition-all p-7">

                {if let Some(img) = item.cover_image_url {
                    view! {
                        <div class="w-full h-40 overflow-hidden mb-5">
                            <img src={img} alt="" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
                        </div>
                    }.into_view()
                } else { view! {}.into_view() }}

                {if config.show_date {
                    if let Some(date) = item.published_at {
                        view! {
                            <div class="jetbrains text-[0.58rem] uppercase tracking-widest text-[#6b6a64] mb-2">{date}</div>
                        }.into_view()
                    } else { view! {}.into_view() }
                } else { view! {}.into_view() }}

                <h3 class="font-display text-base font-bold text-[#1B365D] leading-snug mb-2 group-hover:text-[#2a4d87] transition-colors">
                    {item.title}
                </h3>

                {if let Some(excerpt) = item.excerpt {
                    view! {
                        <p class="text-[#504e49] text-sm leading-relaxed mb-4 line-clamp-3">{excerpt}</p>
                    }.into_view()
                } else { view! {}.into_view() }}

                {if config.show_tags && !item.tags.is_empty() {
                    view! {
                        <div class="flex flex-wrap gap-1.5 mt-3">
                            {item.tags.into_iter().map(|tag| view! {
                                <span class="border border-[#1B365D]/20 text-[#1B365D] px-2 py-0.5 jetbrains text-[0.55rem] uppercase tracking-wider">
                                    {tag}
                                </span>
                            }).collect_view()}
                        </div>
                    }.into_view()
                } else { view! {}.into_view() }}
            </article>
        </a>
    }
}
