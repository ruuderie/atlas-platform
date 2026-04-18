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
    let design = use_context::<crate::pages::landing::DesignConfig>()
        .unwrap_or_default();
        
    view! {
        <section class=format!("py-16 md:py-20 w-full {} {}", 
            if design.elevation_strategy == "tonal-shifts" { "bg-surface-container-low" } else { "bg-surface-container border-y border-outline-variant/30" },
            if design.container_strategy == "asymmetrical-gutters" { "px-[8.5rem]" } else { "px-4" }
        )>
            <div class=format!("mx-auto {}", 
                if design.container_strategy == "asymmetrical-gutters" { "max-w-7xl text-left" } else { "container max-w-5xl text-center" }
            )>
                <h2 class=format!("text-3xl md:text-5xl font-bold text-primary dark:text-primary mb-6 tracking-tight {}", design.heading_font)>
                    {data.title.clone()}
                </h2>
                <p class=format!("text-xl md:text-2xl text-on-surface-variant font-medium leading-relaxed {}", design.body_font)>
                    {data.description.clone()}
                </p>
            </div>
        </section>
    }
}
