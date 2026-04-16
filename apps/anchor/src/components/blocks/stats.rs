use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StatsBlockData {
    pub source: String,                    // "static" | "live_query"
    pub config: StatsConfig,
    #[serde(default)]
    pub items: Vec<StatItem>,              // used when source = "static"
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StatsConfig {
    #[serde(default)]
    pub query_name: Option<String>,
    #[serde(default = "default_60")]
    pub poll_interval_secs: u64,
    #[serde(default = "default_3")]
    pub columns: u32,
    #[serde(default = "default_metric")]
    pub display: String,                   // "metric" | "ticker" | "chart"
    #[serde(default)]
    pub section_title: Option<String>,
}

fn default_60() -> u64 { 60 }
fn default_3() -> u32 { 3 }
fn default_metric() -> String { "metric".to_string() }

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct StatItem {
    pub label: String,
    pub value: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub trend: Option<String>,             // "up" | "down" | "neutral"
}

#[component]
pub fn StatsBlock(data: StatsBlockData) -> impl IntoView {
    let items_to_render = data.items.clone();

    view! {
        <section class="py-12 md:py-16 w-full">
            <div class="container mx-auto px-4 max-w-6xl">
                {if let Some(ref title) = data.config.section_title {
                    view! {
                        <h2 class="text-3xl font-bold text-on-surface mb-8 text-center">
                            {title}
                        </h2>
                    }.into_view()
                } else { view! {}.into_view() }}

                {if items_to_render.is_empty() {
                    view! {
                        <div class="p-8 border border-outline-variant rounded-xl bg-surface-container flex items-center justify-center text-on-surface-variant">
                            "No metrics available."
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class={match data.config.display.as_str() {
                            "ticker" => "flex flex-wrap items-center justify-center gap-6 md:gap-10".to_string(),
                            _ => format!("grid grid-cols-2 lg:grid-cols-{} gap-4 md:gap-8", 
                                std::cmp::min(data.config.columns, 4)), // metric layout default
                        }}>
                            {items_to_render.into_iter().map(|item| {
                                view! { <StatItemView item=item config=data.config.clone() /> }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }}
            </div>
        </section>
    }
}

#[component]
fn StatItemView(item: StatItem, config: StatsConfig) -> impl IntoView {
    match config.display.as_str() {
        "ticker" => view! {
            <div class="flex items-baseline space-x-2">
                <span class="text-sm font-semibold text-on-surface-variant uppercase tracking-wider">{item.label}</span>
                <span class="text-xl md:text-2xl font-black text-on-surface tabular-nums">{item.value}</span>
                {if let Some(unit) = item.unit {
                    view! { <span class="text-primary font-bold text-sm">{unit}</span> }.into_view()
                } else { view! {}.into_view() }}
                {if let Some(trend) = item.trend {
                    let trend_icon = match trend.as_str() {
                        "up" => "trending_up",
                        "down" => "trending_down",
                        _ => "trending_flat"
                    };
                    let trend_color = match trend.as_str() {
                        "up" => "text-green-500",
                        "down" => "text-red-500",
                        _ => "text-gray-400"
                    };
                    view! { <span class=format!("material-symbols-outlined text-sm {}", trend_color)>{trend_icon}</span> }.into_view()
                } else { view! {}.into_view() }}
            </div>
        }.into_view(),
        _ => view! {
            <div class="bg-surface border border-outline-variant rounded-2xl p-6 md:p-8 flex flex-col items-center justify-center text-center shadow-sm hover:shadow-md transition-shadow">
                {if let Some(icon) = item.icon {
                    view! { <span class="material-symbols-outlined text-4xl text-primary/80 mb-4">{icon}</span> }.into_view()
                } else { view! {}.into_view() }}
                
                <div class="flex items-baseline justify-center gap-1 mb-2">
                    <span class="text-4xl md:text-5xl font-black text-on-surface tabular-nums tracking-tight">{item.value}</span>
                    {if let Some(unit) = item.unit {
                        view! { <span class="text-xl font-bold text-primary ml-1">{unit}</span> }.into_view()
                    } else { view! {}.into_view() }}
                </div>
                
                <h3 class="text-sm md:text-base font-semibold text-on-surface-variant">{item.label}</h3>
                
                {if let Some(trend) = item.trend {
                    let trend_class = match trend.as_str() {
                        "up" => "bg-green-500/10 text-green-600",
                        "down" => "bg-red-500/10 text-red-600",
                        _ => "bg-gray-500/10 text-gray-600"
                    };
                    let trend_icon = match trend.as_str() {
                        "up" => "north_east",
                        "down" => "south_east",
                        _ => "east"
                    };
                    view! { 
                        <div class=format!("mt-3 inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-bold {}", trend_class)>
                            <span class="material-symbols-outlined text-[10px]">{trend_icon}</span>
                            <span>{trend.to_uppercase()}</span>
                        </div>
                    }.into_view()
                } else { view! {}.into_view() }}
            </div>
        }.into_view() // "metric"
    }
}
