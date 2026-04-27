use leptos::*;
use serde::{Deserialize, Serialize};
use crate::components::design_mode::use_kami_mode;

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

/// Parses a bullet that may contain "Role: ...", "Action: ..." or "Impact: ..." labels.
/// Returns a pair of (label, rest) when a label is detected, otherwise (None, full_text).
fn parse_rai_bullet(b: &str) -> (Option<&str>, &str) {
    for label in ["Role", "Action", "Impact"] {
        if let Some(rest) = b.strip_prefix(&format!("{}: ", label)) {
            return (Some(label), rest);
        }
    }
    (None, b)
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
    let source = data.source.clone();
    let config = store_value(data.config.clone());
    let fallback_items = data.items.clone();

    let entries_resource = create_resource(
        move || source.clone(),
        move |src| {
            let fallback = fallback_items.clone();
            async move {
                if src == "tenant_entries" {
                    if let Ok(entries) = crate::resume_engine::get_tenant_entries(None).await {
                        let filter_cat = config.get_value().filter_category.clone().unwrap_or_default();
                        let mapped: Vec<TimelineItem> = entries
                            .into_iter()
                            .filter(|e| filter_cat.is_empty() || e.category.to_string() == filter_cat)
                            .map(|e| TimelineItem {
                                title: e.title,
                                subtitle: e.subtitle,
                                date_range: e.date_range,
                                bullets: e.bullets,
                                metadata: e.metadata.unwrap_or(serde_json::json!({})),
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
        {if use_kami_mode() {
            // ── Kami parchment timeline ───────────────────────────────────────
            view! {
                <section class="py-16 w-full bg-[#f5f4ed]">
                    <div class="container mx-auto px-4 max-w-3xl">
                        {if let Some(ref title) = data.config.section_title {
                            view! {
                                <div class="mb-10">
                                    <div class="jetbrains text-[0.6rem] uppercase tracking-[0.25em] text-[#6b6a64] mb-2">{title}</div>
                                    <div class="w-12 h-px bg-[#1B365D]/30"></div>
                                </div>
                            }.into_view()
                        } else { view! {}.into_view() }}

                        <Suspense fallback=move || view! { <div class="text-[#6b6a64] jetbrains text-xs uppercase animate-pulse">"Loading..."</div> }>
                            {move || {
                                let items_to_render = entries_resource.get().unwrap_or_default();
                                view! {
                                    <div class="space-y-12">
                                        {items_to_render.into_iter().map(|item| {
                                            view! { <KamiTimelineItemView item=item show_bullets=config.get_value().show_bullets show_date=config.get_value().show_date_range /> }
                                        }).collect_view()}
                                    </div>
                                }
                            }}
                        </Suspense>
                    </div>
                </section>
            }.into_view()
        } else {
            // ── Material 3 dark (existing) ────────────────────────────────────
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

                        <Suspense fallback=move || view! { <div class="text-sm font-bold uppercase tracking-wider text-outline animate-pulse">"Loading timeline..."</div> }>
                            {move || {
                                let items_to_render = entries_resource.get().unwrap_or_default();
                                let cfg = config.get_value();

                                if items_to_render.is_empty() {
                                    view! {
                                        <div class="p-8 border border-outline-variant rounded-xl bg-surface-container flex items-center justify-center text-on-surface-variant">
                                            "No timeline items found."
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {
                                        <div class={match cfg.layout.as_str() {
                                            "cards" => "grid grid-cols-1 md:grid-cols-2 gap-6".to_string(),
                                            "compact" => "space-y-4".to_string(),
                                            _ => "relative border-l-2 border-outline-variant pl-8 space-y-12".to_string(),
                                        }}>
                                            {items_to_render.into_iter().map(|item| {
                                                view! {
                                                    <TimelineItemView item=item config=cfg.clone() />
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_view()
                                }
                            }}
                        </Suspense>
                    </div>
                </section>
            }.into_view()
        }}
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

/// Kami parchment timeline entry with Role/Action/Impact bullet parsing.
#[component]
fn KamiTimelineItemView(item: TimelineItem, show_bullets: bool, show_date: bool) -> impl IntoView {
    view! {
        <div class="border-l-2 border-[#1B365D]/20 pl-6">
            // Title + date header row
            <div class="flex flex-col md:flex-row md:items-baseline md:justify-between gap-2 mb-2">
                <div>
                    <h3 class="font-display text-[1.15rem] font-bold text-[#1B365D] leading-snug">
                        {item.title}
                    </h3>
                    {if let Some(sub) = item.subtitle {
                        view! {
                            <div class="jetbrains text-xs text-[#6b6a64] uppercase tracking-wider mt-0.5">
                                {sub}
                            </div>
                        }.into_view()
                    } else { view! {}.into_view() }}
                </div>
                {if show_date {
                    if let Some(dates) = item.date_range {
                        view! {
                            <span class="jetbrains text-[0.6rem] uppercase tracking-widest text-[#6b6a64] whitespace-nowrap">
                                {dates}
                            </span>
                        }.into_view()
                    } else { view! {}.into_view() }
                } else { view! {}.into_view() }}
            </div>

            // Bullets — detect Role/Action/Impact prefix and render as aligned label+text
            {if show_bullets && !item.bullets.is_empty() {
                view! {
                    <ul class="mt-4 space-y-2.5 list-none pl-0">
                        {item.bullets.into_iter().map(|b| {
                            let (label, rest) = parse_rai_bullet(&b);
                            let label = label.map(str::to_string);
                            let rest = rest.to_string();
                            view! {
                                <li class="text-[#504e49] text-sm leading-[1.75] flex gap-2">
                                    {if let Some(lbl) = label {
                                        view! {
                                            <span class="jetbrains text-[0.58rem] uppercase tracking-widest text-[#1B365D] font-bold mt-[0.2rem] shrink-0 w-14 text-right">
                                                {lbl}":"
                                            </span>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <span class="text-[#1B365D]/40 mt-[0.3rem] shrink-0">"—"</span>
                                        }.into_view()
                                    }}
                                    <span>{rest}</span>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                }.into_view()
            } else { view! {}.into_view() }}
        </div>
    }
}
