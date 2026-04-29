use leptos::prelude::*;
use crate::api::onboarding::{
    OnboardingStatusResponse, OnboardingStepStatus,
    get_onboarding_status, skip_step, complete_step, dismiss_wizard,
};

// ──────────────────────────────────────────────────────────────────────────────
// STEP STATUS DOT (progress indicator at the top)
// ──────────────────────────────────────────────────────────────────────────────

#[component]
fn StepDot(is_complete: bool, is_current: bool, is_required: bool) -> impl IntoView {
    let class = if is_current {
        "w-3 h-3 rounded-full bg-indigo-600 ring-2 ring-indigo-200"
    } else if is_complete {
        "w-3 h-3 rounded-full bg-green-500"
    } else if is_required {
        "w-3 h-3 rounded-full border-2 border-gray-300 bg-white"
    } else {
        "w-3 h-3 rounded-full border-2 border-gray-200 bg-white opacity-60"
    };
    view! { <div class=class></div> }
}

// ──────────────────────────────────────────────────────────────────────────────
// INDIVIDUAL STEP CONTENT PANELS
// Each step renders its own lightweight form. Data steps (domain, categories)
// delegate actual submission to their existing API paths and re-fetch the status.
// ──────────────────────────────────────────────────────────────────────────────

#[component]
fn IdentityStep(
    tenant_id: String,
    app_instance_id: String,
    on_complete: Callback<()>,
) -> impl IntoView {
    let site_title = RwSignal::new(String::new());
    let tagline = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let ai = app_instance_id.clone();
    let save = move |_| {
        let title = site_title.get();
        let tag = tagline.get();
        if title.trim().is_empty() {
            error.set(Some("Site name is required.".to_string()));
            return;
        }
        let ai = ai.clone();
        let on_complete = on_complete.clone();
        saving.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            let base = crate::api::client::api_url(&format!("api/tenants/{}/settings", ai));
            let client = crate::api::client::create_client();

            // 1. Save site_title (required)
            let payload = serde_json::json!({"key": "site_title", "value": title});
            let res = crate::api::client::with_credentials(client.post(&base).json(&payload))
                .send()
                .await;
            match res {
                Ok(r) if !r.status().is_success() => {
                    saving.set(false);
                    error.set(Some(format!("Error saving site name: HTTP {}", r.status())));
                    return;
                }
                Err(e) => {
                    saving.set(false);
                    error.set(Some(e.to_string()));
                    return;
                }
                _ => {}
            }

            // 2. Save tagline (optional — only if user provided one)
            if !tag.trim().is_empty() {
                let tl_payload = serde_json::json!({"key": "site_tagline", "value": tag});
                let tl_res = crate::api::client::with_credentials(
                    client.post(&base).json(&tl_payload)
                )
                .send()
                .await;
                if let Err(e) = tl_res {
                    // Tagline failure is non-fatal — warn but still advance
                    leptos::logging::warn!("Failed to save tagline: {}", e);
                }
            }

            saving.set(false);
            on_complete.run(());
        });
    };

    view! {
        <div class="space-y-4">
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1" for="ob-site-title">
                    "Site Name"
                    <span class="text-red-500 ml-1">"*"</span>
                </label>
                <input
                    id="ob-site-title"
                    type="text"
                    placeholder="Acme Corp."
                    class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent text-gray-900"
                    on:input=move |e| site_title.set(event_target_value(&e))
                />
            </div>
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1" for="ob-tagline">
                    "Tagline"
                    <span class="text-gray-400 text-xs ml-1">"(optional)"</span>
                </label>
                <input
                    id="ob-tagline"
                    type="text"
                    placeholder="Building the future of..."
                    class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent text-gray-900"
                    on:input=move |e| tagline.set(event_target_value(&e))
                />
            </div>
            {move || error.get().map(|e| view! {
                <p class="text-sm text-red-600">{e}</p>
            })}
            <button
                id="ob-identity-save"
                class="w-full py-2.5 px-6 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg transition-colors disabled:opacity-50"
                disabled=move || saving.get()
                on:click=save
            >
                {move || if saving.get() { "Saving..." } else { "Save & Continue" }}
            </button>
        </div>
    }
}

#[component]
fn DomainStep(
    app_instance_id: String,
    on_complete: Callback<()>,
) -> impl IntoView {
    let domain = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let ai = app_instance_id.clone();
    let save = move |_| {
        let d = domain.get();
        if d.trim().is_empty() {
            error.set(Some("Domain is required.".to_string()));
            return;
        }
        let ai = ai.clone();
        let on_complete = on_complete.clone();
        saving.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            let url = crate::api::client::api_url(&format!("api/app-instances/{}/domains", ai));
            let client = crate::api::client::create_client();
            let payload = serde_json::json!({"fqdn": d, "is_primary": true});
            let res = crate::api::client::with_credentials(client.post(&url).json(&payload))
                .send()
                .await;
            match res {
                Ok(r) if r.status().is_success() => {
                    saving.set(false);
                    on_complete.run(());
                }
                Ok(r) if r.status().as_u16() == 409 => {
                    // A 409 means the domain record already exists, but we must
                    // verify it belongs to THIS app_instance — not another tenant.
                    // The backend returns {app_instance_id: "..."} in the 409 body.
                    let body: serde_json::Value =
                        r.json().await.unwrap_or(serde_json::Value::Null);
                    let owner = body
                        .get("app_instance_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if owner == ai {
                        // Already bound to us — treat as a successful idempotent bind
                        saving.set(false);
                        on_complete.run(());
                    } else {
                        saving.set(false);
                        error.set(Some(
                            "This domain is already in use by another account. \
                             Please use a different domain or contact support."
                                .to_string(),
                        ));
                    }
                }
                Ok(r) => {
                    saving.set(false);
                    error.set(Some(format!("Error: HTTP {}", r.status())));
                }
                Err(e) => {
                    saving.set(false);
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    view! {
        <div class="space-y-4">
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1" for="ob-domain">
                    "Domain"
                    <span class="text-red-500 ml-1">"*"</span>
                </label>
                <div class="flex items-center gap-2">
                    <span class="text-gray-400 text-sm">"https://"</span>
                    <input
                        id="ob-domain"
                        type="text"
                        placeholder="yourdomain.com"
                        class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent text-gray-900"
                        on:input=move |e| domain.set(event_target_value(&e))
                    />
                </div>
                <p class="text-xs text-gray-500 mt-1">
                    "Make sure your DNS A record points to this platform before binding."
                </p>
            </div>
            {move || error.get().map(|e| view! {
                <p class="text-sm text-red-600">{e}</p>
            })}
            <button
                id="ob-domain-save"
                class="w-full py-2.5 px-6 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg transition-colors disabled:opacity-50"
                disabled=move || saving.get()
                on:click=save
            >
                {move || if saving.get() { "Binding..." } else { "Bind Domain & Continue" }}
            </button>
        </div>
    }
}

#[component]
fn FirstPageStep(
    tenant_id: String,
    app_instance_id: String,
    on_complete: Callback<()>,
) -> impl IntoView {
    let page_title = RwSignal::new(String::new());
    let hero_text = RwSignal::new(String::new());
    let saving = RwSignal::new(false);
    let error = RwSignal::new(Option::<String>::None);

    let ai = app_instance_id.clone();
    let tid = tenant_id.clone();

    let save = move |_| {
        let title = page_title.get();
        let hero = hero_text.get();
        if title.trim().is_empty() {
            error.set(Some("Page title is required.".to_string()));
            return;
        }
        let ai = ai.clone();
        let tid = tid.clone();
        let on_complete = on_complete.clone();
        saving.set(true);
        error.set(None);
        leptos::task::spawn_local(async move {
            let url = crate::api::client::api_url("api/app-pages");
            let client = crate::api::client::create_client();
            let blocks = serde_json::json!([{
                "type": "Hero",
                "content": {
                    "title": title,
                    "subtitle": hero
                }
            }]);
            let payload = serde_json::json!({
                "tenant_id": tid,
                "app_instance_id": ai,
                "slug": "home",
                "title": title,
                "blocks": blocks,
                "is_published": false
            });
            let res = crate::api::client::with_credentials(client.post(&url).json(&payload))
                .send()
                .await;
            match res {
                Ok(r) if r.status().is_success() || r.status().as_u16() == 409 => {
                    saving.set(false);
                    on_complete.run(());
                }
                Ok(r) => {
                    saving.set(false);
                    error.set(Some(format!("Error: HTTP {}", r.status())));
                }
                Err(e) => {
                    saving.set(false);
                    error.set(Some(e.to_string()));
                }
            }
        });
    };

    let ai_for_link = app_instance_id.clone();

    view! {
        <div class="space-y-4">
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1" for="ob-page-title">
                    "Page Title"
                    <span class="text-red-500 ml-1">"*"</span>
                </label>
                <input
                    id="ob-page-title"
                    type="text"
                    placeholder="Home"
                    class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent text-gray-900"
                    on:input=move |e| page_title.set(event_target_value(&e))
                />
            </div>
            <div>
                <label class="block text-sm font-medium text-gray-700 mb-1" for="ob-hero-text">
                    "Hero Text"
                    <span class="text-gray-400 text-xs ml-1">"(optional)"</span>
                </label>
                <input
                    id="ob-hero-text"
                    type="text"
                    placeholder="Welcome to our site"
                    class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-transparent text-gray-900"
                    on:input=move |e| hero_text.set(event_target_value(&e))
                />
            </div>
            <p class="text-xs text-gray-500">
                "This creates a simple home page. You can "
                <a
                    href=format!("/apps/{}/cms", ai_for_link)
                    class="text-indigo-600 underline hover:text-indigo-800"
                >
                    "edit it fully in the CMS"
                </a>
                " after setup."
            </p>
            {move || error.get().map(|e| view! {
                <p class="text-sm text-red-600">{e}</p>
            })}
            <button
                id="ob-page-save"
                class="w-full py-2.5 px-6 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg transition-colors disabled:opacity-50"
                disabled=move || saving.get()
                on:click=save
            >
                {move || if saving.get() { "Creating..." } else { "Create Page & Continue" }}
            </button>
        </div>
    }
}

#[component]
fn GenericCustomStep(
    step: OnboardingStepStatus,
    app_instance_id: String,
    on_complete: Callback<()>,
    on_skip: Option<Callback<()>>,
) -> impl IntoView {
    let saving = RwSignal::new(false);
    let step_id = step.id.clone();
    let ai = app_instance_id.clone();
    let is_required = step.is_required;

    let mark_done = move |_| {
        let ai = ai.clone();
        let step_id = step_id.clone();
        let on_complete = on_complete.clone();
        saving.set(true);
        leptos::task::spawn_local(async move {
            let _ = complete_step(&ai, &step_id).await;
            saving.set(false);
            on_complete.run(());
        });
    };

    let ai2 = app_instance_id.clone();
    let step_id2 = step.id.clone();
    let skip_action = on_skip.map(|cb| {
        move |_: web_sys::MouseEvent| {
            let ai = ai2.clone();
            let sid = step_id2.clone();
            let cb = cb.clone();
            leptos::task::spawn_local(async move {
                let _ = skip_step(&ai, &sid).await;
                cb.run(());
            });
        }
    });

    view! {
        <div class="space-y-4">
            <p class="text-gray-600 text-sm">
                "This step requires manual configuration. Once done, click the button below."
            </p>
            <div class="flex gap-3">
                <button
                    id=format!("ob-{}-complete", step.id)
                    class="flex-1 py-2.5 px-6 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg transition-colors disabled:opacity-50"
                    disabled=move || saving.get()
                    on:click=mark_done
                >
                    {move || if saving.get() { "Marking..." } else { "Mark as Complete" }}
                </button>
                {skip_action.map(|f| view! {
                    <button
                        id=format!("ob-{}-skip", step.id)
                        class="py-2.5 px-4 border border-gray-300 text-gray-600 rounded-lg hover:bg-gray-50 transition-colors text-sm"
                        on:click=f
                    >
                        "Skip"
                    </button>
                })}
            </div>
        </div>
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// ONBOARDING COMPLETE CELEBRATION VIEW
// ──────────────────────────────────────────────────────────────────────────────

#[component]
fn OnboardingComplete(app_instance_id: String) -> impl IntoView {
    // Use leptos_router::A for client-side SPA navigation (avoids full page reload)
    use leptos_router::components::A;
    view! {
        <div class="text-center space-y-6 py-8">
            <div class="text-6xl animate-bounce">"🎉"</div>
            <div>
                <h2 class="text-2xl font-bold text-gray-900">"You're live-ready!"</h2>
                <p class="text-gray-500 mt-2">
                    "All required setup steps are complete. Your app is ready to go."
                </p>
            </div>
            <A
                href=format!("/apps/{}", app_instance_id)
                attr:id="ob-goto-dashboard"
                attr:class="inline-block py-3 px-8 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg transition-colors"
            >
                "Go to Dashboard →"
            </A>
        </div>
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MAIN ONBOARDING WIZARD COMPONENT
// ──────────────────────────────────────────────────────────────────────────────

#[component]
pub fn OnboardingWizard(
    app_instance_id: String,
    tenant_id: String,
    /// Called after the wizard is dismissed so the parent can refetch
    /// `onboarding_status` and immediately show the persistent banner
    /// without requiring a page reload.
    #[prop(optional)]
    on_dismiss: Option<Callback<()>>,
) -> impl IntoView {
    let ai = app_instance_id.clone();
    let status: LocalResource<Result<OnboardingStatusResponse, String>> =
        LocalResource::new(move || {
            let ai = ai.clone();
            async move { get_onboarding_status(&ai).await }
        });

    let current_step_index = RwSignal::new(0usize);
    let dismissed = RwSignal::new(false);

    let ai_dismiss = app_instance_id.clone();
    let dismiss = StoredValue::new(move |_: web_sys::MouseEvent| {
        let ai = ai_dismiss.clone();
        let cb = on_dismiss.clone();
        leptos::task::spawn_local(async move {
            let _ = dismiss_wizard(&ai).await;
            // Notify the parent to refetch so the banner appears immediately
            if let Some(f) = cb {
                f.run(());
            }
        });
        dismissed.set(true);
    });

    let ai_id = app_instance_id.clone();
    let tenant_id_clone = tenant_id.clone();

    view! {
        <Suspense fallback=move || view! {
            <div class="fixed inset-0 bg-white z-50 flex items-center justify-center">
                <div class="animate-spin rounded-full h-12 w-12 border-4 border-indigo-500 border-t-transparent"></div>
            </div>
        }>
            {move || {
                if dismissed.get() {
                    return view! { <div></div> }.into_any();
                }

                match status.get() {
                    None => view! { <div></div> }.into_any(),
                    Some(Err(e)) => view! {
                        <div class="fixed inset-0 bg-white z-50 flex items-center justify-center">
                            <p class="text-red-600">"Failed to load onboarding: " {e}</p>
                        </div>
                    }.into_any(),
                    Some(Ok(s)) => {
                        if s.is_ready {
                            return view! { <div></div> }.into_any();
                        }

                        let steps = s.steps.clone();
                        let total = steps.len();
                        let current = current_step_index.get().min(total.saturating_sub(1));
                        let current_step = steps.get(current).cloned();

                        let ai = ai_id.clone();
                        let tid = tenant_id_clone.clone();

                        let go_next = move || {
                            let new_idx = (current_step_index.get() + 1).min(total);
                            current_step_index.set(new_idx);
                            // Re-fetch status to pick up data changes
                            status.refetch();
                        };

                        let go_prev = move |_: web_sys::MouseEvent| {
                            if current_step_index.get() > 0 {
                                current_step_index.set(current_step_index.get() - 1);
                            }
                        };

                        view! {
                            <div class="fixed inset-0 bg-gray-50 z-50 overflow-y-auto">
                                <div class="min-h-screen flex flex-col">
                                    // ── Header ────────────────────────────
                                    <div class="bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between">
                                        <div class="flex items-center gap-3">
                                            <span class="text-xl font-bold text-indigo-700">"⚡ Setup Wizard"</span>
                                            <span class="text-sm text-gray-400">
                                                "Step " {current + 1} " of " {total}
                                            </span>
                                        </div>
                                        // Progress dots
                                        <div class="hidden sm:flex items-center gap-2">
                                            {steps.iter().enumerate().map(|(i, step)| {
                                                view! {
                                                    <StepDot
                                                        is_complete=step.is_complete
                                                        is_current=(i == current)
                                                        is_required=step.is_required
                                                    />
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>

                                    // ── Body ──────────────────────────────
                                    <div class="flex-1 flex items-center justify-center p-6">
                                        <div class="w-full max-w-lg bg-white rounded-2xl shadow-lg p-8 space-y-6">
                                            {match &current_step {
                                                None => view! {
                                                    <OnboardingComplete app_instance_id=ai.clone() />
                                                }.into_any(),
                                                Some(step) => {
                                                    let step = step.clone();

                                                    // Step header
                                                    let header = view! {
                                                        <div class="space-y-1">
                                                            <div class="flex items-center gap-2">
                                                                <span class="text-xs font-semibold uppercase tracking-widest text-indigo-500">
                                                                    {if step.is_required { "Required" } else { "Optional" }}
                                                                </span>
                                                                {step.is_complete.then(|| view! {
                                                                    <span class="text-xs font-semibold text-green-600 bg-green-50 px-2 py-0.5 rounded-full">"✓ Complete"</span>
                                                                })}
                                                            </div>
                                                            <h2 class="text-2xl font-bold text-gray-900">{step.title.clone()}</h2>
                                                            <p class="text-gray-500">{step.description.clone()}</p>
                                                        </div>
                                                    };

                                                    let step_id = step.id.clone();
                                                    let ai_s = ai.clone();
                                                    let tid_s = tid.clone();

                                                    let go_next_cb = Callback::new(move |_: ()| go_next());

                                                    // Route to the right step component
                                                    let step_body = match step.id.as_str() {
                                                        "identity" => view! {
                                                            <IdentityStep
                                                                tenant_id=tid_s.clone()
                                                                app_instance_id=ai_s.clone()
                                                                on_complete=go_next_cb.clone()
                                                            />
                                                        }.into_any(),
                                                        "domain" => view! {
                                                            <DomainStep
                                                                app_instance_id=ai_s.clone()
                                                                on_complete=go_next_cb.clone()
                                                            />
                                                        }.into_any(),
                                                        "first_page" => view! {
                                                            <FirstPageStep
                                                                tenant_id=tid_s.clone()
                                                                app_instance_id=ai_s.clone()
                                                                on_complete=go_next_cb.clone()
                                                            />
                                                        }.into_any(),
                                                        _ => {
                                                            let skip_cb = (!step.is_required).then(|| {
                                                                Callback::new(move |_: ()| go_next())
                                                            });
                                                            view! {
                                                                <GenericCustomStep
                                                                    step=step.clone()
                                                                    app_instance_id=ai_s.clone()
                                                                    on_complete=go_next_cb.clone()
                                                                    on_skip=skip_cb
                                                                />
                                                            }.into_any()
                                                        }
                                                    };

                                                    view! {
                                                        <div class="space-y-6">
                                                            {header}
                                                            <hr class="border-gray-100" />
                                                            {step_body}
                                                        </div>
                                                    }.into_any()
                                                }
                                            }}
                                        </div>
                                    </div>

                                    // ── Footer navigation ─────────────────
                                    <div class="bg-white border-t border-gray-200 px-6 py-4 flex items-center justify-between">
                                        <button
                                            class="text-sm text-gray-500 hover:text-gray-700 flex items-center gap-1 disabled:opacity-30"
                                            disabled=move || current_step_index.get() == 0
                                            on:click=go_prev
                                        >
                                            "← Back"
                                        </button>
                                        <button
                                            id="ob-dismiss"
                                            class="text-sm text-gray-400 hover:text-gray-600 underline"
                                            on:click=move |e| dismiss.get_value()(e)
                                        >
                                            "I'll do this later →"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    }
                }
            }}
        </Suspense>
    }
}
