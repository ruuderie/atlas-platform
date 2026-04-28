use leptos::*;
use leptos_router::*;
use crate::resume_engine::get_single_tenant_entry;
use crate::components::design_mode::use_kami_mode;
use crate::utils::text::parse_rai;

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
        {move || {
            let kami = use_kami_mode();
            if kami {
                // ── Kami parchment entry detail ────────────────────────────────
                view! {
                    <main class="pt-28 pb-24 bg-surface-container-low min-h-screen">
                        <Suspense fallback=move || view! {
                            <div class="max-w-3xl mx-auto px-4 pt-12 text-[#6b6a64] jetbrains text-xs uppercase animate-pulse">"Loading..."</div>
                        }>
                            {move || match entry_resource.get() {
                                None => view! { <div /> }.into_view(),
                                Some(None) => view! {
                                    <div class="max-w-3xl mx-auto px-4 pt-12 text-center">
                                        <p class="font-display text-[#504e49] mb-6">"Entry not found."</p>
                                        <a href="/p/projects" class="jetbrains text-[0.65rem] uppercase tracking-widest text-[#1B365D] underline">"← Back to Projects"</a>
                                    </div>
                                }.into_view(),
                                Some(Some(entry)) => {
                                    let cover_image = entry.metadata.as_ref()
                                        .and_then(|m| m.get("cover_image_url").and_then(|v| v.as_str()).map(|s| s.to_string()));
                                    let bullets = entry.bullets.clone();

                                    view! {
                                        <div class="max-w-3xl mx-auto px-4 pb-24">
                                            // Breadcrumb
                                            <nav class="mb-8">
                                                <a href="/p/projects"
                                                   class="inline-flex items-center gap-2 text-[#6b6a64] hover:text-[#1B365D] transition-colors jetbrains text-[0.65rem] uppercase tracking-widest">
                                                    "← Projects"
                                                </a>
                                            </nav>

                                            // Parchment card
                                            <article class="bg-[#f5f4ed] shadow-2xl px-10 py-14 md:px-16 md:py-18">

                                                {if let Some(img) = cover_image {
                                                    view! {
                                                        <div class="w-full h-52 overflow-hidden mb-10">
                                                            <img src={img} alt={entry.title.clone()} class="w-full h-full object-cover" />
                                                        </div>
                                                    }.into_view()
                                                } else { view! {}.into_view() }}

                                                // Header
                                                <header class="text-center mb-12 pb-10 border-b border-[#1B365D]/20">
                                                    <div class="jetbrains text-[0.6rem] uppercase tracking-[0.25em] text-[#6b6a64] mb-3">
                                                        {entry.category.to_string()}
                                                    </div>
                                                    <h1 class="font-display text-[1.85rem] font-bold text-[#1B365D] leading-tight mb-4">
                                                        {entry.title}
                                                    </h1>
                                                    {if let Some(sub) = entry.subtitle {
                                                        view! {
                                                            <p class="font-display text-base text-[#504e49] italic leading-relaxed max-w-xl mx-auto">
                                                                {sub}
                                                            </p>
                                                        }.into_view()
                                                    } else { view! {}.into_view() }}
                                                </header>

                                                // Bullets with R/A/I label rendering
                                                {if !bullets.is_empty() {
                                                    view! {
                                                        <ul class="space-y-3 list-none pl-0">
                                                            {bullets.into_iter().map(|b| {
                                                                let (label, rest) = parse_rai(&b);
                                                                let label = label.map(str::to_string);
                                                                let rest = rest.to_string();
                                                                view! {
                                                                    <li class="flex gap-3 text-sm text-[#504e49] leading-[1.75]">
                                                                        {if let Some(lbl) = label {
                                                                            view! {
                                                                                <span class="jetbrains text-[0.58rem] uppercase tracking-widest text-[#1B365D] font-bold mt-[0.2rem] shrink-0 w-14 text-right">
                                                                                    {lbl}":"
                                                                                </span>
                                                                            }.into_view()
                                                                        } else {
                                                                            view! {
                                                                                <span class="text-[#1B365D]/30 mt-[0.25rem] shrink-0">"—"</span>
                                                                            }.into_view()
                                                                        }}
                                                                        <span>{rest}</span>
                                                                    </li>
                                                                }
                                                            }).collect_view()}
                                                        </ul>
                                                    }.into_view()
                                                } else { view! {}.into_view() }}

                                            </article>
                                        </div>
                                    }.into_view()
                                }
                            }}
                        </Suspense>
                    </main>
                }.into_view()
            } else {
                // ── Material 3 dark (existing) ─────────────────────────────────
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
                                    let cover_image = entry.metadata.as_ref()
                                        .and_then(|m| m.get("cover_image_url").and_then(|v| v.as_str()).map(|s| s.to_string()));

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
                                                        view! { <span class="text-sm font-semibold text-on-surface-variant">{date}</span> }.into_view()
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
                }.into_view()
            }
        }}
    }
}
