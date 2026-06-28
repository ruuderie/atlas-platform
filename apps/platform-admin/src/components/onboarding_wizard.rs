use leptos::prelude::*;
use crate::api::onboarding::{
    OnboardingStatusResponse, OnboardingStepStatus,
    get_onboarding_status, skip_step, complete_step, dismiss_wizard,
};

// ── Step indicator dot — left rail ───────────────────────────────────────────

#[component]
fn StepDot(is_complete: bool, is_current: bool, is_required: bool) -> impl IntoView {
    let (bg, border, inner) = if is_complete {
        ("#22c55e", "#22c55e", view! { <span style="font-size:12px;color:#fff;">"✓"</span> }.into_any())
    } else if is_current {
        ("rgba(99,102,241,.2)", "#6366f1", view! { <div style="width:8px;height:8px;border-radius:50%;background:#818cf8;"></div> }.into_any())
    } else if is_required {
        ("rgba(255,255,255,.04)", "rgba(255,255,255,.15)", view! { <div style="width:6px;height:6px;border-radius:50%;background:rgba(255,255,255,.2);"></div> }.into_any())
    } else {
        ("rgba(255,255,255,.02)", "rgba(255,255,255,.08)", view! { <div style="width:4px;height:4px;border-radius:50%;background:rgba(255,255,255,.1);"></div> }.into_any())
    };
    view! {
        <div style=format!(
            "width:32px;height:32px;border-radius:50%;display:flex;align-items:center;\
             justify-content:center;flex-shrink:0;background:{};border:2px solid {};",
            bg, border
        )>
            {inner}
        </div>
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// INDIVIDUAL STEP CONTENT PANELS
// Each step renders its own lightweight form. Data steps (domain, categories)
// delegate actual submission to their existing API paths and re-fetch the status.
// ──────────────────────────────────────────────────────────────────────────────

#[component]
fn IdentityStep(
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
fn OnboardingComplete(app_instance_id: String, tenant_id: String) -> impl IntoView {
    use leptos_router::components::A;
    
    let email = RwSignal::new(String::new());
    let first_name = RwSignal::new(String::new());
    let last_name = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let setup_url = RwSignal::new(Option::<String>::None);
    let error = RwSignal::new(Option::<String>::None);
    
    let tid = tenant_id.clone();
    
    let provision_action = Action::new_local(move |_: &()| {
        let e = email.get();
        let f = first_name.get();
        let l = last_name.get();
        let tid = tid.clone();
        
        async move {
            if e.is_empty() || f.is_empty() || l.is_empty() {
                error.set(Some("All fields are required.".to_string()));
                return;
            }
            is_submitting.set(true);
            error.set(None);
            
            let url = crate::api::client::api_url(&format!("api/tenants/{}/provision-admin", tid));
            let payload = serde_json::json!({
                "email": e,
                "first_name": f,
                "last_name": l
            });
            let client = crate::api::client::create_client();
            let res = crate::api::client::with_credentials(client.post(&url).json(&payload))
                .send()
                .await;
                
            is_submitting.set(false);
            match res {
                Ok(r) if r.status().is_success() => {
                    if let Ok(body) = r.json::<serde_json::Value>().await {
                        if let Some(s) = body.get("setup_url").and_then(|v| v.as_str()) {
                            setup_url.set(Some(s.to_string()));
                        }
                    }
                }
                Ok(r) => {
                    error.set(Some(format!("Error: HTTP {}", r.status())));
                }
                Err(err) => {
                    error.set(Some(err.to_string()));
                }
            }
        }
    });

    view! {
        <div class="text-center space-y-6 py-8">
            <div class="text-6xl animate-bounce">"🎉"</div>
            <div>
                <h2 class="text-2xl font-bold text-gray-900">"You're live-ready!"</h2>
                <p class="text-gray-500 mt-2">
                    "All required setup steps are complete. Your app is ready to go."
                </p>
            </div>
            
            {move || if let Some(url) = setup_url.get() {
                view! {
                    <div class="bg-green-50 text-green-800 p-4 rounded-lg text-left mt-4 border border-green-200">
                        <h3 class="font-bold mb-2">"Admin Provisioned!"</h3>
                        <p class="text-sm mb-4">"Share this setup link with the new administrator so they can configure their passkey:"</p>
                        <div class="bg-white p-2 rounded border break-all font-mono text-xs select-all">
                            {url}
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="bg-gray-50 p-6 rounded-xl border border-gray-200 text-left space-y-4">
                        <div>
                            <h3 class="font-semibold text-gray-900">"Provision Tenant Owner"</h3>
                            <p class="text-sm text-gray-500">"Create the initial administrator account for this tenant."</p>
                        </div>
                        <div class="space-y-3">
                            <input
                                type="text"
                                placeholder="First Name"
                                class="w-full px-3 py-2 border rounded text-sm"
                                on:input=move |e| first_name.set(event_target_value(&e))
                            />
                            <input
                                type="text"
                                placeholder="Last Name"
                                class="w-full px-3 py-2 border rounded text-sm"
                                on:input=move |e| last_name.set(event_target_value(&e))
                            />
                            <input
                                type="email"
                                placeholder="Email Address"
                                class="w-full px-3 py-2 border rounded text-sm"
                                on:input=move |e| email.set(event_target_value(&e))
                            />
                        </div>
                        {move || error.get().map(|e| view! { <p class="text-sm text-red-600">{e}</p> })}
                        <button
                            class="w-full py-2 bg-gray-900 hover:bg-gray-800 text-white rounded font-medium disabled:opacity-50 transition-colors"
                            disabled=move || is_submitting.get()
                            on:click=move |_| { provision_action.dispatch(()); }
                        >
                            {move || if is_submitting.get() { "Provisioning..." } else { "Create Owner Account" }}
                        </button>
                    </div>
                }.into_any()
            }}

            <div class="pt-4">
                <A
                    href=format!("/apps/{}", app_instance_id)
                    attr:id="ob-goto-dashboard"
                    attr:class="inline-block py-3 px-8 bg-indigo-600 hover:bg-indigo-700 text-white font-semibold rounded-lg transition-colors"
                >
                    "Go to Dashboard →"
                </A>
            </div>
        </div>
    }
}

// ── Main Onboarding Wizard Component ────────────────────────────────────────

#[component]
pub fn OnboardingWizard(
    app_instance_id: String,
    tenant_id: String,
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
            // Loading shimmer — dark overlay with spinner
            <div style="position:fixed;inset:0;background:#0d1117;z-index:50;\
                        display:flex;align-items:center;justify-content:center;">
                <div style="width:48px;height:48px;border-radius:50%;\
                            border:3px solid rgba(99,102,241,.3);\
                            border-top-color:#6366f1;animation:spin 0.8s linear infinite;">
                </div>
            </div>
        }>
            {move || {
                if dismissed.get() {
                    return view! { <div></div> }.into_any();
                }

                match status.get() {
                    None => view! { <div></div> }.into_any(),
                    Some(Err(e)) => view! {
                        <div style="position:fixed;inset:0;background:#0d1117;z-index:50;\
                                    display:flex;align-items:center;justify-content:center;">
                            <div style="background:rgba(239,68,68,.1);border:1px solid rgba(239,68,68,.3);\
                                        border-radius:16px;padding:32px;max-width:480px;text-align:center;">
                                <div style="font-size:32px;margin-bottom:12px;">"⚠️"</div>
                                <p style="color:#fca5a5;font-size:15px;">
                                    "Failed to load setup status: " {e}
                                </p>
                            </div>
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
                        let completed_count = steps.iter().filter(|s| s.is_complete).count();
                        let progress_pct = if total > 1 {
                            (completed_count as f32 / (total - 1) as f32 * 100.0) as u32
                        } else { 0 };

                        let ai = ai_id.clone();
                        let tid = tenant_id_clone.clone();

                        let go_next = move || {
                            let new_idx = (current_step_index.get() + 1).min(total);
                            current_step_index.set(new_idx);
                            status.refetch();
                        };

                        let go_prev = move |_: web_sys::MouseEvent| {
                            if current_step_index.get() > 0 {
                                current_step_index.set(current_step_index.get() - 1);
                            }
                        };

                        view! {
                            // ── Full-screen dark glass takeover ───────────────────
                            <div style="position:fixed;inset:0;z-index:50;overflow:hidden;\
                                        background:linear-gradient(135deg,#070b11 0%,#0d1117 50%,#070b11 100%);\
                                        display:flex;font-family:'Inter',system-ui,sans-serif;">

                                // ── Left rail — step navigator ────────────────────
                                <aside style="width:280px;flex-shrink:0;border-right:1px solid rgba(255,255,255,.06);\
                                              background:rgba(255,255,255,.015);display:flex;flex-direction:column;\
                                              padding:40px 24px;overflow-y:auto;">

                                    // Wordmark
                                    <div style="margin-bottom:40px;">
                                        <div style="font-size:11px;font-weight:700;letter-spacing:.18em;\
                                                    text-transform:uppercase;color:rgba(165,180,252,.6);">
                                            "Atlas Platform"
                                        </div>
                                        <div style="font-size:15px;font-weight:700;color:#e2e8f0;margin-top:4px;">
                                            "Instance Setup"
                                        </div>
                                    </div>

                                    // Step list
                                    <nav style="flex:1;">
                                        {steps.iter().enumerate().map(|(i, step)| {
                                            let is_done    = step.is_complete;
                                            let is_curr    = i == current;
                                            let is_req     = step.is_required;
                                            let step_title = step.title.clone();

                                            view! {
                                                <div style=move || format!(
                                                    "display:flex;align-items:center;gap:12px;\
                                                     padding:10px 12px;border-radius:10px;\
                                                     margin-bottom:4px;cursor:default;transition:background .15s;{}",
                                                    if is_curr { "background:rgba(99,102,241,.12);" } else { "" }
                                                )>
                                                    <StepDot
                                                        is_complete=is_done
                                                        is_current=is_curr
                                                        is_required=is_req
                                                    />
                                                    <div>
                                                        <div style=move || format!(
                                                            "font-size:13px;font-weight:{};{}",
                                                            if is_curr { "600" } else { "400" },
                                                            if is_done { "color:#22c55e;" }
                                                            else if is_curr { "color:#e2e8f0;" }
                                                            else { "color:#475569;" }
                                                        )>
                                                            {step_title.clone()}
                                                        </div>
                                                        <div style="font-size:11px;color:#334155;margin-top:1px;">
                                                            {if is_done { "Complete" } else if is_req { "Required" } else { "Optional" }}
                                                        </div>
                                                    </div>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </nav>

                                    // Progress block
                                    <div style="margin-top:auto;padding-top:24px;\
                                                border-top:1px solid rgba(255,255,255,.06);">
                                        <div style="display:flex;justify-content:space-between;\
                                                    font-size:11px;color:#475569;margin-bottom:8px;">
                                            <span>"Progress"</span>
                                            <span>{format!("{}/{} complete", completed_count, total)}</span>
                                        </div>
                                        <div style="height:4px;background:rgba(255,255,255,.06);\
                                                    border-radius:4px;overflow:hidden;">
                                            <div style=format!(
                                                "height:100%;border-radius:4px;\
                                                 background:linear-gradient(90deg,#6366f1,#818cf8);\
                                                 transition:width .4s ease;width:{}%;",
                                                progress_pct
                                            )></div>
                                        </div>
                                    </div>
                                </aside>

                                // ── Main content area ─────────────────────────────
                                <div style="flex:1;display:flex;flex-direction:column;overflow:hidden;">

                                    // Top bar
                                    <div style="height:64px;flex-shrink:0;\
                                                border-bottom:1px solid rgba(255,255,255,.06);\
                                                display:flex;align-items:center;\
                                                justify-content:space-between;padding:0 32px;">
                                        <div style="display:flex;align-items:center;gap:10px;">
                                            <span style="font-size:13px;font-weight:600;color:#94a3b8;">
                                                {format!("Step {} of {}", current + 1, total)}
                                            </span>
                                            {current_step.as_ref().map(|step| view! {
                                                <span style="font-size:13px;color:#475569;">" — "</span>
                                                <span style="font-size:13px;font-weight:600;color:#e2e8f0;">{step.title.clone()}</span>
                                            })}
                                        </div>
                                        <button
                                            id="ob-dismiss"
                                            on:click=move |e| dismiss.get_value()(e)
                                            style="background:none;border:none;color:#475569;\
                                                   font-size:13px;cursor:pointer;\
                                                   text-decoration:underline;padding:6px 10px;\
                                                   transition:color .15s;"
                                        >
                                            "I'll finish this later"
                                        </button>
                                    </div>

                                    // Step content
                                    <div style="flex:1;overflow-y:auto;display:flex;\
                                                align-items:flex-start;justify-content:center;\
                                                padding:48px 32px;">
                                        <div style="width:100%;max-width:580px;">
                                            {match &current_step {
                                                None => view! {
                                                    <OnboardingComplete
                                                        app_instance_id=ai.clone()
                                                        tenant_id=tid.clone()
                                                    />
                                                }.into_any(),
                                                Some(step) => {
                                                    let step = step.clone();

                                                    // Step header with glassmorphic badge
                                                    let header = {
                                                        let req_label = if step.is_required { "Required" } else { "Optional" };
                                                        let (badge_bg, badge_color) = if step.is_required {
                                                            ("rgba(99,102,241,.15)", "#a5b4fc")
                                                        } else {
                                                            ("rgba(100,116,139,.12)", "#94a3b8")
                                                        };
                                                        view! {
                                                            <div style="margin-bottom:28px;">
                                                                <div style="display:flex;align-items:center;gap:8px;margin-bottom:10px;">
                                                                    <span style=format!(
                                                                        "font-size:11px;font-weight:700;\
                                                                         text-transform:uppercase;letter-spacing:.1em;\
                                                                         background:{};color:{};\
                                                                         padding:3px 10px;border-radius:20px;",
                                                                        badge_bg, badge_color
                                                                    )>
                                                                        {req_label}
                                                                    </span>
                                                                    {step.is_complete.then(|| view! {
                                                                        <span style="font-size:11px;font-weight:700;\
                                                                                     color:#22c55e;\
                                                                                     background:rgba(34,197,94,.1);\
                                                                                     padding:3px 10px;border-radius:20px;">
                                                                            "✓ Complete"
                                                                        </span>
                                                                    })}
                                                                </div>
                                                                <h2 style="font-size:26px;font-weight:800;\
                                                                           color:#f1f5f9;margin:0 0 8px;\
                                                                           letter-spacing:-.3px;">
                                                                    {step.title.clone()}
                                                                </h2>
                                                                <p style="font-size:14px;color:#64748b;line-height:1.65;margin:0;">
                                                                    {step.description.clone()}
                                                                </p>
                                                            </div>
                                                        }
                                                    };

                                                    let ai_s = ai.clone();
                                                    let tid_s = tid.clone();
                                                    let go_next_cb = Callback::new(move |_: ()| go_next());

                                                    // Glassmorphic card wrapping the step form
                                                    let step_body = match step.id.as_str() {
                                                        "identity" => view! {
                                                            <IdentityStep
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
                                                        <div>
                                                            {header}
                                                            // Glassmorphic form card
                                                            <div style="background:rgba(255,255,255,.03);\
                                                                        border:1px solid rgba(255,255,255,.08);\
                                                                        border-radius:16px;padding:28px;\
                                                                        backdrop-filter:blur(8px);">
                                                                {step_body}
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                }
                                            }}
                                        </div>
                                    </div>

                                    // Footer nav
                                    <div style="height:64px;flex-shrink:0;\
                                                border-top:1px solid rgba(255,255,255,.06);\
                                                display:flex;align-items:center;\
                                                justify-content:space-between;padding:0 32px;">
                                        <button
                                            style=move || format!(
                                                "background:rgba(255,255,255,.06);\
                                                 border:1px solid rgba(255,255,255,.1);\
                                                 color:#94a3b8;padding:10px 20px;border-radius:8px;\
                                                 font-size:13px;cursor:pointer;transition:opacity .2s;{}",
                                                if current_step_index.get() == 0 {
                                                    "opacity:.3;pointer-events:none;"
                                                } else { "" }
                                            )
                                            disabled=move || current_step_index.get() == 0
                                            on:click=go_prev
                                        >
                                            "← Back"
                                        </button>
                                        <span style="font-size:11px;color:#1e293b;">
                                            "Your progress is saved automatically."
                                        </span>
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
