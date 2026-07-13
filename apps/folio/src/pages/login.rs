use leptos::prelude::*;
use std::time::Duration;

/// Folio login — dark-harmonized split screen matching public marketing pages.
///
/// Guided auth screens (from stitch `pub_login_v3`):
///   Entry → PasskeyPrompt | MagicRequest → MagicSent
///
/// Passkey registration after first magic-link verify lives at `/auth/passkey-setup`.
///
/// Only one screen is mounted at a time (`Show` / match). Do not stack all screens
/// with CSS `display:none` toggles — BEM `--active` class bindings are fragile in
/// the Leptos view macro and previously left every step visible at once.
#[component]
pub fn Login() -> impl IntoView {
    view! {
        <div class="login-layout">
            <BrandPanel/>
            <AuthPanel/>
        </div>
    }
}

/// Closed set of auth panel screens — typed at the domain boundary.
#[derive(Clone, Copy, PartialEq, Eq)]
enum LoginScreen {
    Entry,
    PasskeyPrompt,
    MagicRequest,
    MagicSent,
}

fn is_valid_email(email: &str) -> bool {
    let email = email.trim();
    !email.is_empty() && email.contains('@') && email.contains('.')
}

// ── Brand panel ───────────────────────────────────────────────────────────────

#[component]
fn BrandPanel() -> impl IntoView {
    view! {
        <aside class="login-brand">
            <div class="login-brand-grid"></div>
            <div class="login-brand-glow"></div>
            <div class="login-brand-inner">
                <a href="/" class="login-brand-logo">
                    <span class="login-logo-mark">"F"</span>
                    <span class="login-logo-text">"Folio"</span>
                </a>
                <p class="login-brand-tagline">"Your portfolio. Finally under control."</p>

                <div class="login-brand-hero-copy">
                    <p class="login-brand-headline">
                        "Run your properties"
                        <br/>
                        "like the business"
                        <br/>
                        <em>"they actually are."</em>
                    </p>
                    <p class="login-brand-subline">
                        "One platform for rent collection, leases, maintenance, vacation rentals, and compliance — built for serious operators."
                    </p>
                </div>

                <div class="login-brand-stats">
                    {[("40%", "Faster maintenance close"), ("1", "Login for your whole portfolio"), ("3", "Countries — US · CA · BR")].iter().map(|(v, l)| view! {
                        <div class="login-stat">
                            <div class="login-stat-val">{*v}</div>
                            <div class="login-stat-label">{*l}</div>
                        </div>
                    }).collect_view()}
                </div>

                <div class="login-brand-roles">
                    {["Landlords", "Property Managers", "STR Hosts", "Brokers", "Vendors"].iter().map(|r| view! {
                        <span class="login-role-tag">{*r}</span>
                    }).collect_view()}
                </div>

                <div class="login-brand-footer">
                    <p class="login-testimonial-quote">
                        "\"Folio replaced four apps. Rent, leases, maintenance, STR calendar — one login, finally.\""
                    </p>
                    <strong class="login-testimonial-name">"Marcus D."</strong>
                    <span class="login-testimonial-meta">"Landlord · 14 units · Miami, FL"</span>
                </div>
            </div>
        </aside>
    }
}

// ── Auth panel ────────────────────────────────────────────────────────────────

#[component]
fn AuthPanel() -> impl IntoView {
    let screen = RwSignal::new(LoginScreen::Entry);
    let email = RwSignal::new(String::new());
    let pending = RwSignal::new(false);
    let err = RwSignal::new(Option::<String>::None);
    let passkey_pending = RwSignal::new(false);
    let countdown = RwSignal::new(900u32);
    let navigate = StoredValue::new(leptos_router::hooks::use_navigate());

    // Live countdown while on MagicSent
    Effect::new(move |_| {
        if screen.get() != LoginScreen::MagicSent {
            return;
        }
        countdown.set(900);
        #[cfg(feature = "hydrate")]
        {
            let handle = set_interval_with_handle(
                move || {
                    countdown.update(|s| {
                        if *s > 0 {
                            *s -= 1;
                        }
                    });
                },
                Duration::from_secs(1),
            )
            .ok();
            on_cleanup(move || {
                if let Some(h) = handle {
                    h.clear();
                }
            });
        }
    });

    let go_entry = move |_| {
        screen.set(LoginScreen::Entry);
        pending.set(false);
        passkey_pending.set(false);
        err.set(None);
    };

    let start_passkey = move |_| {
        if passkey_pending.get() {
            return;
        }
        passkey_pending.set(true);
        err.set(None);
        screen.set(LoginScreen::PasskeyPrompt);
        leptos::task::spawn_local(async move {
            match crate::utils::passkey_js::authenticate_passkey().await {
                Ok(_) => match crate::auth::get_session().await {
                    Ok(info) => {
                        let dest = if !info.has_passkey {
                            "/auth/passkey-setup"
                        } else if !info.onboarding_complete {
                            "/onboarding"
                        } else {
                            "/dashboard"
                        };
                        navigate.with_value(|n| n(dest, Default::default()));
                    }
                    Err(e) => {
                        err.set(Some(e.to_string()));
                        passkey_pending.set(false);
                    }
                },
                Err(e) => {
                    err.set(Some(e));
                    passkey_pending.set(false);
                }
            }
        });
    };

    let retry_passkey = move |_| {
        if passkey_pending.get() {
            return;
        }
        err.set(None);
        passkey_pending.set(true);
        leptos::task::spawn_local(async move {
            match crate::utils::passkey_js::authenticate_passkey().await {
                Ok(_) => match crate::auth::get_session().await {
                    Ok(info) => {
                        let dest = if !info.has_passkey {
                            "/auth/passkey-setup"
                        } else if !info.onboarding_complete {
                            "/onboarding"
                        } else {
                            "/dashboard"
                        };
                        navigate.with_value(|n| n(dest, Default::default()));
                    }
                    Err(e) => {
                        err.set(Some(e.to_string()));
                        passkey_pending.set(false);
                    }
                },
                Err(e) => {
                    err.set(Some(e));
                    passkey_pending.set(false);
                }
            }
        });
    };

    let continue_email = move || {
        let e = email.get().trim().to_string();
        email.set(e.clone());
        if !is_valid_email(&e) {
            err.set(Some("Please enter a valid email address.".to_string()));
            return;
        }
        err.set(None);
        screen.set(LoginScreen::MagicRequest);
    };

    let send_magic_link = move |_| {
        if pending.get() {
            return;
        }
        let e = email.get().trim().to_string();
        email.set(e.clone());
        if !is_valid_email(&e) {
            err.set(Some("Please enter a valid email address.".to_string()));
            screen.set(LoginScreen::Entry);
            return;
        }
        pending.set(true);
        err.set(None);
        leptos::task::spawn_local(async move {
            match crate::auth::request_magic_link(e).await {
                Ok(_) => {
                    screen.set(LoginScreen::MagicSent);
                }
                Err(e) => {
                    err.set(Some(e.to_string()));
                }
            }
            pending.set(false);
        });
    };

    let use_magic_instead = move |_| {
        passkey_pending.set(false);
        err.set(None);
        let e = email.get();
        if is_valid_email(&e) {
            screen.set(LoginScreen::MagicRequest);
        } else {
            screen.set(LoginScreen::Entry);
        }
    };

    let countdown_label = move || {
        let s = countdown.get();
        format!("{}:{:02}", s / 60, s % 60)
    };
    let countdown_pct = move || {
        let s = countdown.get();
        format!("width:{}%", (s as f32 / 900.0) * 100.0)
    };

    // Mount the email <input> only after hydrate. Password-manager / Apple
    // "Hide My Email" extensions inject sibling nodes into type=email fields
    // and break tachys if that input is in the SSR tree. Keep Continue (and the
    // rest of the form) always present so login stays usable.
    let email_field_ready = RwSignal::new(false);
    Effect::new(move |_| {
        email_field_ready.set(true);
    });

    view! {
        <main class="login-auth-panel">
            <div class="login-auth-inner">
                <div class="login-mobile-logo">
                    <span class="login-logo-mark">"F"</span>
                    <span class="login-logo-text">"Folio"</span>
                </div>

                {move || match screen.get() {
                    LoginScreen::Entry => view! {
                        <div class="login-screen login-screen--active">
                            <h1 class="login-auth-h1">"Welcome back"</h1>
                            <p class="login-auth-sub">"Sign in with a passkey or get a secure email link. No password required."</p>

                            <button
                                type="button"
                                class="login-passkey-btn"
                                id="login-passkey-btn"
                                disabled=move || passkey_pending.get()
                                on:click=start_passkey
                            >
                                <span class="login-passkey-icon-box" aria-hidden="true">
                                    <span class="material-symbols-outlined login-icon-fill">"fingerprint"</span>
                                </span>
                                <span class="login-passkey-btn-text">
                                    <span class="login-passkey-btn-label">"Sign in with a passkey"</span>
                                    <span class="login-passkey-btn-sub">"Face ID, Touch ID, or device PIN"</span>
                                </span>
                                <span class="material-symbols-outlined login-passkey-chevron" aria-hidden="true">"chevron_right"</span>
                            </button>

                            <div class="login-divider"><span>"or"</span></div>

                            <div class="login-auth-form">
                                <div class="login-field">
                                    <label class="login-field-label" for="auth-email">"Email address"</label>
                                    <Show
                                        when=move || email_field_ready.get()
                                        fallback=|| view! {
                                            <div
                                                class="login-field-input"
                                                style="min-height:2.75rem;box-sizing:border-box"
                                                aria-hidden="true"
                                            ></div>
                                        }
                                    >
                                        <input
                                            id="auth-email"
                                            type="text"
                                            inputmode="email"
                                            class="login-field-input"
                                            placeholder="you@example.com"
                                            autocomplete="username"
                                            spellcheck="false"
                                            prop:value=move || email.get()
                                            on:input=move |ev| {
                                                email.set(event_target_value(&ev));
                                                if err.get().is_some() {
                                                    err.set(None);
                                                }
                                            }
                                            on:keydown=move |ev| {
                                                if ev.key() == "Enter" {
                                                    ev.prevent_default();
                                                    continue_email();
                                                }
                                            }
                                        />
                                    </Show>
                                </div>
                                <Show when=move || err.get().is_some()>
                                    <p class="login-field-error login-field-error--show">
                                        {move || err.get().unwrap_or_default()}
                                    </p>
                                </Show>
                                <button
                                    type="button"
                                    class="login-auth-btn"
                                    id="login-submit-btn"
                                    on:click=move |_| continue_email()
                                >
                                    "Continue"
                                    <span class="material-symbols-outlined" style="font-size:18px" aria-hidden="true">"arrow_forward"</span>
                                </button>
                            </div>

                            <div class="login-alt-actions">
                                <p class="login-alt-text">
                                    "New to Folio? "
                                    <a href="/#waitlist-wrap" class="login-alt-link">"Join the waitlist →"</a>
                                </p>
                            </div>
                            <p class="login-legal">
                                "By continuing you agree to Folio's "
                                <a href="/legal/terms">"Terms"</a>
                                " and "
                                <a href="/legal/privacy">"Privacy Policy"</a>
                                "."
                            </p>
                        </div>
                    }.into_any(),

                    LoginScreen::PasskeyPrompt => view! {
                        <div class="login-screen login-screen--active">
                            <button type="button" class="login-back-btn" on:click=go_entry>
                                <span class="material-symbols-outlined" style="font-size:16px" aria-hidden="true">"arrow_back"</span>
                                " Back"
                            </button>
                            <div class="login-passkey-prompt">
                                <div class="login-passkey-ring-wrap">
                                    <div class="login-passkey-ring">
                                        <span class="material-symbols-outlined login-icon-fill" style="font-size:40px" aria-hidden="true">"fingerprint"</span>
                                    </div>
                                    <div class="login-passkey-ring-badge">
                                        <span class="material-symbols-outlined login-icon-fill" style="font-size:13px" aria-hidden="true">"fingerprint"</span>
                                    </div>
                                </div>
                                <h1 class="login-auth-h1">"Authenticate with your passkey"</h1>
                                <p class="login-auth-sub">
                                    {move || if err.get().is_some() {
                                        "Authentication didn't complete. Try again, or use a magic link."
                                    } else {
                                        "Your device will prompt you for Face ID, Touch ID, or PIN."
                                    }}
                                </p>
                                <Show when=move || passkey_pending.get()>
                                    <div class="login-dots" aria-hidden="true">
                                        <span class="login-dot login-dot--1"></span>
                                        <span class="login-dot login-dot--2"></span>
                                        <span class="login-dot login-dot--3"></span>
                                    </div>
                                </Show>
                                <Show when=move || err.get().is_some()>
                                    <p class="login-field-error login-field-error--show">
                                        {move || err.get().unwrap_or_default()}
                                    </p>
                                </Show>
                            </div>
                            <button
                                type="button"
                                class="login-auth-btn login-auth-btn--green"
                                disabled=move || passkey_pending.get()
                                on:click=retry_passkey
                            >
                                <span class="material-symbols-outlined login-icon-fill" style="font-size:18px" aria-hidden="true">"fingerprint"</span>
                                {move || if passkey_pending.get() { "Waiting for device..." } else { "Authenticate with Passkey" }}
                            </button>
                            <button type="button" class="login-btn-text" on:click=use_magic_instead>
                                <span class="material-symbols-outlined" style="font-size:14px;vertical-align:middle;margin-right:0.25rem" aria-hidden="true">"mail"</span>
                                "Use a magic link instead"
                            </button>
                        </div>
                    }.into_any(),

                    LoginScreen::MagicRequest => view! {
                        <div class="login-screen login-screen--active">
                            <button type="button" class="login-back-btn" on:click=go_entry>
                                <span class="material-symbols-outlined" style="font-size:16px" aria-hidden="true">"arrow_back"</span>
                                " Back"
                            </button>
                            <h1 class="login-auth-h1">"Sign in with email"</h1>
                            <p class="login-auth-sub">
                                "We'll send a secure sign-in link to "
                                <strong>{move || email.get()}</strong>
                                ". It expires in 15 minutes."
                            </p>
                            <div class="login-email-preview" aria-hidden="true">
                                <div class="login-email-preview-hdr">
                                    <div class="login-email-icon">"F"</div>
                                    <div>
                                        <div class="login-email-subj">"Sign in to Folio"</div>
                                        <div class="login-email-from">"no-reply@folio.app"</div>
                                    </div>
                                </div>
                                <div class="login-email-body">
                                    <p>"Click the link below to sign in securely. This link expires in 15 minutes and can only be used once."</p>
                                    <span class="login-email-cta">
                                        <span class="material-symbols-outlined" style="font-size:13px">"login"</span>
                                        " Log In Now"
                                    </span>
                                </div>
                            </div>
                            <Show when=move || err.get().is_some()>
                                <p class="login-field-error login-field-error--show">
                                    {move || err.get().unwrap_or_default()}
                                </p>
                            </Show>
                            <button
                                type="button"
                                class="login-auth-btn"
                                id="login-send-magic"
                                disabled=move || pending.get()
                                on:click=send_magic_link
                            >
                                <Show
                                    when=move || !pending.get()
                                    fallback=|| view! {
                                        <span class="login-btn-content" style="display:inline-flex;align-items:center;gap:8px">
                                            <span class="login-btn-spinner"></span>
                                            "Sending..."
                                        </span>
                                    }
                                >
                                    <span class="login-btn-content" style="display:inline-flex;align-items:center;gap:8px">
                                        <span class="material-symbols-outlined" style="font-size:18px" aria-hidden="true">"send"</span>
                                        "Send magic link"
                                    </span>
                                </Show>
                            </button>
                        </div>
                    }.into_any(),

                    LoginScreen::MagicSent => view! {
                        <div class="login-screen login-screen--active login-screen--center">
                            <div class="login-status-circle login-status-circle--green">
                                <span class="material-symbols-outlined login-icon-fill" style="font-size:40px" aria-hidden="true">"mark_email_read"</span>
                            </div>
                            <h1 class="login-auth-h1">"Check your email"</h1>
                            <p class="login-auth-sub" style="margin-bottom:0.35rem">"We sent a sign-in link to"</p>
                            <p class="login-sent-email">{move || email.get()}</p>
                            <div class="login-cd-wrap">
                                <div class="login-cd-hdr">
                                    <span>"Link valid for "</span>
                                    <span class="login-cd-time">{move || countdown_label()}</span>
                                </div>
                                <div class="login-cd-track">
                                    <div class="login-cd-fill" style=move || countdown_pct()></div>
                                </div>
                            </div>
                            <div class="login-stack-gap">
                                <button
                                    type="button"
                                    class="login-resend-btn"
                                    disabled=move || pending.get()
                                    on:click=send_magic_link
                                >
                                    <span class="material-symbols-outlined" style="font-size:16px" aria-hidden="true">"refresh"</span>
                                    " Resend link"
                                </button>
                                <button type="button" class="login-btn-text" on:click=go_entry>
                                    "Use a different email"
                                </button>
                            </div>
                        </div>
                    }.into_any(),
                }}
            </div>
        </main>
    }
}
