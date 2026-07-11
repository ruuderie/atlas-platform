//! Shared marketing pricing grid driven by `platform_product_plans` via public product API.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanBillingInterval {
    Month,
    Year,
    Forever,
    Custom,
}

impl Default for PlanBillingInterval {
    fn default() -> Self {
        Self::Month
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MarketingPlan {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub tagline: String,
    pub price_cents: i32,
    #[serde(default = "default_currency")]
    pub currency: String,
    #[serde(default)]
    pub billing_interval: PlanBillingInterval,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default = "default_cta")]
    pub cta_label: String,
    #[serde(default)]
    pub cta_href: Option<String>,
    #[serde(default)]
    pub is_featured: bool,
    #[serde(default)]
    pub sort_order: i32,
}

fn default_currency() -> String {
    "USD".to_string()
}

fn default_cta() -> String {
    "Get started".to_string()
}

pub fn format_plan_price(plan: &MarketingPlan) -> (String, Option<&'static str>) {
    match plan.billing_interval {
        PlanBillingInterval::Custom if plan.price_cents <= 0 => ("Custom".to_string(), None),
        PlanBillingInterval::Forever if plan.price_cents <= 0 => {
            if plan.name.eq_ignore_ascii_case("free") || plan.slug.contains("free") || plan.slug.contains("basic") {
                ("Free".to_string(), None)
            } else {
                ("$0".to_string(), None)
            }
        }
        _ if plan.price_cents <= 0 && matches!(plan.billing_interval, PlanBillingInterval::Forever) => {
            ("$0".to_string(), None)
        }
        _ if plan.price_cents <= 0 => ("$0".to_string(), None),
        _ => {
            let dollars = plan.price_cents / 100;
            let per = match plan.billing_interval {
                PlanBillingInterval::Month => Some("/mo"),
                PlanBillingInterval::Year => Some("/yr"),
                PlanBillingInterval::Forever => None,
                PlanBillingInterval::Custom => None,
            };
            (format!("${dollars}"), per)
        }
    }
}

#[component]
pub fn MarketingPricingGrid(
    plans: Vec<MarketingPlan>,
    #[prop(into)] section_id: String,
    #[prop(into, optional)] eyebrow: Option<String>,
    #[prop(into, optional)] heading: Option<String>,
    #[prop(into, optional)] subtitle: Option<String>,
    #[prop(into, optional)] default_cta_href: Option<String>,
) -> impl IntoView {
    let mut plans = plans;
    plans.sort_by_key(|p| p.sort_order);

    let eyebrow = eyebrow.unwrap_or_else(|| "Pricing".to_string());
    let heading = heading.unwrap_or_else(|| "Simple. Transparent. No surprises.".to_string());
    let default_href = default_cta_href.unwrap_or_else(|| "#waitlist-wrap".to_string());

    view! {
        <section id=section_id class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">{eyebrow}</p>
                <h2 class="mktg-section-h2">{heading}</h2>
                {subtitle.map(|s| view! {
                    <p class="mktg-section-sub" style="max-width:560px;margin:0 auto 2.5rem;">{s}</p>
                })}
                <div class="mktg-pricing-grid">
                    {plans.into_iter().map(|plan| {
                        let (price, per) = format_plan_price(&plan);
                        let featured = plan.is_featured;
                        let check_color = if featured { "#ff6b35" } else { "#06d6a0" };
                        let card_class = if featured {
                            "mktg-pricing-card mktg-pricing-featured"
                        } else {
                            "mktg-pricing-card"
                        };
                        let btn_class = if featured {
                            "mktg-pricing-btn mktg-pricing-btn-accent"
                        } else {
                            "mktg-pricing-btn mktg-pricing-btn-ghost"
                        };
                        let href = plan.cta_href.clone().unwrap_or_else(|| default_href.clone());
                        let cta_id = format!("pricing-{}-cta", plan.slug);
                        view! {
                            <div class=card_class>
                                <span class="mktg-pricing-tier">{plan.name.clone()}</span>
                                <div class="mktg-pricing-price">
                                    {price}
                                    {per.map(|p| view! { <span class="mktg-pricing-per">{p}</span> })}
                                </div>
                                <div class="mktg-pricing-sub">{plan.tagline.clone()}</div>
                                <ul class="mktg-pricing-features">
                                    {plan.features.into_iter().map(|feat| {
                                        let color = check_color;
                                        view! {
                                            <li class="mktg-pf">
                                                <span
                                                    class="material-symbols-outlined"
                                                    style=format!(
                                                        "font-size:16px;color:{color};font-variation-settings:'FILL' 1"
                                                    )
                                                >"check"</span>
                                                {feat}
                                            </li>
                                        }
                                    }).collect_view()}
                                </ul>
                                <a href=href class=btn_class id=cta_id>{plan.cta_label}</a>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}
