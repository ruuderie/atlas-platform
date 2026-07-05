//! Locale-aware formatting for currency, dates, and phone numbers.
//!
//! All formatting is done in pure Rust — no JS `Intl` dependency — so it
//! works identically on the server (SSR) and client (WASM hydration).
//!
//! # Currency
//!
//! | Lang | Symbol | Format example |
//! |------|--------|----------------|
//! | En   | $      | $1,285.00      |
//! | Pt   | R$     | R$ 1.285,00    |
//! | Es   | varies | $1,285 MXN     |
//! | Fr   | CA$    | CA$ 1 285,00   |
//!
//! # Dates
//!
//! Short format used in UI chips, profile stats, job history:
//!
//! | Lang | Format         | Example        |
//! |------|----------------|----------------|
//! | En   | MMM D, YYYY    | Jul 5, 2026    |
//! | Pt   | D de mon. YYYY | 5 de jul. 2026 |
//! | Es   | D de mon. YYYY | 5 de jul. 2026 |
//! | Fr   | D mon. YYYY    | 5 juil. 2026   |

use super::Lang;

// ── Currency ──────────────────────────────────────────────────────────────────

/// Format a monetary amount according to the active locale.
///
/// Returns a display string without a trailing `.00` for whole-dollar amounts
/// in order to keep pricing cards clean.
///
/// # Examples
/// ```
/// assert_eq!(format_currency(285.0, Lang::En), "$285");
/// assert_eq!(format_currency(285.0, Lang::Pt), "R$ 285");
/// assert_eq!(format_currency(1285.5, Lang::En), "$1,285.50");
/// ```
pub fn format_currency(amount: f64, lang: Lang) -> String {
    let (symbol, decimal_sep, thousands_sep) = match lang {
        Lang::En => ("$",   ".", ","),
        Lang::Pt => ("R$ ", ",", "."),
        Lang::Es => ("$",   ".", ","),   // simplified — no per-country currency symbol yet
        Lang::Fr => ("CA$ ",".", " "),
    };

    let whole    = amount as u64;
    let cents    = ((amount - whole as f64) * 100.0).round() as u32;
    let whole_fmt = format_thousands(whole, thousands_sep);

    if cents == 0 {
        format!("{}{}", symbol, whole_fmt)
    } else {
        format!("{}{}{}{:02}", symbol, whole_fmt, decimal_sep, cents)
    }
}

/// Format a whole number with thousands separators.
fn format_thousands(n: u64, sep: &str) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            // Prepend separator (we're building in reverse)
            result.insert(0, sep.chars().next().unwrap_or(','));
        }
        result.insert(0, ch);
    }
    result
}

// ── Date (short) ──────────────────────────────────────────────────────────────

/// Short human-readable date for UI labels.
/// Uses static month abbreviations rather than a `chrono` locale dependency.
///
/// # Examples
/// ```
/// format_date_short(2026, 7, 5, Lang::En)  // "Jul 5, 2026"
/// format_date_short(2026, 7, 5, Lang::Pt)  // "5 de jul. 2026"
/// ```
pub fn format_date_short(year: i32, month: u32, day: u32, lang: Lang) -> String {
    let months_en = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
    let months_pt = ["jan","fev","mar","abr","mai","jun","jul","ago","set","out","nov","dez"];
    let months_es = ["ene","feb","mar","abr","may","jun","jul","ago","sep","oct","nov","dic"];
    let months_fr = ["janv","févr","mars","avr","mai","juin","juil","août","sept","oct","nov","déc"];

    let m = (month.saturating_sub(1) as usize).min(11);

    match lang {
        Lang::En => format!("{} {}, {}", months_en[m], day, year),
        Lang::Pt => format!("{} de {}. {}", day, months_pt[m], year),
        Lang::Es => format!("{} de {}. {}", day, months_es[m], year),
        Lang::Fr => format!("{} {}. {}", day, months_fr[m], year),
    }
}

// ── Phone (future) ────────────────────────────────────────────────────────────

/// Placeholder for locale-aware phone number formatting.
/// Currently returns the number unchanged. Will be implemented when
/// the vendor profile editor supports phone number input.
pub fn format_phone(number: &str, _lang: Lang) -> &str {
    number
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency_en_whole() {
        assert_eq!(format_currency(285.0, Lang::En), "$285");
    }

    #[test]
    fn currency_en_thousands() {
        assert_eq!(format_currency(1285.0, Lang::En), "$1,285");
    }

    #[test]
    fn currency_pt_whole() {
        assert_eq!(format_currency(285.0, Lang::Pt), "R$ 285");
    }

    #[test]
    fn currency_en_cents() {
        assert_eq!(format_currency(29.99, Lang::En), "$29.99");
    }

    #[test]
    fn date_en() {
        assert_eq!(format_date_short(2026, 7, 5, Lang::En), "Jul 5, 2026");
    }

    #[test]
    fn date_pt() {
        assert_eq!(format_date_short(2026, 7, 5, Lang::Pt), "5 de jul. 2026");
    }

    #[test]
    fn date_es() {
        assert_eq!(format_date_short(2026, 7, 5, Lang::Es), "5 de jul. 2026");
    }
}
