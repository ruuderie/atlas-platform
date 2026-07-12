//! Contract: every WizardShell onboarding path wires `VerifiedEmailField`
//! so the backend-captured email can be shown read-only after auth.
//!
//! These are source-level regression tests (no WASM/browser). They fail if a
//! persona wizard drops the read-only email row or if the path table drifts.

/// Route → wizard source file that must contain `VerifiedEmailField`.
const ONBOARD_EMAIL_WIRING: &[(&str, &str, &str)] = &[
    (
        "/onboarding",
        "landlord_wizard.rs",
        include_str!("landlord_wizard.rs"),
    ),
    (
        "/onboard/vendor",
        "vendor_wizard.rs",
        include_str!("vendor_wizard.rs"),
    ),
    (
        "/onboard/pmc",
        "pmc_wizard.rs",
        include_str!("pmc_wizard.rs"),
    ),
    (
        "/onboard/broker",
        "broker_wizard.rs",
        include_str!("broker_wizard.rs"),
    ),
    (
        "/onboard/agent",
        "agent_wizard.rs",
        include_str!("agent_wizard.rs"),
    ),
    (
        "/onboard/property-owner",
        "property_owner_wizard.rs",
        include_str!("property_owner_wizard.rs"),
    ),
    (
        "/onboard/str-guest",
        "str_guest_wizard.rs",
        include_str!("str_guest_wizard.rs"),
    ),
    (
        "/onboard/tenant",
        "tenant_wizard.rs",
        include_str!("tenant_wizard.rs"),
    ),
    (
        "/onboard/cohost",
        "cohost_wizard.rs",
        include_str!("cohost_wizard.rs"),
    ),
    (
        "/onboard/owner",
        "owner_wizard.rs",
        include_str!("owner_wizard.rs"),
    ),
];

#[test]
fn every_onboard_path_wires_verified_email_field() {
    for (route, file, src) in ONBOARD_EMAIL_WIRING {
        assert!(
            src.contains("VerifiedEmailField"),
            "{route} ({file}) must render <VerifiedEmailField/> so OTP/session email is read-only"
        );
        assert!(
            src.contains("WizardShell"),
            "{route} ({file}) must use WizardShell (pre-auth + WizardAuthCtx)"
        );
    }
}

#[test]
fn onboard_email_wiring_covers_ten_persona_paths() {
    assert_eq!(
        ONBOARD_EMAIL_WIRING.len(),
        10,
        "update ONBOARD_EMAIL_WIRING when adding/removing WizardShell personas"
    );
}

#[test]
fn tenant_submit_uses_synced_email_not_empty_literal() {
    let src = include_str!("tenant_wizard.rs");
    assert!(
        src.contains("SyncVerifiedEmail"),
        "tenant wizard must SyncVerifiedEmail into the submit payload"
    );
    assert!(
        !src.contains("email: String::new()"),
        "tenant submit must not hardcode an empty email — use the synced verified email"
    );
}
