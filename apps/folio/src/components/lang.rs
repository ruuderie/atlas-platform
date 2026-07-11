//! Language switching — native i18n support.
//!
//! # ⚠️ DEPRECATION NOTICE
//!
//! The canonical implementation of `Lang`, `get_current_lang`, `set_language`,
//! and `LanguageSwitcher` has been promoted to:
//!
//!   `apps/shared-ui/src/i18n/lang.rs`
//!
//! This Folio-local copy is kept as-is during Phase 8 migration while:
//!   1. `folio/Cargo.toml` is updated to import from `shared-ui::i18n`
//!   2. All marketing page component imports are updated
//!   3. The folio-local `provide_context::<Lang>()` injection is wired in
//!
//! Once Phase 8 is complete, DELETE this file and update all `use crate::components::lang::*`
//! imports to `use shared_ui::i18n::{Lang, LanguageSwitcher};`
//!
//! ## How it works
//!
//! 1. On SSR, the server reads the `folio_lang` cookie (user preference).
//!    If absent, it falls back to geo-detection via `CF-IPCountry` header.
//! 2. The resolved `Lang` is injected into Leptos context by each marketing page.
//! 3. `LanguageSwitcher` renders a compact dropdown in every marketing nav.
//!    When the user picks a language it calls `SetLanguage` server fn (writes
//!    the `folio_lang` cookie), then reloads the page so SSR re-renders with
//!    the new locale.
//!
//! ## Supported languages
//!
//! | Code | Language   | Markets      |
//! |------|------------|--------------|
//! | `en` | English    | US, Canada   |
//! | `pt` | Português  | Brazil       |
//! | `es` | Español    | LATAM        |
//! | `fr` | Français   | Quebec (CA)  |

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── Lang enum ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default, Copy)]
pub enum Lang {
    #[default]
    En,
    Pt,
    Es,
    Fr,
}

impl Lang {
    pub fn from_code(code: &str) -> Self {
        match code {
            "pt" => Lang::Pt,
            "es" => Lang::Es,
            "fr" => Lang::Fr,
            _ => Lang::En,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Pt => "pt",
            Lang::Es => "es",
            Lang::Fr => "fr",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Lang::En => "EN",
            Lang::Pt => "PT",
            Lang::Es => "ES",
            Lang::Fr => "FR",
        }
    }

    pub fn flag(&self) -> &'static str {
        match self {
            Lang::En => "🇺🇸",
            Lang::Pt => "🇧🇷",
            Lang::Es => "🌎",
            Lang::Fr => "🇨🇦",
        }
    }

    pub fn native_name(&self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Pt => "Português",
            Lang::Es => "Español",
            Lang::Fr => "Français",
        }
    }

    /// Infer language from a country code (geo fallback when no cookie set).
    pub fn from_country(country_code: &str) -> Self {
        match country_code {
            "BR" => Lang::Pt,
            "MX" | "CO" | "AR" | "CL" | "PE" | "EC" | "VE" | "UY" => Lang::Es,
            // Quebec detection would require city-level data; default CA to EN
            _ => Lang::En,
        }
    }
}

// ── Server fn: read current language ─────────────────────────────────────────

#[server(GetCurrentLang, "/api")]
pub async fn get_current_lang() -> Result<String, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers: HeaderMap = extract().await?;

    // 1. Check folio_lang cookie
    if let Some(cookie_hdr) = headers.get("cookie").and_then(|v| v.to_str().ok()) {
        for part in cookie_hdr.split(';') {
            let part = part.trim();
            if let Some(val) = part.strip_prefix("folio_lang=") {
                let code = val.trim();
                if matches!(code, "en" | "pt" | "es" | "fr") {
                    return Ok(code.to_string());
                }
            }
        }
    }

    // 2. Fall back to Cloudflare geo
    let country = headers
        .get("CF-IPCountry")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("US");

    Ok(Lang::from_country(country).code().to_string())
}

// ── Server fn: set language cookie ────────────────────────────────────────────

#[server(SetLanguage, "/api")]
pub async fn set_language(lang_code: String) -> Result<(), ServerFnError> {
    use axum::http::HeaderValue;
    use leptos_axum::ResponseOptions;

    if !matches!(lang_code.as_str(), "en" | "pt" | "es" | "fr") {
        return Err(ServerFnError::ServerError("Invalid language code".into()));
    }

    let opts = expect_context::<ResponseOptions>();
    // SameSite=Lax; 1-year expiry; works on all marketing pages
    let cookie = format!(
        "folio_lang={}; Path=/; Max-Age=31536000; SameSite=Lax",
        lang_code
    );
    opts.insert_header(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_str(&cookie).unwrap(),
    );

    Ok(())
}

// ── LanguageSwitcher component ────────────────────────────────────────────────

/// Compact globe dropdown for the marketing nav.
///
/// Usage in any nav:
/// ```ignore
/// <LanguageSwitcher current_lang="en".to_string() />
/// ```
///
/// The `current_lang` prop should be read from the `folio_lang` cookie on the
/// server side and passed in at render time (or read via `GetCurrentLang`
/// resource).
#[component]
pub fn LanguageSwitcher(
    /// The currently active language code ("en", "pt", "es", "fr").
    #[prop(default = "en".to_string())]
    current_lang: String,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let current = Lang::from_code(&current_lang);

    let options = [Lang::En, Lang::Pt, Lang::Es, Lang::Fr];

    view! {
        <div class="lang-switcher" style="position:relative;">
            // ── Trigger button ─────────────────────────────────────
            <button
                class="lang-switcher-btn"
                id="lang-switcher-toggle"
                aria-label="Switch language"
                aria-expanded=move || open.get().to_string()
                aria-haspopup="listbox"
                on:click=move |_| open.update(|o| *o = !*o)
            >
                <span style="font-size:.9rem;">{current.flag()}</span>
                <span style="font-size:.78rem;font-weight:600;letter-spacing:.03em;">{current.label()}</span>
                <span class="material-symbols-outlined" style="font-size:14px;opacity:.6;transition:transform .15s;"
                      style:transform=move || if open.get() { "rotate(180deg)" } else { "rotate(0)" }
                >"expand_more"</span>
            </button>

            // ── Dropdown ───────────────────────────────────────────
            <Show when=move || open.get() fallback=|| ()>
                // Backdrop — click outside to close
                <div
                    style="position:fixed;inset:0;z-index:199;"
                    on:click=move |_| open.set(false)
                ></div>

                <div class="lang-switcher-dropdown" role="listbox" aria-label="Select language">
                    {options.iter().map(|lang| {
                        let code  = lang.code();
                        let flag  = lang.flag();
                        let label = lang.native_name();
                        let is_active = *lang == current;
                        view! {
                            <button
                                class=if is_active {
                                    "lang-option lang-option--active"
                                } else {
                                    "lang-option"
                                }
                                role="option"
                                aria-selected=is_active.to_string()
                                on:click=move |_| {
                                    open.set(false);
                                    leptos::task::spawn_local(async move {
                                        let _ = set_language(code.to_string()).await;
                                        // Reload the page so SSR re-renders with the new locale
                                        if let Some(win) = web_sys::window() {
                                            let _ = win.location().reload();
                                        }
                                    });
                                }
                            >
                                <span style="font-size:1rem;">{flag}</span>
                                <span style="font-size:.85rem;">{label}</span>
                                <Show when=move || is_active fallback=|| ()>
                                    <span class="material-symbols-outlined" style="font-size:13px;color:#06d6a0;margin-left:auto;font-variation-settings:'FILL' 1">"check"</span>
                                </Show>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </Show>
        </div>
    }
}
