use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RichTextData {
    pub html_content: String,
}

#[component]
pub fn RichTextBlock(data: RichTextData) -> impl IntoView {
    view! {
        <section class="py-12 md:py-16 w-full bg-surface dark:bg-surface">
            <div class="container mx-auto px-4 max-w-4xl prose prose-lg dark:prose-invert prose-headings:font-bold prose-a:text-primary hover:prose-a:text-primary-container"
                inner_html=data.html_content.clone()>
            </div>
        </section>
    }
}
