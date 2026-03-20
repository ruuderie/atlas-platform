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

// --- Sub-structs for DirectoryConfig page content ---

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CategoryItem {
    pub slug: String,
    pub label: String,
    pub subtitle: String,
    pub icon: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeaturedListingMock {
    pub slug: String,
    pub title: String,
    pub subtitle: String,
    pub image_url: String,
    pub badge_label: String,
    pub badge_icon: String,
    pub price_label: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProcessStep {
    pub number: String,
    pub title: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostPageContent {
    pub hero_headline: String,
    pub hero_subtitle: String,
    pub form_category_options: Vec<String>,
    pub trust_heading: String,
    pub trust_subtitle: String,
    pub testimonial_quote: String,
    pub testimonial_name: String,
    pub testimonial_title: String,
    pub cta_headline: String,
    pub cta_subtitle: String,
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
    // Page content fields — populated from backend/mock data
    #[serde(default)]
    pub hero_headline: String,
    #[serde(default)]
    pub hero_subtitle: String,
    #[serde(default)]
    pub search_placeholder_keyword: String,
    #[serde(default)]
    pub search_placeholder_location: String,
    #[serde(default)]
    pub categories: Vec<CategoryItem>,
    #[serde(default)]
    pub featured_listings: Vec<FeaturedListingMock>,
    #[serde(default)]
    pub process_steps: Vec<ProcessStep>,
    #[serde(default)]
    pub cta_headline: String,
    #[serde(default)]
    pub cta_subtext: String,
    #[serde(default)]
    pub host_page: Option<HostPageContent>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListingModel {
    pub id: String,
    pub listing_type: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub attributes: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub price: Option<f64>,
    #[serde(default)]
    pub price_type: Option<String>,
    #[serde(default)]
    pub is_featured: bool,
    #[serde(default)]
    pub has_landing_page: bool,
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
            theme_primary_color: Some("25 95% 53%".to_string()),
            theme: ThemeConfig {
                brand_primary: "#f97316".to_string(),
                bg_surface: "#ffffff".to_string(),
                radius_ui: "6px".to_string(),
                font_heading: "Inter, sans-serif".to_string(),
            },
            hero_headline: "Connecticut's Most Trusted Home Renovation Pros.".to_string(),
            hero_subtitle: "Find licensed contractors, handymen, and renovation specialists — vetted and reviewed by your neighbors.".to_string(),
            search_placeholder_keyword: "Kitchen remodel, plumber, handyman...".to_string(),
            search_placeholder_location: "Hartford, Stamford, New Haven...".to_string(),
            categories: vec![
                CategoryItem { slug: "kitchen-bath".to_string(), label: "Kitchen & Bath".to_string(), subtitle: "Remodels & Upgrades".to_string(), icon: "countertops".to_string() },
                CategoryItem { slug: "general-handyman".to_string(), label: "General Handyman".to_string(), subtitle: "Repairs & Odd Jobs".to_string(), icon: "handyman".to_string() },
                CategoryItem { slug: "roofing-siding".to_string(), label: "Roofing & Siding".to_string(), subtitle: "Exterior Specialists".to_string(), icon: "roofing".to_string() },
                CategoryItem { slug: "electrical".to_string(), label: "Electrical".to_string(), subtitle: "Licensed Electricians".to_string(), icon: "electrical_services".to_string() },
                CategoryItem { slug: "painting".to_string(), label: "Painting".to_string(), subtitle: "Professional Painter".to_string(), icon: "professional_painter".to_string() },
            ],
            featured_listings: vec![
                FeaturedListingMock {
                    slug: "ct-contractor-demo".to_string(),
                    title: "Apex CT Renovations".to_string(),
                    subtitle: "Full-Service Remodeling • Hartford, CT".to_string(),
                    image_url: "https://images.unsplash.com/photo-1556909114-f6e7ad7d3136?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80".to_string(),
                    badge_label: "Top Rated".to_string(),
                    badge_icon: "verified".to_string(),
                    price_label: Some("From $5,000".to_string()),
                    tags: vec!["Kitchen & Bath".to_string(), "Licensed".to_string()],
                },
                FeaturedListingMock {
                    slug: "search".to_string(),
                    title: "Precision Plumbing Co.".to_string(),
                    subtitle: "Emergency & Residential • Stamford, CT".to_string(),
                    image_url: "https://images.unsplash.com/photo-1581578731548-c64695cc6952?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80".to_string(),
                    badge_label: "Best of CT 2025".to_string(),
                    badge_icon: "star".to_string(),
                    price_label: None,
                    tags: vec!["Plumbing".to_string(), "24/7 Service".to_string()],
                },
                FeaturedListingMock {
                    slug: "search".to_string(),
                    title: "GreenScape Landscaping".to_string(),
                    subtitle: "Design & Maintenance • New Haven, CT".to_string(),
                    image_url: "https://images.unsplash.com/photo-1558618666-fcd25c85f82e?ixlib=rb-4.0.3&auto=format&fit=crop&w=800&q=80".to_string(),
                    badge_label: "New Listing".to_string(),
                    badge_icon: "new_releases".to_string(),
                    price_label: Some("From $1,200".to_string()),
                    tags: vec!["Landscaping".to_string(), "Eco-Friendly".to_string()],
                },
            ],
            process_steps: vec![
                ProcessStep {
                    number: "01".to_string(),
                    title: "Search & Compare".to_string(),
                    description: "Browse our directory of licensed and insured Connecticut contractors. Filter by service type, location, budget, and customer rating to find the perfect match.".to_string(),
                },
                ProcessStep {
                    number: "02".to_string(),
                    title: "Read Verified Reviews".to_string(),
                    description: "Every contractor is reviewed by real Connecticut homeowners. Check project photos, read detailed feedback, and verify licenses before making contact.".to_string(),
                },
                ProcessStep {
                    number: "03".to_string(),
                    title: "Get Your Free Estimate".to_string(),
                    description: "Connect directly with pros through our secure platform. Request quotes, schedule consultations, and start your renovation project with confidence.".to_string(),
                },
            ],
            cta_headline: "Ready to grow your renovation business?".to_string(),
            cta_subtext: "Join hundreds of Connecticut contractors already getting new customers through our directory.".to_string(),
            host_page: Some(HostPageContent {
                hero_headline: "Your Expertise.\nYour Business.\nOur Platform.".to_string(),
                hero_subtitle: "Join Connecticut's fastest-growing directory for renovation and handyman professionals. We connect skilled tradespeople with homeowners who need quality work done right.".to_string(),
                form_category_options: vec![
                    "Kitchen & Bath Remodeling".to_string(),
                    "General Handyman".to_string(),
                    "Roofing & Siding".to_string(),
                    "Electrical".to_string(),
                    "Plumbing".to_string(),
                    "Painting & Drywall".to_string(),
                    "Landscaping".to_string(),
                ],
                trust_heading: "Why List With Us?".to_string(),
                trust_subtitle: "We connect trusted renovation pros with Connecticut homeowners who are ready to start their projects.".to_string(),
                testimonial_quote: "CT Build Pros didn't just find me customers; they found me homeowners who actually valued quality craftsmanship. My kitchen remodel bookings tripled in the first quarter. It's a community that respects the trade.".to_string(),
                testimonial_name: "Marcus Rivera".to_string(),
                testimonial_title: "Owner, Rivera Custom Renovations (15+ years in CT)".to_string(),
                cta_headline: "Ready to grow your business?".to_string(),
                cta_subtitle: "List your services today. New contractor profiles are reviewed and approved within 48 hours.".to_string(),
            }),
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
            listing_type: "Full-Service Remodeling".to_string(),
            title: "Apex CT Renovations & Handyman Services".to_string(),
            description: "
                <h3>Premium Renovation Services in Connecticut</h3>
                <p>We deliver top-quality craftsmanship for residential homeowners across the state. From full-scale kitchen and bathroom remodels to precision handyman repairs, Apex has been Connecticut's trusted renovation partner for over 15 years.</p>
                <br/>
                <h4>Our Core Services</h4>
                <ul>
                    <li><strong>Kitchen Remodeling:</strong> Custom cabinetry, countertops, backsplashes, and complete layout redesigns.</li>
                    <li><strong>Bathroom Renovation:</strong> Walk-in showers, tile work, vanity upgrades, and accessibility modifications.</li>
                    <li><strong>General Handyman:</strong> Drywall repair, fixture installation, painting, and general maintenance.</li>
                    <li><strong>Basement Finishing:</strong> Full basement conversions including framing, electrical, and flooring.</li>
                </ul>
            ".to_string(),
            attributes: [
                ("hero_headline".to_string(), "Your Dream Connecticut Renovation, Done Right.".to_string()),
                ("hero_subtitle".to_string(), "From concept to completion, our master craftsmen deliver flawless results that redefine your living space.".to_string()),
                ("cta_headline".to_string(), "Get Your Free Project Estimate".to_string()),
                ("cta_text".to_string(), "Request a Free Estimate".to_string()),
                ("years_experience".to_string(), "15+".to_string()),
                ("projects_completed".to_string(), "1,200+".to_string()),
                ("service_area".to_string(), "All of CT".to_string()),
                ("license_number".to_string(), "CT-HIC #0648751".to_string()),
            ].into_iter().collect(),
            city: Some("Hartford".to_string()),
            state: Some("CT".to_string()),
            price: Some(5000.0),
            price_type: Some("starting_at".to_string()),
            is_featured: true,
            has_landing_page: true,
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
fn ListingDetail() -> impl IntoView {
    let params = leptos_router::hooks::use_params_map();
    let slug = move || params.with(|p| p.get("slug").unwrap_or_default());
    
    let listing_resource = Resource::new(
        move || slug(),
        |current_slug| async move {
            if current_slug.is_empty() { Err(ServerFnError::ServerError("No slug provided".to_string())) }
            else { fetch_listing_by_slug_from_api(current_slug).await }
        }
    );

    view! {
        <Suspense fallback=|| view! { <div class="min-h-screen flex items-center justify-center p-8"><div class="w-8 h-8 rounded-full border-4 border-[#004289] border-t-transparent animate-spin"></div></div> }>
            {move || match listing_resource.get() {
                None => view! { <div/> }.into_any(),
                Some(Ok(listing)) => {
                    let hero_headline = listing.attributes.get("hero_headline")
                        .map(|s| s.clone())
                        .unwrap_or_else(|| listing.title.clone());
                    let hero_subtitle = listing.attributes.get("hero_subtitle")
                        .map(|s| s.clone())
                        .unwrap_or_else(|| listing.description.clone());
                    let cta_text = listing.attributes.get("cta_headline")
                        .map(|s| s.clone())
                        .unwrap_or_else(|| "Request a Quote".to_string());
                    let years_exp = listing.attributes.get("years_experience")
                        .map(|s| s.clone()).unwrap_or_else(|| "10+".to_string());
                    let projects = listing.attributes.get("projects_completed")
                        .map(|s| s.clone()).unwrap_or_else(|| "500+".to_string());
                    let service_area = listing.attributes.get("service_area")
                        .map(|s| s.clone()).unwrap_or_else(|| "Local".to_string());
                    let license = listing.attributes.get("license_number")
                        .map(|s| s.clone()).unwrap_or_else(|| "Licensed & Insured".to_string());
                    let location_str = format!("{}, {}",
                        listing.city.clone().unwrap_or_else(|| "Connecticut".to_string()),
                        listing.state.clone().unwrap_or_else(|| "CT".to_string()));
                    let price_display = listing.price.map(|p| format!("${}", p as u64)).unwrap_or_else(|| "Contact for pricing".to_string());
                    let l_id = listing.id.clone();
                    let url_canonical = format!("https://{}/{}", get_host(), slug());
                    let json_ld = format!(
                        r#"{{"@context": "https://schema.org", "@type": "LocalBusiness", "name": "{}", "description": "{}"}}"#,
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
                            // Image Gallery
                            <div class="px-8 max-w-7xl mx-auto pt-6 w-full">
                                <div class="grid grid-cols-12 gap-2 h-[500px] md:h-[600px] rounded-sm overflow-hidden">
                                    <div class="col-span-12 md:col-span-8 row-span-2 relative group cursor-pointer bg-surface-container-low">
                                        <img class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" src="https://images.unsplash.com/photo-1556909114-f6e7ad7d3136?ixlib=rb-4.0.3&auto=format&fit=crop&w=1200&q=80" alt="Kitchen renovation project" />
                                        <div class="absolute top-6 left-6 flex gap-2 z-20">
                                            {if listing.is_featured {
                                                view! { <span class="bg-tertiary text-white text-[10px] font-bold uppercase tracking-wider px-3 py-1.5 rounded-sm">"Top Rated"</span> }.into_any()
                                            } else {
                                                view! { <span/> }.into_any()
                                            }}
                                            <span class="bg-surface-container-lowest text-on-surface text-[10px] font-bold uppercase tracking-wider px-3 py-1.5 rounded-sm flex items-center gap-1">
                                                <span class="material-symbols-outlined text-[14px]">"handyman"</span>
                                                {listing.listing_type.clone()}
                                            </span>
                                        </div>
                                    </div>
                                    <div class="hidden md:block col-span-4 relative group cursor-pointer bg-surface-container-low">
                                        <img class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" src="https://images.unsplash.com/photo-1584622650111-993a426fbf0a?ixlib=rb-4.0.3&auto=format&fit=crop&w=600&q=80" alt="Bathroom renovation" />
                                    </div>
                                    <div class="hidden md:block col-span-4 relative group cursor-pointer bg-surface-container-low">
                                        <img class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" src="https://images.unsplash.com/photo-1581578731548-c64695cc6952?ixlib=rb-4.0.3&auto=format&fit=crop&w=600&q=80" alt="Handyman at work" />
                                        <div class="absolute inset-0 bg-black/30 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity z-20">
                                            <span class="text-white font-bold text-lg">"+12 Photos"</span>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            // Service Header
                            <div class="px-8 max-w-7xl mx-auto pt-10 pb-8 w-full">
                                <div class="flex items-center gap-2 mb-4">
                                    <span class="material-symbols-outlined text-tertiary text-[18px]" style="font-variation-settings: 'FILL' 1;">"verified"</span>
                                    <span class="text-tertiary font-bold tracking-widest text-xs uppercase">"Licensed & Verified"</span>
                                </div>
                                <div class="grid grid-cols-1 lg:grid-cols-3 gap-8 items-start">
                                    <div class="lg:col-span-2">
                                        <h1 class="font-headline text-4xl md:text-5xl font-extrabold tracking-tight text-on-surface mb-4">{hero_headline.clone()}</h1>
                                        <p class="text-on-surface-variant text-lg mb-6 leading-relaxed bg-surface-container-lowest p-6 rounded-xl border border-outline-variant/30">{hero_subtitle}</p>
                                        <div class="flex items-center gap-4 text-on-surface-variant text-sm">
                                            <span class="flex items-center gap-1">
                                                <span class="material-symbols-outlined text-[16px]">"location_on"</span>
                                                {location_str}
                                            </span>
                                            <span>"·"</span>
                                            <span class="flex items-center gap-1">
                                                <span class="material-symbols-outlined text-[16px]" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                                <span class="font-bold text-on-surface">"4.9"</span>
                                                " (87 Reviews)"
                                            </span>
                                        </div>
                                        // Service Stats
                                        <div class="flex gap-10 mt-8 pt-6 border-t border-outline-variant/30">
                                            <div>
                                                <span class="text-xs text-on-surface-variant uppercase tracking-widest font-bold block mb-1">"Experience"</span>
                                                <span class="font-headline font-bold text-lg">{years_exp} " Years"</span>
                                            </div>
                                            <div>
                                                <span class="text-xs text-on-surface-variant uppercase tracking-widest font-bold block mb-1">"Projects"</span>
                                                <span class="font-headline font-bold text-lg">{projects}</span>
                                            </div>
                                            <div>
                                                <span class="text-xs text-on-surface-variant uppercase tracking-widest font-bold block mb-1">"Specialty"</span>
                                                <span class="font-headline font-bold text-lg">{listing.listing_type.clone()}</span>
                                            </div>
                                            <div>
                                                <span class="text-xs text-on-surface-variant uppercase tracking-widest font-bold block mb-1">"License"</span>
                                                <span class="font-headline font-bold text-lg">{license}</span>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            // Content + Sidebar
                            <div class="px-8 max-w-7xl mx-auto py-8 grid grid-cols-1 lg:grid-cols-3 gap-16 w-full">
                                // Main content
                                <div class="lg:col-span-2">
                                    <h2 class="font-headline text-3xl font-bold text-on-surface mb-6">"About This Service"</h2>
                                    <div class="text-on-surface-variant leading-relaxed space-y-6 text-lg">
                                        <div inner_html=listing.description.clone()></div>
                                    </div>
                                    <button class="mt-6 text-[#004289] font-bold flex items-center gap-1 hover:underline underline-offset-4">
                                        "Read full service details"
                                        <span class="material-symbols-outlined text-[18px]">"expand_more"</span>
                                    </button>

                                    // Service Features
                                    <div class="mt-16">
                                        <h3 class="font-headline text-2xl font-bold text-on-surface mb-8">"What's Included"</h3>
                                        <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
                                            {vec![
                                                ("verified_user", "Licensed & Insured"),
                                                ("schedule", "On-Time Guarantee"),
                                                ("handyman", "Skilled Tradespeople"),
                                                ("cleaning_services", "Job-Site Cleanup"),
                                                ("receipt_long", "Detailed Estimates"),
                                                ("support_agent", "Warranty Included"),
                                            ].into_iter().map(|(icon, label)| view! {
                                                <div class="bg-surface-container-low p-4 rounded-lg flex items-center gap-3">
                                                    <span class="material-symbols-outlined text-[#004289] text-xl">{icon}</span>
                                                    <span class="font-medium text-on-surface text-sm">{label}</span>
                                                </div>
                                            }).collect_view()}
                                        </div>
                                    </div>

                                    // Customer Reviews
                                    <div class="mt-16">
                                        <div class="flex items-center justify-between mb-8">
                                            <h3 class="font-headline text-2xl font-bold text-on-surface">"Customer Reviews"</h3>
                                            <div class="text-right">
                                                <div class="text-3xl font-extrabold text-on-surface">"4.9"</div>
                                                <div class="text-xs text-on-surface-variant uppercase tracking-widest font-bold">"Average Rating"</div>
                                            </div>
                                        </div>
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                                            <div class="bg-surface-container-lowest p-6 rounded-xl shadow-sm">
                                                <div class="flex items-center gap-3 mb-4">
                                                    <div class="w-10 h-10 rounded-full bg-surface-container overflow-hidden">
                                                        <img src="https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" alt="Mike D." class="w-full h-full object-cover"/>
                                                    </div>
                                                    <div>
                                                        <div class="font-bold text-on-surface text-sm">"Mike D."</div>
                                                        <div class="text-xs text-on-surface-variant">"Homeowner, West Hartford"</div>
                                                    </div>
                                                </div>
                                                <p class="text-on-surface-variant text-sm leading-relaxed italic">"They completely transformed our kitchen. The attention to detail on the cabinetry and tile work was outstanding. Finished on time and on budget — couldn't ask for more."</p>
                                            </div>
                                            <div class="bg-surface-container-lowest p-6 rounded-xl shadow-sm">
                                                <div class="flex items-center gap-3 mb-4">
                                                    <div class="w-10 h-10 rounded-full bg-surface-container overflow-hidden">
                                                        <img src="https://images.unsplash.com/photo-1438761681033-6461ffad8d80?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" alt="Sarah L." class="w-full h-full object-cover"/>
                                                    </div>
                                                    <div>
                                                        <div class="font-bold text-on-surface text-sm">"Sarah L."</div>
                                                        <div class="text-xs text-on-surface-variant">"Homeowner, Stamford"</div>
                                                    </div>
                                                </div>
                                                <p class="text-on-surface-variant text-sm leading-relaxed italic">"Best handyman service we've ever used. From fixing a leaky faucet to installing new light fixtures, they handle everything professionally. Highly recommend."</p>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // Sticky Sidebar
                                <div class="lg:col-span-1">
                                    <div class="sticky top-28 space-y-6">
                                        // Estimate Card
                                        <div class="bg-surface-container-lowest p-8 rounded-xl shadow-premium">
                                            <div class="flex items-end justify-between mb-6">
                                                <div>
                                                    <span class="text-3xl font-extrabold text-on-surface">{price_display}</span>
                                                    <span class="text-on-surface-variant">" starting"</span>
                                                </div>
                                                <a href="#" class="text-[#004289] font-bold text-sm hover:underline">"See Pricing"</a>
                                            </div>

                                            // Project Info
                                            <div class="grid grid-cols-2 mb-4">
                                                <div class="border border-outline-variant/50 p-3 rounded-tl-lg">
                                                    <span class="text-[10px] uppercase tracking-widest font-bold text-on-surface-variant block">"Service Area"</span>
                                                    <span class="text-sm font-medium text-on-surface">{service_area}</span>
                                                </div>
                                                <div class="border border-outline-variant/50 border-l-0 p-3 rounded-tr-lg">
                                                    <span class="text-[10px] uppercase tracking-widest font-bold text-on-surface-variant block">"Availability"</span>
                                                    <span class="text-sm font-medium text-on-surface">"Within 48 hrs"</span>
                                                </div>
                                            </div>
                                            <div class="border border-outline-variant/50 p-3 rounded-b-lg mb-6">
                                                <span class="text-[10px] uppercase tracking-widest font-bold text-on-surface-variant block">"Project Type"</span>
                                                <div class="flex justify-between items-center">
                                                    <span class="text-sm font-medium text-on-surface">"Residential"</span>
                                                    <span class="material-symbols-outlined text-on-surface-variant text-[18px]">"expand_more"</span>
                                                </div>
                                            </div>

                                            <LeadForm listing_id=l_id.clone() cta_text=cta_text.to_string() />

                                            // Fee breakdown
                                            <div class="mt-6 space-y-3 text-sm">
                                                <div class="flex justify-between text-on-surface-variant">
                                                    <span>"Consultation"</span>
                                                    <span>"Free"</span>
                                                </div>
                                                <div class="flex justify-between text-on-surface-variant">
                                                    <span>"Estimate Visit"</span>
                                                    <span>"No charge"</span>
                                                </div>
                                                <div class="flex justify-between font-bold text-on-surface pt-3 border-t border-outline-variant/30">
                                                    <span>"Starting From"</span>
                                                    <span>{listing.price.map(|p| format!("${}", p as u64)).unwrap_or_else(|| "TBD".to_string())}</span>
                                                </div>
                                            </div>
                                            <p class="text-center text-xs text-on-surface-variant mt-4 flex items-center justify-center gap-1">
                                                <span class="material-symbols-outlined text-[14px]">"schedule"</span>
                                                "Response within 2 hours"
                                            </p>
                                        </div>

                                        // Service Specialist
                                        <div class="bg-surface-container-lowest p-6 rounded-xl shadow-sm">
                                            <div class="flex items-center gap-4">
                                                <div class="w-12 h-12 rounded-lg bg-[#004289] flex items-center justify-center text-white font-bold text-xl">"A"</div>
                                                <div>
                                                    <span class="text-xs text-on-surface-variant uppercase tracking-widest font-bold block">"Service Specialist"</span>
                                                    <div class="font-bold text-on-surface">"Alex Torres"</div>
                                                    <div class="text-xs text-on-surface-variant">"Renovation Advisor"</div>
                                                </div>
                                            </div>
                                        </div>
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
                            <h2 class="font-headline text-3xl font-bold text-on-surface-variant mb-4">"Page Not Found"</h2>
                            <p class="text-on-surface-variant">"The page you are looking for does not exist."</p>
                        </div>
                    </crate::components::layout::MainLayout>
                }.into_any()
            }}
        </Suspense>
    }
}


#[component]
fn HostLanding() -> impl IntoView {
    let config = use_context::<DirectoryConfig>().expect("DirectoryConfig must be provided");
    let hp = config.host_page.clone().unwrap_or_else(|| HostPageContent {
        hero_headline: "List Your Service".to_string(),
        hero_subtitle: "Join our growing directory.".to_string(),
        form_category_options: vec!["General".to_string()],
        trust_heading: "Why List With Us?".to_string(),
        trust_subtitle: "Grow your business.".to_string(),
        testimonial_quote: "Great platform!".to_string(),
        testimonial_name: "Contractor".to_string(),
        testimonial_title: "Owner".to_string(),
        cta_headline: "Ready to grow?".to_string(),
        cta_subtitle: "List today.".to_string(),
    });

    view! {
        <crate::components::layout::MainLayout>
            // Hero Section
            <section class="relative min-h-[700px] flex items-center bg-[#004289] px-8 overflow-hidden">
                <div class="absolute inset-0 opacity-15 pointer-events-none">
                    <img class="w-full h-full object-cover grayscale" src="https://images.unsplash.com/photo-1504307651254-35680f356dfd?ixlib=rb-4.0.3&auto=format&fit=crop&w=2000&q=80" alt="Construction site" />
                </div>
                <div class="relative max-w-7xl mx-auto w-full py-24 z-10 grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
                    <div>
                        <span class="bg-tertiary text-white text-[10px] font-bold uppercase tracking-widest px-4 py-2 rounded-sm inline-block mb-8">"Join Our Directory"</span>
                        <h1 class="font-headline text-white text-5xl md:text-6xl font-extrabold tracking-tighter leading-tight mb-8">
                            "Your Expertise." <br/> "Your Business." <br/> "Our Platform."
                        </h1>
                        <p class="text-on-primary-container text-lg leading-relaxed mb-10 max-w-lg">
                            {hp.hero_subtitle.clone()}
                        </p>
                        <div class="flex items-center gap-4">
                            <div class="flex -space-x-3">
                                <img class="w-10 h-10 rounded-full border-2 border-white object-cover" src="https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" alt="Contractor" />
                                <img class="w-10 h-10 rounded-full border-2 border-white object-cover" src="https://images.unsplash.com/photo-1494790108377-be9c29b29330?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" alt="Contractor" />
                                <img class="w-10 h-10 rounded-full border-2 border-white object-cover" src="https://images.unsplash.com/photo-1438761681033-6461ffad8d80?ixlib=rb-4.0.3&auto=format&fit=crop&w=100&q=80" alt="Contractor" />
                            </div>
                            <div class="text-white">
                                <span class="font-bold">"200+"</span>
                                <span class="text-on-primary-container text-sm ml-1">"CT Pros Already Listed"</span>
                            </div>
                        </div>
                    </div>
                    // Lead Capture Form
                    <div class="bg-surface-container-lowest rounded-xl p-8 shadow-2xl">
                        <h3 class="font-headline text-2xl font-bold text-on-surface mb-2">"List Your Service"</h3>
                        <p class="text-on-surface-variant text-sm mb-8">"New profiles are reviewed and approved within 48 hours."</p>
                        <div class="space-y-5">
                            <div>
                                <label class="text-[10px] uppercase tracking-widest font-bold text-on-surface-variant block mb-2">"Business Name"</label>
                                <input type="text" class="w-full border border-outline-variant/50 rounded-lg px-4 py-3 bg-transparent text-on-surface focus:outline-none focus:border-[#004289] transition-colors" placeholder="Your Business Name" />
                            </div>
                            <div>
                                <label class="text-[10px] uppercase tracking-widest font-bold text-on-surface-variant block mb-2">"Website / Portfolio"</label>
                                <input type="text" class="w-full border border-outline-variant/50 rounded-lg px-4 py-3 bg-transparent text-on-surface focus:outline-none focus:border-[#004289] transition-colors" placeholder="https://yourbusiness.com" />
                            </div>
                            <div class="grid grid-cols-2 gap-4">
                                <div>
                                    <label class="text-[10px] uppercase tracking-widest font-bold text-on-surface-variant block mb-2">"Years in Business"</label>
                                    <input type="text" class="w-full border border-outline-variant/50 rounded-lg px-4 py-3 bg-transparent text-on-surface focus:outline-none focus:border-[#004289] transition-colors" placeholder="e.g. 10" />
                                </div>
                                <div>
                                    <label class="text-[10px] uppercase tracking-widest font-bold text-on-surface-variant block mb-2">"Service Type"</label>
                                    <select class="w-full border border-outline-variant/50 rounded-lg px-4 py-3 bg-transparent text-on-surface focus:outline-none focus:border-[#004289] transition-colors appearance-none">
                                        {hp.form_category_options.iter().map(|opt| view! {
                                            <option>{opt.clone()}</option>
                                        }).collect_view()}
                                    </select>
                                </div>
                            </div>
                            <button class="w-full bg-[#004289] text-white py-4 rounded-lg font-bold text-lg hover:bg-[#00458f] transition-colors">
                                "Submit Application"
                            </button>
                        </div>
                    </div>
                </div>
            </section>

            // Trust Signals — Bento Grid
            <section class="py-24 px-8 max-w-7xl mx-auto">
                <div class="text-center mb-20">
                    <h2 class="font-headline text-4xl font-extrabold tracking-tight text-on-surface mb-4">{hp.trust_heading.clone()}</h2>
                    <p class="text-on-surface-variant max-w-2xl mx-auto">{hp.trust_subtitle.clone()}</p>
                </div>
                <div class="grid grid-cols-12 gap-6">
                    // Top row
                    <div class="col-span-12 md:col-span-8 relative bg-surface-container-low rounded-xl p-10 overflow-hidden min-h-[280px] flex flex-col justify-end">
                        <div class="absolute inset-0 opacity-20">
                            <img class="w-full h-full object-cover" src="https://images.unsplash.com/photo-1504307651254-35680f356dfd?ixlib=rb-4.0.3&auto=format&fit=crop&w=1200&q=80" alt="" />
                        </div>
                        <div class="relative z-10">
                            <div class="w-12 h-12 rounded-lg bg-[#004289] flex items-center justify-center mb-4">
                                <span class="material-symbols-outlined text-white text-2xl">"auto_awesome"</span>
                            </div>
                            <h3 class="font-headline text-2xl font-bold text-on-surface mb-2">"Targeted Local Exposure"</h3>
                            <p class="text-on-surface-variant max-w-md">"Homeowners in your area searching for exactly the services you provide. Our SEO puts your profile in front of customers ready to hire."</p>
                        </div>
                    </div>
                    <div class="col-span-12 md:col-span-4 bg-tertiary rounded-xl p-10 flex flex-col justify-end text-white min-h-[280px]">
                        <div class="w-12 h-12 rounded-lg bg-white/20 flex items-center justify-center mb-4">
                            <span class="material-symbols-outlined text-2xl">"trending_up"</span>
                        </div>
                        <h3 class="font-headline text-2xl font-bold mb-2">"3x More Leads"</h3>
                        <p class="text-white/80">"Listed contractors receive on average 3 times more quote requests than those relying on word-of-mouth alone."</p>
                    </div>
                    // Bottom row
                    <div class="col-span-12 md:col-span-4 bg-surface-container-low rounded-xl p-10 flex flex-col justify-end min-h-[240px]">
                        <div class="w-12 h-12 rounded-lg bg-[#004289]/10 flex items-center justify-center mb-4">
                            <span class="material-symbols-outlined text-[#004289] text-2xl">"shield"</span>
                        </div>
                        <h3 class="font-headline text-xl font-bold text-on-surface mb-2">"Verified Badge"</h3>
                        <p class="text-on-surface-variant text-sm">"Get a verified license badge on your profile. Homeowners trust verified contractors 4x more."</p>
                    </div>
                    <div class="col-span-12 md:col-span-8 bg-surface-container-low rounded-xl p-10 flex flex-col justify-end min-h-[240px]">
                        <div class="w-12 h-12 rounded-lg bg-[#004289]/10 flex items-center justify-center mb-4">
                            <span class="material-symbols-outlined text-[#004289] text-2xl">"group"</span>
                        </div>
                        <h3 class="font-headline text-xl font-bold text-on-surface mb-2">"Professional Profile"</h3>
                        <p class="text-on-surface-variant max-w-md">"Showcase your best work with project photos, customer reviews, and service details that help homeowners choose you."</p>
                    </div>
                </div>
            </section>

            // Testimonial
            <section class="py-24 px-8 border-t border-outline-variant/30">
                <div class="max-w-7xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-16 items-start">
                    <div>
                        <span class="material-symbols-outlined text-[#004289] text-5xl mb-8 block">"format_quote"</span>
                        <blockquote class="font-headline text-3xl md:text-4xl font-bold text-on-surface leading-tight mb-10">
                            {hp.testimonial_quote.clone()}
                        </blockquote>
                        <div class="flex items-center gap-4">
                            <img class="w-12 h-12 rounded-full object-cover" src="https://images.unsplash.com/photo-1560250097-0b93528c311a?auto=format&fit=crop&w=100" alt={hp.testimonial_name.clone()} />
                            <div>
                                <div class="font-bold text-on-surface">{hp.testimonial_name.clone()}</div>
                                <div class="text-sm text-on-surface-variant">{hp.testimonial_title.clone()}</div>
                            </div>
                        </div>
                    </div>
                    <div class="grid grid-cols-2 gap-4">
                        <div class="bg-surface-container-lowest p-6 rounded-xl shadow-sm">
                            <div class="flex gap-0.5 text-[#004289] mb-3">
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                            </div>
                            <p class="text-sm text-on-surface-variant italic leading-relaxed">"Best platform for getting new residential clients. My phone started ringing within the first week."</p>
                        </div>
                        <div class="bg-surface-container-lowest p-6 rounded-xl shadow-sm">
                            <div class="flex gap-0.5 text-[#004289] mb-3">
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                            </div>
                            <p class="text-sm text-on-surface-variant italic leading-relaxed">"The verified badge alone has been worth it. Customers trust us immediately."</p>
                        </div>
                        <div class="bg-surface-container-lowest p-6 rounded-xl shadow-sm">
                            <div class="flex gap-0.5 text-[#004289] mb-3">
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                            </div>
                            <p class="text-sm text-on-surface-variant italic leading-relaxed">"Revenue increased by 40% in our first quarter on the platform."</p>
                        </div>
                        <div class="bg-surface-container-lowest p-6 rounded-xl shadow-sm">
                            <div class="flex gap-0.5 text-[#004289] mb-3">
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                                <span class="material-symbols-outlined text-lg" style="font-variation-settings: 'FILL' 1;">"star"</span>
                            </div>
                            <p class="text-sm text-on-surface-variant italic leading-relaxed">"Finally, a directory that values quality work over just listing volume."</p>
                        </div>
                    </div>
                </div>
            </section>

            // Final CTA
            <section class="py-24 px-8">
                <div class="max-w-4xl mx-auto text-center">
                    <h2 class="font-headline text-4xl md:text-5xl font-extrabold tracking-tight text-on-surface mb-6">{hp.cta_headline.clone()}</h2>
                    <p class="text-on-surface-variant max-w-2xl mx-auto mb-12">{hp.cta_subtitle.clone()}</p>
                    <div class="flex flex-col md:flex-row gap-4 justify-center">
                        <a href="#" class="bg-[#004289] text-white px-10 py-4 rounded-lg font-bold hover:bg-[#00458f] transition-colors inline-block text-center">"Apply Now"</a>
                        <a href="/search" class="border border-outline-variant text-on-surface px-10 py-4 rounded-lg font-bold hover:bg-surface-container transition-colors inline-block text-center">"Browse Directory"</a>
                    </div>
                </div>
            </section>
        </crate::components::layout::MainLayout>
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
            <section class="relative min-h-[870px] flex items-center bg-[#004289] px-8 overflow-hidden">
                <div class="absolute inset-0 opacity-20 pointer-events-none">
                    <img class="w-full h-full object-cover grayscale" src="https://images.unsplash.com/photo-1504307651254-35680f356dfd?ixlib=rb-4.0.3&auto=format&fit=crop&w=2000&q=80" alt="Home renovation" />
                </div>
                <div class="relative max-w-7xl mx-auto w-full py-24 z-10">
                    <div class="max-w-3xl">
                        <h1 class="font-headline text-white text-6xl md:text-7xl font-extrabold tracking-tighter leading-tight mb-8">
                            {config.hero_headline.clone()}
                        </h1>
                        <p class="text-on-primary-container text-xl md:text-2xl font-light mb-12 max-w-2xl leading-relaxed">
                            {config.hero_subtitle.clone()}
                        </p>
                        <div class="bg-surface-container-lowest p-2 rounded-xl shadow-xl flex flex-col md:flex-row gap-2">
                            <div class="flex-1 flex items-center px-4 py-3 gap-3 border-b md:border-b-0 md:border-r border-outline-variant/20">
                                <span class="material-symbols-outlined text-outline">"search"</span>
                                <input class="w-full border-none focus:ring-0 bg-transparent text-on-surface placeholder:text-outline font-medium focus:outline-none" placeholder={config.search_placeholder_keyword.clone()} type="text" name="q" />
                            </div>
                            <div class="flex-1 flex items-center px-4 py-3 gap-3 border-b md:border-b-0 md:border-r border-outline-variant/20">
                                <span class="material-symbols-outlined text-outline">"location_on"</span>
                                <input class="w-full border-none focus:ring-0 bg-transparent text-on-surface placeholder:text-outline font-medium focus:outline-none" placeholder={config.search_placeholder_location.clone()} type="text" name="location" />
                            </div>
                            <div class="flex-1 flex items-center px-4 py-3 gap-3">
                                <span class="material-symbols-outlined text-outline">"category"</span>
                                <select class="w-full border-none focus:ring-0 bg-transparent text-on-surface font-medium appearance-none focus:outline-none">
                                    <option>"All Services"</option>
                                    {config.categories.iter().map(|cat| view! {
                                        <option>{cat.label.clone()}</option>
                                    }).collect_view()}
                                </select>
                            </div>
                            <a href="/search" class="bg-[#004289] text-white px-10 py-4 rounded-lg font-bold hover:bg-[#00458f] transition-colors text-center">
                                "Search Directory"
                            </a>
                        </div>
                    </div>
                </div>
            </section>

            // Featured Listings
            <section class="py-24 px-8 max-w-7xl mx-auto">
                <div class="flex justify-between items-end mb-16">
                    <div class="max-w-xl">
                        <span class="text-tertiary font-bold tracking-widest text-xs uppercase mb-3 block">"Top Rated"</span>
                        <h2 class="font-headline text-4xl font-extrabold tracking-tight text-on-surface mb-4">"Featured Service Providers"</h2>
                        <p class="text-on-surface-variant leading-relaxed">"Vetted, reviewed, and trusted by Connecticut homeowners for quality renovations and repairs."</p>
                    </div>
                    <a href="/search" class="hidden md:flex items-center gap-2 text-[#004289] font-bold hover:underline underline-offset-8">
                        "View All Directory " <span class="material-symbols-outlined">"arrow_forward"</span>
                    </a>
                </div>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-12">
                    {config.featured_listings.iter().map(|fl| {
                        let href = format!("/{}", fl.slug);
                        view! {
                            <a href={href} class="group cursor-pointer block">
                                <div class="relative aspect-[4/5] overflow-hidden mb-6 rounded-sm bg-surface-container-low">
                                    <img class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" src={fl.image_url.clone()} alt={fl.title.clone()} />
                                    <div class="absolute top-4 right-4 bg-white/90 backdrop-blur-md px-3 py-1 rounded-full flex items-center gap-1 shadow-sm">
                                        <span class="material-symbols-outlined text-tertiary text-sm" style="font-variation-settings: 'FILL' 1;">{fl.badge_icon.clone()}</span>
                                        <span class="text-[10px] font-bold text-on-surface uppercase tracking-wider">{fl.badge_label.clone()}</span>
                                    </div>
                                </div>
                                <div class="flex justify-between items-start mb-2">
                                    <h3 class="font-headline text-xl font-bold text-on-surface group-hover:text-[#004289] transition-colors">{fl.title.clone()}</h3>
                                    {match &fl.price_label {
                                        Some(price) => view! { <span class="text-tertiary font-bold">{price.clone()}</span> }.into_any(),
                                        None => view! { <span class="material-symbols-outlined text-outline hover:text-[#004289] transition-colors">"favorite"</span> }.into_any(),
                                    }}
                                </div>
                                <p class="text-sm text-on-surface-variant font-medium mb-4">{fl.subtitle.clone()}</p>
                                <div class="flex gap-2">
                                    {fl.tags.iter().map(|tag| view! {
                                        <span class="bg-surface-container-high px-3 py-1 rounded-full text-[10px] font-bold text-on-surface-variant uppercase tracking-tighter">{tag.clone()}</span>
                                    }).collect_view()}
                                </div>
                            </a>
                        }
                    }).collect_view()}
                </div>
            </section>

            // Explore by Specialization
            <section class="bg-surface-container-low py-24">
                <div class="max-w-7xl mx-auto px-8">
                    <div class="text-center mb-20">
                        <h2 class="font-headline text-4xl font-extrabold tracking-tight text-on-surface mb-4">"Browse by Service Type"</h2>
                        <p class="text-on-surface-variant max-w-2xl mx-auto">"Find the right professional for your project from our curated categories."</p>
                    </div>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-6">
                        {config.categories.iter().map(|cat| {
                            let href = format!("/search?category={}", cat.slug);
                            view! {
                                <a href={href} class="bg-surface-container-lowest p-8 rounded-xl shadow-sm hover:shadow-md transition-all flex flex-col items-center text-center cursor-pointer border border-transparent hover:border-[#004289]/10">
                                    <div class="w-16 h-16 rounded-full bg-secondary-container flex items-center justify-center mb-6">
                                        <span class="material-symbols-outlined text-on-secondary-container text-3xl">{cat.icon.clone()}</span>
                                    </div>
                                    <h4 class="font-headline font-bold text-lg mb-2">{cat.label.clone()}</h4>
                                    <p class="text-xs text-on-surface-variant">{cat.subtitle.clone()}</p>
                                </a>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </section>

            // How It Works — Process
            <section class="py-24 px-8 max-w-7xl mx-auto">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-24 items-center">
                    <div class="relative">
                        <div class="aspect-square bg-surface-container rounded-sm overflow-hidden shadow-2xl">
                            <img class="w-full h-full object-cover" src="https://images.unsplash.com/photo-1581578731548-c64695cc6952?ixlib=rb-4.0.3&auto=format&fit=crop&w=1000&q=80" alt="Contractor at work" />
                        </div>
                        <div class="absolute -bottom-10 -right-10 bg-[#004289] p-12 text-white hidden md:block rounded-sm shadow-xl">
                            <div class="text-5xl font-extrabold mb-2">"200+"</div>
                            <div class="text-on-primary-container font-headline tracking-widest text-xs uppercase">"Verified CT Pros"</div>
                        </div>
                    </div>
                    <div>
                        <span class="text-tertiary font-bold tracking-widest text-xs uppercase mb-3 block">"How It Works"</span>
                        <h2 class="font-headline text-4xl font-extrabold tracking-tight text-on-surface mb-12">"Find your perfect contractor"</h2>
                        <div class="space-y-10">
                            {config.process_steps.iter().map(|step| view! {
                                <div class="flex gap-6">
                                    <div class="flex-shrink-0 w-12 h-12 rounded-full border border-outline-variant flex items-center justify-center font-headline font-bold text-[#004289]">{step.number.clone()}</div>
                                    <div>
                                        <h4 class="font-headline font-bold text-xl mb-2">{step.title.clone()}</h4>
                                        <p class="text-on-surface-variant leading-relaxed">{step.description.clone()}</p>
                                    </div>
                                </div>
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </section>

            // Final CTA
            <section class="bg-[#004289] py-24">
                <div class="max-w-4xl mx-auto px-8 text-center">
                    <h2 class="font-headline text-white text-4xl md:text-5xl font-extrabold mb-8 tracking-tight">{config.cta_headline.clone()}</h2>
                    <div class="flex flex-col md:flex-row gap-4 justify-center">
                        <a href="/list-property" class="bg-white text-[#004289] px-10 py-4 rounded-lg font-bold hover:bg-slate-100 transition-colors inline-block text-center">"List Your Service"</a>
                        <a href="/search" class="border border-on-primary-container text-white px-10 py-4 rounded-lg font-bold hover:bg-white/10 transition-colors inline-block text-center">"Browse Directory"</a>
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
                <leptos_router::components::Route path=leptos_router::path!("/list-property") view=HostLanding />
                <leptos_router::components::Route path=leptos_router::path!(":slug") view=ListingDetail />
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
