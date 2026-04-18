use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HeroBlockData {
    // Accept both component format (title/subtitle) and seed format (heading/subheading)
    #[serde(alias = "heading", default)]
    pub title: String,
    #[serde(alias = "subheading", default)]
    pub subtitle: String,
    #[serde(alias = "primary_cta_text", default)]
    pub cta_text: Option<String>,
    #[serde(alias = "primary_cta_link", default)]
    pub cta_link: Option<String>,
    #[serde(alias = "background_image", default)]
    pub background_image_url: Option<String>,
    #[serde(default)]
    pub layout: Option<String>,
}

#[component]
pub fn HeroBlock(data: HeroBlockData) -> impl IntoView {
    let layout_type = data.layout.unwrap_or_else(|| "standard".to_string());

    if layout_type == "minimal" {
        view! {
            <section class="w-full py-20 md:py-32 bg-surface text-center px-4">
                <div class="container mx-auto max-w-4xl flex flex-col items-center justify-center">
                    <h1 class="text-5xl md:text-7xl font-extrabold text-on-surface mb-6 tracking-tight" inner_html=data.title.clone()>
                    </h1>
                    
                    {if !data.subtitle.is_empty() {
                        view!{ <p class="text-xl md:text-2xl text-on-surface-variant mt-4 max-w-3xl mx-auto font-medium">
                            {data.subtitle.clone()}
                        </p> }.into_view()
                    } else { view!{}.into_view() }}

                    {if data.cta_text.is_some() && !data.cta_text.clone().unwrap_or_default().is_empty() {
                        view!{ <div class="mt-10">
                            <a href=data.cta_link.clone().unwrap_or_else(|| "#".to_string()) 
                               class="inline-block bg-primary hover:bg-primary-container text-white font-bold py-4 px-10 rounded-lg transition-colors duration-300 text-lg shadow-sm">
                                {data.cta_text.clone().unwrap()}
                            </a>
                        </div> }.into_view()
                    } else { view!{}.into_view() }}
                </div>
            </section>
        }.into_view()
    } else {
        // Corporate / Default full-bleed layout
        let bg_style = match data.background_image_url {
            Some(url) if !url.is_empty() => format!("background-image: url('{}'); background-size: cover; background-position: center;", url),
            _ => "background-color: #000000;".to_string(), // fallback black or dark theme
        };

        view! {
            <section class="relative w-full h-[80vh] min-h-[500px] flex items-center justify-center overflow-hidden" style=bg_style>
                <div class="absolute inset-0 bg-black bg-opacity-60 z-0"></div>
                
                <div class="relative z-10 container mx-auto px-4 text-center flex flex-col items-center justify-center">
                    <h1 class="text-4xl md:text-5xl lg:text-7xl font-bold text-white mb-6 uppercase tracking-tight" inner_html=data.title.clone()>
                    </h1>
                    
                    {if !data.subtitle.is_empty() {
                        view!{ <p class="text-xl md:text-2xl text-gray-200 mt-4 max-w-4xl mx-auto font-medium">
                            {data.subtitle.clone()}
                        </p> }.into_view()
                    } else { view!{}.into_view() }}

                    {if data.cta_text.is_some() && !data.cta_text.clone().unwrap_or_default().is_empty() {
                        view!{ <div class="mt-10">
                            <a href=data.cta_link.clone().unwrap_or_else(|| "#".to_string()) 
                               class="inline-block bg-primary hover:bg-primary-container text-white font-bold py-4 px-8 rounded transition-colors duration-300 text-lg shadow-lg">
                                {data.cta_text.clone().unwrap()}
                            </a>
                        </div> }.into_view()
                    } else { view!{}.into_view() }}
                </div>
            </section>
        }.into_view()
    }
}
