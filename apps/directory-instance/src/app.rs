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
        <div class="bg-white border border-neutral-200/60 rounded-2xl p-8 shadow-premium">
            {move || if success.get() {
                view! {
                    <div class="text-center space-y-4 animate-fade-scale">
                        <div class="inline-flex items-center justify-center w-14 h-14 rounded-2xl bg-gradient-to-br from-green-400 to-emerald-500 text-white mb-4 shadow-lg shadow-green-500/20">
                            <svg class="w-7 h-7" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7"></path></svg>
                        </div>
                        <h3 class="text-xl font-bold">"Success!"</h3>
                        <p class="text-neutral-500 text-sm">"Your request has been received. Check your email for next steps."</p>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="space-y-6">
                        <div class="flex items-center justify-between border-b pb-4 mb-4 border-neutral-200/60">
                            <h3 class="text-lg font-bold">"Request Information"</h3>
                            <span class="text-[11px] font-semibold px-2.5 py-1 bg-primary/10 text-primary rounded-lg">
                                "Step " {move || step.get()} " / 2"
                            </span>
                        </div>
                        
                        {move || if !error.get().is_empty() {
                            view! { <p class="text-destructive text-sm bg-destructive/5 border border-destructive/20 p-3 rounded-xl animate-slide-up">{error.get()}</p> }.into_any()
                        } else { view!{ <span/> }.into_any() }}
                        
                        // Honeypot Field
                        <div class="hidden" aria-hidden="true">
                            <input type="text" name="_bot_check" prop:value=bot_check on:input=move |ev| bot_check.set(event_target_value(&ev)) tabindex="-1" />
                        </div>

                        {move || if step.get() == 1 {
                            view! {
                                <div class="space-y-4 animate-slide-up">
                                    <div class="grid gap-1.5">
                                        <label class="text-[13px] font-semibold text-neutral-700">"Full Name *"</label>
                                        <input type="text" placeholder="John Doe" class="flex h-11 w-full rounded-xl border border-neutral-200 bg-white px-4 py-2 text-sm focus:ring-2 focus:ring-primary/30 focus:border-primary/40 outline-none transition-all duration-200" prop:value=name on:input=move |ev| name.set(event_target_value(&ev)) />
                                    </div>
                                    <div class="grid gap-1.5">
                                        <label class="text-[13px] font-semibold text-neutral-700">"Email Address *"</label>
                                        <input type="email" placeholder="john@example.com" class="flex h-11 w-full rounded-xl border border-neutral-200 bg-white px-4 py-2 text-sm focus:ring-2 focus:ring-primary/30 focus:border-primary/40 outline-none transition-all duration-200" prop:value=email on:input=move |ev| email.set(event_target_value(&ev)) />
                                    </div>
                                    
                                    <button 
                                        class="mt-4 w-full bg-gradient-to-r from-primary to-primary/90 text-white hover:opacity-90 rounded-xl h-11 px-4 py-2 font-semibold text-sm transition-all duration-200 shadow-glow"
                                        on:click=handle_next
                                    >
                                        "Continue"
                                    </button>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="space-y-4 animate-slide-up">
                                    <div class="grid gap-1.5">
                                        <label class="text-[13px] font-semibold text-neutral-700 flex justify-between">
                                            <span>"Phone Number"</span>
                                            <span class="text-neutral-400 text-[11px] font-normal">"Optional"</span>
                                        </label>
                                        <input type="tel" placeholder="+1 (555) 000-0000" class="flex h-11 w-full rounded-xl border border-neutral-200 bg-white px-4 py-2 text-sm focus:ring-2 focus:ring-primary/30 focus:border-primary/40 outline-none transition-all duration-200" prop:value=phone on:input=move |ev| phone.set(event_target_value(&ev)) />
                                    </div>
                                    <div class="grid gap-1.5">
                                        <label class="text-[13px] font-semibold text-neutral-700">"What are you looking for? *"</label>
                                        <textarea rows="3" placeholder="I'm interested in..." class="flex w-full rounded-xl border border-neutral-200 bg-white px-4 py-2 text-sm focus:ring-2 focus:ring-primary/30 focus:border-primary/40 outline-none transition-all duration-200" prop:value=intent on:input=move |ev| intent.set(event_target_value(&ev))></textarea>
                                    </div>
                                    
                                    <div class="flex gap-2 mt-4">
                                        <button 
                                            class="w-1/3 border border-neutral-200 bg-white hover:bg-neutral-50 text-neutral-600 rounded-xl h-11 px-4 py-2 font-semibold text-sm transition-all duration-200"
                                            on:click=move |_| { step.set(1); error.set("".to_string()); }
                                        >
                                            "Back"
                                        </button>
                                        <button 
                                            class="w-2/3 bg-gradient-to-r from-primary to-primary/90 text-white hover:opacity-90 rounded-xl h-11 px-4 py-2 font-semibold text-sm disabled:opacity-50 transition-all duration-200 flex justify-center items-center shadow-glow"
                                            on:click=handle_submit
                                            disabled=is_submitting
                                        >
                                            {move || if is_submitting.get() { "Processing...".to_string() } else { cta_text_sig.get() }}
                                        </button>
                                    </div>
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
                            <div class="w-full relative pb-24">
                                <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_top,_var(--tw-gradient-stops))] from-primary/5 via-background to-background pointer-events-none"></div>

                                // Add the WhatsApp floating button if configured!
                                {whatsapp.map(|hp| {
                                    let wa_link = format!("https://wa.me/{}", hp.replace(&['+', ' ', '-'][..], ""));
                                    view! {
                                        <a href=wa_link target="_blank" rel="noopener noreferrer" class="fixed bottom-8 right-8 z-50 bg-[#25D366] hover:bg-[#128C7E] text-white p-5 rounded-full shadow-2xl shadow-[#25D366]/30 transition-all hover:scale-110 flex items-center justify-center border border-white/20 animate-bounce">
                                            <svg class="w-8 h-8" fill="currentColor" viewBox="0 0 24 24"><path d="M12.031 6.172c-3.181 0-5.767 2.586-5.768 5.766-.001 1.298.38 2.27 1.019 3.287l-.582 2.128 2.182-.573c.978.58 1.911.928 3.145.929 3.178 0 5.767-2.587 5.768-5.766.001-3.187-2.575-5.77-5.764-5.771zm3.392 8.244c-.144.405-.837.774-1.17.824-.299.045-.677.063-1.092-.069-.252-.08-.57-.187-.988-.365-1.739-.751-2.874-2.502-2.961-2.617-.087-.116-.708-.94-.708-1.793s.448-1.273.607-1.446c.159-.173.346-.217.462-.217l.332.006c.106.005.249-.04.39.298.144.347.491 1.2.534 1.287.043.087.072.188.014.304-.058.116-.087.188-.173.289l-.26.304c-.087.086-.177.18-.076.354.101.174.449.741.964 1.201.662.591 1.221.774 1.391.86s.274.066.376-.05.399-.465.511-.624.22-.132.378-.073c.159.058 1.002.472 1.176.559.173.087.289.13.332.202.043.073.043.423-.101.828z" fill-rule="evenodd" clip-rule="evenodd"></path></svg>
                                        </a>
                                    }.into_any()
                                })}

                                // Sticky Contact Header
                                <div class="sticky top-0 z-50 w-full glass-premium">
                                    <div class="h-[2px] w-full bg-gradient-to-r from-transparent via-primary to-transparent opacity-30"></div>
                                    <div class="max-w-7xl mx-auto px-6 h-[72px] flex items-center justify-between">
                                        <div class="flex items-center gap-3">
                                            <a href="/search" class="w-9 h-9 border border-neutral-200 hover:bg-neutral-100 rounded-xl flex items-center justify-center text-neutral-500 hover:text-foreground transition-all duration-200 mr-1">
                                                <svg class="w-4 h-4 stroke-2" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M10 19l-7-7m0 0l7-7m-7 7h18"></path></svg>
                                            </a>
                                            <div>
                                                <h2 class="text-base font-bold text-foreground leading-tight line-clamp-1">{listing.title.clone()}</h2>
                                                <p class="text-[11px] text-neutral-500 font-semibold uppercase tracking-wider hidden md:block">{listing.listing_type.clone()}</p>
                                            </div>
                                        </div>
                                        <a href="#contact-form" class="bg-gradient-to-r from-primary to-primary/90 text-white px-6 py-2.5 rounded-xl font-semibold shadow-glow hover:shadow-glow-lg transition-all duration-300 text-sm whitespace-nowrap">
                                            {cta_text.to_string()}
                                        </a>
                                    </div>
                                </div>

                                // Premium Airbnb 5-Image Masonry Header
                                <div class="max-w-[1120px] mx-auto pt-8 pb-4 px-6 md:px-10">
                                    // Header Area (Left aligned, no background block)
                                    <div class="space-y-4 mb-8">
                                        <h1 class="text-3xl md:text-[32px] font-semibold text-foreground tracking-tight leading-tight">{hero_headline.to_string()}</h1>
                                        
                                        <div class="flex items-center justify-between text-[15px] font-medium text-foreground">
                                            <div class="flex items-center gap-4 flex-wrap">
                                                <span class="flex items-center gap-1 font-semibold">
                                                    <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20"><path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z"></path></svg>
                                                    "4.92"
                                                </span>
                                                <span class="underline cursor-pointer">"128 reviews"</span>
                                                <span class="text-muted-foreground">"·"</span>
                                                <span class="flex items-center gap-1 opacity-80">
                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
                                                    "Superhost"
                                                </span>
                                                <span class="text-muted-foreground">"·"</span>
                                                <span class="underline cursor-pointer opacity-80">{listing.title.clone()}</span>
                                            </div>
                                            
                                            <div class="hidden sm:flex items-center gap-4">
                                                <button class="flex items-center gap-2 hover:bg-muted px-3 py-2 rounded-lg transition-colors underline">
                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"></path></svg>
                                                    "Share"
                                                </button>
                                                <button class="flex items-center gap-2 hover:bg-muted px-3 py-2 rounded-lg transition-colors underline">
                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"></path></svg>
                                                    "Save"
                                                </button>
                                            </div>
                                        </div>
                                    </div>

                                    // 5-Image Grid Masonry
                                    <div class="h-[300px] sm:h-[400px] md:h-[500px] w-full rounded-2xl overflow-hidden flex gap-2 relative group cursor-pointer">
                                        // Left Big Image
                                        <div class="w-full md:w-1/2 h-full relative overflow-hidden">
                                            <div class="absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors duration-300 z-10 hover:!bg-transparent cursor-pointer"></div>
                                            <img src="https://images.unsplash.com/photo-1600596542815-ffad4c1539a9?q=80&w=1600&auto=format&fit=crop" class="w-full h-full object-cover transition-transform duration-700 hover:scale-105" alt="Main View" />
                                        </div>
                                        // Right 4-Grid
                                        <div class="hidden md:flex w-1/2 h-full flex-col gap-2">
                                            <div class="flex-1 flex gap-2 h-1/2">
                                                <div class="w-1/2 h-full relative overflow-hidden">
                                                    <div class="absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors duration-300 z-10 hover:!bg-transparent cursor-pointer"></div>
                                                    <img src="https://images.unsplash.com/photo-1600607687931-cebf0305ab59?q=80&w=800&auto=format&fit=crop" class="w-full h-full object-cover transition-transform duration-700 hover:scale-105" alt="Detail 1" />
                                                </div>
                                                <div class="w-1/2 h-full relative overflow-hidden">
                                                    <div class="absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors duration-300 z-10 hover:!bg-transparent cursor-pointer"></div>
                                                    <img src="https://images.unsplash.com/photo-1600585154340-be6161a56a0c?q=80&w=800&auto=format&fit=crop" class="w-full h-full object-cover transition-transform duration-700 hover:scale-105" alt="Detail 2" />
                                                </div>
                                            </div>
                                            <div class="flex-1 flex gap-2 h-1/2">
                                                <div class="w-1/2 h-full relative overflow-hidden">
                                                    <div class="absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors duration-300 z-10 hover:!bg-transparent cursor-pointer"></div>
                                                    <img src="https://images.unsplash.com/photo-1600566753190-17f0baa2a6c3?q=80&w=800&auto=format&fit=crop" class="w-full h-full object-cover transition-transform duration-700 hover:scale-105" alt="Detail 3" />
                                                </div>
                                                <div class="w-1/2 h-full relative overflow-hidden">
                                                    <div class="absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors duration-300 z-10 hover:!bg-transparent cursor-pointer"></div>
                                                    <img src="https://images.unsplash.com/photo-1600210492486-724fe5c67fb0?q=80&w=800&auto=format&fit=crop" class="w-full h-full object-cover transition-transform duration-700 hover:scale-105" alt="Detail 4" />
                                                </div>
                                            </div>
                                        </div>
                                        
                                        <button class="absolute bottom-6 right-6 z-20 bg-white border border-foreground px-4 py-2 rounded-lg font-semibold text-[15px] shadow-sm flex items-center gap-2 hover:bg-muted transition-colors active:scale-95">
                                            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"></path></svg>
                                            "Show all photos"
                                        </button>
                                    </div>
                                </div>
                                
                                // Trust Signals / Social Proof Bar
                                <div class="w-full bg-white/80 backdrop-blur-xl border-b border-neutral-200/60 relative z-20">
                                    <div class="max-w-6xl mx-auto px-6 py-5 flex flex-wrap justify-center gap-8 md:gap-16 text-[13px] font-semibold tracking-wider uppercase text-neutral-500">
                                        <div class="flex items-center gap-2.5 hover:text-foreground transition-colors duration-200"><div class="p-1.5 rounded-lg bg-emerald-500/10 text-emerald-600"><svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg></div> "Verified"</div>
                                        <div class="flex items-center gap-2.5 hover:text-foreground transition-colors duration-200"><div class="p-1.5 rounded-lg bg-amber-500/10 text-amber-600"><svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20"><path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z"></path></svg></div> "Top Rated"</div>
                                        <div class="flex items-center gap-2.5 hover:text-foreground transition-colors duration-200"><div class="p-1.5 rounded-lg bg-blue-500/10 text-blue-600"><svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v8l9-11h-7z"></path></svg></div> "Fast Response"</div>
                                    </div>
                                </div>
                                
                                // Content & Form Layout Section
                                <div class="max-w-[1120px] mx-auto px-6 py-12 grid grid-cols-1 lg:grid-cols-[1fr_32%_32%] lg:grid-cols-12 gap-10 md:gap-20 relative z-10">
                                    // Left details panel
                                    <div class="lg:col-span-8 space-y-12">
                                        // Main description body
                                        <div class="prose prose-lg max-w-none text-foreground/80 font-normal leading-[1.6]">
                                            <div class="flex items-center gap-4 mb-8 pb-8 border-b border-border">
                                                <div class="w-14 h-14 rounded-full bg-muted flex items-center justify-center font-bold text-xl overflow-hidden border border-border">
                                                    <img src="https://images.unsplash.com/photo-1560250097-[...]&auto=format&fit=crop" class="w-full h-full object-cover" alt="Host"/>
                                                </div>
                                                <div>
                                                    <h3 class="text-lg font-bold text-foreground m-0 leading-tight">"Hosted by Directory Professional"</h3>
                                                    <p class="text-[15px] text-muted-foreground m-0">"5 years hosting" <span class="mx-1">"·"</span> "Response rate: 100%"</p>
                                                </div>
                                            </div>
                                            <div inner_html=listing.description.clone()></div>
                                        </div>
                                        
                                        // Features & Attributes Grid
                                        <div class="bg-neutral-50/50 border border-neutral-200/60 rounded-2xl p-8 md:p-10">
                                            <h3 class="text-xl font-bold text-foreground mb-6">"Features & Amenities"</h3>
                                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
                                                {
                                                    let ignored_keys = ["hero_headline", "whatsapp_number", "cta_text"];
                                                    listing.attributes.iter()
                                                        .filter(|(k, _)| !ignored_keys.contains(&k.as_str()))
                                                        .map(|(k, v)| {
                                                            let display_key = k.replace("_", " ");
                                                            view! {
                                                                <div class="flex items-start gap-4 p-5 bg-white border border-neutral-200/60 rounded-xl shadow-premium hover:shadow-card-hover hover:-translate-y-0.5 transition-all duration-300">
                                                                    <div class="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center text-primary flex-shrink-0">
                                                                        <shared_ui::components::attribute_icon::AttributeIcon name=k.clone() class="w-6 h-6".to_string() />
                                                                    </div>
                                                                    <div>
                                                                        <p class="text-xs font-bold uppercase tracking-wider text-muted-foreground mb-1">{display_key}</p>
                                                                        <p class="text-foreground font-semibold line-clamp-2">{v.clone()}</p>
                                                                    </div>
                                                                </div>
                                                            }
                                                        }).collect_view()
                                                }
                                                {
                                                    let ignored_keys = ["hero_headline", "whatsapp_number", "cta_text"];
                                                    if listing.attributes.iter().filter(|(k, _)| !ignored_keys.contains(&k.as_str())).count() == 0 {
                                                        view! {
                                                            <div class="col-span-full text-center py-8 text-muted-foreground font-medium">
                                                                "No additional amenities listed."
                                                            </div>
                                                        }.into_any()
                                                    } else { view!{<div/>}.into_any() }
                                                }
                                            </div>
                                        </div>
                                    </div>
                                    
                                    // Right sticky lead form conversion engine (Airbnb Booking Card style)
                                    <div class="lg:col-span-4 relative">
                                        <div class="sticky top-[100px]">
                                            <div id="contact-form" class="bg-white rounded-2xl shadow-premium-lg border border-neutral-200/60 p-6 relative overflow-hidden">
                                                <div class="absolute top-0 left-0 right-0 h-[3px] bg-gradient-to-r from-primary via-purple-500 to-pink-500 opacity-70"></div>
                                                <div class="flex items-baseline gap-1.5 mb-6 pt-2">
                                                    <span class="text-2xl font-bold text-foreground">"$1,200"</span>
                                                    <span class="text-sm text-neutral-500 font-medium">"Est. Total"</span>
                                                </div>
                                                
                                                <div class="bg-white rounded-lg p-0 mb-4 border-none">
                                                    <LeadForm listing_id=l_id.clone() cta_text=cta_text.to_string() />
                                                </div>
                                                
                                                <div class="text-center mt-4 text-[13px] text-neutral-400">
                                                    "You won't be charged yet"
                                                </div>
                                                
                                                <div class="mt-5 pt-5 border-t border-neutral-200/60 flex justify-between text-[13px] text-neutral-500 hover:text-foreground transition-colors cursor-pointer">
                                                    <span>"Report this listing"</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                
                                // Related Listings Carousel
                                <section class="max-w-7xl mx-auto w-full px-6 py-16 border-t border-neutral-200/60 mt-12 bg-white rounded-t-3xl">
                                    <div class="accent-line mb-3"></div>
                                    <h3 class="text-2xl font-bold tracking-tight text-foreground mb-10">"Similar Professionals"</h3>
                                    <div class="flex overflow-x-auto gap-6 pb-8 snap-x pl-2 mb-[-2rem] -mx-2 hide-scrollbar">
                                        {vec![
                                            ("Apex CT Renovations", "Construction", "ct-contractor-demo", "New Haven, CT"),
                                            ("Elite HVAC Professionals", "Plumbing & HVAC", "elite-hvac", "Stamford, CT"),
                                            ("Green Lawn Pros", "Landscaping", "green-lawn", "Hartford, CT"),
                                        ].into_iter().filter(|(_, _, slug, _)| *slug != l_id.as_str()).map(|(name, cat, slug, loc)| view! {
                                            <a href=format!("/{}", slug) class="snap-start flex-shrink-0 w-80 group bg-white border border-border shadow-sm hover:shadow-xl hover:border-primary/40 rounded-2xl overflow-hidden transition-all duration-300">
                                                <div class="w-full h-48 bg-muted relative overflow-hidden">
                                                    <svg class="w-16 h-16 text-muted-foreground/30 absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 group-hover:scale-110 transition-transform duration-500" fill="currentColor" viewBox="0 0 24 24"><path d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"></path></svg>
                                                </div>
                                                <div class="p-6">
                                                    <p class="text-xs text-primary font-bold uppercase tracking-wider mb-2">{cat}</p>
                                                    <h4 class="text-xl font-bold mb-1 group-hover:text-primary transition-colors text-foreground">{name}</h4>
                                                    <div class="flex items-center gap-1 text-muted-foreground text-sm font-medium mt-3">
                                                        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"></path><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"></path></svg>
                                                        {loc}
                                                    </div>
                                                </div>
                                            </a>
                                        }).collect_view()}
                                    </div>
                                </section>
                            </div>
                        </crate::components::layout::MainLayout>
                    }.into_any()
                },
                Some(Err(_)) => view! {
                    <Title text="Page Not Found" />
                    <div class="min-h-[60vh] flex flex-col items-center justify-center text-center p-8">
                        <h2 class="text-3xl font-bold text-muted-foreground mb-4">"Page Not Found"</h2>
                        <p>"The page you are looking for does not exist."</p>
                    </div>
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
            <div class="relative flex flex-col bg-background">
                // Massive Immersive Hero Section
                <section class="relative pt-32 pb-40 lg:pt-48 lg:pb-56 px-4 md:px-10 w-full flex flex-col justify-center overflow-hidden">
                    <img src="https://images.unsplash.com/photo-1560518883-ce09059eeffa?q=80&w=2000&auto=format&fit=crop" class="absolute inset-0 w-full h-full object-cover scale-[1.02] transform origin-center" alt="Hero Background" />
                    <div class="absolute inset-0 bg-gradient-to-b from-black/50 via-black/30 to-black/70"></div>
                    
                    <div class="max-w-[1000px] mx-auto text-center space-y-8 relative z-10 animate-slide-up">
                        <h1 class="text-4xl md:text-6xl lg:text-7xl font-extrabold tracking-tight drop-shadow-lg leading-[1.08] text-white">
                            "Find Exactly What " <br class="hidden md:block" /> "You Need in " <span class="bg-gradient-to-r from-blue-400 via-primary to-purple-400 bg-clip-text text-transparent drop-shadow-xl">{config.name.clone()}</span>
                        </h1>
                        
                        <p class="text-lg md:text-xl text-white/80 max-w-xl mx-auto leading-relaxed font-normal">
                            {config.description.clone()}
                        </p>
                        
                        // Smart Top-level Search Bar (Zillow/Airbnb Style)
                        <form class="mt-10 mx-auto relative flex w-full max-w-3xl shadow-premium-xl rounded-2xl bg-white/95 backdrop-blur-sm focus-within:shadow-glow-lg transition-all duration-500" action="/search" method="GET">
                            <div class="flex-1 flex items-center px-6 md:px-8 border-r border-neutral-200/60 hover:bg-neutral-50/50 cursor-pointer rounded-l-2xl transition-colors">
                                <div class="flex flex-col flex-1 py-3 md:py-3.5 text-left">
                                    <span class="text-[10px] md:text-[11px] font-bold uppercase tracking-widest text-neutral-500 mb-0.5">"Location"</span>
                                    <input type="text" name="location" class="bg-transparent text-foreground font-medium text-sm md:text-base focus:outline-none placeholder:text-neutral-400 w-full truncate" placeholder="Search destinations" autocomplete="off" />
                                </div>
                            </div>
                            <div class="flex-1 hidden sm:flex items-center px-8 hover:bg-neutral-50/50 cursor-pointer transition-colors">
                                <div class="flex flex-col flex-1 py-3.5 text-left">
                                    <span class="text-[11px] font-bold uppercase tracking-widest text-neutral-500 mb-0.5">"Service"</span>
                                    <input type="text" name="q" class="bg-transparent text-foreground font-medium text-base focus:outline-none placeholder:text-neutral-400 w-full truncate" placeholder="What do you need?" autocomplete="off" />
                                </div>
                            </div>
                            <div class="pl-2 pr-2.5 py-2.5 flex items-center rounded-r-2xl">
                                <button type="submit" class="h-12 w-12 md:h-12 md:w-auto md:px-8 rounded-xl bg-gradient-to-r from-primary to-primary/90 text-white font-semibold text-sm transition-all duration-300 hover:shadow-glow flex items-center justify-center gap-2">
                                    <svg class="w-5 h-5 stroke-[2.5]" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"></path></svg>
                                    <span class="hidden md:block">"Search"</span>
                                </button>
                            </div>
                        </form>
                        
                        <div class="flex flex-wrap justify-center gap-3 mt-6">
                            <a href="/search?category=plumbers" class="px-4 py-2 rounded-xl bg-white/15 backdrop-blur-sm text-white/90 text-sm font-medium hover:bg-white/25 transition-all duration-200 border border-white/10">"Plumbers"</a>
                            <a href="/search?category=contractors" class="px-4 py-2 rounded-xl bg-white/15 backdrop-blur-sm text-white/90 text-sm font-medium hover:bg-white/25 transition-all duration-200 border border-white/10">"Contractors"</a>
                            <a href="/search?category=real-estate" class="px-4 py-2 rounded-xl bg-white/15 backdrop-blur-sm text-white/90 text-sm font-medium hover:bg-white/25 transition-all duration-200 border border-white/10">"Real Estate"</a>
                        </div>
                    </div>
                </section>

                <div class="bg-background relative z-20 pt-8">
                    <crate::components::category_nav::CategoryNavigation />
                </div>
                
                // Recently Added Section (Borderless/Flush Image Cards)
                <section class="max-w-[1440px] mx-auto w-full px-6 md:px-10 pb-24 pt-14 animate-slide-up">
                    <div class="flex items-center justify-between mb-10">
                        <div>
                            <div class="accent-line mb-3"></div>
                            <h3 class="text-2xl font-bold tracking-tight text-foreground">"Recently Added"</h3>
                        </div>
                        <a href="/search" class="text-sm font-semibold text-primary hover:text-primary/80 transition-colors flex items-center gap-1.5 group">
                            "View all"
                            <svg class="w-4 h-4 group-hover:translate-x-0.5 transition-transform duration-200" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"></path></svg>
                        </a>
                    </div>
                    // Horizontal scroll container
                    <div class="flex overflow-x-auto gap-6 pb-10 snap-x snap-mandatory mx-[-1rem] px-4 hide-scrollbar">
                        {vec![
                            ("Apex CT Renovations", "Construction & Remodeling", "ct-contractor-demo", "New Haven, CT", "https://images.unsplash.com/photo-1503387762-592deb58ef4e?q=80&w=800&auto=format&fit=crop"),
                            ("Elite HVAC Professionals", "Plumbing & HVAC", "elite-hvac", "Stamford, CT", "https://images.unsplash.com/photo-1585435465945-bef5a93f8849?q=80&w=800&auto=format&fit=crop"),
                            ("Green Lawn Pros", "Outdoor & Landscaping", "green-lawn", "Hartford, CT", "https://images.unsplash.com/photo-1558904541-efa843a96f09?q=80&w=800&auto=format&fit=crop"),
                            ("Prime Real Estate CT", "Real Estate", "prime-real-estate", "Greenwich, CT", "https://images.unsplash.com/photo-1560518883-ce09059eeffa?q=80&w=800&auto=format&fit=crop"),
                        ].into_iter().map(|(name, cat, slug, loc, img)| view! {
                            <a href=format!("/{}", slug) class="snap-start flex-shrink-0 w-72 group cursor-pointer">
                                <div class="w-full h-72 relative overflow-hidden rounded-2xl mb-3 transition-all duration-500 group-hover:shadow-card-hover group-hover:-translate-y-1 bg-neutral-100">
                                    <img src=img alt=name class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105" />
                                    <div class="absolute inset-0 bg-gradient-to-t from-black/10 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500"></div>
                                    <div class="absolute top-3 left-3 bg-white/95 backdrop-blur-sm px-2.5 py-1 text-[11px] font-bold text-neutral-700 uppercase rounded-lg shadow-sm tracking-wider">"New"</div>
                                    <button class="absolute top-4 right-4 p-2 rounded-full bg-white/20 hover:bg-white backdrop-blur-md text-white hover:text-rose-500 transition-colors">
                                        <svg class="w-5 h-5 stroke-2" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"></path></svg>
                                    </button>
                                </div>
                                <div class="px-1 text-left">
                                    <div class="flex justify-between items-start mb-0.5">
                                        <h4 class="text-[15px] font-semibold text-foreground truncate max-w-[80%]">{name}</h4>
                                        <div class="flex items-center text-[14px] font-medium gap-1">
                                            <svg class="w-3.5 h-3.5 text-amber-500" fill="currentColor" viewBox="0 0 20 20"><path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z"></path></svg>
                                            "4.9"
                                        </div>
                                    </div>
                                    <p class="text-[14px] text-neutral-500 truncate">{cat}</p>
                                    <p class="text-[14px] text-neutral-400 mt-0.5">{loc}</p>
                                    <p class="text-[14px] font-semibold text-foreground mt-2 inline-flex items-center gap-1 group-hover:text-primary transition-colors duration-200">
                                        "View Availability" 
                                    </p>
                                </div>
                            </a>
                        }).collect_view()}
                    </div>
                </section>
            </div>
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
