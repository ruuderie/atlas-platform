use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// ── Request ───────────────────────────────────────────────────────────────────

/// Payload for `POST /api/admin/tenants/provision`.
///
/// Creates a fully wired tenant atomically: tenant, account, app_instance(s),
/// app_domain, CMS scaffolding, admin user, and a one-time passkey setup link.
#[derive(Debug, Deserialize, Validate)]
pub struct ProvisionTenantPayload {
    /// Internal slug — lowercase alphanumeric + hyphens, globally unique.
    /// Used to derive Ingress resource names. Example: "acme-corp"
    #[validate(length(min = 2, max = 63), custom(function = "validate_slug"))]
    pub tenant_name: String,

    /// Human-readable display name shown in platform-admin. Example: "Acme Corp"
    #[validate(length(min = 1, max = 255))]
    pub display_name: String,

    /// Primary FQDN for this tenant's anchor-app — no scheme, no port, no path.
    /// Must not already exist in app_domains. Example: "acme.com"
    #[validate(length(min = 4, max = 253))]
    pub domain: String,

    /// Email address of the tenant's first admin user. They receive the setup link.
    #[validate(email)]
    pub admin_email: String,

    pub admin_first_name: String,
    pub admin_last_name: String,

    /// Which AtlasApps to provision for this tenant.
    /// Defaults to `["anchor"]` when omitted.
    pub apps: Option<Vec<String>>,

    /// Bypass the DNS TXT record ownership verification.
    /// Must be explicitly set to true by PlatformSuperAdmin.
    pub bypass_dns_verification: Option<bool>,
}

// ── Response ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProvisionTenantResponse {
    pub tenant_id: Uuid,
    pub account_id: Uuid,
    pub domain: String,
    /// One-time passkey setup URL for the new tenant admin.
    /// The operator copies this and sends it to the tenant.
    pub setup_url: String,
}

// ── Validation helpers ────────────────────────────────────────────────────────

/// Validates a tenant slug: lowercase alphanum + hyphens, no leading/trailing hyphens.
pub fn validate_slug(slug: &str) -> Result<(), validator::ValidationError> {
    let ok = !slug.is_empty()
        && slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-');
    if ok {
        Ok(())
    } else {
        Err(validator::ValidationError::new("invalid_slug"))
    }
}

/// Validates a domain string for use as an app_domain entry.
///
/// Rules:
/// - No scheme (`https://`, `http://`)
/// - No trailing slash or path segment
/// - No port suffix
/// - Must be a valid-looking FQDN (at least one dot)
/// - Must not exceed 253 characters
pub fn validate_domain(domain: &str) -> Result<(), String> {
    if domain.contains("://") {
        return Err("Domain must not include a scheme (remove 'https://' or 'http://')".to_string());
    }
    if domain.contains('/') {
        return Err("Domain must not include a path (remove everything after the first '/')".to_string());
    }
    if domain.contains(':') {
        return Err("Domain must not include a port number".to_string());
    }
    if domain.len() > 253 {
        return Err("Domain exceeds maximum length of 253 characters".to_string());
    }
    if !domain.contains('.') && domain != "localhost" {
        return Err("Domain must be a fully-qualified domain name (e.g. acme.com)".to_string());
    }
    // Each label must be 1–63 chars, alphanumeric + hyphen, no leading/trailing hyphen
    for label in domain.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err(format!("Domain label '{}' is invalid (must be 1–63 characters)", label));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(format!("Domain label '{}' must not start or end with a hyphen", label));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_domain_rejects_scheme() {
        assert!(validate_domain("https://acme.com").is_err());
        assert!(validate_domain("http://acme.com").is_err());
    }

    #[test]
    fn validate_domain_rejects_path() {
        assert!(validate_domain("acme.com/path").is_err());
        assert!(validate_domain("acme.com/").is_err());
    }

    #[test]
    fn validate_domain_rejects_port() {
        assert!(validate_domain("acme.com:8080").is_err());
    }

    #[test]
    fn validate_domain_accepts_fqdn() {
        assert!(validate_domain("acme.com").is_ok());
        assert!(validate_domain("sub.acme.co.uk").is_ok());
        assert!(validate_domain("dev.my-company.io").is_ok());
    }

    #[test]
    fn validate_domain_accepts_localhost() {
        assert!(validate_domain("localhost").is_ok());
    }
}
