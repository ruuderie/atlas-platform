use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GridItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub link_url: Option<String>,
    // icon field used by seed data (material icon name)
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GridBlockData {
    pub section_title: Option<String>,
    #[serde(default)]
    pub columns: Option<u32>,
    pub items: Vec<GridItem>,
}

#[component]
pub fn GridBlock(data: GridBlockData) -> impl IntoView {
    let design = use_context::<crate::pages::landing::DesignConfig>()
        .unwrap_or_default();
        
    view! {
        <section class=format!("py-16 md:py-24 bg-surface dark:bg-surface w-full {}", 
            if design.container_strategy == "asymmetrical-gutters" { "px-[8.5rem]" } else { "px-4" }
        )>
            <div class=format!("mx-auto {}", 
                if design.container_strategy == "asymmetrical-gutters" { "max-w-none" } else { "container max-w-7xl" }
            )>
                {if data.section_title.is_some() && !data.section_title.clone().unwrap_or_default().is_empty() {
                    view!{ <h2 class=format!("text-3xl md:text-5xl font-bold text-primary dark:text-primary mb-16 {} {} {}", 
                        if design.container_strategy == "asymmetrical-gutters" { "text-left tracking-tighter" } else { "text-center tracking-tight" },
                        if design.background_pattern == "blueprint-grid" { "border-b-2 border-primary inline-block pb-4" } else { "" },
                        design.heading_font
                    )>
                        {data.section_title.clone().unwrap()}
                    </h2> }.into_view()
                } else { view!{}.into_view() }}

                <div class=format!("grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 {}", 
                    if design.container_strategy == "asymmetrical-gutters" { "gap-12" } else { "gap-8" }
                )>
                    {data.items.into_iter().map(|item| {
                        view! {
                            <a href=item.link_url.clone().unwrap_or_else(|| "#".to_string()) 
                               class=format!("group block h-full overflow-hidden transition-all duration-300 {} {} {}", 
                                   if design.elevation_strategy == "tonal-shifts" { "bg-surface-container-low hover:bg-surface-container-highest" } else { "bg-surface-container hover:bg-surface-container-high" },
                                   if design.elevation_strategy == "flat-ghost" { "border-none" } else { "border border-outline-variant/30 hover:border-primary hover:shadow-2xl transform hover:-translate-y-1" },
                                   design.border_radius_base
                               )>
                                {if item.image_url.is_some() && !item.image_url.clone().unwrap_or_default().is_empty() {
                                    view!{ <div class="w-full h-48 overflow-hidden bg-surface-container-high">
                                        <img src=item.image_url.clone().unwrap() alt=item.title.clone() class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
                                    </div> }.into_view()
                                } else { view!{}.into_view() }}
                                <div class="p-8">
                                    <h3 class=format!("text-2xl font-bold text-primary dark:text-on-surface mb-4 group-hover:text-primary transition-colors {}", design.heading_font)>
                                        {item.title.clone()}
                                    </h3>
                                    {if item.icon.is_some() && !item.icon.clone().unwrap_or_default().is_empty() {
                                        view!{ <div class=format!("mb-6 inline-block px-3 py-1 bg-secondary-container/20 text-secondary {} {}", design.meta_font, design.border_radius_base)>
                                            <span class="text-sm">{item.icon.clone().unwrap()}</span>
                                        </div> }.into_view()
                                    } else { view!{}.into_view() }}
                                    <p class=format!("text-lg text-on-surface-variant leading-relaxed {}", design.body_font)>
                                        {item.description.clone()}
                                    </p>
                                </div>
                            </a>
                        }
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}
