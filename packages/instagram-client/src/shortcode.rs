// ---------------------------------------------------------------------------
// Shortcode extraction
// ---------------------------------------------------------------------------

/// Extract the shortcode from various Instagram URL formats.
pub fn extract_shortcode(url: &str) -> String {
    let url = url.split(['?', '#']).next().unwrap_or(url);
    let parts: Vec<&str> = url.split('/').collect();
    let tags = ["p", "reel", "tv", "reels"];
    let index = parts
        .iter()
        .position(|part| tags.contains(part))
        .unwrap_or(0)
        + 1;
    parts.get(index).unwrap_or(&"").to_string()
}

/// Alphabet Instagram uses for media shortcodes.
const SHORTCODE_ALPHABET: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Decode a media shortcode into its numeric media ID.
pub(crate) fn media_id_from_shortcode(shortcode: &str) -> Option<u64> {
    shortcode.bytes().try_fold(0u64, |media_id, byte| {
        let digit = SHORTCODE_ALPHABET
            .iter()
            .position(|candidate| *candidate == byte)? as u64;
        media_id.checked_mul(64)?.checked_add(digit)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_query_from_shortcode() {
        assert_eq!(
            extract_shortcode("https://www.instagram.com/reel/DfHo8XJSXBP/?cplk=aGc5"),
            "DfHo8XJSXBP"
        );
        assert_eq!(
            extract_shortcode("https://www.instagram.com/reel/DfHo8XJSXBP?cplk=aGc5"),
            "DfHo8XJSXBP"
        );
        assert_eq!(
            extract_shortcode("https://www.instagram.com/p/DfHo8XJSXBP/"),
            "DfHo8XJSXBP"
        );
    }

    #[test]
    fn decodes_shortcode_to_media_id() {
        assert_eq!(
            media_id_from_shortcode("Df1qX-atfxJ"),
            Some(4032315406901705801)
        );
        assert_eq!(
            media_id_from_shortcode("Df6c1gyN3cN"),
            Some(4033663238900512525)
        );
        assert_eq!(media_id_from_shortcode("not-a/valid"), None);
    }
}
