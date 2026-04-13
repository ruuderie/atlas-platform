use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FormBuilderData {
    pub form_id: String,
    pub title: String,
    pub description: Option<String>,
    // TODO: Phase 2 will implement full schema-driven rendering
}

#[component]
pub fn FormBuilderBlock(data: FormBuilderData) -> impl IntoView {
    view! {
        <section class="py-16 bg-surface-container-low w-full">
            <div class="container mx-auto px-4 max-w-3xl">
                <div class="bg-surface border border-outline-variant/30 rounded-2xl p-8 md:p-12 shadow-xl">
                    <h2 class="text-3xl font-bold text-primary mb-4 text-center">
                        {data.title.clone()}
                    </h2>
                    {if data.description.is_some() && !data.description.clone().unwrap_or_default().is_empty() {
                        view!{ <p class="text-center text-on-surface-variant mb-8">
                            {data.description.clone().unwrap()}
                        </p> }.into_view()
                    } else { view!{}.into_view() }}
                    
                    <div class="flex items-center justify-center p-12 border-2 border-dashed border-outline-variant rounded-xl bg-surface-container-lowest">
                        <p class="text-on-surface-variant jetbrains">"Multi-Step Webform Engine (Under Construction)"</p>
                    </div>
                </div>
            </div>
        </section>
    }
}
