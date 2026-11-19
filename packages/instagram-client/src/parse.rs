use crate::error::{InstagramError, Result};
use crate::request::graphql_error_from_response;
use crate::types::{CarouselItem, Dimensions, InstagramPost, MediaOwner, MediaType, RawMediaItem};

/// A post is only usable when every media item resolved to a URL.
fn has_complete_media(post: &InstagramPost) -> bool {
    !post.media_items.is_empty() && post.media_items.iter().all(|item| !item.url().is_empty())
}

/// Pull the media node out of the logged-out media-ID response.
fn logged_out_media_node(response: &serde_json::Value) -> Option<&serde_json::Value> {
    let media = response.pointer("/data/xig_polaris_media")?;
    if let Some(node) = media
        .get("if_not_gated_logged_out")
        .filter(|node| !node.is_null())
    {
        return Some(node);
    }
    media.get("media_type").map(|_| media)
}

/// Parse a media-ID GraphQL response into a post.
pub(crate) fn parse_logged_out_media_response(
    response: &serde_json::Value,
    shortcode: &str,
) -> Result<InstagramPost> {
    let Some(node) = logged_out_media_node(response) else {
        if let Some(error) = graphql_error_from_response(response) {
            return Err(error);
        }
        return Err(InstagramError::NoItems {
            shortcode: shortcode.to_string(),
        });
    };

    let item: RawMediaItem = serde_json::from_value(node.clone())?;
    let post = normalize_item(&item);
    if !has_complete_media(&post) {
        return Err(InstagramError::MissingData);
    }
    Ok(post)
}

/// Parse a shortcode GraphQL response into a post.
pub(crate) fn parse_shortcode_media_response(
    response: &serde_json::Value,
    shortcode: &str,
) -> Result<InstagramPost> {
    let Some(items) = response
        .pointer("/data/xdt_api__v1__media__shortcode__web_info/items")
        .and_then(|value| value.as_array())
    else {
        if let Some(error) = graphql_error_from_response(response) {
            return Err(error);
        }
        return Err(InstagramError::NoItems {
            shortcode: shortcode.to_string(),
        });
    };

    let item: RawMediaItem = serde_json::from_value(
        items
            .first()
            .ok_or_else(|| InstagramError::NoItems {
                shortcode: shortcode.to_string(),
            })?
            .clone(),
    )?;

    let post = normalize_item(&item);
    if !has_complete_media(&post) {
        return Err(InstagramError::MissingData);
    }
    Ok(post)
}

fn normalize_item(item: &RawMediaItem) -> InstagramPost {
    let media_type: MediaType = item.media_type.unwrap_or(1).into();
    let is_video = media_type == MediaType::Video;

    let image_url = item
        .image_versions2
        .as_ref()
        .and_then(|versions| versions.candidates.as_ref())
        .and_then(|candidates| candidates.first())
        .and_then(|candidate| candidate.url.clone());

    let video_url = item
        .video_versions
        .as_ref()
        .and_then(|versions| versions.first())
        .and_then(|version| version.url.clone());

    let owner = item.user.as_ref().map(|user| MediaOwner {
        username: user.username.clone().unwrap_or_default(),
        full_name: user.full_name.clone().unwrap_or_default(),
        is_verified: user.is_verified.unwrap_or(false),
        is_private: user.is_private.unwrap_or(false),
    });

    let media_items = if let Some(ref carousel) = item.carousel_media {
        carousel
            .iter()
            .map(|child| {
                let child_image_url = child
                    .image_versions2
                    .as_ref()
                    .and_then(|versions| versions.candidates.as_ref())
                    .and_then(|candidates| candidates.first())
                    .and_then(|candidate| candidate.url.clone());
                let child_video_url = child
                    .video_versions
                    .as_ref()
                    .and_then(|versions| versions.first())
                    .and_then(|version| version.url.clone());
                if child.media_type == Some(2) {
                    CarouselItem::Video {
                        url: child_video_url.unwrap_or_default(),
                    }
                } else {
                    CarouselItem::Image {
                        url: child_image_url.unwrap_or_default(),
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

    let dimensions = match (item.original_width, item.original_height) {
        (Some(width), Some(height)) => Some(Dimensions { width, height }),
        _ => None,
    };

    let thumbnail_url = if is_video {
        item.image_versions2
            .as_ref()
            .and_then(|versions| versions.candidates.as_ref())
            .and_then(|candidates| candidates.first())
            .and_then(|candidate| candidate.url.clone())
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
            .and_then(|caption| caption.text.clone())
            .unwrap_or_default(),
        owner: owner.unwrap_or(MediaOwner {
            username: String::new(),
            full_name: String::new(),
            is_verified: false,
            is_private: false,
        }),
        like_count: item.like_count.unwrap_or(0),
        comment_count: item.comment_count.unwrap_or(0),
        dimensions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_logged_out_media_payload() {
        let payload = json!({
            "data": {"xig_polaris_media": {
                "__typename": "XIGPolarisVideoMedia",
                "pk": "4033663238900512525",
                "code": "Df6c1gyN3cN",
                "if_not_gated_logged_out": {
                    "__typename": "XIGPolarisVideoMedia",
                    "pk": "4033663238900512525",
                    "code": "Df6c1gyN3cN",
                    "media_type": 2,
                    "original_width": 1080,
                    "original_height": 1920,
                    "image_versions2": {"candidates": [
                        {"url": "https://example.com/thumbnail.jpg"}
                    ]},
                    "video_versions": [
                        {"url": "https://example.com/video.mp4"}
                    ],
                    "user": {"username": "cnn", "full_name": "CNN", "is_verified": true},
                    "caption": {"text": "reel caption"},
                    "like_count": 274913,
                    "comment_count": 31520
                }
            }}
        });

        let post = parse_logged_out_media_response(&payload, "Df6c1gyN3cN").unwrap();

        assert_eq!(post.shortcode, "Df6c1gyN3cN");
        assert_eq!(post.media_type, MediaType::Video);
        assert_eq!(post.media_items[0].url(), "https://example.com/video.mp4");
        assert_eq!(
            post.thumbnail_url.as_deref(),
            Some("https://example.com/thumbnail.jpg")
        );
        assert_eq!(post.owner.username, "cnn");
        assert_eq!(post.caption, "reel caption");
        assert_eq!(post.like_count, 274913);
        assert_eq!(post.comment_count, 31520);
        assert_eq!(
            post.dimensions.as_ref().map(|d| (d.width, d.height)),
            Some((1080, 1920))
        );
    }

    #[test]
    fn parses_logged_out_carousel_payload() {
        let payload = json!({
            "data": {"xig_polaris_media": {
                "if_not_gated_logged_out": {
                    "pk": "4032525312506263120",
                    "code": "Df2aGgPtPZQ",
                    "media_type": 8,
                    "user": {"username": "natgeo"},
                    "caption": {"text": "carousel caption"},
                    "carousel_media": [
                        {
                            "media_type": 1,
                            "image_versions2": {"candidates": [
                                {"url": "https://example.com/first.jpg"}
                            ]}
                        },
                        {
                            "media_type": 2,
                            "image_versions2": {"candidates": [
                                {"url": "https://example.com/second-thumb.jpg"}
                            ]},
                            "video_versions": [
                                {"url": "https://example.com/second.mp4"}
                            ]
                        }
                    ]
                }
            }}
        });

        let post = parse_logged_out_media_response(&payload, "Df2aGgPtPZQ").unwrap();

        assert_eq!(post.media_items.len(), 2);
        assert_eq!(post.media_items[0].url(), "https://example.com/first.jpg");
        assert_eq!(post.media_items[1].url(), "https://example.com/second.mp4");
        assert!(has_complete_media(&post));
    }

    #[test]
    fn surfaces_logged_out_graphql_error() {
        let payload = json!({
            "errors": [{
                "message": "Rate limit exceeded",
                "severity": "CRITICAL",
                "code": 1675004
            }]
        });

        assert!(matches!(
            parse_logged_out_media_response(&payload, "Df6c1gyN3cN"),
            Err(InstagramError::RateLimited)
        ));
    }

    #[test]
    fn parses_public_graphql_image_payload() {
        let payload = json!({
            "data": {"xdt_api__v1__media__shortcode__web_info": {"items": [{
                "pk": "4021793451401780101",
                "code": "DfQR9Xuy5uF",
                "media_type": 1,
                "original_width": 1184,
                "original_height": 1568,
                "image_versions2": {"candidates": [
                    {"url": "https://example.com/full.jpg", "width": 1184, "height": 1568}
                ]},
                "user": {
                    "pk": "40563360963",
                    "username": "oiaflol",
                    "full_name": "Owens Memes",
                    "is_verified": false,
                    "is_private": false
                },
                "caption": {"text": "Nome completo: Derek"},
                "like_count": 184601,
                "comment_count": 267
            }]}}
        });

        let items = payload
            .pointer("/data/xdt_api__v1__media__shortcode__web_info/items")
            .and_then(|value| value.as_array())
            .unwrap();
        let item: RawMediaItem = serde_json::from_value(items[0].clone()).unwrap();
        let post = normalize_item(&item);

        assert_eq!(post.shortcode, "DfQR9Xuy5uF");
        assert_eq!(post.media_type, MediaType::Image);
        assert_eq!(post.media_items.len(), 1);
        assert_eq!(post.media_items[0].url(), "https://example.com/full.jpg");
        assert_eq!(post.caption, "Nome completo: Derek");
        assert_eq!(post.owner.username, "oiaflol");
        assert_eq!(post.like_count, 184601);
        assert_eq!(post.comment_count, 267);
        assert_eq!(
            post.dimensions.as_ref().map(|d| (d.width, d.height)),
            Some((1184, 1568))
        );
    }

    #[test]
    fn parses_public_graphql_video_payload() {
        let payload = json!({
            "data": {"xdt_api__v1__media__shortcode__web_info": {"items": [{
                "pk": "3890123456789012345",
                "code": "DTnSIf9CYU6",
                "media_type": 2,
                "product_type": "clips",
                "original_width": 720,
                "original_height": 1280,
                "image_versions2": {"candidates": [
                    {"url": "https://example.com/thumbnail.jpg", "width": 1080, "height": 1920}
                ]},
                "video_versions": [
                    {"url": "https://example.com/video.mp4", "width": 720, "height": 1280}
                ],
                "user": {"username": "oiaflol", "full_name": "Owens Memes"},
                "caption": {"text": "reel caption"},
                "like_count": 100,
                "comment_count": 5
            }]}}
        });

        let items = payload
            .pointer("/data/xdt_api__v1__media__shortcode__web_info/items")
            .and_then(|value| value.as_array())
            .unwrap();
        let item: RawMediaItem = serde_json::from_value(items[0].clone()).unwrap();
        let post = normalize_item(&item);

        assert_eq!(post.media_type, MediaType::Video);
        assert_eq!(post.media_items[0].url(), "https://example.com/video.mp4");
        assert_eq!(
            post.thumbnail_url.as_deref(),
            Some("https://example.com/thumbnail.jpg")
        );
        assert!(has_complete_media(&post));
    }

    #[test]
    fn rejects_post_without_media_urls() {
        let payload = json!({
            "data": {"xdt_api__v1__media__shortcode__web_info": {"items": [{
                "code": "DTnSIf9CYU6",
                "media_type": 2,
                "user": {"username": "oiaflol"},
                "caption": {"text": "reel caption"}
            }]}}
        });

        let items = payload
            .pointer("/data/xdt_api__v1__media__shortcode__web_info/items")
            .and_then(|value| value.as_array())
            .unwrap();
        let item: RawMediaItem = serde_json::from_value(items[0].clone()).unwrap();
        let post = normalize_item(&item);

        assert!(!has_complete_media(&post));
    }
}
