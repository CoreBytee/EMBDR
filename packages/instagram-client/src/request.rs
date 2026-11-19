use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT};
use reqwest::{Client, StatusCode};

use crate::constants::{
    BASE_URL, DEFAULT_USER_AGENT, GRAPHQL_URL, MAX_RETRIES, PUBLIC_GRAPHQL_URL,
    PUBLIC_GRAPHQL_USER_AGENT, PUBLIC_WEB_APP_ID, PUBLIC_WEB_ASBD_ID, RATE_LIMIT_ERROR_CODE,
};
use crate::error::{InstagramError, Result};

// ---------------------------------------------------------------------------
// CSRF token
// ---------------------------------------------------------------------------

/// Fetch an anonymous session from instagram.com and extract the `csrftoken`
/// cookie value.
pub async fn get_csrf_token(client: &Client) -> Result<String> {
    let response = client
        .get(BASE_URL)
        .headers(default_headers(None))
        .send()
        .await?;

    let cookies: Vec<String> = response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok().map(String::from))
        .collect();
    let combined = cookies.join("; ");

    combined
        .split(';')
        .find_map(|cookie| {
            let cookie = cookie.trim();
            cookie.strip_prefix("csrftoken=").map(String::from)
        })
        .ok_or(InstagramError::NoCsrfToken)
}

// ---------------------------------------------------------------------------
// Page tokens
// ---------------------------------------------------------------------------

/// Markers used by Instagram's bootloader to expose the `LSD` token.
const LSD_TOKEN_MARKERS: [&str; 2] = [r#"["LSD",[],{"token":""#, r#""LSD",[],{"token":""#];

/// Markers used by Instagram's bootloader to expose the `fb_dtsg` token.
const FB_DTSG_TOKEN_MARKERS: [&str; 2] = [
    r#"["DTSGInitData",[],{"token":""#,
    r#"["DTSGInitialData",[],{"token":""#,
];

/// Extract a bootloader token from an Instagram page.
fn extract_bootloader_token(html: &str, markers: &[&str]) -> Option<String> {
    markers.iter().find_map(|marker| {
        let value_start = html.find(marker)? + marker.len();
        let value_rest = &html[value_start..];
        let value_end = value_rest.find('"')?;
        Some(value_rest[..value_end].to_string())
    })
}

/// Extract the `LSD` token from an Instagram page.
fn extract_lsd_token(html: &str) -> Option<String> {
    extract_bootloader_token(html, &LSD_TOKEN_MARKERS)
}

/// Extract the `fb_dtsg` token from an Instagram page.
fn extract_fb_dtsg_token(html: &str) -> Option<String> {
    extract_bootloader_token(html, &FB_DTSG_TOKEN_MARKERS)
}

/// Anonymous tokens embedded in an Instagram page.
///
/// The public GraphQL request carries them for parity with the web client,
/// which is how instagrapi keeps the anonymous endpoint working.
#[derive(Debug, Clone, Default)]
pub(crate) struct PageTokens {
    pub(crate) lsd_token: Option<String>,
    pub(crate) fb_dtsg_token: Option<String>,
    pub(crate) csrf_token: Option<String>,
}

/// Fetch the tokens from the post page, which also seeds the cookie store.
pub(crate) async fn fetch_page_tokens(client: &Client, referer: &str) -> PageTokens {
    let response = match client
        .get(referer)
        .header(USER_AGENT, PUBLIC_GRAPHQL_USER_AGENT)
        .header(reqwest::header::ACCEPT_LANGUAGE, "en-US,en;q=0.8")
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return PageTokens::default(),
    };

    let csrf_token = response
        .headers()
        .get_all(reqwest::header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .find_map(|cookie| cookie.trim().strip_prefix("csrftoken=").map(String::from));

    let html = response.text().await.unwrap_or_default();

    PageTokens {
        lsd_token: extract_lsd_token(&html),
        fb_dtsg_token: extract_fb_dtsg_token(&html),
        csrf_token,
    }
}

// ---------------------------------------------------------------------------
// GraphQL helpers
// ---------------------------------------------------------------------------

fn default_headers(csrf_token: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://www.instagram.com/"),
    );
    if let Some(csrf_token) = csrf_token {
        headers.insert(
            "X-CSRFToken",
            HeaderValue::from_str(csrf_token).unwrap_or(HeaderValue::from_static("")),
        );
    }
    headers.insert("X-IG-App-ID", HeaderValue::from_static(PUBLIC_WEB_APP_ID));
    headers.insert(
        "X-Requested-With",
        HeaderValue::from_static("XMLHttpRequest"),
    );
    headers
}

/// Headers used by the anonymous public web GraphQL endpoint.
fn public_graphql_headers(
    page_tokens: &PageTokens,
    referer: &str,
    friendly_name: &str,
) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(PUBLIC_GRAPHQL_USER_AGENT),
    );
    headers.insert(reqwest::header::ACCEPT, HeaderValue::from_static("*/*"));
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        HeaderValue::from_static("en-US,en;q=0.8"),
    );
    headers.insert(
        REFERER,
        HeaderValue::from_str(referer).unwrap_or(HeaderValue::from_static(BASE_URL)),
    );
    headers.insert(reqwest::header::ORIGIN, HeaderValue::from_static(BASE_URL));
    headers.insert("X-ASBD-ID", HeaderValue::from_static(PUBLIC_WEB_ASBD_ID));
    headers.insert("X-IG-App-ID", HeaderValue::from_static(PUBLIC_WEB_APP_ID));
    headers.insert(
        "X-FB-Friendly-Name",
        HeaderValue::from_str(friendly_name).unwrap_or(HeaderValue::from_static("")),
    );
    if let Some(lsd_token) = &page_tokens.lsd_token {
        headers.insert(
            "X-FB-LSD",
            HeaderValue::from_str(lsd_token).unwrap_or(HeaderValue::from_static("")),
        );
    }
    if let Some(csrf_token) = &page_tokens.csrf_token {
        headers.insert(
            "X-CSRFToken",
            HeaderValue::from_str(csrf_token).unwrap_or(HeaderValue::from_static("")),
        );
    }
    headers
}

fn encode_form(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(key, value)| format!("{}={}", key, urlencoding::encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

/// Extract a GraphQL error from a response body.
///
/// Instagram reports throttling with HTTP 200 and an `errors` array, so the
/// HTTP status alone cannot be trusted.
pub(crate) fn graphql_error_from_response(response: &serde_json::Value) -> Option<InstagramError> {
    let error = response.get("errors")?.as_array()?.first()?;
    let message = error
        .get("message")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown GraphQL error")
        .to_string();
    let code = error.get("code").and_then(serde_json::Value::as_i64);

    if code == Some(RATE_LIMIT_ERROR_CODE) || message.to_ascii_lowercase().contains("rate limit") {
        Some(InstagramError::RateLimited)
    } else {
        Some(InstagramError::GraphQlError { code, message })
    }
}

/// Perform a GraphQL POST with automatic retry on 429 / 403.
async fn post_graphql(
    client: &Client,
    url: &str,
    headers: HeaderMap,
    body: String,
) -> Result<serde_json::Value> {
    let mut last_status = None;
    for attempt in 0..MAX_RETRIES {
        let response = client
            .post(url)
            .headers(headers.clone())
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            )
            .body(body.clone())
            .send()
            .await?;

        let status = response.status();
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
            let body = response.text().await.unwrap_or_default();
            let value = match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(value) => value,
                // Instagram sometimes answers a successful status with its
                // HTML shell instead of JSON; surface the body for debugging.
                Err(_) => {
                    return Err(InstagramError::GraphQl {
                        status,
                        body: body.chars().take(500).collect(),
                    });
                }
            };

            // `data` accompanies partial errors; only fail when it is absent.
            if value.get("data").is_some_and(|data| !data.is_null()) {
                return Ok(value);
            }
            if let Some(error) = graphql_error_from_response(&value) {
                return Err(error);
            }

            return Ok(value);
        }

        let body = response.text().await.unwrap_or_default();
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

/// Query the legacy `/graphql/query` endpoint (used for the clips fallback).
pub(crate) async fn graphql_post(
    client: &Client,
    csrf_token: &str,
    doc_id: &str,
    variables: serde_json::Value,
) -> Result<serde_json::Value> {
    let variables = variables.to_string();
    let body = encode_form(&[("variables", &variables), ("doc_id", doc_id)]);
    let headers = default_headers(Some(csrf_token));

    post_graphql(client, GRAPHQL_URL, headers, body).await
}

/// Query the anonymous public web GraphQL endpoint (`/api/graphql`).
///
/// This is the endpoint Instagram's own web client uses.
pub(crate) async fn public_graphql_post(
    client: &Client,
    doc_id: &str,
    variables: serde_json::Value,
    page_tokens: &PageTokens,
    referer: &str,
    friendly_name: &str,
) -> Result<serde_json::Value> {
    let variables = variables.to_string();
    let mut fields = vec![
        ("variables", variables.as_str()),
        ("doc_id", doc_id),
        ("server_timestamps", "true"),
    ];
    if let Some(lsd_token) = &page_tokens.lsd_token {
        fields.push(("lsd", lsd_token));
    }
    if let Some(fb_dtsg_token) = &page_tokens.fb_dtsg_token {
        fields.push(("fb_dtsg", fb_dtsg_token));
    }

    let body = encode_form(&fields);
    let headers = public_graphql_headers(page_tokens, referer, friendly_name);

    post_graphql(client, PUBLIC_GRAPHQL_URL, headers, body).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_lsd_token_from_bootloader_payload() {
        let html = r#"<script>require(["Bootloader"],function(b){b.handle(["LSD",[],{"token":"AdRls42S_yYLawLab23DX7kjOQs"}])})</script>"#;
        assert_eq!(
            extract_lsd_token(html).as_deref(),
            Some("AdRls42S_yYLawLab23DX7kjOQs")
        );

        let fallback_marker = r#"{"LSD",[],{"token":"shortToken"}"#;
        assert_eq!(
            extract_lsd_token(fallback_marker).as_deref(),
            Some("shortToken")
        );

        assert_eq!(extract_lsd_token("<html>no token here</html>"), None);
    }

    #[test]
    fn extracts_fb_dtsg_token_from_bootloader_payload() {
        let html = r#"<script>require(["Bootloader"],function(b){b.handle(["DTSGInitData",[],{"token":"AdRls42S_yYLawLab23DX7kjOQs"}])})</script>"#;
        assert_eq!(
            extract_fb_dtsg_token(html).as_deref(),
            Some("AdRls42S_yYLawLab23DX7kjOQs")
        );

        let fallback_marker = r#"["DTSGInitialData",[],{"token":"shortToken"}"#;
        assert_eq!(
            extract_fb_dtsg_token(fallback_marker).as_deref(),
            Some("shortToken")
        );

        assert_eq!(extract_fb_dtsg_token("<html>no token here</html>"), None);
    }

    #[test]
    fn maps_rate_limit_body_to_rate_limited_error() {
        let payload = json!({
            "errors": [{
                "message": "Rate limit exceeded",
                "severity": "CRITICAL",
                "code": 1675004
            }],
            "extensions": {"is_final": true}
        });

        assert!(matches!(
            graphql_error_from_response(&payload),
            Some(InstagramError::RateLimited)
        ));
    }

    #[test]
    fn maps_other_graphql_errors_to_graphql_error() {
        let payload = json!({
            "errors": [{
                "message": "Something went wrong",
                "code": 1234
            }]
        });

        assert!(matches!(
            graphql_error_from_response(&payload),
            Some(InstagramError::GraphQlError {
                code: Some(1234),
                ..
            })
        ));
    }
}
