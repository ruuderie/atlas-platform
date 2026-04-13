use leptos::*;

use crate::pages::services::HighlightsGallery;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SiteSettings {
    pub current_focus: String,
    pub status: String,
    pub hero_quote: String,
    pub hero_subtitle: String,
    pub site_title: String,
    pub lc_title: String,
    pub lc_desc: String,
    pub lc_label: String,
    pub lc_placeholder: String,
    pub lc_btn: String,
    pub lc_footer: String,
    pub lc_endpoint: String,
    pub status_color: String,
    pub webhook_url: String,
    pub admin_email: String,
    pub google_analytics_id: String,
    pub booking_url: String,
    pub terms_html: String,
    pub privacy_html: String,
    pub github_url: String,
    pub x_url: String,
    pub linkedin_url: String,
    pub b2b_enabled: bool,
    pub meta_title: String,
    pub meta_description: String,
    pub og_image: String,
}

impl Default for SiteSettings {
    fn default() -> Self {
        Self {
            current_focus: "Cloud CMS & Form Generation Engine".into(),
            status: "Systems Online".into(),
            hero_quote: "Build dynamic commercial lending portals with integrated webform engines.".into(),
            hero_subtitle: "ANCHOR // THE ULTIMATE DYNAMIC PLATFORM FOR FINANCIAL INSTITUTIONS AND LENDERS.".into(),
            site_title: "ANCHOR PLATFORM".into(),
            lc_title: "Join Early Access".into(),
            lc_desc: "Get notified when the block builder goes live.".into(),
            lc_label: "Developer Email".into(),
            lc_placeholder: "dev@example.com".into(),
            lc_btn: "Request Access".into(),
            lc_footer: "* We will provision your tenant sandbox within 24 hours.".into(),
            lc_endpoint: "/api/contact".into(),
            status_color: "#3b82f6".into(),
            webhook_url: "".into(),
            admin_email: "".into(),
            google_analytics_id: "".into(),
            booking_url: "".into(),
            terms_html: "".into(),
            privacy_html: "".into(),
            github_url: "".into(),
            x_url: "".into(),
            linkedin_url: "".into(),
            b2b_enabled: true,
            meta_title: "Anchor Platform - Dynamic CMS".into(),
            meta_description: "A multi-tenant, zero-config CMS architecture built strictly for the Atlas Platform.".into(),
            og_image: "".into(),
        }
    }
}

#[server(GetSiteSettings, "/api")]
pub async fn get_site_settings() -> Result<SiteSettings, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use crate::atlas_client::fetch_atlas_data;
    use serde_json::Value;

    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let headers = extract::<axum::http::HeaderMap>().await.unwrap_or_default();
    let host = headers.get(axum::http::header::HOST).and_then(|h| h.to_str().ok()).map(|s| s.to_string());

    let mut settings = SiteSettings::default();

    if let Some(tenant_id) = tenant.0 {
        let endpoint = format!("/api/app-instances/{}/anchor", tenant_id);
        
        #[derive(serde::Deserialize)]
        struct AppInstanceResponse {
            settings: Option<Value>,
        }
        
        if let Ok(res) = fetch_atlas_data::<AppInstanceResponse>(&endpoint, Some(tenant_id), host).await {
            if let Some(json_settings) = res.settings {
                if let Ok(parsed) = serde_json::from_value::<SiteSettings>(json_settings) {
                    settings = parsed;
                }
            }
        }
    }

    Ok(settings)
}

#[server(UpdateSiteSettings, "/api")]
pub async fn update_site_settings(
    current_focus: String,
    status: String,
    hero_quote: String,
    hero_subtitle: String,
    site_title: String,
    lc_title: String,
    lc_desc: String,
    lc_label: String,
    lc_placeholder: String,
    lc_btn: String,
    lc_footer: String,
    lc_endpoint: String,
    status_color: String,
    webhook_url: String,
    admin_email: String,
    google_analytics_id: String,
    booking_url: String,
    terms_html: String,
    privacy_html: String,
    github_url: String,
    x_url: String,
    linkedin_url: String,
    b2b_enabled: bool,
    meta_title: String,
    meta_description: String,
    og_image: String,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tid = tenant.0;

    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'current_focus' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(current_focus)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'status' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(status)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'hero_quote' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(hero_quote)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'hero_subtitle' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(hero_subtitle)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'site_title' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(site_title)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'lead_capture_title' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(lc_title)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'lead_capture_desc' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(lc_desc)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'lead_capture_label' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(lc_label)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'lead_capture_placeholder' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(lc_placeholder)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'lead_capture_btn' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(lc_btn)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'lead_capture_footer' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(lc_footer)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'lead_capture_endpoint' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(lc_endpoint)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'status_color' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(status_color)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'webhook_url' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(webhook_url)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("UPDATE site_settings SET value = $1 WHERE key = 'admin_email' AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(admin_email)
        .bind(tid)
        .execute(&state.pool)
        .await?;
        
    sqlx::query("INSERT INTO site_settings (tenant_id, key, value) VALUES ($2, 'google_analytics_id', $1) ON CONFLICT (tenant_id, key) DO UPDATE SET value = $1").bind(google_analytics_id).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO site_settings (tenant_id, key, value) VALUES ($2, 'booking_url', $1) ON CONFLICT (tenant_id, key) DO UPDATE SET value = $1").bind(booking_url).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO site_settings (tenant_id, key, value) VALUES ($2, 'terms_html', $1) ON CONFLICT (tenant_id, key) DO UPDATE SET value = $1").bind(terms_html).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO site_settings (tenant_id, key, value) VALUES ($2, 'privacy_html', $1) ON CONFLICT (tenant_id, key) DO UPDATE SET value = $1").bind(privacy_html).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO site_settings (tenant_id, key, value) VALUES ($2, 'github_url', $1) ON CONFLICT (tenant_id, key) DO UPDATE SET value = $1").bind(github_url).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO site_settings (tenant_id, key, value) VALUES ($2, 'x_url', $1) ON CONFLICT (tenant_id, key) DO UPDATE SET value = $1").bind(x_url).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO site_settings (tenant_id, key, value) VALUES ($2, 'linkedin_url', $1) ON CONFLICT (tenant_id, key) DO UPDATE SET value = $1").bind(linkedin_url).bind(tid).execute(&state.pool).await?;

    let b2b_str = if b2b_enabled { "true" } else { "false" };
    sqlx::query("INSERT INTO site_settings (tenant_id, key, value) VALUES ($2, 'b2b_enabled', $1) ON CONFLICT (tenant_id, key) DO UPDATE SET value = $1").bind(b2b_str).bind(tid).execute(&state.pool).await?;

    sqlx::query("INSERT INTO site_settings (tenant_id, key, value) VALUES ($2, 'meta_title', $1) ON CONFLICT (tenant_id, key) DO UPDATE SET value = $1").bind(meta_title).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO site_settings (tenant_id, key, value) VALUES ($2, 'meta_description', $1) ON CONFLICT (tenant_id, key) DO UPDATE SET value = $1").bind(meta_description).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO site_settings (tenant_id, key, value) VALUES ($2, 'og_image', $1) ON CONFLICT (tenant_id, key) DO UPDATE SET value = $1").bind(og_image).bind(tid).execute(&state.pool).await?;

    Ok(())
}

#[server(HandleLeadCapture, "/api")]
pub async fn handle_lead_capture(email: String, options: Vec<String>) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let settings = get_site_settings().await.unwrap_or_default();

    let prefs_json = serde_json::to_value(&options).unwrap_or(serde_json::json!([]));

    let _ = sqlx::query("INSERT INTO mailing_list (tenant_id, email, list_type, preferences) VALUES ($1, $2, $3, $4) ON CONFLICT (tenant_id, email) DO UPDATE SET preferences = $4")
        .bind(tenant.0)
        .bind(&email)
        .bind("general")
        .bind(&prefs_json)
        .execute(&state.pool)
        .await;

    if !settings.webhook_url.is_empty() {
        let payload = serde_json::json!({
            "email": &email,
            "options": &options
        });
        let client = reqwest::Client::new();
        match client
            .post(&settings.webhook_url)
            .json(&payload)
            .send()
            .await
        {
            Ok(res) if res.status().is_success() => {
                println!("Webhook successfully triggered for {}", email);
            }
            Ok(res) => {
                println!(
                    "Webhook returned non-success for {}: {}",
                    email,
                    res.status()
                );
            }
            Err(e) => {
                println!("Failed to trigger webhook for {}: {:?}", email, e);
            }
        }
    } else {
        println!(
            "NEW LEAD CAPTURE (NO WEBHOOK CONFIGURED): {} requested {:?}",
            email, options
        );
    }

    if !settings.admin_email.is_empty() {
        let subject = format!("New Lead Capture: {}", email);
        let body = format!("<h3>New lead captured!</h3><p><strong>Email:</strong> {}</p><p><strong>Options requested:</strong> {:?}</p>", email, options);
        let _ = crate::email::send_email(settings.admin_email.clone(), subject, body).await;
    }

    Ok(())
}

#[component]
pub fn Landing() -> impl IntoView {
    let settings_resource = create_resource(|| (), |_| get_site_settings());
    let stats_resource = create_resource(|| (), |_| crate::components::nav::get_bitcoin_stats());

    let (email, set_email) = create_signal(String::new());
    let (selected_options, set_selected_options) =
        create_signal(std::collections::HashSet::<String>::new());
    let (submitted, set_submitted) = create_signal(false);

    let submit_action = create_action(move |_: &()| {
        let e = email.get_untracked();
        let opts: Vec<String> = selected_options.get_untracked().into_iter().collect();
        async move {
            let _ = handle_lead_capture(e, opts).await;
            set_submitted.set(true);
        }
    });

    view! {
        <main class="pt-32 pb-24">
            // Hero Section
            <section class="w-full grid grid-cols-1 md:grid-cols-12 gap-12 min-h-[716px] items-start px-4 md:px-[8.5rem]">
                <div class="w-full md:col-span-12 lg:col-span-8 flex flex-col items-start">
                    <div class="inline-block bg-surface-container-high px-3 py-1 jetbrains text-[0.625rem] font-medium tracking-widest text-on-surface-variant mb-8 uppercase">
                        "WELCOME TO THE PLATFORM"
                    </div>
                    <h1 class="text-5xl sm:text-6xl md:text-[6rem] leading-[0.9] font-extrabold tracking-[-0.04em] text-primary mb-12 uppercase break-words">
                        <Suspense fallback=move || view! { <span>"..."</span> }>
                            {move || settings_resource.get().unwrap_or(Ok(SiteSettings::default())).unwrap_or(SiteSettings::default()).site_title}
                        </Suspense>
                    </h1>
                    <p class="text-lg sm:text-xl md:text-2xl font-medium tracking-tight text-on-surface-variant max-w-2xl leading-relaxed uppercase">
                        <Suspense fallback=move || view! { <span>"..."</span> }>
                            {move || settings_resource.get().unwrap_or(Ok(SiteSettings::default())).unwrap_or(SiteSettings::default()).hero_subtitle}
                        </Suspense>
                    </p>
                    <div class="mt-20 flex space-x-12 items-end">
                        <Suspense fallback=move || view! { <div class="jetbrains text-xs">"Loading..."</div> }>
                            {move || {
                                let settings = settings_resource.get().unwrap_or(Ok(SiteSettings::default())).unwrap_or(SiteSettings::default());
                                view! {
                                    <div class="flex space-x-12 items-end">
                                        <div class="flex flex-col">
                                            <span class="jetbrains text-[0.65rem] uppercase text-outline mb-2 uppercase">"Current focus"</span>
                                            <span class="text-sm font-bold text-on-surface">{settings.current_focus}</span>
                                        </div>
                                        <div class="flex flex-col">
                                            <span class="jetbrains text-[0.65rem] uppercase tracking-[0.2em] text-outline mb-3 inline-flex items-center">
                                                <span class="w-1.5 h-1.5 rounded-full mr-2" style=format!("background-color: {}; box-shadow: 0 0 8px {};", settings.status_color, settings.status_color)></span>
                                                "STATUS"
                                            </span>
                                            <span class="text-on-surface font-bold tracking-tight text-lg">
                                                {settings.status}
                                            </span>
                                        </div>
                                    </div>
                                }
                            }}
                        </Suspense>
                    </div>
                </div>

                <div class="col-span-12 lg:col-span-4 space-y-8 mt-12 lg:mt-0">
                    <div class="bg-surface-container-low p-8 border-l-4 border-secondary flex flex-col justify-between aspect-square lg:aspect-auto lg:min-h-[400px]">
                        <div>
                            <span class="material-symbols-outlined text-[#f7931a] text-4xl mb-6">"format_quote"</span>
                            <p class="text-lg italic font-medium text-on-surface leading-snug">
                                <Suspense fallback=move || view! { <span>"..."</span> }>
                                    {move || format!("\"{}\"", settings_resource.get().unwrap_or(Ok(SiteSettings::default())).unwrap_or(SiteSettings::default()).hero_quote)}
                                </Suspense>
                            </p>
                        </div>
                        <Suspense fallback=move || view! { <div class="mt-8">"Hydrating Network Stats..."</div> }>
                            {move || {
                                let stats = stats_resource.get()
                                    .unwrap_or_else(|| Ok(crate::components::nav::BitcoinStats { difficulty: 0.0, tx_count: 0, size: 0, weight: 0 }))
                                    .unwrap_or(crate::components::nav::BitcoinStats { difficulty: 0.0, tx_count: 0, size: 0, weight: 0 });
                                view! {
                                    <div class="mt-8 space-y-6">
                                        <div class="space-y-2">
                                            <div class="flex justify-between items-end">
                                                <span class="jetbrains text-[0.65rem] uppercase text-outline">"Latest Block Weight"</span>
                                                <span class="jetbrains text-[0.65rem] text-[#f7931a]">{format!("{:.2}%", (stats.weight as f64 / 4_000_000.0) * 100.0)}</span>
                                            </div>
                                            <div class="h-1 bg-surface-container-highest w-full overflow-hidden">
                                                <div class="h-full bg-[#f7931a]" style=format!("width: {:.2}%", (stats.weight as f64 / 4_000_000.0) * 100.0)></div>
                                            </div>
                                            <div class="jetbrains text-[0.55rem] text-outline-variant uppercase">{format!("{} / 4,000,000 WU", stats.weight)}</div>
                                        </div>
                                        <div class="space-y-2">
                                            <div class="flex justify-between items-end">
                                                <span class="jetbrains text-[0.65rem] uppercase text-outline">"Recent Mempool Transactions"</span>
                                                <span class="jetbrains text-[0.65rem] text-primary">{stats.tx_count.to_string()}</span>
                                            </div>
                                            <div class="h-1 bg-surface-container-highest w-full overflow-hidden">
                                                <div class="h-full bg-primary w-[33%]"></div>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }}
                        </Suspense>
                    </div>
                </div>
            </section>

            // Lead Capture Section
            <section class="mt-32 flex flex-col items-center justify-center py-32 bg-surface-container-low relative">
                <div class="absolute top-0 left-0 w-full h-px bg-outline-variant/30"></div>
                <div class="max-w-xl w-full space-y-12 px-6 shadow-none">
                    {move || if submitted.get() {
                        view! {
                            <div class="text-center space-y-6">
                                <span class="material-symbols-outlined text-secondary text-5xl">"check_circle"</span>
                                <h2 class="text-3xl font-extrabold tracking-tight text-primary">"REQUEST LOGGED"</h2>
                                <p class="text-on-surface-variant font-medium">"Your selections have been securely transmitted."</p>
                            </div>
                        }.into_view()
                    } else {
                        view! {
                            <div class="text-center space-y-4">
                                <h2 class="text-4xl font-extrabold tracking-tight text-primary">
                                    <Suspense fallback=move || view! { <span>"..."</span> }>
                                        {move || settings_resource.get().unwrap_or(Ok(SiteSettings::default())).unwrap_or(SiteSettings::default()).lc_title}
                                    </Suspense>
                                </h2>
                                <p class="text-on-surface-variant font-medium">
                                    <Suspense fallback=move || view! { <span>"..."</span> }>
                                        {move || settings_resource.get().unwrap_or(Ok(SiteSettings::default())).unwrap_or(SiteSettings::default()).lc_desc}
                                    </Suspense>
                                </p>
                            </div>
                            <div class="space-y-8 w-full bg-transparent border-0 outline-none">
                                <div class="relative w-full group">
                                    <label class="jetbrains text-[0.65rem] uppercase tracking-[0.1em] text-outline text-left block mb-2">
                                        <Suspense fallback=move || view! { <span>"..."</span> }>
                                            {move || settings_resource.get().unwrap_or(Ok(SiteSettings::default())).unwrap_or(SiteSettings::default()).lc_label}
                                        </Suspense>
                                    </label>
                                    <input type="email" prop:value=email on:input=move |ev| set_email.set(event_target_value(&ev)) placeholder="user@organization.domain" class="w-full bg-transparent border-none border-b-2 border-outline-variant focus:border-primary focus:ring-0 px-0 py-4 jetbrains text-lg text-on-surface placeholder:text-outline-variant/50 transition-all rounded-none" />
                                </div>
                                <Suspense fallback=move || view! { <div class="jetbrains text-xs">"Loading options..."</div> }>
                                    {move || {
                                        let options_res = create_resource(|| (), |_| get_lead_options());
                                        view! {
                                            <div class="space-y-4 text-left border border-outline-variant/30 p-6 bg-surface-container-lowest/50">
                                                <Transition fallback=move || view! { <div>"..."</div> }>
                                                {move || match options_res.get() {
                                                    Some(Ok(options)) => options.into_iter().map(|opt| {
                                                        let k = opt.value_key.clone();
                                                        view! {
                                                        <label class="flex items-center space-x-3 cursor-pointer group">
                                                            <input type="checkbox"
                                                                class="w-5 h-5 bg-transparent border-2 border-outline-variant text-primary focus:ring-primary focus:ring-offset-surface-container-low"
                                                                on:change=move |ev| {
                                                                    if event_target_checked(&ev) {
                                                                        set_selected_options.update(|set| { set.insert(k.clone()); });
                                                                    } else {
                                                                        set_selected_options.update(|set| { set.remove(&k); });
                                                                    }
                                                                }
                                                            />
                                                            <span class="jetbrains text-sm text-on-surface group-hover:text-primary transition-colors">{opt.label}</span>
                                                        </label>
                                                        }
                                                    }).collect_view(),
                                                    _ => view! { <div>"No options available."</div> }.into_view(),
                                                }}
                                                </Transition>
                                            </div>
                                        }
                                    }}
                                </Suspense>
                                <div class="space-y-4">
                                    <button on:click=move |_| submit_action.dispatch(()) class="w-full bg-secondary text-on-primary py-6 jetbrains font-bold text-sm tracking-[0.2em] uppercase hover:bg-on-secondary-fixed-variant transition-colors rounded-none outline-none border-none shadow-none">
                                        <Suspense fallback=move || view! { <span>"EXECUTE..."</span> }>
                                            {move || settings_resource.get().unwrap_or(Ok(SiteSettings::default())).unwrap_or(SiteSettings::default()).lc_btn}
                                        </Suspense>
                                    </button>
                                    <p class="jetbrains text-[0.625rem] text-outline uppercase tracking-widest text-center">
                                        <Suspense fallback=move || view! { <span>"*"</span> }>
                                            {move || settings_resource.get().unwrap_or(Ok(SiteSettings::default())).unwrap_or(SiteSettings::default()).lc_footer}
                                        </Suspense>
                                    </p>
                                </div>
                            </div>
                        }.into_view()
                    }}
                </div>
            </section>

            <div class="px-4 md:px-[8.5rem] mt-24">
                <HighlightsGallery />
            </div>
        </main>
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct LeadCaptureOption {
    pub id: i32,
    pub value_key: String,
    pub label: String,
    pub is_active: bool,
    pub display_order: i32,
}

#[server(GetLeadOptions, "/api")]
pub async fn get_lead_options() -> Result<Vec<LeadCaptureOption>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let rows = sqlx::query("SELECT id, value_key, label, is_active, display_order FROM lead_capture_options WHERE is_active = true AND tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC")
        .bind(tenant.0)
        .fetch_all(&state.pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| LeadCaptureOption {
            id: row.get("id"),
            value_key: row.get("value_key"),
            label: row.get("label"),
            is_active: row.get("is_active"),
            display_order: row.get("display_order"),
        })
        .collect())
}

#[server(GetAllLeadOptions, "/api")]
pub async fn get_all_lead_options() -> Result<Vec<LeadCaptureOption>, ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let rows = sqlx::query("SELECT id, value_key, label, is_active, display_order FROM lead_capture_options WHERE tenant_id IS NOT DISTINCT FROM $1 ORDER BY display_order ASC")
        .bind(tenant.0)
        .fetch_all(&state.pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| LeadCaptureOption {
            id: row.get("id"),
            value_key: row.get("value_key"),
            label: row.get("label"),
            is_active: row.get("is_active"),
            display_order: row.get("display_order"),
        })
        .collect())
}

#[server(UpsertLeadOption, "/api")]
pub async fn upsert_lead_option(
    id: Option<i32>,
    value_key: String,
    label: String,
    is_active: bool,
    display_order: i32,
) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    if let Some(option_id) = id {
        sqlx::query("UPDATE lead_capture_options SET value_key = $1, label = $2, is_active = $3, display_order = $4 WHERE id = $5 AND tenant_id IS NOT DISTINCT FROM $6")
            .bind(value_key)
            .bind(label)
            .bind(is_active)
            .bind(display_order)
            .bind(option_id)
            .bind(tenant.0)
            .execute(&state.pool)
            .await?;
    } else {
        sqlx::query("INSERT INTO lead_capture_options (tenant_id, value_key, label, is_active, display_order) VALUES ($1, $2, $3, $4, $5)")
            .bind(tenant.0)
            .bind(value_key)
            .bind(label)
            .bind(is_active)
            .bind(display_order)
            .execute(&state.pool)
            .await?;
    }
    Ok(())
}

#[server(DeleteLeadOption, "/api")]
pub async fn delete_lead_option(id: i32) -> Result<(), ServerFnError> {
    use crate::auth::check_session;
    use axum::Extension;
    use leptos_axum::extract;
    if !check_session().await.unwrap_or(false) {
        return Err(ServerFnError::ServerError("Unauthorized".into()));
    }
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    sqlx::query("DELETE FROM lead_capture_options WHERE id = $1 AND tenant_id IS NOT DISTINCT FROM $2")
        .bind(id)
        .bind(tenant.0)
        .execute(&state.pool)
        .await?;
    Ok(())
}
