use leptos::prelude::*;
use crate::app::ListingModel;

#[component]
pub fn SearchGrid(
    results: Vec<ListingModel>,
    set_selected: WriteSignal<Option<ListingModel>>,
    view_mode: ReadSignal<String>
) -> impl IntoView {
    view! {
        <div class=move || format!("transition-all duration-500 {}", if view_mode.get() == "grid" { "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-x-6 gap-y-10" } else { "space-y-6 flex flex-col" })>
            {results.clone().into_iter().map(move |listing| {
                let listing_clone1 = listing.clone();
                let listing_clone2 = listing.clone();
                view! {
                <div on:click=move |_| set_selected.set(Some(listing_clone1.clone())) class=move || format!("group cursor-pointer flex transition-all duration-300 {}", if view_mode.get() == "grid" { "flex-col hover:-translate-y-1" } else { "flex-col sm:flex-row gap-6 border-b border-neutral-200/60 pb-8 hover:bg-neutral-50/50 rounded-2xl p-2 -m-2" })>
                    // Image Container
                    <div class=move || format!("relative overflow-hidden rounded-2xl bg-neutral-100 flex-shrink-0 transition-all duration-500 {}", if view_mode.get() == "grid" { "w-full aspect-[20/19] mb-3 group-hover:shadow-card-hover" } else { "w-full sm:w-[320px] h-60 sm:h-auto group-hover:shadow-premium" })>
                        // Fav Icon
                        <button class="absolute top-3 right-3 p-2 rounded-full z-20 text-white/80 hover:text-rose-500 hover:bg-white hover:scale-110 active:scale-95 transition-all duration-200 drop-shadow-md">
                            <svg class="w-5 h-5 stroke-2" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"></path></svg>
                        </button>
                        // Type Badge  
                        <div class="absolute top-3 left-3 bg-white/95 backdrop-blur-sm px-2.5 py-1 text-[11px] font-bold text-neutral-700 rounded-lg shadow-sm z-20 uppercase tracking-wider">
                            {listing_clone2.listing_type.clone()}
                        </div>
                        
                        // Hover overlay with gradient
                        <div class="absolute inset-0 bg-gradient-to-t from-black/10 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 z-10"></div>
                        
                        // Image placeholder
                        <div class="absolute inset-0 bg-gradient-to-br from-neutral-100 via-neutral-200/50 to-neutral-100 flex items-center justify-center">
                            <svg class="w-14 h-14 text-neutral-300 group-hover:scale-110 group-hover:text-neutral-400 transition-all duration-700" fill="currentColor" viewBox="0 0 24 24"><path d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path></svg>
                        </div>
                    </div>
                    
                    // Detail Text 
                    <div class="flex-grow flex flex-col justify-start">
                        <div class="flex justify-between items-start mb-0.5">
                            <h3 class="text-[15px] font-semibold text-foreground leading-tight truncate pr-4">{listing_clone2.title.clone()}</h3>
                            <div class="flex items-center text-[14px] font-medium text-foreground gap-1 flex-shrink-0">
                                <svg class="w-3.5 h-3.5 text-amber-500" fill="currentColor" viewBox="0 0 20 20"><path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z"></path></svg>
                                "4.9"
                            </div>
                        </div>
                        <div class="text-[14px] text-neutral-500 truncate">{listing_clone2.listing_type.clone()}</div>
                        <div class=move || format!("text-[14px] text-neutral-400 mt-0.5 {}", if view_mode.get() == "grid" { "truncate" } else { "line-clamp-2 mt-2" })>
                            {listing_clone2.description.clone()}
                        </div>
                        
                        <div class="mt-2 pt-1 flex items-center font-semibold text-[14px] text-foreground group-hover:text-primary transition-colors duration-200">
                            <span class="underline decoration-neutral-300 group-hover:decoration-primary transition-colors">"Check prices"</span>
                            <svg class="w-3.5 h-3.5 ml-1 opacity-0 -translate-x-1 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path></svg>
                        </div>
                    </div>
                </div>
            }}).collect_view()}
        </div>
    }
}

#[component]
pub fn RefinementSidebar() -> impl IntoView {
    view! {
        <div class="w-full hidden md:block mt-4 sticky top-[108px]">
            <h4 class="font-semibold text-base mb-5 text-foreground tracking-tight">"Filters"</h4>
            
            <div class="space-y-6">
                // Category Filter
                <div class="pb-6 border-b border-neutral-200/60">
                    <h5 class="font-semibold text-[13px] mb-3 text-neutral-600 uppercase tracking-wider">"Service Type"</h5>
                    <div class="flex flex-wrap gap-2">
                        <button class="px-4 py-2 rounded-full bg-foreground text-background text-[13px] font-medium transition-all duration-200 active:scale-95 shadow-sm">
                            "Show All"
                        </button>
                        <button class="px-4 py-2 rounded-full border border-neutral-200 bg-white text-neutral-600 hover:border-neutral-400 hover:text-foreground text-[13px] font-medium transition-all duration-200 active:scale-95">
                            "Contractors"
                        </button>
                        <button class="px-4 py-2 rounded-full border border-neutral-200 bg-white text-neutral-600 hover:border-neutral-400 hover:text-foreground text-[13px] font-medium transition-all duration-200 active:scale-95">
                            "Plumbers"
                        </button>
                        <button class="px-4 py-2 rounded-full border border-neutral-200 bg-white text-neutral-600 hover:border-neutral-400 hover:text-foreground text-[13px] font-medium transition-all duration-200 active:scale-95">
                            "Electricians"
                        </button>
                        <button class="px-4 py-2 rounded-full border border-neutral-200 bg-white text-neutral-600 hover:border-neutral-400 hover:text-foreground text-[13px] font-medium transition-all duration-200 active:scale-95">
                            "HVAC"
                        </button>
                    </div>
                </div>
                
                // Features
                <div class="pb-6 border-b border-neutral-200/60">
                    <h5 class="font-semibold text-[13px] mb-3 text-neutral-600 uppercase tracking-wider">"Features"</h5>
                    <div class="space-y-3">
                        <label class="flex items-center gap-3 cursor-pointer group py-1">
                            <div class="w-5 h-5 rounded-md border-2 border-neutral-300 group-hover:border-primary flex items-center justify-center transition-all duration-200">
                                <svg class="w-3 h-3 text-transparent" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"></path></svg>
                            </div>
                            <span class="text-[14px] text-neutral-600 group-hover:text-foreground transition-colors">"Instant Booking"</span>
                        </label>
                        <label class="flex items-center gap-3 cursor-pointer group py-1">
                            <div class="w-5 h-5 rounded-md border-2 border-neutral-300 group-hover:border-primary flex items-center justify-center transition-all duration-200">
                                <svg class="w-3 h-3 text-transparent" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"></path></svg>
                            </div>
                            <span class="text-[14px] text-neutral-600 group-hover:text-foreground transition-colors">"Verified Pro"</span>
                        </label>
                        <label class="flex items-center gap-3 cursor-pointer group py-1">
                            <div class="w-5 h-5 rounded-md border-2 border-neutral-300 group-hover:border-primary flex items-center justify-center transition-all duration-200">
                                <svg class="w-3 h-3 text-transparent" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"></path></svg>
                            </div>
                            <span class="text-[14px] text-neutral-600 group-hover:text-foreground transition-colors">"Free Estimates"</span>
                        </label>
                    </div>
                </div>
            </div>
        </div>
    }
}
