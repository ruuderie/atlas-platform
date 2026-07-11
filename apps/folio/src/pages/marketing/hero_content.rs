use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct HeroContent {
    #[serde(default)]
    pub eyebrow: Option<String>,
    #[serde(default)]
    pub headline: Option<String>,
    #[serde(default)]
    pub headline_accent: Option<String>,
    #[serde(default)]
    pub subhead: Option<String>,
    #[serde(default)]
    pub cta_label: Option<String>,
    #[serde(default)]
    pub cta_href: Option<String>,
    #[serde(default)]
    pub proof_items: Vec<String>,
    #[serde(default)]
    pub pricing_eyebrow: Option<String>,
    #[serde(default)]
    pub pricing_heading: Option<String>,
    #[serde(default)]
    pub pricing_subtitle: Option<String>,
}

impl HeroContent {
    pub fn from_value(v: &Value) -> Self {
        serde_json::from_value(v.clone()).unwrap_or_default()
    }

    pub fn has_content(&self) -> bool {
        self.eyebrow.as_ref().is_some_and(|v| !v.trim().is_empty())
            || self.headline.as_ref().is_some_and(|v| !v.trim().is_empty())
            || self
                .headline_accent
                .as_ref()
                .is_some_and(|v| !v.trim().is_empty())
            || self.subhead.as_ref().is_some_and(|v| !v.trim().is_empty())
            || self
                .cta_label
                .as_ref()
                .is_some_and(|v| !v.trim().is_empty())
            || self.cta_href.as_ref().is_some_and(|v| !v.trim().is_empty())
            || self.proof_items.iter().any(|v| !v.trim().is_empty())
            || self
                .pricing_eyebrow
                .as_ref()
                .is_some_and(|v| !v.trim().is_empty())
            || self
                .pricing_heading
                .as_ref()
                .is_some_and(|v| !v.trim().is_empty())
            || self
                .pricing_subtitle
                .as_ref()
                .is_some_and(|v| !v.trim().is_empty())
    }
}
