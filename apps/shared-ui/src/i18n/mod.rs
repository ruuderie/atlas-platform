//! Atlas i18n — Platform-level language detection and switching.
//!
//! # Architecture
//!
//! This module lives in `shared-ui` so every Atlas app (Folio, Folio PM,
//! Folio Broker, future apps) gets the same language infrastructure without
//! duplicating server functions or cookie logic.
//!
//! ## What's here
//!
//! | Item                  | Description                                         |
//! |-----------------------|-----------------------------------------------------|
//! | [`Lang`]              | Enum of supported languages (En/Pt/Es/Fr)           |
//! | [`get_current_lang`]  | Server fn — reads cookie, falls back to CF geo      |
//! | [`set_language`]      | Server fn — writes 1-year `folio_lang` cookie       |
//! | [`LanguageSwitcher`]  | Globe dropdown nav component                        |
//! | [`format`]            | Locale-aware currency, date, and phone formatting   |
//!
//! ## Cookie
//! Cookie name: `folio_lang`  
//! Path: `/` (valid for all Atlas apps on the same domain)  
//! Max-Age: 31536000 (1 year)  
//! SameSite: `Lax`
//!
//! ## Geo fallback
//! When no cookie is set, language is inferred from the Cloudflare `CF-IPCountry`
//! header injected at the edge on every SSR request.
//!
//! | Country codes         | Language  |
//! |-----------------------|-----------|
//! | `BR`                  | Pt        |
//! | `MX CO AR CL PE EC…` | Es        |
//! | `CA` (+ city Quebec)  | En (Fr in future) |
//! | Everything else       | En        |
//!
//! ## Usage in any Atlas app
//!
//! ```rust
//! // In page root — resolve + provide lang context once:
//! use shared_ui::i18n::{Lang, get_current_lang};
//!
//! let lang_res = Resource::new(|| (), |_| get_current_lang());
//! // provide_context(resolved_lang);
//!
//! // In any child component:
//! use shared_ui::i18n::{LanguageSwitcher};
//! // <LanguageSwitcher current_lang="en".to_string() />
//! ```

pub mod lang;
pub mod format;

pub use lang::{Lang, get_current_lang, set_language, LanguageSwitcher};
pub use format::{format_currency, format_date_short};
