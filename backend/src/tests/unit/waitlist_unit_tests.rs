//! Waitlist handler — pure unit tests.
//!
//! Tests `WaitlistBody` deserialization and the response shape contract.
//! No DB, no I/O, no async — these run in milliseconds.
//!
//! Run with: `cargo test -p backend waitlist_unit_tests`

#[cfg(test)]
mod waitlist_body_tests {
    use crate::handlers::pub_products::WaitlistBody;
    use serde_json::json;

    // ── Minimal payload (email only) ─────────────────────────────────────────

    #[test]
    fn deserializes_email_only() {
        let v: WaitlistBody = serde_json::from_value(json!({ "email": "test@example.com" }))
            .expect("email-only payload must deserialize");
        assert_eq!(v.email, "test@example.com");
        assert!(v.name.is_none());
        assert!(v.phone.is_none());
        assert!(v.company.is_none());
        assert!(v.role.is_none());
        assert!(v.portfolio_size_label.is_none());
        assert!(v.utm_source.is_none());
    }

    // ── Full marketing-page payload ──────────────────────────────────────────

    #[test]
    fn deserializes_full_marketing_page_payload() {
        let v: WaitlistBody = serde_json::from_value(json!({
            "email":                "landlord@example.com",
            "name":                 "Marcus Davis",
            "phone":                "+1-305-000-0001",
            "role":                 "Landlord",
            "portfolio_size_label": "6–20 units",
            "utm_source":           "social",
            "utm_medium":           "instagram",
            "utm_campaign":         "launch-q3-2026",
            "utm_content":          "hero-cta",
            "utm_term":             "landlord software",
            "gclid":                "Cj0test",
            "fbclid":               "IwAR0test",
            "msclkid":              null,
            "referrer":             "https://instagram.com",
            "landing_url":          "https://folio.app/",
            "variant_slug":         "folio-home-us-en",
        }))
        .expect("full payload must deserialize");

        assert_eq!(v.email, "landlord@example.com");
        assert_eq!(v.name.as_deref(), Some("Marcus Davis"));
        assert_eq!(v.role.as_deref(), Some("Landlord"));
        assert_eq!(v.portfolio_size_label.as_deref(), Some("6–20 units"));
        assert_eq!(v.utm_source.as_deref(), Some("social"));
        assert_eq!(v.utm_campaign.as_deref(), Some("launch-q3-2026"));
        assert_eq!(v.gclid.as_deref(), Some("Cj0test"));
        assert_eq!(v.fbclid.as_deref(), Some("IwAR0test"));
        assert!(
            v.msclkid.is_none(),
            "explicit null must deserialize to None"
        );
        assert_eq!(v.referrer.as_deref(), Some("https://instagram.com"));
    }

    // ── Role values accepted as strings ──────────────────────────────────────

    #[test]
    fn all_role_labels_deserialize_as_strings() {
        // These are the labels sent by the marketing page pill buttons
        let roles = [
            "Landlord",
            "Property Manager",
            "STR Host",
            "Tenant",
            "Vendor",
            "Investor",
        ];
        for role in &roles {
            let v: WaitlistBody = serde_json::from_value(json!({
                "email": "x@example.com",
                "role":  role,
            }))
            .unwrap_or_else(|e| panic!("role={role:?} failed: {e}"));
            assert_eq!(v.role.as_deref(), Some(*role));
        }
    }

    // ── Portfolio size labels ────────────────────────────────────────────────

    #[test]
    fn all_portfolio_size_labels_deserialize() {
        let sizes = [
            "1–5 units",
            "6–20 units",
            "21–100 units",
            "100+ units",
            "Not applicable",
        ];
        for size in &sizes {
            let v: WaitlistBody = serde_json::from_value(json!({
                "email":                "x@example.com",
                "portfolio_size_label": size,
            }))
            .unwrap_or_else(|e| panic!("size={size:?} failed: {e}"));
            assert_eq!(v.portfolio_size_label.as_deref(), Some(*size));
        }
    }

    // ── Missing optional fields → None (not error) ───────────────────────────

    #[test]
    fn missing_optional_fields_produce_none_not_error() {
        // Omit every optional field — only email provided
        let v: WaitlistBody = serde_json::from_value(json!({
            "email": "min@example.com"
        }))
        .expect("minimal payload must not fail");

        // All optional fields must be None
        assert!(v.role.is_none());
        assert!(v.portfolio_size_label.is_none());
        assert!(v.utm_source.is_none());
        assert!(v.utm_medium.is_none());
        assert!(v.utm_campaign.is_none());
        assert!(v.utm_content.is_none());
        assert!(v.utm_term.is_none());
        assert!(v.gclid.is_none());
        assert!(v.fbclid.is_none());
        assert!(v.msclkid.is_none());
        assert!(v.referrer.is_none());
        assert!(v.landing_url.is_none());
        assert!(v.plan.is_none());
        assert!(v.unit_count.is_none());
    }

    // ── email field is required ──────────────────────────────────────────────

    #[test]
    fn missing_email_fails_deserialization() {
        let result = serde_json::from_value::<WaitlistBody>(json!({
            "role": "Landlord"
        }));
        assert!(
            result.is_err(),
            "email is required; missing it must fail deserialization"
        );
    }
}
