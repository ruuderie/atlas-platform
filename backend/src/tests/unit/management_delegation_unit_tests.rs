//! Unit tests for same-tenant PM delegation helpers (no DB).

#[cfg(test)]
mod tests {
    use crate::services::pm::management_delegation::{
        DelegationScope, ManagementDelegationService,
    };
    use crate::types::pm::PmContractType;
    use uuid::Uuid;

    #[test]
    fn parse_assets_dedupes_and_orders() {
        let a = Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap();
        let b = Uuid::parse_str("bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb").unwrap();
        let ids = ManagementDelegationService::parse_invite_asset_ids(
            Some(a),
            Some(&format!(" {b} , {a} ")),
        );
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0], a);
        assert_eq!(ids[1], b);
    }

    #[test]
    fn empty_assets_means_portfolio_hire() {
        assert!(ManagementDelegationService::parse_invite_asset_ids(None, Some("")).is_empty());
    }

    #[test]
    fn terms_metadata_portfolio() {
        let invite = Uuid::nil();
        let v = ManagementDelegationService::build_terms_metadata(
            DelegationScope::Portfolio,
            &[],
            invite,
        );
        assert_eq!(v["scope"], "portfolio");
        assert!(v["asset_ids"].as_array().unwrap().is_empty());
    }

    #[test]
    fn rejects_legacy_contract_type_alias() {
        assert_eq!(
            PmContractType::ManagementAgreement.to_string(),
            "management_agreement"
        );
        assert!(PmContractType::try_from("property_management_agreement".to_string()).is_err());
    }

    #[test]
    fn scope_try_from() {
        assert_eq!(
            DelegationScope::try_from("asset").unwrap(),
            DelegationScope::Asset
        );
        assert!(DelegationScope::try_from("book").is_err());
    }
}
