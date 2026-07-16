//! Direct-mail provider registry smoke tests (no DB).

#[cfg(test)]
mod tests {
    use crate::services::pm::direct_mail::{
        resolve_direct_mail_provider, DirectMailError,
    };

    #[test]
    fn resolves_manual_provider() {
        let p = resolve_direct_mail_provider("dm_manual").expect("manual");
        assert_eq!(p.provider_id(), "dm_manual");
        assert!(p.parse_webhook(&serde_json::json!({})).unwrap().is_empty());
    }

    #[test]
    fn lob_stub_not_implemented() {
        let p = resolve_direct_mail_provider("dm_lob").expect("lob");
        let err = p
            .parse_webhook(&serde_json::json!({}))
            .expect_err("stub");
        assert!(matches!(err, DirectMailError::NotImplemented("dm_lob")));
    }

    #[test]
    fn property_radar_stub_not_implemented() {
        let p = resolve_direct_mail_provider("dm_property_radar").expect("pr");
        let err = p
            .parse_webhook(&serde_json::json!({}))
            .expect_err("stub");
        assert!(matches!(
            err,
            DirectMailError::NotImplemented("dm_property_radar")
        ));
    }

    #[test]
    fn unknown_provider_returns_none() {
        assert!(resolve_direct_mail_provider("dm_usps").is_none());
    }
}
