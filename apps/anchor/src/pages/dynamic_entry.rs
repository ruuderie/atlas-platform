use leptos::*;
use leptos_router::*;
use crate::resume_engine::get_single_tenant_entry;

#[component]
pub fn DynamicEntry() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").cloned().unwrap_or_default());

    let entry_resource = create_resource(
        move || slug(),
        |s| async move {
            if s.is_empty() {
                return None;
            }
            get_single_tenant_entry(s).await.unwrap_or(None)
        },
    );

    view! {
        <main class="pt-32 pb-24 min-h-screen bg-surface">
            <Suspense fallback=move || view! { <div class="flex justify-center py-20 text-on-surface-variant animate-pulse font-mono tracking-wider">"Loading..."</div> }>
                {move || match entry_resource.get() {
                    None => view! { <div class="flex justify-center py-20 text-on-surface-variant animate-pulse font-mono tracking-wider">"Loading..."</div> }.into_view(),
                    Some(None) => view! {
                        <div class="container mx-auto px-4 max-w-4xl text-center py-20">
                            <h1 class="text-4xl font-black text-on-surface mb-4">"Entry Not Found"</h1>
                            <p class="text-on-surface-variant mb-8">"The case study or project you are looking for does not exist."</p>
                            <a href="/p/projects" class="inline-flex items-center gap-2 bg-primary text-on-primary px-6 py-3 rounded-full font-bold hover:bg-primary/90 transition-colors">
                                <span class="material-symbols-outlined text-[1.2rem]">"arrow_back"</span>
                                "Back to Projects"
                            </a>
                        </div>
                    }.into_view(),
                    Some(Some(entry)) => {
                        let cover_image = entry.metadata.as_ref().and_then(|m| m.get("cover_image_url").and_then(|v| v.as_str()).map(|s| s.to_string()));
                        
                        view! {
                            <article class="container mx-auto px-4 max-w-4xl">
                                {if let Some(img) = cover_image {
                                    view! {
                                        <div class="w-full h-64 md:h-96 rounded-3xl overflow-hidden mb-12 shadow-lg border border-outline-variant/30">
                                            <img src={img} class="w-full h-full object-cover" alt={entry.title.clone()} />
                                        </div>
                                    }.into_view()
                                } else { view! {}.into_view() }}

                                <header class="mb-12">
                                    <div class="flex items-center gap-3 mb-4">
                                        <span class="px-3 py-1 bg-primary/10 text-primary text-xs font-bold uppercase tracking-wider rounded-full border border-primary/20">
                                            {entry.category.to_string()}
                                        </span>
                                        {if let Some(date) = entry.date_range.clone().or(entry.published_at.clone()) {
                                            view! { <span class="text-sm font-semibold text-on-surface-variant">"{date}"</span> }.into_view()
                                        } else { view! {}.into_view() }}
                                    </div>
                                    
                                    <h1 class="text-5xl md:text-6xl font-black text-on-surface tracking-tight mb-6 leading-tight">
                                        {entry.title.clone()}
                                    </h1>
                                    
                                    {if let Some(sub) = entry.subtitle.clone() {
                                        view! { <p class="text-xl md:text-2xl text-on-surface-variant font-light leading-relaxed">{sub}</p> }.into_view()
                                    } else { view! {}.into_view() }}
                                </header>

                                <div class="prose prose-lg prose-on-surface max-w-none">
                                    {if !entry.bullets.is_empty() {
                                        view! {
                                            <ul>
                                                {entry.bullets.into_iter().map(|b| view! { <li>{b}</li> }).collect_view()}
                                            </ul>
                                        }.into_view()
                                    } else { view! {}.into_view() }}
                                </div>
                                
                                <div class="mt-16 pt-8 border-t border-outline-variant/50">
                                    <a href="javascript:history.back()" class="inline-flex items-center gap-2 text-on-surface-variant hover:text-primary transition-colors font-semibold">
                                        <span class="material-symbols-outlined text-[1.2rem]">"arrow_back"</span>
                                        "Go Back"
                                    </a>
                                </div>
                            </article>
                        }.into_view()
                    }
                }}
            </Suspense>
        </main>
    }
}
