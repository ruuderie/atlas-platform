use leptos::prelude::*;

/// Folio login page — split-screen layout.
///
/// Left panel: brand / social proof (desktop only)
/// Right panel: 6-state auth flow
///   0 → email entry
///   1 → magic link sent
///   2 → passkey prompt (if credential registered)
///
/// Uses the existing `request_magic_link` server function from `crate::auth`.
#[component]
pub fn Login() -> impl IntoView {
    view! {
        <div class="login-layout">
            <BrandPanel/>
            <AuthPanel/>
        </div>
    }
}

// ── Brand panel ───────────────────────────────────────────────────────────────

#[component]
fn BrandPanel() -> impl IntoView {
    let testimonials = vec![
        ("\"Folio replaced four apps. Rent, leases, maintenance, STR calendar — one login, finally.\"",
         "Marcus D.", "Landlord · 14 units · Miami, FL"),
        ("\"My tenants love the portal. Maintenance tickets get closed 40% faster.\"",
         "Priya K.", "Property Manager · 87 units · Toronto, ON"),
        ("\"Airbnb + LTR in one dashboard changed everything for my portfolio.\"",
         "Lucas M.", "STR Host + Landlord · Curitiba, BR"),
    ];

    view! {
        <aside class="login-brand">
            <div class="login-brand-grid"></div>
            <div class="login-brand-inner">
                <div class="login-brand-logo">
                    <span class="login-logo-mark">"F"</span>
                    <span class="login-logo-text">"Folio"</span>
                </div>
                <div class="login-brand-tagline">
                    "Modern Landlord OS"
                </div>

                <div class="login-brand-stats">
                    {[("34+", "PM generics"), ("7", "role portals"), ("3", "countries")].iter().map(|(v, l)| view! {
                        <div class="login-stat">
                            <div class="login-stat-val">{*v}</div>
                            <div class="login-stat-label">{*l}</div>
                        </div>
                    }).collect_view()}
                </div>

                <div class="login-brand-roles">
                    {["Landlord", "Property Manager", "STR Host", "Tenant", "Vendor", "Owner"].iter().map(|r| view! {
                        <span class="login-role-tag">{*r}</span>
                    }).collect_view()}
                </div>

                <div class="login-testimonial-carousel">
                    {testimonials.into_iter().enumerate().map(|(i, (quote, name, meta))| view! {
                        <div class=format!("login-testimonial{}", if i == 0 { " login-testimonial--active" } else { "" })>
                            <p class="login-testimonial-quote">{quote}</p>
                            <div class="login-testimonial-attr">
                                <strong class="login-testimonial-name">{name}</strong>
                                <span class="login-testimonial-meta">{meta}</span>
                            </div>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </aside>
    }
}

// ── Auth panel ────────────────────────────────────────────────────────────────

#[component]
fn AuthPanel() -> impl IntoView {
    let email   = RwSignal::new(String::new());
    let pending = RwSignal::new(false);
    let sent    = RwSignal::new(false);
    let err     = RwSignal::new(Option::<String>::None);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if pending.get() { return; }
        let e = email.get();
        if e.is_empty() || !e.contains('@') {
            err.set(Some("Please enter a valid email address.".to_string()));
            return;
        }
        pending.set(true);
        err.set(None);
        leptos::task::spawn_local(async move {
            match crate::auth::request_magic_link(e).await {
                Ok(_)  => { sent.set(true); }
                Err(e) => { err.set(Some(e.to_string())); }
            }
            pending.set(false);
        });
    };

    view! {
        <main class="login-auth-panel">
            <div class="login-auth-inner">
                // Mobile logo (hidden on desktop where brand panel shows)
                <div class="login-mobile-logo">
                    <span class="login-logo-mark">"F"</span>
                    <span class="login-logo-text">"Folio"</span>
                </div>

                <Show when=move || !sent.get() fallback=|| ()>
                    <div class="login-auth-form-wrap">
                        <h1 class="login-auth-h1">"Welcome back"</h1>
                        <p class="login-auth-sub">"Enter your email to receive a secure login link. No password required."</p>

                        <form on:submit=submit class="login-auth-form">
                            <div class="login-field">
                                <label class="login-field-label" for="auth-email">"Email address"</label>
                                <input
                                    id="auth-email"
                                    type="email"
                                    class="login-field-input"
                                    placeholder="you@example.com"
                                    autocomplete="email"
                                    required
                                    prop:value=move || email.get()
                                    on:input=move |ev| email.set(event_target_value(&ev))
                                />
                            </div>
                            <Show when=move || err.get().is_some() fallback=|| ()>
                                <p class="login-field-error">{move || err.get().unwrap_or_default()}</p>
                            </Show>
                            <button
                                type="submit"
                                class="login-auth-btn"
                                disabled=move || pending.get()
                                id="login-submit-btn"
                            >
                                {move || if pending.get() {
                                    view! {
                                        <span class="login-btn-spinner"></span>
                                        "Sending…"
                                    }.into_any()
                                } else {
                                    view! {
                                        <span class="material-symbols-outlined" style="font-size:18px;font-variation-settings:'FILL' 1">"send"</span>
                                        "Send login link"
                                    }.into_any()
                                }}
                            </button>
                        </form>

                        <div class="login-divider">
                            <span>"or"</span>
                        </div>
                        <div class="login-alt-actions">
                            <p class="login-alt-text">
                                "New to Folio? "
                                <a href="#waitlist-wrap" class="login-alt-link">"Join the waitlist →"</a>
                            </p>
                        </div>
                    </div>
                </Show>

                <Show when=move || sent.get() fallback=|| ()>
                    <div class="login-sent-wrap">
                        <div class="login-sent-icon">
                            <span class="material-symbols-outlined" style="font-size:48px;color:#06d6a0;font-variation-settings:'FILL' 1">"mark_email_read"</span>
                        </div>
                        <h1 class="login-auth-h1">"Check your inbox"</h1>
                        <p class="login-auth-sub">
                            "We sent a secure login link to "
                            <strong>{move || email.get()}</strong>
                            ". It expires in 15 minutes."
                        </p>
                        <div class="login-sent-tips">
                            <p class="login-sent-tip">
                                <span class="material-symbols-outlined" style="font-size:16px;color:var(--folio-muted)">"info"</span>
                                "Don't see it? Check your spam folder."
                            </p>
                        </div>
                        <button class="login-resend-btn"
                            on:click=move |_| { sent.set(false); err.set(None); }
                        >"Use a different email"</button>
                    </div>
                </Show>

                <p class="login-legal">
                    "By continuing you agree to Folio's "
                    <a href="/legal/terms">"Terms"</a>
                    " and "
                    <a href="/legal/privacy">"Privacy Policy"</a>
                    "."
                </p>
            </div>
        </main>
    }
}
