//! Visitor geo-detection from Cloudflare request headers.
//!
//! Cloudflare injects these on every request at the edge — zero latency,
//! no external API call, no PII stored:
//!
//! | Header           | Example   | Notes                          |
//! |------------------|-----------|--------------------------------|
//! | `CF-IPCountry`   | `"US"`    | ISO 3166-1 alpha-2             |
//! | `CF-IPContinent` | `"NA"`    | Continent code                 |
//! | `CF-IPCity`      | `"Miami"` | Approximate city               |
//!
//! Falls back to `"US"` / `"NA"` / `None` when headers are absent
//! (local dev, direct connections bypassing Cloudflare, tests).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
pub struct VisitorGeo {
    /// ISO 3166-1 alpha-2 country code, e.g. `"US"`, `"BR"`, `"CA"`.
    /// Defaults to `"US"` when the Cloudflare header is absent.
    pub country_code: String,
    /// Continent code, e.g. `"NA"`, `"SA"`, `"EU"`.
    pub continent: String,
    /// Approximate city name, or `None` when absent.
    pub city: Option<String>,
}

impl VisitorGeo {
    /// Returns the landing-page variant slug that best matches this visitor's geography.
    /// Operators must create matching rows in `app_page_variants` for these slugs.
    /// Falls through to the US-English default for any unrecognised country.
    pub fn variant_slug(&self) -> &'static str {
        match self.country_code.as_str() {
            "BR" => "folio-home-br-pt",
            "CA" => "folio-home-ca-en",
            "MX" | "CO" | "AR" | "CL" | "PE" | "EC" => "folio-home-latam-es",
            _ => "folio-home-us-en",
        }
    }
}

/// Server function: reads Cloudflare geo headers from the incoming SSR request.
///
/// Called once per marketing page render on the server. The result is embedded
/// as a data attribute on the waitlist form (`data-variant-id`, `data-country`)
/// so the client can include it in the waitlist API payload without a
/// second round-trip.
#[cfg(feature = "ssr")]
#[leptos::server(GetVisitorGeo, "/api")]
pub async fn get_visitor_geo() -> Result<VisitorGeo, leptos::server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers: HeaderMap = extract().await?;

    let get_header = |name: &str| -> String {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_default()
    };

    let country_code = {
        let raw = get_header("CF-IPCountry");
        // CF-IPCountry is "XX" when Cloudflare can't determine the country.
        if raw.is_empty() || raw == "XX" {
            "US".to_string()
        } else {
            raw
        }
    };

    let continent = {
        let raw = get_header("CF-IPContinent");
        if raw.is_empty() {
            "NA".to_string()
        } else {
            raw
        }
    };

    let city = {
        let raw = get_header("CF-IPCity");
        if raw.is_empty() {
            None
        } else {
            Some(raw)
        }
    };

    Ok(VisitorGeo {
        country_code,
        continent,
        city,
    })
}

/// Client-side stub (used when the server function is not available).
/// Returns the US-English default.
#[cfg(not(feature = "ssr"))]
pub async fn get_visitor_geo() -> Result<VisitorGeo, leptos::server_fn::error::ServerFnError> {
    Ok(VisitorGeo {
        country_code: "US".to_string(),
        continent: "NA".to_string(),
        city: None,
    })
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn geo(country: &str) -> VisitorGeo {
        VisitorGeo {
            country_code: country.to_string(),
            continent: "NA".to_string(),
            city: None,
        }
    }

    // ── variant_slug mapping ─────────────────────────────────────────────────

    #[test]
    fn brazil_maps_to_pt_variant() {
        assert_eq!(geo("BR").variant_slug(), "folio-home-br-pt");
    }

    #[test]
    fn canada_maps_to_ca_variant() {
        assert_eq!(geo("CA").variant_slug(), "folio-home-ca-en");
    }

    #[test]
    fn latam_countries_map_to_es_variant() {
        for code in &["MX", "CO", "AR", "CL", "PE", "EC"] {
            assert_eq!(
                geo(code).variant_slug(),
                "folio-home-latam-es",
                "expected latam-es for {code}"
            );
        }
    }

    #[test]
    fn us_maps_to_us_default() {
        assert_eq!(geo("US").variant_slug(), "folio-home-us-en");
    }

    #[test]
    fn unknown_country_falls_back_to_us_default() {
        for code in &["GB", "AU", "DE", "FR", "JP", "ZZ", "XX", ""] {
            assert_eq!(
                geo(code).variant_slug(),
                "folio-home-us-en",
                "expected us-en fallback for unknown code {code:?}"
            );
        }
    }

    // ── Default ──────────────────────────────────────────────────────────────

    #[test]
    fn default_is_us_na() {
        let d = VisitorGeo::default();
        assert_eq!(d.country_code, "");
        assert_eq!(d.continent, "");
        assert!(d.city.is_none());
        // Default falls through to the US-English variant
        assert_eq!(d.variant_slug(), "folio-home-us-en");
    }

    // ── PartialEq ────────────────────────────────────────────────────────────

    #[test]
    fn two_identical_geos_are_equal() {
        let a = geo("US");
        let b = geo("US");
        assert_eq!(a, b);
    }

    #[test]
    fn different_countries_are_not_equal() {
        assert_ne!(geo("US"), geo("BR"));
    }
}
