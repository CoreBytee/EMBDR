mod constants;
mod error;
mod parse;
mod request;
mod shortcode;
mod types;

pub use error::{InstagramError, Result};
pub use request::get_csrf_token;
pub use shortcode::extract_shortcode;
pub use types::{
    Caption, CarouselItem, Dimensions, ImageCandidate, ImageVersions, InstagramPost, MediaOwner,
    MediaType, Owner, RawMediaItem, VideoVersion,
};

use reqwest::Client;
use reqwest::header::USER_AGENT;
use serde_json::json;
use tokio::sync::OnceCell;

use crate::constants::{
    BASE_URL, DEFAULT_USER_AGENT, DOC_ID_FETCH_LOGGED_OUT_MEDIA, DOC_ID_FETCH_MEDIA,
    DOC_ID_FETCH_PLAY_COUNT, FRIENDLY_NAME_FETCH_LOGGED_OUT_MEDIA, FRIENDLY_NAME_FETCH_MEDIA,
};
use crate::parse::{parse_logged_out_media_response, parse_shortcode_media_response};
use crate::request::{PageTokens, fetch_page_tokens, graphql_post, public_graphql_post};
use crate::shortcode::media_id_from_shortcode;

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Instagram API client that manages a `reqwest::Client` with cookie store
/// and cached anonymous tokens.
pub struct InstagramClient {
    http_client: Client,
    csrf_token: OnceCell<String>,
    page_tokens: OnceCell<PageTokens>,
}

impl InstagramClient {
    /// Create a new client with an internally-built `reqwest::Client`
    /// (cookie store enabled).
    pub fn new() -> Result<Self> {
        let client = Client::builder().cookie_store(true).build()?;
        Ok(Self {
            http_client: client,
            csrf_token: OnceCell::new(),
            page_tokens: OnceCell::new(),
        })
    }

    /// Create a new client with a provided `reqwest::Client`
    pub fn new_with_http(http_client: Client) -> Self {
        Self {
            http_client,
            csrf_token: OnceCell::new(),
            page_tokens: OnceCell::new(),
        }
    }

    /// Fetch (or return cached) CSRF token from instagram.com.
    pub async fn get_csrf_token(&self) -> Result<&str> {
        let token = self
            .csrf_token
            .get_or_try_init(|| get_csrf_token(&self.http_client))
            .await?;
        Ok(token.as_str())
    }

    /// Fetch (or return cached) anonymous tokens from a post page.
    ///
    /// The tokens are embedded in every page's bootloader payload and are
    /// fetched once per client. They are not strictly required for the media
    /// query, so a missing token is not an error — the request is simply sent
    /// without it.
    async fn get_page_tokens(&self, referer: &str) -> &PageTokens {
        self.page_tokens
            .get_or_init(|| async { fetch_page_tokens(&self.http_client, referer).await })
            .await
    }

    /// Fetch media for a post/reel/igtv by shortcode.
    ///
    /// The logged-out media-ID query is tried first because the shortcode
    /// `PolarisPostRootQuery` is gated for anonymous clients. If it fails the
    /// shortcode query is still attempted as a fallback.
    pub async fn fetch_media_item(&self, shortcode: &str) -> Result<InstagramPost> {
        let referer = format!("{BASE_URL}/p/{shortcode}/");
        let mut page_tokens = self.get_page_tokens(&referer).await.clone();
        if page_tokens.csrf_token.is_none() {
            page_tokens.csrf_token = self.get_csrf_token().await.ok().map(String::from);
        }

        let mut logged_out_error = None;

        if let Some(media_id) = media_id_from_shortcode(shortcode) {
            let variables = json!({ "media_id": media_id.to_string() });
            match public_graphql_post(
                &self.http_client,
                DOC_ID_FETCH_LOGGED_OUT_MEDIA,
                variables,
                &page_tokens,
                &referer,
                FRIENDLY_NAME_FETCH_LOGGED_OUT_MEDIA,
            )
            .await
            {
                Ok(response) => match parse_logged_out_media_response(&response, shortcode) {
                    Ok(post) => return Ok(post),
                    Err(error) => logged_out_error = Some(error),
                },
                Err(error) => logged_out_error = Some(error),
            }
        }

        let variables = json!({
            "shortcode": shortcode,
            "__relay_internal__pv__PolarisShortDramaEnabledrelayprovider": false,
            "__relay_internal__pv__PolarisMultiCaptionCarouselEnabledrelayprovider": true,
        });

        let response = public_graphql_post(
            &self.http_client,
            DOC_ID_FETCH_MEDIA,
            variables,
            &page_tokens,
            &referer,
            FRIENDLY_NAME_FETCH_MEDIA,
        )
        .await;

        match response {
            Ok(response) => match parse_shortcode_media_response(&response, shortcode) {
                Ok(post) => Ok(post),
                Err(fallback_error) => Err(logged_out_error.unwrap_or(fallback_error)),
            },
            Err(fallback_error) => Err(logged_out_error.unwrap_or(fallback_error)),
        }
    }

    /// Fetch play count via the clips connection fallback.
    ///
    /// `user_id` can be a numeric PK (as `&str`) or username — the endpoint
    /// expects the numeric user PK though.
    pub async fn fetch_play_count(&self, user_id: &str, shortcode: &str) -> Result<Option<u64>> {
        let csrf_token = self.get_csrf_token().await?;

        let variables = json!({
            "data": {
                "include_feed_video": true,
                "page_size": 12,
                "target_user_id": user_id,
            }
        });

        let response = graphql_post(
            &self.http_client,
            csrf_token,
            DOC_ID_FETCH_PLAY_COUNT,
            variables,
        )
        .await?;

        let edges = response
            .pointer("/data/xdt_api__v1__clips__user__connection_v2/edges")
            .and_then(|value| value.as_array());

        if let Some(edges) = edges {
            for edge in edges {
                let media_code = edge
                    .pointer("/node/media/code")
                    .and_then(|value| value.as_str());
                if media_code == Some(shortcode) {
                    let play_count = edge
                        .pointer("/node/media/play_count")
                        .and_then(|value| value.as_u64());
                    return Ok(play_count);
                }
            }
        }

        Ok(None)
    }

    /// Resolve a short share URL (`/share/p/...`) to its canonical form.
    pub async fn resolve_share(&self, url: &str) -> Result<String> {
        if !url.contains("/share/") {
            return Ok(url.to_string());
        }
        let response = self
            .http_client
            .get(url)
            .header(USER_AGENT, DEFAULT_USER_AGENT)
            .send()
            .await?;

        // reqwest follows redirects by default; the final URL is the resolved one.
        Ok(response.url().to_string())
    }

    /// Convenience: fetch media from a raw Instagram URL (handles share
    /// resolution, shortcode extraction, and fetching in one call).
    pub async fn fetch_from_url(&self, url: &str) -> Result<InstagramPost> {
        let resolved = self.resolve_share(url).await?;
        let shortcode = extract_shortcode(&resolved);

        self.fetch_media_item(&shortcode).await
    }
}
