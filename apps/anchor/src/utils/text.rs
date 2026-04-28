/// Shared text utilities for the Kami design system.
///
/// These functions are used across multiple pages and components to ensure
/// consistent Role/Action/Impact parsing and markdown excerpt generation.
/// Having a single canonical implementation prevents the two call sites
/// from diverging silently as the format evolves.

/// Parses a Role/Action/Impact prefix from a bullet string.
///
/// Returns `(Some("Role"|"Action"|"Impact"), rest_of_bullet)` when the
/// bullet starts with a known label, or `(None, original)` otherwise.
///
/// # Examples
/// ```
/// use anchor::utils::text::parse_rai;
/// let (label, rest) = parse_rai("Role: Lead engineer on core infrastructure");
/// assert_eq!(label, Some("Role"));
/// assert_eq!(rest, "Lead engineer on core infrastructure");
///
/// let (label, rest) = parse_rai("plain bullet");
/// assert_eq!(label, None);
/// assert_eq!(rest, "plain bullet");
/// ```
pub fn parse_rai(b: &str) -> (Option<&str>, &str) {
    for label in ["Role", "Action", "Impact"] {
        if let Some(rest) = b.strip_prefix(&format!("{}: ", label)) {
            return (Some(label), rest);
        }
    }
    (None, b)
}

/// Generates a plain-text excerpt from a markdown string.
///
/// Renders the markdown to HTML via `pulldown_cmark`, strips HTML tags,
/// trims whitespace, and truncates to `max_chars` with an ellipsis.
///
/// This is intentionally kept in a shared utility so the same logic is
/// used whether called server-side (in `get_posts`) or in a component.
pub fn markdown_excerpt(md: &str, max_chars: usize) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(md, opts);
    let mut html_buf = String::new();
    html::push_html(&mut html_buf, parser);

    // Strip HTML tags: split on '<', discard everything up to the next '>'.
    let text: String = html_buf
        .split('<')
        .enumerate()
        .map(|(i, s)| {
            if i == 0 {
                s.to_string()
            } else {
                s.split_once('>').map(|(_, after)| after).unwrap_or("").to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("");

    let trimmed = text.trim();
    if trimmed.len() > max_chars {
        format!("{}\u{2026}", &trimmed[..max_chars])
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rai_known_labels() {
        let (l, r) = parse_rai("Role: Principal engineer");
        assert_eq!(l, Some("Role"));
        assert_eq!(r, "Principal engineer");

        let (l, r) = parse_rai("Action: Migrated legacy monolith");
        assert_eq!(l, Some("Action"));
        assert_eq!(r, "Migrated legacy monolith");

        let (l, r) = parse_rai("Impact: Reduced latency by 40%");
        assert_eq!(l, Some("Impact"));
        assert_eq!(r, "Reduced latency by 40%");
    }

    #[test]
    fn test_parse_rai_plain_bullet() {
        let (l, r) = parse_rai("plain bullet with no prefix");
        assert_eq!(l, None);
        assert_eq!(r, "plain bullet with no prefix");
    }

    #[test]
    fn test_parse_rai_case_sensitive() {
        // Labels are case-sensitive; "role:" should not match.
        let (l, r) = parse_rai("role: lowercase should not match");
        assert_eq!(l, None);
        assert_eq!(r, "role: lowercase should not match");
    }

    #[test]
    fn test_markdown_excerpt_truncates() {
        let md = "Hello world this is a long piece of text.";
        let exc = markdown_excerpt(md, 10);
        assert!(exc.ends_with('\u{2026}'), "should end with ellipsis");
        // char count is tricky with multibyte ellipsis; check byte length
        assert!(exc.len() <= 14, "should be at most 10 chars + ellipsis");
    }

    #[test]
    fn test_markdown_excerpt_strips_markdown() {
        let md = "**bold** and _italic_";
        let exc = markdown_excerpt(md, 200);
        assert!(!exc.contains('<'), "should not contain HTML tags");
        assert!(!exc.contains('*'), "markdown syntax should be stripped");
    }
}
