//! Phone input helpers for onboarding wizards.
//!
//! Display: NANP as-you-type `(555) 123-4567`
//! Wire: E.164 `+15551234567` (required for landlord profile)

/// Digits only from free-form phone input.
pub fn digits_only(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit()).collect()
}

/// Format a US/NANP number while the user types.
///
/// Accepts pasted values with `+1`, spaces, dashes, etc. Caps at 10 national digits.
pub fn format_nanp_input(raw: &str) -> String {
    let mut digits = digits_only(raw);
    // Drop a leading country-code 1 when the user typed/pasted 11 digits.
    if digits.len() == 11 && digits.starts_with('1') {
        digits = digits[1..].to_string();
    }
    if digits.len() > 10 {
        digits.truncate(10);
    }

    match digits.len() {
        0 => String::new(),
        1..=3 => digits,
        4..=6 => format!("({}) {}", &digits[..3], &digits[3..]),
        _ => format!("({}) {}-{}", &digits[..3], &digits[3..6], &digits[6..]),
    }
}

/// Normalize to E.164 for US numbers.
///
/// Phone is required for landlord onboarding — blank and incomplete inputs error.
pub fn to_e164_us(raw: &str) -> Result<String, &'static str> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Phone number is required");
    }

    let mut digits = digits_only(trimmed);
    if digits.len() == 11 && digits.starts_with('1') {
        digits = digits[1..].to_string();
    }

    if digits.len() != 10 {
        return Err("Enter a 10-digit US phone number");
    }

    Ok(format!("+1{digits}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_as_you_type() {
        assert_eq!(format_nanp_input("5"), "5");
        assert_eq!(format_nanp_input("555"), "555");
        assert_eq!(format_nanp_input("5551"), "(555) 1");
        assert_eq!(format_nanp_input("5551234567"), "(555) 123-4567");
        assert_eq!(format_nanp_input("+1 (555) 123-4567"), "(555) 123-4567");
        assert_eq!(format_nanp_input("15551234567"), "(555) 123-4567");
    }

    #[test]
    fn e164_round_trip() {
        assert_eq!(to_e164_us("(555) 123-4567").unwrap(), "+15551234567");
        assert_eq!(to_e164_us("").unwrap_err(), "Phone number is required");
        assert!(to_e164_us("555").is_err());
    }
}
