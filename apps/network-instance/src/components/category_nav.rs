use leptos::prelude::*;

#[component]
pub fn CategoryNavigation() -> impl IntoView {
    view! {
        <section class="max-w-[1440px] mx-auto w-full px-6 md:px-10 py-5 animate-slide-up border-b border-neutral-200/60">
            <div class="relative">
                // Fade edges
                <div class="absolute left-0 top-0 bottom-0 w-12 bg-background z-10 pointer-events-none" style="opacity:0.8"></div>
                <div class="absolute right-0 top-0 bottom-0 w-12 bg-background z-10 pointer-events-none" style="opacity:0.8"></div>
                
                <div class="flex overflow-x-auto gap-8 pb-1 snap-x snap-mandatory hide-scrollbar px-2">
                    {vec![
                        ("Contractors", "contractors", "M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5M9 7h1m-1 4h1m4-4h1m-1 4h1m-5 10v-5a1 1 0 011-1h2a1 1 0 011 1v5m-4 0h4"),
                        ("Plumbers", "plumbers", "M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"),
                        ("Electricians", "electricians", "M13 10V3L4 14h7v8l9-11h-7z"),
                        ("Landscaping", "landscaping", "M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064"),
                        ("Real Estate", "real-estate", "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"),
                        ("Cleaning", "cleaning", "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"),
                        ("Roofing", "roofing", "M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z"),
                        ("Painters", "painters", "M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01"),
                    ].into_iter().map(|(title, queries, path)| view! {
                        <a href=format!("/search?category={}", queries) class="snap-start flex flex-col items-center gap-2 min-w-max text-neutral-400 hover:text-foreground pb-3 transition-all duration-300 cursor-pointer group relative">
                            <div class="w-12 h-12 rounded-xl bg-neutral-100 group-hover:bg-primary/10 flex items-center justify-center transition-all duration-300 group-hover:scale-105">
                                <svg class="w-6 h-6 stroke-[1.5] text-neutral-500 group-hover:text-primary transition-colors duration-300" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d=path></path></svg>
                            </div>
                            <span class="text-xs font-medium tracking-wide whitespace-nowrap text-neutral-500 group-hover:text-foreground group-hover:font-semibold transition-all duration-200">{title}</span>
                            // Active indicator bar
                            <div class="absolute -bottom-0.5 left-1/2 -translate-x-1/2 w-0 h-[2px] bg-primary rounded-full group-hover:w-full transition-all duration-300"></div>
                        </a>
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}
