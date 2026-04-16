use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AccordionBlockData {
    pub config: AccordionConfig,
    pub items: Vec<AccordionItem>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AccordionConfig {
    #[serde(default)]
    pub section_title: Option<String>,
    #[serde(default = "default_single")]
    pub mode: String,                      // "single" | "multi"
    #[serde(default)]
    pub default_open_index: Option<u32>,
}

fn default_single() -> String { "single".to_string() }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AccordionItem {
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub badge: Option<String>,
}

#[component]
pub fn AccordionBlock(data: AccordionBlockData) -> impl IntoView {
    // Note: State logic for single/multi expand should theoretically use signals.
    // For Phase 2 we implement the view structure with standard HTML 'details/summary' pattern 
    // which gives native accordion functionality without Leptos JS interop overhead.
    // Standard HTML <details> doesn't support generic 'single' mode without JS, but is extremely lightweight.
    
    view! {
        <section class="py-12 md:py-16 w-full">
            <div class="container mx-auto px-4 max-w-3xl">
                {if let Some(ref title) = data.config.section_title {
                    view! {
                        <h2 class="text-3xl font-bold text-on-surface mb-8 text-center md:text-left">
                            {title}
                        </h2>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}

                <div class="space-y-4">
                    {data.items.into_iter().enumerate().map(|(idx, item)| {
                        // Native details/summary element
                        let is_open = data.config.default_open_index == Some(idx as u32);
                        
                        view! {
                            <details class="group bg-surface border border-outline-variant rounded-xl overflow-hidden shadow-sm" open=is_open>
                                <summary class="flex items-center justify-between p-5 cursor-pointer list-none hover:bg-surface-container-lowest transition-colors outline-none focus:ring-2 focus:ring-primary focus:ring-inset">
                                    <div class="flex items-center gap-4">
                                        <h3 class="text-xl font-semibold text-on-surface group-open:text-primary transition-colors">
                                            {item.title}
                                        </h3>
                                        {if let Some(badge) = item.badge {
                                            view! { <span class="px-2.5 py-0.5 rounded text-xs font-bold uppercase tracking-wide bg-primary/10 text-primary">{badge}</span> }.into_view()
                                        } else { view! {}.into_view() }}
                                    </div>
                                    <span class="material-symbols-outlined text-on-surface-variant group-open:rotate-180 transition-transform duration-300">
                                        "expand_more"
                                    </span>
                                </summary>
                                <div class="px-5 pb-5 pt-0 text-on-surface-variant leading-relaxed">
                                    // Normally we would render markdown here
                                    {item.body}
                                </div>
                            </details>
                        }
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}
