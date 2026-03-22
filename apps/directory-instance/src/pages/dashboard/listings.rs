use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use crate::auth::get_auth_token;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DashboardListingModel {
    pub id: String,
    pub title: String,
    pub listing_type: String,
    #[serde(default)]
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateListingInput {
    pub title: String,
    pub description: String,
    pub listing_type: String,
    pub city: String,
    pub state: String,
    pub price: Option<f64>,
    pub directory_id: String,
}

#[server]
pub async fn fetch_my_listings_api(token: Option<String>) -> Result<Vec<DashboardListingModel>, ServerFnError> {
    let active_token = if let Some(t) = token { t } else if let Some(t) = get_auth_token() { t } else { return Ok(vec![]); };

    let url = "http://127.0.0.1:8000/api/listings/my-listings";
    let client = reqwest::Client::new();
    let res = client.get(url)
        .header("Authorization", format!("Bearer {}", active_token))
        .send()
        .await?;
    
    if res.status().is_success() {
        Ok(res.json::<Vec<DashboardListingModel>>().await?)
    } else {
        Ok(vec![])
    }
}

#[server]
pub async fn create_listing_api(token: Option<String>, payload: CreateListingInput) -> Result<(), ServerFnError> {
    let active_token = if let Some(t) = token { t } else if let Some(t) = get_auth_token() { t } else { return Err(ServerFnError::ServerError("Unauthorized".into())); };

    let url = "http://127.0.0.1:8000/api/listings/my-listings";
    let client = reqwest::Client::new();
    let res = client.post(url)
        .header("Authorization", format!("Bearer {}", active_token))
        .json(&payload)
        .send()
        .await?;
    
    if res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::ServerError("Failed to create".into()))
    }
}

#[component]
pub fn DashboardListings() -> impl IntoView {
    let (show_form, set_show_form) = signal(false);
    let (trigger, set_trigger) = signal(0);
    
    let token = get_auth_token();
    let token2 = token.clone();
    
    let listings_resource = Resource::new(
        move || trigger.get(),
        move |_| {
            let t = token.clone();
            async move { fetch_my_listings_api(t).await }
        }
    );

    // Form states
    let title = RwSignal::new("".to_string());
    let description = RwSignal::new("".to_string());
    let listing_type = RwSignal::new("General Service".to_string());
    let city = RwSignal::new("".to_string());
    let state = RwSignal::new("".to_string());
    let price = RwSignal::new("".to_string());
    
    let is_submitting = RwSignal::new(false);
    let error = RwSignal::new("".to_string());

    let handle_submit = Callback::new(move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if is_submitting.get() { return; }
        
        let p_val = price.get().parse::<f64>().ok();
        
        let payload = CreateListingInput {
            title: title.get(),
            description: description.get(),
            listing_type: listing_type.get(),
            city: city.get(),
            state: state.get(),
            price: p_val,
            directory_id: "".to_string(), // Can be dummy since the backend uses profile's directory ID as source of truth now
        };
        
        is_submitting.set(true);
        error.set("".to_string());
        
        // Use standard window.location to redirect just to guarantee full refresh
        // Alternatively to trigger the refresh via leptos task:
        let token = token2.clone(); // Use token2 from the component's scope
        let trigger_refresh = set_trigger.clone(); // Use set_trigger for refresh
        leptos::task::spawn_local(async move {
            match create_listing_api(token, payload).await {
                Ok(_) => {
                    // Reset form
                    title.set("".to_string());
                    description.set("".to_string());
                    city.set("".to_string());
                    state.set("".to_string());
                    price.set("".to_string());
                    set_show_form.set(false);
                    // trigger refresh of listings
                    trigger_refresh.update(|v| *v += 1);
                }
                Err(e) => error.set(e.to_string()),
            }
            is_submitting.set(false);
        });
    });

    view! {
        <div class="space-y-6 animate-fade-scale">
            <div class="flex justify-between items-center mb-8">
                <h1 class="text-3xl font-headline font-extrabold text-on-surface tracking-tight">"My Listings"</h1>
                <button class="bg-[#004289] text-white px-4 py-2 rounded-lg font-bold hover:bg-[#00336b] transition-colors shadow-sm flex items-center gap-2"
                        on:click=move |_| set_show_form.update(|v| *v = !*v)>
                    {move || if show_form.get() { 
                        view!{ <span class="material-symbols-outlined text-sm">"close"</span>"Cancel" }.into_any() 
                    } else { 
                        view!{ <span class="material-symbols-outlined text-sm">"add"</span>"Create Listing" }.into_any() 
                    }}
                </button>
            </div>
            
            {move || if show_form.get() {
                view! {
                    <div class="bg-white p-6 rounded-2xl shadow-sm border border-outline-variant/30 mb-8">
                        <h2 class="text-xl font-headline font-bold text-on-surface mb-6">"Create New Listing"</h2>
                        <form on:submit=move |ev| handle_submit.run(ev) class="space-y-4">
                            {move || if !error.get().is_empty() {
                                view! { <div class="bg-error/10 text-error p-3 rounded-xl text-sm">{error.get()}</div> }.into_any()
                            } else { view! { <span/> }.into_any() }}
                            
                            <div>
                                <label class="block text-xs font-bold text-on-surface-variant uppercase mb-2">"Listing Title"</label>
                                <input type="text" required class="w-full px-4 py-3 border border-outline-variant/50 rounded-xl bg-surface-container-lowest focus:ring-2 focus:ring-[#004289] outline-none" placeholder="e.g. Acme Plumbing Co." prop:value=move || title.get() on:input=move |ev| title.set(event_target_value(&ev)) />
                            </div>
                            
                            <div>
                                <label class="block text-xs font-bold text-on-surface-variant uppercase mb-2">"Description"</label>
                                <textarea required rows="4" class="w-full px-4 py-3 border border-outline-variant/50 rounded-xl bg-surface-container-lowest focus:ring-2 focus:ring-[#004289] outline-none" placeholder="Describe your services..." prop:value=move || description.get() on:input=move |ev| description.set(event_target_value(&ev))></textarea>
                            </div>
                            
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div>
                                    <label class="block text-xs font-bold text-on-surface-variant uppercase mb-2">"Service Type / Category"</label>
                                    <input type="text" required class="w-full px-4 py-3 border border-outline-variant/50 rounded-xl bg-surface-container-lowest focus:ring-2 focus:ring-[#004289] outline-none" placeholder="e.g. Plumbing" prop:value=move || listing_type.get() on:input=move |ev| listing_type.set(event_target_value(&ev)) />
                                </div>
                                <div>
                                    <label class="block text-xs font-bold text-on-surface-variant uppercase mb-2">"Starting Price (Optional)"</label>
                                    <input type="number" step="0.01" class="w-full px-4 py-3 border border-outline-variant/50 rounded-xl bg-surface-container-lowest focus:ring-2 focus:ring-[#004289] outline-none" placeholder="0.00" prop:value=move || price.get() on:input=move |ev| price.set(event_target_value(&ev)) />
                                </div>
                            </div>
                            
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div>
                                    <label class="block text-xs font-bold text-on-surface-variant uppercase mb-2">"City"</label>
                                    <input type="text" required class="w-full px-4 py-3 border border-outline-variant/50 rounded-xl bg-surface-container-lowest focus:ring-2 focus:ring-[#004289] outline-none" placeholder="Hartford" prop:value=move || city.get() on:input=move |ev| city.set(event_target_value(&ev)) />
                                </div>
                                <div>
                                    <label class="block text-xs font-bold text-on-surface-variant uppercase mb-2">"State"</label>
                                    <input type="text" required class="w-full px-4 py-3 border border-outline-variant/50 rounded-xl bg-surface-container-lowest focus:ring-2 focus:ring-[#004289] outline-none" placeholder="CT" prop:value=move || state.get() on:input=move |ev| state.set(event_target_value(&ev)) />
                                </div>
                            </div>
                            
                            <div class="pt-4 flex justify-end">
                                <button type="submit" disabled=move || is_submitting.get() class="bg-[#004289] text-white px-8 py-3 rounded-xl font-bold hover:bg-[#00336b] transition-colors shadow-sm disabled:opacity-50">
                                    {move || if is_submitting.get() { "Saving..." } else { "Save Listing" }}
                                </button>
                            </div>
                        </form>
                    </div>
                }.into_any()
            } else { view! { <span/> }.into_any() }}

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <Suspense fallback=|| view! { <div class="col-span-full py-12 flex justify-center"><div class="animate-spin w-8 h-8 border-4 border-[#004289] border-t-transparent rounded-full"></div></div> }>
                    {move || match listings_resource.get() {
                        None => view! { <div/> }.into_any(),
                        Some(Err(e)) => view! { <div class="col-span-full text-error p-6 bg-error/5 rounded-xl border border-error/20">"Failed to load listings: " {e.to_string()}</div> }.into_any(),
                        Some(Ok(items)) => {
                            if items.is_empty() {
                                view! {
                                    <div class="col-span-full py-16 text-center bg-white rounded-2xl border border-outline-variant/30 flex flex-col items-center justify-center">
                                        <div class="w-16 h-16 bg-surface-container rounded-full flex items-center justify-center text-on-surface-variant mb-4">
                                            <span class="material-symbols-outlined text-3xl">"inventory_2"</span>
                                        </div>
                                        <h3 class="font-headline font-bold text-xl text-on-surface mb-2">"No Listings Yet"</h3>
                                        <p class="text-on-surface-variant text-sm max-w-sm mb-6">"You don't have any active listings right now. Create one to be discovered by homeowners."</p>
                                        <button class="text-[#004289] font-bold hover:underline" on:click=move |_| set_show_form.set(true)>"Create your first listing."</button>
                                    </div>
                                }.into_any()
                            } else {
                                items.into_iter().map(|item| {
                                    view! {
                                        <div class="bg-white rounded-2xl shadow-sm border border-outline-variant/30 overflow-hidden group hover:shadow-md transition-shadow">
                                            <div class="aspect-video bg-surface-container-low relative">
                                                <div class="absolute top-3 right-3 bg-white/90 backdrop-blur-md px-3 py-1 rounded-full flex items-center gap-1 shadow-sm">
                                                    <span class="text-[10px] font-bold text-on-surface uppercase tracking-wider">{item.status.clone()}</span>
                                                </div>
                                            </div>
                                            <div class="p-6">
                                                <div class="text-[10px] font-bold text-tertiary uppercase tracking-wider mb-2">{item.listing_type.clone()}</div>
                                                <h3 class="font-headline font-bold text-lg text-on-surface mb-2 truncate">{item.title.clone()}</h3>
                                                <div class="flex items-center justify-between mt-6 pt-4 border-t border-outline-variant/30">
                                                    <a href=format!("/{}", item.id) class="text-sm font-bold text-[#004289] hover:text-[#00336b] flex items-center gap-1" target="_blank">
                                                        "View Link" <span class="material-symbols-outlined text-[14px]">"open_in_new"</span>
                                                    </a>
                                                    <button class="text-sm font-bold text-on-surface-variant hover:text-on-surface flex items-center gap-1">
                                                        <span class="material-symbols-outlined text-[14px]">"edit"</span> "Edit"
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    }
                                }).collect_view().into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
