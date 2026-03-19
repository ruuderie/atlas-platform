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
        <div class="flex flex-col md:flex-row bg-background text-foreground font-sans w-full h-[calc(100vh-74px)]">
            
            // Left Pane
            <div class="w-full md:w-[60%] lg:w-[55%] flex flex-col h-full overflow-y-auto custom-scroll">
                // Search Header — glass with gradient top accent
                <div class="p-6 md:p-8 border-b border-neutral-200/60 bg-white/80 backdrop-blur-xl sticky top-0 z-30">
                    <div class="h-[2px] w-full bg-gradient-to-r from-primary via-purple-500 to-pink-500 opacity-30 absolute top-0 left-0 right-0"></div>
                    <div class="flex items-center gap-3 mb-5">
                        <a href="/" class="text-neutral-400 hover:text-foreground hover:bg-neutral-100 p-2 rounded-xl transition-all duration-200 flex items-center justify-center">
                            <svg class="w-5 h-5 stroke-2" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M10 19l-7-7m0 0l7-7m-7 7h18"></path></svg>
                        </a>
                        <h1 class="text-xl font-bold tracking-tight">"Find Services"</h1>
                    </div>
                    
                    <form class="relative flex rounded-xl overflow-hidden bg-white border border-neutral-200 focus-within:border-primary/40 focus-within:shadow-glow transition-all duration-300 max-w-full" action="/search" method="GET">
                        <div class="absolute inset-y-0 left-4 flex items-center pointer-events-none text-neutral-400">
                            <svg class="w-5 h-5 stroke-[1.5]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
                        </div>
                        <input 
                            type="text" 
                            name="q" 
                            prop:value=search_term 
                            class="w-full h-12 pl-12 pr-4 bg-transparent text-sm focus:outline-none placeholder:text-neutral-400 text-foreground font-medium"
                            placeholder="Search by location, category, or keyword..." 
                        />
                        <button type="submit" class="bg-gradient-to-r from-primary to-primary/90 text-white h-12 px-6 font-semibold text-sm hover:opacity-90 transition-opacity flex-shrink-0">
                            "Search"
                        </button>
                    </form>
                    
                    <RefinementSidebar />
                </div>

                // Results Feed
                <div class="p-6 md:p-8 flex-grow bg-neutral-50/50">
                    <Suspense fallback=|| view! { <div class="flex justify-center p-24"><div class="w-8 h-8 border-[3px] border-primary border-t-transparent rounded-full animate-spin"></div></div> }>
                        {move || match search_resource.get() {
                            None => view! { <div/> }.into_any(),
                            Some(Err(e)) => view! { 
                                <div class="max-w-xl mx-auto p-10 text-center bg-white border border-destructive/20 text-destructive rounded-2xl shadow-premium">
                                    <h3 class="font-bold text-xl mb-2">"Search Failed"</h3>
                                    <p class="text-sm opacity-80 font-mono">{e.to_string()}</p>
                                </div> 
                            }.into_any(),
                            Some(Ok(paginated_results)) if paginated_results.items.is_empty() && !search_term().is_empty() => {
                                view! { 
                                    <div class="text-center p-16 max-w-2xl mx-auto bg-white border border-neutral-200 rounded-2xl shadow-premium">
                                        <div class="w-16 h-16 bg-neutral-100 rounded-2xl flex items-center justify-center mx-auto mb-6">
                                            <svg class="w-8 h-8 text-neutral-400" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                                        </div>
                                        <p class="text-xl text-foreground font-bold">"No results for '" <span class="text-primary">{search_term()}</span> "'."</p>
                                        <p class="mt-3 text-neutral-500 text-sm">"Try removing filters or expanding your search."</p>
                                        <button class="mt-6 bg-transparent border border-primary/30 text-primary hover:bg-primary/5 px-5 py-2 rounded-xl font-semibold text-sm transition-all">"Clear All Filters"</button>
                                    </div>
                                }.into_any()
                            },
                            Some(Ok(paginated_results)) if paginated_results.items.is_empty() => {
                                view! { 
                                    <div class="text-center p-16 max-w-2xl mx-auto">
                                        <div class="w-14 h-14 bg-primary/10 rounded-2xl flex items-center justify-center mx-auto mb-5">
                                            <svg class="w-7 h-7 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
                                        </div>
                                        <h2 class="text-xl text-foreground font-bold">"Start Your Search"</h2>
                                        <p class="mt-2 text-neutral-500 text-sm max-w-md mx-auto">"Enter a location or service type above to discover premium professionals."</p>
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
                                        <div class="flex items-center justify-between pb-4 border-b border-neutral-200/60 mb-5">
                                            <h3 class="text-base font-semibold tracking-tight text-foreground">
                                                <span class="text-neutral-500 font-normal">"Showing "</span>
                                                <span class="text-primary font-bold">{results_count}</span>
                                                <span class="text-neutral-500 font-normal">" results"</span>
                                            </h3>
                                            <div class="flex items-center gap-3">
                                                <div class="flex p-0.5 bg-neutral-100 rounded-lg">
                                                    <button 
                                                        class=move || format!("p-1.5 rounded-md transition-all duration-200 {}", if view_mode.get() == "list" { "bg-white text-foreground shadow-sm" } else { "text-neutral-400 hover:text-foreground" })
                                                        on:click=move |e| { e.prevent_default(); set_view_mode.set("list".to_string()); }
                                                    >
                                                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path></svg>
                                                    </button>
                                                    <button 
                                                        class=move || format!("p-1.5 rounded-md transition-all duration-200 {}", if view_mode.get() == "grid" { "bg-white text-foreground shadow-sm" } else { "text-neutral-400 hover:text-foreground" })
                                                        on:click=move |e| { e.prevent_default(); set_view_mode.set("grid".to_string()); }
                                                    >
                                                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"></path></svg>
                                                    </button>
                                                </div>
                                                <select class="text-[13px] font-medium bg-transparent border-none text-neutral-500 outline-none cursor-pointer hover:text-foreground transition-colors hidden sm:block">
                                                    <option>"Recommended"</option>
                                                    <option>"Highest Rated"</option>
                                                </select>
                                            </div>
                                        </div>
                                        
                                        <SearchGrid results=results set_selected=set_selected_listing view_mode=view_mode />
                                        
                                        // Pagination
                                        <div class="mt-10 flex justify-center pb-10">
                                            {if total_pages > 1 {
                                                view! {
                                                    <nav class="inline-flex rounded-xl shadow-premium border border-neutral-200/60 bg-white p-1 gap-0.5">
                                                        <a href=move || {
                                                            let prev = if current_page > 1 { current_page - 1 } else { 1 };
                                                            format!("?q={}&category={}&page={}", search_term(), category_term().unwrap_or_default(), prev)
                                                        } class=move || format!("px-3.5 py-1.5 font-medium text-[13px] rounded-lg transition-colors {}", if current_page == 1 { "text-neutral-400 cursor-not-allowed pointer-events-none" } else { "text-neutral-600 hover:bg-neutral-100" })>
                                                            "Prev"
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
                                                                           "px-3.5 py-1.5 font-bold text-[13px] bg-gradient-to-br from-primary to-primary/90 text-white rounded-lg shadow-sm"
                                                                       } else {
                                                                           "px-3.5 py-1.5 font-medium text-[13px] text-neutral-600 hover:bg-neutral-100 rounded-lg transition-colors"
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
                                                        } class=move || format!("px-3.5 py-1.5 font-medium text-[13px] rounded-lg transition-colors {}", if current_page == total_pages { "text-neutral-400 cursor-not-allowed pointer-events-none" } else { "text-neutral-600 hover:bg-neutral-100" })>
                                                            "Next"
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
            <div class="hidden md:block md:w-[40%] lg:w-[45%] h-full sticky top-0 bg-neutral-100 border-l border-neutral-200/60 relative overflow-y-auto custom-scroll">
                {move || {
                    if let Some(listing) = selected_listing.get() {
                        view! {
                            <div class="min-h-full w-full bg-neutral-50 flex flex-col items-center justify-start p-8 animate-fade-scale">
                                <div class="w-full max-w-lg bg-white rounded-2xl shadow-premium-lg border border-neutral-200/60 overflow-hidden relative group">
                                    <div class="h-72 bg-neutral-100 relative overflow-hidden">
                                        <img src="https://images.unsplash.com/photo-1513694203232-719a280e022f?q=80&w=800&auto=format&fit=crop" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-700" alt="Preview Image" />
                                        <div class="absolute inset-0 bg-gradient-to-t from-black/50 via-transparent to-transparent"></div>
                                        <button class="absolute top-4 right-4 bg-white/90 backdrop-blur-md p-2 rounded-xl hover:bg-white text-neutral-500 hover:text-foreground transition-all duration-200 z-20 shadow-sm" on:click=move |_| set_selected_listing.set(None)>
                                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
                                        </button>
                                        <div class="absolute bottom-5 left-5 z-20">
                                            <div class="bg-gradient-to-r from-primary to-primary/80 px-3 py-1 text-[11px] font-bold text-white uppercase rounded-lg shadow-sm mb-2 inline-block tracking-wider">{listing.listing_type.clone()}</div>
                                            <h2 class="text-2xl font-bold text-white leading-tight drop-shadow-md">{listing.title.clone()}</h2>
                                        </div>
                                    </div>
                                    <div class="p-7">
                                        <p class="text-neutral-500 text-sm leading-relaxed mb-6">{listing.description.clone()}</p>
                                        
                                        <div class="space-y-3">
                                            <a href=format!("/{}", listing.id) class="w-full flex items-center justify-center gap-2 py-3.5 bg-gradient-to-r from-primary to-primary/90 text-white text-sm font-bold rounded-xl shadow-glow hover:shadow-glow-lg hover:opacity-95 transition-all duration-300 cursor-pointer">
                                                <span>"View Full Profile"</span>
                                                <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14 5l7 7m0 0l-7 7m7-7H3"></path></svg>
                                            </a>
                                            <button class="w-full flex items-center justify-center py-3 bg-white border border-neutral-200 text-neutral-600 font-semibold text-sm rounded-xl hover:bg-neutral-50 hover:border-neutral-300 transition-all duration-200 cursor-pointer" on:click=move |_| set_selected_listing.set(None)>
                                                "Back to Search"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="w-full h-full relative">
                                <img src="https://images.unsplash.com/photo-1524661135-423995f22d0b?q=80&w=1200&auto=format&fit=crop" class="w-full h-full object-cover opacity-50 saturate-50" alt="Map Area" />
                                <div class="absolute inset-0 bg-white/20 flex items-center justify-center backdrop-blur-[3px]">
                                    <div class="bg-white p-8 rounded-2xl shadow-premium-lg border border-neutral-200/60 text-center max-w-sm animate-fade-scale">
                                        <div class="w-14 h-14 bg-primary/10 rounded-2xl flex items-center justify-center mx-auto mb-4">
                                            <svg class="w-7 h-7 text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"></path></svg>
                                        </div>
                                        <h4 class="text-lg font-bold text-foreground">"Interactive Map"</h4>
                                        <p class="text-[13px] text-neutral-500 mt-2 leading-relaxed">"Select a listing to view details. Map syncs with your search."</p>
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
