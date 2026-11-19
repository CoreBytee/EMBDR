use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstagramError {
    #[error("HTTP request failed: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("CSRF token not found in response")]
    NoCsrfToken,

    #[error("GraphQL request failed with status {status}: {body}")]
    GraphQl { status: StatusCode, body: String },

    #[error("Instagram GraphQL error (code {code:?}): {message}")]
    GraphQlError { code: Option<i64>, message: String },

    #[error("API returned no items for shortcode {shortcode}")]
    NoItems { shortcode: String },

    #[error("Rate limited by Instagram")]
    RateLimited,

    #[error("Failed to resolve share URL")]
    ShareUrlResolveFailed,

    #[error("Missing data in response")]
    MissingData,

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, InstagramError>;
