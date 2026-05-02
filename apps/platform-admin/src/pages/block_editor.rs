use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── Block type discriminant ───────────────────────────────────────────────────

/// The canonical block type discriminant used in `blocks_payload`.
/// Must match the `type` field in the JSON array stored in `app_pages.blocks_payload`.
///
/// Compatible with Anchor's existing seed format — same field names and
/// field aliases as in `anchor/src/components/blocks/`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Block {
    Hero(HeroData),
    Grid(GridData),
    Callout(CalloutData),
    RichText(RichTextData),
    Stats(StatsData),
    RawHtml(RawHtmlData),
    #[serde(other)]
    Unknown,
}

// ── Block data types (mirrors anchor block schemas) ───────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct HeroData {
    #[serde(alias = "heading", default)]
    pub title: String,
    #[serde(alias = "subheading", default)]
    pub subtitle: String,
    #[serde(default)]
    pub cta_text: Option<String>,
    #[serde(default)]
    pub cta_link: Option<String>,
    #[serde(default)]
    pub layout: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct GridItem {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub link_url: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct GridData {
    #[serde(default)]
    pub section_title: Option<String>,
    #[serde(default)]
    pub columns: Option<u32>,
    #[serde(default)]
    pub items: Vec<GridItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct CalloutData {
    #[serde(alias = "text", default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub style: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct RichTextData {
    #[serde(default)]
    pub html: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct StatItem {
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub value: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct StatsData {
    #[serde(default)]
    pub section_title: Option<String>,
    #[serde(default)]
    pub items: Vec<StatItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct RawHtmlData {
    #[serde(default)]
    pub html: String,
}

// ── Parse helper ─────────────────────────────────────────────────────────────

/// Parses a `blocks_payload` JSON string into a `Vec<Block>`.
/// Returns an empty vec on parse failure — editor shows "invalid JSON" warning.
pub fn parse_blocks(json: &str) -> (Vec<Block>, Option<String>) {
    if json.trim().is_empty() || json.trim() == "[]" {
        return (vec![], None);
    }
    match serde_json::from_str::<Vec<serde_json::Value>>(json) {
        Ok(values) => {
            let blocks = values
                .into_iter()
                .map(|v| serde_json::from_value::<Block>(v).unwrap_or(Block::Unknown))
                .collect();
            (blocks, None)
        }
        Err(e) => (vec![], Some(format!("JSON parse error: {e}"))),
    }
}

// ── Block preview renderer ────────────────────────────────────────────────────

/// Admin-facing block preview card.
/// Renders a lightweight summary of each block type — NOT the full Anchor renderer.
/// Full rendering happens on the live site via AnchorApp.
#[component]
pub fn BlockPreview(block: Block, index: usize) -> impl IntoView {
    match block {
        Block::Hero(d) => view! {
            <div class="border border-primary/20 bg-surface-container-high rounded-lg p-4 space-y-1">
                <div class="flex items-center gap-2 mb-2">
                    <span class="material-symbols-outlined text-primary text-sm">"web_asset"</span>
                    <span class="text-[10px] font-bold uppercase tracking-wider text-primary">"Hero Block #"{index + 1}</span>
                </div>
                <p class="text-sm font-bold text-on-surface">{d.title}</p>
                <p class="text-xs text-on-surface-variant">{d.subtitle}</p>
                {d.cta_text.map(|cta| view! {
                    <span class="inline-block px-2 py-0.5 bg-primary/10 text-primary text-[10px] rounded font-bold mt-1">{cta}</span>
                })}
            </div>
        }.into_any(),

        Block::Grid(d) => view! {
            <div class="border border-secondary/20 bg-surface-container-high rounded-lg p-4 space-y-2">
                <div class="flex items-center gap-2 mb-2">
                    <span class="material-symbols-outlined text-secondary text-sm">"grid_view"</span>
                    <span class="text-[10px] font-bold uppercase tracking-wider text-secondary">"Grid Block #"{index + 1}</span>
                    <span class="ml-auto text-[10px] text-on-surface-variant">{d.items.len()}" items"</span>
                </div>
                {d.section_title.map(|t| view! {
                    <p class="text-sm font-bold text-on-surface">{t}</p>
                })}
                <div class="grid grid-cols-3 gap-2 pt-1">
                    {d.items.iter().take(6).map(|item| view! {
                        <div class="bg-surface-container rounded p-2 text-center">
                            <p class="text-[10px] font-bold text-on-surface truncate">{item.title.clone()}</p>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        }.into_any(),

        Block::Callout(d) => view! {
            <div class="border border-tertiary/20 bg-surface-container-high rounded-lg p-4">
                <div class="flex items-center gap-2 mb-2">
                    <span class="material-symbols-outlined text-tertiary text-sm">"campaign"</span>
                    <span class="text-[10px] font-bold uppercase tracking-wider text-tertiary">"Callout Block #"{index + 1}</span>
                </div>
                <p class="text-sm font-bold text-on-surface">{d.title}</p>
                <p class="text-xs text-on-surface-variant mt-1">{d.description}</p>
            </div>
        }.into_any(),

        Block::RichText(d) => view! {
            <div class="border border-outline-variant/30 bg-surface-container-high rounded-lg p-4">
                <div class="flex items-center gap-2 mb-2">
                    <span class="material-symbols-outlined text-on-surface-variant text-sm">"article"</span>
                    <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">"Rich Text #"{index + 1}</span>
                </div>
                <div class="text-xs text-on-surface-variant font-mono line-clamp-3">
                    {d.html.chars().take(120).collect::<String>()}
                </div>
            </div>
        }.into_any(),

        Block::Stats(d) => view! {
            <div class="border border-primary/20 bg-surface-container-high rounded-lg p-4">
                <div class="flex items-center gap-2 mb-2">
                    <span class="material-symbols-outlined text-primary text-sm">"bar_chart"</span>
                    <span class="text-[10px] font-bold uppercase tracking-wider text-primary">"Stats Block #"{index + 1}</span>
                    <span class="ml-auto text-[10px] text-on-surface-variant">{d.items.len()}" stats"</span>
                </div>
                {d.section_title.map(|t| view! { <p class="text-sm font-bold text-on-surface">{t}</p> })}
                <div class="flex gap-3 pt-2">
                    {d.items.iter().take(4).map(|s| view! {
                        <div class="bg-surface-container rounded p-2 flex-1 text-center">
                            <p class="text-base font-black text-primary">{s.value.clone()}</p>
                            <p class="text-[9px] text-on-surface-variant uppercase">{s.label.clone()}</p>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        }.into_any(),

        Block::RawHtml(_) => view! {
            <div class="border border-error/20 bg-surface-container-high rounded-lg p-4">
                <div class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-error text-sm">"code"</span>
                    <span class="text-[10px] font-bold uppercase tracking-wider text-error">"Raw HTML #"{index + 1}</span>
                    <span class="ml-auto text-[10px] text-on-surface-variant italic">"rendered on-site only"</span>
                </div>
            </div>
        }.into_any(),

        Block::Unknown => view! {
            <div class="border border-outline-variant/30 bg-surface-container rounded-lg p-4">
                <div class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-on-surface-variant text-sm">"help_outline"</span>
                    <span class="text-[10px] font-bold uppercase tracking-wider text-on-surface-variant">"Unknown Block #"{index + 1}</span>
                </div>
            </div>
        }.into_any(),
    }
}

// ── Block palette (for the add-block UI) ──────────────────────────────────────

#[derive(Clone)]
pub struct BlockTemplate {
    pub label: &'static str,
    pub icon: &'static str,
    pub json: &'static str,
}

pub fn block_templates() -> Vec<BlockTemplate> {
    vec![
        BlockTemplate {
            label: "Hero",
            icon: "web_asset",
            json: r#"{"type":"hero","title":"Your Headline","subtitle":"A compelling subheading","cta_text":"Get Started","cta_link":"/"}"#,
        },
        BlockTemplate {
            label: "Grid",
            icon: "grid_view",
            json: r#"{"type":"grid","section_title":"Features","columns":3,"items":[{"title":"Feature 1","description":"Description here","icon":"star"},{"title":"Feature 2","description":"Description here","icon":"bolt"},{"title":"Feature 3","description":"Description here","icon":"shield"}]}"#,
        },
        BlockTemplate {
            label: "Callout",
            icon: "campaign",
            json: r#"{"type":"callout","title":"Important Message","description":"Supporting detail text."}"#,
        },
        BlockTemplate {
            label: "Rich Text",
            icon: "article",
            json: r#"{"type":"rich_text","html":"<p>Your content here.</p>"}"#,
        },
        BlockTemplate {
            label: "Stats",
            icon: "bar_chart",
            json: r#"{"type":"stats","section_title":"By the numbers","items":[{"label":"Users","value":"10K+"},{"label":"Uptime","value":"99.9%"}]}"#,
        },
        BlockTemplate {
            label: "Raw HTML",
            icon: "code",
            json: r#"{"type":"raw_html","html":"<!-- custom embed -->"}"#,
        },
    ]
}
