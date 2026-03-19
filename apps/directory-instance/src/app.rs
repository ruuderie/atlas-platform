use leptos::prelude::*;
use leptos_router::components::Router;
use leptos_meta::{provide_meta_context, Title, Stylesheet, Meta};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_brand_primary")]
    pub brand_primary: String,
    #[serde(default = "default_bg_surface")]
    pub bg_surface: String,
    #[serde(default = "default_radius_ui")]
    pub radius_ui: String,
    #[serde(default = "default_font_heading")]
    pub font_heading: String,
}

fn default_brand_primary() -> String { "#2563eb".to_string() }
fn default_bg_surface() -> String { "#ffffff".to_string() }
fn default_radius_ui() -> String { "4px".to_string() }
fn default_font_heading() -> String { "Inter, system-ui, sans-serif".to_string() }

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            brand_primary: default_brand_primary(),
            bg_surface: default_bg_surface(),
            radius_ui: default_radius_ui(),
            font_heading: default_font_heading(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DirectoryConfig {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub description: String,
    pub theme_primary_color: Option<String>,
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListingModel {
    pub id: String,
    pub listing_type: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaginatedListings<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
    pub total_pages: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateLeadInput {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub message: String,
    pub listing_id: Option<String>,
    pub _bot_check: Option<String>,
}

// Server functions automatically handle the isomorphic divide.
// On the server, they execute directly. On the client, they make a POST request to the server.
#[server]
pub async fn fetch_directory_config_from_api(domain: String) -> Result<DirectoryConfig, ServerFnError> {
    if domain.contains("localhost") || domain.contains("127.0.0.1") {
        return Ok(DirectoryConfig {
            id: "mock-dir-ct-build".to_string(),
            name: "CT Build Pros".to_string(),
            domain: domain.clone(),
            description: "The premier directory for top-rated construction and renovation services across Connecticut.".to_string(),
            theme_primary_color: Some("25 95% 53%".to_string()), // A construction orange
            theme: ThemeConfig {
                brand_primary: "#f97316".to_string(),
                bg_surface: "#ffffff".to_string(),
                radius_ui: "6px".to_string(),
                font_heading: "Inter, sans-serif".to_string(),
            }
        });
    }

    let url = format!("http://127.0.0.1:8000/directories/lookup?domain={}", domain);
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await?;
    
    if res.status().is_success() {
        Ok(res.json::<DirectoryConfig>().await?)
    } else {
        Err(ServerFnError::ServerError(format!("Error: {}", res.status())))
    }
}

#[server]
pub async fn fetch_listing_by_slug_from_api(slug: String) -> Result<ListingModel, ServerFnError> {
    if slug == "ct-contractor-demo" {
        return Ok(ListingModel {
            id: "mock-ct-contractor-id-123".to_string(),
            listing_type: "landing_page".to_string(),
            title: "Apex CT Renovations & Handyman Services".to_string(),
            description: "
                <h3>Premium Construction Services in Connecticut!</h3>
                <p>We deliver top-quality craftsmanship for both residential homeowners and commercial businesses. From full-scale house renovations to precision handyman repairs, Apex has Connecticut covered.</p>
                <br/>
                <h4>Our Core Services</h4>
                <ul>
                    <li><strong>Residential Remodeling:</strong> Kitchens, Bathrooms, and full extensions.</li>
                    <li><strong>Commercial Build-Outs:</strong> Modern retail and office space layouts.</li>
                    <li><strong>Professional Handyman:</strong> Fast and reliable repairs for properties across the state.</li>
                </ul>
            ".to_string(),
            attributes: [
                ("hero_headline".to_string(), "Your Dream Connecticut Build, Realized Today.".to_string()),
                ("cta_text".to_string(), "Request a Free Estimate".to_string()),
                ("sq_ft".to_string(), "1200".to_string()),
                ("parking".to_string(), "2 spaces".to_string())
            ].into_iter().collect(),
        });
    }

    let url = format!("http://127.0.0.1:8000/api/listings/by-slug/{}", slug);
    let client = reqwest::Client::new();
    let res = client.get(&url).send().await?;
    
    if res.status().is_success() {
        Ok(res.json::<ListingModel>().await?)
    } else {
        Err(ServerFnError::ServerError("not-found".into()))
    }
}

#[server]
pub async fn submit_lead_to_api(payload: CreateLeadInput) -> Result<(), ServerFnError> {
    let url = "http://127.0.0.1:8000/api/leads";
    let client = reqwest::Client::new();
    let res = client.post(url).json(&payload).send().await?;
    
    if res.status().is_success() {
        Ok(())
    } else {
        Err(ServerFnError::ServerError("Submission failed".into()))
    }
}

#[component]
fn LeadForm(listing_id: String, cta_text: String) -> impl IntoView {
    let query = leptos_router::hooks::use_query_map();
    let utm_source = move || query.with(|q| q.get("utm_source").unwrap_or_default());
    let utm_medium = move || query.with(|q| q.get("utm_medium").unwrap_or_default());
    let utm_campaign = move || query.with(|q| q.get("utm_campaign").unwrap_or_default());

    let step = RwSignal::new(1u8);
    let name = RwSignal::new("".to_string());
    let email = RwSignal::new("".to_string());
    let phone = RwSignal::new("".to_string());
    let intent = RwSignal::new("".to_string());
    let bot_check = RwSignal::new("".to_string());
    
    let is_submitting = RwSignal::new(false);
    let success = RwSignal::new(false);
    let error = RwSignal::new("".to_string());

    let listing_id_sig = RwSignal::new(listing_id);
    let cta_text_sig = RwSignal::new(cta_text);

    let handle_next = move |_| {
        error.set("".to_string());
        if name.get().trim().is_empty() {
            error.set("Please enter your full name.".to_string());
            return;
        }
        let e = email.get();
        if !e.contains('@') || !e.contains('.') || e.len() < 5 {
            error.set("Please enter a valid email address.".to_string());
            return;
        }
        step.set(2);
    };

    let handle_submit = move |_| {
        if is_submitting.get() { return; }
        error.set("".to_string());
        
        if intent.get().trim().is_empty() {
            error.set("Please provide a brief intent or message.".to_string());
            return;
        }

        is_submitting.set(true);
        
        let mut final_message = intent.get();
        let source = utm_source();
        let medium = utm_medium();
        let campaign = utm_campaign();
        
        if !source.is_empty() || !medium.is_empty() || !campaign.is_empty() {
            final_message.push_str("\n\n--- Tracking Info ---");
            if !source.is_empty() { final_message.push_str(&format!("\nSource: {}", source)); }
            if !medium.is_empty() { final_message.push_str(&format!("\nMedium: {}", medium)); }
            if !campaign.is_empty() { final_message.push_str(&format!("\nCampaign: {}", campaign)); }
        }
        
        let p = phone.get();
        let phone_opt = if p.is_empty() { None } else { Some(p) };

        let payload = CreateLeadInput {
            name: name.get(),
            email: email.get(),
            phone: phone_opt,
            message: final_message,
            listing_id: Some(listing_id_sig.get()),
            _bot_check: Some(bot_check.get()),
        };
        
        leptos::task::spawn_local(async move {
            match submit_lead_to_api(payload).await {
                Ok(_) => { success.set(true); }
                Err(e) => { error.set(e.to_string()); }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <div class="bg-white">
            {move || if success.get() {
                view! {
                    <div class="text-center space-y-4 py-8 animate-fade-scale">
                        <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-emerald-100 text-emerald-600 mb-2">
                            <span class="material-symbols-outlined text-3xl" data-icon="check_circle">"check_circle"</span>
                        </div>
                        <h3 class="text-xl font-bold font-headline text-slate-900">"Request Sent"</h3>
                        <p class="text-slate-500 font-body text-sm">"The host will review your request and get back to you shortly."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="space-y-4">
                        {move || if !error.get().is_empty() {
                            view! { <p class="text-error text-sm bg-error-container text-on-error-container p-3 rounded-xl">{error.get()}</p> }.into_any()
                        } else { view!{ <span/> }.into_any() }}
                        
                        // Honeypot Field
                        <div class="hidden" aria-hidden="true">
                            <input type="text" name="_bot_check" prop:value=bot_check on:input=move |ev| bot_check.set(event_target_value(&ev)) tabindex="-1" />
                        </div>

                        {move || if step.get() == 1 {
                            view! {
                                <div class="border border-slate-300 rounded-xl overflow-hidden mb-6 animate-slide-up">
                                    <div class="flex border-b border-slate-300">
                                        <div class="flex-1 p-3 border-r border-slate-300 bg-slate-50">
                                            <label class="block text-[10px] font-bold text-slate-800 uppercase tracking-widest mb-1">"Full Name"</label>
                                            <input type="text" class="w-full bg-transparent outline-none font-body text-slate-900 text-sm font-semibold" placeholder="Jane Doe" prop:value=name on:input=move |ev| name.set(event_target_value(&ev)) />
                                        </div>
                                        <div class="flex-1 p-3 bg-slate-50">
                                            <label class="block text-[10px] font-bold text-slate-800 uppercase tracking-widest mb-1">"Email"</label>
                                            <input type="email" class="w-full bg-transparent outline-none font-body text-slate-900 text-sm font-semibold" placeholder="name@domain.com" prop:value=email on:input=move |ev| email.set(event_target_value(&ev)) />
                                        </div>
                                    </div>
                                </div>
                                <button class="w-full bg-primary text-white py-4 rounded-xl font-bold text-lg hover:opacity-90 transition-opacity mb-4" on:click=handle_next>
                                    "Continue"
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <div class="border border-slate-300 rounded-xl overflow-hidden mb-6 animate-slide-up">
                                    <div class="p-3 border-b border-slate-300 bg-slate-50">
                                        <label class="block text-[10px] font-bold text-slate-800 uppercase tracking-widest mb-1">"Phone Number (Opt)"</label>
                                        <input type="tel" class="w-full bg-transparent outline-none font-body text-slate-900 text-sm font-semibold" placeholder="+1 (555) 000-0000" prop:value=phone on:input=move |ev| phone.set(event_target_value(&ev)) />
                                    </div>
                                    <div class="p-3 bg-slate-50">
                                        <label class="block text-[10px] font-bold text-slate-800 uppercase tracking-widest mb-1">"Message"</label>
                                        <textarea rows="3" class="w-full bg-transparent outline-none font-body text-slate-900 text-sm font-semibold" placeholder="I'm interested in..." prop:value=intent on:input=move |ev| intent.set(event_target_value(&ev))></textarea>
                                    </div>
                                </div>
                                
                                <div class="flex gap-2">
                                    <button class="w-1/3 bg-slate-100 text-slate-700 py-4 rounded-xl font-bold hover:bg-slate-200 transition-opacity mb-4" on:click=move |_| { step.set(1); error.set("".to_string()); }>
                                        "Back"
                                    </button>
                                    <button class="w-2/3 bg-primary text-white py-4 rounded-xl font-bold text-lg hover:opacity-90 transition-opacity mb-4 disabled:opacity-50 flex justify-center items-center" on:click=handle_submit disabled=is_submitting>
                                        {move || if is_submitting.get() { "Sending...".to_string() } else { cta_text_sig.get() }}
                                    </button>
                                </div>
                            }.into_any()
                        }}
                    </div>
                }.into_any()
            }}
        </div>
    }
}

#[component]
fn LandingPage() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();
    let slug = move || params.with(|p| p.get("slug").unwrap_or_default());
    
    // Use standard Resource for SSR data fetching instead of LocalResource
    let listing_resource = Resource::new(
        move || slug(),
        |current_slug| async move {
            if current_slug.is_empty() { Err(ServerFnError::ServerError("No slug provided".to_string())) }
            else { fetch_listing_by_slug_from_api(current_slug).await }
        }
    );

    view! {
        <Suspense fallback=|| view! { <div class="min-h-screen flex items-center justify-center p-8"><div class="w-8 h-8 rounded-full border-4 border-primary border-t-transparent animate-spin"></div></div> }>
            {move || match listing_resource.get() {
                None => view! { <div/> }.into_any(),
                Some(Ok(listing)) => {
                    let hero_headline = listing.attributes.get("hero_headline")
                        .map(|s| s.clone())
                        .unwrap_or_else(|| "Discover Our Services".to_string());
                        
                    let whatsapp = listing.attributes.get("whatsapp_number")
                        .filter(|s| !s.is_empty())
                        .map(|s| s.clone());
                        
                    let cta_text = listing.attributes.get("cta_text")
                        .map(|s| s.clone())
                        .unwrap_or_else(|| "Get Started".to_string());
                        
                    let l_id = listing.id.clone();
                    let url_canonical = format!("https://{}/{}", get_host(), slug());
                        
                    let json_ld = format!(
                        r#"{{
                            "@context": "https://schema.org",
                            "@type": "LocalBusiness",
                            "name": "{}",
                            "description": "{}"
                        }}"#,
                        listing.title.replace("\"", "\\\""),
                        hero_headline.replace("\"", "\\\"")
                    );
                        
                    view! {
                        <crate::components::seo::Seo 
                            title=listing.title.clone()
                            description=hero_headline.to_string()
                            og_type="website".to_string()
                            script_json_ld=json_ld
                            canonical_url=url_canonical
                        />
                        
                        <crate::components::layout::MainLayout>
                            <div class="px-8 py-6 max-w-7xl mx-auto border-b border-slate-200 w-full mt-4">
                                <div class="flex items-center gap-2 text-sm text-slate-500 font-body mb-4">
                                    <a class="hover:text-primary transition-colors" href="/search">"Directory"</a>
                                    <span class="material-symbols-outlined text-[16px]">"chevron_right"</span>
                                    <span class="text-slate-900 font-bold">{listing.title.clone()}</span>
                                </div>
                                <div class="flex flex-col md:flex-row md:justify-between md:items-end gap-4">
                                    <div>
                                        <h1 class="text-4xl md:text-5xl font-bold font-headline text-on-primary-fixed block mb-2">{listing.title.clone()}</h1>
                                        <div class="flex items-center gap-4 text-slate-600 font-body text-sm mt-3">
                                            <span class="flex items-center gap-1 font-bold"><span class="material-symbols-outlined text-[16px]">"star"</span> "4.96"</span>
                                            <span class="underline">"128 reviews"</span>
                                            <span>"·"</span>
                                            <span class="flex items-center gap-1"><span class="material-symbols-outlined text-[16px]">"location_on"</span> {listing.listing_type.clone()}</span>
                                        </div>
                                    </div>
                                    <div class="flex gap-4">
                                        <button class="flex items-center gap-2 px-4 py-2 rounded-lg hover:bg-slate-100 font-bold text-slate-700 transition-colors"><span class="material-symbols-outlined text-[20px]">"ios_share"</span> "Share"</button>
                                        <button class="flex items-center gap-2 px-4 py-2 rounded-lg hover:bg-slate-100 font-bold text-slate-700 transition-colors"><span class="material-symbols-outlined text-[20px]">"favorite_border"</span> "Save"</button>
                                    </div>
                                </div>
                            </div>
                            
                            <div class="px-8 max-w-7xl mx-auto py-8 w-full">
                                <div class="grid grid-cols-4 grid-rows-2 gap-4 h-[400px] md:h-[600px] rounded-3xl overflow-hidden">
                                    <div class="col-span-4 md:col-span-2 row-span-2 relative group cursor-pointer bg-slate-100">
                                        <img class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" src="https://images.unsplash.com/photo-1600596542815-ffad4c1539a9?ixlib=rb-4.0.3&auto=format&fit=crop&w=1200&q=80" alt="Main View" />
                                    </div>
                                    <div class="hidden md:block col-span-1 row-span-1 relative group cursor-pointer bg-slate-100">
                                        <img class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" src="https://images.unsplash.com/photo-1600607687931-cebf0305ab59?ixlib=rb-4.0.3&auto=format&fit=crop&w=600&q=80" alt="Detail 1" />
                                    </div>
                                    <div class="hidden md:block col-span-1 row-span-1 relative group cursor-pointer bg-slate-100">
                                        <img class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" src="https://images.unsplash.com/photo-1600585154340-be6161a56a0c?ixlib=rb-4.0.3&auto=format&fit=crop&w=600&q=80" alt="Detail 2" />
                                    </div>
                                    <div class="hidden md:block col-span-2 row-span-1 relative group cursor-pointer bg-slate-100">
                                        <img class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" src="https://images.unsplash.com/photo-1600566753190-17f0baa2a6c3?ixlib=rb-4.0.3&auto=format&fit=crop&w=1200&q=80" alt="Detail 3" />
                                        <button class="absolute bottom-6 right-6 z-20 bg-white border border-slate-200 px-4 py-2 rounded-lg font-bold text-sm shadow-sm flex items-center gap-2 hover:bg-slate-50 transition-colors">
                                            <span class="material-symbols-outlined text-[20px]">"grid_view"</span>
                                            "Show all photos"
                                        </button>
                                    </div>
                                </div>
                            </div>
                            
                            <div class="px-8 max-w-7xl mx-auto py-12 grid grid-cols-1 lg:grid-cols-12 gap-16 w-full">
                                <div class="lg:col-span-8">
                                    <h2 class="text-3xl font-bold font-headline mb-6 text-on-primary-fixed">"Architectural Philosophy"</h2>
                                    <div class="prose prose-slate max-w-none font-body text-slate-600 space-y-6 leading-relaxed text-lg">
                                        <div inner_html=listing.description.clone()></div>
                                    </div>

                                    <hr class="my-12 border-slate-200" />

                                    <h3 class="text-2xl font-bold font-headline mb-8 text-on-primary-fixed">"Curated Amenities"</h3>
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-y-6 gap-x-12">
                                        {
                                            let ignored_keys = ["hero_headline", "whatsapp_number", "cta_text"];
                                            listing.attributes.iter()
                                                .filter(|(k, _)| !ignored_keys.contains(&k.as_str()))
                                                .map(|(k, v)| {
                                                    let display_key = k.replace("_", " ");
                                                    view! {
                                                        <div class="flex items-center gap-4 text-slate-700 font-body">
                                                            <div class="w-10 h-10 rounded-full bg-slate-100 flex items-center justify-center text-slate-600">
                                                                <shared_ui::components::attribute_icon::AttributeIcon name=k.clone() class="w-5 h-5".to_string() />
                                                            </div>
                                                            <div>
                                                                <span class="font-bold mr-2 capitalize">{display_key}</span>
                                                                <span class="text-slate-500">{v.clone()}</span>
                                                            </div>
                                                        </div>
                                                    }
                                                }).collect_view()
                                        }
                                        {
                                            let ignored_keys = ["hero_headline", "whatsapp_number", "cta_text"];
                                            if listing.attributes.iter().filter(|(k, _)| !ignored_keys.contains(&k.as_str())).count() == 0 {
                                                view! {
                                                    <div class="col-span-full text-slate-500 font-medium">
                                                        "No additional amenities listed."
                                                    </div>
                                                }.into_any()
                                            } else { view!{<div/>}.into_any() }
                                        }
                                    </div>

                                    <hr class="my-12 border-slate-200" />

                                    <h3 class="text-2xl font-bold font-headline mb-8 text-on-primary-fixed">"Guest Experiences"</h3>
                                    <div class="flex items-center gap-4 mb-8">
                                        <div class="text-5xl font-headline font-bold text-on-primary-fixed">"4.96"</div>
                                        <div>
                                            <div class="flex gap-1 text-primary">
                                                <span class="material-symbols-outlined">"star"</span><span class="material-symbols-outlined">"star"</span><span class="material-symbols-outlined">"star"</span><span class="material-symbols-outlined">"star"</span><span class="material-symbols-outlined">"star"</span>
                                            </div>
                                            <div class="font-bold text-slate-600 mt-1">"Based on 128 verified stays"</div>
                                        </div>
                                    </div>
                                    <div class="grid grid-cols-1 md:grid-cols-2 gap-8">
                                        <div class="bg-slate-50 p-6 rounded-2xl border border-slate-100">
                                            <div class="flex items-center gap-4 mb-4">
                                                <div class="w-10 h-10 rounded-full bg-slate-200 overflow-hidden"><img src="https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" alt="Guest" class="w-full h-full object-cover"/></div>
                                                <div>
                                                    <div class="font-bold text-slate-900">"Michael Chen"</div>
                                                    <div class="text-xs text-slate-500">"Stayed October 2025"</div>
                                                </div>
                                            </div>
                                            <p class="text-slate-600 text-sm leading-relaxed">"The interplay of light throughout the day was remarkable. You can tell every detail was considered by the architect. A truly restorative environment."</p>
                                        </div>
                                        <div class="bg-slate-50 p-6 rounded-2xl border border-slate-100">
                                            <div class="flex items-center gap-4 mb-4">
                                                <div class="w-10 h-10 rounded-full bg-slate-200 overflow-hidden"><img src="https://images.unsplash.com/photo-1438761681033-6461ffad8d80?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" alt="Guest" class="w-full h-full object-cover"/></div>
                                                <div>
                                                    <div class="font-bold text-slate-900">"Elena Rodriguez"</div>
                                                    <div class="text-xs text-slate-500">"Stayed September 2025"</div>
                                                </div>
                                            </div>
                                            <p class="text-slate-600 text-sm leading-relaxed">"Impeccable curation. The property seamlessly integrates with the surrounding landscape while offering world-class comfort."</p>
                                        </div>
                                    </div>
                                </div>

                                <div class="lg:col-span-4 flex flex-col gap-6 relative pb-12">
                                    <div class="sticky top-28 bg-white border border-slate-200 rounded-3xl p-8 shadow-sm">
                                        <div class="flex items-end justify-between mb-6">
                                            <div class="text-3xl font-bold text-on-primary-fixed">"$450 " <span class="text-base font-normal text-slate-500">"/ night"</span></div>
                                            <div class="flex items-center gap-1 font-bold text-sm"><span class="material-symbols-outlined text-sm">"star"</span> "4.96"</div>
                                        </div>
                                        
                                        <LeadForm listing_id=l_id.clone() cta_text=cta_text.to_string() />
                                        
                                        <p class="text-center text-sm text-slate-500 font-body">"You won't be charged yet"</p>
                                    </div>
                                    
                                    <div class="sticky top-[450px] bg-slate-50 border border-slate-200 rounded-3xl p-8">
                                        <div class="flex items-center gap-4 mb-6">
                                            <div class="w-16 h-16 rounded-full overflow-hidden">
                                                <img src="https://images.unsplash.com/photo-1560250097-0b93528c311a?auto=format&fit=crop" class="w-full h-full object-cover" alt="Host"/>
                                            </div>
                                            <div>
                                                <div class="font-bold text-lg text-on-primary-fixed">"Hosted by Professional"</div>
                                                <div class="text-sm text-slate-500">"Joined May 2021"</div>
                                            </div>
                                        </div>
                                        <div class="flex items-center gap-2 text-slate-700 mb-2 font-body text-sm">
                                            <span class="material-symbols-outlined text-lg">"verified"</span> "Identity verified"
                                        </div>
                                        <div class="flex items-center gap-2 text-slate-700 mb-6 font-body text-sm">
                                            <span class="material-symbols-outlined text-lg">"workspace_premium"</span> "Superhost"
                                        </div>
                                        <button class="w-full border-2 border-slate-900 text-slate-900 py-3 rounded-xl font-bold hover:bg-slate-900 hover:text-white transition-colors">
                                            "Message Host"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </crate::components::layout::MainLayout>
                    }.into_any()
                },
                Some(Err(_)) => view! {
                    <crate::components::layout::MainLayout>
                        <Title text="Page Not Found" />
                        <div class="min-h-[60vh] flex flex-col items-center justify-center text-center p-8">
                            <h2 class="text-3xl font-bold text-slate-500 mb-4 font-headline">"Page Not Found"</h2>
                            <p class="font-body text-slate-600">"The page you are looking for does not exist."</p>
                        </div>
                    </crate::components::layout::MainLayout>
                }.into_any()
            }}
        </Suspense>
    }
}

pub fn get_host() -> String {
    #[cfg(feature = "ssr")]
    {
        use axum::http::request::Parts;
        if let Some(req_parts) = use_context::<Parts>() {
            req_parts.headers.get("host")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("localhost").to_string()
        } else {
            "localhost".to_string()
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        window().location().hostname().unwrap_or_else(|_| "localhost".to_string())
    }
}

#[component]
fn Home() -> impl IntoView {
    let config = use_context::<DirectoryConfig>().expect("DirectoryConfig must be provided");

    view! {
        <crate::components::layout::MainLayout>
            // Hero Section
            <section class="hero-gradient text-white py-32 px-8 relative overflow-hidden">
                <div class="absolute inset-0 bg-[url('https://images.unsplash.com/photo-1600596542815-ffad4c1539a9?ixlib=rb-4.0.3&auto=format&fit=crop&w=2000&q=80')] bg-cover bg-center mix-blend-overlay opacity-20"></div>
                <div class="max-w-4xl mx-auto text-center relative z-10">
                    <h1 class="text-5xl md:text-7xl font-extrabold mb-8 font-headline leading-tight tracking-tight mt-12">
                        "Discover the World's Most " <span class="text-transparent bg-clip-text bg-gradient-to-r from-blue-200 to-white">"Exceptional"</span> " Spaces"
                    </h1>
                    <p class="text-xl md:text-2xl mb-12 font-body text-blue-100 font-light max-w-2xl mx-auto leading-relaxed">
                        {config.description.clone()}
                    </p>
                    
                    <form action="/search" method="GET" class="bg-white/10 backdrop-blur-md p-4 rounded-2xl flex flex-col md:flex-row gap-4 max-w-3xl mx-auto border border-white/20 shadow-2xl">
                        <div class="flex-1 bg-white/10 border border-white/10 rounded-xl px-4 py-3 flex items-center gap-3 focus-within:bg-white/20 transition-colors">
                            <span class="material-symbols-outlined text-blue-200" data-icon="location_on">"location_on"</span>
                            <div class="flex-1 text-left">
                                <label class="block text-[10px] font-bold text-blue-200 uppercase tracking-wider mb-0.5">"Destination"</label>
                                <input name="location" type="text" placeholder="Where to?" class="bg-transparent w-full text-white placeholder-white/60 focus:outline-none font-bold" />
                            </div>
                        </div>
                        <div class="flex-1 bg-white/10 border border-white/10 rounded-xl px-4 py-3 flex items-center gap-3 focus-within:bg-white/20 transition-colors hidden sm:flex">
                            <span class="material-symbols-outlined text-blue-200" data-icon="service">"home_repair_service"</span>
                            <div class="flex-1 text-left">
                                <label class="block text-[10px] font-bold text-blue-200 uppercase tracking-wider mb-0.5">"Service"</label>
                                <input name="q" type="text" placeholder="What are you looking for?" class="bg-transparent w-full text-white placeholder-white/60 focus:outline-none font-bold" />
                            </div>
                        </div>
                        <button type="submit" class="bg-white text-primary px-8 py-4 rounded-xl font-bold hover:bg-blue-50 transition-colors flex items-center justify-center gap-2 whitespace-nowrap shadow-lg">
                            <span class="material-symbols-outlined" data-icon="search">"search"</span>
                            "Search"
                        </button>
                    </form>
                    
                    <div class="mt-8 flex items-center justify-center gap-6 text-sm font-medium text-blue-200">
                        <span class="flex items-center gap-2">
                            <span class="material-symbols-outlined text-[18px]" data-icon="check_circle">"check_circle"</span>
                            "Verified Properties"
                        </span>
                        <span class="flex items-center gap-2">
                            <span class="material-symbols-outlined text-[18px]" data-icon="workspace_premium">"workspace_premium"</span>
                            "Curated Selection"
                        </span>
                    </div>
                </div>
            </section>

            // Trust Signals / USP Bento
            <section class="py-24 px-8 max-w-7xl mx-auto">
                <div class="text-center mb-16">
                    <h2 class="text-3xl font-bold text-on-primary-fixed font-headline mb-4">"The " {config.name.clone()} " Standard"</h2>
                    <p class="text-slate-500 font-body max-w-2xl mx-auto">"Every property in our directory undergoes a rigorous 50-point architectural inspection."</p>
                </div>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                    <div class="bg-slate-50 rounded-2xl p-8 border border-slate-100 hover:border-slate-300 transition-colors group">
                        <div class="w-14 h-14 bg-white rounded-xl shadow-sm flex items-center justify-center mb-6 text-primary group-hover:scale-110 transition-transform">
                            <span class="material-symbols-outlined text-3xl" data-icon="architecture">"architecture"</span>
                        </div>
                        <h3 class="text-xl font-bold text-slate-900 mb-3 font-headline">"Design Pedigree"</h3>
                        <p class="text-slate-600 font-body leading-relaxed text-sm">"Only properties authored by recognized architectural studios or exhibiting exceptional vernacular craftsmanship are admitted."</p>
                    </div>
                    <div class="bg-slate-50 rounded-2xl p-8 border border-slate-100 hover:border-slate-300 transition-colors group">
                        <div class="w-14 h-14 bg-white rounded-xl shadow-sm flex items-center justify-center mb-6 text-primary group-hover:scale-110 transition-transform">
                            <span class="material-symbols-outlined text-3xl" data-icon="verified_user">"verified_user"</span>
                        </div>
                        <h3 class="text-xl font-bold text-slate-900 mb-3 font-headline">"Verified Quality"</h3>
                        <p class="text-slate-600 font-body leading-relaxed text-sm">"Physical inspections ensure photography accurately represents the spatial reality and material condition."</p>
                    </div>
                    <div class="bg-slate-50 rounded-2xl p-8 border border-slate-100 hover:border-slate-300 transition-colors group">
                        <div class="w-14 h-14 bg-white rounded-xl shadow-sm flex items-center justify-center mb-6 text-primary group-hover:scale-110 transition-transform">
                            <span class="material-symbols-outlined text-3xl" data-icon="concierge">"concierge"</span>
                        </div>
                        <h3 class="text-xl font-bold text-slate-900 mb-3 font-headline">"Curated Context"</h3>
                        <p class="text-slate-600 font-body leading-relaxed text-sm">"Detailed guides to the surrounding landscape, local design history, and architectural significance."</p>
                    </div>
                </div>
            </section>

            // Categories / Curator's Selection
            <section class="py-24 px-8 bg-surface-container-low">
                <div class="max-w-7xl mx-auto">
                    <div class="flex justify-between items-end mb-12">
                        <div>
                            <h2 class="text-3xl font-bold text-slate-900 font-headline mb-4">"Curator's Selection"</h2>
                            <p class="text-slate-600 font-body">"Recently added standout properties."</p>
                        </div>
                        <a href="/search" class="hidden md:flex items-center gap-2 text-primary font-bold hover:gap-3 transition-all">
                            "View Directory"
                            <span class="material-symbols-outlined text-sm" data-icon="arrow_forward">"arrow_forward"</span>
                        </a>
                    </div>
                    
                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                        {vec![
                            ("Brutalist Retreat", "Tulum, Mexico", "https://images.unsplash.com/photo-1512917774080-9991f1c4c750?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80", "ct-contractor-demo"),
                            ("Mid-Century Modernist", "Palm Springs, CA", "https://images.unsplash.com/photo-1558036117-15d82a90b9b1?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80", "elite-hvac"),
                            ("Alpine Minimalist", "Swiss Alps", "https://images.unsplash.com/photo-1449844908441-8829872d2607?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80", "green-lawn")
                        ].into_iter().map(|(title, loc, img, slug)| view! {
                            <a href=format!("/{}", slug) class="group block bg-white rounded-2xl overflow-hidden shadow-sm hover:shadow-xl transition-all duration-500 border border-slate-100">
                                <div class="relative h-64 overflow-hidden">
                                    <div class="absolute inset-0 bg-black/20 group-hover:bg-transparent transition-colors z-10"></div>
                                    <img src=img class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-700" alt=title />
                                    <div class="absolute top-4 right-4 z-20 bg-white/90 backdrop-blur-sm px-3 py-1 rounded-full text-xs font-bold text-slate-800">
                                        "Available"
                                    </div>
                                </div>
                                <div class="p-6">
                                    <h3 class="text-xl font-bold text-slate-900 mb-2 font-headline group-hover:text-primary transition-colors">{title}</h3>
                                    <div class="flex items-center gap-2 text-slate-500 text-sm font-body mb-4">
                                        <span class="material-symbols-outlined text-sm" data-icon="location_on">"location_on"</span>
                                        {loc}
                                    </div>
                                    <div class="flex justify-between items-center pt-4 border-t border-slate-100">
                                        <span class="font-bold text-slate-900">"From $450" <span class="text-sm text-slate-500 font-normal">"/ night"</span></span>
                                        <span class="text-primary font-bold text-sm">"View Details"</span>
                                    </div>
                                </div>
                            </a>
                        }).collect_view()}
                    </div>
                </div>
            </section>

            // Testimonial / Social Proof
            <section class="py-24 px-8 max-w-7xl mx-auto border-t border-slate-200">
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
                    <div>
                        <div class="mb-8">
                            <span class="material-symbols-outlined text-4xl text-primary" data-icon="format_quote">"format_quote"</span>
                        </div>
                        <h2 class="text-3xl md:text-4xl font-bold text-slate-900 font-headline leading-tight mb-8">
                            "Finding properties that meet our editorial standards used to require exhaustive research. " {config.name.clone()} " has consolidated the world's best residential architecture into one reliable platform."
                        </h2>
                        <div class="flex items-center gap-4">
                            <div class="w-12 h-12 rounded-full overflow-hidden">
                                <img src="https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-4.0.3&auto=format&fit=crop&w=200&q=80" class="w-full h-full object-cover" alt="Reviewer" />
                            </div>
                            <div>
                                <div class="font-bold text-slate-900">"Sarah Jenkins"</div>
                                <div class="text-sm text-slate-500 font-body">"Editor in Chief, modernismo"</div>
                            </div>
                        </div>
                    </div>
                    <div class="grid grid-cols-2 gap-4 h-[500px]">
                        <div class="w-full h-full pt-12">
                            <img src="https://images.unsplash.com/photo-1513694203232-719a280e022f?ixlib=rb-4.0.3&auto=format&fit=crop&w=600&q=80" class="w-full h-full object-cover rounded-2xl shadow-lg" alt="Testimonial visual 1" />
                        </div>
                        <div class="w-full h-full pb-12">
                            <img src="https://images.unsplash.com/photo-1600585154340-be6161a56a0c?ixlib=rb-4.0.3&auto=format&fit=crop&w=600&q=80" class="w-full h-full object-cover rounded-2xl shadow-lg" alt="Testimonial visual 2" />
                        </div>
                    </div>
                </div>
            </section>

            // Final CTA
            <section class="py-24 px-8 hero-gradient text-white text-center">
                <div class="max-w-3xl mx-auto">
                    <h2 class="text-4xl md:text-5xl font-bold font-headline mb-6 tracking-tight">"Own an Architectural Masterpiece?"</h2>
                    <p class="text-xl text-blue-100 font-body mb-10 max-w-2xl mx-auto leading-relaxed">
                        "Join our exclusive directory and connect with a curated audience of design-conscious travelers who appreciate the value of exceptional spaces."
                    </p>
                    <div class="flex flex-col sm:flex-row gap-4 justify-center">
                        <button class="bg-white text-primary px-8 py-4 rounded-xl font-bold hover:shadow-lg hover:scale-105 transition-all duration-300">
                            "Submit Property"
                        </button>
                        <button class="bg-transparent border-2 border-white text-white px-8 py-4 rounded-xl font-bold hover:bg-white/10 transition-colors">
                            "Learn About Criteria"
                        </button>
                    </div>
                </div>
            </section>
        </crate::components::layout::MainLayout>
    }
}

#[component]
fn InnerApp(config: DirectoryConfig) -> impl IntoView {
    // 1. Provide Context Globally
    provide_context(config.clone());
    
    // 2. Generate CSS Injection based on ThemeConfig
    let css_payload = format!(
        ":root {{
            --primary: {};
            --bg-surface: {};
            --radius-ui: {};
            --font-heading: {};
        }}",
        config.theme.brand_primary,
        config.theme.bg_surface,
        config.theme.radius_ui,
        config.theme.font_heading
    );

    view! {
        <Title text=config.name.clone() />
        <style>{css_payload}</style>
        
        <Router>
            <leptos_router::components::Routes fallback=|| view! { "Not Found" }>
                <leptos_router::components::Route path=leptos_router::path!("/") view=Home />
                <leptos_router::components::Route path=leptos_router::path!("/search") view=crate::pages::search::Search />
                <leptos_router::components::Route path=leptos_router::path!(":slug") view=LandingPage />
            </leptos_router::components::Routes>
        </Router>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    
    let host = get_host();
    let err_host = host.clone();
    
    let directory_resource = Resource::new(
        move || host.clone(),
        |h| async move { fetch_directory_config_from_api(h).await }
    );
    
    view! {
        <Stylesheet id="leptos" href="/pkg/directory-instance.css"/>
        
        <Suspense fallback=|| view! { <div class="min-h-screen flex items-center justify-center"><div class="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin"></div></div> }>
            {move || match directory_resource.get() {
                None => view! { <div/> }.into_any(),
                Some(Ok(config)) => view! { <InnerApp config=config /> }.into_any(),
                Some(Err(e)) => view! {
                    <Title text="Directory Offline" />
                    <div class="min-h-screen flex items-center justify-center p-4 bg-background">
                        <div class="text-center space-y-6 max-w-lg glass-panel border-destructive/30 p-12 rounded-3xl animate-in zoom-in-95 duration-500 shadow-2xl shadow-destructive/10">
                            <div class="w-24 h-24 bg-destructive/10 text-destructive rounded-full flex items-center justify-center mx-auto mb-6">
                                <svg class="w-12 h-12" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"></path></svg>
                            </div>
                            <h1 class="text-4xl font-extrabold text-foreground">"Connection Dropped"</h1>
                            <p class="text-xl text-muted-foreground font-light">"We couldn't connect a dedicated database record to [" <span class="text-foreground font-mono bg-white/5 px-2 py-1 rounded">{err_host.clone()}</span> "]."</p>
                            <p class="text-sm bg-black/40 p-4 rounded-lg font-mono text-destructive/80 mt-8 break-all shadow-inner">{e.to_string()}</p>
                        </div>
                    </div>
                }.into_any()
            }}
        </Suspense>
    }
}
