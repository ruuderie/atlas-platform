use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimelineBlockData {
    pub source: String,                    // "static" | "tenant_entries"
    pub config: TimelineBlockConfig,
    #[serde(default)]
    pub items: Vec<TimelineItem>,          // used when source = "static"
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimelineBlockConfig {
    #[serde(default)]
    pub filter_category: Option<String>,   // e.g. "work", "project"
    #[serde(default)]
    pub filter_metadata: Option<String>,   // e.g. "is_client_project=true"
    #[serde(default)]
    pub show_date_range: bool,
    #[serde(default)]
    pub show_bullets: bool,
    #[serde(default = "default_layout")]
    pub layout: String,                    // "detailed" | "compact" | "cards"
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub section_title: Option<String>,
}

fn default_layout() -> String {
    "detailed".to_string()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct TimelineItem {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub date_range: Option<String>,
    #[serde(default)]
    pub bullets: Vec<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[component]
pub fn TimelineBlock(data: TimelineBlockData) -> impl IntoView {
    // For Phase 2, we will only render the static items passed in, or a placeholder if DB fetch is needed.
    // Full DB fetch integration via Suspense will be hooked up after the components are built.
    
    // Fall back to items or render empty if source is empty.
    let items_to_render = data.items.clone();

    view! {
        <section class="py-12 md:py-16 w-full">
            <div class="container mx-auto px-4 max-w-4xl">
                {if let Some(ref title) = data.config.section_title {
                    view! {
                        <h2 class="text-3xl font-bold text-on-surface mb-8">
                            {title}
                        </h2>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}

                {if items_to_render.is_empty() {
                    view! {
                        <div class="p-8 border border-outline-variant rounded-xl bg-surface-container flex items-center justify-center text-on-surface-variant">
                            "No timeline items found."
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class={match data.config.layout.as_str() {
                            "cards" => "grid grid-cols-1 md:grid-cols-2 gap-6".to_string(),
                            "compact" => "space-y-4".to_string(),
                            _ => "relative border-l-2 border-outline-variant pl-8 space-y-12".to_string(),
                        }}>
                            {items_to_render.into_iter().map(|item| {
                                view! {
                                    <TimelineItemView item=item config=data.config.clone() />
                                }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }}
            </div>
        </section>
    }
}

#[component]
fn TimelineItemView(item: TimelineItem, config: TimelineBlockConfig) -> impl IntoView {
    match config.layout.as_str() {
        "cards" => view! {
            <div class="bg-surface border border-outline-variant hover:border-primary/50 transition-colors rounded-xl p-6 shadow-sm">
                <h3 class="text-xl font-bold text-on-surface mb-1">{item.title}</h3>
                {if let Some(sub) = item.subtitle {
                    view! { <div class="text-primary font-medium text-sm mb-3">{sub}</div> }.into_view()
                } else { view! {}.into_view() }}
                
                {if config.show_date_range {
                    if let Some(dates) = item.date_range {
                        view! {
                            <div class="text-xs text-on-surface-variant uppercase tracking-wider mb-4">{dates}</div>
                        }.into_view()
                    } else { view! {}.into_view() }
                } else { view! {}.into_view() }}

                {if config.show_bullets && !item.bullets.is_empty() {
                    view! {
                        <ul class="text-on-surface-variant text-sm space-y-2 !list-none pl-0">
                            {item.bullets.into_iter().map(|b| view! { <li><span class="text-primary mr-2">"•"</span>{b}</li> }).collect_view()}
                        </ul>
                    }.into_view()
                } else { view! {}.into_view() }}
            </div>
        }.into_view(),
        "compact" => view! {
            <div class="flex flex-col sm:flex-row sm:items-baseline sm:justify-between py-3 border-b border-outline-variant/30 last:border-0 hover:bg-surface-container/30 px-4 -mx-4 rounded">
                <div class="flex-grow">
                    <h3 class="text-lg font-bold text-on-surface flex items-center gap-2">
                        {item.title}
                        {if let Some(sub) = item.subtitle {
                            view! { <span class="font-normal text-on-surface-variant text-base">"— "{sub}</span> }.into_view()
                        } else { view! {}.into_view() }}
                    </h3>
                </div>
                {if config.show_date_range {
                    if let Some(dates) = item.date_range {
                        view! {
                            <div class="text-sm font-medium text-on-surface-variant whitespace-nowrap mt-1 sm:mt-0 opacity-80">{dates}</div>
                        }.into_view()
                    } else { view! {}.into_view() }
                } else { view! {}.into_view() }}
            </div>
        }.into_view(),
        _ => view! {
            <div class="relative">
                <div class="absolute -left-[41px] top-1 h-5 w-5 rounded-full border-4 border-surface bg-primary shadow-sm z-10" />
                <div class="mb-2 flex flex-col md:flex-row md:items-baseline md:justify-between">
                    <div>
                        <h3 class="text-2xl font-bold text-on-surface">{item.title}</h3>
                        {if let Some(sub) = item.subtitle {
                            view! { <div class="text-primary font-medium text-lg tracking-wide">{sub}</div> }.into_view()
                        } else { view! {}.into_view() }}
                    </div>
                    {if config.show_date_range {
                        if let Some(dates) = item.date_range {
                            view! {
                                <div class="text-sm font-semibold text-on-surface-variant uppercase tracking-wider bg-surface-container px-3 py-1 rounded-full w-fit mt-2 md:mt-0 border border-outline-variant/40">
                                    {dates}
                                </div>
                            }.into_view()
                        } else { view! {}.into_view() }
                    } else { view! {}.into_view() }}
                </div>
                {if config.show_bullets && !item.bullets.is_empty() {
                    view! {
                        <div class="prose prose-on-surface max-w-none prose-p:my-1 prose-li:my-1 mt-4 text-on-surface-variant">
                            <ul>
                                {item.bullets.into_iter().map(|b| view! { <li>{b}</li> }).collect_view()}
                            </ul>
                        </div>
                    }.into_view()
                } else { view! {}.into_view() }}
            </div>
        }.into_view() // "detailed"
    }
}
