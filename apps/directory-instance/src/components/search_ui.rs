use leptos::prelude::*;
use crate::app::ListingModel;

#[component]
pub fn SearchGrid(
    results: Vec<ListingModel>,
    set_selected: WriteSignal<Option<ListingModel>>,
    view_mode: ReadSignal<String>
) -> impl IntoView {
    view! {
        <div class=move || format!("transition-all duration-500 {}", if view_mode.get() == "grid" { "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-x-6 gap-y-10" } else { "space-y-8 flex flex-col" })>
            {results.clone().into_iter().map(move |listing| {
                let listing_clone1 = listing.clone();
                let listing_clone2 = listing.clone();
                view! {
                <div on:click=move |_| set_selected.set(Some(listing_clone1.clone())) class=move || format!("group cursor-pointer flex transition-all duration-300 {}", if view_mode.get() == "grid" { "flex-col" } else { "flex-col sm:flex-row gap-6 border-b border-slate-200 pb-8 hover:bg-slate-50 rounded-2xl p-2 -m-2" })>
                    // Image Container
                    <div class=move || format!("relative overflow-hidden rounded-2xl bg-slate-100 flex-shrink-0 transition-all duration-500 {}", if view_mode.get() == "grid" { "w-full aspect-[20/19] mb-3 group-hover:shadow-md" } else { "w-full sm:w-[320px] h-64 sm:h-auto group-hover:shadow-md" })>
                        // Fav Icon
                        <button class="absolute top-3 right-3 p-2 rounded-full z-20 text-white/90 hover:text-rose-500 hover:scale-110 active:scale-95 transition-all duration-200 drop-shadow-md">
                            <span class="material-symbols-outlined text-[24px]">"favorite"</span>
                        </button>
                        
                        // Guest Favorite Badge Overlay (mocked condition)
                        <div class="absolute top-3 left-3 bg-white/95 backdrop-blur-sm px-3 py-1 text-xs font-bold text-slate-800 rounded-full shadow-sm z-20 flex items-center gap-1">
                            <span class="material-symbols-outlined text-[14px] text-amber-500">"workspace_premium"</span>
                            "Top Rated"
                        </div>
                        
                        // Hover overlay with gradient
                        <div class="absolute inset-0 bg-gradient-to-t from-black/20 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500 z-10"></div>
                        
                        // Authentic image instead of placeholder
                        <img src="https://images.unsplash.com/photo-1600596542815-ffad4c1539a9?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80" 
                             alt="Listing Preview" 
                             class="absolute inset-0 w-full h-full object-cover group-hover:scale-105 transition-transform duration-700" 
                        />
                    </div>
                    
                    // Detail Text 
                    <div class="flex-grow flex flex-col justify-start pt-1">
                        <div class="flex justify-between items-start mb-1">
                            <h3 class="text-base font-bold font-headline text-on-primary-fixed leading-tight truncate pr-4">{listing_clone2.title.clone()}</h3>
                            <div class="flex items-center text-sm font-bold text-slate-700 gap-1 flex-shrink-0">
                                <span class="material-symbols-outlined text-[16px]">"star"</span>
                                "4.96"
                            </div>
                        </div>
                        <div class="text-sm text-slate-500 font-body mb-1">{listing_clone2.listing_type.clone()}</div>
                        <div class=move || format!("text-sm text-slate-600 font-body leading-relaxed {}", if view_mode.get() == "grid" { "truncate" } else { "line-clamp-2 mt-2" })>
                            {listing_clone2.description.clone()}
                        </div>
                        
                        <div class="mt-auto pt-4 flex items-center font-bold text-sm text-on-primary-fixed group-hover:text-primary transition-colors duration-200">
                            <span class="underline decoration-slate-300 group-hover:decoration-primary transition-colors">"View availability"</span>
                            <span class="material-symbols-outlined text-[18px] ml-1 opacity-0 -translate-x-1 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300">"chevron_right"</span>
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
        <div class="w-full mt-6 sticky top-[108px] bg-white border border-slate-200 rounded-3xl p-6 shadow-sm">
            <h4 class="font-bold font-headline text-xl mb-6 text-on-primary-fixed pb-4 border-b border-slate-200">"Filters"</h4>
            
            <div class="space-y-8">
                // Price Range
                <div>
                    <h5 class="font-bold text-sm mb-4 text-slate-800 font-headline">"Price range"</h5>
                    <div class="flex items-center gap-4">
                        <div class="border border-slate-300 rounded-xl px-4 py-2 w-full">
                            <span class="text-xs text-slate-500 block">"Minimum"</span>
                            <div class="flex items-center"><span class="text-slate-500 mr-1">"$"</span><input type="number" class="w-full outline-none font-bold text-slate-800" placeholder="100" /></div>
                        </div>
                        <span class="text-slate-300">"-"</span>
                        <div class="border border-slate-300 rounded-xl px-4 py-2 w-full">
                            <span class="text-xs text-slate-500 block">"Maximum"</span>
                            <div class="flex items-center"><span class="text-slate-500 mr-1">"$"</span><input type="number" class="w-full outline-none font-bold text-slate-800" placeholder="500+" /></div>
                        </div>
                    </div>
                </div>

                <hr class="border-slate-200" />

                // Category Filter
                <div>
                    <h5 class="font-bold text-sm mb-4 text-slate-800 font-headline">"Service Category"</h5>
                    <div class="space-y-3 font-body text-slate-600">
                        <label class="flex items-center justify-between cursor-pointer group">
                            <div class="flex items-center gap-3">
                                <div class="w-5 h-5 rounded border border-slate-300 group-hover:border-slate-900 flex items-center justify-center transition-all bg-slate-900 text-white">
                                    <span class="material-symbols-outlined text-[14px]">"check"</span>
                                </div>
                                <span class="font-medium">"All Categories"</span>
                            </div>
                        </label>
                        <label class="flex items-center justify-between cursor-pointer group">
                            <div class="flex items-center gap-3">
                                <div class="w-5 h-5 rounded border border-slate-300 group-hover:border-slate-900 flex items-center justify-center transition-all">
                                </div>
                                <span>"Contractors"</span>
                            </div>
                        </label>
                        <label class="flex items-center justify-between cursor-pointer group">
                            <div class="flex items-center gap-3">
                                <div class="w-5 h-5 rounded border border-slate-300 group-hover:border-slate-900 flex items-center justify-center transition-all">
                                </div>
                                <span>"Plumbers"</span>
                            </div>
                        </label>
                        <label class="flex items-center justify-between cursor-pointer group">
                            <div class="flex items-center gap-3">
                                <div class="w-5 h-5 rounded border border-slate-300 group-hover:border-slate-900 flex items-center justify-center transition-all">
                                </div>
                                <span>"HVAC"</span>
                            </div>
                        </label>
                    </div>
                </div>
                
                <hr class="border-slate-200" />

                // Amenities / Features
                <div>
                    <h5 class="font-bold text-sm mb-4 text-slate-800 font-headline">"Amenities"</h5>
                    <div class="space-y-3 font-body text-slate-600">
                        <label class="flex items-center gap-3 cursor-pointer group">
                            <div class="w-5 h-5 rounded border border-slate-300 group-hover:border-slate-900 flex items-center justify-center transition-all"></div>
                            <span>"Instant Booking"</span>
                        </label>
                        <label class="flex items-center gap-3 cursor-pointer group">
                            <div class="w-5 h-5 rounded border border-slate-300 group-hover:border-slate-900 flex items-center justify-center transition-all text-slate-900 bg-slate-900">
                                <span class="material-symbols-outlined text-[14px] text-white">"check"</span>
                            </div>
                            <span class="font-medium text-slate-900">"Verified Pro"</span>
                        </label>
                        <label class="flex items-center gap-3 cursor-pointer group">
                            <div class="w-5 h-5 rounded border border-slate-300 group-hover:border-slate-900 flex items-center justify-center transition-all"></div>
                            <span>"Free Estimates"</span>
                        </label>
                    </div>
                </div>

                <button class="w-full border-2 border-slate-900 text-slate-900 py-3 rounded-xl font-bold hover:bg-slate-900 hover:text-white transition-colors">
                    "Show 128 results"
                </button>
            </div>
        </div>
    }
}
