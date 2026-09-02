use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use tokio::sync::OnceCell;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

mod string_or_number {
    use serde::{self, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<Option<u64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de;

        struct StringOrNumberVisitor;

        impl<'de> de::Visitor<'de> for StringOrNumberVisitor {
            type Value = Option<u64>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or number that can be parsed as u64, or null")
            }

            fn visit_none<E>(self) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }

            fn visit_unit<E>(self) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(None)
            }

            fn visit_u64<E>(self, value: u64) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Some(value))
            }

            fn visit_i64<E>(self, value: i64) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Some(value as u64))
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value.is_empty() {
                    Ok(None)
                } else {
                    value.parse::<u64>().map(Some).map_err(de::Error::custom)
                }
            }
        }

        deserializer.deserialize_any(StringOrNumberVisitor)
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum InstagramError {
    #[error("HTTP request failed: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("CSRF token not found in response")]
    NoCsrfToken,

    #[error("GraphQL request failed with status {status}: {body}")]
    GraphQl { status: StatusCode, body: String },

    #[error("API returned no items for shortcode {shortcode}")]
    NoItems { shortcode: String },

    #[error("Rate limited — retries exhausted")]
    RateLimited,

    #[error("Failed to resolve share URL")]
    ShareUrlResolveFailed,

    #[error("Missing data in response")]
    MissingData,

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, InstagramError>;

// ---------------------------------------------------------------------------
// Types — raw API response (v1/iPhone format)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct RawMediaItem {
    #[serde(default, deserialize_with = "string_or_number::deserialize")]
    pub pk: Option<u64>,
    pub code: Option<String>,
    pub media_type: Option<u32>,
    pub original_width: Option<u32>,
    pub original_height: Option<u32>,
    pub video_duration: Option<f64>,
    pub image_versions2: Option<ImageVersions>,
    pub video_versions: Option<Vec<VideoVersion>>,
    pub user: Option<Owner>,
    pub caption: Option<Caption>,
    pub like_count: Option<u64>,
    pub comment_count: Option<u64>,
    pub play_count: Option<u64>,
    pub carousel_media: Option<Vec<RawMediaItem>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageVersions {
    pub candidates: Option<Vec<ImageCandidate>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ImageCandidate {
    pub url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VideoVersion {
    pub url: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Owner {
    #[serde(default, deserialize_with = "string_or_number::deserialize")]
    pub pk: Option<u64>,
    pub username: Option<String>,
    pub full_name: Option<String>,
    pub is_verified: Option<bool>,
    pub is_private: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Caption {
    pub text: Option<String>,
}

// ---------------------------------------------------------------------------
// Types — normalized / public API
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct InstagramPost {
    pub shortcode: String,
    pub media_type: MediaType,
    pub media_items: Vec<CarouselItem>,
    pub thumbnail_url: Option<String>,
    pub caption: String,
    pub owner: MediaOwner,
    pub like_count: u64,
    pub comment_count: u64,
    pub dimensions: Option<Dimensions>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaType {
    Image,
    Video,
    Sidecar,
}

impl From<u32> for MediaType {
    fn from(v: u32) -> Self {
        match v {
            1 => MediaType::Image,
            2 => MediaType::Video,
            8 => MediaType::Sidecar,
            _ => MediaType::Image,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MediaOwner {
    pub username: String,
    pub full_name: String,
    pub is_verified: bool,
    pub is_private: bool,
}

#[derive(Debug, Clone)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub enum CarouselItem {
    Image { url: String },
    Video { url: String },
}

impl CarouselItem {
    pub fn url(&self) -> &str {
        match self {
            Self::Image { url } | Self::Video { url } => url,
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DOC_ID_FETCH_MEDIA: &str = "27128499623469141";
const DOC_ID_FETCH_PLAY_COUNT: &str = "27234427476213202";

const BASE_URL: &str = "https://www.instagram.com";
const GRAPHQL_URL: &str = "https://www.instagram.com/graphql/query";

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.3";

const MAX_RETRIES: u32 = 3;

// ---------------------------------------------------------------------------
// CSRF token
// ---------------------------------------------------------------------------

/// Fetch an anonymous session from instagram.com and extract the `csrftoken`
/// cookie value.
pub async fn get_csrf_token(client: &Client) -> Result<String> {
    let resp = client
        .get(BASE_URL)
        .headers(default_headers(None))
        .send()
        .await?;

    let cookies: Vec<String> = resp
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok().map(String::from))
        .collect();
    let combined = cookies.join("; ");

    combined
        .split(';')
        .find_map(|c| {
            let c = c.trim();
            c.strip_prefix("csrftoken=").map(String::from)
        })
        .ok_or(InstagramError::NoCsrfToken)
}

// ---------------------------------------------------------------------------
// Shortcode extraction
// ---------------------------------------------------------------------------

/// Extract the shortcode from various Instagram URL formats.
pub fn extract_shortcode(url: &str) -> String {
    let parts: Vec<&str> = url.split('/').collect();
    let tags = ["p", "reel", "tv", "reels"];
    let idx = parts.iter().position(|s| tags.contains(s)).unwrap_or(0) + 1;
    parts.get(idx).unwrap_or(&"").to_string()
}

// ---------------------------------------------------------------------------
// Share URL resolution
// ---------------------------------------------------------------------------

/// Resolve an `/share/` redirect to the canonical Instagram URL.

// ---------------------------------------------------------------------------
// GraphQL helpers
// ---------------------------------------------------------------------------

fn default_headers(csrf: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://www.instagram.com/"),
    );
    if let Some(csrf) = csrf {
        headers.insert(
            "X-CSRFToken",
            HeaderValue::from_str(csrf).unwrap_or(HeaderValue::from_static("")),
        );
    }
    headers.insert("X-IG-App-ID", HeaderValue::from_static("936619743392459"));
    headers.insert(
        "X-Requested-With",
        HeaderValue::from_static("XMLHttpRequest"),
    );
    headers
}

/// Perform a GraphQL POST with automatic retry on 429 / 403.
async fn graphql_post(
    client: &Client,
    csrf: &str,
    doc_id: &str,
    variables: serde_json::Value,
) -> Result<serde_json::Value> {
    let body_str = format!(
        "variables={}&doc_id={}",
        urlencoding::encode(&variables.to_string()),
        doc_id,
    );

    let headers = default_headers(Some(csrf));

    let mut last_status = None;
    for attempt in 0..MAX_RETRIES {
        let resp = client
            .post(GRAPHQL_URL)
            .headers(headers.clone())
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body_str.clone())
            .send()
            .await?;

        let status = resp.status();
        if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::FORBIDDEN {
            last_status = Some(status);
            let wait_ms = 1000 * 2u64.pow(attempt);
            tracing::warn!(
                "GraphQL request got {} — retrying in {}ms (attempt {})",
                status,
                wait_ms,
                attempt + 1,
            );
            tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;
            continue;
        }

        if status.is_success() {
            return resp.json::<serde_json::Value>().await.map_err(Into::into);
        }

        let body = resp.text().await.unwrap_or_default();
        return Err(InstagramError::GraphQl { status, body });
    }

    // All retries exhausted
    if let Some(status) = last_status {
        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(InstagramError::RateLimited);
        }
        return Err(InstagramError::GraphQl {
            status,
            body: "retries exhausted (403)".into(),
        });
    }

    Err(InstagramError::RateLimited)
}

// ---------------------------------------------------------------------------
// Normalize helper
// ---------------------------------------------------------------------------

fn normalize_item(item: &RawMediaItem) -> InstagramPost {
    let media_type: MediaType = item.media_type.unwrap_or(1).into();
    let is_video = media_type == MediaType::Video;

    let image_url = item
        .image_versions2
        .as_ref()
        .and_then(|iv| iv.candidates.as_ref())
        .and_then(|c| c.first())
        .and_then(|c| c.url.clone());

    let video_url = item
        .video_versions
        .as_ref()
        .and_then(|v| v.first())
        .and_then(|v| v.url.clone());

    let owner = item.user.as_ref().map(|u| MediaOwner {
        username: u.username.clone().unwrap_or_default(),
        full_name: u.full_name.clone().unwrap_or_default(),
        is_verified: u.is_verified.unwrap_or(false),
        is_private: u.is_private.unwrap_or(false),
    });

    let media_items = if let Some(ref carousel) = item.carousel_media {
        carousel
            .iter()
            .map(|child| {
                let child_img = child
                    .image_versions2
                    .as_ref()
                    .and_then(|iv| iv.candidates.as_ref())
                    .and_then(|c| c.first())
                    .and_then(|c| c.url.clone());
                let child_vid = child
                    .video_versions
                    .as_ref()
                    .and_then(|v| v.first())
                    .and_then(|v| v.url.clone());
                if child.media_type == Some(2) {
                    CarouselItem::Video {
                        url: child_vid.unwrap_or_default(),
                    }
                } else {
                    CarouselItem::Image {
                        url: child_img.unwrap_or_default(),
                    }
                }
            })
            .collect()
    } else {
        vec![if is_video {
            CarouselItem::Video {
                url: video_url.unwrap_or_default(),
            }
        } else {
            CarouselItem::Image {
                url: image_url.unwrap_or_default(),
            }
        }]
    };

    let dims = match (item.original_width, item.original_height) {
        (Some(w), Some(h)) => Some(Dimensions {
            width: w,
            height: h,
        }),
        _ => None,
    };

    let thumbnail_url = if is_video {
        item.image_versions2
            .as_ref()
            .and_then(|iv| iv.candidates.as_ref())
            .and_then(|c| c.first())
            .and_then(|c| c.url.clone())
    } else {
        None
    };

    InstagramPost {
        shortcode: item.code.clone().unwrap_or_default(),
        media_type,
        media_items,
        thumbnail_url,
        caption: item
            .caption
            .as_ref()
            .and_then(|c| c.text.clone())
            .unwrap_or_default(),
        owner: owner.unwrap_or(MediaOwner {
            username: String::new(),
            full_name: String::new(),
            is_verified: false,
            is_private: false,
        }),
        like_count: item.like_count.unwrap_or(0),
        comment_count: item.comment_count.unwrap_or(0),
        dimensions: dims,
    }
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Instagram API client that manages a `reqwest::Client` with cookie store
/// and a cached CSRF token.
pub struct InstagramClient {
    http_client: Client,
    csrf: OnceCell<String>,
}

impl InstagramClient {
    /// Create a new client with an internally-built `reqwest::Client`
    /// (cookie store enabled).
    pub fn new() -> Result<Self> {
        let client = Client::builder().cookie_store(true).build()?;
        Ok(Self {
            http_client: client,
            csrf: OnceCell::new(),
        })
    }

    /// Create a new client with a provided `reqwest::Client`
    pub fn new_with_http(http_client: Client) -> Self {
        Self {
            http_client,
            csrf: OnceCell::new(),
        }
    }

    /// Fetch (or return cached) CSRF token from instagram.com.
    pub async fn get_csrf_token(&self) -> Result<&str> {
        let token = self
            .csrf
            .get_or_try_init(|| get_csrf_token(&self.http_client))
            .await?;
        Ok(token.as_str())
    }

    /// Fetch media for a post/reel/igtv by shortcode.
    pub async fn fetch_media_item(&self, shortcode: &str) -> Result<InstagramPost> {
        let csrf = self.get_csrf_token().await?;

        let variables = json!({
            "shortcode": shortcode,
            "__relay_internal__pv__PolarisAIGMMediaWebLabelEnabledrelayprovider": false,
        });

        let json = graphql_post(&self.http_client, &csrf, DOC_ID_FETCH_MEDIA, variables).await?;

        let items = json
            .pointer("/data/xdt_api__v1__media__shortcode__web_info/items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| InstagramError::NoItems {
                shortcode: shortcode.to_string(),
            })?;

        let item: RawMediaItem = serde_json::from_value(
            items
                .first()
                .ok_or_else(|| InstagramError::NoItems {
                    shortcode: shortcode.to_string(),
                })?
                .clone(),
        )?;

        let normalized = normalize_item(&item);

        Ok(normalized)
    }

    /// Fetch play count via the clips connection fallback.
    ///
    /// `user_id` can be a numeric PK (as `&str`) or username — the endpoint
    /// expects the numeric user PK though.
    pub async fn fetch_play_count(&self, user_id: &str, shortcode: &str) -> Result<Option<u64>> {
        let csrf = self.get_csrf_token().await?;

        let variables = json!({
            "data": {
                "include_feed_video": true,
                "page_size": 12,
                "target_user_id": user_id,
            }
        });

        let json =
            graphql_post(&self.http_client, &csrf, DOC_ID_FETCH_PLAY_COUNT, variables).await?;

        let edges = json
            .pointer("/data/xdt_api__v1__clips__user__connection_v2/edges")
            .and_then(|v| v.as_array());

        if let Some(edges) = edges {
            for edge in edges {
                let media_code = edge.pointer("/node/media/code").and_then(|v| v.as_str());
                if media_code == Some(shortcode) {
                    let play_count = edge
                        .pointer("/node/media/play_count")
                        .and_then(|v| v.as_u64());
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
        let resp = self
            .http_client
            .get(url)
            .header(USER_AGENT, DEFAULT_USER_AGENT)
            .send()
            .await?;

        // reqwest follows redirects by default; the final URL is the resolved one.
        Ok(resp.url().to_string())
    }

    /// Convenience: fetch media from a raw Instagram URL (handles share
    /// resolution, shortcode extraction, and fetching in one call).
    pub async fn fetch_from_url(&self, url: &str) -> Result<InstagramPost> {
        let resolved = self.resolve_share(url).await?;
        let shortcode = extract_shortcode(&resolved);
        self.fetch_media_item(&shortcode).await
    }
}
