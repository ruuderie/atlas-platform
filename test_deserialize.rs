use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimelineBlockData {
    pub source: String,
    pub config: TimelineBlockConfig,
    #[serde(default)]
    pub items: Vec<TimelineItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimelineBlockConfig {
    #[serde(default)]
    pub filter_category: Option<String>,
    #[serde(default)]
    pub filter_metadata: Option<String>,
    #[serde(default)]
    pub show_date_range: bool,
    #[serde(default)]
    pub show_bullets: bool,
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub section_title: Option<String>,
}

fn default_layout() -> String { "detailed".to_string() }

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

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum DynamicBlock {
    Timeline(TimelineBlockData),
}

impl<'de> serde::Deserialize<'de> for DynamicBlock {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let map = serde_json::Map::deserialize(deserializer).map_err(serde::de::Error::custom)?;
        let (key, value) = map.into_iter().next().unwrap();
        match key.as_str() {
            "Timeline" => Ok(DynamicBlock::Timeline(serde_json::from_value(value).unwrap())),
            _ => panic!("unknown"),
        }
    }
}

fn main() {
    let j = r#"
    [
        {
            "Timeline": {
                "source": "tenant_entries",
                "config": {
                    "filter_category": "work",
                    "show_date_range": true,
                    "show_bullets": true,
                    "layout": "detailed",
                    "section_title": "Work Experience"
                },
                "items": []
            }
        }
    ]
    "#;
    let blocks: Vec<DynamicBlock> = serde_json::from_str(j).unwrap();
    println!("Parsed: {:?}", blocks);
}
