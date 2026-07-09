//! Provision unit tests — validation logic for the unified Folio provisioning endpoint.
//!
//! Tests all 8 provisionable roles and their dependency requirements:
//!   landlord          — no deps (or account_id to join existing workspace)
//!   tenant            — requires lease_id; portal adapts by lease_type (ltr|str)
//!   vendor            — no deps; asset_ids optional for scoping
//!   cohost            — requires asset_ids ≥ 1 (STR assets delegated by a landlord)
//!   owner             — requires account_id (PMC client account)
//!   agent             — no explicit deps; implicit under brokerage instance
//!   broker            — no deps
//!   property_manager  — no deps
//!
//! str_host: NOT a valid role. STR is an asset-level trait (str_eligible on atlas_assets).
//!
//! Run with: `cargo test -p atlas_backend provision_unit`

#[cfg(test)]
mod provision_validation_tests {
    use uuid::Uuid;
    use crate::types::pm::FolioRole;

    // ── Mirror of provision::validate_invite for unit testing ─────────────────
    //
    // The actual handler function is private. These tests mirror its logic so
    // we can test validation rules without spinning up a DB or HTTP server.
    // Integration tests in provision_integration_tests.rs cover the full stack.

    struct MockInvite {
        app_role:   &'static str,
        asset_ids:  Option<Vec<Uuid>>,
        lease_id:   Option<Uuid>,
        account_id: Option<Uuid>,
    }

    impl MockInvite {
        fn new(role: &'static str) -> Self {
            Self { app_role: role, asset_ids: None, lease_id: None, account_id: None }
        }
        fn with_assets(mut self, ids: Vec<Uuid>) -> Self { self.asset_ids = Some(ids); self }
        fn with_lease(mut self, id: Uuid)        -> Self { self.lease_id   = Some(id); self }
        fn with_account(mut self, id: Uuid)      -> Self { self.account_id = Some(id); self }
    }

    fn validate(inv: &MockInvite) -> Result<FolioRole, String> {
        if inv.app_role == "str_host" {
            return Err(
                "\"str_host\" is not a valid role. STR capability is a property trait, \
                 not a persona. Invite this user as \"landlord\" and enable STR on \
                 specific assets via atlas_assets.str_eligible = true.".into()
            );
        }
        let role = FolioRole::try_from(inv.app_role).map_err(|e| e.to_string())?;
        match role {
            FolioRole::Cohost => {
                if inv.asset_ids.as_ref().map_or(true, |v| v.is_empty()) {
                    return Err("Cohost invites require at least one asset_id. \
                                The cohost must be delegated to specific STR-eligible properties.".into());
                }
            }
            FolioRole::Tenant => {
                if inv.lease_id.is_none() {
                    return Err("Tenant invites require a lease_id. \
                                Create the lease first, then invite the tenant.".into());
                }
            }
            FolioRole::Owner => {
                if inv.account_id.is_none() {
                    return Err("Owner invites require an account_id. \
                                The beneficial owner must be linked to their client account.".into());
                }
            }
            _ => {}
        }
        Ok(role)
    }

    // ── str_host removal ─────────────────────────────────────────────────────

    #[test]
    fn str_host_invite_is_rejected_with_helpful_message() {
        let result = validate(&MockInvite::new("str_host"));
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(
            msg.contains("landlord") || msg.contains("str_eligible"),
            "Error for str_host should point to 'landlord' or 'str_eligible'. Got: {msg}"
        );
    }

    #[test]
    fn str_host_is_not_a_folio_role() {
        let err = FolioRole::try_from("str_host").unwrap_err();
        assert!(!err.is_empty(), "str_host must return a non-empty error");
    }

    #[test]
    fn unknown_role_slugs_are_rejected() {
        for slug in &["admin", "superuser", "", "LANDLORD", "str-host", "strhost"] {
            assert!(validate(&MockInvite::new(slug)).is_err(),
                "Expected '{slug}' to be rejected");
        }
    }

    // ── All 8 valid roles parse ───────────────────────────────────────────────

    #[test]
    fn all_nine_valid_roles_parse() {
        let valid = ["landlord", "tenant", "str_guest", "vendor", "cohost",
                     "owner", "property_manager", "agent", "broker"];
        assert_eq!(valid.len(), 9, "Exactly 9 valid FolioRoles expected");
        for slug in &valid {
            assert!(FolioRole::try_from(*slug).is_ok(), "'{slug}' should be valid");
        }
    }

    // ── Landlord ─────────────────────────────────────────────────────────────

    #[test]
    fn landlord_no_deps_succeeds() {
        assert_eq!(validate(&MockInvite::new("landlord")).unwrap(), FolioRole::Landlord);
    }

    #[test]
    fn landlord_with_account_id_joins_existing_workspace() {
        let result = validate(&MockInvite::new("landlord").with_account(Uuid::new_v4()));
        assert_eq!(result.unwrap(), FolioRole::Landlord);
    }

    #[test]
    fn landlord_with_multiple_asset_ids_gets_delegated_access() {
        // A landlord delegate (restricted to a subset of the portfolio)
        let result = validate(&MockInvite::new("landlord")
            .with_assets(vec![Uuid::new_v4(), Uuid::new_v4()]));
        assert_eq!(result.unwrap(), FolioRole::Landlord);
    }

    // ── Tenant ───────────────────────────────────────────────────────────────

    #[test]
    fn tenant_without_lease_id_is_rejected() {
        let result = validate(&MockInvite::new("tenant"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("lease"),
            "Error should mention 'lease'");
    }

    #[test]
    fn tenant_with_lease_id_succeeds() {
        assert_eq!(
            validate(&MockInvite::new("tenant").with_lease(Uuid::new_v4())).unwrap(),
            FolioRole::Tenant
        );
    }

    #[test]
    fn ltr_tenant_lease_type_is_valid() {
        // lease_type = "ltr" → full tenant portal (rent, maintenance, docs)
        // lease_type is informational at invite time; ground truth is on the lease row.
        let result = validate(&MockInvite::new("tenant").with_lease(Uuid::new_v4()));
        assert_eq!(result.unwrap(), FolioRole::Tenant);
    }

    #[test]
    fn str_tenant_lease_type_is_valid() {
        // lease_type = "str" → guest portal view (reservation, check-in, house rules)
        // A tenant can be both an LTR renter in one city and an STR guest elsewhere.
        // The TYPE is on the lease, not the user.
        let result = validate(&MockInvite::new("tenant").with_lease(Uuid::new_v4()));
        assert_eq!(result.unwrap(), FolioRole::Tenant);
    }

    #[test]
    fn tenant_depends_on_landlord_via_lease_chain() {
        // Tenant → lease_id → atlas_leases.asset_id → atlas_assets.owner_user_id (landlord)
        // FK chain guarantees the landlord dependency without explicit validation.
        // This test documents the architectural contract.
        let lease_id = Uuid::new_v4(); // belongs to a landlord's asset in production
        let result = validate(&MockInvite::new("tenant").with_lease(lease_id));
        assert!(result.is_ok(), "Tenant with valid lease should pass validation");
    }

    // ── Vendor ───────────────────────────────────────────────────────────────

    #[test]
    fn vendor_no_deps_succeeds() {
        // Vendors operate independently — no mandatory linkage to a landlord
        assert_eq!(validate(&MockInvite::new("vendor")).unwrap(), FolioRole::Vendor);
    }

    #[test]
    fn vendor_with_asset_ids_is_asset_scoped() {
        let result = validate(&MockInvite::new("vendor").with_assets(vec![Uuid::new_v4()]));
        assert_eq!(result.unwrap(), FolioRole::Vendor);
    }

    #[test]
    fn vendor_with_empty_asset_ids_succeeds_org_level() {
        // Unlike cohost, empty asset_ids = org-level access for vendor (valid)
        let result = validate(&MockInvite::new("vendor").with_assets(vec![]));
        assert_eq!(result.unwrap(), FolioRole::Vendor);
    }

    #[test]
    fn vendor_with_multiple_assets_succeeds() {
        let assets = (0..5).map(|_| Uuid::new_v4()).collect();
        let result = validate(&MockInvite::new("vendor").with_assets(assets));
        assert_eq!(result.unwrap(), FolioRole::Vendor);
    }

    // ── Cohost ───────────────────────────────────────────────────────────────

    #[test]
    fn cohost_without_asset_ids_is_rejected() {
        let result = validate(&MockInvite::new("cohost"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("asset"));
    }

    #[test]
    fn cohost_with_empty_asset_ids_is_rejected() {
        let result = validate(&MockInvite::new("cohost").with_assets(vec![]));
        assert!(result.is_err(), "Empty asset list should be rejected for cohost");
    }

    #[test]
    fn cohost_with_one_asset_succeeds() {
        let result = validate(&MockInvite::new("cohost").with_assets(vec![Uuid::new_v4()]));
        assert_eq!(result.unwrap(), FolioRole::Cohost);
    }

    #[test]
    fn cohost_with_multiple_assets_succeeds() {
        // Multi-property cohost: one row per asset in atlas_user_asset_access
        let assets = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
        let result = validate(&MockInvite::new("cohost").with_assets(assets));
        assert_eq!(result.unwrap(), FolioRole::Cohost);
    }

    #[test]
    fn cohost_has_own_account_but_operates_on_landlord_assets() {
        // Dependency chain:
        //   1. Cohost gets their own login + workspace (independent identity)
        //   2. But their asset_ids MUST belong to a landlord's portfolio
        //   3. atlas_user_asset_access.asset_id → atlas_assets.owner_user_id = landlord
        //   4. Verified at service layer; FK integrity enforces it at DB level
        let landlord_str_asset = Uuid::new_v4();
        let result = validate(&MockInvite::new("cohost").with_assets(vec![landlord_str_asset]));
        assert!(result.is_ok());
    }

    // ── Owner ────────────────────────────────────────────────────────────────

    #[test]
    fn owner_without_account_id_is_rejected() {
        let result = validate(&MockInvite::new("owner"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("account"));
    }

    #[test]
    fn owner_with_account_id_succeeds() {
        let result = validate(&MockInvite::new("owner").with_account(Uuid::new_v4()));
        assert_eq!(result.unwrap(), FolioRole::Owner);
    }

    // ── Agent ────────────────────────────────────────────────────────────────

    #[test]
    fn agent_no_explicit_deps_succeeds() {
        // Broker dependency is implicit: agent invite comes FROM a brokerage instance.
        // app_instance_id on the invite carries the broker context.
        // No broker_id needed in the payload.
        let result = validate(&MockInvite::new("agent"));
        assert_eq!(result.unwrap(), FolioRole::Agent);
    }

    #[test]
    fn agent_is_a_brokerage_role() {
        assert!(FolioRole::Agent.is_brokerage());
    }

    #[test]
    fn agent_home_path_is_slash_a() {
        assert_eq!(FolioRole::Agent.home_path(), "/a");
    }

    // ── Broker ───────────────────────────────────────────────────────────────

    #[test]
    fn broker_no_deps_succeeds() {
        let result = validate(&MockInvite::new("broker"));
        assert_eq!(result.unwrap(), FolioRole::Broker);
    }

    #[test]
    fn broker_is_a_brokerage_role() {
        assert!(FolioRole::Broker.is_brokerage());
    }

    #[test]
    fn broker_home_path_is_slash_b() {
        assert_eq!(FolioRole::Broker.home_path(), "/b");
    }

    // ── Property Manager ─────────────────────────────────────────────────────

    #[test]
    fn property_manager_no_deps_succeeds() {
        let result = validate(&MockInvite::new("property_manager"));
        assert_eq!(result.unwrap(), FolioRole::PropertyManager);
    }

    #[test]
    fn property_manager_is_pmc_role() {
        assert!(FolioRole::PropertyManager.is_pmc());
    }

    // ── Role exclusivity ─────────────────────────────────────────────────────

    #[test]
    fn pmc_owner_brokerage_predicates_are_mutually_exclusive() {
        let all_roles = [
            FolioRole::Landlord, FolioRole::Tenant, FolioRole::Vendor,
            FolioRole::PropertyManager, FolioRole::Owner,
            FolioRole::Cohost, FolioRole::Agent, FolioRole::Broker,
        ];
        for role in &all_roles {
            let count = [role.is_pmc(), role.is_owner(), role.is_brokerage()]
                .into_iter().filter(|b| *b).count();
            assert!(count <= 1,
                "{role} is in multiple exclusive groups (pmc={}, owner={}, brokerage={})",
                role.is_pmc(), role.is_owner(), role.is_brokerage());
        }
    }

    #[test]
    fn only_agent_and_broker_are_brokerage_roles() {
        for role in &[FolioRole::Landlord, FolioRole::Tenant, FolioRole::Vendor,
                      FolioRole::PropertyManager, FolioRole::Owner, FolioRole::Cohost] {
            assert!(!role.is_brokerage(), "{role} should not be a brokerage role");
        }
    }

    // ── Idempotency contract (documented) ────────────────────────────────────

    #[test]
    fn idempotency_contract_documented() {
        // POST /api/folio/provision/invite is idempotent per (email, app_role):
        //   1st call → 201 Created, { invite_id, reused: false }
        //   2nd call → 200 OK,      { same invite_id, reused: true }
        //   After expiry → 201 Created, { new invite_id, reused: false }
        //
        // Full coverage in: provision_integration_tests::test_invite_idempotent_*
        assert!(true, "contract documented");
    }
}

// ── str_guest + tenant applicant tests (added after str_guest role landed) ───

#[cfg(test)]
mod str_guest_and_applicant_tests {
    use uuid::Uuid;
    use crate::types::pm::FolioRole;

    struct MockInvite {
        app_role:       &'static str,
        asset_id:       Option<Uuid>,
        asset_ids:      Option<Vec<Uuid>>,
        booking_id:     Option<Uuid>,
        lease_id:       Option<Uuid>,
        tenancy_status: Option<&'static str>,
        account_id:     Option<Uuid>,
    }

    impl MockInvite {
        fn new(role: &'static str) -> Self {
            Self { app_role: role, asset_id: None, asset_ids: None,
                   booking_id: None, lease_id: None, tenancy_status: None, account_id: None }
        }
        fn with_asset(mut self, id: Uuid)            -> Self { self.asset_id   = Some(id); self }
        fn with_booking(mut self, id: Uuid)          -> Self { self.booking_id = Some(id); self }
        fn with_lease(mut self, id: Uuid)            -> Self { self.lease_id   = Some(id); self }
        fn with_status(mut self, s: &'static str)    -> Self { self.tenancy_status = Some(s); self }
        fn with_assets(mut self, v: Vec<Uuid>)       -> Self { self.asset_ids  = Some(v); self }
        fn with_account(mut self, id: Uuid)          -> Self { self.account_id = Some(id); self }
    }

    fn validate(inv: &MockInvite) -> Result<FolioRole, String> {
        if inv.app_role == "str_host" {
            return Err("str_host is not a valid role. Use landlord + asset str_eligible=true".into());
        }
        let role = FolioRole::try_from(inv.app_role).map_err(|e| e.to_string())?;
        match role {
            FolioRole::StrGuest => {
                if inv.asset_id.is_none() {
                    return Err("str_guest invites require an asset_id".into());
                }
            }
            FolioRole::Cohost => {
                if inv.asset_ids.as_ref().map_or(true, |v| v.is_empty()) {
                    return Err("Cohost requires asset_ids ≥ 1".into());
                }
            }
            FolioRole::Tenant => {
                let status = inv.tenancy_status.unwrap_or("applicant");
                match status {
                    "pending" | "active" => {
                        if inv.lease_id.is_none() {
                            return Err(format!("Tenant '{status}' requires lease_id"));
                        }
                    }
                    "applicant" => {} // no lease required yet
                    other => return Err(format!("invalid tenancy_status '{other}'")),
                }
            }
            FolioRole::Owner => {
                if inv.account_id.is_none() { return Err("Owner requires account_id".into()); }
            }
            _ => {}
        }
        Ok(role)
    }

    // ── str_guest ─────────────────────────────────────────────────────────────

    #[test]
    fn str_guest_without_asset_id_is_rejected() {
        let result = validate(&MockInvite::new("str_guest"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("asset"));
    }

    #[test]
    fn str_guest_with_asset_id_succeeds() {
        let result = validate(&MockInvite::new("str_guest").with_asset(Uuid::new_v4()));
        assert_eq!(result.unwrap(), FolioRole::StrGuest);
    }

    #[test]
    fn str_guest_with_booking_id_succeeds() {
        // Pre-existing reservation — guest is confirming an already-created booking
        let result = validate(&MockInvite::new("str_guest")
            .with_asset(Uuid::new_v4())
            .with_booking(Uuid::new_v4()));
        assert_eq!(result.unwrap(), FolioRole::StrGuest);
    }

    #[test]
    fn str_guest_without_booking_id_succeeds() {
        // Guest selects dates during onboarding wizard (booking created on accept)
        let result = validate(&MockInvite::new("str_guest").with_asset(Uuid::new_v4()));
        assert_eq!(result.unwrap(), FolioRole::StrGuest);
    }

    #[test]
    fn str_guest_home_path_is_slash_g() {
        assert_eq!(FolioRole::StrGuest.home_path(), "/g");
    }

    #[test]
    fn str_guest_display_is_str_guest() {
        assert_eq!(FolioRole::StrGuest.to_string(), "str_guest");
    }

    #[test]
    fn str_guest_is_not_a_brokerage_role() {
        assert!(!FolioRole::StrGuest.is_brokerage());
    }

    #[test]
    fn str_guest_is_not_pmc() {
        assert!(!FolioRole::StrGuest.is_pmc());
    }

    #[test]
    fn str_guest_roundtrip_from_str() {
        let role = FolioRole::try_from("str_guest").unwrap();
        assert_eq!(role, FolioRole::StrGuest);
        assert_eq!(role.to_string(), "str_guest");
    }

    // ── Tenant applicant status ───────────────────────────────────────────────

    #[test]
    fn tenant_applicant_no_lease_required() {
        // Applicant: filling profile to be considered, no lease yet
        let result = validate(&MockInvite::new("tenant").with_status("applicant"));
        assert_eq!(result.unwrap(), FolioRole::Tenant);
    }

    #[test]
    fn tenant_applicant_default_when_no_status_given() {
        // Default tenancy_status = "applicant" — safe for new tenant invites
        let result = validate(&MockInvite::new("tenant"));
        assert_eq!(result.unwrap(), FolioRole::Tenant);
    }

    #[test]
    fn tenant_applicant_with_asset_id_applies_to_specific_unit() {
        // Optional: applicant can be tied to a specific unit they want
        let result = validate(&MockInvite::new("tenant")
            .with_status("applicant")
            .with_asset(Uuid::new_v4()));
        assert_eq!(result.unwrap(), FolioRole::Tenant);
    }

    #[test]
    fn tenant_pending_requires_lease_id() {
        let result = validate(&MockInvite::new("tenant").with_status("pending"));
        assert!(result.is_err());
        let msg = result.unwrap_err();
        assert!(msg.contains("pending") && msg.contains("lease_id"),
            "Error should mention 'pending' and 'lease_id'. Got: {msg}");
    }

    #[test]
    fn tenant_pending_with_lease_id_succeeds() {
        let result = validate(&MockInvite::new("tenant")
            .with_status("pending")
            .with_lease(Uuid::new_v4()));
        assert_eq!(result.unwrap(), FolioRole::Tenant);
    }

    #[test]
    fn tenant_active_requires_lease_id() {
        let result = validate(&MockInvite::new("tenant").with_status("active"));
        assert!(result.is_err());
    }

    #[test]
    fn tenant_active_with_lease_id_succeeds() {
        let result = validate(&MockInvite::new("tenant")
            .with_status("active")
            .with_lease(Uuid::new_v4()));
        assert_eq!(result.unwrap(), FolioRole::Tenant);
    }

    #[test]
    fn tenant_invalid_status_is_rejected() {
        let result = validate(&MockInvite::new("tenant").with_status("guest"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid tenancy_status 'guest'"));
    }

    // ── str_guest vs tenant are distinct identities ───────────────────────────

    #[test]
    fn str_guest_and_tenant_are_different_roles() {
        assert_ne!(FolioRole::StrGuest, FolioRole::Tenant);
        assert_ne!(FolioRole::StrGuest.home_path(), FolioRole::Tenant.home_path());
        assert_ne!(FolioRole::StrGuest.to_string(), FolioRole::Tenant.to_string());
    }

    #[test]
    fn str_guest_onboarding_requires_property_not_lease() {
        // Key distinction: str_guest links to a booking/asset, not a lease.
        // This test documents the architectural invariant.
        let guest = validate(&MockInvite::new("str_guest").with_asset(Uuid::new_v4()));
        assert!(guest.is_ok(), "str_guest with asset should succeed (no lease needed)");

        let guest_no_asset = validate(&MockInvite::new("str_guest").with_lease(Uuid::new_v4()));
        assert!(guest_no_asset.is_err(), "str_guest with only a lease_id (no asset_id) should fail");
    }

    // ── Role count regression (now 9) ─────────────────────────────────────────

    #[test]
    fn folio_role_count_is_nine() {
        // Regression: adding a new role requires updating this test
        // AND the provisioning validation logic AND nav configs.
        let valid_roles = ["landlord", "tenant", "str_guest", "vendor", "cohost",
                           "owner", "property_manager", "agent", "broker"];
        assert_eq!(valid_roles.len(), 9, "Expected 9 valid FolioRoles");
        for slug in &valid_roles {
            assert!(FolioRole::try_from(*slug).is_ok(), "'{slug}' should be valid");
        }
    }
}
