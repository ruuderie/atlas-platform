//! Lang — supported languages, detection, cookie persistence, and switcher UI.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── Lang enum ─────────────────────────────────────────────────────────────────

/// Supported UI languages across all Atlas apps.
///
/// Add new variants here when a new locale is ready for production.
/// Each variant must have entries in every `.ftl` file before it can be exposed
/// in the `LanguageSwitcher` dropdown.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default, Copy)]
pub enum Lang {
    /// English — United States, Canada (default)
    #[default]
    En,
    /// Português Brasileiro — Brazil
    Pt,
    /// Español — LATAM (Mexico, Colombia, Argentina, Chile, Peru, Ecuador…)
    Es,
    /// Français — Quebec, Canada (future — not yet in .ftl files)
    Fr,
}

impl Lang {
    /// Parse a language code string into a `Lang` variant.
    /// Unknown codes fall back to `En`.
    pub fn from_code(code: &str) -> Self {
        match code {
            "pt" => Lang::Pt,
            "es" => Lang::Es,
            "fr" => Lang::Fr,
            _ => Lang::En,
        }
    }

    /// BCP 47 language tag (used in `<html lang="...">` and HTTP headers).
    pub fn code(&self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Pt => "pt",
            Lang::Es => "es",
            Lang::Fr => "fr",
        }
    }

    /// Short uppercase label shown in the nav switcher button.
    pub fn label(&self) -> &'static str {
        match self {
            Lang::En => "EN",
            Lang::Pt => "PT",
            Lang::Es => "ES",
            Lang::Fr => "FR",
        }
    }

    /// Emoji flag for the switcher button.
    pub fn flag(&self) -> &'static str {
        match self {
            Lang::En => "🇺🇸",
            Lang::Pt => "🇧🇷",
            Lang::Es => "🌎",
            Lang::Fr => "🇨🇦",
        }
    }

    /// Native language name shown in the dropdown options.
    pub fn native_name(&self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Pt => "Português",
            Lang::Es => "Español",
            Lang::Fr => "Français",
        }
    }

    /// Infer language from a Cloudflare `CF-IPCountry` code.
    /// Used as a geo fallback when no `folio_lang` cookie is set.
    pub fn from_country(country_code: &str) -> Self {
        match country_code {
            "BR" => Lang::Pt,
            "MX" | "CO" | "AR" | "CL" | "PE" | "EC" | "VE" | "UY" | "PY" | "BO" | "GT" | "HN"
            | "SV" | "NI" | "CR" | "PA" | "DO" | "CU" => Lang::Es,
            // Quebec detection requires city-level data; default CA to EN for now.
            // When FR translations are ready: match "CA" with city "Quebec City" etc.
            _ => Lang::En,
        }
    }

    /// All languages currently exposed to end users.
    /// `Fr` is omitted until translations are complete.
    pub fn available() -> &'static [Lang] {
        &[Lang::En, Lang::Pt, Lang::Es]
    }
}

// ── Server fn: read current language ─────────────────────────────────────────

/// Resolves the current language for an SSR request.
///
/// Resolution order:
///   1. `folio_lang` cookie (explicit user preference — overrides everything)
///   2. `CF-IPCountry` Cloudflare geo header (edge-injected, zero-latency)
///   3. Default: `"en"`
///
/// Returns the BCP 47 language code string (e.g. `"en"`, `"pt"`, `"es"`).
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

    // 2. Fall back to Cloudflare CF-IPCountry geo header
    let country = headers
        .get("CF-IPCountry")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("US");

    Ok(Lang::from_country(country).code().to_string())
}

// ── Server fn: set language cookie ───────────────────────────────────────────

/// Persists the user's language preference as a 1-year `folio_lang` cookie.
///
/// The cookie is scoped to `Path=/` so it applies to all Atlas apps running
/// under the same domain (folio, folio-pm, folio-broker, etc.).
///
/// After calling this, the client should `window.location.reload()` so SSR
/// re-renders the page in the new locale.
#[server(SetLanguage, "/api")]
pub async fn set_language(lang_code: String) -> Result<(), ServerFnError> {
    use axum::http::HeaderValue;
    use leptos_axum::ResponseOptions;

    if !matches!(lang_code.as_str(), "en" | "pt" | "es" | "fr") {
        return Err(ServerFnError::ServerError("Invalid language code".into()));
    }

    let opts = expect_context::<ResponseOptions>();
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

/// Compact language switcher for the marketing nav.
///
/// Renders a `🌎 EN ▾` button that opens a dropdown with all available languages.
/// Selecting a language calls [`set_language`] and reloads the page.
///
/// # Usage
///
/// ```rust
/// // Read current lang from server fn and pass it in:
/// let lang_res = Resource::new(|| (), |_| get_current_lang());
/// view! {
///     <Suspense fallback=|| ()>
///         {move || lang_res.get().and_then(|r| r.ok()).map(|code| view! {
///             <LanguageSwitcher current_lang=code/>
///         })}
///     </Suspense>
/// }
/// ```
#[component]
pub fn LanguageSwitcher(
    /// BCP 47 language code of the currently active language ("en", "pt", "es").
    #[prop(default = "en".to_string())]
    current_lang: String,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let current = Lang::from_code(&current_lang);

    view! {
        <div class="lang-switcher" style="position:relative;">
            // ── Trigger button ─────────────────────────────────────────────
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
                <span
                    class="material-symbols-outlined"
                    style="font-size:14px;opacity:.6;transition:transform .15s;"
                    style:transform=move || if open.get() { "rotate(180deg)" } else { "rotate(0)" }
                >"expand_more"</span>
            </button>

            // ── Dropdown ───────────────────────────────────────────────────
            {move || open.get().then(|| view! {
                // Backdrop — click outside to close
                <div
                    style="position:fixed;inset:0;z-index:199;"
                    on:click=move |_| open.set(false)
                ></div>

                <div
                    class="lang-switcher-dropdown"
                    role="listbox"
                    aria-label="Select language"
                >
                    {Lang::available().iter().map(|lang| {
                        let code      = lang.code();
                        let flag      = lang.flag();
                        let label     = lang.native_name();
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
                                        if let Some(win) = web_sys::window() {
                                            let _ = win.location().reload();
                                        }
                                    });
                                }
                            >
                                <span style="font-size:1rem;">{flag}</span>
                                <span style="font-size:.85rem;">{label}</span>
                                {is_active.then(|| view! {
                                    <span
                                        class="material-symbols-outlined"
                                        style="font-size:13px;color:#06d6a0;margin-left:auto;\
                                               font-variation-settings:'FILL' 1"
                                    >"check"</span>
                                })}
                            </button>
                        }
                    }).collect_view()}
                </div>
            })}
        </div>
    }
}
