use crate::webauthn_registry::effective_tld_plus_one;

#[cfg(test)]
mod tests {
    use super::*;

    // ── effective_tld_plus_one ───────────────────────────────────────────────

    #[test]
    fn strips_single_subdomain() {
        assert_eq!(effective_tld_plus_one("dev.buildwithruud.com"), "buildwithruud.com");
    }

    #[test]
    fn strips_deep_subdomain() {
        assert_eq!(effective_tld_plus_one("a.b.c.example.com"), "example.com");
    }

    #[test]
    fn apex_domain_unchanged() {
        assert_eq!(effective_tld_plus_one("buildwithruud.com"), "buildwithruud.com");
    }

    #[test]
    fn localhost_unchanged() {
        assert_eq!(effective_tld_plus_one("localhost"), "localhost");
    }

    #[test]
    fn platform_subdomain_to_base() {
        // Platform admin env subdomains must map to the shared platform base domain.
        // Passkeys registered on dev/uat/prod all share rpId = "oply.co".
        assert_eq!(effective_tld_plus_one("dev.atlas.oply.co"), "oply.co");
        assert_eq!(effective_tld_plus_one("uat.atlas.oply.co"), "oply.co");
    }

    // ── Multi-label TLD support (requires addr/PSL — would fail with naive split) ──

    #[test]
    fn multi_label_tld_co_uk() {
        // Naive split would return "co.uk" (wrong). PSL gives "example.co.uk".
        assert_eq!(effective_tld_plus_one("example.co.uk"), "example.co.uk");
        assert_eq!(effective_tld_plus_one("sub.example.co.uk"), "example.co.uk");
    }

    #[test]
    fn multi_label_tld_com_au() {
        assert_eq!(effective_tld_plus_one("shop.example.com.au"), "example.com.au");
    }

    #[test]
    fn ip_address_passthrough() {
        // IPs are not domain names — must not panic, must return host unchanged.
        assert_eq!(effective_tld_plus_one("192.168.1.1"), "192.168.1.1");
    }

    // ── Security: rp_id must be a suffix of the origin host ─────────────────
    // WebauthnBuilder validates that rpId is a registrable-domain suffix of the
    // origin host. If we pass the full subdomain as rpId, it IS a valid suffix
    // of itself — but passkeys registered from the apex domain will fail auth.
    // The eTLD+1 ensures both apex and subdomain origins share the same rpId.

    #[test]
    fn rp_id_is_suffix_of_origin_host() {
        let hosts = vec![
            ("dev.buildwithruud.com", "buildwithruud.com"),
            ("buildwithruud.com", "buildwithruud.com"),
            ("uat.atlas.oply.co", "oply.co"),
            ("sub.example.co.uk", "example.co.uk"),
        ];
        for (host, expected_rp_id) in hosts {
            let rp_id = effective_tld_plus_one(host);
            assert_eq!(&rp_id, expected_rp_id, "host={}", host);
            assert!(
                host.ends_with(&rp_id),
                "rpId '{}' must be a suffix of origin host '{}'",
                rp_id, host
            );
        }
    }
}
