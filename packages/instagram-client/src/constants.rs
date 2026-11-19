pub(crate) const DOC_ID_FETCH_MEDIA: &str = "27830990013244856";
pub(crate) const DOC_ID_FETCH_PLAY_COUNT: &str = "27234427476213202";

/// The logged-out GraphQL query for a post by media ID.
///
/// The shortcode-based `PolarisPostRootQuery` above is gated for anonymous
/// clients (it answers with `Rate limit exceeded`), while this logged-out
/// query still returns the full media payload.
pub(crate) const DOC_ID_FETCH_LOGGED_OUT_MEDIA: &str = "27130156389949648";

/// Instagram reports throttling inside a successful (HTTP 200) GraphQL body
/// with this error code.
pub(crate) const RATE_LIMIT_ERROR_CODE: i64 = 1675004;

pub(crate) const BASE_URL: &str = "https://www.instagram.com";
pub(crate) const GRAPHQL_URL: &str = "https://www.instagram.com/graphql/query";

/// The public web GraphQL endpoint. Unlike `GRAPHQL_URL` it does not require a
/// logged-in session as long as the request carries an `lsd` token, the web
/// app id and the `PolarisPostRootQuery` friendly name.
pub(crate) const PUBLIC_GRAPHQL_URL: &str = "https://www.instagram.com/api/graphql";
pub(crate) const FRIENDLY_NAME_FETCH_MEDIA: &str = "PolarisPostRootQuery";
pub(crate) const FRIENDLY_NAME_FETCH_LOGGED_OUT_MEDIA: &str =
    "PolarisLoggedOutDesktopWWWPostRootContentQuery";

pub(crate) const PUBLIC_WEB_APP_ID: &str = "936619743392459";
pub(crate) const PUBLIC_WEB_ASBD_ID: &str = "129477";

pub(crate) const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.3";

/// Instagram answers `/api/graphql` with the JavaScript shell (a 200 text/html
/// page) instead of JSON for desktop Chrome and Firefox user agents. The Safari
/// user agent is served the GraphQL payload, which is why instagrapi uses it
/// for its public requests too.
pub(crate) const PUBLIC_GRAPHQL_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_13_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/11.1.2 Safari/605.1.15";

pub(crate) const MAX_RETRIES: u32 = 3;
