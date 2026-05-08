use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────────────────────
// TYPES (mirror the backend handler response structs)
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnboardingStepStatus {
    pub id: String,
    pub title: String,
    pub description: String,
    pub is_required: bool,
    pub is_complete: bool,
    pub is_skipped: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnboardingStatusResponse {
    pub app_instance_id: String,
    pub tenant_id: String,
    pub app_type: String,
    pub steps: Vec<OnboardingStepStatus>,
    pub is_ready: bool,
    pub dismissed_at: Option<String>,
}

// ──────────────────────────────────────────────────────────────────────────────
// SERVER FUNCTIONS (SSR — token-gated via the public backend routes)
// ──────────────────────────────────────────────────────────────────────────────

#[server(GetOnboardingStatus, "/api")]
pub async fn get_onboarding_status(
    app_instance_id: String,
    token: String,
) -> Result<OnboardingStatusResponse, ServerFnError> {
    let url = format!(
        "{}/onboarding/status/{}?token={}",
        crate::atlas_client::get_atlas_api_url(),
        app_instance_id,
        token,
    );
    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ServerFnError::<server_fn::error::NoCustomError>::ServerError(e.to_string()))?;
    if res.status().is_success() {
        res.json::<OnboardingStatusResponse>()
            .await
            .map_err(|e| ServerFnError::<server_fn::error::NoCustomError>::ServerError(e.to_string()))
    } else {
        Err(ServerFnError::<server_fn::error::NoCustomError>::ServerError(
            format!("HTTP {}", res.status()),
        ))
    }
}

#[server(CompleteOnboardingStep, "/api")]
pub async fn complete_onboarding_step(
    app_instance_id: String,
    step_id: String,
    token: String,
) -> Result<(), ServerFnError> {
    // POST mutation: token goes in Authorization header, not the query string.
    // Query params are logged by reverse proxies, CDNs, and stored in browser history.
    let url = format!(
        "{}/onboarding/step/{}/{}",
        crate::atlas_client::get_atlas_api_url(),
        app_instance_id,
        step_id,
    );
    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::<server_fn::error::NoCustomError>::ServerError(e.to_string()))?;
    if res.status().is_success() || res.status().as_u16() == 204 {
        Ok(())
    } else {
        Err(ServerFnError::<server_fn::error::NoCustomError>::ServerError(
            format!("HTTP {}", res.status()),
        ))
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// STEP DOT
// ──────────────────────────────────────────────────────────────────────────────

#[component]
fn StepDot(is_complete: bool, is_current: bool) -> impl IntoView {
    let class = if is_current {
        "w-3 h-3 rounded-full bg-indigo-600 ring-2 ring-indigo-200"
    } else if is_complete {
        "w-3 h-3 rounded-full bg-green-500"
    } else {
        "w-3 h-3 rounded-full border-2 border-gray-300 bg-white"
    };
    view! { <div class=class></div> }
}

// ──────────────────────────────────────────────────────────────────────────────
// MAIN PAGE COMPONENT
// ──────────────────────────────────────────────────────────────────────────────

/// Tenant self-service onboarding portal.
///
/// Route: `/setup?token=<setup_token>&app=<app_instance_id>`
///
/// This page is publicly accessible but all mutations require the `setup_token`
/// to be present in the query string. The token is validated server-side by
/// comparing against the `setup_token` TenantSetting for this app instance.
#[component]
pub fn TenantOnboarding() -> impl IntoView {
    let query = use_query_map();

    let token = move || query.with(|q| q.get("token").cloned().unwrap_or_default());
    let app_instance_id = move || query.with(|q| q.get("app").cloned().unwrap_or_default());

    let status_resource = Resource::new(
        move || (app_instance_id(), token()),
        |(ai, tok)| async move { get_onboarding_status(ai, tok).await },
    );

    let current_step_index = create_rw_signal(0usize);
    let (action_error, set_action_error) = create_signal(Option::<String>::None);
    let (completing, set_completing) = create_signal(false);

    view! {
        <leptos_meta::Title text="App Setup — Onboarding Wizard" />
        <leptos_meta::Meta name="robots" content="noindex, nofollow" />

        <main class="min-h-screen bg-gradient-to-br from-slate-50 to-indigo-50 flex flex-col">
            // ── Header ───────────────────────────────────────────────────────
            <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between">
                <span class="text-xl font-bold text-indigo-700">"⚡ Account Setup"</span>
                <Transition fallback=|| ()>
                    {move || status_resource.get().and_then(Result::ok).map(|s| {
                        let total = s.steps.len();
                        let done = s.steps.iter().filter(|st| st.is_complete).count();
                        let current = current_step_index.get().min(total.saturating_sub(1));
                        view! {
                            <div class="flex items-center gap-4">
                                // Progress dots
                                <div class="hidden sm:flex items-center gap-2">
                                    {s.steps.iter().enumerate().map(|(i, step)| {
                                        view! {
                                            <StepDot
                                                is_complete=step.is_complete
                                                is_current=(i == current)
                                            />
                                        }
                                    }).collect_view()}
                                </div>
                                <span class="text-sm text-gray-500">
                                    {format!("{} / {} complete", done, total)}
                                </span>
                            </div>
                        }.into_view()
                    })}
                </Transition>
            </div>

            // ── Body ─────────────────────────────────────────────────────────
            <div class="flex-1 flex items-center justify-center p-6">
                <Transition fallback=move || view! {
                    <div class="text-center">
                        <div class="animate-spin rounded-full h-12 w-12 border-4 border-indigo-500 border-t-transparent mx-auto mb-4"></div>
                        <p class="text-gray-500">"Loading your setup wizard..."</p>
                    </div>
                }>
                    {move || {
                        let tok = token();
                        let ai = app_instance_id();

                        if tok.is_empty() || ai.is_empty() {
                            return view! {
                                <div class="w-full max-w-md text-center bg-white rounded-2xl shadow p-10">
                                    <div class="text-5xl mb-4">"🔒"</div>
                                    <h1 class="text-xl font-bold text-gray-900">"Invalid Setup Link"</h1>
                                    <p class="text-gray-500 mt-2">"This link is invalid or has expired. Please contact your platform administrator for a new setup link."</p>
                                </div>
                            }.into_view();
                        }

                        match status_resource.get() {
                            None => view! { <div></div> }.into_view(),
                            Some(Err(e)) => view! {
                                <div class="w-full max-w-md text-center bg-white rounded-2xl shadow p-10">
                                    <div class="text-5xl mb-4">"⚠️"</div>
                                    <h1 class="text-xl font-bold text-gray-900">"Setup Error"</h1>
                                    <p class="text-gray-500 mt-2">{e.to_string()}</p>
                                    <p class="text-xs text-gray-400 mt-2">"Your setup link may have expired."</p>
                                </div>
                            }.into_view(),
                            Some(Ok(status)) => {
                                if status.is_ready {
                                    return view! {
                                        <div class="w-full max-w-md text-center bg-white rounded-2xl shadow p-10 space-y-6">
                                            <div class="text-6xl animate-bounce">"🎉"</div>
                                            <div>
                                                <h1 class="text-2xl font-bold text-gray-900">"You're all set!"</h1>
                                                <p class="text-gray-500 mt-2">"Your account is configured and ready. You can now start using the platform."</p>
                                            </div>
                                            <div class="bg-green-50 text-green-700 text-sm font-medium rounded-lg px-4 py-3">
                                                "✓ All required steps complete"
                                            </div>
                                        </div>
                                    }.into_view();
                                }

                                let steps = status.steps.clone();
                                let total = steps.len();

                                // Jump to the first incomplete required step automatically
                                let auto_idx = steps.iter().position(|s| s.is_required && !s.is_complete).unwrap_or(0);
                                if current_step_index.get_untracked() == 0 {
                                    current_step_index.set(auto_idx);
                                }

                                let current = current_step_index.get().min(total.saturating_sub(1));
                                let current_step = steps.get(current).cloned();

                                let ai_clone = ai.clone();
                                let tok_clone = tok.clone();

                                view! {
                                    <div class="w-full max-w-lg bg-white rounded-2xl shadow-lg">
                                        {match current_step {
                                            None => view! {
                                                <div class="p-8 text-center space-y-4">
                                                    <div class="text-5xl">"✓"</div>
                                                    <p class="text-gray-600">"All steps reviewed!"</p>
                                                </div>
                                            }.into_view(),
                                            Some(step) => {
                                                let step_id = step.id.clone();
                                                let ai_s = ai_clone.clone();
                                                let tok_s = tok_clone.clone();

                                                let complete_action = create_action(move |_: &()| {
                                                    let ai = ai_s.clone();
                                                    let sid = step_id.clone();
                                                    let tok = tok_s.clone();
                                                    async move {
                                                        complete_onboarding_step(ai, sid, tok).await
                                                    }
                                                });

                                                // Advance when the action succeeds
                                                Effect::new(move |_| {
                                                    if let Some(Ok(_)) = complete_action.value().get() {
                                                        let new_idx = (current_step_index.get() + 1).min(total);
                                                        current_step_index.set(new_idx);
                                                        status_resource.refetch();
                                                        set_action_error.set(None);
                                                        set_completing.set(false);
                                                    } else if let Some(Err(e)) = complete_action.value().get() {
                                                        set_action_error.set(Some(e.to_string()));
                                                        set_completing.set(false);
                                                    }
                                                });

                                                view! {
                                                    <div class="p-8 space-y-6">
                                                        // Step header
                                                        <div class="space-y-1">
                                                            <div class="flex items-center gap-2">
                                                                <span class="text-xs font-semibold uppercase tracking-widest text-indigo-500">
                                                                    {if step.is_required { "Required" } else { "Optional" }}
                                                                </span>
                                                                <span class="text-xs text-gray-400">
                                                                    {format!("Step {} of {}", current + 1, total)}
                                                                </span>
                                                                {step.is_complete.then(|| view! {
                                                                    <span class="text-xs font-semibold text-green-600 bg-green-50 px-2 py-0.5 rounded-full">
                                                                        "✓ Complete"
                                                                    </span>
                                                                })}
                                                            </div>
                                                            <h2 class="text-2xl font-bold text-gray-900">{step.title.clone()}</h2>
                                                            <p class="text-gray-500">{step.description.clone()}</p>
                                                        </div>

                                                        <hr class="border-gray-100" />

                                                        // Step guidance
                                                        <div class="bg-indigo-50 rounded-xl p-4 text-sm text-indigo-800">
                                                            <p class="font-medium mb-1">"What to do:"</p>
                                                            {match step.id.as_str() {
                                                                "identity" => view! {
                                                                    <p>"Contact your platform administrator to set your site name and branding, or log into your admin dashboard to update it under Settings → Identity."</p>
                                                                }.into_view(),
                                                                "domain" => view! {
                                                                    <p>"Provide your domain name to your platform administrator. Make sure your DNS A record points to the platform IP before confirming."</p>
                                                                }.into_view(),
                                                                "design" => view! {
                                                                    <p>"Your admin can configure colors and typography in Settings → Design. Once done, confirm below."</p>
                                                                }.into_view(),
                                                                "first_page" => view! {
                                                                    <p>"Your platform admin will create your initial home page. You'll be able to edit it in the CMS after setup is complete."</p>
                                                                }.into_view(),
                                                                "categories" => view! {
                                                                    <p>"Categories organize your listings. Your admin will set these up. Once they're ready, confirm below."</p>
                                                                }.into_view(),
                                                                "first_template" => view! {
                                                                    <p>"A listing template has been selected for your network. Confirm once you've reviewed it with your admin."</p>
                                                                }.into_view(),
                                                                _ => view! {
                                                                    <p>"Please coordinate with your platform administrator to complete this step, then confirm below."</p>
                                                                }.into_view()
                                                            }}
                                                        </div>

                                                        {move || action_error.get().map(|e| view! {
                                                            <p class="text-sm text-red-600">{e}</p>
                                                        })}

                                                        // Actions
                                                        <div class="flex gap-3">
                                                            {(!step.is_complete).then(|| {
                                                                let ca = complete_action.clone();
                                                                view! {
                                                                    <button
                                                                        id=format!("ob-tenant-{}-confirm", step.id)
                                                                        class="flex-1 py-3 px-6 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-xl transition-colors disabled:opacity-50"
                                                                        disabled=move || completing.get()
                                                                        on:click=move |_| {
                                                                            set_completing.set(true);
                                                                            ca.dispatch(());
                                                                        }
                                                                    >
                                                                        {move || if completing.get() { "Confirming..." } else { "Confirm & Continue →" }}
                                                                    </button>
                                                                }.into_view()
                                                            })}
                                                            {step.is_complete.then(|| view! {
                                                                <button
                                                                    id=format!("ob-tenant-{}-next", step.id)
                                                                    class="flex-1 py-3 px-6 bg-green-600 hover:bg-green-700 text-white font-semibold rounded-xl transition-colors"
                                                                    on:click=move |_| {
                                                                        let new_idx = (current_step_index.get() + 1).min(total);
                                                                        current_step_index.set(new_idx);
                                                                    }
                                                                >
                                                                    "Next Step →"
                                                                </button>
                                                            })}
                                                        </div>
                                                    </div>
                                                }.into_view()
                                            }
                                        }}

                                        // Navigation footer
                                        <div class="border-t border-gray-100 px-8 py-4 flex items-center justify-between">
                                            <button
                                                class="text-sm text-gray-500 hover:text-gray-700 disabled:opacity-30"
                                                disabled=move || current_step_index.get() == 0
                                                on:click=move |_| {
                                                    if current_step_index.get() > 0 {
                                                        current_step_index.set(current_step_index.get() - 1);
                                                    }
                                                }
                                            >
                                                "← Back"
                                            </button>
                                            <span class="text-xs text-gray-400">
                                                "Powered by Atlas Platform"
                                            </span>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                        }
                    }}
                </Transition>
            </div>
        </main>
    }
}
