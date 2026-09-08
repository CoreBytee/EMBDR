use reqwest::Client;
use url::Url;

use tiktok_client::TiktokClient;

use crate::sources::{MediaAuthor, MediaData, MediaItem, MediaProperty, Source};

pub struct TiktokSource {
    tiktok_client: TiktokClient,
}

impl TiktokSource {
    pub fn new() -> Self {
        let tiktok_client = match std::env::var("PROXY_URL") {
            Ok(proxy_url) if !proxy_url.is_empty() => {
                let proxy = reqwest::Proxy::all(&proxy_url).expect("Failed to parse PROXY_URL");
                let http_client = Client::builder()
                    .cookie_store(true)
                    .proxy(proxy)
                    .build()
                    .expect("Failed to build HTTP client with proxy");
                TiktokClient::new_with_http_client(http_client)
            }
            _ => TiktokClient::new().expect("Failed to create tiktok client"),
        };
        Self { tiktok_client }
    }
}

/// Parse stats from a TikTok description.
/// Format: "**❤️ 341 💬 6 🔁 59**" at the end of the text.
/// Numbers may have K/M suffixes (e.g., "203.1K").
/// Returns (cleaned_description, likes, comments, reposts).
fn parse_tiktok_stats(content: &str) -> (Option<String>, u64, u64, u64) {
    let mut likes = 0u64;
    let mut comments = 0u64;
    let mut reposts = 0u64;

    let text = content.trim();

    // Find the stats line: starts with ❤ (may be inside **bold**)
    let stats_start = text.rfind("❤");
    if let Some(start) = stats_start {
        let stats_line = &text[start..];

        // Collect all segments, splitting on any whitespace (including \u2000)
        let segments: Vec<&str> = stats_line.split(|c: char| c.is_whitespace()).filter(|s| !s.is_empty()).collect();
        let mut i = 0;
        while i < segments.len() {
            let seg = segments[i];
            let has_emoji = seg.contains('❤') || seg.contains('💬') || seg.contains('🔁');

            if has_emoji {
                // Extract digits (and optional decimal/K/M) from this segment
                if let Some(num) = extract_number(seg) {
                    assign_stat(seg, num, &mut likes, &mut comments, &mut reposts);
                }
                // Also check the next segment for a standalone number
                else if i + 1 < segments.len() {
                    if let Some(num) = extract_number(segments[i + 1]) {
                        assign_stat(seg, num, &mut likes, &mut comments, &mut reposts);
                        i += 1;
                    }
                }
            }
            i += 1;
        }

        // Strip the stats line and trailing bold markdown from the description
        let cleaned = text[..start].trim_end_matches("**").trim().to_string();
        let description = if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        };

        (description, likes, comments, reposts)
    } else {
        let description = if text.is_empty() {
            None
        } else {
            Some(text.to_string())
        };
        (description, 0, 0, 0)
    }
}

/// Extract a number from a segment, handling K/M suffixes.
fn extract_number(s: &str) -> Option<u64> {
    let num_str: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
    if num_str.is_empty() {
        return None;
    }

    let has_k = s.contains('K') || s.contains('k');
    let has_m = s.contains('M') || s.contains('m');

    let base: f64 = num_str.parse().ok()?;
    let value = if has_m {
        base * 1_000_000.0
    } else if has_k {
        base * 1_000.0
    } else {
        base
    };

    Some(value as u64)
}

/// Assign a parsed number to the correct stat based on which emoji is present.
fn assign_stat(seg: &str, num: u64, likes: &mut u64, comments: &mut u64, reposts: &mut u64) {
    if seg.contains('❤') { *likes = num; }
    else if seg.contains('💬') { *comments = num; }
    else if seg.contains('🔁') { *reposts = num; }
}

#[async_trait::async_trait]
impl Source for TiktokSource {
    fn name(&self) -> String {
        "TikTok".to_string()
    }

    fn id(&self) -> String {
        "tiktok".to_string()
    }

    fn short_id(&self) -> String {
        "tt".to_string()
    }

    fn color(&self) -> u32 {
        0xff004f
    }

    fn predicate(&self, url: &Url) -> bool {
        let Some(hostname) = url.host_str() else {
            return false;
        };

        let hostname_matches =
            hostname == "www.tiktok.com" || hostname == "tiktok.com" || hostname == "vm.tiktok.com";

        return hostname_matches;
    }

    async fn extract_media(
        &self,
        url: &Url,
    ) -> Result<crate::sources::MediaData, Box<dyn std::error::Error + Send + Sync>> {
        let tiktok_post = self.tiktok_client.fetch_from_url(url.as_str()).await?;

        let (description, likes, comments, reposts) =
            parse_tiktok_stats(&tiktok_post.content);

        let mut properties = Vec::new();
        if likes > 0 {
            properties.push(MediaProperty::LikeCount(likes));
        }
        if comments > 0 {
            properties.push(MediaProperty::CommentCount(comments));
        }
        if reposts > 0 {
            properties.push(MediaProperty::RepostCount(reposts));
        }

        Ok(MediaData {
            id: tiktok_post.id.clone(),
            title: None,
            author: MediaAuthor {
                name: tiktok_post.account.display_name.clone(),
                url: format!("https://www.tiktok.com/@{}", tiktok_post.account.username),
            },
            community: None,
            description,
            items: tiktok_post
                .media_attachments
                .into_iter()
                .map(|item| MediaItem {
                    url: item.url.clone(),
                })
                .collect(),
            properties,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tiktok_stats() {
        let content = "**Deze Guy😭😭😭**\n#rijbewijs #rijles #rijschool \n\n❤️341 💬 6 🔁 59";
        let (desc, likes, comments, reposts) = parse_tiktok_stats(content);
        assert_eq!(desc.unwrap(), "**Deze Guy😭😭😭**\n#rijbewijs #rijles #rijschool");
        assert_eq!(likes, 341);
        assert_eq!(comments, 6);
        assert_eq!(reposts, 59);
    }

    #[test]
    fn test_parse_tiktok_stats_no_stats() {
        let content = "Just a regular post";
        let (desc, likes, comments, reposts) = parse_tiktok_stats(content);
        assert_eq!(desc.unwrap(), "Just a regular post");
        assert_eq!(likes, 0);
        assert_eq!(comments, 0);
        assert_eq!(reposts, 0);
    }

    #[test]
    fn test_parse_tiktok_stats_only_likes() {
        let content = "Nice video ❤️ 1200";
        let (desc, likes, comments, reposts) = parse_tiktok_stats(content);
        assert_eq!(desc.unwrap(), "Nice video");
        assert_eq!(likes, 1200);
        assert_eq!(comments, 0);
        assert_eq!(reposts, 0);
    }

    #[test]
    fn test_parse_tiktok_stats_trailing_bold() {
        let content = "Some caption **bold text** ❤️ 100 💬 5 🔁 2";
        let (desc, likes, comments, reposts) = parse_tiktok_stats(content);
        assert_eq!(desc.unwrap(), "Some caption **bold text**");
        assert_eq!(likes, 100);
        assert_eq!(comments, 5);
        assert_eq!(reposts, 2);
    }

    #[test]
    fn test_parse_tiktok_stats_k_suffix() {
        let content = "**❤️\u{2000}203.1K\u{2000}💬\u{2000}718\u{2000}🔁\u{2000}4.8K**";
        let (desc, likes, comments, reposts) = parse_tiktok_stats(content);
        assert_eq!(desc, None);
        assert_eq!(likes, 203_100);
        assert_eq!(comments, 718);
        assert_eq!(reposts, 4_800);
    }

    #[test]
    fn test_parse_tiktok_stats_bold_wrapped() {
        let content = "Caption\n\n**❤️ 537 💬 3 🔁 13**";
        let (desc, likes, comments, reposts) = parse_tiktok_stats(content);
        assert_eq!(desc.unwrap(), "Caption");
        assert_eq!(likes, 537);
        assert_eq!(comments, 3);
        assert_eq!(reposts, 13);
    }
}
