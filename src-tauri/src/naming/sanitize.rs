/// Sanitizes path components according to Jellyfin and cross-platform filesystem standards.
///
/// Guardrails:
/// - Replace `:` with ` -` (e.g. "Dune: Part Two" -> "Dune - Part Two")
/// - Replace `/` and `\` with `-`
/// - Strip `< > " | ? *`
/// - Strip control characters (0..=31)
/// - Strip trailing periods and spaces (Windows NTFS safety)
pub fn sanitize_path_component(input: &str) -> String {
    // 1. Replace colons with " -"
    let step1 = input.replace(':', " -");

    // 2. Replace slashes with "-"
    let step2 = step1.replace('/', "-").replace('\\', "-");

    // 3. Strip illegal characters: < > " | ? * and control chars
    let mut cleaned: String = step2
        .chars()
        .filter(|&c| {
            !matches!(c, '<' | '>' | '"' | '|' | '?' | '*') && !c.is_control()
        })
        .collect();

    // 4. Collapse consecutive spaces
    while cleaned.contains("  ") {
        cleaned = cleaned.replace("  ", " ");
    }

    // 5. Trim leading/trailing whitespace and trailing dots (Windows forbidden)
    let trimmed = cleaned.trim();
    let mut result = trimmed.to_string();
    while result.ends_with('.') || result.ends_with(' ') {
        result.pop();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_colon() {
        assert_eq!(
            sanitize_path_component("Dune: Part Two"),
            "Dune - Part Two"
        );
    }

    #[test]
    fn test_sanitize_slashes() {
        assert_eq!(
            sanitize_path_component("Face/Off"),
            "Face-Off"
        );
    }

    #[test]
    fn test_sanitize_illegal_chars() {
        assert_eq!(
            sanitize_path_component("Who Framed Roger Rabbit? <1988> *\"special\"|"),
            "Who Framed Roger Rabbit 1988 special"
        );
    }

    #[test]
    fn test_sanitize_trailing_dots_and_spaces() {
        assert_eq!(
            sanitize_path_component("Se7en...   "),
            "Se7en"
        );
    }
}
