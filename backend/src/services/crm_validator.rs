#![allow(dead_code, unused)]
use once_cell::sync::Lazy;
use std::collections::HashSet;

static BLOCKED_DOMAINS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let mut s = HashSet::new();
    s.insert("test.com");
    s.insert("example.com");
    s.insert("tempmail.com");
    s.insert("mailinator.com");
    s.insert("junk.com");
    s.insert("trashmail.com");
    s
});

/// Cleans and sanitizes a phone input, strictly validating it against the international E.164 standard.
/// Must be formatted like: +1234567890 (length between 7 and 15 digits including +).
pub fn validate_and_sanitize_phone(phone: &str) -> Result<String, String> {
    let trimmed = phone.trim();
    if trimmed.is_empty() {
        return Ok("".to_string());
    }

    // Keep only digits and the leading plus symbol
    let cleaned: String = trimmed
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '+')
        .collect();

    // Check E.164 compliance using standard string rules:
    // Starts with '+' followed by 7 to 15 digits (total length 8 to 16 characters including '+')
    if cleaned.starts_with('+') && cleaned.len() >= 8 && cleaned.len() <= 16 {
        // Ensure every character after the first is a digit and not '0' immediately after country code if invalid
        let after_plus = &cleaned[1..];
        if after_plus.chars().all(|c| c.is_ascii_digit()) && !after_plus.starts_with('0') {
            return Ok(cleaned);
        }
    }

    Err("Invalid phone format. Please enter a valid international number in E.164 format (e.g., +15551234567).".to_string())
}

/// Performs a deep validation of the email, validating syntax and verifying domain DNS resolving.
pub async fn validate_email_deep(email: &str) -> Result<String, String> {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return Err("Email address cannot be empty.".to_string());
    }

    // 1. Basic format matching using standard string parsers:
    // Must contain exactly one '@', and domain part must have at least one '.'
    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2 {
        return Err("Invalid email address format (e.g. user@domain.com).".to_string());
    }

    let username = parts[0];
    let domain = parts[1].to_lowercase();

    if username.is_empty() || domain.is_empty() {
        return Err("Invalid email address format (e.g. user@domain.com).".to_string());
    }

    if !domain.contains('.') || domain.starts_with('.') || domain.ends_with('.') {
        return Err("Invalid email address format (e.g. user@domain.com).".to_string());
    }

    // 2. Filter domain blocks
    if BLOCKED_DOMAINS.contains(domain.as_str()) {
        return Err(format!(
            "The domain '{}' is blocked or reserved for testing. Please use a valid email domain.",
            domain
        ));
    }

    // 3. DNS Host resolution verification
    // Use tokio's system host resolver to check if the domain is registered and resolvable.
    let host_to_resolve = format!("{}:80", domain);
    match tokio::net::lookup_host(host_to_resolve.as_str()).await {
        Ok(mut addrs) => {
            if addrs.next().is_some() {
                Ok(trimmed.to_string())
            } else {
                Err(format!(
                    "The email domain '{}' could not be resolved. Please verify the spelling or domain status.",
                    domain
                ))
            }
        }
        Err(_) => Err(format!(
            "The email domain '{}' does not resolve to an active mail handler or host.",
            domain
        )),
    }
}
