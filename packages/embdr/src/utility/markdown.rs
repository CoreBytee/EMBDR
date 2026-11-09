/// Returns true if the character is an emoji or part of an emoji sequence.
///
/// Discord rejects masked links whose label contains an emoji, so emoji need
/// to be kept outside the link label.
fn is_emoji_character(character: char) -> bool {
    let is_emoji_component = matches!(
        character,
        '\u{200D}'                   // zero width joiner
            | '\u{FE0E}' | '\u{FE0F}' // variation selectors
            | '\u{20E3}'              // combining enclosing keycap
            | '\u{1F3FB}'..='\u{1F3FF}' // skin tone modifiers
            | '\u{1F9B0}'..='\u{1F9B3}' // hair components
            | '\u{1F1E6}'..='\u{1F1FF}' // regional indicators (flags)
            | '\u{E0020}'..='\u{E007F}' // tag characters
    );

    if is_emoji_component {
        return true;
    }

    let mut buffer = [0; 4];
    emojis::get(character.encode_utf8(&mut buffer)).is_some()
}

/// Escapes characters that Discord would otherwise interpret as markdown.
fn escape_markdown(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());

    for character in text.chars() {
        if matches!(
            character,
            '\\' | '`' | '*' | '_' | '~' | '|' | '>' | '[' | ']'
        ) {
            escaped.push('\\');
        }

        escaped.push(character);
    }

    escaped
}

/// Escapes characters that would prematurely end a masked link destination.
fn escape_link_target(url: &str) -> String {
    url.replace('(', "%28").replace(')', "%29")
}

/// Builds a Discord masked link while keeping emoji outside the link label.
///
/// Discord does not render masked links when the label contains an emoji, so
/// the label is split into linked text segments and unlinked emoji segments.
/// Markdown characters in the label and parentheses in the URL are escaped so
/// the link is always parsed correctly.
pub fn masked_link(label: &str, url: &str) -> String {
    let mut result = String::new();
    let mut text_segment = String::new();
    let link_target = escape_link_target(url);

    for character in label.chars() {
        if is_emoji_character(character) {
            if !text_segment.is_empty() {
                result.push_str(&format!(
                    "[{}]({})",
                    escape_markdown(&text_segment),
                    link_target
                ));
                text_segment.clear();
            }

            result.push(character);
        } else {
            text_segment.push(character);
        }
    }

    if !text_segment.is_empty() {
        result.push_str(&format!(
            "[{}]({})",
            escape_markdown(&text_segment),
            link_target
        ));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://www.reddit.com/r/ik_ihe/comments/1xrxszq/ik_ihe/";

    #[test]
    fn test_masked_link_without_emoji() {
        assert_eq!(
            masked_link("r/ik_ihe", URL),
            format!("[r/ik\\_ihe]({})", URL)
        );
    }

    #[test]
    fn test_masked_link_with_emoji() {
        assert_eq!(
            masked_link("ik🩵ihe", URL),
            format!("[ik]({0})🩵[ihe]({0})", URL)
        );
    }

    #[test]
    fn test_masked_link_with_leading_emoji() {
        assert_eq!(
            masked_link("🩵ik_ihe", URL),
            format!("🩵[ik\\_ihe]({})", URL)
        );
    }

    #[test]
    fn test_masked_link_with_brackets() {
        assert_eq!(
            masked_link("[OC] Title", URL),
            format!("[\\[OC\\] Title]({})", URL)
        );
    }

    #[test]
    fn test_masked_link_with_unmatched_closing_bracket() {
        assert_eq!(
            masked_link("Title] here", URL),
            format!("[Title\\] here]({})", URL)
        );
    }

    #[test]
    fn test_masked_link_with_markdown_formatting() {
        assert_eq!(
            masked_link("*star* _underline_ ~strike~", URL),
            format!("[\\*star\\* \\_underline\\_ \\~strike\\~]({})", URL)
        );
    }

    #[test]
    fn test_masked_link_with_backslash() {
        assert_eq!(
            masked_link("back\\slash", URL),
            format!("[back\\\\slash]({})", URL)
        );
    }

    #[test]
    fn test_masked_link_with_parentheses_in_url() {
        assert_eq!(
            masked_link("label", "https://example.com/wiki/Title_(disambiguation)"),
            "[label](https://example.com/wiki/Title_%28disambiguation%29)"
        );
    }

    #[test]
    fn test_masked_link_with_only_emoji() {
        assert_eq!(masked_link("🩵", URL), "🩵");
    }

    #[test]
    fn test_masked_link_with_skin_tone() {
        assert_eq!(masked_link("👍🏽 nice", URL), format!("👍🏽[ nice]({})", URL));
    }

    #[test]
    fn test_masked_link_with_zwj_sequence() {
        assert_eq!(
            masked_link("👨‍👩‍👧 family", URL),
            format!("👨‍👩‍👧[ family]({})", URL)
        );
    }

    #[test]
    fn test_masked_link_with_flag() {
        assert_eq!(
            masked_link("🇳🇱 Nederland", URL),
            format!("🇳🇱[ Nederland]({})", URL)
        );
    }

    #[test]
    fn test_masked_link_with_variation_selector() {
        assert_eq!(masked_link("❤️ love", URL), format!("❤️[ love]({})", URL));
    }

    #[test]
    fn test_masked_link_with_keycap() {
        assert_eq!(
            masked_link("1️⃣ place", URL),
            format!("[1]({0})️⃣[ place]({0})", URL)
        );
    }

    #[test]
    fn test_masked_link_with_only_emoji_sequence() {
        assert_eq!(masked_link("🎉🎉", URL), "🎉🎉");
    }
}
