use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BadgeListBlockData {
    pub source: String,
    pub config: BadgeListConfig,
    #[serde(default)]
    pub items: Vec<BadgeItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BadgeListConfig {
    #[serde(default)]
    pub filter_category: Option<String>,
    #[serde(default)]
    pub filter_metadata: Option<String>,
    #[serde(default = "default_3")]
    pub columns: u32,
    #[serde(default = "default_badge")]
    pub display: String,                   // "badge" | "list" | "logo-grid"
    #[serde(default)]
    pub section_title: Option<String>,
}

fn default_3() -> u32 { 3 }
fn default_badge() -> String { "badge".to_string() }

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BadgeItem {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

#[component]
pub fn BadgeListBlock(data: BadgeListBlockData) -> impl IntoView {
    let items_to_render = data.items.clone();

    view! {
        <section class="py-12 md:py-16 w-full">
            <div class="container mx-auto px-4 max-w-5xl">
                {if let Some(ref title) = data.config.section_title {
                    view! {
                        <h2 class="text-3xl font-bold text-on-surface mb-8 text-center md:text-left">
                            {title}
                        </h2>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}

                {if items_to_render.is_empty() {
                    view! {
                        <div class="p-8 border border-outline-variant rounded-xl bg-surface-container flex items-center justify-center text-on-surface-variant">
                            "No badges found."
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class={match data.config.display.as_str() {
                            "logo-grid" => "flex flex-wrap items-center justify-center gap-8 md:gap-12 opacity-80".to_string(),
                            "list" => "flex flex-wrap gap-3".to_string(),
                            _ => format!("grid grid-cols-2 md:grid-cols-{} lg:grid-cols-{} gap-4", 
                                std::cmp::min(data.config.columns, 4), 
                                data.config.columns),
                        }}>
                            {items_to_render.into_iter().map(|item| {
                                view! { <BadgeItemView item=item config=data.config.clone() /> }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }}
            </div>
        </section>
    }
}

#[component]
fn BadgeItemView(item: BadgeItem, config: BadgeListConfig) -> impl IntoView {
    match config.display.as_str() {
        "logo-grid" => view! {
            <div class="flex flex-col items-center justify-center p-4 hover:scale-110 transition-transform cursor-pointer grayscale hover:grayscale-0" title={item.title.clone()}>
                {if let Some(img) = item.icon_url {
                    view! { <img src={img} alt={item.title.clone()} class="h-12 md:h-16 w-auto object-contain" /> }.into_view()
                } else {
                    view! { <span class="font-bold font-mono text-xl">{item.title.clone()}</span> }.into_view()
                }}
            </div>
        }.into_view(),
        "list" => view! {
            <div class="inline-flex items-center space-x-2 bg-surface-container-high border border-outline/20 px-4 py-2 rounded-full text-on-surface hover:bg-primary hover:text-on-primary transition-colors cursor-default shadow-sm hover:shadow">
                {if let Some(img) = item.icon_url {
                    view! { <img src={img} alt="" class="h-4 w-4 object-contain brightness-0 invert" /> }.into_view() // Assume icons are white when hovered, otherwise complex CSS needed
                } else { view! {}.into_view() }}
                <span class="font-medium text-sm whitespace-nowrap">{item.title}</span>
            </div>
        }.into_view(),
        _ => view! {
            <div class="flex items-center p-4 bg-surface-container-low border border-outline-variant hover:border-primary/50 transition-colors rounded-xl shadow-sm h-full group">
                {if let Some(img) = item.icon_url {
                    view! { 
                        <div class="bg-surface p-3 rounded-lg border border-outline-variant/50 mr-4 shadow-sm group-hover:shadow transition-shadow">
                            <img src={img} alt="" class="h-8 w-8 object-contain" />
                        </div>
                    }.into_view()
                } else { 
                    view! {
                        <div class="bg-primary/10 text-primary p-3 rounded-lg mr-4 h-14 w-14 flex items-center justify-center font-bold text-xl group-hover:bg-primary group-hover:text-on-primary transition-colors">
                            {item.title.chars().next().unwrap_or('?').to_uppercase().to_string()}
                        </div>
                    }.into_view()
                }}
                <div>
                    <h3 class="font-bold text-on-surface text-base leading-tight group-hover:text-primary transition-colors">{item.title}</h3>
                    {if let Some(sub) = item.subtitle {
                        view! { <div class="text-xs text-on-surface-variant mt-1">{sub}</div> }.into_view()
                    } else { view! {}.into_view() }}
                </div>
            </div>
        }.into_view() // "badge"
    }
}
