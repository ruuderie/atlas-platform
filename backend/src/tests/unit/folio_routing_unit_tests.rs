//! Folio routing + display helpers — pure unit tests.
//!
//! Tests the backend type system methods that directly drive frontend routing
//! decisions in the Folio Leptos app. No DB, no I/O, no async.
//!
//! Run with: `cargo test -p backend folio_routing_unit_tests`

#[cfg(test)]
mod folio_role_tests {
    use crate::types::pm::FolioRole;

    // ── Display (snake_case serde wire format) ────────────────────────────────

    #[test]
    fn display_is_snake_case() {
        assert_eq!(FolioRole::Landlord.to_string(), "landlord");
        assert_eq!(FolioRole::Tenant.to_string(), "tenant");
        assert_eq!(FolioRole::Vendor.to_string(), "vendor");
        assert_eq!(FolioRole::PropertyManager.to_string(), "property_manager");
        assert_eq!(FolioRole::Owner.to_string(), "owner");
        assert_eq!(FolioRole::Cohost.to_string(), "cohost");
        assert_eq!(FolioRole::Agent.to_string(), "agent");
        assert_eq!(FolioRole::Broker.to_string(), "broker");
        // str_host is NOT a valid role — STR is an asset trait
        assert!(
            FolioRole::try_from("str_host").is_err(),
            "str_host must not parse"
        );
    }

    // ── TryFrom roundtrip ─────────────────────────────────────────────────────

    #[test]
    fn tryfrom_string_roundtrip_all_variants() {
        let variants = [
            FolioRole::Landlord,
            FolioRole::Tenant,
            FolioRole::Vendor,
            FolioRole::PropertyManager,
            FolioRole::Owner,
            FolioRole::Cohost,
            FolioRole::Agent,
            FolioRole::Broker,
        ];
        for role in &variants {
            let s = role.to_string();
            let back = FolioRole::try_from(s.as_str())
                .unwrap_or_else(|e| panic!("roundtrip failed for {role}: {e}"));
            assert_eq!(&back, role, "roundtrip mismatch for {role}");
        }
    }

    #[test]
    fn tryfrom_unknown_string_returns_err() {
        assert!(FolioRole::try_from("admin").is_err());
        assert!(FolioRole::try_from("superuser").is_err());
        assert!(FolioRole::try_from("").is_err());
        assert!(
            FolioRole::try_from("Landlord").is_err(),
            "case-sensitive: PascalCase must fail"
        );
    }

    // ── Default ───────────────────────────────────────────────────────────────

    #[test]
    fn default_is_landlord() {
        assert_eq!(FolioRole::default(), FolioRole::Landlord);
    }

    // ── home_path() ───────────────────────────────────────────────────────────
    // These paths are consumed by the Folio login redirect and RoleRedirect component.
    // Changing them without updating the frontend router will break login.

    #[test]
    fn home_path_all_variants() {
        assert_eq!(FolioRole::Landlord.home_path(), "/dashboard");
        assert_eq!(FolioRole::Tenant.home_path(), "/my-home");
        assert_eq!(FolioRole::Vendor.home_path(), "/work-orders");
        assert_eq!(FolioRole::PropertyManager.home_path(), "/pm");
        assert_eq!(FolioRole::Owner.home_path(), "/owner");
        assert_eq!(FolioRole::Cohost.home_path(), "/ch");
        assert_eq!(FolioRole::Agent.home_path(), "/a");
        assert_eq!(FolioRole::Broker.home_path(), "/b");
    }

    #[test]
    fn home_paths_start_with_slash() {
        let roles = [
            FolioRole::Landlord,
            FolioRole::Tenant,
            FolioRole::Vendor,
            FolioRole::PropertyManager,
            FolioRole::Owner,
            FolioRole::Cohost,
            FolioRole::Agent,
            FolioRole::Broker,
        ];
        for role in &roles {
            let p = role.home_path();
            assert!(
                p.starts_with('/'),
                "home_path for {role} = {p:?} must start with '/'"
            );
        }
    }

    // ── Role predicate methods ─────────────────────────────────────────────────

    #[test]
    fn is_pmc_only_property_manager() {
        assert!(FolioRole::PropertyManager.is_pmc());
        assert!(!FolioRole::Landlord.is_pmc());
        assert!(!FolioRole::Tenant.is_pmc());
        assert!(!FolioRole::Vendor.is_pmc());
        assert!(!FolioRole::Owner.is_pmc());
        assert!(!FolioRole::Cohost.is_pmc());
        assert!(!FolioRole::Agent.is_pmc());
        assert!(!FolioRole::Broker.is_pmc());
    }

    #[test]
    fn is_owner_only_owner() {
        assert!(FolioRole::Owner.is_owner());
        assert!(!FolioRole::Landlord.is_owner());
        assert!(!FolioRole::PropertyManager.is_owner());
        assert!(!FolioRole::Tenant.is_owner());
    }

    #[test]
    fn is_brokerage_only_agent_and_broker() {
        assert!(FolioRole::Agent.is_brokerage());
        assert!(FolioRole::Broker.is_brokerage());
        assert!(!FolioRole::Landlord.is_brokerage());
        assert!(!FolioRole::Tenant.is_brokerage());
        assert!(!FolioRole::Vendor.is_brokerage());
        assert!(!FolioRole::PropertyManager.is_brokerage());
        assert!(!FolioRole::Owner.is_brokerage());
    }

    // ── Predicates are mutually exclusive (PMC / owner / brokerage) ───────────

    #[test]
    fn pmc_owner_brokerage_predicates_are_mutually_exclusive() {
        let roles = [
            FolioRole::Landlord,
            FolioRole::Tenant,
            FolioRole::Vendor,
            FolioRole::PropertyManager,
            FolioRole::Owner,
            FolioRole::Cohost,
            FolioRole::Agent,
            FolioRole::Broker,
        ];
        for role in &roles {
            let count = [role.is_pmc(), role.is_owner(), role.is_brokerage()]
                .into_iter()
                .filter(|b| *b)
                .count();
            assert!(
                count <= 1,
                "role {role} is in multiple exclusive groups (pmc={}, owner={}, brokerage={})",
                role.is_pmc(),
                role.is_owner(),
                role.is_brokerage()
            );
        }
    }

    // ── Serde roundtrip ───────────────────────────────────────────────────────

    #[test]
    fn serde_json_roundtrip_all_variants() {
        let variants = [
            FolioRole::Landlord,
            FolioRole::Tenant,
            FolioRole::Vendor,
            FolioRole::PropertyManager,
            FolioRole::Owner,
            FolioRole::Cohost,
            FolioRole::Agent,
            FolioRole::Broker,
        ];
        for role in &variants {
            let json = serde_json::to_string(role)
                .unwrap_or_else(|e| panic!("serialize failed for {role}: {e}"));
            let back: FolioRole = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("deserialize failed for {role} from {json:?}: {e}"));
            assert_eq!(&back, role, "serde roundtrip mismatch for {role}");
        }
    }

    #[test]
    fn serde_json_snake_case_wire_format() {
        // The Folio frontend depends on this exact wire format for session deserialization
        assert_eq!(
            serde_json::to_string(&FolioRole::PropertyManager).unwrap(),
            "\"property_manager\""
        );
        assert_eq!(
            serde_json::to_string(&FolioRole::Landlord).unwrap(),
            "\"landlord\""
        );
        assert_eq!(
            serde_json::to_string(&FolioRole::Broker).unwrap(),
            "\"broker\""
        );
    }
}
