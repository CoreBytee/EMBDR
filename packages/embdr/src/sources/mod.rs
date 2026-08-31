use url::Url;

use crate::sources::instagram::InstagramSource;

mod instagram;

#[async_trait::async_trait]
pub trait Source {
    /// Returns the name of the source.
    fn name(&self) -> String;

    /// Returns the unique identifier of the source.
    fn id(&self) -> String;

    /// Returns the short identifier of the source.
    fn short_id(&self) -> String;

    /// Returns the color associated with the source.
    fn color(&self) -> u32;

    /// Returns true if the source can handle the given URL.
    fn predicate(&self, url: &Url) -> bool;

    /// Returns the media data for the given URL.
    async fn extract_media(
        &self,
        url: &Url,
    ) -> Result<MediaData, Box<dyn std::error::Error + Send + Sync>>;
}

pub fn get_sources() -> Sources {
    vec![Box::new(InstagramSource::new())]
}

pub type Sources = Vec<Box<dyn Source + Send + Sync>>;

#[derive(Debug)]
pub struct MediaData {
    pub id: String,
    pub author: MediaAuthor,
    pub description: Option<String>,
    pub items: Vec<MediaItem>,
    pub properties: Vec<MediaProperty>,
}

#[derive(Debug)]
pub struct MediaAuthor {
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct MediaItem {
    pub url: String,
}

#[derive(Debug)]
pub enum MediaProperty {
    LikeCount(u64),
    CommentCount(u64),
}

impl MediaProperty {
    pub fn emoji(&self) -> String {
        match self {
            MediaProperty::LikeCount(_) => "❤️".to_string(),
            MediaProperty::CommentCount(_) => "💬".to_string(),
        }
    }
}
