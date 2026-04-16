use leptos::*;
use serde::{Deserialize, Serialize};

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
    pub layout: String,                    // "cards" | "list"
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
    let items_to_render = data.items.clone();

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

                {if items_to_render.is_empty() {
                    view! {
                        <div class="p-12 border border-outline-variant border-dashed rounded-2xl bg-surface-container-lowest flex items-center justify-center text-on-surface-variant">
                            "No content available."
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class={match data.config.layout.as_str() {
                            "list" => "flex flex-col gap-6 max-w-4xl".to_string(), // list layout
                            _ => "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8".to_string() // cards layout
                        }}>
                            {items_to_render.into_iter().map(|item| {
                                view! { <FeedItemView item=item config=data.config.clone() /> }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }}
            </div>
        </section>
    }
}

#[component]
fn FeedItemView(item: ContentFeedItem, config: ContentFeedConfig) -> impl IntoView {
    match config.layout.as_str() {
        "list" => view! {
            <a href=format!("/p/{}", item.slug) class="group flex flex-col sm:flex-row gap-6 p-6 bg-surface border border-outline-variant rounded-2xl hover:border-primary transition-all shadow-sm hover:shadow">
                {if let Some(img) = item.cover_image_url {
                    view! {
                        <div class="w-full sm:w-48 h-48 sm:h-auto shrink-0 rounded-xl overflow-hidden bg-surface-container-high">
                            <img src={img} alt="" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
                        </div>
                    }.into_view()
                } else { view! {}.into_view() }}
                
                <div class="flex flex-col flex-grow justify-center py-2">
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
            </a>
        }.into_view(),
        _ => view! {
            <a href=format!("/p/{}", item.slug) class="group flex flex-col bg-surface border border-outline-variant rounded-2xl overflow-hidden hover:border-primary transition-all shadow-sm hover:shadow">
                {if let Some(img) = item.cover_image_url {
                    view! {
                        <div class="w-full h-56 bg-surface-container-high overflow-hidden">
                            <img src={img} alt="" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
                        </div>
                    }.into_view()
                } else { view! {}.into_view() }}
                
                <div class="p-6 flex flex-col flex-grow">
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
            </a>
        }.into_view() // "cards"
    }
}
