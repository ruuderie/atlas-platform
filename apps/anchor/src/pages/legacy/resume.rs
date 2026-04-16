use leptos::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum JobType {
    #[default]
    DirectHire,
    Contract,
    CorpToCorp,
}

impl FromStr for JobType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Contract" => Ok(JobType::Contract),
            "CorpToCorp" => Ok(JobType::CorpToCorp),
            _ => Ok(JobType::DirectHire),
        }
    }
}

impl std::fmt::Display for JobType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobType::DirectHire => write!(f, "DirectHire"),
            JobType::Contract => write!(f, "Contract"),
            JobType::CorpToCorp => write!(f, "CorpToCorp"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct JobRecord {
    pub id: i32,
    pub date_range: String,
    pub role: String,
    pub company: String,
    pub bullets: Vec<String>,
    pub employment_type: JobType,
    pub parent_company: Option<String>,
    pub tags: Vec<String>,
    pub hide_date: bool,
}

#[server(GetJobs, "/api")]
pub async fn get_jobs() -> Result<Vec<JobRecord>, ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    use sqlx::Row;

    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let rows = sqlx::query("SELECT id, title, date_range, bullets, metadata FROM tenant_entries WHERE category = 'work' ORDER BY id DESC")
        .fetch_all(&state.pool)
        .await?;

    let jobs = rows
        .into_iter()
        .map(|row| {
            let meta: Option<serde_json::Value> = row.try_get("metadata").unwrap_or(None);
            let company = meta
                .as_ref()
                .and_then(|m| m.get("company"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let parent_company = meta
                .as_ref()
                .and_then(|m| m.get("parent_company"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let et_str = meta
                .as_ref()
                .and_then(|m| m.get("employment_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("DirectHire");
            let employment_type = JobType::from_str(&et_str).unwrap_or_default();

            let tags: Vec<String> = meta
                .as_ref()
                .and_then(|m| m.get("tags"))
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_else(|| {
                    meta.as_ref()
                        .and_then(|m| m.get("tags"))
                        .and_then(|v| v.as_str())
                        .map(|s| {
                            s.split(',')
                                .map(|x| x.trim().to_string())
                                .filter(|x| !x.is_empty())
                                .collect()
                        })
                        .unwrap_or_default()
                });

            let bullets_val: serde_json::Value =
                row.try_get("bullets").unwrap_or(serde_json::json!([]));
            let bullets: Vec<String> = serde_json::from_value(bullets_val).unwrap_or_default();
            let date_range_opt: Option<String> = row.try_get("date_range").unwrap_or(None);

            JobRecord {
                id: row.get("id"),
                date_range: date_range_opt.unwrap_or_default(),
                role: row.get("title"),
                company,
                bullets,
                employment_type,
                parent_company,
                tags,
                hide_date: meta
                    .as_ref()
                    .and_then(|m| m.get("hide_date"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
            }
        })
        .collect();

    Ok(jobs)
}

#[component]
pub fn Resume() -> impl IntoView {
    let download_pdf = create_action(|id: &i32| {
        let profile_id = *id;
        use crate::resume_engine::download_resume;
        async move {
            if let Ok(bytes) = download_resume(profile_id).await {
                use web_sys::js_sys::{Array, Uint8Array};
                use web_sys::{Blob, BlobPropertyBag, Url};

                let uint8_arr = Uint8Array::from(bytes.as_slice());
                let parts = Array::new();
                parts.push(&uint8_arr);

                let props = BlobPropertyBag::new();
                props.set_type("application/pdf");

                if let Ok(blob) = Blob::new_with_u8_array_sequence_and_options(&parts, &props) {
                    if let Ok(url) = Url::create_object_url_with_blob(&blob) {
                        let document = leptos::document();
                        if let Ok(a) = document.create_element("a") {
                            let _ = a.set_attribute("href", &url);
                            let _ = a.set_attribute("download", "ruuderie_cv.pdf");
                            use web_sys::wasm_bindgen::JsCast;
                            let html_a = a.unchecked_into::<web_sys::HtmlElement>();
                            html_a.click();
                        }
                    }
                }
            }
        }
    });

    let profiles_resource = create_resource(
        || (),
        |_| async move {
            crate::resume_engine::get_entry_collections()
                .await
                .unwrap_or_default()
        },
    );

    let (active_profile_id, set_active_profile_id) = create_signal(None::<i32>);
    let (show_modal, set_show_modal) = create_signal(false);
    let (lead_name, set_lead_name) = create_signal(String::new());
    let (lead_email, set_lead_email) = create_signal(String::new());

    // Automatically select the first PUBLIC profile on load
    create_effect(move |_| {
        if let Some(mut profiles) = profiles_resource.get() {
            profiles.retain(|p| p.is_public);
            if !profiles.is_empty() && active_profile_id.get_untracked().is_none() {
                set_active_profile_id.set(Some(profiles[0].id));
            }
        }
    });

    let profile_data_resource = create_resource(
        move || active_profile_id.get(),
        |id_opt| async move {
            crate::resume_engine::get_tenant_entries(id_opt)
                .await
                .unwrap_or_default()
        },
    );

    let handle_download_click = move |_| {
        set_show_modal.set(true);
    };

    let submit_lead = move |_| {
        let name = lead_name.get_untracked();
        let email = lead_email.get_untracked();
        let id_opt = active_profile_id.get_untracked();

        if email.is_empty() || name.is_empty() {
            return;
        }

        spawn_local(async move {
            let _ = crate::pages::dynamic_landing::handle_dynamic_lead(
                "resume_download".to_string(),
                email,
                vec![format!("Name: {}", name)],
            )
            .await;

            if let Some(id) = id_opt {
                download_pdf.dispatch(id);
            }
            set_show_modal.set(false);
        });
    };

    view! {
        <main class="pt-32 pb-24 px-4 md:px-[8.5rem] bg-surface-container-low min-h-screen">
            <div class="mb-24 flex flex-col md:flex-row justify-between md:items-end max-w-4xl border-b-2 border-outline-variant/30 pb-8">
                <crate::components::dynamic_header::DynamicPageHeader route_path="/resume".to_string() badge_color="primary".to_string() />

                <div class="mt-8 md:mt-0 flex flex-col items-start md:items-end">
                    <button
                        on:click=handle_download_click
                        class="bg-secondary shadow-none border-none outline-none text-white px-6 py-3 font-label text-sm tracking-widest font-bold uppercase hover:bg-on-secondary-fixed transition-colors flex items-center gap-2 rounded-none"
                    >
                        <span class="material-symbols-outlined">"download"</span>
                        "GET_PDF"
                    </button>
                    <div class="text-[0.6rem] text-outline mt-2 jetbrains text-left md:text-right">"GENERATES LATEX >> PDF VIA TECTONIC"</div>
                </div>
            </div>

            <div class="max-w-4xl mb-16">
                <Suspense fallback=move || view! { <div class="text-xs jetbrains text-outline uppercase animate-pulse">"Loading profiles..."</div> }>
                    {move || {
                        let all_profiles = profiles_resource.get().unwrap_or_default();
                        let profiles: Vec<_> = all_profiles.into_iter().filter(|p| p.is_public).collect();
                        if profiles.is_empty() {
                            view! { <div class="text-sm jetbrains text-outline">"No public profiles available."</div> }.into_view()
                        } else {
                            view! {
                                <div class="flex flex-col md:flex-row gap-4 items-start md:items-center">
                                    <label class="jetbrains text-xs text-secondary font-bold uppercase tracking-widest block">"ACTIVE PROFILE_SET //"</label>
                                    <select
                                        class="bg-surface border-2 border-outline-variant/30 text-on-surface text-sm font-bold jetbrains px-4 py-2 outline-none focus:border-primary transition-colors cursor-pointer w-full md:w-auto"
                                        on:change=move |ev| {
                                            if let Ok(id) = event_target_value(&ev).parse::<i32>() {
                                                set_active_profile_id.set(Some(id));
                                            }
                                        }
                                    >
                                        {profiles.into_iter().map(|p| view! {
                                            <option
                                                value=p.id
                                                selected=move || active_profile_id.get() == Some(p.id)
                                            >
                                                {p.name}
                                            </option>
                                        }).collect_view()}
                                    </select>
                                </div>
                            }.into_view()
                        }
                    }}
                </Suspense>
            </div>

            <Show when=move || show_modal.get()>
                <div class="fixed inset-0 z-[60] flex items-center justify-center bg-background/90 p-6 backdrop-blur-sm">
                    <div class="relative w-full max-w-lg bg-surface-container-highest p-8 blueprint-overlay shadow-[0_0_40px_rgba(0,184,212,0.1)]">
                        <button on:click=move |_| set_show_modal.set(false) class="absolute -top-4 -right-4 bg-surface-container-high border-2 border-outline-variant p-2 rounded-full text-secondary hover:text-error hover:border-error transition-all shadow-xl z-[70]">
                            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="3">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>

                        <div class="mb-8 border-b-2 border-outline-variant/30 pb-4">
                            <h2 class="text-2xl font-extrabold text-primary uppercase tracking-widest mb-2">"AUTHORIZATION REQUIRED"</h2>
                            <p class="jetbrains text-xs text-secondary/80">"Please identify yourself to compile and download this profile's TeX payload."</p>
                        </div>

                        <div class="space-y-6">
                            <div class="flex flex-col gap-2">
                                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Full Name"</label>
                                <input type="text" prop:value=lead_name on:input=move |ev| set_lead_name.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="John Doe" />
                            </div>
                            <div class="flex flex-col gap-2">
                                <label class="jetbrains text-[0.65rem] uppercase text-outline tracking-wider">"Email Address"</label>
                                <input type="email" prop:value=lead_email on:input=move |ev| set_lead_email.set(event_target_value(&ev)) class="bg-surface p-3 border border-outline-variant focus:border-primary focus:ring-0 text-sm jetbrains" placeholder="guest@example.com" />
                            </div>

                            <button
                                on:click=submit_lead
                                disabled=move || lead_name.get().is_empty() || lead_email.get().is_empty() || download_pdf.pending().get()
                                class="w-full bg-primary text-on-primary py-4 mt-4 jetbrains text-xs font-bold tracking-[0.2em] uppercase hover:bg-primary-container disabled:opacity-50 disabled:cursor-not-allowed transition-colors shadow-lg flex justify-center items-center gap-2"
                            >
                                <Show when=move || download_pdf.pending().get() fallback=|| view! { <span>"EXTRACT COMPILED PAYLOAD"</span> }>
                                    <span class="material-symbols-outlined animate-spin">"autorenew"</span>
                                    <span>"COMPILING LATEX..."</span>
                                </Show>
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            <Suspense fallback=move || view! { <div class="text-on-surface-variant font-bold jetbrains uppercase h-64 flex items-center justify-center">"Hydrating systems database..."</div> }>
                {move || {
                    let entries = profile_data_resource.get().unwrap_or_default();

                    let categories_to_render = vec![
                        (crate::resume_engine::ResumeCategory::Work, "Experience"),
                        (crate::resume_engine::ResumeCategory::Education, "Education"),
                        (crate::resume_engine::ResumeCategory::Skill, "Skills"),
                        (crate::resume_engine::ResumeCategory::Project, "Projects"),
                        (crate::resume_engine::ResumeCategory::Language, "Languages"),
                        (crate::resume_engine::ResumeCategory::Volunteer, "Volunteering"),
                        (crate::resume_engine::ResumeCategory::Extracurricular, "Extracurriculars"),
                        (crate::resume_engine::ResumeCategory::Hobby, "Hobbies"),
                    ];

                    view! {
                        <div class="max-w-4xl space-y-24">
                            {categories_to_render.into_iter().map(|(cat_enum, section_title)| {
                                let cat_entries: Vec<_> = entries.iter()
                                    .filter(|e| e.category == cat_enum && e.is_visible)
                                    .cloned()
                                    .collect();

                                if cat_entries.is_empty() {
                                    view! { <div class="hidden"></div> }.into_view()
                                } else {
                                    view! {
                                        <div class="block">
                                            <h2 class="text-3xl font-extrabold text-on-surface mb-12 border-l-4 border-secondary pl-4 uppercase">{section_title}</h2>
                                            <div class="space-y-16">
                                                {cat_entries.into_iter().map(|entry| view! {
                                                    <section class="grid grid-cols-1 md:grid-cols-12 gap-4 md:gap-8 border-l border-outline-variant/30 pl-4 md:pl-0 md:border-l-0">
                                                        <div class="md:col-span-3 font-label text-xs sm:text-sm text-outline font-bold pt-1 uppercase tracking-widest">
                                                            <span class="md:hidden text-secondary mr-2">"↳"</span>
                                                            {entry.date_range.unwrap_or_default()}
                                                        </div>
                                                        <div class="md:col-span-9 bg-surface-container p-6 md:p-8 blueprint-overlay shadow-none border-0 ring-0 hover:bg-surface-container-high transition-colors">
                                                            <h3 class="text-xl md:text-2xl font-bold text-primary mb-1">{entry.title}</h3>
                                                            {match entry.subtitle {
                                                                Some(sub) => view! { <div class="text-secondary font-medium mb-6">{sub}</div> }.into_view(),
                                                                None => view! { <div class="mb-6"></div> }.into_view()
                                                            }}
                                                            <ul class="text-on-surface-variant leading-relaxed text-sm space-y-3 list-none p-0 m-0">
                                                                {entry.bullets.into_iter().map(|b| view! {
                                                                    <li class="relative pl-4 before:content-['>'] before:absolute before:-left-1 before:text-secondary before:font-bold">
                                                                        {b}
                                                                    </li>
                                                                }).collect_view()}
                                                            </ul>
                                                        </div>
                                                    </section>
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    }.into_view()
                                }
                            }).collect_view()}
                        </div>
                    }
                }}
            </Suspense>
        </main>
    }
}
