use leptos::prelude::*;
use serde_json::Value;

pub fn has_cms_blocks(blocks: &Value) -> bool {
    match blocks {
        Value::Array(items) => !items.is_empty(),
        Value::Object(map) => map
            .get("blocks")
            .and_then(Value::as_array)
            .map(|items| !items.is_empty())
            .unwrap_or(!map.is_empty()),
        _ => false,
    }
}

fn block_items(blocks: &Value) -> Vec<Value> {
    match blocks {
        Value::Array(items) => items.clone(),
        Value::Object(map) => map
            .get("blocks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn field(block: &Value, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| block.get(*key).and_then(Value::as_str))
        .unwrap_or_default()
        .to_string()
}

fn string_list(block: &Value, key: &str) -> Vec<String> {
    block
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.as_str().map(str::to_string).or_else(|| {
                        item.get("title")
                            .and_then(Value::as_str)
                            .map(str::to_string)
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[component]
pub fn BlockRenderer(hero: Value, blocks: Value) -> impl IntoView {
    let mut items = block_items(&blocks);
    if items.is_empty() && hero.is_object() {
        let mut hero_block = hero.clone();
        if let Some(obj) = hero_block.as_object_mut() {
            obj.entry("type".to_string())
                .or_insert(Value::String("hero".to_string()));
        }
        items.push(hero_block);
    }

    view! {
        <div class="folio-mktg cms-blocks">
            {items.into_iter().filter_map(|block| {
                let block_type = field(&block, &["type", "kind"]);
                match block_type.as_str() {
                    "hero" => {
                        let eyebrow = field(&block, &["eyebrow", "kicker"]);
                        let title = field(&block, &["title", "headline", "heading"]);
                        let subtitle = field(&block, &["subtitle", "subheadline", "body", "description"]);
                        let cta_label = field(&block, &["cta_label", "cta"]);
                        let cta_href = field(&block, &["cta_href", "href", "url"]);
                        Some(view! {
                            <section class="mktg-hero">
                                <div class="mktg-hero-grid-overlay"></div>
                                <div class="mktg-hero-inner">
                                    {(!eyebrow.is_empty()).then(|| view! { <div class="mktg-eyebrow">{eyebrow}</div> })}
                                    {(!title.is_empty()).then(|| view! { <h1 class="mktg-hero-h1">{title}</h1> })}
                                    {(!subtitle.is_empty()).then(|| view! { <p class="mktg-hero-sub">{subtitle}</p> })}
                                    {(!cta_label.is_empty()).then(|| view! {
                                        <a class="mktg-btn-accent mktg-btn-lg" href=if cta_href.is_empty() { "#waitlist-wrap".to_string() } else { cta_href }>
                                            {cta_label}
                                        </a>
                                    })}
                                </div>
                            </section>
                        }.into_any())
                    }
                    "features" => {
                        let title = field(&block, &["title", "headline", "heading"]);
                        let subtitle = field(&block, &["subtitle", "body", "description"]);
                        let features = string_list(&block, "items");
                        Some(view! {
                            <section class="mktg-section mktg-features">
                                <div class="mktg-section-inner">
                                    {(!title.is_empty()).then(|| view! { <h2 class="mktg-section-h2">{title}</h2> })}
                                    {(!subtitle.is_empty()).then(|| view! { <p class="mktg-section-sub">{subtitle}</p> })}
                                    <div class="mktg-feature-grid">
                                        {features.into_iter().map(|item| view! {
                                            <div class="mktg-feature-cell">
                                                <span class="material-symbols-outlined mktg-feature-icon">"check_circle"</span>
                                                <h3 class="mktg-feature-title">{item}</h3>
                                            </div>
                                        }).collect_view()}
                                    </div>
                                </div>
                            </section>
                        }.into_any())
                    }
                    "cta" => {
                        let title = field(&block, &["title", "headline", "heading"]);
                        let body = field(&block, &["body", "description", "subtitle"]);
                        let cta_label = field(&block, &["cta_label", "cta"]);
                        let cta_href = field(&block, &["cta_href", "href", "url"]);
                        Some(view! {
                            <section class="mktg-cta-section">
                                <div class="mktg-section-inner mktg-cta-inner">
                                    {(!title.is_empty()).then(|| view! { <h2 class="mktg-cta-h2">{title}</h2> })}
                                    {(!body.is_empty()).then(|| view! { <p class="mktg-cta-sub">{body}</p> })}
                                    {(!cta_label.is_empty()).then(|| view! {
                                        <a class="mktg-btn-accent mktg-btn-lg" href=if cta_href.is_empty() { "#waitlist-wrap".to_string() } else { cta_href }>
                                            {cta_label}
                                        </a>
                                    })}
                                </div>
                            </section>
                        }.into_any())
                    }
                    "rich_text" => {
                        let title = field(&block, &["title", "heading"]);
                        let body = field(&block, &["body", "text", "content"]);
                        Some(view! {
                            <section class="mktg-section">
                                <div class="mktg-section-inner">
                                    {(!title.is_empty()).then(|| view! { <h2 class="mktg-section-h2">{title}</h2> })}
                                    {(!body.is_empty()).then(|| view! { <p class="mktg-section-sub">{body}</p> })}
                                </div>
                            </section>
                        }.into_any())
                    }
                    "lead_form" => {
                        let title = field(&block, &["title", "headline", "heading"]);
                        let cta_label = field(&block, &["cta_label", "cta"]);
                        Some(view! {
                            <section class="mktg-section">
                                <div class="mktg-section-inner" style="text-align:center">
                                    {(!title.is_empty()).then(|| view! { <h2 class="mktg-section-h2">{title}</h2> })}
                                    <a class="mktg-btn-accent mktg-btn-lg" href="#waitlist-wrap">
                                        {if cta_label.is_empty() { "Get early access".to_string() } else { cta_label }}
                                    </a>
                                </div>
                            </section>
                        }.into_any())
                    }
                    _ => None,
                }
            }).collect_view()}
        </div>
    }
}
