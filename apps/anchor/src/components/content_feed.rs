use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContentNode {
    pub id: String,
    pub category: String, // 'project', 'certification', 'blog_post'
    pub title: String,
    pub subtitle: Option<String>,
    pub date_label: Option<String>,
    pub status: Option<String>,
    pub tags: Vec<String>,
    pub bullets: Vec<String>,
    pub markdown: Option<String>,
    pub link_url: Option<String>,
    pub is_highlight: bool,
    /// Content rendering format: 'markdown' (default), 'latex', or 'mdlatex'
    /// - 'markdown'  : render via pulldown_cmark (existing path)
    /// - 'latex'     : raw LaTeX, client-side KaTeX auto-render
    /// - 'mdlatex'   : Markdown with $...$ / $$...$$ delimiters, KaTeX for math
    #[serde(default = "default_content_format")]
    pub content_format: String,
}

fn default_content_format() -> String {
    "markdown".to_string()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LayoutMode {
    List,
    Grid,
    Carousel,
}

#[component]
pub fn ContentFeed(
    nodes: Vec<ContentNode>,
    #[prop(default = LayoutMode::Grid)] layout: LayoutMode,
) -> impl IntoView {
    match layout {
        LayoutMode::Grid => view! {
            <div class="space-y-16 max-w-5xl">
                {nodes.into_iter().map(|node| {
                    let is_in_progress = node.status.as_deref().unwrap_or("COMPLETED").to_uppercase() == "IN PROGRESS";
                    let status_color = if is_in_progress { "bg-[#f7931a] text-black" } else { "bg-surface-container-highest text-on-surface" };
                    
                    view! {
                        <article class="bg-surface-container-low p-8 md:p-12 relative border-l-4 border-secondary shadow-none ring-0">
                            {node.status.clone().map(|status| view! {
                                <div class=format!("absolute top-0 right-0 {} px-3 py-1 text-xs font-bold jetbrains uppercase tracking-widest hidden md:block", status_color)>
                                    {status.to_uppercase()}
                                </div>
                            })}
                            <h2 class="text-3xl font-extrabold text-primary mb-2">{node.title}</h2>
                            
                            {node.link_url.clone().map(|url| view! {
                                <a href=url.clone() target="_blank" rel="noopener noreferrer" class="text-sm font-label text-outline hover:text-secondary hover:underline transition-colors flex items-center gap-2 mb-6 w-fit cursor-pointer">
                                    <span class="material-symbols-outlined text-sm">"link"</span>
                                    {url}
                                </a>
                            })}
                            
                            {node.subtitle.clone().map(|sub| view! {
                                <p class="text-lg font-bold text-on-surface mb-6">{sub}</p>
                            })}
                            
                            {(!node.tags.is_empty()).then(|| view! {
                                <div class="flex flex-wrap gap-2 mb-8">
                                    {node.tags.into_iter().map(|tag| view! {
                                        <div class="bg-surface-container-highest px-3 py-1 text-xs font-bold text-on-surface jetbrains uppercase border-b border-r border-outline-variant/50">
                                            {tag}
                                        </div>
                                    }).collect_view()}
                                </div>
                            })}

                            {(!node.bullets.is_empty()).then(|| view! {
                                <ul class="text-on-surface-variant leading-relaxed text-sm space-y-4 list-none p-0 m-0">
                                    {node.bullets.into_iter().map(|b| view! {
                                        <li class="relative pl-5 before:content-['//'] before:absolute before:-left-1 before:text-secondary before:font-bold before:jetbrains">
                                            {b}
                                        </li>
                                    }).collect_view()}
                                </ul>
                            })}
                        </article>
                    }
                }).collect_view()}
            </div>
        }.into_view(),
        LayoutMode::List => view! {
            <div class="space-y-12 max-w-4xl">
                {nodes.into_iter().map(|node| {
                    // Branch rendering on content_format:
                    // - 'markdown'       : render via pulldown_cmark (Mermaid-aware)
                    // - 'latex'/'mdlatex': pass raw content to client-side KaTeX auto-render
                    let html_output = if let Some(md) = &node.markdown {
                        match node.content_format.as_str() {
                            "latex" | "mdlatex" => {
                                // For LaTeX content: emit the raw source in a KaTeX-ready div.
                                // KaTeX auto-render (loaded conditionally in app.rs for these
                                // formats) picks up .katex-content divs on the client.
                                // The 'mdlatex' path runs through pulldown_cmark first, then
                                // KaTeX handles inline $...$ / $$...$$ delimiters on the client.
                                if node.content_format == "mdlatex" {
                                    // Run through Markdown first, KaTeX handles math delimiters
                                    let mut options = pulldown_cmark::Options::empty();
                                    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
                                    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                                    let parser = pulldown_cmark::Parser::new_ext(md, options);
                                    let mut html = String::new();
                                    pulldown_cmark::html::push_html(&mut html, parser);
                                    format!("<div class='katex-content' data-format='mdlatex'>{}</div>", html)
                                } else {
                                    // Pure LaTeX: wrap in pre, KaTeX auto-render processes it
                                    format!("<div class='katex-content' data-format='latex'><pre class='katex-source'>{}</pre></div>",
                                        html_escape::encode_text(md))
                                }
                            }
                            _ => {
                                // Default: 'markdown'
                                let mut options = pulldown_cmark::Options::empty();
                                options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
                                options.insert(pulldown_cmark::Options::ENABLE_TABLES);
                                let parser = pulldown_cmark::Parser::new_ext(md, options);
                                let mut html = String::new();
                                pulldown_cmark::html::push_html(&mut html, parser);
                                html
                            }
                        }
                    } else {
                        String::new()
                    };

                    let link = node.link_url.clone();

                    let card = view! {
                        <article class="bg-surface-container p-8 hover:bg-surface-container-high transition-colors group cursor-pointer border-l-4 border-transparent hover:border-secondary">
                            <div class="flex flex-col md:flex-row md:justify-between items-start mb-4 gap-4">
                                <h3 class="text-2xl font-bold text-primary group-hover:text-secondary transition-colors leading-snug">
                                    {node.title}
                                </h3>
                                {node.date_label.map(|date| view! {
                                    <span class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider whitespace-nowrap pt-1">
                                        {date} {node.subtitle.map(|sub| format!(" // {}", sub)).unwrap_or_default()}
                                    </span>
                                })}
                            </div>
                            
                            {(!html_output.is_empty()).then(|| view! {
                                <div class="text-on-surface-variant leading-relaxed text-sm mb-6 max-w-2xl prose prose-invert prose-p:text-sm prose-a:text-secondary prose-a:no-underline hover:prose-a:underline" inner_html=html_output>
                                    {
                                        #[cfg(target_arch = "wasm32")]
                                        let _ = js_sys::eval("if(window.renderMermaid) window.renderMermaid();");
                                    }
                                </div>
                            })}
                            
                            {(!node.bullets.is_empty()).then(|| view! {
                                <ul class="text-on-surface-variant leading-relaxed text-sm mb-6 space-y-4 list-none p-0 m-0">
                                    {node.bullets.into_iter().map(|b| view! {
                                        <li class="relative pl-5 before:content-['//'] before:absolute before:-left-1 before:text-secondary before:font-bold before:jetbrains">
                                            {b}
                                        </li>
                                    }).collect_view()}
                                </ul>
                            })}

                            <div class="flex flex-wrap gap-4 mt-6">
                                {node.tags.into_iter().map(|tag| view! {
                                    <span class="bg-surface-container-highest px-3 py-1 jetbrains text-[0.65rem] font-bold text-on-surface-variant uppercase">{tag}</span>
                                }).collect_view()}
                            </div>
                        </article>
                    };

                    if let Some(href) = link {
                        view! {
                            <A href=href class="block no-underline outline-none">{card}</A>
                        }.into_view()
                    } else {
                        card.into_view()
                    }
                }).collect_view()}
            </div>
        }.into_view(),
        LayoutMode::Carousel => view! {
            <div class="flex overflow-x-auto gap-6 pb-8 snap-x pl-2 pr-6 max-w-6xl hide-scrollbar-custom">
                {nodes.into_iter().enumerate().map(|(idx, node)| {
                    let has_link = node.link_url.is_some();
                    
                    let bg_accent = if node.is_highlight { "bg-primary/5 border-primary/20 text-primary" } else { "bg-surface-container border-outline-variant/30 text-secondary" };
                    let hover_accent = if node.is_highlight { "hover:border-primary/60 hover:bg-primary/10" } else { "hover:border-secondary/60 hover:bg-surface-container-high" };
                    let label_bg = if node.is_highlight { "bg-primary/10 text-primary" } else { "bg-secondary/10 text-secondary" };
                    let icon = if node.is_highlight { "model_training" } else { "verified_user" };
                    let id_prefix = if node.is_highlight { "EXEC" } else { "AUTH" };

                    let content = view! {
                        <div class=format!("w-72 sm:w-80 shrink-0 snap-start border {} {} transition-colors group flex flex-col h-[280px] relative shadow-none", bg_accent, hover_accent)>
                            <div class=format!("absolute top-0 right-0 {} px-2 py-0.5 text-[0.5rem] font-bold jetbrains uppercase", label_bg)>
                                {format!("{}_{:03}", id_prefix, idx + 1)}
                            </div>
                            
                            <div class="p-6 flex items-start h-full flex-col">
                                <span class="material-symbols-outlined text-4xl mb-4 group-hover:scale-110 transition-transform">{icon}</span>
                                
                                <h3 class="font-bold text-lg mb-2 text-on-surface line-clamp-3">{node.title}</h3>
                                
                                {node.subtitle.map(|s| view! {
                                    <p class="text-sm font-medium text-on-surface-variant mb-2 line-clamp-2">{s}</p>
                                })}
                                
                                {node.date_label.map(|d| view! {
                                    <span class="text-[0.65rem] jetbrains uppercase font-bold tracking-widest text-outline mt-auto border-t border-outline-variant/30 pt-4 w-full block">
                                        "VERIFIED: " {d}
                                    </span>
                                })}
                            </div>
                        </div>
                    };
                    
                    if has_link {
                        view! {
                            <a href=node.link_url.clone().unwrap() target="_blank" rel="noopener noreferrer" class="block cursor-pointer outline-none">
                                {content}
                            </a>
                        }.into_view()
                    } else {
                        view! { <div class="block outline-none">{content}</div> }.into_view()
                    }
                }).collect_view()}
                <style>
                    ".hide-scrollbar-custom::-webkit-scrollbar { display: none; }
                     .hide-scrollbar-custom { -ms-overflow-style: none; scrollbar-width: none; }"
                </style>
            </div>
        }.into_view(),
    }
}
