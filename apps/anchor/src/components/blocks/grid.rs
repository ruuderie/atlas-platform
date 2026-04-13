use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GridItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub image_url: Option<String>,
    pub link_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GridBlockData {
    pub section_title: Option<String>,
    pub items: Vec<GridItem>,
}

#[component]
pub fn GridBlock(data: GridBlockData) -> impl IntoView {
    view! {
        <section class="py-16 md:py-24 bg-surface dark:bg-surface w-full">
            <div class="container mx-auto px-4 max-w-7xl">
                {if data.section_title.is_some() && !data.section_title.clone().unwrap_or_default().is_empty() {
                    view!{ <h2 class="text-3xl md:text-5xl font-extrabold text-center text-[#003366] dark:text-primary mb-16 tracking-tight">
                        {data.section_title.clone().unwrap()}
                    </h2> }.into_view()
                } else { view!{}.into_view() }}

                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                    {data.items.into_iter().map(|item| {
                        view! {
                            <a href=item.link_url.clone().unwrap_or_else(|| "#".to_string()) 
                               class="group block h-full bg-surface-container-low border border-outline-variant/30 rounded-xl overflow-hidden hover:shadow-2xl hover:border-primary transition-all duration-300 transform hover:-translate-y-1">
                                {if item.image_url.is_some() && !item.image_url.clone().unwrap_or_default().is_empty() {
                                    view!{ <div class="w-full h-48 overflow-hidden bg-surface-container-high">
                                        <img src=item.image_url.clone().unwrap() alt=item.title.clone() class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
                                    </div> }.into_view()
                                } else { view!{}.into_view() }}
                                <div class="p-8">
                                    <h3 class="text-2xl font-bold text-[#003366] dark:text-on-surface mb-4 group-hover:text-primary transition-colors">
                                        {item.title.clone()}
                                    </h3>
                                    <p class="text-lg text-on-surface-variant leading-relaxed">
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
