use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CalloutBlockData {
    // Seed stores the callout copy as `text`; admin UI may use `title`
    #[serde(alias = "text", default)]
    pub title: String,
    #[serde(default)]
    pub description: String,
    // `style` field from seed ("primary" etc) — stored but not rendered yet
    #[serde(default)]
    pub style: Option<String>,
}

#[component]
pub fn CalloutBlock(data: CalloutBlockData) -> impl IntoView {
    view! {
        <section class="py-16 md:py-20 bg-surface-container-low border-y border-outline-variant/30 w-full">
            <div class="container mx-auto px-4 max-w-5xl text-center">
                <h2 class="text-3xl md:text-5xl font-extrabold text-[#003366] dark:text-primary mb-6 tracking-tight">
                    {data.title.clone()}
                </h2>
                <p class="text-xl md:text-2xl text-on-surface-variant font-medium leading-relaxed max-w-4xl mx-auto">
                    {data.description.clone()}
                </p>
            </div>
        </section>
    }
}
