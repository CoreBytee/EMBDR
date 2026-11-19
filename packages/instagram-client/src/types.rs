use serde::Deserialize;

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
    fn from(value: u32) -> Self {
        match value {
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

#[derive(Debug, Clone, Deserialize)]
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
