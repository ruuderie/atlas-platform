//! Human-readable place labels for asset pickers (street · unit name).

/// Format an asset option like `"33 Orchard St · Unit 2"`.
///
/// Street is primary; the asset name is appended when it differs from the street.
/// When there is no street, falls back to name, then city/state.
pub fn format_asset_place_label(
    name: &str,
    address_line_1: Option<&str>,
    city: Option<&str>,
    state: Option<&str>,
) -> String {
    let name = name.trim();
    let street = address_line_1.map(str::trim).filter(|s| !s.is_empty());
    let city_state = [city, state]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(", ");

    match street {
        Some(street) if !name.is_empty() && !name.eq_ignore_ascii_case(street) => {
            format!("{street} · {name}")
        }
        Some(street) => street.to_string(),
        None if !name.is_empty() && !city_state.is_empty() => {
            format!("{name} · {city_state}")
        }
        None if !name.is_empty() => name.to_string(),
        None if !city_state.is_empty() => city_state,
        None => "Untitled".to_string(),
    }
}
