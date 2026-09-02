use serde::Deserialize;
use thiserror::Error;
use url::Url;

mod html;

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.3";

#[derive(Debug, Error)]
pub enum TiktokError {
    #[error("HTTP request failed: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("URL parse error: {0}")]
    ShareResolveError(String),
}

pub type Result<T> = std::result::Result<T, TiktokError>;

#[derive(Debug, Clone, Deserialize)]
pub struct TiktokPost {
    pub id: String,
    pub url: String,
    pub created_at: String,
    pub content: String,
    pub account: TiktokUser,
    pub media_attachments: Vec<TiktokMedia>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TiktokUser {
    pub id: String,
    pub display_name: String,
    pub username: String,
    pub avatar: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TiktokMedia {
    pub id: String,
    #[serde(rename = "type")]
    pub media_type: String,
    pub url: String,
    pub preview_url: String,
}

pub struct TiktokClient {
    http_client: reqwest::Client,
}

impl TiktokClient {
    pub fn new() -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .user_agent(DEFAULT_USER_AGENT)
            .build()?;
        return Ok(TiktokClient::new_with_http_client(http_client));
    }

    pub fn new_with_http_client(http_client: reqwest::Client) -> Self {
        TiktokClient { http_client }
    }

    /// Resolves a TikTok share URL to the original video URL.
    pub async fn resolve_share(&self, url: &str) -> Result<String> {
        let url = Url::parse(url)
            .map_err(|_| TiktokError::ShareResolveError("Failed to parse url".into()))?;

        let slash_count = url.path().chars().filter(|&c| c == '/').count();
        let max_slashes = if url.path().ends_with('/') { 2 } else { 1 };

        if url.domain() != Some("vm.tiktok.com") || slash_count > max_slashes {
            return Ok(url.to_string());
        }

        let response = self.http_client.get(url).send().await?;
        let mut final_url = response.url().clone();
        final_url.set_query(None);
        return Ok(final_url.to_string());
    }

    pub async fn fetch_from_url(&self, url: &str) -> Result<TiktokPost> {
        let resolved_url = self.resolve_share(url).await?;
        let parsed_url = Url::parse(resolved_url.as_str())
            .map_err(|_| TiktokError::ShareResolveError("Failed to parse url".into()))?;

        let post_id = parsed_url
            .path_segments()
            .and_then(|segments| segments.last())
            .ok_or_else(|| {
                TiktokError::ShareResolveError("Could not extract post ID from URL".into())
            })?;

        let api_url = format!(
            "https://offload.tnktok.com/users/username/statuses/{}",
            post_id
        );

        let response = self.http_client.get(&api_url).send().await?;
        let mut post: TiktokPost = response.json().await?;
        post.content = html::html_to_markdown(&post.content);
        Ok(post)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn resolve_share() {
        let client = TiktokClient::new().unwrap();
        let url = "https://vm.tiktok.com/ZGdQJxgjN/";
        let resolved_url = client.resolve_share(url).await.unwrap();
        assert_eq!(
            resolved_url,
            "https://www.tiktok.com/@rijschoolehvlegend/photo/7680933567869816097"
        );
    }

    #[tokio::test]
    async fn fetch_from_url() {
        let client = TiktokClient::new().unwrap();
        let url = "https://tiktok.com/@kieth_saint/video/7680002452866747680";
        let post = client.fetch_from_url(url).await.unwrap();
        assert_eq!(post.id, "7680002452866747680");
        assert_eq!(post.account.username, "kieth_saint");
    }
}
