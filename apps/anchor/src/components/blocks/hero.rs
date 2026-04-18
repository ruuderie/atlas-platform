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
    let design = use_context::<crate::pages::landing::DesignConfig>()
        .unwrap_or_default();
    
    let layout_type = data.layout.unwrap_or_else(|| "standard".to_string());

    if layout_type == "minimal" {
        view! {
            <section class="w-full py-20 md:py-32 bg-surface text-center px-4">
                <div class="container mx-auto max-w-4xl flex flex-col items-center justify-center">
                    <h1 class=format!("text-5xl md:text-7xl font-extrabold text-on-surface mb-6 tracking-tight {}", design.heading_font) inner_html=data.title.clone()>
                    </h1>
                    
                    {if !data.subtitle.is_empty() {
                        view!{ <p class=format!("text-xl md:text-2xl text-on-surface-variant mt-4 mx-auto {}", design.body_font)>
                            {data.subtitle.clone()}
                        </p> }.into_view()
                    } else { view!{}.into_view() }}

                    {if data.cta_text.is_some() && !data.cta_text.clone().unwrap_or_default().is_empty() {
                        view!{ <div class="mt-10">
                            <a href=data.cta_link.clone().unwrap_or_else(|| "#".to_string()) 
                               class=format!("inline-block bg-primary hover:bg-primary-container text-white font-bold transition-colors duration-300 text-lg shadow-sm {} {} {}", design.button_padding, design.border_radius_base, design.heading_font)>
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
            _ => "background-color: var(--color-surface, #000000);".to_string(), // fallback
        };

        view! {
            <section class=format!("relative w-full h-[80vh] min-h-[500px] flex items-center justify-center overflow-hidden {}", 
                if design.container_strategy == "asymmetrical-gutters" { "justify-start px-[8.5rem]" } else { "" }
            ) style=bg_style>

                {if design.background_pattern == "blueprint-grid" {
                    view! { <div class="absolute inset-0 bg-[#f8fafa] z-0" style="background-image: radial-gradient(var(--color-outline-variant, #bfc7cf) 0.5px, transparent 0.5px); background-size: 40px 40px; opacity: 0.3;"></div> }.into_view()
                } else if design.background_pattern == "radial-glow" {
                    view! { <div class="absolute inset-0 z-0 opacity-80" style="background: radial-gradient(circle at center, var(--color-primary-container) 0%, var(--color-primary) 100%);"></div> }.into_view()
                } else {
                    view! { <div class="absolute inset-0 bg-black bg-opacity-60 z-0"></div> }.into_view()
                }}
                
                <div class=format!("relative z-10 w-full flex flex-col {}", 
                    if design.container_strategy == "asymmetrical-gutters" { "items-start text-left max-w-7xl" } else { "items-center text-center container mx-auto px-4" }
                )>
                    <h1 class=format!("text-4xl md:text-5xl lg:text-7xl font-bold mb-6 {} {} {}", 
                        if design.background_pattern == "blueprint-grid" { "text-primary" } else { "text-white uppercase" }, 
                        if design.container_strategy == "asymmetrical-gutters" { "tracking-tighter" } else { "tracking-tight" },
                        design.heading_font
                    ) inner_html=data.title.clone()>
                    </h1>
                    
                    {if !data.subtitle.is_empty() {
                        view!{ <p class=format!("text-xl md:text-2xl mt-4 font-medium {} {}", 
                            if design.background_pattern == "blueprint-grid" { "text-on-surface" } else { "text-gray-200" },
                            design.body_font
                        )>
                            {data.subtitle.clone()}
                        </p> }.into_view()
                    } else { view!{}.into_view() }}

                    {if data.cta_text.is_some() && !data.cta_text.clone().unwrap_or_default().is_empty() {
                        view!{ <div class="mt-10">
                            <a href=data.cta_link.clone().unwrap_or_else(|| "#".to_string()) 
                               class=format!("inline-block bg-primary text-on-primary font-bold transition-all duration-300 {} {} {} {}", 
                                   if design.elevation_strategy == "tonal-shifts" { "hover:bg-primary-container hover:text-on-primary-container" } else { "hover:opacity-90" },
                                   design.button_padding, 
                                   design.border_radius_base,
                                   design.meta_font
                               )>
                                {data.cta_text.clone().unwrap()}
                            </a>
                        </div> }.into_view()
                    } else { view!{}.into_view() }}
                </div>
            </section>
        }.into_view()
    }
}
