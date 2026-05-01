use leptos::*;
use crate::components::widget_registry::WidgetInstance;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct DesignConfig {
    pub heading_font: String,
    pub body_font: String,
    pub meta_font: String,
    pub border_radius_base: String,
    pub container_strategy: String,
    pub background_pattern: String,
    pub elevation_strategy: String,
    pub button_padding: String,
    pub nav_layout: String,
    /// When true, applies the Kami academic paper design system to this tenant.
    /// Set via migration for buildwithruud; defaults to false for all other tenants.
    #[serde(default)]
    pub kami_mode: bool,
}

impl Default for DesignConfig {
    fn default() -> Self {
        Self {
            heading_font: "font-sans".into(),
            body_font: "font-sans".into(),
            meta_font: "font-mono".into(),
            border_radius_base: "rounded-none".into(),
            container_strategy: "centered-standard".into(),
            background_pattern: "none".into(),
            elevation_strategy: "flat-ghost".into(),
            button_padding: "px-6 py-2".into(),
            nav_layout: "solid-full".into(),
            kami_mode: false,
        }
    }
}

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
    pub theme_primary_color: Option<String>,
    pub design_config: Option<DesignConfig>,
    /// Tenant-configured widget instances. Parsed from app_instances.settings.widgets[].
    /// Defaults to empty — tenants with no widgets configured get no widget rendering.
    #[serde(default)]
    pub widgets: Vec<WidgetInstance>,
}

impl Default for SiteSettings {
    fn default() -> Self {
        Self {
            current_focus: "Anchor CMS Platform".into(),
            status: "Configuring".into(),
            hero_quote: "Your site is live. Add pages and content from the platform admin.".into(),
            hero_subtitle: "ANCHOR // MULTI-TENANT CMS PLATFORM — POWERED BY ATLAS.".into(),
            site_title: "ANCHOR PLATFORM".into(),
            lc_title: "Coming Soon".into(),
            lc_desc: "This site is being set up. Check back soon.".into(),
            lc_label: "Email".into(),
            lc_placeholder: "you@example.com".into(),
            lc_btn: "Notify Me".into(),
            lc_footer: "".into(),
            lc_endpoint: "/api/contact".into(),
            status_color: "#6366f1".into(),
            webhook_url: "".into(),
            admin_email: "".into(),
            google_analytics_id: "".into(),
            booking_url: "".into(),
            terms_html: "".into(),
            privacy_html: "".into(),
            github_url: "".into(),
            x_url: "".into(),
            linkedin_url: "".into(),
            b2b_enabled: false,
            meta_title: "Anchor — Powered by Atlas Platform".into(),
            meta_description: "A dynamic, multi-tenant CMS site built on the Atlas Platform.".into(),
            og_image: "".into(),
            theme_primary_color: None,
            design_config: None,
            widgets: vec![],
        }
    }
}

#[server(GetSiteSettings, "/api")]
pub async fn get_site_settings() -> Result<SiteSettings, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;

    let mut settings = SiteSettings::default();

    if let Some(tenant_id) = tenant.0 {
        // Primary: read flat KV settings from tenant_setting table directly
        match sqlx::query("SELECT key, value FROM tenant_setting WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_all(&state.pool)
            .await
        {
            Ok(rows) => {
                tracing::info!("[get_site_settings] tenant={} rows={}", tenant_id, rows.len());
                for row in rows {
                    let key: String = row.get("key");
                    let val: String = row.get("value");
                    match key.as_str() {
                        "current_focus" => settings.current_focus = val,
                        "status" => settings.status = val,
                        "hero_quote" => settings.hero_quote = val,
                        "hero_subtitle" => settings.hero_subtitle = val,
                        "site_title" => settings.site_title = val,
                        "lead_capture_title" => settings.lc_title = val,
                        "lead_capture_desc" => settings.lc_desc = val,
                        "lead_capture_label" => settings.lc_label = val,
                        "lead_capture_placeholder" => settings.lc_placeholder = val,
                        "lead_capture_btn" => settings.lc_btn = val,
                        "lead_capture_footer" => settings.lc_footer = val,
                        "lead_capture_endpoint" => settings.lc_endpoint = val,
                        "status_color" => settings.status_color = val,
                        "webhook_url" => settings.webhook_url = val,
                        "admin_email" => settings.admin_email = val,
                        "google_analytics_id" => settings.google_analytics_id = val,
                        "booking_url" => settings.booking_url = val,
                        "terms_html" => settings.terms_html = val,
                        "privacy_html" => settings.privacy_html = val,
                        "github_url" => settings.github_url = val,
                        "x_url" => settings.x_url = val,
                        "linkedin_url" => settings.linkedin_url = val,
                        "b2b_enabled" => settings.b2b_enabled = val == "true",
                        "meta_title" => settings.meta_title = val,
                        "meta_description" => settings.meta_description = val,
                        "og_image" => settings.og_image = val,
                        _ => {}
                    }
                }
            }
            Err(e) => {
                // Log the error explicitly so we can diagnose UAT failures.
                // Fall through with defaults — do not propagate as a hard error.
                tracing::error!("[get_site_settings] DB error querying tenant_setting: {:?}", e);
            }
        }

        // Secondary: read app_instances.settings via the backend API.
        // This supplements design_config, theme_primary_color, and widgets which are not
        // stored in tenant_setting (they live in the JSONB blob on app_instances).
        // The backend /api/app-instances/{tenant_id}/{app_type} merges both sources.
        let headers = extract::<axum::http::HeaderMap>().await.unwrap_or_default();
        let host = headers.get(axum::http::header::HOST).and_then(|h| h.to_str().ok()).map(|s| s.to_string());
        let endpoint = format!("/api/app-instances/{}/anchor", tenant_id);

        match crate::atlas_client::fetch_atlas_data::<serde_json::Value>(&endpoint, Some(tenant_id), host).await {
            Ok(inst_json) => {
                // The backend response is the AppInstance model with settings merged in.
                // Fields: { id, tenant_id, app_type, settings: { ...merged JSONB + tenant_setting KV... } }
                let s = inst_json.get("settings").cloned().unwrap_or_else(|| serde_json::json!({}));

                // design_config — only set if not already populated by tenant_setting
                if settings.design_config.is_none() {
                    if let Some(dc) = s.get("design_config") {
                        settings.design_config = serde_json::from_value(dc.clone()).ok();
                    }
                }
                // theme_primary_color
                if settings.theme_primary_color.is_none() {
                    if let Some(tc) = s.get("theme_primary_color").and_then(|v| v.as_str()) {
                        if !tc.is_empty() {
                            settings.theme_primary_color = Some(tc.to_string());
                        }
                    }
                }
                // widgets array — allows nav-registered widgets like BitcoinBlockClock
                if settings.widgets.is_empty() {
                    if let Some(ws) = s.get("widgets") {
                        if let Ok(widget_list) = serde_json::from_value::<Vec<crate::components::widget_registry::WidgetInstance>>(ws.clone()) {
                            settings.widgets = widget_list;
                        }
                    }
                }

                // Fallback: if tenant_setting gave us defaults for key fields, try the
                // merged JSONB. This handles tenants whose settings lived only in
                // app_instances.settings before the canonicalize migration ran.
                if settings.site_title == "ANCHOR PLATFORM" {
                    if let Some(v) = s.get("site_title").and_then(|v| v.as_str()) {
                        if !v.is_empty() { settings.site_title = v.to_string(); }
                    }
                    if let Some(v) = s.get("current_focus").and_then(|v| v.as_str()) {
                        if !v.is_empty() { settings.current_focus = v.to_string(); }
                    }
                    if let Some(v) = s.get("status").and_then(|v| v.as_str()) {
                        if !v.is_empty() { settings.status = v.to_string(); }
                    }
                    if let Some(v) = s.get("hero_quote").and_then(|v| v.as_str()) {
                        if !v.is_empty() { settings.hero_quote = v.to_string(); }
                    }
                    if let Some(v) = s.get("hero_subtitle").and_then(|v| v.as_str()) {
                        if !v.is_empty() { settings.hero_subtitle = v.to_string(); }
                    }
                    // LC fields
                    if let Some(v) = s.get("lead_capture_title").and_then(|v| v.as_str()) {
                        if !v.is_empty() { settings.lc_title = v.to_string(); }
                    }
                    if let Some(v) = s.get("lead_capture_btn").and_then(|v| v.as_str()) {
                        if !v.is_empty() { settings.lc_btn = v.to_string(); }
                    }
                    if let Some(v) = s.get("lead_capture_desc").and_then(|v| v.as_str()) {
                        if !v.is_empty() { settings.lc_desc = v.to_string(); }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("[get_site_settings] Backend API unavailable ({}), using tenant_setting data only", e);
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

    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'current_focus', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(current_focus)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'status', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(status)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'hero_quote', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(hero_quote)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'hero_subtitle', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(hero_subtitle)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'site_title', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(site_title)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'lead_capture_title', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(lc_title)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'lead_capture_desc', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(lc_desc)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'lead_capture_label', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(lc_label)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'lead_capture_placeholder', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(lc_placeholder)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'lead_capture_btn', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(lc_btn)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'lead_capture_footer', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(lc_footer)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'lead_capture_endpoint', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(lc_endpoint)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'status_color', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(status_color)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'webhook_url', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(webhook_url)
        .bind(tid)
        .execute(&state.pool)
        .await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'admin_email', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()")
        .bind(admin_email)
        .bind(tid)
        .execute(&state.pool)
        .await?;
        
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'google_analytics_id', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()").bind(google_analytics_id).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'booking_url', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()").bind(booking_url).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'terms_html', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()").bind(terms_html).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'privacy_html', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()").bind(privacy_html).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'github_url', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()").bind(github_url).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'x_url', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()").bind(x_url).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'linkedin_url', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()").bind(linkedin_url).bind(tid).execute(&state.pool).await?;

    let b2b_str = if b2b_enabled { "true" } else { "false" };
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'b2b_enabled', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()").bind(b2b_str).bind(tid).execute(&state.pool).await?;

    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'meta_title', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()").bind(meta_title).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'meta_description', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()").bind(meta_description).bind(tid).execute(&state.pool).await?;
    sqlx::query("INSERT INTO tenant_setting (id, tenant_id, key, value, updated_at, created_at) VALUES (gen_random_uuid(), $2, 'og_image', $1, NOW(), NOW()) ON CONFLICT (tenant_id, key) DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()").bind(og_image).bind(tid).execute(&state.pool).await?;

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
