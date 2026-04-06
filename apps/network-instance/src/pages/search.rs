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
        Ok(r) => {
            Err(ServerFnError::ServerError(format!("API Error: Status {}", r.status())))
        },
        Err(e) => {
            Err(ServerFnError::ServerError(format!("Request Error: {}", e)))
        }
    }
}

#[component]
pub fn Search() -> impl IntoView {
    let query = use_query_map();
    let search_term = move || query.with(|q| q.get("q").unwrap_or_default());
    let category_term = move || query.with(|q| q.get("category").map(|s| s.clone()));
    let page_str = move || query.with(|q| q.get("page").unwrap_or_else(|| "1".to_string()));

    let (_selected_listing, set_selected_listing) = signal::<Option<ListingModel>>(None);

    let search_resource = Resource::new(
        move || (search_term(), category_term(), page_str()),
        |(q, cat, p)| async move {
            search_listings_from_api(q, cat, p).await
        }
    );

    let config = use_context::<crate::app::NetworkConfig>().expect("NetworkConfig context must be available");

    view! {
        <Seo title=format!("{} - Search Results", config.name) />
        
        <crate::components::layout::MainLayout>
            // Page Header
            <div class="bg-surface py-12 px-8">
                <div class="max-w-7xl mx-auto">
                    <span class="text-tertiary font-bold tracking-widest text-xs uppercase mb-3 block">"Service Providers"</span>
                    <div class="flex items-end justify-between">
                        <div>
                            <h1 class="font-headline text-4xl md:text-5xl font-extrabold tracking-tight text-on-surface mb-4">"Find Local Pros"</h1>
                            <p class="text-on-surface-variant max-w-xl leading-relaxed">"Browse verified renovation contractors, handymen, and home service professionals across Connecticut."</p>
                        </div>
                        <Suspense fallback=|| view! { <span/> }>
                            {move || match search_resource.get() {
                                Some(Ok(ref res)) => view! {
                                    <div class="hidden md:flex items-center gap-2 bg-surface-container-lowest px-5 py-3 rounded-lg text-sm shadow-sm">
                                        <span class="text-[#004289] font-extrabold text-lg">{res.total}</span>
                                        <span class="text-on-surface-variant font-medium">"Results Found"</span>
                                    </div>
                                }.into_any(),
                                _ => view! { <span/> }.into_any(),
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>

            // Main Content: Sidebar + Results
            <div class="max-w-7xl mx-auto px-8 py-12 flex gap-16">
                // Left Sidebar
                <div class="hidden lg:block w-64 flex-shrink-0">
                    <RefinementSidebar />
                </div>

                // Results Area
                <div class="flex-1 min-w-0">
                    <Suspense fallback=|| view! { <div class="flex justify-center p-24"><div class="w-8 h-8 border-[3px] border-[#004289] border-t-transparent rounded-full animate-spin"></div></div> }>
                        {move || match search_resource.get() {
                            None => view! { <div/> }.into_any(),
                            Some(Err(e)) => view! { 
                                <div class="max-w-xl mx-auto p-10 text-center bg-error-container text-on-error-container rounded-lg">
                                    <h3 class="font-bold font-headline text-xl mb-2">"Search Failed"</h3>
                                    <p class="text-sm opacity-80">{e.to_string()}</p>
                                </div> 
                            }.into_any(),
                            Some(Ok(paginated_results)) if paginated_results.items.is_empty() => {
                                view! { 
                                    <div class="text-center py-24 max-w-lg mx-auto">
                                        <div class="w-16 h-16 bg-surface-container rounded-full flex items-center justify-center mx-auto mb-6">
                                            <span class="material-symbols-outlined text-[32px] text-outline">"travel_explore"</span>
                                        </div>
                                        <h2 class="font-headline text-2xl font-bold text-on-surface mb-3">"No Results Found"</h2>
                                        <p class="text-on-surface-variant">"Try adjusting your filters or search criteria to discover more service providers."</p>
                                    </div>
                                }.into_any()
                            },
                            Some(Ok(paginated_results)) => {
                                let results = paginated_results.items.clone();
                                let total_pages = paginated_results.total_pages;
                                let current_page = paginated_results.page;
                                
                                view! {
                                    <div class="animate-slide-up">
                                        <SearchGrid results=results set_selected=set_selected_listing />
                                        
                                        // Pagination
                                        <div class="mt-16 flex justify-center pb-12">
                                            {if total_pages > 1 {
                                                view! {
                                                    <nav class="inline-flex items-center gap-2">
                                                        <a href=move || {
                                                            let prev = if current_page > 1 { current_page - 1 } else { 1 };
                                                            format!("?q={}&category={}&page={}", search_term(), category_term().unwrap_or_default(), prev)
                                                        } class=move || format!("w-10 h-10 flex items-center justify-center rounded-lg transition-colors {}", if current_page == 1 { "text-outline-variant cursor-not-allowed pointer-events-none" } else { "text-on-surface hover:bg-surface-container" })>
                                                            <span class="material-symbols-outlined text-[20px]">"chevron_left"</span>
                                                        </a>
                                                        
                                                        {
                                                            let start_p = if current_page > 2 { current_page - 2 } else { 1 };
                                                            let end_p = (start_p + 4).min(total_pages);
                                                            let mut pg_views: Vec<AnyView> = Vec::new();
                                                            for p in start_p..=end_p {
                                                                pg_views.push(view! {
                                                                    <a href=format!("?q={}&category={}&page={}", search_term(), category_term().unwrap_or_default(), p) 
                                                                       class=if p == current_page {
                                                                           "w-10 h-10 flex items-center justify-center font-bold text-sm bg-[#004289] text-white rounded-lg"
                                                                       } else {
                                                                           "w-10 h-10 flex items-center justify-center font-medium text-sm text-on-surface-variant hover:bg-surface-container rounded-lg transition-colors"
                                                                       }
                                                                    >
                                                                        {p}
                                                                    </a>
                                                                }.into_any());
                                                            }
                                                            // Ellipsis if needed
                                                            if end_p < total_pages {
                                                                pg_views.push(view! {
                                                                    <span class="w-10 h-10 flex items-center justify-center text-on-surface-variant">"..."</span>
                                                                }.into_any());
                                                            }
                                                            pg_views
                                                        }
                                                        
                                                        <a href=move || {
                                                            let next = if current_page < total_pages { current_page + 1 } else { total_pages };
                                                            format!("?q={}&category={}&page={}", search_term(), category_term().unwrap_or_default(), next)
                                                        } class=move || format!("w-10 h-10 flex items-center justify-center rounded-lg transition-colors {}", if current_page == total_pages { "text-outline-variant cursor-not-allowed pointer-events-none" } else { "text-on-surface hover:bg-surface-container" })>
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
        </crate::components::layout::MainLayout>
    }
}
