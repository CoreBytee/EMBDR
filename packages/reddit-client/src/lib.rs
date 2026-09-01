use reqwest::header::{HeaderMap, ACCEPT, USER_AGENT};
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

    #[error("Failed to parse RSS feed: {0}")]
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

const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";

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

/// Build the RSS feed URL for a Reddit post.
fn to_rss_url(url: &str) -> String {
    let trimmed = url.trim_end_matches('/');
    format!("{}/.rss", trimmed)
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

fn default_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, DEFAULT_USER_AGENT.parse().unwrap());
    headers.insert(ACCEPT, "application/atom+xml".parse().unwrap());
    headers
}

/// Reddit client that fetches post data via RSS feeds.
pub struct RedditClient {
    http_client: Client,
}

impl RedditClient {
    /// Create a new client with default settings.
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .default_headers(default_headers())
            .cookie_store(true)
            .build()?;
        Ok(Self {
            http_client: client,
        })
    }

    /// Create a new client with a provided `reqwest::Client`.
    pub fn new_with_http(http_client: Client) -> Self {
        Self { http_client }
    }

    /// Fetch post data from a Reddit URL via its RSS feed.
    pub async fn fetch_from_url(&self, url: &str) -> Result<RedditPost> {
        let rss_url = to_rss_url(url);

        let max_retries = 3;
        let mut last_status = None;

        for attempt in 0..max_retries {
            let response = self.http_client.get(&rss_url).send().await?;

            let status = response.status();
            if status.is_success() {
                let xml = response.text().await?;
                return parse_rss_feed(&xml, url);
            }

            if status == StatusCode::TOO_MANY_REQUESTS {
                last_status = Some(status);
                let wait_ms = 1000 * 2u64.pow(attempt);
                tracing::warn!(
                    "Reddit RSS request got 429 — retrying in {}ms (attempt {})",
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
// RSS parsing
// ---------------------------------------------------------------------------

fn parse_rss_feed(xml: &str, original_url: &str) -> Result<RedditPost> {
    let document = Html::parse_document(xml);

    // The first <entry> in the feed is the post itself; subsequent entries are comments.
    let entry_selector =
        Selector::parse("entry").map_err(|e| RedditError::Parse(format!("Invalid selector: {}", e)))?;

    let post_entry = document
        .select(&entry_selector)
        .next()
        .ok_or_else(|| RedditError::NoData {
            url: original_url.to_string(),
        })?;

    // Extract post ID from <id> tag (format: t3_xxxxx)
    let id = get_text_content(&document, &post_entry, "id")
        .map(|s| s.trim_start_matches("t3_").to_string())
        .unwrap_or_default();

    // Extract title
    let title = get_text_content(&document, &post_entry, "title").unwrap_or_default();

    // Extract author name from <author><name>
    let author = {
        let author_name_sel = Selector::parse("author name")
            .map_err(|e| RedditError::Parse(format!("Invalid selector: {}", e)))?;
        post_entry
            .select(&author_name_sel)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default()
            .trim_start_matches("/u/")
            .trim_start_matches("u/")
            .to_string()
    };

    // Extract permalink from <link href="...">
    let permalink = post_entry
        .attr("href")
        .or_else(|| {
            let link_sel = Selector::parse("link[rel='alternate']").ok()?;
            post_entry.select(&link_sel).next()?.attr("href")
        })
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Extract media from the HTML content of the first entry
    let media_items = extract_media_from_entry(&post_entry);

    // Extract selftext from the content HTML
    let selftext = extract_selftext_from_entry(&post_entry);

    // Count comments (entries after the first one)
    let num_comments = document.select(&entry_selector).count().saturating_sub(1) as u64;

    Ok(RedditPost {
        id,
        title,
        author,
        selftext,
        permalink,
        score: 0, // RSS doesn't include score
        num_comments,
        media_items,
    })
}

fn get_text_content(
    _document: &Html,
    element: &scraper::ElementRef,
    tag: &str,
) -> Option<String> {
    let sel = Selector::parse(tag).ok()?;
    let el = element.select(&sel).next()?;
    Some(el.text().collect::<String>())
}

fn extract_selftext_from_entry(entry: &scraper::ElementRef) -> Option<String> {
    let content_sel = Selector::parse("content").ok()?;
    let content_el = entry.select(&content_sel).next()?;
    let html_content = content_el.text().collect::<String>();

    // Parse the HTML content to extract text
    let fragment = Html::parse_fragment(&html_content);
    let md_sel = Selector::parse("div.md p").ok();
    if let Some(sel) = md_sel {
        let text: String = fragment
            .select(&sel)
            .map(|p| p.text().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n\n");
        if !text.is_empty() {
            return Some(text);
        }
    }

    // Fallback: extract text from any paragraph
    let p_sel = Selector::parse("p").ok()?;
    let text: String = fragment
        .select(&p_sel)
        .map(|p| p.text().collect::<String>())
        .collect::<Vec<_>>()
        .join("\n\n");
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn extract_media_from_entry(entry: &scraper::ElementRef) -> Vec<RedditMediaItem> {
    let mut items = Vec::new();

    let content_sel = Selector::parse("content").ok();
    if let Some(sel) = content_sel {
        if let Some(content_el) = entry.select(&sel).next() {
            let html_content = content_el.text().collect::<String>();
            let fragment = Html::parse_fragment(&html_content);

            // 1) Check for video sources first — return immediately if found
            let video_sel = Selector::parse("video source, video").ok();
            if let Some(sel) = video_sel {
                for el in fragment.select(&sel) {
                    if let Some(src) = el.attr("src") {
                        if src.contains("v.redd.it") || src.contains("video") {
                            let cleaned = src.replace("&amp;", "&");
                            items.push(RedditMediaItem::Video { url: cleaned });
                            return items;
                        }
                    }
                }
            }

            // 2) Look for direct <a href="...i.redd.it..."> links (full resolution)
            let link_sel = Selector::parse("a[href]").ok();
            if let Some(sel) = link_sel {
                for el in fragment.select(&sel) {
                    if let Some(href) = el.attr("href") {
                        let cleaned = href.replace("&amp;", "&");
                        if cleaned.contains("v.redd.it") && cleaned.contains("DASH_") {
                            if !items.iter().any(|item| item.url() == &cleaned) {
                                items.push(RedditMediaItem::Video { url: cleaned });
                            }
                        } else if (cleaned.contains("i.redd.it")
                            || cleaned.contains("i.imgur.com"))
                            && !items.iter().any(|item| item.url() == &cleaned)
                        {
                            items.push(RedditMediaItem::Image { url: cleaned });
                        }
                    }
                }
            }

            // 3) Only add preview.redd.it if we have no i.redd.it images yet
            let has_full_res = items.iter().any(|item| match item {
                RedditMediaItem::Image { url } => url.contains("i.redd.it") || url.contains("i.imgur.com"),
                _ => false,
            });
            if !has_full_res {
                let img_sel = Selector::parse("img").ok();
                if let Some(sel) = img_sel {
                    for el in fragment.select(&sel) {
                        if let Some(src) = el.attr("src") {
                            let cleaned = src.replace("&amp;", "&");
                            if (cleaned.contains("i.redd.it")
                                || cleaned.contains("preview.redd.it")
                                || cleaned.contains("i.imgur.com"))
                                && !items.iter().any(|item| item.url() == &cleaned)
                            {
                                items.push(RedditMediaItem::Image { url: cleaned });
                            }
                        }
                    }
                }
            }
        }
    }

    // 4) Fallback: <media:thumbnail> only if nothing else found
    if items.is_empty() {
        let thumb_sel = Selector::parse("media|thumbnail, thumbnail").ok();
        if let Some(sel) = thumb_sel {
            for el in entry.select(&sel) {
                if let Some(url) = el.attr("url") {
                    let cleaned = url.replace("&amp;", "&");
                    items.push(RedditMediaItem::Image { url: cleaned });
                }
            }
        }
    }

    items
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
    fn test_to_rss_url() {
        let url = "https://www.reddit.com/r/rust/comments/abc123/title/";
        assert_eq!(
            to_rss_url(url),
            "https://www.reddit.com/r/rust/comments/abc123/title/.rss"
        );

        let url = "https://old.reddit.com/r/rust/comments/abc123/title";
        assert_eq!(
            to_rss_url(url),
            "https://old.reddit.com/r/rust/comments/abc123/title/.rss"
        );
    }
}
