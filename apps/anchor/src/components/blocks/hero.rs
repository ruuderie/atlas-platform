use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HeroBlockData {
    pub title: String,
    pub subtitle: String,
    pub cta_text: Option<String>,
    pub cta_link: Option<String>,
    pub background_image_url: Option<String>,
}

#[component]
pub fn HeroBlock(data: HeroBlockData) -> impl IntoView {
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
                           class="inline-block bg-[#003366] hover:bg-[#002244] text-white font-bold py-4 px-8 rounded transition-colors duration-300 text-lg shadow-lg">
                            {data.cta_text.clone().unwrap()}
                        </a>
                    </div> }.into_view()
                } else { view!{}.into_view() }}
            </div>
        </section>
    }
}
