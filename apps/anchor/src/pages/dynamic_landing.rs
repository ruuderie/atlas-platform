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
}

#[server(GetLandingPage, "/api")]
pub async fn get_landing_page(slug: String) -> Result<Option<LandingPageRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let row_opt = sqlx::query("SELECT * FROM landing_pages WHERE slug = $1")
        .bind(&slug)
        .fetch_optional(&state.pool)
        .await?;

    if let Some(row) = row_opt {
        Ok(Some(LandingPageRecord {
            id: row.get("id"),
            slug: row.get("slug"),
            title: row.get("title"),
            description: row.get("description"),
            hero_title: row.get("hero_title"),
            hero_subtitle: row.get("hero_subtitle"),
            lead_capture_title: row.get("lead_capture_title"),
            lead_capture_desc: row.get("lead_capture_desc"),
            lead_capture_btn: row.get("lead_capture_btn"),
            options_json: row.get("options_json"),
        }))
    } else {
        Ok(None)
    }
}

#[server(GetAllLandingPages, "/api")]
pub async fn get_all_landing_pages() -> Result<Vec<LandingPageRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;

    let rows = sqlx::query("SELECT * FROM landing_pages ORDER BY id DESC")
        .fetch_all(&state.pool)
        .await?;

    let pages = rows
        .into_iter()
        .map(|row| LandingPageRecord {
            id: row.get("id"),
            slug: row.get("slug"),
            title: row.get("title"),
            description: row.get("description"),
            hero_title: row.get("hero_title"),
            hero_subtitle: row.get("hero_subtitle"),
            lead_capture_title: row.get("lead_capture_title"),
            lead_capture_desc: row.get("lead_capture_desc"),
            lead_capture_btn: row.get("lead_capture_btn"),
            options_json: row.get("options_json"),
        })
        .collect();

    Ok(pages)
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

                                <div class="bg-surface-container-low p-8 border-l-4 border-primary my-12">
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
                                </div>
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
