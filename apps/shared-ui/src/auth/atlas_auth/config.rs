#[derive(Clone, Debug)]
pub struct AtlasAuthConfig {
    pub api_prefix: &'static str,
    pub atlas_api_url: Option<String>,
}

impl Default for AtlasAuthConfig {
    fn default() -> Self {
        Self {
            api_prefix: "/api",
            atlas_api_url: None,
        }
    }
}
