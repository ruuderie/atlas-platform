use leptos::*;
use leptos_meta::{Meta, Title};

use crate::pages::landing::get_site_settings;

fn render_markdown(md: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);

    let parser = pulldown_cmark::Parser::new_ext(md, options);
    let mut html_output = String::new();
    pulldown_cmark::html::push_html(&mut html_output, parser);
    html_output
}

#[component]
pub fn Terms() -> impl IntoView {
    let settings_res = create_resource(|| (), |_| get_site_settings());

    view! {
        <Title text="Terms of Service | Anchor" />
        <Meta name="description" content="Terms of Service and conditions for engaging with Anchor and OPLYST INTERNATIONAL, LLC."/>
        <main class="min-h-screen pt-32 pb-24 px-6 md:px-12 max-w-4xl mx-auto">
            <Transition fallback=move || view! { <div class="text-outline jetbrains">"Loading terms..."</div> }>
                {move || match settings_res.get() {
                    Some(Ok(settings)) => {
                        let html = render_markdown(&settings.terms_html);
                        view! {
                            <article class="prose prose-invert prose-emerald max-w-none
                                prose-headings:font-display prose-headings:font-bold 
                                prose-h1:text-4xl prose-h2:text-2xl prose-h3:text-xl
                                prose-a:text-primary prose-a:no-underline hover:prose-a:underline
                                prose-p:text-on-surface-variant prose-p:leading-relaxed
                                prose-li:text-on-surface-variant"
                                inner_html=html
                            >
                                {
                                    #[cfg(target_arch = "wasm32")]
                                    let _ = js_sys::eval("if(window.renderMermaid) window.renderMermaid();");
                                }
                            </article>
                        }.into_view()
                    },
                    _ => view! { <div class="text-error">"Failed to load terms."</div> }.into_view()
                }}
            </Transition>
        </main>
    }
}

#[component]
pub fn Privacy() -> impl IntoView {
    let settings_res = create_resource(|| (), |_| get_site_settings());

    view! {
        <Title text="Privacy Policy | Anchor" />
        <Meta name="description" content="Privacy Policy covering data handling, cookies, and digital security at Anchor."/>
        <main class="min-h-screen pt-32 pb-24 px-6 md:px-12 max-w-4xl mx-auto">
            <Transition fallback=move || view! { <div class="text-outline jetbrains">"Loading privacy policy..."</div> }>
                {move || match settings_res.get() {
                    Some(Ok(settings)) => {
                        let html = render_markdown(&settings.privacy_html);
                        view! {
                            <article class="prose prose-invert prose-emerald max-w-none
                                prose-headings:font-display prose-headings:font-bold 
                                prose-h1:text-4xl prose-h2:text-2xl prose-h3:text-xl
                                prose-a:text-primary prose-a:no-underline hover:prose-a:underline
                                prose-p:text-on-surface-variant prose-p:leading-relaxed
                                prose-li:text-on-surface-variant"
                                inner_html=html
                            >
                                {
                                    #[cfg(target_arch = "wasm32")]
                                    let _ = js_sys::eval("if(window.renderMermaid) window.renderMermaid();");
                                }
                            </article>
                        }.into_view()
                    },
                    _ => view! { <div class="text-error">"Failed to load privacy policy."</div> }.into_view()
                }}
            </Transition>
        </main>
    }
}
