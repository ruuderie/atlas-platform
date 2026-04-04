use leptos::*;
use leptos_meta::{Meta, Title};

use crate::b2b::{get_case_studies, get_highlights, get_services};
use crate::pages::landing::get_site_settings;

#[component]
pub fn Services() -> impl IntoView {
    fn render_markdown(md: &str) -> String {
        let mut options = pulldown_cmark::Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
        let parser = pulldown_cmark::Parser::new_ext(md, options);
        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, parser);
        html_output
    }
    let services_res = create_resource(|| (), |_| get_services(true));
    let case_studies_res = create_resource(|| (), |_| get_case_studies(true));
    let settings_res = create_resource(|| (), |_| get_site_settings());

    view! {
        <Transition fallback=move || view! { <div class="min-h-screen pt-32 pb-24 px-6 md:px-12 flex justify-center items-center jetbrains text-outline">"Authenticating Protocol..."</div> }>
            {move || match settings_res.get() {
                Some(Ok(settings)) => {
                    if !settings.b2b_enabled {
                        view! {
                            <Title text="Not Found" />
                            <main class="min-h-screen pt-32 pb-24 px-6 md:px-12 max-w-7xl mx-auto flex flex-col items-center justify-center text-center">
                                <span class="material-symbols-outlined text-[6rem] text-error mb-8">"error"</span>
                                <h1 class="font-display font-extrabold text-5xl md:text-7xl leading-tight mb-6">"404 Not Found"</h1>
                                <p class="text-xl text-outline-variant font-medium leading-relaxed max-w-2xl mb-12">"The requested operations protocol is currently offline or does not exist."</p>
                                <a href="/" class="bg-surface text-on-surface border border-outline-variant font-bold jetbrains uppercase tracking-widest px-8 py-4 hover:bg-surface-container transition-colors">"RETURN TO BASE"</a>
                            </main>
                        }.into_view()
                    } else {
                        view! {
                            <Title text="Services & Consulting | Anchor" />
                            <Meta name="description" content="Strategic engineering and technical architecture consulting by Ruud Salym Erie. High-stakes platform modernization, SaaS delivery, and rigorous Rust solutions."/>
                            <Meta property="og:title" content="Services & Consulting | Anchor"/>
                            <Meta property="og:description" content="Strategic engineering and technical architecture consulting by Ruud Salym Erie. High-stakes platform modernization, SaaS delivery, and rigorous Rust solutions."/>
                            <main class="min-h-screen pt-32 pb-24 px-6 md:px-12 max-w-7xl mx-auto space-y-32">
            // Header Section
            <section class="max-w-3xl border-b-2 border-outline-variant/30 pb-8 mb-24">
                <crate::components::dynamic_header::DynamicPageHeader route_path="/services".to_string() badge_color="primary".to_string() />
                <a href="/book" class="inline-block bg-primary text-on-primary font-bold jetbrains uppercase tracking-widest px-8 py-4 hover:bg-primary-container hover:text-on-primary-container transition-colors shadow-[4px_4px_0_var(--md-sys-color-on-background)] hover:shadow-none hover:translate-y-1 hover:translate-x-1">
                    "Schedule Discovery"
                </a>
            </section>

            // Services Grid
            <section class="space-y-12">
                <div class="flex items-center gap-4">
                    <div class="h-[1px] bg-outline-variant/30 flex-1"></div>
                    <h2 class="font-label text-sm uppercase tracking-[0.2em] text-secondary font-bold">"Core Offerings"</h2>
                    <div class="h-[1px] bg-outline-variant/30 flex-1"></div>
                </div>

                <Transition fallback=move || view! { <div class="text-center text-outline jetbrains">"Loading services..."</div> }>
                    {move || match services_res.get() {
                        Some(Ok(items)) => {
                            if items.is_empty() {
                                view! { <div class="text-center text-outline">"Services are currently being updated. Check back soon."</div> }.into_view()
                            } else {
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                                        {items.into_iter().map(|item| view! {
                                            <div class="bg-surface-container-low p-8 border border-outline-variant/20 hover:border-primary/50 transition-colors group flex flex-col h-full">
                                                <h3 class="font-bold text-2xl mb-4 text-on-surface group-hover:text-primary transition-colors">{&item.title}</h3>
                                                <p class="text-on-surface-variant lg:text-lg mb-8 flex-1 leading-relaxed">{&item.description}</p>

                                                <div class="space-y-4 mb-8">
                                                    <div class="font-label text-xs uppercase tracking-widest text-secondary font-bold">"Deliverables"</div>
                                                    <ul class="space-y-2">
                                                        {item.deliverables.into_iter().map(|d| view! {
                                                            <li class="flex items-start gap-3 text-sm text-on-surface">
                                                                <span class="material-symbols-outlined text-[1rem] text-primary shrink-0 mt-0.5">"check_circle"</span>
                                                                <span class="leading-relaxed">{d}</span>
                                                            </li>
                                                        }).collect_view()}
                                                    </ul>
                                                </div>

                                                {item.price_range.map(|range| view! {
                                                    <div class="mt-auto pt-6 border-t border-outline-variant/10 font-jetbrains text-sm text-outline font-medium tracking-wide">
                                                        {range}
                                                    </div>
                                                })}
                                            </div>
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            }
                        },
                        _ => view! { <div class="text-error">"Failed to load services"</div> }.into_view(),
                    }}
                </Transition>
            </section>

            // Case Studies / Proof
            <section class="space-y-12 bg-surface-container py-16 px-8 rounded-3xl -mx-8 relative overflow-hidden">
                <div class="absolute -right-32 -top-32 w-96 h-96 bg-primary/5 blur-3xl rounded-full"></div>

                <div class="flex items-center gap-4 relative z-10">
                    <h2 class="font-label text-[0.65rem] uppercase tracking-widest text-outline">"Proof of Impact"</h2>
                    <div class="h-[1px] bg-outline-variant/20 flex-1"></div>
                </div>

                <Transition fallback=move || view! { <div class="text-center text-outline jetbrains">"Loading case studies..."</div> }>
                    {move || match case_studies_res.get() {
                        Some(Ok(items)) => {
                            if items.is_empty() {
                                view! { <div class="text-center text-outline italic">"Select case studies available upon request."</div> }.into_view()
                            } else {
                                view! {
                                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-12 relative z-10">
                                        {items.into_iter().map(|item| view! {
                                            <div class="flex flex-col gap-6">
                                                <div class="inline-block bg-primary/10 text-primary px-4 py-1 font-jetbrains text-[0.65rem] uppercase tracking-widest font-bold self-start">
                                                    {&item.client_name}
                                                </div>
                                                <div class="space-y-6">
                                                    <div>
                                                        <h4 class="font-bold text-on-surface mb-2">"The Challenge"</h4>
                                                        <div class="text-on-surface-variant text-sm leading-relaxed prose prose-invert prose-p:text-sm max-w-none" inner_html=render_markdown(&item.problem)>
                                                            {
                                                                #[cfg(target_arch = "wasm32")]
                                                                let _ = js_sys::eval("if(window.renderMermaid) window.renderMermaid();");
                                                            }
                                                        </div>
                                                    </div>
                                                    <div>
                                                        <h4 class="font-bold text-on-surface mb-2">"The Solution"</h4>
                                                        <div class="text-on-surface-variant text-sm leading-relaxed prose prose-invert prose-p:text-sm max-w-none" inner_html=render_markdown(&item.solution)>
                                                            {
                                                                #[cfg(target_arch = "wasm32")]
                                                                let _ = js_sys::eval("if(window.renderMermaid) window.renderMermaid();");
                                                            }
                                                        </div>
                                                    </div>
                                                    <div class="bg-surface p-4 border-l-4 border-tertiary">
                                                        <h4 class="font-bold text-on-surface mb-1 text-sm">"Impact"</h4>
                                                        <p class="text-tertiary font-medium text-sm">{&item.roi_impact}</p>
                                                    </div>
                                                </div>
                                            </div>
                                        }).collect_view()}
                                    </div>
                                }.into_view()
                            }
                        },
                        _ => view! { <div class="text-error">"Failed to load case studies"</div> }.into_view(),
                    }}
                </Transition>
            </section>

            // Highlights component
            <HighlightsGallery />

        </main>
                        }.into_view()
                    }
                },
                _ => view! { <div class="text-error mt-32 px-12">"System Failure"</div> }.into_view()
            }}
        </Transition>
    }
}

#[component]
pub fn HighlightsGallery() -> impl IntoView {
    let highlights_res = create_resource(|| (), |_| get_highlights(true));

    view! {
        <section class="py-16 overflow-hidden">
            <div class="flex items-center gap-4 mb-12">
                <h2 class="font-label text-sm uppercase tracking-[0.2em] text-secondary font-bold shrink-0">"Featured In & Highlights"</h2>
                <div class="h-[1px] bg-outline-variant/30 flex-1"></div>
            </div>

            <Transition fallback=move || view! { <div class="text-outline jetbrains px-6">"Loading highlights..."</div> }>
                {move || match highlights_res.get() {
                    Some(Ok(items)) => {
                        if items.is_empty() {
                            view! { <div class="hidden"></div> }.into_view()
                        } else {
                            view! {
                                <div class="flex overflow-x-auto gap-6 pb-8 snap-x pl-2 pr-6 hide-scrollbar">
                                    {items.into_iter().map(|item| {
                                        let has_link = !item.url.is_empty();
                                        let content = view! {
                                            <div class="w-72 sm:w-80 shrink-0 snap-start bg-surface border border-outline-variant/20 hover:border-primary/40 transition-colors group flex flex-col h-full rounded-tr-3xl">
                                                {item.image_url.clone().map(|url| view! {
                                                    <div class="h-40 w-full overflow-hidden rounded-tr-3xl bg-surface-container-low border-b border-outline-variant/10">
                                                        <img src=url alt=item.title.clone() class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" />
                                                    </div>
                                                })}
                                                <div class="p-6 flex flex-col flex-1">
                                                    <h3 class="font-bold text-lg mb-2 text-on-surface group-hover:text-primary transition-colors">{&item.title}</h3>
                                                    {item.description.clone().map(|d| view! {
                                                        <p class="text-on-surface-variant text-sm leading-relaxed mb-4 flex-1">{d}</p>
                                                    })}
                                                    {if has_link {
                                                        Some(view! {
                                                            <div class="mt-auto font-jetbrains text-[0.65rem] uppercase tracking-widest text-primary font-bold flex items-center gap-1 group-hover:gap-2 transition-all">
                                                                "EXPLORE" <span class="material-symbols-outlined text-[0.8rem]">"arrow_forward"</span>
                                                            </div>
                                                        })
                                                    } else {
                                                        None
                                                    }}
                                                </div>
                                            </div>
                                        };

                                        if has_link {
                                            view! { <a href=item.url target="_blank" rel="noopener noreferrer" class="block h-full cursor-pointer">{content}</a> }.into_view()
                                        } else {
                                            view! { <div class="block h-full">{content}</div> }.into_view()
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_view()
                        }
                    },
                    _ => view! { <div class="text-error px-6">"Failed to load highlights"</div> }.into_view()
                }}
            </Transition>

            <style>
                ".hide-scrollbar::-webkit-scrollbar { display: none; }
                 .hide-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }"
            </style>
        </section>
    }
}
