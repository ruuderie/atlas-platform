use leptos::prelude::*;
use crate::app::ListingModel;

#[component]
pub fn SearchGrid(
    results: Vec<ListingModel>,
    set_selected: WriteSignal<Option<ListingModel>>,
    view_mode: ReadSignal<String>
) -> impl IntoView {
    let images = vec![
        "https://images.unsplash.com/photo-1581578731548-c64695cc6952?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80",
        "https://images.unsplash.com/photo-1504307651254-35680f356dfd?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80",
        "https://images.unsplash.com/photo-1585704032915-c3400ca199e7?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80",
        "https://images.unsplash.com/photo-1523413363574-c30aa1c2a516?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80",
        "https://images.unsplash.com/photo-1562259929-b4e1fd3aef09?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80",
    ];

    let badges = vec![
        "Top Rated",
        "Licensed & Insured",
        "Best of 2024",
        "New Listing",
        "Verified Pro",
    ];

    let locations = vec![
        "Hartford, CT",
        "Stamford, CT",
        "New Haven, CT",
        "Bridgeport, CT",
        "Danbury, CT",
    ];

    view! {
        <div class="space-y-0">
            {results.clone().into_iter().enumerate().map(move |(i, listing)| {
                let listing_clone1 = listing.clone();
                let listing_clone2 = listing.clone();
                let img = images[i % images.len()];
                let badge = badges[i % badges.len()];
                let location = locations[i % locations.len()];
                let years_exp = match i % 5 { 0 => "15", 1 => "8", 2 => "22", 3 => "12", _ => "5" };
                let projects = match i % 5 { 0 => "200+", 1 => "150+", 2 => "500+", 3 => "300+", _ => "75+" };
                let is_featured = i % 3 == 0;

                view! {
                    <article class="group flex flex-col md:flex-row gap-8 py-10 border-b border-outline-variant/30 last:border-b-0 cursor-pointer" on:click=move |_| set_selected.set(Some(listing_clone1.clone()))>
                        // Image
                        <div class="relative w-full md:w-80 h-64 md:h-auto aspect-[4/3] rounded-sm overflow-hidden flex-shrink-0 bg-surface-container-low">
                            <img class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" src=img alt=listing_clone2.title.clone() />
                        </div>
                        // Details
                        <div class="flex-1 flex flex-col justify-between py-1">
                            <div>
                                <div class="flex items-center gap-3 mb-3">
                                    {if is_featured {
                                        view! {
                                            <span class="text-[10px] font-bold uppercase tracking-widest text-white bg-[#004289] px-3 py-1 rounded-sm">{badge}</span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span class="text-[10px] font-bold uppercase tracking-widest text-on-surface-variant border border-outline-variant px-3 py-1 rounded-sm">{badge}</span>
                                        }.into_any()
                                    }}
                                    <span class="flex items-center gap-1 text-sm text-on-surface-variant">
                                        <span class="material-symbols-outlined text-[16px]">"location_on"</span>
                                        {location}
                                    </span>
                                </div>
                                <h3 class="font-headline text-2xl font-bold text-on-surface mb-3 group-hover:text-[#004289] transition-colors">{listing_clone2.title.clone()}</h3>
                                <p class="text-on-surface-variant leading-relaxed line-clamp-2 mb-6">{listing_clone2.description.clone()}</p>
                            </div>
                            <div class="flex items-center justify-between">
                                <div class="flex items-center gap-6 text-sm text-on-surface-variant">
                                    <span class="flex items-center gap-1.5">
                                        <span class="material-symbols-outlined text-[18px]">"schedule"</span>
                                        {years_exp} " yrs exp"
                                    </span>
                                    <span class="flex items-center gap-1.5">
                                        <span class="material-symbols-outlined text-[18px]">"construction"</span>
                                        {projects} " projects"
                                    </span>
                                </div>
                                <a href=format!("/{}", listing_clone2.id) class="flex items-center gap-1 text-[#004289] font-bold hover:underline underline-offset-4 transition-all">
                                    "View Details"
                                    <span class="material-symbols-outlined text-[18px]">"arrow_forward"</span>
                                </a>
                            </div>
                        </div>
                    </article>
                }
            }).collect_view()}
        </div>
    }
}

#[component]
pub fn RefinementSidebar() -> impl IntoView {
    view! {
        <aside class="w-full space-y-10">
            // Category
            <div>
                <h4 class="flex items-center gap-2 font-headline font-bold text-sm mb-4 text-on-surface uppercase tracking-wider">
                    <span class="material-symbols-outlined text-[18px]">"tune"</span>
                    "Category"
                </h4>
                <div class="space-y-3 font-body text-on-surface-variant">
                    <label class="flex items-center gap-3 cursor-pointer group">
                        <input type="checkbox" checked class="w-4 h-4 accent-[#004289] rounded" />
                        <span class="font-medium text-on-surface">"Kitchen & Bath"</span>
                    </label>
                    <label class="flex items-center gap-3 cursor-pointer group">
                        <input type="checkbox" class="w-4 h-4 accent-[#004289] rounded" />
                        <span>"General Handyman"</span>
                    </label>
                    <label class="flex items-center gap-3 cursor-pointer group">
                        <input type="checkbox" class="w-4 h-4 accent-[#004289] rounded" />
                        <span>"Roofing & Siding"</span>
                    </label>
                    <label class="flex items-center gap-3 cursor-pointer group">
                        <input type="checkbox" class="w-4 h-4 accent-[#004289] rounded" />
                        <span>"Plumbing & HVAC"</span>
                    </label>
                </div>
            </div>

            // Investment Range
            <div>
                <h4 class="flex items-center gap-2 font-headline font-bold text-sm mb-4 text-on-surface uppercase tracking-wider">
                    <span class="material-symbols-outlined text-[18px]">"payments"</span>
                    "Budget Range"
                </h4>
                <div class="px-1">
                    <input type="range" min="500" max="50000" value="10000" class="w-full accent-[#004289] cursor-pointer" />
                    <div class="flex justify-between text-xs text-on-surface-variant mt-2">
                        <span>"$500"</span>
                        <span>"$50K+"</span>
                    </div>
                </div>
            </div>

            // Customer Rating
            <div>
                <h4 class="flex items-center gap-2 font-headline font-bold text-sm mb-4 text-on-surface uppercase tracking-wider">
                    <span class="material-symbols-outlined text-[18px]">"star"</span>
                    "Customer Rating"
                </h4>
                <div class="flex gap-2">
                    <button class="bg-[#004289] text-white px-4 py-2 rounded-full text-xs font-bold flex items-center gap-1">"4.5+ " <span class="material-symbols-outlined text-[14px]">"star"</span></button>
                    <button class="border border-outline-variant text-on-surface-variant px-4 py-2 rounded-full text-xs font-bold hover:border-[#004289] hover:text-[#004289] transition-colors">"4.0+"</button>
                    <button class="border border-outline-variant text-on-surface-variant px-4 py-2 rounded-full text-xs font-bold hover:border-[#004289] hover:text-[#004289] transition-colors">"3.5+"</button>
                </div>
            </div>

            // Map Preview
            <div>
                <h4 class="font-headline font-bold text-sm mb-4 text-on-surface">"Map Preview"</h4>
                <div class="relative h-52 rounded-lg overflow-hidden bg-surface-container-low">
                    <img src="https://images.unsplash.com/photo-1524661135-423995f22d0b?q=80&w=600&auto=format&fit=crop" class="w-full h-full object-cover opacity-60 saturate-50" alt="Map Preview" />
                    <div class="absolute inset-0 flex items-center justify-center">
                        <button class="bg-[#004289] text-white px-6 py-2 rounded-lg font-bold text-sm shadow-lg hover:bg-[#00458f] transition-colors flex items-center gap-2">
                            <span class="material-symbols-outlined text-[18px]">"map"</span>
                            "Browse Map"
                        </button>
                    </div>
                </div>
            </div>
        </aside>
    }
}
