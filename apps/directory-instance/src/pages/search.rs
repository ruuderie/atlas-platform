use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use crate::app::{ListingModel, PaginatedListings};
use crate::components::seo::Seo;
use crate::components::search_ui::{SearchGrid, RefinementSidebar};

#[server]
pub async fn search_listings_from_api(query: String, category: Option<String>, page_str: String) -> Result<PaginatedListings<ListingModel>, ServerFnError> {
    let page = page_str.parse::<u64>().unwrap_or(1);
    let limit = 12;
    
    let mut url = format!("http://127.0.0.1:8000/listings/search?q={}&page={}&limit={}", query, page, limit);
    if let Some(ref c) = category {
        url = format!("{}&category={}", url, c);
    }
    
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await;
    
    match res {
        Ok(r) if r.status().is_success() => {
            Ok(r.json::<PaginatedListings<ListingModel>>().await?)
        },
        _ => {
            let mock_filter = query.to_lowercase();
            let cat_filter = category.unwrap_or_default().to_lowercase();
            
            let mut mocks = Vec::with_capacity(55);
            let categories = ["Contractors", "Plumbers", "HVAC", "Electricians", "Cleaning", "Landscaping"];
            
            for i in 1..=55 {
                let cat = categories[i % categories.len()];
                mocks.push(ListingModel {
                    id: format!("mock-listing-{}", i),
                    listing_type: cat.to_string(),
                    title: format!("{} Premium Service Pro #{}", cat, i),
                    description: format!("We deliver top-quality craftsmanship and reliable services for residential homeowners and commercial businesses. Decades of experience in {} spanning across Connecticut.", cat),
                    attributes: std::collections::HashMap::from([("hero_headline".to_string(), "Your Local Experts".to_string())]),
                });
            }
            
            if !mock_filter.is_empty() {
                mocks.retain(|m| m.title.to_lowercase().contains(&mock_filter) || m.description.to_lowercase().contains(&mock_filter));
            }
            if !cat_filter.is_empty() {
                mocks.retain(|m| m.listing_type.to_lowercase() == cat_filter || m.description.to_lowercase().contains(&cat_filter));
            }
            
            let total = mocks.len() as u64;
            let total_pages = (total as f64 / limit as f64).ceil() as u64;
            let start = ((page - 1) * limit) as usize;
            let end = (start + limit as usize).min(mocks.len());
            let items = if start < mocks.len() { mocks[start..end].to_vec() } else { vec![] };
            
            Ok(PaginatedListings {
                items,
                total,
                page,
                limit,
                total_pages: if total_pages > 0 { total_pages } else { 1 },
            })
        }
    }
}

#[component]
pub fn Search() -> impl IntoView {
    let query = use_query_map();
    let search_term = move || query.with(|q| q.get("q").unwrap_or_default());
    let category_term = move || query.with(|q| q.get("category").map(|s| s.clone()));
    let page_str = move || query.with(|q| q.get("page").unwrap_or_else(|| "1".to_string()));

    let (view_mode, set_view_mode) = signal("grid".to_string());
    let (selected_listing, set_selected_listing) = signal::<Option<ListingModel>>(None);

    let search_resource = Resource::new(
        move || (search_term(), category_term(), page_str()),
        |(q, cat, p)| async move {
            search_listings_from_api(q, cat, p).await
        }
    );

    let config = use_context::<crate::app::DirectoryConfig>().expect("DirectoryConfig context must be available");

    view! {
        <Seo title=format!("{} - Search Results", config.name) />
        
        <crate::components::layout::MainLayout>
        <div class="flex flex-col md:flex-row bg-white text-slate-800 font-body w-full h-[calc(100vh-74px)]">
            
            // Left Pane
            <div class="w-full md:w-[60%] lg:w-[55%] flex flex-col h-full overflow-y-auto custom-scroll">
                // Search Header
                <div class="p-6 md:p-8 border-b border-slate-200 bg-white sticky top-0 z-30">
                    <div class="flex items-center gap-3 mb-6">
                        <a href="/" class="text-slate-500 hover:text-slate-900 bg-slate-100 hover:bg-slate-200 p-2.5 rounded-full transition-all duration-200 flex items-center justify-center">
                            <span class="material-symbols-outlined text-[20px]">"arrow_back"</span>
                        </a>
                        <h1 class="text-2xl font-bold font-headline text-on-primary-fixed tracking-tight">"Find Services"</h1>
                    </div>
                    
                    <form class="relative flex items-center rounded-full overflow-hidden bg-white border border-slate-300 shadow-sm hover:shadow-md focus-within:ring-2 focus-within:ring-slate-900 focus-within:border-transparent transition-all duration-300 max-w-full" action="/search" method="GET">
                        <div class="absolute inset-y-0 left-5 flex items-center pointer-events-none text-slate-500">
                            <span class="material-symbols-outlined text-[20px]">"search"</span>
                        </div>
                        <input 
                            type="text" 
                            name="q" 
                            prop:value=search_term 
                            class="w-full h-14 pl-14 pr-4 bg-transparent text-sm focus:outline-none placeholder:text-slate-500 text-slate-900 font-medium"
                            placeholder="Location, service, or professional name" 
                        />
                        <button type="submit" class="bg-slate-900 text-white h-10 px-6 mx-2 rounded-full font-bold text-sm hover:bg-slate-800 transition-colors flex-shrink-0">
                            "Search"
                        </button>
                    </form>
                    
                    <RefinementSidebar />
                </div>

                // Results Feed
                <div class="p-6 md:p-8 flex-grow bg-white">
                    <Suspense fallback=|| view! { <div class="flex justify-center p-24"><div class="w-8 h-8 border-[3px] border-slate-900 border-t-transparent rounded-full animate-spin"></div></div> }>
                        {move || match search_resource.get() {
                            None => view! { <div/> }.into_any(),
                            Some(Err(e)) => view! { 
                                <div class="max-w-xl mx-auto p-10 text-center bg-rose-50 border border-rose-200 text-rose-700 rounded-3xl">
                                    <h3 class="font-bold font-headline text-xl mb-2">"Search Failed"</h3>
                                    <p class="text-sm opacity-80 font-mono">{e.to_string()}</p>
                                </div> 
                            }.into_any(),
                            Some(Ok(paginated_results)) if paginated_results.items.is_empty() && !search_term().is_empty() => {
                                view! { 
                                    <div class="text-center p-16 max-w-2xl mx-auto bg-slate-50 border border-slate-200 rounded-3xl">
                                        <div class="w-16 h-16 bg-white rounded-full flex items-center justify-center mx-auto mb-6 shadow-sm">
                                            <span class="material-symbols-outlined text-[32px] text-slate-400">"search_off"</span>
                                        </div>
                                        <p class="text-xl text-slate-900 font-bold font-headline">"No results for '" <span class="text-slate-500">{search_term()}</span> "'."</p>
                                        <p class="mt-3 text-slate-500 text-sm font-body">"Try adjusting your search or filters to find what you're looking for."</p>
                                        <button class="mt-8 bg-white border border-slate-300 text-slate-900 hover:bg-slate-50 hover:border-slate-900 px-6 py-3 rounded-full font-bold text-sm transition-all shadow-sm">"Clear All Filters"</button>
                                    </div>
                                }.into_any()
                            },
                            Some(Ok(paginated_results)) if paginated_results.items.is_empty() => {
                                view! { 
                                    <div class="text-center p-16 max-w-2xl mx-auto">
                                        <div class="w-16 h-16 bg-slate-100 rounded-full flex items-center justify-center mx-auto mb-6">
                                            <span class="material-symbols-outlined text-[32px] text-slate-900">"travel_explore"</span>
                                        </div>
                                        <h2 class="text-2xl text-slate-900 font-bold font-headline">"Start Your Search"</h2>
                                        <p class="mt-3 text-slate-500 text-sm max-w-md mx-auto font-body">"Enter a location or service type above to discover premium professionals tailored to your project."</p>
                                    </div>
                                }.into_any()
                            },
                            Some(Ok(paginated_results)) => {
                                let results_count = paginated_results.total;
                                let results = paginated_results.items.clone();
                                let total_pages = paginated_results.total_pages;
                                let current_page = paginated_results.page;
                                
                                let items_json = results.iter()
                                    .enumerate()
                                    .map(|(i, r)| format!(r#"{{"@type":"ListItem","position":{},"url":"https://{}/{}"}}"#, i + 1, config.domain, r.id))
                                    .collect::<Vec<String>>()
                                    .join(",");
                                let json_ld = format!(r#"{{ "@context": "https://schema.org", "@type": "ItemList", "itemListElement": [{}] }}"#, items_json);
                                
                                view! {
                                    <crate::components::seo::Seo 
                                        title=format!("{} ({}) - {}", search_term(), results_count, config.name)
                                        description="Search through premium local specialists.".to_string()
                                        script_json_ld=json_ld
                                    />
                                    <div class="space-y-6 animate-slide-up">
                                        <div class="flex items-center justify-between pb-4 mb-5">
                                            <h3 class="text-base font-bold font-headline text-slate-800 tracking-tight">
                                                <span>"Over "</span>
                                                <span>{results_count}</span>
                                                <span>" places"</span>
                                            </h3>
                                            <div class="flex items-center gap-3">
                                                <div class="flex p-1 bg-slate-100 rounded-full">
                                                    <button 
                                                        class=move || format!("p-2 rounded-full transition-all duration-200 flex items-center justify-center {}", if view_mode.get() == "list" { "bg-white text-slate-900 shadow-sm" } else { "text-slate-500 hover:text-slate-900" })
                                                        on:click=move |e| { e.prevent_default(); set_view_mode.set("list".to_string()); }
                                                    >
                                                        <span class="material-symbols-outlined text-[18px]">"view_list"</span>
                                                    </button>
                                                    <button 
                                                        class=move || format!("p-2 rounded-full transition-all duration-200 flex items-center justify-center {}", if view_mode.get() == "grid" { "bg-white text-slate-900 shadow-sm" } else { "text-slate-500 hover:text-slate-900" })
                                                        on:click=move |e| { e.prevent_default(); set_view_mode.set("grid".to_string()); }
                                                    >
                                                        <span class="material-symbols-outlined text-[18px]">"grid_view"</span>
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                        
                                        <SearchGrid results=results set_selected=set_selected_listing view_mode=view_mode />
                                        
                                        // Pagination
                                        <div class="mt-12 flex justify-center pb-12">
                                            {if total_pages > 1 {
                                                view! {
                                                    <nav class="inline-flex gap-2">
                                                        <a href=move || {
                                                            let prev = if current_page > 1 { current_page - 1 } else { 1 };
                                                            format!("?q={}&category={}&page={}", search_term(), category_term().unwrap_or_default(), prev)
                                                        } class=move || format!("w-10 h-10 flex items-center justify-center rounded-full transition-colors {}", if current_page == 1 { "text-slate-300 cursor-not-allowed pointer-events-none" } else { "text-slate-600 hover:bg-slate-100" })>
                                                            <span class="material-symbols-outlined text-[20px]">"chevron_left"</span>
                                                        </a>
                                                        
                                                        // Render up to 5 page squares around current_page
                                                        {
                                                            let start_p = if current_page > 2 { current_page - 2 } else { 1 };
                                                            let end_p = (start_p + 4).min(total_pages);
                                                            let mut pg_views = Vec::new();
                                                            for p in start_p..=end_p {
                                                                pg_views.push(view! {
                                                                    <a href=format!("?q={}&category={}&page={}", search_term(), category_term().unwrap_or_default(), p) 
                                                                       class=if p == current_page {
                                                                           "w-10 h-10 flex items-center justify-center font-bold text-sm bg-slate-900 text-white rounded-full"
                                                                       } else {
                                                                           "w-10 h-10 flex items-center justify-center font-medium text-sm text-slate-600 hover:bg-slate-100 rounded-full transition-colors"
                                                                       }
                                                                    >
                                                                        {p}
                                                                    </a>
                                                                });
                                                            }
                                                            pg_views
                                                        }
                                                        
                                                        <a href=move || {
                                                            let next = if current_page < total_pages { current_page + 1 } else { total_pages };
                                                            format!("?q={}&category={}&page={}", search_term(), category_term().unwrap_or_default(), next)
                                                        } class=move || format!("w-10 h-10 flex items-center justify-center rounded-full transition-colors {}", if current_page == total_pages { "text-slate-300 cursor-not-allowed pointer-events-none" } else { "text-slate-600 hover:bg-slate-100" })>
                                                            <span class="material-symbols-outlined text-[20px]">"chevron_right"</span>
                                                        </a>
                                                    </nav>
                                                }.into_any()
                                            } else {
                                                view! { <div/> }.into_any()
                                            }}
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </div>
            
            // Right Pane (Map / Preview)
            <div class="hidden md:block md:w-[40%] lg:w-[45%] h-full sticky top-0 bg-slate-100 border-l border-slate-200 relative overflow-y-auto custom-scroll">
                {move || {
                    if let Some(listing) = selected_listing.get() {
                        view! {
                            <div class="min-h-full w-full bg-slate-50 flex flex-col items-center justify-center p-8 animate-fade-scale">
                                <div class="w-full max-w-lg bg-white rounded-3xl shadow-xl overflow-hidden relative group">
                                    <div class="h-64 bg-slate-200 relative overflow-hidden">
                                        <img src="https://images.unsplash.com/photo-1600596542815-ffad4c1539a9?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80" 
                                             class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-700" alt="Preview Image" />
                                        
                                        <button class="absolute top-4 right-4 bg-white/90 backdrop-blur-md p-2 rounded-full hover:bg-white text-slate-800 transition-all duration-200 z-20 shadow-sm" on:click=move |_| set_selected_listing.set(None)>
                                            <span class="material-symbols-outlined text-[20px]">"close"</span>
                                        </button>
                                        <div class="absolute bottom-5 left-5 z-20">
                                            <div class="bg-white/95 backdrop-blur-sm px-3 py-1 text-xs font-bold text-slate-800 uppercase rounded-full shadow-sm mb-2 inline-block">
                                                {listing.listing_type.clone()}
                                            </div>
                                        </div>
                                    </div>
                                    <div class="p-8">
                                        <h2 class="text-2xl font-bold font-headline text-on-primary-fixed leading-tight mb-3">{listing.title.clone()}</h2>
                                        <p class="text-slate-600 text-sm font-body leading-relaxed mb-8">{listing.description.clone()}</p>
                                        
                                        <div class="space-y-4">
                                            <a href=format!("/{}", listing.id) class="w-full flex items-center justify-center gap-2 py-4 bg-slate-900 text-white text-sm font-bold rounded-xl hover:bg-slate-800 transition-all duration-300">
                                                <span>"View Full Profile"</span>
                                                <span class="material-symbols-outlined text-[20px]">"arrow_forward"</span>
                                            </a>
                                            <button class="w-full flex items-center justify-center py-4 bg-white border border-slate-300 text-slate-700 font-bold text-sm rounded-xl hover:bg-slate-50 transition-all duration-200" on:click=move |_| set_selected_listing.set(None)>
                                                "Back Setup"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="w-full h-full relative">
                                // Nice map abstract background
                                <img src="https://images.unsplash.com/photo-1524661135-423995f22d0b?q=80&w=1200&auto=format&fit=crop" class="w-full h-full object-cover opacity-60 saturate-50" alt="Map Area" />
                                
                                <div class="absolute inset-0 flex flex-col justify-between p-6 pb-20">
                                    // Top map controls
                                    <div class="flex justify-between items-start pt-2">
                                        <div class="bg-white px-4 py-2 rounded-full shadow-sm flex items-center gap-2">
                                            <span class="w-2 h-2 rounded-full bg-green-500"></span>
                                            <span class="text-xs font-bold text-slate-700">"Search as I move the map"</span>
                                        </div>
                                        <div class="flex flex-col gap-2">
                                            <button class="w-10 h-10 bg-white rounded-lg shadow-sm flex items-center justify-center hover:bg-slate-50 text-slate-700 transition">
                                                <span class="material-symbols-outlined text-[20px]">"add"</span>
                                            </button>
                                            <button class="w-10 h-10 bg-white rounded-lg shadow-sm flex items-center justify-center hover:bg-slate-50 text-slate-700 transition">
                                                <span class="material-symbols-outlined text-[20px]">"remove"</span>
                                            </button>
                                        </div>
                                    </div>
                                    
                                    // Map Prompt
                                    <div class="bg-white/95 backdrop-blur-md p-6 rounded-3xl shadow-lg border border-white/50 text-center max-w-sm mx-auto self-center">
                                        <div class="w-14 h-14 bg-slate-100 rounded-full flex items-center justify-center mx-auto mb-4">
                                            <span class="material-symbols-outlined text-[28px] text-slate-900">"map"</span>
                                        </div>
                                        <h4 class="text-lg font-bold font-headline text-slate-900">"Interactive Map"</h4>
                                        <p class="text-sm text-slate-600 font-body mt-2 leading-relaxed">"Hover over listings on the left to see them highlighted on the map."</p>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        </div>
        </crate::components::layout::MainLayout>
    }
}
