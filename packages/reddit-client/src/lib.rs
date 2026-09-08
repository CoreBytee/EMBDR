use reqwest::header::{HeaderMap, USER_AGENT};
use reqwest::{Client, StatusCode};
use scraper::{Html, Selector};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum RedditError {
    #[error("HTTP request failed: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("API returned no data for URL: {url}")]
    NoData { url: String },

    #[error("Unexpected response with status {status}: {body}")]
    UnexpectedResponse { status: StatusCode, body: String },

    #[error("Failed to parse page: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, RedditError>;

// ---------------------------------------------------------------------------
// Types — normalized / public API
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct RedditPost {
    pub id: String,
    pub title: String,
    pub author: String,
    pub subreddit: String,
    pub selftext: Option<String>,
    pub permalink: String,
    pub score: u64,
    pub num_comments: u64,
    pub media_items: Vec<RedditMediaItem>,
}

#[derive(Debug, Clone)]
pub enum RedditMediaItem {
    Image { url: String },
    Video { url: String },
}

impl RedditMediaItem {
    pub fn url(&self) -> &str {
        match self {
            Self::Image { url } | Self::Video { url } => url,
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const VXREDDIT_BASE_URL: &str = "https://vxreddit.com";

/// User-Agent that matches vxReddit's bot detection for social-preview crawlers.
const BOT_USER_AGENT: &str = "Discordbot/2.0";

// ---------------------------------------------------------------------------
// URL helpers
// ---------------------------------------------------------------------------

/// Extract the post ID from various Reddit URL formats.
pub fn extract_post_id(url: &str) -> Option<String> {
    let parts: Vec<&str> = url.split('/').collect();
    let comments_idx = parts.iter().position(|s| *s == "comments")?;
    let post_id = parts.get(comments_idx + 1)?;
    Some(post_id.to_string())
}

fn strip_reddit_domain(url: &str) -> &str {
    url.trim_start_matches("https://www.reddit.com")
        .trim_start_matches("https://reddit.com")
        .trim_start_matches("https://old.reddit.com")
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

fn bot_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, BOT_USER_AGENT.parse().unwrap());
    headers
}

/// Reddit client that fetches post data via vxReddit's embed pages.
pub struct RedditClient {
    http_client: Client,
}

impl RedditClient {
    /// Create a new client with default settings.
    pub fn new() -> Result<Self> {
        let http_client = Client::builder()
            .default_headers(bot_headers())
            .cookie_store(true)
            .build()?;
        Ok(Self { http_client })
    }

    /// Create a new client with a provided `reqwest::Client`.
    pub fn new_with_http(http_client: Client) -> Self {
        Self { http_client }
    }

    /// Fetch post data from a Reddit URL via vxReddit's embed page.
    pub async fn fetch_from_url(&self, url: &str) -> Result<RedditPost> {
        let path = strip_reddit_domain(url);
        let vx_url = format!("{}{}", VXREDDIT_BASE_URL, path);

        let max_retries = 3;
        let mut last_status = None;

        for attempt in 0..max_retries {
            let response = self.http_client.get(&vx_url).send().await?;

            let status = response.status();
            if status.is_success() {
                let html = response.text().await?;
                return parse_embed_page(&html, url);
            }

            if status == StatusCode::TOO_MANY_REQUESTS {
                last_status = Some(status);
                let wait_ms = 1000 * 2u64.pow(attempt);
                tracing::warn!(
                    "vxReddit request got 429 — retrying in {}ms (attempt {})",
                    wait_ms,
                    attempt + 1,
                );
                tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
                continue;
            }

            let body = response.text().await.unwrap_or_default();
            return Err(RedditError::UnexpectedResponse { status, body });
        }

        Err(RedditError::UnexpectedResponse {
            status: last_status.unwrap_or(StatusCode::TOO_MANY_REQUESTS),
            body: "rate limited — retries exhausted".to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// HTML meta tag parsing
// ---------------------------------------------------------------------------

fn parse_embed_page(html: &str, original_url: &str) -> Result<RedditPost> {
    let document = Html::parse_document(html);

    let meta_sel = Selector::parse("meta").map_err(|e| {
        RedditError::Parse(format!("Invalid selector: {}", e))
    })?;

    let mut og_site_name = None;
    let mut og_title = None;
    let mut og_description = None;
    let mut og_url = None;
    let mut og_images: Vec<String> = Vec::new();
    let mut og_video = None;
    let mut oembed_title = None;

    for meta in document.select(&meta_sel) {
        let prop = meta
            .attr("property")
            .or_else(|| meta.attr("name"))
            .unwrap_or_default();
        let content = meta.attr("content").unwrap_or_default();

        match prop {
            "og:site_name" => og_site_name = Some(content.to_string()),
            "og:title" => og_title = Some(content.to_string()),
            "og:description" => og_description = Some(content.to_string()),
            "og:url" => og_url = Some(content.to_string()),
            "og:image" => og_images.push(content.to_string()),
            "og:video" => og_video = Some(content.to_string()),
            _ => {}
        }
    }

    // Extract oembed title from <link rel="alternate" type="application/json+oembed">
    let link_sel = Selector::parse("link[rel='alternate'][type='application/json+oembed']").ok();
    if let Some(sel) = link_sel {
        if let Some(link) = document.select(&sel).next() {
            oembed_title = link.attr("title").map(|s| s.to_string());
        }
    }

    // Determine the actual post title:
    // - For text/link posts where title==text: og:title is "vxReddit", real title is in oembed title.
    // - For text posts with selftext: og:title has the real title.
    // - For image/video posts: og:title has the real title.
    let is_vxreddit_title = og_title.as_deref() == Some("vxReddit");

    let title = if is_vxreddit_title {
        oembed_title
            .clone()
            .or_else(|| og_description.clone())
            .unwrap_or_default()
    } else {
        og_title.unwrap_or_default()
    };

    // Selftext: og:description contains selftext only when it differs from the
    // resolved title AND there are no media items (image/video posts put the
    // title in og:description).
    let has_media = !og_images.is_empty() || og_video.is_some();
    let selftext = og_description
        .filter(|s| !s.is_empty())
        .filter(|desc| *desc != title)
        .filter(|_| !has_media);

    // Parse og:site_name: "{author} on r/{subreddit} - ⬆️ {upvotes} | 💬 {comments}"
    let (author, subreddit, score, num_comments) =
        parse_stats_line(og_site_name.as_deref().unwrap_or_default());

    // Build permalink
    let permalink = og_url
        .clone()
        .unwrap_or_else(|| original_url.to_string());

    // Extract post ID from permalink
    let id = extract_post_id(&permalink).unwrap_or_default();

    // Build media items.
    // For video posts, og:image is just the thumbnail — skip it.
    let mut media_items: Vec<RedditMediaItem> = Vec::new();

    if let Some(video_url) = og_video {
        media_items.push(RedditMediaItem::Video { url: video_url });
    } else {
        for img_url in &og_images {
            if !media_items
                .iter()
                .any(|item| item.url() == img_url.as_str())
            {
                media_items.push(RedditMediaItem::Image {
                    url: img_url.clone(),
                });
            }
        }
    }

    Ok(RedditPost {
        id,
        title,
        author,
        subreddit,
        selftext,
        permalink,
        score,
        num_comments,
        media_items,
    })
}

/// Parse the stats line from og:site_name.
/// Format: "{author} on r/{subreddit} - ⬆️ {upvotes} | 💬 {comments}"
fn parse_stats_line(line: &str) -> (String, String, u64, u64) {
    let mut author = String::new();
    let mut subreddit = String::new();
    let mut score = 0u64;
    let mut num_comments = 0u64;

    // Extract author: everything before " on "
    if let Some(on_idx) = line.find(" on ") {
        author = line[..on_idx]
            .trim_start_matches("u/")
            .trim_start_matches("/u/")
            .to_string();
    }

    // Extract subreddit: between "on " and " - "
    if let Some(on_idx) = line.find(" on ") {
        let rest = &line[on_idx + 4..];
        if let Some(dash_idx) = rest.find(" - ") {
            subreddit = rest[..dash_idx].to_string();
        }
    }

    // Extract upvotes: between "⬆️ " and " |" or end
    if let Some(up_idx) = line.find("⬆️ ") {
        let rest = &line[up_idx + "⬆️ ".len()..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        score = num_str.parse().unwrap_or(0);
    }

    // Extract comments: after "💬 "
    if let Some(comment_idx) = line.find("💬 ") {
        let rest = &line[comment_idx + "💬 ".len()..];
        let num_str: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        num_comments = num_str.parse().unwrap_or(0);
    }

    (author, subreddit, score, num_comments)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_post_id() {
        let url = "https://www.reddit.com/r/rust/comments/abc123/title_here/";
        assert_eq!(extract_post_id(url).unwrap(), "abc123");

        let url = "https://www.reddit.com/comments/abc123/title/";
        assert_eq!(extract_post_id(url).unwrap(), "abc123");

        let url = "https://www.reddit.com/r/programming";
        assert!(extract_post_id(url).is_none());
    }

    #[test]
    fn test_parse_stats_line() {
        let (author, sub, score, comments) =
            parse_stats_line("u/iquizuanswer on r/Piracy - ⬆️ 1380 | 💬 34");
        assert_eq!(author, "iquizuanswer");
        assert_eq!(sub, "r/Piracy");
        assert_eq!(score, 1380);
        assert_eq!(comments, 34);
    }

    #[test]
    fn test_parse_stats_line_no_comments() {
        let (author, sub, score, comments) =
            parse_stats_line("u/test on r/rust - ⬆️ 42");
        assert_eq!(author, "test");
        assert_eq!(sub, "r/rust");
        assert_eq!(score, 42);
        assert_eq!(comments, 0);
    }

    #[test]
    fn test_strip_reddit_domain() {
        assert_eq!(
            strip_reddit_domain("https://www.reddit.com/r/rust/comments/abc123/"),
            "/r/rust/comments/abc123/"
        );
        assert_eq!(
            strip_reddit_domain("https://old.reddit.com/r/rust/comments/abc123/"),
            "/r/rust/comments/abc123/"
        );
        assert_eq!(
            strip_reddit_domain("https://reddit.com/r/rust/comments/abc123/"),
            "/r/rust/comments/abc123/"
        );
    }
}
