use leptos::prelude::*;
use serde_json::Value;
use shared_ui::marketing::MarketingSectionBlockType;

pub fn has_full_page_block(blocks: &Value) -> bool {
    block_items(blocks).iter().any(|block| {
        MarketingSectionBlockType::try_from(field(block, &["type", "kind"]).as_str())
            .is_ok_and(|block_type| block_type == MarketingSectionBlockType::FullPage)
    })
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

pub fn field(block: &Value, keys: &[&str]) -> String {
    keys.iter()
        .find_map(|key| block.get(*key).and_then(Value::as_str))
        .unwrap_or_default()
        .to_string()
}

fn field_opt(block: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| block.get(*key).and_then(Value::as_str))
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SectionBlocks {
    pub stats: Option<StatsBlock>,
    pub feature_grid: Option<FeatureGridBlock>,
    pub personas: Option<PersonasBlock>,
    pub cta: Option<CtaBlock>,
    pub beta_strip: Option<BetaStripBlock>,
    pub markets: Option<MarketsBlock>,
    pub payment_rails: Option<PaymentRailsBlock>,
    pub str_section: Option<CardSectionBlock>,
    pub tenant_portal: Option<CardSectionBlock>,
    pub footer: Option<FooterBlock>,
    pub nav_sections: Option<NavSectionsBlock>,
    pub pricing_intro: Option<PricingIntroBlock>,
    pub trade_categories: Option<TradeCategoriesBlock>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatItem {
    pub value: String,
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatsBlock {
    pub items: Vec<StatItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FeatureItem {
    pub icon: String,
    pub title: String,
    pub description: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FeatureGridBlock {
    pub eyebrow: Option<String>,
    pub heading: Option<String>,
    pub items: Vec<FeatureItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PersonaItem {
    pub icon: Option<String>,
    pub title: String,
    pub subhead: Option<String>,
    pub accent: Option<String>,
    pub bullets: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PersonasBlock {
    pub eyebrow: Option<String>,
    pub heading: Option<String>,
    pub subhead: Option<String>,
    pub items: Vec<PersonaItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CtaBlock {
    pub eyebrow: Option<String>,
    pub heading: Option<String>,
    pub subhead: Option<String>,
    pub button_label: Option<String>,
    pub button_href: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BetaStripBlock {
    pub title: Option<String>,
    pub body: Option<String>,
    pub button_label: Option<String>,
    pub button_href: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MarketItem {
    pub flag: String,
    pub name: String,
    pub desc: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MarketsBlock {
    pub eyebrow: Option<String>,
    pub heading: Option<String>,
    pub subhead: Option<String>,
    pub items: Vec<MarketItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RailItem {
    pub icon: String,
    pub name: String,
    pub desc: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PaymentRailsBlock {
    pub eyebrow: Option<String>,
    pub heading: Option<String>,
    pub subhead: Option<String>,
    pub items: Vec<RailItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SectionCardItem {
    pub icon: String,
    pub title: String,
    pub desc: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CardSectionBlock {
    pub eyebrow: Option<String>,
    pub heading: Option<String>,
    pub subhead: Option<String>,
    pub items: Vec<SectionCardItem>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FooterLink {
    pub label: String,
    pub href: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FooterBlock {
    pub tagline: Option<String>,
    pub links: Vec<FooterLink>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NavSectionsBlock {
    pub items: Vec<FooterLink>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PricingIntroBlock {
    pub eyebrow: Option<String>,
    pub heading: Option<String>,
    pub subtitle: Option<String>,
    pub audience_callout: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TradeCategoryItem {
    pub key: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TradeCategoriesBlock {
    pub items: Vec<TradeCategoryItem>,
}

pub fn parse_section_blocks(blocks: &Value) -> SectionBlocks {
    let mut parsed = SectionBlocks::default();

    for block in block_items(blocks) {
        let Ok(block_type) =
            MarketingSectionBlockType::try_from(field(&block, &["type", "kind"]).as_str())
        else {
            continue;
        };

        match block_type {
            MarketingSectionBlockType::Stats if parsed.stats.is_none() => {
                parsed.stats = Some(StatsBlock {
                    items: block_array(&block, "items")
                        .into_iter()
                        .map(|item| StatItem {
                            value: field(&item, &["value"]),
                            label: field(&item, &["label"]),
                        })
                        .filter(|item| !item.value.is_empty() || !item.label.is_empty())
                        .collect(),
                });
            }
            MarketingSectionBlockType::FeatureGrid if parsed.feature_grid.is_none() => {
                parsed.feature_grid = Some(FeatureGridBlock {
                    eyebrow: field_opt(&block, &["eyebrow"]),
                    heading: field_opt(&block, &["heading", "title"]),
                    items: block_array(&block, "items")
                        .into_iter()
                        .map(|item| FeatureItem {
                            icon: field(&item, &["icon"]),
                            title: field(&item, &["title", "name"]),
                            description: field(&item, &["description", "desc", "body"]),
                        })
                        .filter(|item| !item.title.is_empty() || !item.description.is_empty())
                        .collect(),
                });
            }
            MarketingSectionBlockType::Personas if parsed.personas.is_none() => {
                parsed.personas = Some(PersonasBlock {
                    eyebrow: field_opt(&block, &["eyebrow"]),
                    heading: field_opt(&block, &["heading", "title"]),
                    subhead: field_opt(&block, &["subhead", "subtitle", "description"]),
                    items: block_array(&block, "items")
                        .into_iter()
                        .map(|item| PersonaItem {
                            icon: field_opt(&item, &["icon"]),
                            title: field(&item, &["title", "name"]),
                            subhead: field_opt(&item, &["subhead", "subtitle", "label"]),
                            accent: field_opt(&item, &["accent"]),
                            bullets: string_list(&item, "bullets"),
                        })
                        .filter(|item| !item.title.is_empty() || !item.bullets.is_empty())
                        .collect(),
                });
            }
            MarketingSectionBlockType::Cta if parsed.cta.is_none() => {
                parsed.cta = Some(CtaBlock {
                    eyebrow: field_opt(&block, &["eyebrow"]),
                    heading: field_opt(&block, &["heading", "title"]),
                    subhead: field_opt(&block, &["subhead", "subtitle", "body", "description"]),
                    button_label: field_opt(&block, &["button_label", "cta_label", "label"]),
                    button_href: field_opt(&block, &["button_href", "cta_href", "href"]),
                });
            }
            MarketingSectionBlockType::BetaStrip if parsed.beta_strip.is_none() => {
                parsed.beta_strip = Some(BetaStripBlock {
                    title: field_opt(&block, &["title", "heading"]),
                    body: field_opt(&block, &["body", "description", "subhead"]),
                    button_label: field_opt(&block, &["button_label", "cta_label", "label"]),
                    button_href: field_opt(&block, &["button_href", "cta_href", "href"]),
                });
            }
            MarketingSectionBlockType::Markets if parsed.markets.is_none() => {
                parsed.markets = Some(MarketsBlock {
                    eyebrow: field_opt(&block, &["eyebrow"]),
                    heading: field_opt(&block, &["heading", "title"]),
                    subhead: field_opt(&block, &["subhead", "subtitle", "description"]),
                    items: block_array(&block, "items")
                        .into_iter()
                        .map(|item| MarketItem {
                            flag: field(&item, &["flag", "icon"]),
                            name: field(&item, &["name", "title"]),
                            desc: field(&item, &["desc", "description"]),
                        })
                        .filter(|item| !item.name.is_empty() || !item.desc.is_empty())
                        .collect(),
                });
            }
            MarketingSectionBlockType::PaymentRails if parsed.payment_rails.is_none() => {
                parsed.payment_rails = Some(PaymentRailsBlock {
                    eyebrow: field_opt(&block, &["eyebrow"]),
                    heading: field_opt(&block, &["heading", "title"]),
                    subhead: field_opt(&block, &["subhead", "subtitle", "description"]),
                    items: block_array(&block, "items")
                        .into_iter()
                        .map(|item| RailItem {
                            icon: field(&item, &["icon"]),
                            name: field(&item, &["name", "title"]),
                            desc: field(&item, &["desc", "description"]),
                        })
                        .filter(|item| !item.name.is_empty() || !item.desc.is_empty())
                        .collect(),
                });
            }
            MarketingSectionBlockType::StrSection if parsed.str_section.is_none() => {
                parsed.str_section = Some(parse_card_section(&block));
            }
            MarketingSectionBlockType::TenantPortal if parsed.tenant_portal.is_none() => {
                parsed.tenant_portal = Some(parse_card_section(&block));
            }
            MarketingSectionBlockType::Footer if parsed.footer.is_none() => {
                parsed.footer = Some(FooterBlock {
                    tagline: field_opt(&block, &["tagline", "subhead"]),
                    links: parse_links(&block),
                });
            }
            MarketingSectionBlockType::NavSections if parsed.nav_sections.is_none() => {
                parsed.nav_sections = Some(NavSectionsBlock {
                    items: parse_links(&block),
                });
            }
            MarketingSectionBlockType::PricingIntro if parsed.pricing_intro.is_none() => {
                parsed.pricing_intro = Some(PricingIntroBlock {
                    eyebrow: field_opt(&block, &["eyebrow"]),
                    heading: field_opt(&block, &["heading", "title"]),
                    subtitle: field_opt(&block, &["subtitle", "subhead", "description"]),
                    audience_callout: field_opt(&block, &["audience_callout"]),
                });
            }
            MarketingSectionBlockType::TradeCategories if parsed.trade_categories.is_none() => {
                parsed.trade_categories = Some(TradeCategoriesBlock {
                    items: block_array(&block, "items")
                        .into_iter()
                        .map(|item| TradeCategoryItem {
                            key: field(&item, &["key", "slug", "value"]),
                            label: field(&item, &["label", "title", "name"]),
                            description: field_opt(&item, &["description", "desc", "body"]),
                        })
                        .filter(|item| !item.key.is_empty() && !item.label.is_empty())
                        .collect(),
                });
            }
            _ => {}
        }
    }

    parsed
}

fn block_array(block: &Value, key: &str) -> Vec<Value> {
    block
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn parse_card_section(block: &Value) -> CardSectionBlock {
    CardSectionBlock {
        eyebrow: field_opt(block, &["eyebrow"]),
        heading: field_opt(block, &["heading", "title"]),
        subhead: field_opt(block, &["subhead", "subtitle", "description"]),
        items: block_array(block, "items")
            .into_iter()
            .map(|item| SectionCardItem {
                icon: field(&item, &["icon"]),
                title: field(&item, &["title", "name"]),
                desc: field(&item, &["desc", "description"]),
            })
            .filter(|item| !item.title.is_empty() || !item.desc.is_empty())
            .collect(),
    }
}

fn parse_links(block: &Value) -> Vec<FooterLink> {
    block_array(block, "links")
        .into_iter()
        .chain(block_array(block, "items"))
        .map(|item| FooterLink {
            label: field(&item, &["label", "title", "name"]),
            href: field(&item, &["href", "url"]),
        })
        .filter(|item| !item.label.is_empty() && !item.href.is_empty())
        .collect()
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
                let Ok(block_type) =
                    MarketingSectionBlockType::try_from(field(&block, &["type", "kind"]).as_str())
                else {
                    return None;
                };

                match block_type {
                    MarketingSectionBlockType::Hero => {
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
                    MarketingSectionBlockType::FeatureGrid => {
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
                    MarketingSectionBlockType::Cta => {
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
                    MarketingSectionBlockType::RichText => {
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
                    _ => None,
                }
            }).collect_view()}
        </div>
    }
}
