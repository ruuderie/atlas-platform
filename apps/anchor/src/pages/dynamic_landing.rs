use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LandingPageRecord {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub hero_title: String,
    pub hero_subtitle: String,
    pub lead_capture_title: String,
    pub lead_capture_desc: String,
    pub lead_capture_btn: String,
    pub options_json: String,
    pub dynamic_blocks_json: String,
}

use crate::components::blocks::hero::{HeroBlock, HeroBlockData};
use crate::components::blocks::grid::{GridBlock, GridBlockData};
use crate::components::blocks::rich_text::{RichTextBlock, RichTextData};
use crate::components::blocks::callout::{CalloutBlock, CalloutBlockData};
use crate::components::blocks::form_builder::{FormBuilderBlock, FormBuilderData};

// DynamicBlock represents one block entry in a blocks_payload array.
// JSON storage format: {"BlockTypeName": { ...block fields... }}
// We use a custom Deserialize impl because #[serde(untagged)] with all-defaulted
// fields causes greedy first-variant matching — "Hero" would eat Grid and Callout objects.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum DynamicBlock {
    Hero(HeroBlockData),
    Grid(GridBlockData),
    Callout(CalloutBlockData),
    RichText(RichTextData),
    FormBuilder(FormBuilderData),
}

impl<'de> serde::Deserialize<'de> for DynamicBlock {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Deserialize the outer object as a raw map so we can read the key name first
        let map = serde_json::Map::deserialize(deserializer)
            .map_err(serde::de::Error::custom)?;

        if map.len() != 1 {
            return Err(serde::de::Error::custom(
                format!("DynamicBlock must have exactly one key, got {}", map.len())
            ));
        }

        let (key, value) = map.into_iter().next().unwrap();

        match key.as_str() {
            "Hero" => {
                let data = serde_json::from_value::<HeroBlockData>(value)
                    .map_err(serde::de::Error::custom)?;
                Ok(DynamicBlock::Hero(data))
            }
            "Grid" => {
                let data = serde_json::from_value::<GridBlockData>(value)
                    .map_err(serde::de::Error::custom)?;
                Ok(DynamicBlock::Grid(data))
            }
            "Callout" => {
                let data = serde_json::from_value::<CalloutBlockData>(value)
                    .map_err(serde::de::Error::custom)?;
                Ok(DynamicBlock::Callout(data))
            }
            "RichText" => {
                let data = serde_json::from_value::<RichTextData>(value)
                    .map_err(serde::de::Error::custom)?;
                Ok(DynamicBlock::RichText(data))
            }
            "FormBuilder" => {
                let data = serde_json::from_value::<FormBuilderData>(value)
                    .map_err(serde::de::Error::custom)?;
                Ok(DynamicBlock::FormBuilder(data))
            }
            unknown => Err(serde::de::Error::custom(
                format!("Unknown block type: '{}'", unknown)
            )),
        }
    }
}


#[server(GetLandingPage, "/api")]
pub async fn get_landing_page(slug: String) -> Result<Option<LandingPageRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::atlas_client::fetch_atlas_data;

    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let headers = extract::<axum::http::HeaderMap>().await.unwrap_or_default();
    let host = headers.get(axum::http::header::HOST).and_then(|h| h.to_str().ok()).map(|s| s.to_string());

    if let Some(tenant_id) = tenant.0 {
        let endpoint = format!("/api/public/pages/{}/{}", tenant_id, slug);
        
        #[derive(serde::Deserialize)]
        struct AppPageResp {
            title: String,
            description: String,
            hero_payload: Option<serde_json::Value>,
            blocks_payload: Option<serde_json::Value>,
        }
        
        if let Ok(page) = fetch_atlas_data::<AppPageResp>(&endpoint, Some(tenant_id), host).await {
            let hero = page.hero_payload.unwrap_or(serde_json::json!({}));
            // blocks_payload IS the blocks array directly (e.g. [{"Hero":{...}}, {"Grid":{...}}])
            let blocks_payload = page.blocks_payload.unwrap_or(serde_json::json!([]));
            // For CMS pages using the block engine, the full array is passed as dynamic_blocks_json.
            // hero_title/subtitle fall back to the legacy hero_payload fields for old-style pages.
            let dynamic_blocks_json = serde_json::to_string(&blocks_payload).unwrap_or_else(|_| "[]".to_string());
            
            return Ok(Some(LandingPageRecord {
                id: 0,
                slug,
                title: page.title,
                description: page.description,
                hero_title: hero["hero_title"].as_str().unwrap_or_default().to_string(),
                hero_subtitle: hero["hero_subtitle"].as_str().unwrap_or_default().to_string(),
                lead_capture_title: String::new(),
                lead_capture_desc: String::new(),
                lead_capture_btn: String::new(),
                options_json: "{}".to_string(),
                dynamic_blocks_json,
            }));
        }
    }

    Ok(None)
}

#[server(GetAllLandingPages, "/api")]
pub async fn get_all_landing_pages() -> Result<Vec<LandingPageRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::atlas_client::fetch_atlas_data;

    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let headers = extract::<axum::http::HeaderMap>().await.unwrap_or_default();
    let host = headers.get(axum::http::header::HOST).and_then(|h| h.to_str().ok()).map(|s| s.to_string());

    if let Some(tenant_id) = tenant.0 {
        let endpoint = format!("/api/public/pages/{}", tenant_id);
        
        #[derive(serde::Deserialize)]
        struct AppPageResp {
            slug: String,
            title: String,
            description: String,
            hero_payload: Option<serde_json::Value>,
            blocks_payload: Option<serde_json::Value>,
        }
        
        if let Ok(pages) = fetch_atlas_data::<Vec<AppPageResp>>(&endpoint, Some(tenant_id), host).await {
            let mapped = pages.into_iter().map(|page| {
                let hero = page.hero_payload.unwrap_or(serde_json::json!({}));
                let blocks = page.blocks_payload.unwrap_or(serde_json::json!({}));
                
                LandingPageRecord {
                    id: 0,
                    slug: page.slug,
                    title: page.title,
                    description: page.description,
                    hero_title: hero["hero_title"].as_str().unwrap_or_default().to_string(),
                    hero_subtitle: hero["hero_subtitle"].as_str().unwrap_or_default().to_string(),
                    lead_capture_title: blocks["lead_capture_title"].as_str().unwrap_or_default().to_string(),
                    lead_capture_desc: blocks["lead_capture_desc"].as_str().unwrap_or_default().to_string(),
                    lead_capture_btn: blocks["lead_capture_btn"].as_str().unwrap_or("Submit").to_string(),
                    options_json: blocks["options_json"].as_str().unwrap_or("{}").to_string(),
                    dynamic_blocks_json: blocks["dynamic_blocks"].to_string(),
                }
            }).collect();
            return Ok(mapped);
        }
    }

    Ok(vec![])
}

#[server(AddLandingPage, "/api")]
pub async fn add_landing_page(
    slug: String,
    title: String,
    description: String,
    hero_title: String,
    hero_subtitle: String,
    lead_capture_title: String,
    lead_capture_desc: String,
    lead_capture_btn: String,
    options_json: String,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    sqlx::query("INSERT INTO landing_pages (slug, title, description, hero_title, hero_subtitle, lead_capture_title, lead_capture_desc, lead_capture_btn, options_json) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
        .bind(slug).bind(title).bind(description).bind(hero_title).bind(hero_subtitle).bind(lead_capture_title).bind(lead_capture_desc).bind(lead_capture_btn).bind(options_json)
        .execute(&state.pool).await?;
    Ok(())
}

#[server(UpdateLandingPage, "/api")]
pub async fn update_landing_page(
    id: i32,
    slug: String,
    title: String,
    description: String,
    hero_title: String,
    hero_subtitle: String,
    lead_capture_title: String,
    lead_capture_desc: String,
    lead_capture_btn: String,
    options_json: String,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    sqlx::query("UPDATE landing_pages SET slug = $1, title = $2, description = $3, hero_title = $4, hero_subtitle = $5, lead_capture_title = $6, lead_capture_desc = $7, lead_capture_btn = $8, options_json = $9 WHERE id = $10")
        .bind(slug).bind(title).bind(description).bind(hero_title).bind(hero_subtitle).bind(lead_capture_title).bind(lead_capture_desc).bind(lead_capture_btn).bind(options_json).bind(id)
        .execute(&state.pool).await?;
    Ok(())
}

#[server(DeleteLandingPage, "/api")]
pub async fn delete_landing_page(id: i32) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    sqlx::query("DELETE FROM landing_pages WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?;
    Ok(())
}

#[server(HandleDynamicLead, "/api")]
pub async fn handle_dynamic_lead(
    slug: String,
    email: String,
    options: Vec<String>,
) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let prefs_json = serde_json::to_value(&options).unwrap_or(serde_json::json!([]));
    let _ = sqlx::query("INSERT INTO mailing_list (email, list_type, preferences) VALUES ($1, $2, $3) ON CONFLICT (email) DO UPDATE SET preferences = $3")
        .bind(&email)
        .bind(&slug)
        .bind(&prefs_json)
        .execute(&state.pool)
        .await;
    Ok(())
}

#[component]
pub fn DynamicLanding() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").cloned().unwrap_or_default());

    let (email, set_email) = create_signal(String::new());
    let (selected_options, set_selected_options) =
        create_signal(std::collections::HashSet::<String>::new());
    let (submitted, set_submitted) = create_signal(false);

    let submit_action = create_action(move |_: &()| {
        let e = email.get_untracked();
        let s = slug();
        let opts: Vec<String> = selected_options.get_untracked().into_iter().collect();
        async move {
            let _ = handle_dynamic_lead(s, e, opts).await;
            set_submitted.set(true);
        }
    });

    let page_res = create_resource(move || slug(), |s| get_landing_page(s));

    view! {
        <main class="pt-32 pb-24 px-4 md:px-[8.5rem]">
            <Suspense fallback=move || view! { <div class="text-center pt-24 jetbrains text-outline">"LOADING..."</div> }>
                {move || match page_res.get() {
                    Some(Ok(Some(page))) => {
                        let options_map: std::collections::HashMap<String, String> = serde_json::from_str(&page.options_json).unwrap_or_default();
                        let options_stored = store_value(options_map);
                        let has_options = options_stored.with_value(|v| !v.is_empty());

                        view! {
                            <section class="max-w-4xl mx-auto items-start">
                                <div class="inline-block bg-surface-container-high px-3 py-1 jetbrains text-[0.625rem] font-medium tracking-widest text-on-surface-variant mb-8 uppercase">
                                    {&page.title}
                                </div>
                                <h1 class="text-5xl md:text-[5rem] leading-[0.9] font-extrabold tracking-[-0.04em] text-primary mb-8 uppercase" inner_html=page.hero_title.clone()>
                                </h1>
                                <p class="text-xl md:text-2xl font-medium tracking-tight text-on-surface-variant leading-relaxed mb-8">
                                    {&page.hero_subtitle}
                                </p>

                                <div class="w-full">
                                    {move || {
                                        let parsed_blocks: Vec<DynamicBlock> = serde_json::from_str(&page.dynamic_blocks_json).unwrap_or_default();
                                        parsed_blocks.into_iter().map(|block| match block {
                                            DynamicBlock::Hero(data) => view! { <HeroBlock data=data /> }.into_view(),
                                            DynamicBlock::Grid(data) => view! { <GridBlock data=data /> }.into_view(),
                                            DynamicBlock::RichText(data) => view! { <RichTextBlock data=data /> }.into_view(),
                                            DynamicBlock::Callout(data) => view! { <CalloutBlock data=data /> }.into_view(),
                                            DynamicBlock::FormBuilder(data) => view! { <FormBuilderBlock data=data /> }.into_view(),
                                        }).collect_view()
                                    }}
                                </div>

                                {if !page.lead_capture_title.is_empty() {
                                    view!{ <div class="bg-surface-container-low p-8 border-l-4 border-primary my-12">
                                        {if submitted.get() {
                                        view! {
                                            <div class="text-center space-y-6 py-8">
                                                <span class="material-symbols-outlined text-secondary text-5xl">"check_circle"</span>
                                                <h2 class="text-3xl font-extrabold tracking-tight text-primary">"CONNECTION ESTABLISHED"</h2>
                                                <p class="text-on-surface-variant font-medium">"Your selections have been securely transmitted."</p>
                                            </div>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <div class="space-y-8">
                                                <div class="space-y-2">
                                                    <h3 class="text-2xl font-bold tracking-tight text-on-surface">
                                                        {&page.lead_capture_title}
                                                    </h3>
                                                    <p class="text-on-surface-variant">
                                                        {&page.lead_capture_desc}
                                                    </p>
                                                </div>
                                                <div class="space-y-4 w-full bg-transparent border-0 outline-none">
                                                    <div class="relative w-full group">
                                                        <input type="email" prop:value=email on:input=move |ev| set_email.set(event_target_value(&ev))
                                                            placeholder="Email Address"
                                                            class="w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-4 jetbrains text-lg text-on-surface placeholder:text-outline-variant/50 transition-all rounded-none" />
                                                    </div>

                                                    <Show when=move || has_options>
                                                        <div class="space-y-4 text-left border border-outline-variant/30 p-6 bg-surface-container-lowest/50 mt-4">
                                                            {move || options_stored.with_value(|map| map.clone().into_iter().map(|(key, label)| {
                                                                view! {
                                                                <label class="flex items-center space-x-3 cursor-pointer group">
                                                                    <input type="checkbox"
                                                                        class="w-5 h-5 bg-transparent border-2 border-outline-variant text-primary focus:ring-primary focus:ring-offset-surface-container-low"
                                                                        on:change=move |ev| {
                                                                            let k = key.clone();
                                                                            if event_target_checked(&ev) {
                                                                                set_selected_options.update(|set| { set.insert(k); });
                                                                            } else {
                                                                                set_selected_options.update(|set| { set.remove(&k); });
                                                                            }
                                                                        }
                                                                    />
                                                                    <span class="jetbrains text-sm text-on-surface group-hover:text-primary transition-colors">{label}</span>
                                                                </label>
                                                                }
                                                            }).collect_view())}
                                                        </div>
                                                    </Show>

                                                    <div class="pt-4">
                                                        <button on:click=move |_| submit_action.dispatch(()) class="w-full bg-primary text-on-primary py-6 jetbrains font-bold text-sm tracking-[0.2em] uppercase hover:bg-primary-container transition-colors rounded-none outline-none border-none shadow-none">
                                                            {&page.lead_capture_btn}
                                                        </button>
                                                    </div>
                                                </div>
                                            </div>
                                        }.into_view()
                                    }}
                                    </div> }.into_view()
                                } else { view!{}.into_view() }}
                            </section>
                        }.into_view()
                    },
                    Some(Ok(None)) | Some(Err(_)) => view! { <div class="text-center pt-24 jetbrains text-error">"PAGE NOT FOUND"</div> }.into_view(),
                    None => view! { <div/> }.into_view()
                }}
            </Suspense>
        </main>
    }
}

/// Root `/` component — renders the CMS `home` page for the current tenant.
/// Falls back to a clean Anchor platform placeholder when no page is configured yet.
#[component]
pub fn DynamicHomeLanding() -> impl IntoView {
    let page_res = create_resource(|| "home".to_string(), |s| get_landing_page(s));

    // For block-based pages the Hero block itself is the first visual element (full-bleed).
    // We don't want the outer pt-32 padding eating into it. Instead, render blocks
    // without padding — each block manages its own layout.
    // For legacy pages (hero_title non-empty) or the fallback placeholder we keep the padded wrapper.
    view! {
        <Suspense fallback=move || view! { <div class="pt-32 pb-24 px-4 text-center jetbrains text-outline">"LOADING..."</div> }>
            {move || match page_res.get() {
                // CMS block-based page: render blocks full-bleed, no outer padding
                Some(Ok(Some(page))) => {
                    let parsed_blocks: Vec<DynamicBlock> = serde_json::from_str(&page.dynamic_blocks_json).unwrap_or_default();
                    let has_blocks = !parsed_blocks.is_empty();

                    if has_blocks {
                        // Block-based page — let each block manage its own layout/padding
                        view! {
                            <main>
                                {parsed_blocks.into_iter().map(|block| match block {
                                    DynamicBlock::Hero(data) => view! { <HeroBlock data=data /> }.into_view(),
                                    DynamicBlock::Grid(data) => view! { <GridBlock data=data /> }.into_view(),
                                    DynamicBlock::RichText(data) => view! { <RichTextBlock data=data /> }.into_view(),
                                    DynamicBlock::Callout(data) => view! { <CalloutBlock data=data /> }.into_view(),
                                    DynamicBlock::FormBuilder(data) => view! { <FormBuilderBlock data=data /> }.into_view(),
                                }).collect_view()}
                            </main>
                        }.into_view()
                    } else {
                        // Legacy page with hero_title/hero_subtitle in hero_payload
                        view! {
                            <main class="pt-32 pb-24 px-4 md:px-[8.5rem]">
                                <section class="max-w-4xl mx-auto">
                                    <h1 class="text-5xl md:text-[5rem] leading-[0.9] font-extrabold tracking-[-0.04em] text-primary mb-8 uppercase"
                                        inner_html=page.hero_title.clone()>
                                    </h1>
                                    <p class="text-xl md:text-2xl font-medium tracking-tight text-on-surface-variant leading-relaxed mb-12">
                                        {&page.hero_subtitle}
                                    </p>
                                </section>
                            </main>
                        }.into_view()
                    }
                },
                // No CMS page configured — show neutral Anchor platform placeholder
                Some(Ok(None)) | Some(Err(_)) => view! {
                    <main class="pt-32 pb-24 px-4 md:px-[8.5rem]">
                        <section class="max-w-3xl mx-auto flex flex-col items-start min-h-[60vh] pt-12">
                            <div class="inline-block bg-surface-container-high px-3 py-1 jetbrains text-[0.625rem] font-medium tracking-widest text-on-surface-variant mb-8 uppercase">
                                "ANCHOR PLATFORM"
                            </div>
                            <h1 class="text-5xl md:text-[5.5rem] leading-[0.9] font-extrabold tracking-[-0.04em] text-primary mb-8 uppercase">
                                "Site is being configured"
                            </h1>
                            <p class="text-xl font-medium tracking-tight text-on-surface-variant leading-relaxed mb-12 max-w-xl">
                                "This Anchor instance is live. Add pages and content from the platform admin to get started."
                            </p>
                            <div class="flex items-center space-x-3">
                                <span class="w-2 h-2 rounded-full bg-[#6366f1]" style="box-shadow: 0 0 8px #6366f1;"></span>
                                <span class="jetbrains text-xs uppercase text-outline tracking-widest">"Infrastructure Online"</span>
                            </div>
                        </section>
                    </main>
                }.into_view(),
                None => view! { <div/> }.into_view(),
            }}
        </Suspense>
    }
}
